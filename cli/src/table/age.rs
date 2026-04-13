use chrono::{DateTime, Datelike, Timelike, Utc};
use dawn::domain::task::Timestamp;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;

#[derive(Debug, PartialEq, Error)]
pub(crate) enum AgeError {
    #[error("Invalid age delta: from={from}, to={to}")]
    InvalidDelta { from: i64, to: i64 },
    #[error("Timestamp out of DateTime range: {0}")]
    OutOfRange(i64),
}

#[derive(Debug)]
pub(crate) struct Age(String);

impl Age {
    pub(crate) fn new(entry: &Timestamp, now: i64) -> Result<Self, AgeError> {
        let from_secs = entry.as_seconds();
        // entry timestamp must not be in the future
        if now < from_secs {
            return Err(AgeError::InvalidDelta {
                from: from_secs,
                to: now,
            });
        }
        let from =
            DateTime::<Utc>::from_timestamp(from_secs, 0).ok_or(AgeError::OutOfRange(from_secs))?;
        let to = DateTime::<Utc>::from_timestamp(now, 0).ok_or(AgeError::OutOfRange(now))?;
        Ok(Age(format_age(from, to)))
    }
}

impl Display for Age {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn format_age(from: DateTime<Utc>, to: DateTime<Utc>) -> String {
    let diff = (to - from).num_seconds();
    if diff < MINUTE {
        return format!("{diff}s");
    }
    if diff < HOUR {
        return format!("{}min", diff / MINUTE);
    }
    if diff < DAY {
        return format!("{}h", diff / HOUR);
    }
    if diff < 2 * WEEK {
        return format!("{}d", diff / DAY);
    }

    let (years, months) = calendar_ym_diff(from, to);

    match (years, months) {
        (0, m) if m < 3 => format!("{}w", diff / WEEK),
        (0, m) => format!("{m}mo"),
        (y, 0) => format!("{y}y"),
        (y, m) => format!("{y}y{m}mo"),
    }
}

/* Calendar (years, months) diff using borrow. Borrows a month when `to` is
earlier in the month than `from` (comparing day-of-month, then time-of-day).
Caller guarantees `to >= from`, so `total_months >= 0`. */
fn calendar_ym_diff(from: DateTime<Utc>, to: DateTime<Utc>) -> (i32, u32) {
    let year_diff = to.year() - from.year();
    let month_diff = to.month() as i32 - from.month() as i32;
    let is_month_incomplete =
        (to.day(), to.num_seconds_from_midnight()) < (from.day(), from.num_seconds_from_midnight());
    let total_months = year_diff * 12 + month_diff - i32::from(is_month_incomplete);
    (total_months / 12, (total_months % 12) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(secs: i64) -> Timestamp {
        Timestamp::new(secs).unwrap()
    }

    fn ymd(y: i32, m: u32, d: u32) -> i64 {
        Utc.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap().timestamp()
    }

    #[test]
    fn seconds() {
        let now = 1_000_000;
        assert_eq!(Age::new(&ts(now - 30), now).unwrap().to_string(), "30s");
    }

    #[test]
    fn zero_delta() {
        let now = 1_000_000;
        assert_eq!(Age::new(&ts(now), now).unwrap().to_string(), "0s");
    }

    #[test]
    fn minutes() {
        let now = 1_000_000;
        assert_eq!(
            Age::new(&ts(now - 3 * MINUTE), now).unwrap().to_string(),
            "3min"
        );
    }

    #[test]
    fn hours() {
        let now = 1_000_000;
        assert_eq!(
            Age::new(&ts(now - 5 * HOUR), now).unwrap().to_string(),
            "5h"
        );
    }

    #[test]
    fn days() {
        let now = 100_000_000;
        assert_eq!(
            Age::new(&ts(now - 10 * DAY), now).unwrap().to_string(),
            "10d"
        );
    }

    #[test]
    fn days_just_under_two_weeks() {
        let now = 100_000_000;
        assert_eq!(
            Age::new(&ts(now - 13 * DAY), now).unwrap().to_string(),
            "13d"
        );
    }

    #[test]
    fn weeks_at_two_week_boundary() {
        let now = 100_000_000;
        assert_eq!(
            Age::new(&ts(now - 14 * DAY), now).unwrap().to_string(),
            "2w"
        );
    }

    #[test]
    fn weeks_eleven() {
        let now = 100_000_000;
        assert_eq!(
            Age::new(&ts(now - 11 * WEEK), now).unwrap().to_string(),
            "11w"
        );
    }

    #[test]
    fn calendar_months_exact_three() {
        let from = ymd(2024, 1, 15);
        let to = ymd(2024, 4, 15);
        assert_eq!(Age::new(&ts(from), to).unwrap().to_string(), "3mo");
    }

    #[test]
    fn calendar_months_borrow_still_weeks() {
        /* 2024-01-20 → 2024-04-10: day-of-month borrow → 2 calendar months → weeks */
        let from = ymd(2024, 1, 20);
        let to = ymd(2024, 4, 10);
        assert_eq!(Age::new(&ts(from), to).unwrap().to_string(), "11w");
    }

    #[test]
    fn calendar_months_nine_across_year() {
        let from = ymd(2024, 3, 20);
        let to = ymd(2025, 1, 10);
        /* month borrow: 2025-01 vs 2024-03 → 10 months, day-of-month borrow → 9 */
        assert_eq!(Age::new(&ts(from), to).unwrap().to_string(), "9mo");
    }

    #[test]
    fn year_and_months() {
        let from = ymd(2024, 1, 15);
        let to = ymd(2025, 4, 15);
        assert_eq!(Age::new(&ts(from), to).unwrap().to_string(), "1y3mo");
    }

    #[test]
    fn year_exact_no_months() {
        let from = ymd(2024, 1, 15);
        let to = ymd(2025, 1, 15);
        assert_eq!(Age::new(&ts(from), to).unwrap().to_string(), "1y");
    }

    #[test]
    fn year_just_before_anniversary_shows_months() {
        /* 2024-01-15 → 2025-01-14: not yet 1 year → 11mo */
        let from = ymd(2024, 1, 15);
        let to = ymd(2025, 1, 14);
        assert_eq!(Age::new(&ts(from), to).unwrap().to_string(), "11mo");
    }

    #[test]
    fn two_years_no_months() {
        let from = ymd(2023, 6, 1);
        let to = ymd(2025, 6, 1);
        assert_eq!(Age::new(&ts(from), to).unwrap().to_string(), "2y");
    }

    #[test]
    fn invalid_negative_delta() {
        let now = 1_000_000;
        let err = Age::new(&ts(now + 100), now).unwrap_err();
        assert_eq!(
            err,
            AgeError::InvalidDelta {
                from: now + 100,
                to: now,
            }
        );
    }
}
