use chrono::{DateTime, Duration, Timelike};
use chrono_tz::Tz;

pub fn normalize_next(dt: DateTime<Tz>) -> Option<DateTime<Tz>> {
    Some(dt.with_nanosecond(0)? + Duration::seconds(1))
}
