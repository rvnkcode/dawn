use crate::table::age::{Age, AgeError};
use chrono::{DateTime, Local, TimeZone, Utc};
use dawn::domain::task::Timestamp;

pub(crate) fn format_absolute(ts: &Timestamp) -> Result<String, AgeError> {
    format_absolute_in(ts, &Local)
}

pub(crate) fn format_with_age(ts: &Timestamp, now: i64) -> Result<String, AgeError> {
    Ok(format!(
        "{} ({})",
        format_absolute_in(ts, &Local)?,
        Age::new(ts, now)?
    ))
}

fn format_absolute_in<Tz>(ts: &Timestamp, tz: &Tz) -> Result<String, AgeError>
where
    Tz: TimeZone,
    Tz::Offset: std::fmt::Display,
{
    let secs = ts.as_seconds();
    let utc = DateTime::<Utc>::from_timestamp(secs, 0).ok_or(AgeError::OutOfRange(secs))?;
    Ok(utc
        .with_timezone(tz)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};

    fn kst() -> FixedOffset {
        FixedOffset::east_opt(9 * 3600).unwrap()
    }

    fn ts(secs: i64) -> Timestamp {
        Timestamp::new(secs).unwrap()
    }

    #[test]
    fn format_absolute_in_renders_local_date() {
        // 2024-01-15 00:00:00 UTC → 2024-01-15 09:00:00 KST
        let epoch = chrono::Utc
            .with_ymd_and_hms(2024, 1, 15, 0, 0, 0)
            .unwrap()
            .timestamp();
        let out = format_absolute_in(&ts(epoch), &kst()).unwrap();
        assert_eq!(out, "2024-01-15 09:00:00");
    }

    #[test]
    fn format_absolute_in_crosses_date_boundary() {
        // 2024-01-14 23:30:00 UTC → 2024-01-15 08:30:00 KST
        let epoch = chrono::Utc
            .with_ymd_and_hms(2024, 1, 14, 23, 30, 0)
            .unwrap()
            .timestamp();
        let out = format_absolute_in(&ts(epoch), &kst()).unwrap();
        assert_eq!(out, "2024-01-15 08:30:00");
    }

    #[test]
    fn format_with_age_appends_parenthesized_age() {
        let now = 1_000_000;
        let out = format_with_age(&ts(now - 30), now).unwrap();
        assert!(out.ends_with(" (30s)"), "unexpected suffix in: {out}");
    }
}
