use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::Tz;

use crate::cron::{evaluator::{calendar::Calendar, day::matches_day}, ir::CronIr};

/// Returns whether the given date-time satisfies the compiled schedule.
///
/// Every numeric field together with the calendar-based day rules must
/// match for the schedule to be considered satisfied.
pub fn matches(ir: &CronIr, dt: DateTime<Tz>) -> bool {
    let calendar = Calendar::from(&dt);

    ir.second.contains(dt.second())
        && ir.minute.contains(dt.minute())
        && ir.hour.contains(dt.hour())
        && ir.month.contains(dt.month())
        && matches_year(ir, dt.year())
        && matches_day(ir, &calendar)
}

fn matches_year(ir: &CronIr, year: i32) -> bool {
    let Ok(year) = u32::try_from(year) else {
        return false;
    };

    match &ir.year {
        Some(years) => years.contains(year),
        None => true,
    }
}

