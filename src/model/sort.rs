//! Sorting of the file list (Alt+S).

use std::cmp::Ordering;
use std::time::SystemTime;

use super::tree::FileEntry;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortKey {
    #[default]
    Name,
    Ext,
    Date,
    Size,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sort {
    pub key: SortKey,
    pub descending: bool,
    /// In Branch/Showall: groups by directory before sorting.
    pub path_first: bool,
}

impl Sort {
    /// E.g. "Name↑", "Size↓".
    pub fn label(&self) -> String {
        let key = match self.key {
            SortKey::Name => "Name",
            SortKey::Ext => "Ext",
            SortKey::Date => "Date",
            SortKey::Size => "Size",
        };
        format!("{key}{}", if self.descending { "↓" } else { "↑" })
    }
}

/// What a file is sorted by, built once per file before sorting: the
/// lowercased name lives in one buffer shared by all files, and its first
/// 16 bytes are kept inline (2,000,000 names, each in an allocation of its
/// own, made Showall of a whole disk take seconds).
struct Key {
    /// Size, modification time in nanoseconds from 1970, or the first
    /// 8 bytes of the extension, by the sort key; 0 for Name.
    primary: i128,
    /// The first 16 bytes of the lowercased name, zero-padded, big-endian.
    head: u128,
    /// Where the lowercased name starts in the buffer.
    start: usize,
    /// Position of the file's directory in the tree.
    dir_rank: usize,
    /// Length of the lowercased name, in bytes.
    len: u32,
    /// Where the extension starts within the name (`len` if it has none).
    ext: u32,
    /// Whether `primary` holds the whole extension (Ext).
    ext_exact: bool,
}

/// The first `N` bytes of `bytes` as a big-endian number, zero-padded:
/// when two of these differ, they order like the whole byte strings.
fn head<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut out = [0u8; N];
    let n = bytes.len().min(N);
    out[..n].copy_from_slice(&bytes[..n]);
    out
}

/// Signed nanoseconds from 1970: orders like `SystemTime`.
fn nanos(time: SystemTime) -> i128 {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d.as_nanos() as i128,
        Err(e) => -(e.duration().as_nanos() as i128),
    }
}

/// Extension (without the dot); hidden files like ".bashrc" have none.
#[cfg(test)]
fn extension(name: &str) -> &str {
    &name[ext_start(name.as_bytes())..]
}

/// Where the extension of `name` starts (its length when it has none).
fn ext_start(name: &[u8]) -> usize {
    match name.iter().rposition(|&b| b == b'.') {
        Some(0) | None => name.len(),
        Some(i) => i + 1,
    }
}

impl Sort {
    /// The leading part of the order packed into one number: when two
    /// files' prefixes differ, they order as their whole keys do; when they
    /// are equal, the whole keys are compared. Each field is encoded so as
    /// to keep its order; one that had to be cut short is followed not by
    /// the name but by the end of the range its cut values fall in.
    fn prefix(&self, k: &Key) -> u128 {
        let wide = |high: u64, low: u64| (u128::from(high) << 64) | u128::from(low);
        let name = (k.head >> 64) as u64;
        let (primary, then) = match self.key {
            SortKey::Size => (k.primary as u64, name),
            // By its first 8 bytes: whole if it is no longer than that and
            // has no NUL (which the zero padding could not tell apart).
            SortKey::Ext => (k.primary as u64, if k.ext_exact { name } else { u64::MAX }),
            // Times from 1678 to 2262; the others at either end.
            _ if k.primary > i128::from(i64::MAX) => (u64::MAX, u64::MAX),
            _ if k.primary < i128::from(i64::MIN) => (0, 0),
            SortKey::Date | SortKey::Name => ((k.primary as i64 as u64) ^ (1 << 63), name),
        };
        match (self.key, self.path_first) {
            (SortKey::Name, false) => k.head,
            (SortKey::Name, true) => wide(k.dir_rank as u64, name),
            (_, false) => wide(primary, then),
            (_, true) => wide(k.dir_rank as u64, primary),
        }
    }
}

/// Sorts `items`; `entry` gives each one's file and the position of its
/// directory in the tree, which breaks ties between equal names (or comes
/// first, with `path_first`). Equal keys keep the order they came in
/// (reversed, in descending order). In descending order, directory groups (with `path_first`) stay
/// in tree order; only the contents of each group are reversed.
///
/// What moves while sorting is a 32-byte pair (prefix, position); the
/// position in `items` breaks the last tie, so the order is total and the
/// unstable sort gives what a stable one would.
pub fn sort_by<'a, T: Copy>(
    items: &mut Vec<T>,
    sort: &Sort,
    entry: impl Fn(&T) -> (&'a FileEntry, usize),
) {
    let mut names: Vec<u8> = Vec::new();
    let mut keys: Vec<Key> = Vec::with_capacity(items.len());
    for item in items.iter() {
        let (file, dir_rank) = entry(item);
        let start = names.len();
        let raw = file.name.as_encoded_bytes();
        if raw.is_ascii() {
            names.extend(raw.iter().map(u8::to_ascii_lowercase));
        } else {
            // `str::to_lowercase` (final sigma and all), as the order
            // has always been.
            names.extend_from_slice(file.name.to_string_lossy().to_lowercase().as_bytes());
        }
        let name = &names[start..];
        let ext = ext_start(name);
        let primary = match sort.key {
            SortKey::Name => 0,
            SortKey::Ext => u64::from_be_bytes(head(&name[ext..])).into(),
            SortKey::Date => nanos(file.modified.unwrap_or(SystemTime::UNIX_EPOCH)),
            SortKey::Size => file.size.into(),
        };
        let to_u32 = |n: usize| u32::try_from(n).expect("a file name is shorter than 4 GiB");
        keys.push(Key {
            primary,
            head: u128::from_be_bytes(head(name)),
            start,
            dir_rank,
            len: to_u32(name.len()),
            ext: to_u32(ext),
            ext_exact: name.len() - ext <= 8 && !name[ext..].contains(&0),
        });
    }

    let name = |k: &Key| &names[k.start..k.start + k.len as usize];
    let ext = |k: &Key| &names[k.start + k.ext as usize..k.start + k.len as usize];
    let group = |k: &Key| if sort.path_first { k.dir_rank } else { 0 };
    let whole = |a: &Key, b: &Key| {
        group(a)
            .cmp(&group(b))
            .then(a.primary.cmp(&b.primary))
            .then_with(|| match sort.key {
                SortKey::Ext => ext(a).cmp(ext(b)),
                _ => Ordering::Equal,
            })
            .then(a.head.cmp(&b.head))
            .then_with(|| name(a).cmp(name(b)))
            .then(a.dir_rank.cmp(&b.dir_rank))
    };
    let mut order: Vec<(u128, usize)> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| (sort.prefix(k), i))
        .collect();
    order.sort_unstable_by(|&(pa, a), &(pb, b)| {
        pa.cmp(&pb)
            .then_with(|| whole(&keys[a], &keys[b]))
            .then(a.cmp(&b))
    });
    if sort.descending {
        for run in order.chunk_by_mut(|&(_, a), &(_, b)| group(&keys[a]) == group(&keys[b])) {
            run.reverse();
        }
    }
    drop(keys);
    drop(names);
    let old = std::mem::take(items);
    items.extend(order.iter().map(|&(_, i)| old[i]));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::tree::FileKind;
    use proptest::prelude::*;
    use std::time::Duration;

    /// The sort as it was up to 0.9.0 (an owned key per file, a stable
    /// sort): the new one must give the very same order.
    mod reference {
        use super::super::{Sort, SortKey, extension};
        use crate::model::tree::FileEntry;
        use std::time::SystemTime;

        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        pub struct SortKeyValue {
            group: usize,
            primary: Primary,
            name: String,
            dir_rank: usize,
        }

        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        enum Primary {
            Text(String),
            Time(SystemTime),
            Size(u64),
        }

        pub fn key_for(sort: &Sort, file: &FileEntry, dir_rank: usize) -> SortKeyValue {
            let name = file.name.to_string_lossy().to_lowercase();
            let primary = match sort.key {
                SortKey::Name => Primary::Text(String::new()),
                SortKey::Ext => Primary::Text(extension(&name).to_string()),
                SortKey::Date => Primary::Time(file.modified.unwrap_or(SystemTime::UNIX_EPOCH)),
                SortKey::Size => Primary::Size(file.size),
            };
            SortKeyValue {
                group: if sort.path_first { dir_rank } else { 0 },
                primary,
                name,
                dir_rank,
            }
        }

        pub fn sort_by<T>(items: &mut Vec<T>, sort: &Sort, key: impl Fn(&T) -> SortKeyValue) {
            let mut keyed: Vec<(SortKeyValue, T)> = items.drain(..).map(|t| (key(&t), t)).collect();
            keyed.sort_by(|a, b| a.0.cmp(&b.0));
            if sort.descending {
                let mut start = 0;
                while start < keyed.len() {
                    let group = keyed[start].0.group;
                    let end = keyed[start..]
                        .iter()
                        .position(|(k, _)| k.group != group)
                        .map_or(keyed.len(), |n| start + n);
                    keyed[start..end].reverse();
                    start = end;
                }
            }
            items.extend(keyed.into_iter().map(|(_, t)| t));
        }
    }

    fn file(name: &str, size: u64, secs: u64) -> FileEntry {
        FileEntry {
            name: name.into(),
            size,
            disk: size,
            link: None,
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(secs)),
            mode: 0o644,
            kind: FileKind::File,
            tagged: false,
        }
    }

    fn sorted(files: &[(FileEntry, usize)], sort: Sort) -> Vec<String> {
        let mut items: Vec<&(FileEntry, usize)> = files.iter().collect();
        sort_by(&mut items, &sort, |&(f, rank)| (f, *rank));
        items
            .iter()
            .map(|(f, _)| f.name.to_string_lossy().into_owned())
            .collect()
    }

    fn sample() -> Vec<(FileEntry, usize)> {
        vec![
            (file("b.txt", 30, 100), 0),
            (file("A.rs", 10, 300), 0),
            (file("c.md", 20, 200), 1),
            (file("a.rs", 5, 50), 1),
        ]
    }

    #[test]
    fn sorts_by_each_key() {
        let s = |key| Sort {
            key,
            ..Sort::default()
        };
        assert_eq!(
            sorted(&sample(), s(SortKey::Name)),
            ["A.rs", "a.rs", "b.txt", "c.md"]
        );
        assert_eq!(
            sorted(&sample(), s(SortKey::Ext)),
            ["c.md", "A.rs", "a.rs", "b.txt"]
        );
        assert_eq!(
            sorted(&sample(), s(SortKey::Date)),
            ["a.rs", "b.txt", "c.md", "A.rs"]
        );
        assert_eq!(
            sorted(&sample(), s(SortKey::Size)),
            ["a.rs", "A.rs", "c.md", "b.txt"]
        );
    }

    #[test]
    fn descending_and_path_first() {
        let size_desc = Sort {
            key: SortKey::Size,
            descending: true,
            path_first: false,
        };
        assert_eq!(
            sorted(&sample(), size_desc),
            ["b.txt", "c.md", "A.rs", "a.rs"]
        );

        let by_dir = Sort {
            path_first: true,
            ..Sort::default()
        };
        assert_eq!(sorted(&sample(), by_dir), ["A.rs", "b.txt", "a.rs", "c.md"]);
        let by_dir_desc = Sort {
            descending: true,
            ..by_dir
        };
        assert_eq!(
            sorted(&sample(), by_dir_desc),
            ["b.txt", "A.rs", "c.md", "a.rs"]
        );
    }

    #[test]
    fn extensions_and_labels() {
        assert_eq!(extension(".bashrc"), "");
        assert_eq!(extension("a.tar.gz"), "gz");
        assert_eq!(extension("Makefile"), "");
        assert_eq!(Sort::default().label(), "Name↑");
    }

    /// Names that tie once lowercased, share long prefixes, have no or
    /// many dots, or are not ASCII (the final sigma lowercases by context).
    fn name() -> impl Strategy<Value = String> {
        let piece = prop::sample::select(vec![
            "a",
            "A",
            "b",
            ".",
            "..",
            "_",
            " ",
            "0",
            "file_",
            "longprefix_0123456789",
            "\u{3a3}",
            "\u{3c3}",
            "\u{3c2}",
            "\u{c9}",
            "\u{e9}",
            "\u{212a}",
            "k",
            "\u{130}",
            "i",
            "\u{df}",
            "z\u{7f}",
            "\u{1f600}",
            "\0",
            ".abcdefgh",
            ".abcdefghij",
            ".abcdefghi\u{e9}",
        ]);
        prop::collection::vec(piece, 0..6).prop_map(|p| p.concat())
    }

    fn entry() -> impl Strategy<Value = (FileEntry, usize)> {
        (
            name(),
            0u64..4,
            // Around 1970, and far enough either way to leave the range
            // the prefix holds times in.
            prop::option::of((
                prop_oneof![
                    -3i64..3,
                    // 1653: before 1678, but after 1601, where the times
                    // of Windows start.
                    Just(-10_000_000_000i64),
                    Just(-10_000_000_001i64),
                    Just(1i64 << 40),
                    Just((1i64 << 40) + 1)
                ],
                prop::sample::select(vec![0u32, 1, 999_999_999]),
            )),
            0usize..4,
        )
            .prop_map(|(name, size, time, rank)| {
                let mut f = file(&name, size, 0);
                f.modified = time.map(|(secs, nanos)| {
                    let base = if secs < 0 {
                        SystemTime::UNIX_EPOCH.checked_sub(Duration::from_secs(secs.unsigned_abs()))
                    } else {
                        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(secs as u64))
                    };
                    base.and_then(|t| t.checked_add(Duration::from_nanos(u64::from(nanos))))
                        .expect("a time every system can hold")
                });
                (f, rank)
            })
    }

    proptest! {
        /// Every sort mode gives the order the old sort gave.
        #[test]
        fn same_order_as_before(files in prop::collection::vec(entry(), 0..40)) {
            for key in [SortKey::Name, SortKey::Ext, SortKey::Date, SortKey::Size] {
                for descending in [false, true] {
                    for path_first in [false, true] {
                        let sort = Sort { key, descending, path_first };
                        let mut old: Vec<usize> = (0..files.len()).collect();
                        reference::sort_by(&mut old, &sort, |&i| {
                            reference::key_for(&sort, &files[i].0, files[i].1)
                        });
                        let mut new: Vec<usize> = (0..files.len()).collect();
                        sort_by(&mut new, &sort, |&i| (&files[i].0, files[i].1));
                        prop_assert_eq!(new, old, "{:?}", sort);
                    }
                }
            }
        }
    }

    /// The time of `sort_by` alone over a large list, against the old sort:
    /// `XY_SORT_N=2000000 cargo test --release sort_measure -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn sort_measure() {
        use std::time::Instant;
        let n: usize = std::env::var("XY_SORT_N")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1_000_000);
        const EXTS: &[&str] = &["txt", "rs", "md", "jpg", "json", "pdf", "mp3", "log"];
        let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        // In tree order, as refresh_files has them: 10 files a folder.
        let files: Vec<(FileEntry, usize)> = (0..n)
            .map(|i| {
                let ext = EXTS[(next() % EXTS.len() as u64) as usize];
                let name = format!("file_{i}.{ext}");
                (
                    file(&name, next() % 10_000_000, next() % 300_000_000),
                    i / 10,
                )
            })
            .collect();
        for key in [SortKey::Name, SortKey::Ext, SortKey::Date, SortKey::Size] {
            for path_first in [false, true] {
                let sort = Sort {
                    key,
                    descending: false,
                    path_first,
                };
                let mut items: Vec<(usize, usize)> = (0..n).map(|i| (i, files[i].1)).collect();
                let start = Instant::now();
                sort_by(&mut items, &sort, |&(i, rank)| (&files[i].0, rank));
                let new = start.elapsed();
                let mut items: Vec<(usize, usize)> = (0..n).map(|i| (i, files[i].1)).collect();
                let start = Instant::now();
                reference::sort_by(&mut items, &sort, |&(i, rank)| {
                    reference::key_for(&sort, &files[i].0, rank)
                });
                let old = start.elapsed();
                println!(
                    "{n} files, {key:?}, path_first {path_first}: {:.0} ms (before: {:.0} ms)",
                    new.as_secs_f64() * 1000.0,
                    old.as_secs_f64() * 1000.0
                );
            }
        }
    }
}
