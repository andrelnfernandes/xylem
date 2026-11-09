//! A field's slice of the line, as the value the layout says it is.
//!
//! Decimals stay text with their exact places (`1234.50`), never a float: money
//! must not round, and the SQL filter reads the text as a number anyway. Sums for
//! the checks are integers at a fixed scale, so they do not round either.

use std::borrow::Cow;

use super::xlayout::{DateFormat, DecimalSep, Field, Kind, MAX_DECIMALS, Sign, TimeFormat, Trim};
use crate::viewer::records::Cell;

/// A value for the grid, and whether the slice was not what the layout says.
pub fn cell(field: &Field, raw: &str) -> (Cell<'static>, bool) {
    match field.kind {
        Kind::Text | Kind::Code | Kind::Filler => (text(trim(raw, field.trim)), false),
        Kind::Int => match number(field, raw) {
            Number::Blank => (Cell::Null, false),
            Number::Value(v) => match i64::try_from(v) {
                Ok(v) => (Cell::Int(v), false),
                // Beyond i64: the digits, exact.
                Err(_) => (text(&v.to_string()), false),
            },
            Number::Invalid => (text(raw.trim()), true),
        },
        Kind::Decimal => match number(field, raw) {
            Number::Blank => (Cell::Null, false),
            Number::Value(v) => (text(&with_places(v, places(field, raw))), false),
            Number::Invalid => (text(raw.trim()), true),
        },
        Kind::Date => match date(raw, field.date) {
            Moment::Blank => (Cell::Null, false),
            Moment::Valid(d) => (text(&d), false),
            Moment::Invalid => (text(raw.trim()), true),
        },
        Kind::Time => match time(raw, field.time) {
            Moment::Blank => (Cell::Null, false),
            Moment::Valid(t) => (text(&t), false),
            Moment::Invalid => (text(raw.trim()), true),
        },
    }
}

fn text(s: &str) -> Cell<'static> {
    if s.is_empty() {
        Cell::Null
    } else {
        Cell::Text(Cow::Owned(s.to_string()))
    }
}

fn trim(raw: &str, how: Trim) -> &str {
    match how {
        Trim::Right => raw.trim_end_matches(' '),
        Trim::Left => raw.trim_start_matches(' '),
        Trim::Both => raw.trim_matches(' '),
        Trim::None => raw,
    }
}

/// The value of a numeric field at [`MAX_DECIMALS`] places, for the checks.
pub fn scaled(field: &Field, raw: &str) -> Option<i128> {
    match number(field, raw) {
        Number::Value(v) => {
            let scale = places(field, raw);
            v.checked_mul(10i128.checked_pow(MAX_DECIMALS - scale)?)
        }
        // A total left blank says zero.
        Number::Blank => Some(0),
        Number::Invalid => None,
    }
}

/// A scaled value as the field shows it.
pub fn show_scaled(field: &Field, value: i128) -> String {
    let places = if field.kind == Kind::Decimal {
        field.decimals
    } else {
        0
    };
    let unit = 10i128.pow(MAX_DECIMALS - places);
    let rounded = value / unit;
    if value % unit == 0 {
        with_places(rounded, places)
    } else {
        with_places(value, MAX_DECIMALS)
    }
}

enum Number {
    Blank,
    /// The value with as many places as [`places`] says.
    Value(i128),
    Invalid,
}

/// The decimal places of a value: the layout's when implied, else as written
/// (`1255,4` has one), never more than [`MAX_DECIMALS`].
fn places(field: &Field, raw: &str) -> u32 {
    if field.kind != Kind::Decimal || field.decimal_sep == DecimalSep::Implied {
        return field.decimals.min(MAX_DECIMALS);
    }
    let sep = if field.decimal_sep == DecimalSep::Comma {
        ','
    } else {
        '.'
    };
    let written = raw.trim().split_once(sep).map_or(0, |(_, frac)| {
        frac.chars().filter(char::is_ascii_digit).count()
    });
    (written as u32).clamp(field.decimals.min(MAX_DECIMALS), MAX_DECIMALS)
}

fn number(field: &Field, raw: &str) -> Number {
    let s = raw.trim_matches(' ');
    if s.is_empty() {
        return Number::Blank;
    }
    let (negative, digits): (bool, Cow<'_, str>) = match field.sign {
        Sign::None => match s.strip_prefix('-') {
            Some(rest) => (true, Cow::Borrowed(rest)),
            None => (false, Cow::Borrowed(s.strip_prefix('+').unwrap_or(s))),
        },
        Sign::Leading => match s.as_bytes()[0] {
            b'-' => (true, Cow::Borrowed(&s[1..])),
            b'+' => (false, Cow::Borrowed(&s[1..])),
            _ => (false, Cow::Borrowed(s)),
        },
        Sign::Trailing => match s.as_bytes()[s.len() - 1] {
            b'-' => (true, Cow::Borrowed(&s[..s.len() - 1])),
            b'+' => (false, Cow::Borrowed(&s[..s.len() - 1])),
            _ => (false, Cow::Borrowed(s)),
        },
        Sign::Overpunch => {
            let Some(last) = s.chars().last() else {
                return Number::Blank;
            };
            let (neg, digit) = match last {
                '{' => (false, '0'),
                '}' => (true, '0'),
                'A'..='I' => (false, (b'1' + (last as u8 - b'A')) as char),
                'J'..='R' => (true, (b'1' + (last as u8 - b'J')) as char),
                '0'..='9' => (false, last),
                _ => return Number::Invalid,
            };
            let mut d = s[..s.len() - last.len_utf8()].to_string();
            d.push(digit);
            (neg, Cow::Owned(d))
        }
    };
    let digits = digits.trim_matches(' ');
    let value = match (field.kind, field.decimal_sep) {
        (Kind::Decimal, DecimalSep::Comma | DecimalSep::Dot) => {
            let sep = if field.decimal_sep == DecimalSep::Comma {
                ','
            } else {
                '.'
            };
            let (int, frac) = digits.split_once(sep).unwrap_or((digits, ""));
            let places = places(field, raw) as usize;
            if frac.len() > places || (int.is_empty() && frac.is_empty()) {
                return Number::Invalid;
            }
            let mut all = String::with_capacity(int.len() + places);
            all.push_str(int);
            all.push_str(frac);
            all.extend(std::iter::repeat_n('0', places - frac.len()));
            parse_digits(&all)
        }
        _ => parse_digits(digits),
    };
    match value {
        Some(v) if negative => Number::Value(-v),
        Some(v) => Number::Value(v),
        None => Number::Invalid,
    }
}

fn parse_digits(s: &str) -> Option<i128> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    // 38 digits always fit in an i128.
    let s = s.trim_start_matches('0');
    if s.len() > 38 {
        return None;
    }
    if s.is_empty() {
        return Some(0);
    }
    s.parse().ok()
}

/// `123450` with 2 places -> `1234.50`.
fn with_places(value: i128, places: u32) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let places = places as usize;
    let mut out = String::new();
    if negative {
        out.push('-');
    }
    if places == 0 {
        out.push_str(&digits);
        return out;
    }
    if digits.len() <= places {
        out.push_str("0.");
        out.extend(std::iter::repeat_n('0', places - digits.len()));
        out.push_str(&digits);
    } else {
        let (int, frac) = digits.split_at(digits.len() - places);
        out.push_str(int);
        out.push('.');
        out.push_str(frac);
    }
    out
}

enum Moment {
    Blank,
    Valid(String),
    Invalid,
}

fn two(s: &str) -> Option<u32> {
    s.parse().ok()
}

fn leap(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn days_in(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Two-digit years: 00 to 69 are 2000s, 70 to 99 the 1900s.
fn century(yy: u32) -> u32 {
    if yy < 70 { 2000 + yy } else { 1900 + yy }
}

fn date(raw: &str, format: DateFormat) -> Moment {
    let s = raw.trim_matches(' ');
    if s.is_empty() || s.bytes().all(|b| b == b'0') {
        return Moment::Blank;
    }
    let wanted = match format {
        DateFormat::Yyyymmdd | DateFormat::Ddmmyyyy => 8,
        DateFormat::Yymmdd | DateFormat::Ddmmyy => 6,
        DateFormat::Yyyyddd => 7,
    };
    if s.len() != wanted || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Moment::Invalid;
    }
    let (y, m, d) = match format {
        DateFormat::Yyyymmdd => (s[0..4].parse().ok(), two(&s[4..6]), two(&s[6..8])),
        DateFormat::Ddmmyyyy => (s[4..8].parse().ok(), two(&s[2..4]), two(&s[0..2])),
        DateFormat::Yymmdd => (two(&s[0..2]).map(century), two(&s[2..4]), two(&s[4..6])),
        DateFormat::Ddmmyy => (two(&s[4..6]).map(century), two(&s[2..4]), two(&s[0..2])),
        DateFormat::Yyyyddd => {
            let (Some(y), Some(mut day)) =
                (s[0..4].parse::<u32>().ok(), s[4..7].parse::<u32>().ok())
            else {
                return Moment::Invalid;
            };
            let mut month = 1;
            while month <= 12 && day > days_in(y, month) {
                day -= days_in(y, month);
                month += 1;
            }
            if day == 0 || month > 12 {
                return Moment::Invalid;
            }
            (Some(y), Some(month), Some(day))
        }
    };
    match (y, m, d) {
        (Some(y), Some(m), Some(d))
            if y > 0 && (1..=12).contains(&m) && d >= 1 && d <= days_in(y, m) =>
        {
            Moment::Valid(format!("{y:04}-{m:02}-{d:02}"))
        }
        _ => Moment::Invalid,
    }
}

fn time(raw: &str, format: TimeFormat) -> Moment {
    let s = raw.trim_matches(' ');
    if s.is_empty() {
        return Moment::Blank;
    }
    let wanted = match format {
        TimeFormat::Hhmmss => 6,
        TimeFormat::Hhmm => 4,
    };
    if s.len() != wanted || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Moment::Invalid;
    }
    let h = two(&s[0..2]).unwrap_or(99);
    let m = two(&s[2..4]).unwrap_or(99);
    let sec = if wanted == 6 {
        two(&s[4..6]).unwrap_or(99)
    } else {
        0
    };
    if h > 23 || m > 59 || sec > 59 {
        return Moment::Invalid;
    }
    Moment::Valid(format!("{h:02}:{m:02}:{sec:02}"))
}

#[cfg(test)]
mod tests {
    use super::super::xlayout::At;
    use super::*;
    use proptest::prelude::*;

    fn field(kind: Kind) -> Field {
        Field {
            name: "f".into(),
            at: At::Pos { start: 1, end: 1 },
            kind,
            trim: Trim::Right,
            sign: Sign::None,
            decimals: 0,
            decimal_sep: DecimalSep::Implied,
            date: DateFormat::Ddmmyyyy,
            time: TimeFormat::Hhmmss,
            desc: String::new(),
            values: Vec::new(),
            check: None,
        }
    }

    fn shown(f: &Field, raw: &str) -> (String, bool) {
        let (c, bad) = cell(f, raw);
        (c.display(), bad)
    }

    #[test]
    fn codes_keep_their_leading_zeros() {
        let f = field(Kind::Code);
        assert_eq!(shown(&f, "00012345  "), ("00012345".into(), false));
        assert_eq!(cell(&f, "     ").0, Cell::Null);
    }

    #[test]
    fn implied_decimals_are_exact() {
        let mut f = field(Kind::Decimal);
        f.decimals = 2;
        assert_eq!(shown(&f, "000000000123450"), ("1234.50".into(), false));
        assert_eq!(shown(&f, "000000000000005"), ("0.05".into(), false));
        assert_eq!(
            shown(&f, "0000000000000x5"),
            ("0000000000000x5".into(), true)
        );
        assert_eq!(cell(&f, "   ").0, Cell::Null);
    }

    #[test]
    fn written_decimals_keep_what_was_written() {
        let mut f = field(Kind::Decimal);
        f.decimal_sep = DecimalSep::Comma;
        assert_eq!(shown(&f, "1129998,99"), ("1129998.99".into(), false));
        assert_eq!(shown(&f, "10000"), ("10000".into(), false));
        assert_eq!(shown(&f, "0,010"), ("0.010".into(), false));
        assert!(shown(&f, "1.255,42").1);
        f.decimals = 2;
        assert_eq!(shown(&f, "18,5"), ("18.50".into(), false));
    }

    #[test]
    fn signs_and_overpunch() {
        let mut f = field(Kind::Int);
        f.sign = Sign::Trailing;
        assert_eq!(shown(&f, "0042-"), ("-42".into(), false));
        f.sign = Sign::Overpunch;
        assert_eq!(shown(&f, "004B"), ("42".into(), false));
        assert_eq!(shown(&f, "004K"), ("-42".into(), false));
        assert_eq!(shown(&f, "004}"), ("-40".into(), false));
        assert!(shown(&f, "004%").1);
    }

    #[test]
    fn dates_become_iso_and_bad_ones_stay_as_written() {
        let f = field(Kind::Date);
        assert_eq!(shown(&f, "29022024"), ("2024-02-29".into(), false));
        assert_eq!(shown(&f, "29022023"), ("29022023".into(), true));
        assert_eq!(cell(&f, "00000000").0, Cell::Null);
        let mut f = field(Kind::Date);
        f.date = DateFormat::Yyyyddd;
        assert_eq!(shown(&f, "2024060"), ("2024-02-29".into(), false));
        assert!(shown(&f, "2023366").1);
        f.date = DateFormat::Ddmmyy;
        assert_eq!(shown(&f, "010169"), ("2069-01-01".into(), false));
        assert_eq!(shown(&f, "010170"), ("1970-01-01".into(), false));
    }

    #[test]
    fn times_are_checked() {
        let f = field(Kind::Time);
        assert_eq!(shown(&f, "235959"), ("23:59:59".into(), false));
        assert!(shown(&f, "240000").1);
    }

    #[test]
    fn huge_ints_stay_exact() {
        let f = field(Kind::Int);
        assert_eq!(
            shown(&f, "99999999999999999999"),
            ("99999999999999999999".into(), false)
        );
    }

    #[test]
    fn scaled_values_add_up_without_rounding() {
        let mut f = field(Kind::Decimal);
        f.decimals = 2;
        let a = scaled(&f, "0000000010").unwrap();
        let b = scaled(&f, "0000000020").unwrap();
        assert_eq!(show_scaled(&f, a + b), "0.30");
        assert_eq!(scaled(&field(Kind::Int), "000120"), Some(120_000_000_000));
    }

    proptest! {
        #[test]
        fn any_slice_decodes_without_panic(raw in ".{0,40}", kind in 0u8..7, sign in 0u8..4, sep in 0u8..3, places in 0u32..12) {
            let mut f = field([Kind::Text, Kind::Code, Kind::Int, Kind::Decimal, Kind::Date, Kind::Time, Kind::Filler][kind as usize]);
            f.sign = [Sign::None, Sign::Leading, Sign::Trailing, Sign::Overpunch][sign as usize];
            f.decimal_sep = [DecimalSep::Implied, DecimalSep::Dot, DecimalSep::Comma][sep as usize];
            f.decimals = places.min(MAX_DECIMALS);
            let _ = cell(&f, &raw);
            let _ = scaled(&f, &raw);
        }
    }
}
