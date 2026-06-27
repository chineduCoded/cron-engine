use chrono::{
    DateTime, Datelike, Duration
};
use chrono_tz::Tz;

use crate::cron::{
    ir::CronIr,
    scheduler::next::matches,
};

pub fn prev_before_tz(
    ir: &CronIr,
    mut current: DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    let min_year = ir.min_year() as i32;

    while current.year() >= min_year {
        if matches(ir, current) {
            return Some(current);
        }

        current -= Duration::seconds(1);
    }

    None
}
