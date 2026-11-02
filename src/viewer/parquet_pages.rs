//! What the pages of a Parquet row group will need, found out before reading it.
//!
//! The `parquet` crate decompresses a page whole. It trusts the size the page
//! declares, up to 2 GiB per page and column, and its GZIP codec reads on to the end
//! of the stream whatever was declared. So the headers are walked first, without
//! decompressing, and GZIP pages are tried with a limit. The crate keeps its page
//! headers private: the few fields needed are read here, from Thrift's compact
//! format.

use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};

use parquet::basic::Compression;
use parquet::file::metadata::{ColumnChunkMetaData, RowGroupMetaData};

use super::records::{Budget, Stoppable};

/// Nesting in a page header that is not plausible (real ones have 3 levels).
const MAX_DEPTH: u8 = 16;

const DICTIONARY_PAGE: i32 = 2;
const INDEX_PAGE: i32 = 1;

fn damaged(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("damaged page: {what}"))
}

fn byte(r: &mut dyn Read) -> io::Result<u8> {
    let mut one = [0u8];
    r.read_exact(&mut one)?;
    Ok(one[0])
}

fn varint(r: &mut dyn Read) -> io::Result<u64> {
    let mut value = 0u64;
    for shift in (0..70).step_by(7) {
        let b = byte(r)?;
        value |= u64::from(b & 0x7F) << shift;
        if b & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(damaged("number too long"))
}

fn zigzag(r: &mut dyn Read) -> io::Result<i64> {
    let n = varint(r)?;
    Ok((n >> 1) as i64 ^ -((n & 1) as i64))
}

fn skip_bytes(r: &mut dyn Read, n: u64) -> io::Result<()> {
    let copied = io::copy(&mut r.take(n), &mut io::sink())?;
    if copied < n {
        return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
    }
    Ok(())
}

/// Skips a value. Every value read uses at least one byte, so what a length
/// claims is bounded by the input.
fn skip_value(r: &mut dyn Read, kind: u8, depth: u8, in_list: bool) -> io::Result<()> {
    if depth == 0 {
        return Err(damaged("too deep"));
    }
    match kind {
        // As a field, a boolean is all in the field's type.
        1 | 2 => {
            if in_list {
                byte(r)?;
            }
        }
        3 => {
            byte(r)?;
        }
        4..=6 => {
            varint(r)?;
        }
        7 => skip_bytes(r, 8)?,
        8 => {
            let n = varint(r)?;
            skip_bytes(r, n)?;
        }
        9 | 10 => {
            let head = byte(r)?;
            let count = match head >> 4 {
                15 => varint(r)?,
                n => u64::from(n),
            };
            for _ in 0..count {
                skip_value(r, head & 0x0F, depth - 1, true)?;
            }
        }
        11 => {
            let count = varint(r)?;
            if count > 0 {
                let kinds = byte(r)?;
                for _ in 0..count {
                    skip_value(r, kinds >> 4, depth - 1, true)?;
                    skip_value(r, kinds & 0x0F, depth - 1, true)?;
                }
            }
        }
        12 => read_struct(r, depth - 1, &mut |_, _, _| Ok(false))?,
        _ => return Err(damaged("unknown type")),
    }
    Ok(())
}

/// Reads a struct, giving each field to `visit`, which says if it read the value.
fn read_struct(
    r: &mut dyn Read,
    depth: u8,
    visit: &mut dyn FnMut(i16, u8, &mut dyn Read) -> io::Result<bool>,
) -> io::Result<()> {
    if depth == 0 {
        return Err(damaged("too deep"));
    }
    let mut id = 0i16;
    loop {
        let head = byte(r)?;
        if head == 0 {
            return Ok(());
        }
        id = match head >> 4 {
            0 => zigzag(r)? as i16,
            delta => id.wrapping_add(i16::from(delta)),
        };
        let kind = head & 0x0F;
        if !visit(id, kind, r)? {
            skip_value(r, kind, depth, false)?;
        }
    }
}

fn int_field(r: &mut dyn Read) -> io::Result<i32> {
    i32::try_from(zigzag(r)?).map_err(|_| damaged("number too large"))
}

/// The fields of a page header that matter here.
#[derive(Debug, Default, PartialEq)]
struct PageHeader {
    kind: Option<i32>,
    uncompressed: Option<i32>,
    compressed: Option<i32>,
    /// Data page v2: levels come first, never compressed.
    levels: i64,
    /// Data page v2: the values may not be compressed at all.
    values_stored: bool,
}

fn read_header(r: &mut dyn Read) -> io::Result<PageHeader> {
    let mut page = PageHeader::default();
    read_struct(r, MAX_DEPTH, &mut |id, kind, r| {
        match (id, kind) {
            (1, 5) => page.kind = Some(int_field(r)?),
            (2, 5) => page.uncompressed = Some(int_field(r)?),
            (3, 5) => page.compressed = Some(int_field(r)?),
            (8, 12) => read_struct(r, MAX_DEPTH - 1, &mut |id, kind, r| {
                match (id, kind) {
                    (5 | 6, 5) => page.levels += i64::from(int_field(r)?.max(0)),
                    (7, 2) => page.values_stored = true,
                    _ => return Ok(false),
                }
                Ok(true)
            })?,
            _ => return Ok(false),
        }
        Ok(true)
    })?;
    Ok(page)
}

/// Whether a stream gives no more than `most` bytes, looking no further.
fn fits(decoder: impl Read, most: u64) -> io::Result<bool> {
    let given = io::copy(&mut decoder.take(most + 1), &mut io::sink())?;
    Ok(given <= most)
}

/// Sizes in a column chunk.
#[derive(Debug, Default, PartialEq)]
struct ChunkPages {
    dictionary: u64,
    largest: u64,
}

fn walk_chunk(
    file: &File,
    column: &ColumnChunkMetaData,
    limit: u64,
    budget: &Budget,
) -> Result<ChunkPages, io::Error> {
    let start = column
        .dictionary_page_offset()
        .unwrap_or_else(|| column.data_page_offset());
    let (Ok(start), Ok(length)) = (
        u64::try_from(start),
        u64::try_from(column.compressed_size()),
    ) else {
        return Err(damaged("negative position"));
    };
    let gzip = matches!(column.compression(), Compression::GZIP(_));

    let mut input = BufReader::new(Stoppable::new(file, budget.clone()));
    input.seek(SeekFrom::Start(start))?;
    let mut remaining = length;
    let mut pages = ChunkPages::default();
    while remaining > 0 {
        let mut limited = (&mut input).take(remaining);
        let header = read_header(&mut limited)?;
        remaining = limited.limit();

        let (Some(kind), Some(uncompressed), Some(compressed)) =
            (header.kind, header.uncompressed, header.compressed)
        else {
            return Err(damaged("incomplete header"));
        };
        let (Ok(uncompressed), Ok(compressed)) =
            (u64::try_from(uncompressed), u64::try_from(compressed))
        else {
            return Err(damaged("negative size"));
        };
        if compressed > remaining {
            return Err(damaged("larger than its column"));
        }
        remaining -= compressed;
        if kind == INDEX_PAGE {
            input.seek_relative(compressed as i64)?;
            continue;
        }
        if uncompressed > limit {
            return Err(over_limit(uncompressed, limit));
        }
        if kind == DICTIONARY_PAGE {
            pages.dictionary = pages.dictionary.saturating_add(uncompressed);
        } else {
            pages.largest = pages.largest.max(uncompressed);
        }

        // Levels are stored as they are, ahead of the compressed values.
        let levels = header.levels as u64;
        let (values, most) = (
            compressed.checked_sub(levels),
            uncompressed.checked_sub(levels),
        );
        let (Some(values), Some(most)) = (values, most) else {
            return Err(damaged("levels larger than the page"));
        };
        if !gzip || header.values_stored || values == 0 || most == 0 {
            input.seek_relative(compressed as i64)?;
            continue;
        }
        let at = input.stream_position()?;
        skip_bytes(&mut input, levels)?;
        let data = (&mut input).take(values);
        if !fits(flate2::read::MultiGzDecoder::new(data), most)? {
            return Err(damaged("gives more than it declares"));
        }
        input.seek(SeekFrom::Start(at + compressed))?;
    }
    Ok(pages)
}

fn over_limit(needed: u64, limit: u64) -> io::Error {
    let mib = |n: u64| n.div_ceil(1 << 20);
    io::Error::new(
        io::ErrorKind::OutOfMemory,
        format!(
            "Parquet pages need {} MiB at once, over the limit of {} MiB",
            mib(needed),
            mib(limit)
        ),
    )
}

/// Refuses a row group whose pages would hold more than `limit` bytes in memory at
/// once, or that lies about what it holds. Gives the size of its dictionaries.
pub fn check_row_group(
    file: &File,
    group: &RowGroupMetaData,
    wanted: impl Fn(usize) -> bool,
    limit: u64,
    budget: &Budget,
) -> io::Result<u64> {
    let (mut dictionaries, mut held) = (0u64, 0u64);
    for (i, column) in group.columns().iter().enumerate() {
        if !wanted(i) {
            continue;
        }
        let pages = walk_chunk(file, column, limit, budget)?;
        dictionaries = dictionaries.saturating_add(pages.dictionary);
        held = held
            .saturating_add(pages.dictionary)
            .saturating_add(pages.largest);
        if held > limit {
            return Err(over_limit(held, limit));
        }
    }
    Ok(dictionaries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// A compact-format field with a delta id and an i32 value.
    fn int(delta: u8, value: i32) -> Vec<u8> {
        let mut out = vec![delta << 4 | 5];
        let mut n = ((value as i64) << 1 ^ (value as i64 >> 63)) as u64;
        loop {
            let b = (n & 0x7F) as u8;
            n >>= 7;
            if n == 0 {
                out.push(b);
                return out;
            }
            out.push(b | 0x80);
        }
    }

    #[test]
    fn reads_the_sizes_and_skips_what_it_does_not_want() {
        let mut bytes = [int(1, 0), int(1, 300), int(1, -7)].concat();
        bytes.push(1 << 4 | 5); // crc
        bytes.extend([4]);
        // data_page_header: a struct with statistics, a list and a binary value.
        bytes.push(1 << 4 | 12);
        bytes.extend(int(1, 5));
        bytes.push(1 << 4 | 8);
        bytes.extend([3, b'a', b'b', b'c']);
        bytes.push(1 << 4 | 9);
        bytes.extend([2 << 4 | 5, 2, 4]);
        bytes.push(1 << 4 | 1); // a true with no value
        bytes.push(0);
        bytes.push(0);
        let page = read_header(&mut Cursor::new(&bytes)).unwrap();
        assert_eq!(page.kind, Some(0));
        assert_eq!(page.uncompressed, Some(300));
        assert_eq!(page.compressed, Some(-7));
    }

    #[test]
    fn reads_a_v2_header() {
        let mut bytes = [int(1, 3), int(1, 100), int(1, 50)].concat();
        bytes.push(5 << 4 | 12);
        bytes.extend(int(5, 4));
        bytes.extend(int(1, 6));
        bytes.push(1 << 4 | 2); // is_compressed = false
        bytes.push(0);
        bytes.push(0);
        let page = read_header(&mut Cursor::new(&bytes)).unwrap();
        assert_eq!(page.levels, 10);
        assert!(page.values_stored);
    }

    #[test]
    fn a_hostile_header_ends_in_an_error() {
        // Cut short, nested forever, a huge list of nothing, an unknown type.
        let forever = [1 << 4 | 12].repeat(1000);
        let list = vec![1 << 4 | 9, 0xF0 | 12, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        for bytes in [vec![1 << 4 | 5], forever, list, vec![1 << 4 | 13]] {
            assert!(read_header(&mut Cursor::new(&bytes)).is_err());
        }
    }
}
