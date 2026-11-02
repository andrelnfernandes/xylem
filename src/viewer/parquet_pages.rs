//! What the pages of a Parquet row group need, found out as they are read.
//!
//! The `parquet` crate decompresses a page whole. It trusts the size the page
//! declares, up to 2 GiB per page and column, and its GZIP codec reads on to the end
//! of the stream whatever was declared. So the file it reads from looks at each page
//! on the way: the header before the crate gets it, and GZIP data, with a limit,
//! before the crate decompresses it. A page never reached costs nothing. The crate
//! keeps its page headers private: the few fields needed are read here, from
//! Thrift's compact format.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::sync::Mutex;

use bytes::Bytes;
use parquet::basic::Compression;
use parquet::errors::{ParquetError, Result as ParquetResult};
use parquet::file::metadata::RowGroupMetaData;
use parquet::file::reader::{ChunkReader, Length};

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
pub(super) struct PageHeader {
    kind: Option<i32>,
    uncompressed: Option<i32>,
    pub(super) compressed: Option<i32>,
    /// Data page v2: levels come first, never compressed.
    levels: i64,
    /// Data page v2: the values may not be compressed at all.
    values_stored: bool,
}

pub(super) fn read_header(r: &mut dyn Read) -> io::Result<PageHeader> {
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

/// A column chunk, and the most its pages held so far.
struct Column {
    start: u64,
    end: u64,
    gzip: bool,
    dictionary: u64,
    largest: u64,
}

/// A page whose header was read, waiting for its data to be asked for.
struct Page {
    compressed: u64,
    uncompressed: u64,
    levels: u64,
    values_stored: bool,
    gzip: bool,
}

#[derive(Default)]
struct Seen {
    columns: Vec<Column>,
    /// By where the data starts.
    pages: HashMap<u64, Page>,
}

/// The file as the `parquet` crate reads one row group of it: refuses pages that
/// would hold more than `limit` bytes in memory at once, or that lie.
pub struct GuardedFile {
    file: File,
    limit: u64,
    budget: Budget,
    seen: Mutex<Seen>,
    dictionaries: u64,
}

impl GuardedFile {
    /// Looks at the first page of each column wanted, the one that may be a
    /// dictionary, which the size of a batch depends on.
    pub fn new(
        file: File,
        group: &RowGroupMetaData,
        wanted: impl Fn(usize) -> bool,
        limit: u64,
        budget: Budget,
    ) -> io::Result<Self> {
        let mut columns = Vec::new();
        for (i, column) in group.columns().iter().enumerate() {
            if !wanted(i) {
                continue;
            }
            let start = column
                .dictionary_page_offset()
                .unwrap_or_else(|| column.data_page_offset());
            let (Ok(start), Ok(length)) = (
                u64::try_from(start),
                u64::try_from(column.compressed_size()),
            ) else {
                return Err(damaged("negative position"));
            };
            columns.push(Column {
                start,
                end: start.saturating_add(length),
                gzip: matches!(column.compression(), Compression::GZIP(_)),
                dictionary: 0,
                largest: 0,
            });
        }
        let mut guarded = GuardedFile {
            file,
            limit,
            budget,
            seen: Mutex::new(Seen {
                columns,
                pages: HashMap::new(),
            }),
            dictionaries: 0,
        };
        let firsts: Vec<u64> = guarded
            .lock()
            .columns
            .iter()
            .filter(|column| column.end > column.start)
            .map(|column| column.start)
            .collect();
        for start in firsts {
            guarded.read_page_header(start)?;
        }
        let dictionaries = guarded
            .lock()
            .columns
            .iter()
            .fold(0, |sum: u64, column| sum.saturating_add(column.dictionary));
        guarded.dictionaries = dictionaries;
        Ok(guarded)
    }

    /// The size of the dictionaries of the columns wanted.
    pub fn dictionaries(&self) -> u64 {
        self.dictionaries
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Seen> {
        self.seen.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Reads a page's header and takes note of it. The reader goes back untouched,
    /// so that its block is not read twice.
    fn read_page_header(&self, at: u64) -> io::Result<BufReader<File>> {
        if self.budget.over() {
            return Err(io::Error::other("the reading was stopped"));
        }
        let mut reader = self
            .file
            .get_read(at)
            .map_err(|e| io::Error::other(e.to_string()))?;
        let block = reader.fill_buf()?;
        let mut rest = block;
        let (header, length) = match read_header(&mut rest) {
            Ok(header) => (header, block.len() - rest.len()),
            // Longer than a block (statistics can be).
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                let again = self
                    .file
                    .get_read(at)
                    .map_err(|e| io::Error::other(e.to_string()))?;
                let mut limited = again.take(u64::MAX);
                let header = read_header(&mut limited)?;
                (header, (u64::MAX - limited.limit()) as usize)
            }
            Err(e) => return Err(e),
        };
        self.note_page(at, length as u64, &header)?;
        Ok(reader)
    }

    fn note_page(&self, at: u64, length: u64, header: &PageHeader) -> io::Result<()> {
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
        let data = at.saturating_add(length);
        let mut seen = self.lock();
        if seen.pages.contains_key(&data) {
            return Ok(());
        }
        let Some(column) = seen
            .columns
            .iter_mut()
            .find(|column| column.start <= at && at < column.end)
        else {
            return Err(damaged("outside its column"));
        };
        if data.saturating_add(compressed) > column.end {
            return Err(damaged("larger than its column"));
        }
        if kind == INDEX_PAGE {
            return Ok(());
        }
        if uncompressed > self.limit {
            return Err(over_limit(uncompressed, self.limit));
        }
        if kind == DICTIONARY_PAGE {
            // The size of a batch counts on it: only where a dictionary belongs.
            if at != column.start {
                return Err(damaged("a dictionary after the data"));
            }
            column.dictionary = uncompressed;
        } else {
            column.largest = column.largest.max(uncompressed);
        }
        let gzip = column.gzip;
        let held = seen.columns.iter().fold(0u64, |sum, column| {
            sum.saturating_add(column.dictionary)
                .saturating_add(column.largest)
        });
        if held > self.limit {
            return Err(over_limit(held, self.limit));
        }
        seen.pages.insert(
            data,
            Page {
                compressed,
                uncompressed,
                levels: header.levels as u64,
                values_stored: header.values_stored,
                gzip,
            },
        );
        Ok(())
    }

    /// Whether what a page's GZIP data gives stays within what it declares.
    fn check_page_data(&self, page: &Page, data: &[u8]) -> io::Result<()> {
        // Levels are stored as they are, ahead of the compressed values.
        let (Some(values), Some(most)) = (
            page.compressed.checked_sub(page.levels),
            page.uncompressed.checked_sub(page.levels),
        ) else {
            return Err(damaged("levels larger than the page"));
        };
        if !page.gzip || page.values_stored || values == 0 || most == 0 {
            return Ok(());
        }
        let values = &data[page.levels as usize..];
        let input = Stoppable::new(values, self.budget.clone());
        if !fits(flate2::read::MultiGzDecoder::new(input), most)? {
            return Err(damaged("gives more than it declares"));
        }
        Ok(())
    }
}

fn refused(e: io::Error) -> ParquetError {
    ParquetError::General(e.to_string())
}

impl Length for GuardedFile {
    fn len(&self) -> u64 {
        self.file.len()
    }
}

impl ChunkReader for GuardedFile {
    type T = BufReader<File>;

    fn get_read(&self, start: u64) -> ParquetResult<Self::T> {
        self.read_page_header(start).map_err(refused)
    }

    fn get_bytes(&self, start: u64, length: usize) -> ParquetResult<Bytes> {
        let page = self.lock().pages.remove(&start);
        let Some(page) = page else {
            return Err(refused(damaged("data with no header")));
        };
        if page.compressed != length as u64 {
            return Err(refused(damaged("another size than it declares")));
        }
        let data = self.file.get_bytes(start, length)?;
        self.check_page_data(&page, &data).map_err(refused)?;
        Ok(data)
    }
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
