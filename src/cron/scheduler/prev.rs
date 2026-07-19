use chrono::DateTime;
use chrono_tz::Tz;

use crate::cron::{ir::CronIr, scheduler::{scheduler::Direction, search::search}};

/// Finds the last occurrence strictly before the supplied instant.
pub fn prev_before_tz(
    ir: &CronIr,
    dt: DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    search(ir, dt, Direction::Backward)
}
