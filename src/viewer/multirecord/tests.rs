//! The engine over synthetic files, with made-up data.

use super::catalog::{self, CATALOG};
use super::*;
use proptest::prelude::*;

/// Keeps all a reading gives, shown as the grid would.
#[derive(Default, Debug)]
struct Seen {
    sheets: Vec<String>,
    current: usize,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    structure: Option<Structure>,
    notices: Vec<String>,
    domains: Vec<Vec<(String, String)>>,
    limit: Option<usize>,
}

impl RecordSink for Seen {
    fn sheets(&mut self, names: &[String], current: usize) {
        self.sheets = names.to_vec();
        self.current = current;
    }
    fn columns(&mut self, names: &[String]) {
        self.columns = names.to_vec();
    }
    fn columns_cut(&mut self) {}
    fn structure(&mut self, structure: Structure) {
        self.structure = Some(structure);
    }
    fn domains(&mut self, domains: &[Vec<(String, String)>]) {
        self.domains = domains.to_vec();
    }
    fn notice(&mut self, text: &str) {
        self.notices.push(text.to_string());
    }
    fn record(&mut self, cells: &[Cell<'_>]) -> ControlFlow<()> {
        if self.limit.is_some_and(|l| self.rows.len() >= l) {
            return ControlFlow::Break(());
        }
        self.rows.push(cells.iter().map(Cell::display).collect());
        ControlFlow::Continue(())
    }
}

fn catalog_layout(path: &str) -> Layout {
    let text = CATALOG.iter().find(|(p, _)| *p == path).unwrap().1;
    xlayout::parse(text).unwrap()
}

fn cnab() -> Layout {
    catalog_layout("br/bancario/cnab240-febraban.xlayout.toml")
}

fn sped() -> Layout {
    catalog_layout("br/fiscal/sped-generico.xlayout.toml")
}

/// A line of 240 positions, blank but for the pieces given: (start, text).
fn line(pieces: &[(usize, &str)]) -> String {
    let mut chars = vec![' '; 240];
    for (start, text) in pieces {
        for (i, c) in text.chars().enumerate() {
            chars[start - 1 + i] = c;
        }
    }
    chars.into_iter().collect()
}

/// A payment file: one batch of credits (segments A and B) of two payments.
pub(crate) fn payments(lines_in_trailer: &str, sum: &str) -> String {
    let header = line(&[
        (1, "341"),
        (4, "0000"),
        (8, "0"),
        (18, "2"),
        (19, "11222333000181"),
        (73, "EMPRESA FICTICIA LTDA"),
        (103, "BANCO FICTICIO"),
        (143, "1"),
        (144, "09112026"),
        (152, "143000"),
        (158, "000001"),
        (164, "103"),
    ]);
    let batch = line(&[
        (1, "341"),
        (4, "0001"),
        (8, "1"),
        (9, "C"),
        (10, "20"),
        (12, "01"),
        (14, "046"),
        (18, "2"),
        (19, "11222333000181"),
        (73, "EMPRESA FICTICIA LTDA"),
    ]);
    let a = |seq: &str, value: &str, name: &str| {
        line(&[
            (1, "341"),
            (4, "0001"),
            (8, "3"),
            (9, seq),
            (14, "A"),
            (15, "0"),
            (16, "00"),
            (44, name),
            (94, "10112026"),
            (102, "BRL"),
            (120, value),
        ])
    };
    let b = |seq: &str| {
        line(&[
            (1, "341"),
            (4, "0001"),
            (8, "3"),
            (9, seq),
            (14, "B"),
            (18, "1"),
            (19, "00012345678909"),
        ])
    };
    let trailer = line(&[
        (1, "341"),
        (4, "0001"),
        (8, "5"),
        (18, lines_in_trailer),
        (24, sum),
    ]);
    let end = line(&[
        (1, "341"),
        (4, "9999"),
        (8, "9"),
        (18, "000001"),
        (24, "000008"),
    ]);
    [
        header,
        batch,
        a("00001", "000000000012345", "FORNECEDOR UM"),
        b("00002"),
        a("00003", "000000000100000", "FORNECEDOR DOIS"),
        b("00004"),
        trailer,
        end,
    ]
    .join("\r\n")
        + "\r\n"
}

fn write(text: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = crate::testing::tempdir();
    let path = tmp.path().join("data.rem");
    std::fs::write(&path, text).unwrap();
    (tmp, path)
}

fn read_all(path: &Path, layout: &Layout, target: Target) -> (Seen, Scan) {
    let mut seen = Seen::default();
    let reading = Reading {
        target: &target,
        charset: None,
        count_all: true,
    };
    let scan = read(path, layout, reading, Budget::unlimited(), &mut seen).unwrap();
    (seen, scan)
}

fn about(seen: &Seen) -> String {
    seen.structure.as_ref().unwrap().about.join("\n")
}

#[test]
fn a_payment_file_has_a_sheet_per_record_in_the_order_they_appear() {
    let (_tmp, path) = write(payments("000006", "000000000000112345").as_bytes());
    let layout = cnab();
    let (seen, scan) = read_all(&path, &layout, Target::First);
    assert_eq!(
        seen.sheets,
        [
            "0 - Header de Arquivo (1)",
            "1C - Header de Lote — Pagamentos (1)",
            "3A - Segmento A — Crédito em conta, cheque, OP, TED, Pix (2)",
            "3B - Segmento B — Dados do favorecido (2)",
            "5C - Trailer de Lote — Pagamentos (1)",
            "9 - Trailer de Arquivo (1)",
        ]
    );
    assert_eq!(scan.keys[0], "r:0");
    assert_eq!(scan.end, End::Complete);
    assert_eq!(seen.current, 0);
    assert_eq!(seen.columns[0], "#line");
    assert_eq!(seen.rows.len(), 1);
    let row = &seen.rows[0];
    let col = |name: &str| row[seen.columns.iter().position(|c| c == name).unwrap()].clone();
    assert_eq!(col("#line"), "1");
    assert_eq!(col("banco"), "341");
    assert_eq!(col("data_geracao"), "2026-11-09");
    assert_eq!(col("hora_geracao"), "14:30:00");
    assert_eq!(col("numero_inscricao"), "11222333000181");
    assert!(
        !seen.columns.iter().any(|c| c.starts_with("cnab_")),
        "filler is hidden"
    );
    // The meaning of a code, for the cell's footer.
    let at = seen
        .columns
        .iter()
        .position(|c| c == "codigo_remessa_retorno")
        .unwrap();
    assert!(
        seen.domains[at]
            .iter()
            .any(|(v, m)| v == "1" && m.starts_with("Remessa"))
    );
    let text = about(&seen);
    assert!(
        text.contains("Checks of totals: 4 passed, 0 failed"),
        "{text}"
    );
    assert!(seen.notices.is_empty());
}

#[test]
fn a_sheet_is_read_by_its_key_with_exact_values() {
    let (_tmp, path) = write(payments("000006", "000000000000112345").as_bytes());
    let layout = cnab();
    let (seen, _) = read_all(&path, &layout, Target::Key("r:3A".into()));
    assert_eq!(seen.current, 2);
    assert_eq!(seen.rows.len(), 2);
    let value = seen
        .columns
        .iter()
        .position(|c| c == "valor_pagamento")
        .unwrap();
    let line = seen.columns.iter().position(|c| c == "#line").unwrap();
    assert_eq!(seen.rows[0][value], "123.45");
    assert_eq!(seen.rows[1][value], "1000.00");
    assert_eq!(seen.rows[1][line], "5");
    let s = seen.structure.unwrap();
    assert_eq!(s.titles[1], "Position");
    let filler = s.columns.iter().find(|c| c[0] == "cnab_227").unwrap();
    assert_eq!(filler[1], "227..229", "filler is listed in the structure");
}

#[test]
fn totals_that_disagree_are_reported() {
    // 5 lines said, 6 counted; the sum is off by one cent.
    let (_tmp, path) = write(payments("000005", "000000000000112344").as_bytes());
    let (seen, _) = read_all(&path, &cnab(), Target::First);
    assert_eq!(seen.notices, ["2 checks failed: i shows"]);
    let text = about(&seen);
    assert!(text.contains("2 failed"), "{text}");
    assert!(
        text.contains("line 7, 5C quantidade_registros_lote: says 5, counted 6"),
        "{text}"
    );
    assert!(
        text.contains("line 7, 5C soma_valores: says 1123.44, counted 1123.45"),
        "{text}"
    );
}

#[test]
fn a_collection_return_reads_its_own_segments() {
    let header = line(&[
        (1, "001"),
        (4, "0000"),
        (8, "0"),
        (143, "2"),
        (144, "09112026"),
        (164, "103"),
    ]);
    let batch = line(&[
        (1, "001"),
        (4, "0001"),
        (8, "1"),
        (9, "T"),
        (10, "01"),
        (14, "060"),
    ]);
    let t = line(&[
        (1, "001"),
        (4, "0001"),
        (8, "3"),
        (9, "00001"),
        (14, "T"),
        (16, "06"),
        (74, "30112026"),
        (82, "000000000050000"),
    ]);
    let u = line(&[
        (1, "001"),
        (4, "0001"),
        (8, "3"),
        (9, "00002"),
        (14, "U"),
        (16, "06"),
        (78, "000000000050000"),
    ]);
    let trailer = line(&[(1, "001"), (4, "0001"), (8, "5"), (18, "000004")]);
    let end = line(&[
        (1, "001"),
        (4, "9999"),
        (8, "9"),
        (18, "000001"),
        (24, "000006"),
    ]);
    let text = [header, batch, t, u, trailer, end].join("\n");
    let (_tmp, path) = write(text.as_bytes());
    let (seen, scan) = read_all(&path, &cnab(), Target::Key("r:3U".into()));
    assert_eq!(scan.keys, ["r:0", "r:1R", "r:3T", "r:3U", "r:5R", "r:9"]);
    let paid = seen.columns.iter().position(|c| c == "valor_pago").unwrap();
    assert_eq!(seen.rows[0][paid], "500.00");
    assert!(
        about(&seen).contains("3 passed, 0 failed"),
        "{}",
        about(&seen)
    );
}

#[test]
fn a_segment_of_a_service_not_declared_goes_to_the_lines_with_no_record() {
    // A debit batch (operation D) has an A segment of its own.
    let header = line(&[(1, "001"), (4, "0000"), (8, "0"), (143, "1"), (164, "103")]);
    let batch = line(&[(1, "001"), (4, "0001"), (8, "1"), (9, "D"), (10, "05")]);
    let a = line(&[(1, "001"), (4, "0001"), (8, "3"), (14, "A")]);
    let text = [header, batch, a, "garbage".to_string()].join("\n");
    let (_tmp, path) = write(text.as_bytes());
    let (seen, scan) = read_all(&path, &cnab(), Target::Key(ORPHANS.into()));
    assert_eq!(scan.keys, ["r:0", "!"]);
    assert_eq!(seen.sheets[1], "! No record type (3)");
    assert_eq!(seen.columns, ["#line", "line"]);
    assert_eq!(seen.rows[2], ["4", "garbage"]);
}

/// A SPED file, signed: the signature follows the 9999.
fn sped_file(c100_count: &str) -> Vec<u8> {
    let lines = [
        "|0000|019|0|01102026|31102026|EMPRESA FICTÍCIA|11222333000181||SP|123456789|3550308|||A|1|",
        "|0001|0|",
        "|0990|3|",
        "|C001|0|",
        "|C100|0|1|PART01|55|00|001|123|35261011222333000181550010000001231000001234|01102026|01102026|1500,50|",
        "|C100|0|1|PART02|55|00|001|124||02102026|02102026|10|",
        "|C990|4|",
        "|9001|0|",
        "|9900|0000|1|",
        "|9900|0001|1|",
        "|9900|0990|1|",
        "|9900|C001|1|",
        &format!("|9900|C100|{c100_count}|"),
        "|9900|C990|1|",
        "|9900|9001|1|",
        "|9900|9900|10|",
        "|9900|9990|1|",
        "|9900|9999|1|",
        "|9990|13|",
        "|9999|20|",
    ];
    let mut bytes = charset_1252(&lines.join("\r\n"));
    bytes.extend_from_slice(b"\r\nSBRCAAEPDR\x00\x01\x02 signature bytes");
    bytes
}

fn charset_1252(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| match c {
            'Í' => 0xCD,
            c if c.is_ascii() => c as u8,
            _ => b'?',
        })
        .collect()
}

#[test]
fn a_sped_file_numbers_the_fields_it_does_not_declare_and_checks_block_9() {
    let (_tmp, path) = write(&sped_file("2"));
    let layout = sped();
    let (seen, scan) = read_all(&path, &layout, Target::Key("c:C100".into()));
    assert_eq!(
        scan.keys,
        ["c:0000", "r:x001", "r:x990", "c:C100", "r:9900", "r:9999"]
    );
    assert_eq!(seen.sheets[3], "C100 (2)");
    assert_eq!(seen.sheets[1], "x001 - Abertura do bloco (3)");
    assert_eq!(seen.columns[..4], ["#line", "REG", "CAMPO_02", "CAMPO_03"]);
    assert_eq!(seen.rows[0][1], "C100");
    // The key of 44 digits, whole.
    assert_eq!(
        seen.rows[0][9],
        "35261011222333000181550010000001231000001234"
    );
    assert_eq!(seen.rows[1][9], "", "an empty field is empty");
    let text = about(&seen);
    assert!(
        text.contains("Checks of totals: 14 passed, 0 failed"),
        "{text}"
    );
    assert!(
        text.contains("bytes after the closing record were not read"),
        "{text}"
    );
    // The charset the viewer shows the file in.
    let mut zero = Seen::default();
    let reading = Reading {
        target: &Target::First,
        charset: Some(Charset::Cp1252),
        count_all: true,
    };
    read(&path, &layout, reading, Budget::unlimited(), &mut zero).unwrap();
    assert!(
        zero.rows[0].iter().any(|c| c == "EMPRESA FICTÍCIA"),
        "{:?}",
        zero.rows[0]
    );
}

#[test]
fn a_wrong_count_in_9900_is_reported() {
    let (_tmp, path) = write(&sped_file("3"));
    let (seen, _) = read_all(&path, &sped(), Target::First);
    let text = about(&seen);
    assert!(
        text.contains("9900 QTD_REG_BLC: says 3, counted 2"),
        "{text}"
    );
}

#[test]
fn a_batch_stops_when_it_has_its_rows() {
    let (_tmp, path) = write(payments("000006", "000000000000112345").as_bytes());
    let mut seen = Seen {
        limit: Some(1),
        ..Default::default()
    };
    let target = Target::Key("r:3A".into());
    let reading = Reading {
        target: &target,
        charset: None,
        count_all: false,
    };
    let scan = read(&path, &cnab(), reading, Budget::unlimited(), &mut seen).unwrap();
    assert_eq!(scan.end, End::Refused);
    assert_eq!(seen.rows.len(), 1);
    assert!(about(&seen).contains("checks were not done"));
}

#[test]
fn a_line_without_end_is_cut_and_set_aside() {
    let mut text = payments("000006", "000000000000112345").into_bytes();
    text.extend(std::iter::repeat_n(b'x', MAX_LINE_BYTES * 2));
    let (_tmp, path) = write(&text);
    let (seen, _) = read_all(&path, &cnab(), Target::Key(ORPHANS.into()));
    assert_eq!(seen.rows.len(), 1);
    assert!(seen.rows[0][1].len() <= 4096);
    assert!(about(&seen).contains("Lines longer than 512 KiB"));
}

#[test]
fn accented_text_is_cut_by_characters_or_by_bytes() {
    let text = "schema = 1\nname = \"t\"\nline_length = 6\n[[records]]\nname = \"r\"\nfields = [\n  { name = \"a\", pos = [1, 3], type = \"text\" },\n  { name = \"b\", pos = [4, 6], type = \"text\" },\n]\n";
    let chars = xlayout::parse(text).unwrap();
    let bytes = xlayout::parse(&text.replace(
        "line_length = 6",
        "line_length = 6\noffset_unit = \"bytes\"",
    ))
    .unwrap();
    let (_tmp, path) = write("AÇÃOxy\n".as_bytes());
    let (seen, _) = read_all(&path, &chars, Target::First);
    assert_eq!(seen.rows[0][1..], ["AÇÃ", "Oxy"]);
    let (seen, _) = read_all(&path, &bytes, Target::First);
    // "AÇ" is 3 bytes; the next 3 bytes are Ã and O.
    assert_eq!(seen.rows[0][1..], ["AÇ", "ÃO"]);
}

#[test]
fn every_catalog_record_covers_its_whole_line() {
    for (path, text) in CATALOG {
        let layout = xlayout::parse(text).unwrap();
        if let Format::Positional { .. } = layout.format {
            for r in &layout.records {
                assert_eq!(r.line_length, Some(240), "{path} {}", r.id);
            }
        }
    }
}

#[test]
fn detection_reads_only_the_trusted_layouts() {
    let (_tmp, path) = write(payments("000006", "000000000000112345").as_bytes());
    // A hostile layout next to the file is never used by itself.
    std::fs::write(
        path.with_file_name("data.xlayout.toml"),
        "schema = 1\nname = \"evil\"\n[detect]\nline_length = 240\n[[records]]\nname = \"r\"\nfields = [{ name = \"f\", pos = [1, 240], type = \"text\" }]\n",
    )
    .unwrap();
    let start = std::fs::read(&path).unwrap();
    let found = catalog::detect(&path, &start, Charset::Utf8).unwrap();
    assert_eq!(found.layout.name, "FEBRABAN CNAB 240 — Padrão Geral");
    // But K offers it, first, as the one next to the file.
    let choices = catalog::choices(&path, false);
    assert_eq!(choices[0].label, "data.xlayout.toml");
    assert_eq!(choices[0].section, catalog::Section::Here);
}

#[test]
fn a_dbf_gives_the_layout_of_its_sdf() {
    use dbase::{FieldName, TableWriterBuilder};
    let tmp = crate::testing::tempdir();
    let dbf = tmp.path().join("people.dbf");
    let name = |s: &str| FieldName::try_from(s).unwrap();
    let writer = TableWriterBuilder::new()
        .add_character_field(name("NAME"), 10)
        .add_numeric_field(name("SALARY"), 8, 2)
        .add_date_field(name("BORN"))
        .add_logical_field(name("ACTIVE"))
        .build_with_file_dest(&dbf)
        .unwrap();
    drop(writer);
    let layout = catalog::sdf_from_dbf(&dbf).unwrap();
    let fields: Vec<_> = layout.records[0]
        .fields
        .iter()
        .map(|f| (f.name.as_str(), f.at, f.kind))
        .collect();
    assert_eq!(
        fields,
        [
            ("NAME", At::Pos { start: 1, end: 10 }, Kind::Text),
            ("SALARY", At::Pos { start: 11, end: 18 }, Kind::Decimal),
            ("BORN", At::Pos { start: 19, end: 26 }, Kind::Date),
            ("ACTIVE", At::Pos { start: 27, end: 27 }, Kind::Text),
        ]
    );
    let sdf = tmp.path().join("people.txt");
    std::fs::write(
        &sdf,
        "Ana        1234.5019800131T\r\nRui         -12.00        F\r\n\x1a",
    )
    .unwrap();
    let (seen, _) = read_all(&sdf, &layout, Target::First);
    assert_eq!(seen.rows[0][1..], ["Ana", "1234.50", "1980-01-31", "T"]);
    assert_eq!(seen.rows[1][1..], ["Rui", "-12.00", "", "F"]);
    // The end-of-file mark of DOS is no record.
    assert_eq!(seen.rows.len(), 2);
    assert_eq!(seen.sheets, ["SDF - Records (2)"]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn any_bytes_are_read_without_panic(data in proptest::collection::vec(any::<u8>(), 0..4096), cut in 0usize..4) {
        let (_tmp, path) = write(&data);
        for layout in [cnab(), sped()] {
            let target = match cut {
                0 => Target::First,
                1 => Target::Key(ORPHANS.into()),
                2 => Target::Key("r:3A".into()),
                _ => Target::Key("c:C100".into()),
            };
            let _ = read_all(&path, &layout, target);
        }
    }

    #[test]
    fn lines_made_of_pipes_and_digits_are_read_without_panic(lines in proptest::collection::vec("[|0-9A-Z,]{0,60}", 0..40)) {
        let (_tmp, path) = write(lines.join("\n").as_bytes());
        let _ = read_all(&path, &sped(), Target::First);
    }
}
