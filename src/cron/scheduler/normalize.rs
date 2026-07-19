use chrono::{DateTime, Duration, Timelike};
use chrono_tz::Tz;

use crate::cron::scheduler::scheduler::Direction;

/// Normalizes the starting instant for schedule navigation.
///
/// Forward searches begin one second after the supplied instant, while
/// backward searches begin one second before it. Nanoseconds are
/// discarded so that scheduling operates at whole-second precision.
pub fn normalize(
    dt: DateTime<Tz>,
    direction: Direction,
) -> Option<DateTime<Tz>> {
    let dt = dt.with_nanosecond(0)?;

    Some(match direction {
        Direction::Forward => dt + Duration::seconds(1),
        Direction::Backward => dt - Duration::seconds(1),
    })
}
