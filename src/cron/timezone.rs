//! Time zone conversion helpers.
//!
//! Utilities in this module resolve local times while correctly handling
//! daylight-saving transitions.

use chrono::{DateTime, NaiveDateTime, TimeZone};

/// Resolves a naive local date-time within a time zone.
///
/// During daylight-saving transitions:
///
/// - A unique local time is returned directly.
/// - Ambiguous times resolve to the earlier occurrence.
/// - Non-existent local times return `None`.
///
/// # Returns
///
/// `None` if the local time does not exist.
pub fn resolve_local<Tz>(tz: &Tz, naive: NaiveDateTime) -> Option<DateTime<Tz>>
where
    Tz: TimeZone,
{
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => Some(dt),

        chrono::LocalResult::Ambiguous(earlier, _) => Some(earlier),

        chrono::LocalResult::None => None,
    }
}
