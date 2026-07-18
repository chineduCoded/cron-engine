use chrono::{DateTime, Datelike, Duration};
use chrono_tz::Tz;

use crate::cron::{ir::CronIr, scheduler::next::matches};

/// Finds the last occurrence at or before the supplied date-time. 
///
/// The search proceeds backwards one second at a time until a matching 
/// occurrence is found or the minimum supported year is reached. 
///
/// Returns `None` if no previous occurrence exists.
pub fn prev_before_tz(ir: &CronIr, mut current: DateTime<Tz>) -> Option<DateTime<Tz>> {
    let min_year = ir.min_year() as i32;

    while current.year() >= min_year {
        if matches(ir, current) {
            return Some(current);
        }

        current -= Duration::seconds(1);
    }

    None
}
