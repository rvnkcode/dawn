use crate::table::age::{Age, AgeError};
use chrono::{DateTime, Local, Utc};
use dawn::domain::task::Timestamp;

pub(crate) fn format_with_age(ts: &Timestamp, now: i64) -> Result<String, AgeError> {
    Ok(format!("{} ({})", format_absolute(ts)?, Age::new(ts, now)?))
}

pub(crate) fn format_absolute(ts: &Timestamp) -> Result<String, AgeError> {
    let secs = ts.as_seconds();
    let utc = DateTime::<Utc>::from_timestamp(secs, 0).ok_or(AgeError::OutOfRange(secs))?;
    Ok(utc
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(secs: i64) -> Timestamp {
        Timestamp::new(secs).unwrap()
    }

    #[test]
    fn format_with_age_appends_parenthesized_age() {
        let now = 1_000_000;
        let out = format_with_age(&ts(now - 30), now).unwrap();
        assert!(out.ends_with(" (30s)"), "unexpected suffix in: {out}");
    }
}
