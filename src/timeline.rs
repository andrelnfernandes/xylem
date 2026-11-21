//! The Timeline (Alt+V, T; docs/plans/views.md): the files of the list in a
//! histogram by the date they were changed. It reads nothing from the disk:
//! only the dates already in memory. A bar, or a range of bars, goes back to
//! the list as a condition of W on `modified`.
//!
//! Dates before 1980 and in the future go into a bar of their own at each
//! end: hostile or wrong dates exist, and must not flatten the chart.

use std::time::SystemTime;

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, Timelike};

/// Most bars of a scale: past it, a finer scale is refused.
pub const MAX_BARS: usize = 200_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scale {
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

impl Scale {
    pub fn name(self) -> &'static str {
        match self {
            Scale::Hours => "hours",
            Scale::Days => "days",
            Scale::Weeks => "weeks",
            Scale::Months => "months",
            Scale::Years => "years",
        }
    }

    pub fn finer(self) -> Option<Scale> {
        match self {
            Scale::Hours => None,
            Scale::Days => Some(Scale::Hours),
            Scale::Weeks => Some(Scale::Days),
            Scale::Months => Some(Scale::Weeks),
            Scale::Years => Some(Scale::Months),
        }
    }

    pub fn coarser(self) -> Option<Scale> {
        match self {
            Scale::Hours => Some(Scale::Days),
            Scale::Days => Some(Scale::Weeks),
            Scale::Weeks => Some(Scale::Months),
            Scale::Months => Some(Scale::Years),
            Scale::Years => None,
        }
    }

    /// The scale for dates from `first` to `last`.
    pub fn for_span(first: NaiveDateTime, last: NaiveDateTime) -> Scale {
        let days = (last - first).num_days();
        if days <= 2 {
            Scale::Hours
        } else if days <= 90 {
            Scale::Days
        } else if days <= 365 {
            Scale::Weeks
        } else if days <= 3650 {
            Scale::Months
        } else {
            Scale::Years
        }
    }

    /// The start of the bar that holds `t`.
    pub fn floor(self, t: NaiveDateTime) -> NaiveDateTime {
        let day = t.date();
        let start = match self {
            Scale::Hours => return day.and_hms_opt(t.hour(), 0, 0).unwrap_or(t),
            Scale::Days => day,
            Scale::Weeks => day - Duration::days(i64::from(day.weekday().num_days_from_monday())),
            Scale::Months => day.with_day(1).unwrap_or(day),
            Scale::Years => NaiveDate::from_ymd_opt(day.year(), 1, 1).unwrap_or(day),
        };
        start.and_hms_opt(0, 0, 0).unwrap_or(t)
    }

    /// The start of the bar after the one starting at `start`.
    pub fn next(self, start: NaiveDateTime) -> NaiveDateTime {
        match self {
            Scale::Hours => start + Duration::hours(1),
            Scale::Days => start + Duration::days(1),
            Scale::Weeks => start + Duration::days(7),
            Scale::Months => {
                let (y, m) = (start.year(), start.month());
                let (y, m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                NaiveDate::from_ymd_opt(y, m, 1)
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .unwrap_or(start + Duration::days(31))
            }
            Scale::Years => NaiveDate::from_ymd_opt(start.year() + 1, 1, 1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .unwrap_or(start + Duration::days(366)),
        }
    }

    /// How a bar is called: `2026-11-18 14h`, `2026-11-18`, `week of
    /// 2026-11-16`, `2026-11`, `2026`.
    pub fn label(self, start: NaiveDateTime) -> String {
        match self {
            Scale::Hours => start.format("%Y-%m-%d %Hh").to_string(),
            Scale::Days => start.format("%Y-%m-%d").to_string(),
            Scale::Weeks => start.format("week of %Y-%m-%d").to_string(),
            Scale::Months => start.format("%Y-%m").to_string(),
            Scale::Years => start.format("%Y").to_string(),
        }
    }

    /// The short mark under the axis.
    pub fn tick(self, start: NaiveDateTime) -> String {
        match self {
            Scale::Hours => start.format("%d %Hh").to_string(),
            Scale::Days | Scale::Weeks => start.format("%m-%d").to_string(),
            Scale::Months => start.format("%Y-%m").to_string(),
            Scale::Years => start.format("%Y").to_string(),
        }
    }
}

/// What a bar holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    /// Before 1980.
    Older,
    /// From `start` to `end` (not included).
    Period {
        start: NaiveDateTime,
        end: NaiveDateTime,
    },
    /// In the future (more than a day ahead of now).
    Future,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bar {
    pub span: Span,
    pub files: u64,
    pub bytes: u64,
}

/// The dates of a list, and its bars at one scale.
#[derive(Debug, Clone)]
pub struct Timeline {
    /// Local date and size of each file with a date, sorted.
    dates: Vec<(NaiveDateTime, u64)>,
    /// Files with no date.
    pub undated: u64,
    /// The first date in the future and the earliest date not too old.
    future_from: NaiveDateTime,
    older_than: NaiveDateTime,
    pub scale: Scale,
    pub bars: Vec<Bar>,
}

fn local(t: SystemTime) -> NaiveDateTime {
    let dt: chrono::DateTime<Local> = t.into();
    dt.naive_local()
}

/// How a date is written for W (`modified` is local time as text).
pub fn sql_date(t: NaiveDateTime) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

impl Timeline {
    /// The bars of `files` (date and size), at the scale their dates call
    /// for; `now` tells the future.
    pub fn new(
        files: impl IntoIterator<Item = (Option<SystemTime>, u64)>,
        now: SystemTime,
    ) -> Self {
        let mut undated = 0;
        let mut dates: Vec<(NaiveDateTime, u64)> = files
            .into_iter()
            .filter_map(|(t, size)| {
                if t.is_none() {
                    undated += 1;
                }
                Some((local(t?), size))
            })
            .collect();
        dates.sort_unstable();
        let future_from = local(now) + Duration::days(1);
        let older_than = NaiveDate::from_ymd_opt(1980, 1, 1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .unwrap_or_default();
        let inside: Vec<NaiveDateTime> = dates
            .iter()
            .map(|d| d.0)
            .filter(|d| *d >= older_than && *d < future_from)
            .collect();
        let scale = match (inside.first(), inside.last()) {
            (Some(&a), Some(&b)) => Scale::for_span(a, b),
            _ => Scale::Years,
        };
        let mut t = Timeline {
            dates,
            undated,
            future_from,
            older_than,
            scale,
            bars: Vec::new(),
        };
        t.build();
        t
    }

    pub fn files(&self) -> usize {
        self.dates.len()
    }

    /// The bars at the current scale. False, and nothing changed, if there
    /// would be too many.
    fn build(&mut self) -> bool {
        let inside = |d: &NaiveDateTime| *d >= self.older_than && *d < self.future_from;
        let mut older = Bar {
            span: Span::Older,
            files: 0,
            bytes: 0,
        };
        let mut future = Bar {
            span: Span::Future,
            ..older
        };
        let first = self.dates.iter().map(|d| d.0).find(|d| inside(d));
        let last = self.dates.iter().rev().map(|d| d.0).find(|d| inside(d));
        let mut bars = Vec::new();
        if let (Some(first), Some(last)) = (first, last) {
            let mut start = self.scale.floor(first);
            while start <= last {
                if bars.len() >= MAX_BARS {
                    return false;
                }
                let end = self.scale.next(start);
                bars.push(Bar {
                    span: Span::Period { start, end },
                    files: 0,
                    bytes: 0,
                });
                start = end;
            }
        }
        let mut at = 0;
        for &(d, size) in &self.dates {
            let bar = if d < self.older_than {
                &mut older
            } else if d >= self.future_from {
                &mut future
            } else {
                // The dates are sorted: the bar only moves forward.
                while let Some(Bar {
                    span: Span::Period { end, .. },
                    ..
                }) = bars.get(at)
                {
                    if d < *end {
                        break;
                    }
                    at += 1;
                }
                &mut bars[at]
            };
            bar.files += 1;
            bar.bytes = bar.bytes.saturating_add(size);
        }
        if older.files > 0 {
            bars.insert(0, older);
        }
        if future.files > 0 {
            bars.push(future);
        }
        self.bars = bars;
        true
    }

    /// Another scale; false if it would make too many bars.
    pub fn set_scale(&mut self, scale: Scale) -> bool {
        let old = self.scale;
        self.scale = scale;
        if self.build() {
            return true;
        }
        self.scale = old;
        self.build();
        false
    }

    /// The bar holding `t`, at the current scale.
    pub fn bar_of(&self, t: NaiveDateTime) -> Option<usize> {
        self.bars.iter().position(|b| match b.span {
            Span::Period { start, end } => t >= start && t < end,
            Span::Older => t < self.older_than,
            Span::Future => t >= self.future_from,
        })
    }

    /// The condition of W for the bars from `a` to `b`.
    pub fn condition(&self, a: usize, b: usize) -> Option<String> {
        let (a, b) = (a.min(b), a.max(b));
        let from = match self.bars.get(a)?.span {
            Span::Older => None,
            Span::Period { start, .. } => Some(start),
            Span::Future => Some(self.future_from),
        };
        let to = match self.bars.get(b)?.span {
            Span::Older => Some(self.older_than),
            Span::Period { end, .. } => Some(end),
            Span::Future => None,
        };
        Some(match (from, to) {
            (Some(f), Some(t)) => format!(
                "modified >= '{}' AND modified < '{}'",
                sql_date(f),
                sql_date(t)
            ),
            (Some(f), None) => format!("modified >= '{}'", sql_date(f)),
            (None, Some(t)) => format!("modified < '{}'", sql_date(t)),
            (None, None) => "modified IS NOT NULL".into(),
        })
    }

    /// The name of a bar.
    pub fn label(&self, i: usize) -> String {
        match self.bars.get(i).map(|b| b.span) {
            Some(Span::Older) => "before 1980".into(),
            Some(Span::Future) => "in the future".into(),
            Some(Span::Period { start, .. }) => self.scale.label(start),
            None => String::new(),
        }
    }

    pub fn tick(&self, i: usize) -> String {
        match self.bars.get(i).map(|b| b.span) {
            Some(Span::Older) => "<1980".into(),
            Some(Span::Future) => "future".into(),
            Some(Span::Period { start, .. }) => self.scale.tick(start),
            None => String::new(),
        }
    }
}

/// `t` as a `SystemTime`, for the tests.
#[cfg(test)]
fn at(y: i32, m: u32, d: u32, h: u32) -> SystemTime {
    use chrono::TimeZone;
    Local
        .with_ymd_and_hms(y, m, d, h, 0, 0)
        .single()
        .unwrap()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> SystemTime {
        at(2026, 11, 21, 12)
    }

    #[test]
    fn the_scale_follows_the_span() {
        let t = Timeline::new(
            [
                (Some(at(2026, 11, 20, 9)), 1),
                (Some(at(2026, 11, 20, 17)), 2),
            ],
            now(),
        );
        assert_eq!(t.scale, Scale::Hours);
        assert_eq!(t.bars.len(), 9);
        assert_eq!(t.bars[0].files, 1);
        assert_eq!(t.bars[8].bytes, 2);
        let t = Timeline::new(
            [
                (Some(at(2026, 10, 1, 9)), 1),
                (Some(at(2026, 11, 20, 17)), 2),
            ],
            now(),
        );
        assert_eq!(t.scale, Scale::Days);
        let t = Timeline::new(
            [
                (Some(at(2010, 10, 1, 9)), 1),
                (Some(at(2026, 11, 20, 17)), 2),
            ],
            now(),
        );
        assert_eq!(t.scale, Scale::Years);
        assert_eq!(t.bars.len(), 17);
    }

    #[test]
    fn old_and_future_dates_have_bars_of_their_own() {
        let t = Timeline::new(
            [
                (Some(SystemTime::UNIX_EPOCH), 10),
                (Some(at(2026, 11, 18, 9)), 1),
                (Some(at(2026, 11, 19, 9)), 1),
                (Some(at(2099, 1, 1, 0)), 5),
                (None, 7),
            ],
            now(),
        );
        assert_eq!(t.undated, 1);
        assert_eq!(t.bars.first().unwrap().span, Span::Older);
        assert_eq!(t.bars.last().unwrap().span, Span::Future);
        // 1970 and 2099 do not stretch the scale: two days, by the hour.
        assert_eq!(t.scale, Scale::Hours);
        assert_eq!(t.label(0), "before 1980");
        assert_eq!(
            t.condition(0, 0).unwrap(),
            "modified < '1980-01-01 00:00:00'"
        );
        let last = t.bars.len() - 1;
        assert!(
            t.condition(last, last)
                .unwrap()
                .starts_with("modified >= '2026-11-22")
        );
    }

    #[test]
    fn a_bar_and_a_range_become_conditions() {
        let t = Timeline::new(
            [
                (Some(at(2026, 11, 1, 9)), 1),
                (Some(at(2026, 11, 20, 17)), 2),
            ],
            now(),
        );
        assert_eq!(t.scale, Scale::Days);
        let i = t.bar_of(local(at(2026, 11, 18, 3))).unwrap();
        assert_eq!(
            t.condition(i, i).unwrap(),
            "modified >= '2026-11-18 00:00:00' AND modified < '2026-11-19 00:00:00'"
        );
        assert_eq!(
            t.condition(i + 1, i - 1).unwrap(),
            "modified >= '2026-11-17 00:00:00' AND modified < '2026-11-20 00:00:00'"
        );
    }

    #[test]
    fn scales_change_and_count_the_same_files() {
        let mut t = Timeline::new(
            [
                (Some(at(2025, 1, 1, 9)), 1),
                (Some(at(2026, 6, 20, 17)), 2),
                (Some(at(2026, 11, 20, 17)), 4),
            ],
            now(),
        );
        assert_eq!(t.scale, Scale::Months);
        for scale in [Scale::Years, Scale::Weeks, Scale::Days, Scale::Hours] {
            assert!(t.set_scale(scale));
            let files: u64 = t.bars.iter().map(|b| b.files).sum();
            let bytes: u64 = t.bars.iter().map(|b| b.bytes).sum();
            assert_eq!((files, bytes), (3, 7), "{scale:?}");
        }
        assert_eq!(t.label(0), "2025-01-01 09h");
    }

    #[test]
    fn too_many_bars_are_refused() {
        let mut t = Timeline::new(
            [
                (Some(at(1981, 1, 1, 9)), 1),
                (Some(at(2026, 11, 20, 17)), 2),
            ],
            now(),
        );
        assert!(!t.set_scale(Scale::Hours));
        assert_eq!(t.scale, Scale::Years);
        assert!(t.set_scale(Scale::Days));
    }

    #[test]
    fn weeks_start_on_monday() {
        let start = Scale::Weeks.floor(local(at(2026, 11, 21, 12)));
        assert_eq!(Scale::Weeks.label(start), "week of 2026-11-16");
    }
}
