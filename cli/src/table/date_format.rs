use crate::table::age::{Age, AgeError};
use chrono::{DateTime, TimeZone, format::DelayedFormat, format::StrftimeItems};
use dawn::domain::task::Timestamp;

pub(crate) const DATETIME_FMT: &str = "%Y-%m-%d %H:%M:%S";
pub(crate) const DATE_FMT: &str = "%Y-%m-%d";

pub(crate) fn format_with_age<Tz: TimeZone>(
    ts: &Timestamp,
    now: i64,
    tz: &Tz,
) -> Result<String, AgeError>
where
    Tz::Offset: std::fmt::Display,
{
    Ok(format!(
        "{} ({})",
        format_absolute(ts, tz, DATETIME_FMT)?,
        Age::new(ts, now)?
    ))
}

pub(crate) fn format_absolute<Tz: TimeZone>(
    ts: &Timestamp,
    tz: &Tz,
    fmt: &'static str,
) -> Result<DelayedFormat<StrftimeItems<'static>>, AgeError>
where
    Tz::Offset: std::fmt::Display,
{
    let secs = ts.as_seconds();
    let utc = DateTime::from_timestamp(secs, 0).ok_or(AgeError::OutOfRange(secs))?;
    Ok(utc.with_timezone(tz).format(fmt))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Utc};

    fn ts(secs: i64) -> Timestamp {
        Timestamp::new(secs).unwrap()
    }

    fn jst() -> FixedOffset {
        FixedOffset::east_opt(9 * 3600).unwrap()
    }

    #[test]
    fn format_with_age_appends_parenthesized_age() {
        let now = 1_000_000;
        let out = format_with_age(&ts(now - 30), now, &Utc).unwrap();
        assert!(out.ends_with(" (30s)"), "unexpected suffix in: {out}");
    }

    #[test]
    fn format_absolute_with_datetime_fmt_renders_in_provided_timezone() {
        let out = format_absolute(&ts(0), &jst(), DATETIME_FMT)
            .unwrap()
            .to_string();
        assert_eq!(out, "1970-01-01 09:00:00");
    }

    #[test]
    fn format_absolute_with_datetime_fmt_crosses_day_boundary_forward() {
        let out = format_absolute(&ts(20 * 3600), &jst(), DATETIME_FMT)
            .unwrap()
            .to_string();
        assert_eq!(out, "1970-01-02 05:00:00");
    }

    #[test]
    fn format_absolute_with_date_fmt_renders_in_provided_timezone() {
        let out = format_absolute(&ts(0), &jst(), DATE_FMT)
            .unwrap()
            .to_string();
        assert_eq!(out, "1970-01-01");
    }

    #[test]
    fn format_absolute_with_date_fmt_crosses_day_boundary_forward() {
        let out = format_absolute(&ts(20 * 3600), &jst(), DATE_FMT)
            .unwrap()
            .to_string();
        assert_eq!(out, "1970-01-02");
    }

    #[test]
    fn format_absolute_propagates_out_of_range_error() {
        let bad = ts(i64::MAX);
        let err = format_absolute(&bad, &jst(), DATETIME_FMT).unwrap_err();
        assert!(matches!(err, AgeError::OutOfRange(_)));
    }
}
