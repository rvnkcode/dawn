use std::fmt::{self, Display, Formatter};
use thiserror::Error;

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;
const MONTH: i64 = 30 * DAY;
const YEAR: i64 = 365 * DAY;

/// Age format thresholds based on Taskwarrior's `Duration::formatVague`.
/// See: https://github.com/GothenburgBitFactory/libshared/blob/master/src/Duration.cpp
///
/// | Threshold  | Unit          | Example |
/// |------------|---------------|---------|
/// | ≥ 365 days | years (float) | 1.5y    |
/// | ≥ 90 days  | months        | 4mo     |
/// | ≥ 14 days  | weeks         | 3w      |
/// | ≥ 1 day    | days          | 5d      |
/// | ≥ 1 hour   | hours         | 8h      |
/// | ≥ 1 minute | minutes       | 30min   |
/// | ≥ 1 second | seconds       | 45s     |
///
/// (threshold, divisor, unit)
const FORMATS: &[(i64, i64, &str)] = &[
    (3 * MONTH, MONTH, "mo"), // TODO: 3달을 좀 더 정확하게 비교
    (2 * WEEK, WEEK, "w"),
    (DAY, DAY, "d"),
    (HOUR, HOUR, "h"),
    (MINUTE, MINUTE, "min"),
    (0, 1, "s"),
];

#[derive(Debug, PartialEq, Error)]
pub enum AgeError {
    #[error("Invalid age delta: {0}")]
    InvalidDelta(i64),
}

#[derive(Debug)]
pub struct Age(String);

impl Age {
    pub fn new(created_at: &i64, now: &i64) -> Result<Self, AgeError> {
        let delta = now - created_at;

        // TODO: Change format like 1y2mo
        let age_str = if delta >= YEAR {
            format!("{:.1}y", delta as f64 / YEAR as f64)
        } else {
            FORMATS
                .iter()
                .find(|(threshold, _, _)| delta >= *threshold)
                .map(|(_, divisor, unit)| format!("{}{}", delta / divisor, unit))
                .ok_or(AgeError::InvalidDelta(delta))?
        };
        Ok(Age(age_str))
    }
}

impl Display for Age {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_age_seconds() {
        let now = 1_000_000;
        let age = Age::new(&(now - 30), &now).unwrap();
        assert_eq!(age.to_string(), "30s");
    }

    #[test]
    fn test_age_minutes() {
        let now = 1_000_000;
        let age = Age::new(&(now - 3 * MINUTE), &now).unwrap();
        assert_eq!(age.to_string(), "3min");
    }

    #[test]
    fn test_age_hours() {
        let now = 1_000_000;
        let age = Age::new(&(now - 5 * HOUR), &now).unwrap();
        assert_eq!(age.to_string(), "5h");
    }

    #[test]
    fn test_age_days() {
        let now = 1_000_000;
        let age = Age::new(&(now - 10 * DAY), &now).unwrap();
        assert_eq!(age.to_string(), "10d");
    }

    #[test]
    fn test_age_weeks() {
        let now = 100_000_000;
        let age = Age::new(&(now - 3 * WEEK), &now).unwrap();
        assert_eq!(age.to_string(), "3w");

        let age_11w = Age::new(&(now - 11 * WEEK), &now).unwrap();
        assert_eq!(age_11w.to_string(), "11w");
    }

    #[test]
    fn test_age_months() {
        let now = 100_000_000;
        let age = Age::new(&(now - 120 * DAY), &now).unwrap();
        assert_eq!(age.to_string(), "4mo");
    }

    #[test]
    fn test_age_years() {
        let now = 100_000_000;
        let age = Age::new(&(now - 400 * DAY), &now).unwrap();
        assert_eq!(age.to_string(), "1.1y");

        let age_2y = Age::new(&(now - 730 * DAY), &now).unwrap();
        assert_eq!(age_2y.to_string(), "2.0y");
    }

    #[test]
    fn test_age_zero_delta() {
        let now = 1_000_000;
        let age = Age::new(&now, &now).unwrap();
        assert_eq!(age.to_string(), "0s");
    }

    #[test]
    fn test_age_invalid_delta() {
        let now = 1_000_000;
        let result = Age::new(&(now + 100), &now);
        assert_eq!(result.unwrap_err(), AgeError::InvalidDelta(-100));
    }
}
