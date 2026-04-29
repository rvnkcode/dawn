use crate::table::age::{Age, AgeError};
use chrono::{DateTime, TimeZone, format::DelayedFormat, format::StrftimeItems};
use dawn::domain::task::Timestamp;

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
        format_absolute(ts, tz)?,
        Age::new(ts, now)?
    ))
}

pub(crate) fn format_absolute<Tz: TimeZone>(
    ts: &Timestamp,
    tz: &Tz,
) -> Result<DelayedFormat<StrftimeItems<'static>>, AgeError>
where
    Tz::Offset: std::fmt::Display,
{
    let secs = ts.as_seconds();
    let utc = DateTime::from_timestamp(secs, 0).ok_or(AgeError::OutOfRange(secs))?;
    Ok(utc.with_timezone(tz).format("%Y-%m-%d %H:%M:%S"))
}

pub(crate) fn format_date<Tz: TimeZone>(
    ts: &Timestamp,
    tz: &Tz,
) -> Result<DelayedFormat<StrftimeItems<'static>>, AgeError>
where
    Tz::Offset: std::fmt::Display,
{
    let secs = ts.as_seconds();
    let utc = DateTime::from_timestamp(secs, 0).ok_or(AgeError::OutOfRange(secs))?;
    Ok(utc.with_timezone(tz).format("%Y-%m-%d"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Utc};

    fn ts(secs: i64) -> Timestamp {
        Timestamp::new(secs).unwrap()
    }

    #[test]
    fn format_with_age_appends_parenthesized_age() {
        let now = 1_000_000;
        let out = format_with_age(&ts(now - 30), now, &Utc).unwrap();
        assert!(out.ends_with(" (30s)"), "unexpected suffix in: {out}");
    }

    #[test]
    fn format_absolute_renders_in_provided_timezone() {
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();
        let out = format_absolute(&ts(0), &jst).unwrap().to_string();
        assert_eq!(out, "1970-01-01 09:00:00");
    }

    #[test]
    fn format_absolute_crosses_day_boundary_forward() {
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();
        let out = format_absolute(&ts(20 * 3600), &jst).unwrap().to_string();
        assert_eq!(out, "1970-01-02 05:00:00");
    }

    #[test]
    fn format_date_renders_in_provided_timezone() {
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();
        let out = format_date(&ts(0), &jst).unwrap().to_string();
        assert_eq!(out, "1970-01-01");
    }

    #[test]
    fn format_date_crosses_day_boundary_forward() {
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();
        let out = format_date(&ts(20 * 3600), &jst).unwrap().to_string();
        assert_eq!(out, "1970-01-02");
    }
}
