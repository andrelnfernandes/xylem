//! Executables, libraries and objects (docs/plans/viewer-binaries.md): ELF,
//! PE and Mach-O shown as a report, from the `object` crate. What the file
//! is, for what machine, the protections it was built with, whether it is
//! signed (never verified here), and the signs of something hidden: packed
//! sections, data after the end, sections both writable and executable.
//!
//! Nothing is run or loaded: no `ldd`, which runs the program in some cases.
//! Every count read from the file is checked by `object` against the bytes
//! there, and the lists have a limit.

use std::time::Duration;

use object::read::elf::{Dyn as _, FileHeader, ProgramHeader as _};
use object::read::macho::LoadCommandVariant;
use object::read::pe::ImageNtHeaders;
use object::{Endianness, FileKind, Object, ObjectSection, ObjectSegment, ObjectSymbol, ReadRef};

use super::doc_events::DocEvents;
use super::markdown::{self, Doc};

/// Rows of a list in the report: past them it says how many more.
const MAX_ROWS: usize = 2000;
/// Most architectures of a fat Mach-O.
const MAX_ARCHES: usize = 64;
const MAX_EVENTS: usize = 2_000_000;
/// Entropy (bits per byte) over which a section looks packed or encrypted.
const PACKED: f64 = 7.2;

/// A binary by its first bytes, never by its name.
pub fn is_binary(data: &[u8]) -> bool {
    match FileKind::parse(data) {
        Ok(FileKind::Elf32 | FileKind::Elf64 | FileKind::Pe32 | FileKind::Pe64) => true,
        Ok(FileKind::MachO32 | FileKind::MachO64) => true,
        // `cafebabe` is also a Java class: a fat file has few architectures.
        Ok(FileKind::MachOFat32 | FileKind::MachOFat64) => data
            .get(4..8)
            .map(|n| u32::from_be_bytes([n[0], n[1], n[2], n[3]]))
            .is_some_and(|n| n > 0 && n < 20),
        _ => false,
    }
}

/// Bytes of a section read to measure its entropy, at most.
const SAMPLE: u64 = 16 << 20;

/// A file read only where `object` asks, each range checked against the
/// file's size before anything is reserved, and all of them together against
/// a limit: a header that claims a huge table costs nothing.
pub struct Bounded {
    cache: object::read::ReadCache<std::fs::File>,
    len: u64,
    left: std::cell::Cell<u64>,
}

impl Bounded {
    pub fn new(file: std::fs::File, limit: u64) -> std::io::Result<Self> {
        let len = file.metadata()?.len();
        Ok(Bounded {
            cache: object::read::ReadCache::new(file),
            len,
            left: std::cell::Cell::new(limit),
        })
    }

    fn take(&self, size: u64) -> Result<(), ()> {
        let left = self.left.get();
        if size > left {
            return Err(());
        }
        self.left.set(left - size);
        Ok(())
    }
}

impl<'a> ReadRef<'a> for &'a Bounded {
    fn len(self) -> Result<u64, ()> {
        Ok(self.len)
    }

    fn read_bytes_at(self, offset: u64, size: u64) -> Result<&'a [u8], ()> {
        if offset.checked_add(size).ok_or(())? > self.len {
            return Err(());
        }
        self.take(size)?;
        (&self.cache).read_bytes_at(offset, size)
    }

    fn read_bytes_at_until(
        self,
        range: std::ops::Range<u64>,
        delimiter: u8,
    ) -> Result<&'a [u8], ()> {
        if range.end > self.len || range.start > range.end {
            return Err(());
        }
        self.take(4096)?;
        (&self.cache).read_bytes_at_until(range, delimiter)
    }
}

/// The report on the binary in `data` (all of the file), laid out for
/// `width`, or why it could not be read (the bytes are then shown).
pub fn render(
    data: &[u8],
    cut: bool,
    time: Duration,
    budget: usize,
    width: usize,
) -> Result<Doc, String> {
    render_with(data, cut, time, budget, width)
}

/// The report on a binary read from its file, up to `limit` bytes in all.
pub fn render_file(
    path: &std::path::Path,
    limit: u64,
    time: Duration,
    budget: usize,
    width: usize,
) -> Result<Doc, String> {
    let file = super::open_regular(path).map_err(|e| format!("Cannot read the file: {e}"))?;
    let bounded = Bounded::new(file, limit).map_err(|e| format!("Cannot read the file: {e}"))?;
    render_with(&bounded, false, time, budget, width)
}

fn render_with<'d, R: ReadRef<'d>>(
    data: R,
    cut: bool,
    time: Duration,
    budget: usize,
    width: usize,
) -> Result<Doc, String> {
    let mut d = DocEvents::new(MAX_EVENTS);
    match FileKind::parse(data).map_err(|_| "Not a binary this viewer can read".to_string())? {
        FileKind::MachOFat32 | FileKind::MachOFat64 => fat(&mut d, data)?,
        _ => one(&mut d, data, None)?,
    }
    if cut {
        d.paragraph("Only the start of the file was read: the report may miss what is after it.");
    }
    let was_cut = d.cut;
    let mut doc = markdown::from_events("", d.events, time, budget);
    if was_cut {
        doc.mark_cut();
    }
    doc.layout(width);
    Ok(doc)
}

/// A fat (universal) Mach-O: its architectures, then each one.
fn fat<'d, R: ReadRef<'d>>(d: &mut DocEvents, data: R) -> Result<(), String> {
    use object::read::macho::{FatArch, MachOFatFile32, MachOFatFile64};
    let arches: Vec<(String, u64, u64)> = match FileKind::parse(data) {
        Ok(FileKind::MachOFat64) => MachOFatFile64::parse(data)
            .map_err(|e| e.to_string())?
            .arches()
            .iter()
            .take(MAX_ARCHES)
            .map(|a| (format!("{:?}", a.architecture()), a.offset(), a.size()))
            .collect(),
        _ => MachOFatFile32::parse(data)
            .map_err(|e| e.to_string())?
            .arches()
            .iter()
            .take(MAX_ARCHES)
            .map(|a| {
                (
                    format!("{:?}", a.architecture()),
                    a.offset().into(),
                    a.size().into(),
                )
            })
            .collect(),
    };
    d.heading(1, "Universal binary (fat Mach-O)");
    d.field(
        "Architectures",
        &arches
            .iter()
            .map(|a| a.0.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
    for (arch, offset, size) in arches {
        let part = data.read_bytes_at(offset, size).ok();
        match part {
            Some(part) => one(d, part, Some(&arch))?,
            None => d.paragraph(&format!("{arch}: its bytes are outside the file")),
        }
    }
    Ok(())
}

/// What the report says of one protection.
fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// One binary: summary, protections, signature, signs, identity, libraries
/// and lists.
fn one<'d, R: ReadRef<'d>>(d: &mut DocEvents, data: R, arch: Option<&str>) -> Result<(), String> {
    let file = object::File::parse(data)
        .map_err(|e| format!("Not a binary this viewer can read ({e})"))?;
    let kind = FileKind::parse(data).map_err(|e| e.to_string())?;
    let format = match kind {
        FileKind::Elf32 | FileKind::Elf64 => "ELF",
        FileKind::Pe32 | FileKind::Pe64 => "PE",
        FileKind::MachO32 | FileKind::MachO64 => "Mach-O",
        _ => "binary",
    };
    let what = match file.kind() {
        object::ObjectKind::Executable => "executable",
        object::ObjectKind::Dynamic => "shared library or position-independent executable",
        object::ObjectKind::Relocatable => "object file",
        object::ObjectKind::Core => "core dump",
        _ => "binary",
    };
    let title = match arch {
        Some(a) => format!("{format} {what} ({a})"),
        None => format!("{format} {what}"),
    };
    d.heading(if arch.is_some() { 2 } else { 1 }, &title);
    d.field("Architecture", &format!("{:?}", file.architecture()));
    d.field("Bits", if file.is_64() { "64" } else { "32" });
    d.field(
        "Byte order",
        if file.is_little_endian() {
            "little endian"
        } else {
            "big endian"
        },
    );
    d.field("Entry point", &format!("{:#x}", file.entry()));

    let mut checks: Vec<(String, String, bool)> = Vec::new();
    let mut libraries: Vec<String> = Vec::new();
    let mut notes: Vec<(String, String)> = Vec::new();
    match kind {
        FileKind::Elf32 => elf::<object::elf::FileHeader32<Endianness>, R>(
            data,
            &file,
            &mut checks,
            &mut libraries,
            &mut notes,
        ),
        FileKind::Elf64 => elf::<object::elf::FileHeader64<Endianness>, R>(
            data,
            &file,
            &mut checks,
            &mut libraries,
            &mut notes,
        ),
        FileKind::Pe32 => pe::<object::pe::ImageNtHeaders32, R>(data, &mut checks, &mut notes),
        FileKind::Pe64 => pe::<object::pe::ImageNtHeaders64, R>(data, &mut checks, &mut notes),
        FileKind::MachO32 => macho::<object::macho::MachHeader32<Endianness>, R>(
            data,
            &mut checks,
            &mut libraries,
            &mut notes,
        ),
        FileKind::MachO64 => macho::<object::macho::MachHeader64<Endianness>, R>(
            data,
            &mut checks,
            &mut libraries,
            &mut notes,
        ),
        _ => {}
    }
    for (label, value) in &notes {
        d.field(label, value);
    }

    if file.kind() == object::ObjectKind::Core {
        d.paragraph("A core dump: only its summary and segments are shown.");
        segments_table(d, &file);
        return Ok(());
    }

    if !checks.is_empty() {
        d.heading(3, "Protections");
        let rows: Vec<Vec<String>> = checks
            .iter()
            .map(|(name, value, bad)| {
                vec![
                    name.clone(),
                    if *bad {
                        format!("{value} ⚠")
                    } else {
                        value.clone()
                    },
                ]
            })
            .collect();
        d.table(&["Protection".into(), "".into()], &rows);
    }

    // The signs of something hidden.
    let mut signs: Vec<String> = Vec::new();
    let mut end_of_sections = 0u64;
    let mut sections: Vec<Vec<String>> = Vec::new();
    let mut entry_in_exec = false;
    let exec_flags = executable_sections(&file);
    for (i, s) in file.sections().enumerate() {
        let name = s
            .name()
            .map(super::patch::shown)
            .unwrap_or_else(|_| "?".into());
        let range = s.file_range();
        let (offset, fsize) = range.unwrap_or((0, 0));
        end_of_sections = end_of_sections.max(offset.saturating_add(fsize));
        let (w, x) = exec_flags.get(i).copied().unwrap_or((false, false));
        if x && file.entry() >= s.address() && file.entry() < s.address().saturating_add(s.size()) {
            entry_in_exec = true;
        }
        let entropy = range
            .and_then(|(o, n)| data.read_bytes_at(o, n.min(SAMPLE)).ok())
            .filter(|b| b.len() >= 256)
            .map(entropy_of);
        if let Some(e) = entropy
            && e > PACKED
        {
            signs.push(format!(
                "section {name}: entropy {e:.2}, packed or encrypted"
            ));
        }
        let lower = name.to_ascii_lowercase();
        if lower.starts_with("upx")
            || lower.contains("aspack")
            || lower == ".petite"
            || lower.starts_with(".themida")
        {
            signs.push(format!("section {name}: the name of a packer"));
        }
        if w && x {
            signs.push(format!("section {name}: writable and executable"));
        }
        if sections.len() < MAX_ROWS {
            sections.push(vec![
                name,
                format!("{:#x}", s.address()),
                s.size().to_string(),
                range.map_or("-".into(), |(_, n)| n.to_string()),
                range.map_or("-".into(), |(o, _)| format!("{o:#x}")),
                format!(
                    "{}{}{}",
                    "r",
                    if w { "w" } else { "-" },
                    if x { "x" } else { "-" }
                ),
                entropy.map_or("-".into(), |e| format!("{e:.2}")),
            ]);
        }
    }
    for seg in file.segments() {
        let (o, n) = seg.file_range();
        end_of_sections = end_of_sections.max(o.saturating_add(n));
    }
    let total = data.len().unwrap_or(0);
    if end_of_sections > 0
        && total > end_of_sections
        && matches!(
            kind,
            FileKind::Pe32 | FileKind::Pe64 | FileKind::Elf32 | FileKind::Elf64
        )
    {
        signs.push(format!(
            "{} bytes after the last section, from {:#x} (an overlay: installers and some malware keep data there)",
            total - end_of_sections,
            end_of_sections
        ));
    }
    if file.entry() != 0
        && !entry_in_exec
        && !exec_flags.is_empty()
        && file.kind() != object::ObjectKind::Relocatable
    {
        signs.push("the entry point is outside every executable section".into());
    }
    if !signs.is_empty() {
        d.heading(3, "Signs");
        d.start_list(false);
        for s in &signs {
            d.item(s);
        }
        d.end_list(false);
    }

    // Identity.
    let mut ids: Vec<(String, String)> = Vec::new();
    if let Ok(Some(id)) = file.build_id() {
        ids.push(("Build ID".into(), hex(id)));
    }
    if let Ok(Some((name, crc))) = file.gnu_debuglink() {
        ids.push((
            "Debug link".into(),
            format!(
                "{} (crc {crc:#010x})",
                super::patch::shown(&String::from_utf8_lossy(name))
            ),
        ));
    }
    if let Ok(Some(uuid)) = file.mach_uuid() {
        ids.push(("UUID".into(), hex(&uuid)));
    }
    if let Ok(Some(pdb)) = file.pdb_info() {
        ids.push((
            "PDB".into(),
            format!(
                "{} (age {})",
                super::patch::shown(&String::from_utf8_lossy(pdb.path())),
                pdb.age()
            ),
        ));
    }
    if let Some(c) = file.section_by_name(".comment").and_then(|s| s.data().ok()) {
        let text: Vec<String> = c
            .split(|&b| b == 0)
            .filter(|p| !p.is_empty())
            .take(8)
            .map(|p| super::patch::shown(&String::from_utf8_lossy(&p[..p.len().min(200)])))
            .collect();
        if !text.is_empty() {
            ids.push(("Compiler".into(), text.join("; ")));
        }
    }
    if let Some(v) = go_version(&file) {
        ids.push(("Go".into(), v));
    }
    d.field("Debug symbols", yes(file.has_debug_symbols()));
    if !ids.is_empty() {
        d.heading(3, "Identity");
        for (k, v) in &ids {
            d.field(k, v);
        }
    }

    // Libraries and imports.
    let mut imports: Vec<Vec<String>> = Vec::new();
    let mut by_library: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    if let Ok(list) = file.imports() {
        for imp in list {
            let lib = super::patch::shown(&String::from_utf8_lossy(imp.library()));
            *by_library.entry(lib.clone()).or_default() += 1;
            if imports.len() < MAX_ROWS {
                imports.push(vec![
                    lib,
                    super::patch::shown(&String::from_utf8_lossy(imp.name())),
                ]);
            }
        }
    }
    for lib in &libraries {
        by_library.entry(lib.clone()).or_default();
    }
    if !by_library.is_empty() {
        d.heading(3, "Libraries");
        d.start_list(false);
        for (lib, n) in by_library.iter().take(MAX_ROWS) {
            if lib.is_empty() {
                continue;
            }
            d.item(&if *n > 0 {
                format!("{lib} ({n} symbols)")
            } else {
                lib.clone()
            });
        }
        d.end_list(false);
    }

    d.heading(3, "Sections");
    d.table(
        &[
            "Name", "Address", "Size", "In file", "Offset", "Access", "Entropy",
        ]
        .map(String::from),
        &sections,
    );
    if file.sections().count() > MAX_ROWS {
        d.paragraph(&format!("… {} sections in all", file.sections().count()));
    }
    segments_table(d, &file);

    if !imports.is_empty() {
        d.heading(3, "Imports");
        d.table(&["Library".into(), "Symbol".into()], &imports);
        let n: usize = by_library.values().sum();
        if n > imports.len() {
            d.paragraph(&format!("… {n} imports in all"));
        }
    }
    if let Ok(list) = file.exports() {
        let all: Vec<_> = list;
        if !all.is_empty() {
            let rows: Vec<Vec<String>> = all
                .iter()
                .take(MAX_ROWS)
                .map(|e| {
                    vec![
                        super::patch::shown(&String::from_utf8_lossy(e.name())),
                        format!("{:#x}", e.address()),
                    ]
                })
                .collect();
            d.heading(3, "Exports");
            d.table(&["Symbol".into(), "Address".into()], &rows);
            if all.len() > MAX_ROWS {
                d.paragraph(&format!("… {} exports in all", all.len()));
            }
        }
    }
    let mut symbols: Vec<Vec<String>> = Vec::new();
    let mut count = 0usize;
    for s in file.symbols() {
        count += 1;
        if symbols.len() < MAX_ROWS {
            let name = s
                .name()
                .map(super::patch::shown)
                .unwrap_or_else(|_| "?".into());
            if name.is_empty() {
                continue;
            }
            symbols.push(vec![
                name,
                format!("{:#x}", s.address()),
                s.size().to_string(),
                format!("{:?}", s.kind()),
                if s.is_global() {
                    "global".into()
                } else {
                    "local".into()
                },
            ]);
        }
    }
    if !symbols.is_empty() {
        d.heading(3, "Symbols");
        d.table(
            &["Name", "Address", "Size", "Kind", "Scope"].map(String::from),
            &symbols,
        );
        if count > MAX_ROWS {
            d.paragraph(&format!("… {count} symbols in all"));
        }
    }
    Ok(())
}

fn segments_table<'d, R: ReadRef<'d>>(d: &mut DocEvents, file: &object::File<'d, R>) {
    let rows: Vec<Vec<String>> = file
        .segments()
        .take(MAX_ROWS)
        .map(|s| {
            let (o, n) = s.file_range();
            vec![
                s.name()
                    .ok()
                    .flatten()
                    .map(super::patch::shown)
                    .unwrap_or_default(),
                format!("{:#x}", s.address()),
                s.size().to_string(),
                format!("{o:#x}"),
                n.to_string(),
                perms(s.permissions()),
            ]
        })
        .collect();
    if !rows.is_empty() {
        d.heading(3, "Segments");
        d.table(
            &["Name", "Address", "Size", "Offset", "In file", "Access"].map(String::from),
            &rows,
        );
    }
}

fn perms(p: object::Permissions) -> String {
    format!(
        "{}{}{}",
        if p.readable() { "r" } else { "-" },
        if p.writable() { "w" } else { "-" },
        if p.executable() { "x" } else { "-" }
    )
}

/// For each section, in order: writable and executable.
fn executable_sections<'d, R: ReadRef<'d>>(file: &object::File<'d, R>) -> Vec<(bool, bool)> {
    file.sections()
        .map(|s| match s.flags() {
            object::SectionFlags::Elf { sh_flags } => (
                sh_flags & u64::from(object::elf::SHF_WRITE) != 0,
                sh_flags & u64::from(object::elf::SHF_EXECINSTR) != 0,
            ),
            object::SectionFlags::Coff { characteristics } => (
                characteristics & object::pe::IMAGE_SCN_MEM_WRITE != 0,
                characteristics & object::pe::IMAGE_SCN_MEM_EXECUTE != 0,
            ),
            object::SectionFlags::MachO { flags } => (
                false,
                flags
                    & (object::macho::S_ATTR_PURE_INSTRUCTIONS
                        | object::macho::S_ATTR_SOME_INSTRUCTIONS)
                    != 0,
            ),
            _ => (false, s.kind() == object::SectionKind::Text),
        })
        .collect()
}

/// Shannon entropy of the bytes, in bits per byte.
fn entropy_of(data: &[u8]) -> f64 {
    let mut counts = [0usize; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let n = data.len() as f64;
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / n;
            -p * p.log2()
        })
        .sum()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().take(64).map(|b| format!("{b:02x}")).collect()
}

/// The Go version a binary was built with, from its `.go.buildinfo`.
fn go_version<'d, R: ReadRef<'d>>(file: &object::File<'d, R>) -> Option<String> {
    let data = file
        .section_by_name(".go.buildinfo")
        .or_else(|| file.section_by_name("__go_buildinfo"))?
        .data()
        .ok()?;
    let at = data.windows(4).position(|w| w == b"go1.")?;
    let v: String = data[at..]
        .iter()
        .take(32)
        .take_while(|b| b.is_ascii_graphic())
        .map(|&b| b as char)
        .collect();
    Some(v)
}

/// ELF: PIE, NX, RELRO, canary, FORTIFY, RPATH and RUNPATH, interpreter and
/// libraries.
fn elf<'d, H: FileHeader<Endian = Endianness>, R: ReadRef<'d>>(
    data: R,
    file: &object::File<'d, R>,
    checks: &mut Vec<(String, String, bool)>,
    libraries: &mut Vec<String>,
    notes: &mut Vec<(String, String)>,
) {
    let Ok(header) = H::parse(data) else { return };
    let Ok(endian) = header.endian() else { return };
    let Ok(phdrs) = header.program_headers(endian, data) else {
        return;
    };
    let e_type = header.e_type(endian);
    let mut nx = None;
    let mut relro = false;
    let mut bind_now = false;
    let mut pie_flag = false;
    let mut rpath: Vec<String> = Vec::new();
    let dynstr = file.section_by_name(".dynstr").and_then(|s| s.data().ok());
    let string_at = |off: u64| -> Option<String> {
        let strs = dynstr?;
        let off = usize::try_from(off).ok()?;
        let rest = strs.get(off..)?;
        let end = rest
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(rest.len())
            .min(4096);
        Some(super::patch::shown(&String::from_utf8_lossy(&rest[..end])))
    };
    for ph in phdrs.iter().take(4096) {
        match ph.p_type(endian) {
            object::elf::PT_GNU_STACK => {
                nx = Some(ph.p_flags(endian) & object::elf::PF_X == 0);
            }
            object::elf::PT_GNU_RELRO => relro = true,
            object::elf::PT_INTERP => {
                if let Ok(bytes) = ph.data(endian, data) {
                    let end = bytes
                        .iter()
                        .position(|&b| b == 0)
                        .unwrap_or(bytes.len())
                        .min(4096);
                    notes.push((
                        "Interpreter".into(),
                        super::patch::shown(&String::from_utf8_lossy(&bytes[..end])),
                    ));
                }
            }
            object::elf::PT_DYNAMIC => {
                if let Ok(Some(dyns)) = ph.dynamic(endian, data) {
                    for d in dyns.iter().take(100_000) {
                        let tag: i64 = d.d_tag(endian).into();
                        let val: u64 = d.d_val(endian).into();
                        match tag {
                            object::elf::DT_NEEDED => {
                                if let Some(s) = string_at(val) {
                                    libraries.push(s);
                                }
                            }
                            object::elf::DT_RPATH | object::elf::DT_RUNPATH => {
                                if let Some(s) = string_at(val) {
                                    let which = if tag == object::elf::DT_RPATH {
                                        "RPATH"
                                    } else {
                                        "RUNPATH"
                                    };
                                    rpath.push(format!("{which} {s}"));
                                }
                            }
                            object::elf::DT_BIND_NOW => bind_now = true,
                            object::elf::DT_FLAGS => {
                                bind_now |= val & u64::from(object::elf::DF_BIND_NOW) != 0
                            }
                            object::elf::DT_FLAGS_1 => {
                                bind_now |= val & u64::from(object::elf::DF_1_NOW) != 0;
                                pie_flag |= val & u64::from(object::elf::DF_1_PIE) != 0;
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    let pie = e_type == object::elf::ET_DYN
        && (pie_flag || notes.iter().any(|(k, _)| k == "Interpreter"));
    if e_type == object::elf::ET_EXEC || pie {
        checks.push(("PIE".into(), yes(pie).into(), !pie));
    }
    checks.push((
        "NX (stack not executable)".into(),
        nx.map_or("unknown".into(), |v| yes(v).to_string()),
        nx == Some(false),
    ));
    let relro_text = match (relro, bind_now) {
        (true, true) => "full",
        (true, false) => "partial",
        _ => "no",
    };
    checks.push(("RELRO".into(), relro_text.into(), !relro));
    let imported: Vec<String> = file
        .dynamic_symbols()
        .filter(|s| s.is_undefined())
        .take(1_000_000)
        .filter_map(|s| s.name().ok().map(str::to_string))
        .collect();
    let canary = imported
        .iter()
        .any(|n| n == "__stack_chk_fail" || n == "__stack_chk_guard");
    checks.push(("Stack canary".into(), yes(canary).into(), !canary));
    let fortify = imported
        .iter()
        .any(|n| n.starts_with("__") && n.ends_with("_chk") && n != "__stack_chk_fail");
    checks.push(("FORTIFY".into(), yes(fortify).into(), false));
    for r in rpath {
        let risky = r.contains("$ORIGIN/..")
            || r.split([' ', ':'])
                .any(|p| p == "." || p.is_empty() || p.starts_with("/tmp"));
        checks.push(("Library path".into(), r, risky));
    }
}

/// PE: ASLR, DEP, CFG, SafeSEH, signature, .NET, subsystem and date.
fn pe<'d, H: ImageNtHeaders, R: ReadRef<'d>>(
    data: R,
    checks: &mut Vec<(String, String, bool)>,
    notes: &mut Vec<(String, String)>,
) {
    use object::LittleEndian as LE;
    use object::read::pe::{ImageOptionalHeader, PeFile};
    let Ok(file) = PeFile::<H, R>::parse(data) else {
        return;
    };
    let nt = file.nt_headers();
    let opt = nt.optional_header();
    let dll = opt.dll_characteristics();
    let has = |flag: u16| dll & flag != 0;
    use object::pe::*;
    checks.push((
        "ASLR (dynamic base)".into(),
        yes(has(IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE)).into(),
        !has(IMAGE_DLLCHARACTERISTICS_DYNAMIC_BASE),
    ));
    if file.is_64() {
        checks.push((
            "High-entropy ASLR".into(),
            yes(has(IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA)).into(),
            !has(IMAGE_DLLCHARACTERISTICS_HIGH_ENTROPY_VA),
        ));
    }
    checks.push((
        "DEP (NX compatible)".into(),
        yes(has(IMAGE_DLLCHARACTERISTICS_NX_COMPAT)).into(),
        !has(IMAGE_DLLCHARACTERISTICS_NX_COMPAT),
    ));
    checks.push((
        "Control Flow Guard".into(),
        yes(has(IMAGE_DLLCHARACTERISTICS_GUARD_CF)).into(),
        false,
    ));
    checks.push((
        "Force integrity".into(),
        yes(has(IMAGE_DLLCHARACTERISTICS_FORCE_INTEGRITY)).into(),
        false,
    ));
    if !file.is_64() {
        let no_seh = has(IMAGE_DLLCHARACTERISTICS_NO_SEH);
        checks.push(("No SEH".into(), yes(no_seh).into(), false));
    }
    let dirs = file.data_directories();
    let security = dirs
        .get(IMAGE_DIRECTORY_ENTRY_SECURITY)
        .map(|d| d.size.get(LE))
        .unwrap_or(0);
    notes.push((
        "Signature".into(),
        if security > 0 {
            format!("present ({security} bytes), not verified here")
        } else {
            "none".into()
        },
    ));
    if dirs
        .get(IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR)
        .is_some_and(|d| d.size.get(LE) > 0)
    {
        notes.push((".NET".into(), "yes (managed code)".into()));
    }
    let subsystem = match opt.subsystem() {
        IMAGE_SUBSYSTEM_WINDOWS_GUI => "Windows GUI".to_string(),
        IMAGE_SUBSYSTEM_WINDOWS_CUI => "Windows console".to_string(),
        IMAGE_SUBSYSTEM_NATIVE => "native (a driver)".to_string(),
        IMAGE_SUBSYSTEM_EFI_APPLICATION => "EFI application".to_string(),
        n => format!("{n}"),
    };
    notes.push(("Subsystem".into(), subsystem));
    let stamp = nt.file_header().time_date_stamp.get(LE);
    let date = chrono::DateTime::from_timestamp(i64::from(stamp), 0).map_or_else(
        || stamp.to_string(),
        |d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    );
    notes.push((
        "Build date".into(),
        format!("{date} (written by the linker; it can be anything)"),
    ));
}

/// Mach-O: PIE, heap and stack, signature, libraries, minimum system.
fn macho<'d, H: object::read::macho::MachHeader<Endian = Endianness>, R: ReadRef<'d>>(
    data: R,
    checks: &mut Vec<(String, String, bool)>,
    libraries: &mut Vec<String>,
    notes: &mut Vec<(String, String)>,
) {
    use object::macho::*;
    let Ok(header) = H::parse(data, 0) else {
        return;
    };
    let Ok(endian) = header.endian() else { return };
    let flags = header.flags(endian);
    if header.filetype(endian) == MH_EXECUTE {
        checks.push((
            "PIE".into(),
            yes(flags & MH_PIE != 0).into(),
            flags & MH_PIE == 0,
        ));
    }
    checks.push((
        "Stack executable".into(),
        yes(flags & MH_ALLOW_STACK_EXECUTION != 0).into(),
        flags & MH_ALLOW_STACK_EXECUTION != 0,
    ));
    let mut signed = false;
    if let Ok(mut cmds) = header.load_commands(endian, data, 0) {
        let mut n = 0;
        while let Ok(Some(cmd)) = cmds.next() {
            n += 1;
            if n > 100_000 {
                break;
            }
            match cmd.variant() {
                Ok(LoadCommandVariant::LinkeditData(l))
                    if l.cmd.get(endian) == LC_CODE_SIGNATURE =>
                {
                    signed = true
                }
                Ok(LoadCommandVariant::Dylib(d)) => {
                    if let Ok(name) = cmd.string(endian, d.dylib.name) {
                        libraries.push(super::patch::shown(&String::from_utf8_lossy(name)));
                    }
                }
                Ok(LoadCommandVariant::BuildVersion(b)) => {
                    let v = b.minos.get(endian);
                    notes.push((
                        "Minimum system".into(),
                        format!("{}.{}.{}", v >> 16, (v >> 8) & 0xff, v & 0xff),
                    ));
                }
                _ => {}
            }
        }
    }
    notes.push((
        "Signature".into(),
        if signed {
            "present, not verified here".into()
        } else {
            "none".into()
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(data: &[u8]) -> String {
        let doc = render(data, false, Duration::from_secs(5), 1 << 24, 200).unwrap();
        (0..doc.line_count())
            .map(|i| doc.line(i).to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn the_running_test_binary_has_a_report() {
        let me = std::fs::read(std::env::current_exe().unwrap()).unwrap();
        assert!(is_binary(&me));
        let r = report(&me);
        for want in ["Architecture", "Entry point", "Sections"] {
            assert!(r.contains(want), "{want} in\n{}", &r[..r.len().min(3000)]);
        }
        #[cfg(target_os = "linux")]
        for want in ["ELF", "RELRO", "NX (stack not executable)", "Interpreter"] {
            assert!(r.contains(want), "{want} in\n{}", &r[..r.len().min(3000)]);
        }
    }

    #[test]
    fn what_is_not_a_binary() {
        assert!(!is_binary(b"hello"));
        assert!(!is_binary(b"\x7fELF"));
        // A Java class starts like a fat Mach-O, with a big version number.
        assert!(!is_binary(&[
            0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 52, 0, 0, 0, 0
        ]));
        assert!(
            render(
                b"\x7fELF\x02\x01\x01",
                false,
                Duration::from_secs(1),
                1 << 20,
                80
            )
            .is_err()
        );
    }

    #[test]
    fn a_bounded_file_reads_nothing_past_its_end_or_its_limit() {
        let tmp = crate::testing::tempdir();
        let path = tmp.path().join("f");
        std::fs::write(&path, [7u8; 100]).unwrap();
        let b = Bounded::new(std::fs::File::open(&path).unwrap(), 60).unwrap();
        assert_eq!((&b).len(), Ok(100));
        assert!((&b).read_bytes_at(90, 20).is_err(), "past the end");
        assert!((&b).read_bytes_at(u64::MAX, 2).is_err(), "an overflow");
        assert_eq!((&b).read_bytes_at(0, 50).unwrap().len(), 50);
        assert!((&b).read_bytes_at(50, 20).is_err(), "past the limit in all");
        // A file that is not a binary, from its file.
        assert!(render_file(&path, 1 << 20, Duration::from_secs(1), 1 << 20, 80).is_err());
    }

    #[test]
    fn entropy() {
        assert!(entropy_of(&[0; 1000]) < 0.01);
        let all: Vec<u8> = (0..=255).cycle().take(4096).collect();
        assert!((entropy_of(&all) - 8.0).abs() < 0.01);
    }

    proptest::proptest! {
        #[test]
        fn hostile_binaries_never_panic(
            magic in 0usize..5,
            body in proptest::collection::vec(proptest::num::u8::ANY, 0..4096),
        ) {
            let start: &[u8] = [
                &b"\x7fELF\x02\x01\x01\0"[..],
                &b"\x7fELF\x01\x02\x01\0"[..],
                &b"MZ"[..],
                &[0xcf, 0xfa, 0xed, 0xfe][..],
                &[0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 2][..],
            ][magic];
            let mut data = start.to_vec();
            data.extend(&body);
            let _ = render(&data, false, Duration::from_secs(2), 1 << 22, 80);
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(32))]
        #[test]
        fn a_real_binary_with_bytes_changed_never_panics(
            changes in proptest::collection::vec((0usize..8192, proptest::num::u8::ANY), 1..40),
        ) {
            static ME: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
            let mut me = ME
                .get_or_init(|| {
                    let mut me = std::fs::read(std::env::current_exe().unwrap()).unwrap();
                    me.truncate(256 << 10);
                    me
                })
                .clone();
            for (at, b) in changes {
                if let Some(x) = me.get_mut(at) {
                    *x = b;
                }
            }
            let _ = render(&me, true, Duration::from_secs(2), 1 << 22, 80);
        }
    }
}
