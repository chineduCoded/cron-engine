use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::Tz;

use crate::cron::{
    evaluator::{calendar::Calendar, day::matches_day},
    ir::CronIr,
    scheduler::{
        candidate::{Candidate, advance_numeric},
        field::Field,
        normalize::normalize_next,
    },
    timezone::resolve_local,
};

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

pub fn next_after_tz(ir: &CronIr, dt: DateTime<Tz>) -> Option<DateTime<Tz>> {
    let dt = normalize_next(dt)?;

    let tz = dt.timezone();

    let mut candidate = Candidate::from_datetime(dt);

    loop {
        if candidate.year > ir.max_year() as i32 {
            return None;
        }

        if advance_numeric(&mut candidate, ir, Field::Year) {
            continue;
        }

        if advance_numeric(&mut candidate, ir, Field::Month) {
            continue;
        }

        let calendar = Calendar::from(&candidate);

        if !matches_day(ir, &calendar) {
            candidate.next_valid_day();
            candidate.reset_after(Field::Day, ir);
            continue;
        }

        if advance_numeric(&mut candidate, ir, Field::Hour) {
            continue;
        }

        if advance_numeric(&mut candidate, ir, Field::Minute) {
            continue;
        }

        if advance_numeric(&mut candidate, ir, Field::Second) {
            continue;
        }

        let naive = candidate.to_naive()?;

        let Some(dt) = resolve_local(&tz, naive) else {
            candidate.advance_smallest();
            continue;
        };

        if matches(ir, dt) {
            return Some(dt);
        }

        candidate.second += 1;
    }
}
