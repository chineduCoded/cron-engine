use chrono::{DateTime, Duration, Timelike};
use chrono_tz::Tz;

/// Normalizes a starting instant for forward schedule searches. 
///
/// Nanoseconds are discarded and the search advances by one second so 
/// that `next_after()` returns occurrences strictly after the supplied 
/// instant.
pub fn normalize_next(dt: DateTime<Tz>) -> Option<DateTime<Tz>> {
    Some(dt.with_nanosecond(0)? + Duration::seconds(1))
}
