use chrono::DateTime;
use chrono_tz::Tz;

use crate::cron::{
    evaluator::{
        calendar::Calendar, day::matches_day, eval_field::matches
    }, ir::CronIr, scheduler::{
        candidate::{Candidate, navigate_numeric}, 
        field::Field, 
        normalize::normalize, 
        scheduler::Direction,
    }, timezone::resolve_local,
};


pub fn search(
    ir: &CronIr,
    start: DateTime<Tz>,
    direction: Direction,
) -> Option<DateTime<Tz>> {
    let start = normalize(start, direction)?;

    let tz = start.timezone();

    let mut candidate = Candidate::from_datetime(start);

    loop {
        match direction {
            Direction::Forward if candidate.year > ir.max_year() as i32 => {
                return None;
            }

            Direction::Backward if candidate.year < ir.min_year() as i32 => {
                return None;
            }

            _ => {}
        }

        if navigate_numeric(&mut candidate, ir, Field::Year, direction) {
            continue;
        }

        if navigate_numeric(&mut candidate, ir, Field::Month, direction) {
            continue;
        }

        let calendar = Calendar::from(&candidate);

        if !matches_day(ir, &calendar) {
            candidate.move_day(direction);
            candidate.reset_after(Field::Day, ir, direction);
            continue;
        }

        if navigate_numeric(&mut candidate, ir, Field::Hour, direction) {
            continue;
        }

        if navigate_numeric(&mut candidate, ir, Field::Minute, direction) {
            continue;
        }

        if navigate_numeric(&mut candidate, ir, Field::Second, direction) {
            continue;
        }

        let naive = candidate.to_naive()?;

        let Some(dt) = resolve_local(&tz, naive) else {
            candidate.move_smallest(direction);
            continue;
        };

        if matches(ir, dt) {
            return Some(dt);
        }

        candidate.move_smallest(direction);
    }
}
