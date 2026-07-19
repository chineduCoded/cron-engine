use chrono::DateTime;
use chrono_tz::Tz;

use crate::cron::{ 
    ir::CronIr, 
    scheduler::{
        scheduler::Direction, 
        search::search,
    }
};

/// Finds the first occurrence strictly after the supplied instant.
pub fn next_after_tz(
    ir: &CronIr,
    dt: DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    search(ir, dt, Direction::Forward)
}
