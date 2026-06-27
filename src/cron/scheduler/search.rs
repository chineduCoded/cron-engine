use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::Tz;

use crate::cron::{
    evaluator::{
        calendar::Calendar,
        day::matches_day,
    },
    ir::CronIr,
    scheduler::{
        candidate::{Candidate, advance_numeric}, field::Field, normalize::normalize_next,
    }, timezone::resolve_local,
};

pub fn matches(
    ir: &CronIr,
    dt: DateTime<Tz>,
) -> bool {
    let calendar = Calendar::from(&dt);

    ir.second.contains(dt.second())
        && ir.minute.contains(dt.minute())
        && ir.hour.contains(dt.hour())
        && ir.month.contains(dt.month())
        && matches_year(ir, dt.year())
        && matches_day(ir, &calendar)
}

fn matches_year(
    ir: &CronIr,
    year: i32,
) -> bool {
    let Ok(year) = u32::try_from(year) else {
        return false;
    };

    match &ir.year {
        Some(years) => years.contains(year),
        None => true,
    }
}

pub fn next_after(
    ir: &CronIr,
    start: DateTime<Tz>,
) -> Option<DateTime<Tz>> {
    let dt = normalize_next(start)?;

    let tz = dt.timezone();

    let mut candidate = Candidate::from_datetime(dt); 

    loop {
        if candidate.year > ir.max_year() as i32 {
            return None;
        }

        advance_numeric_candidate(ir, &mut candidate);

        let calendar =
            Calendar::from(&candidate);

        if !matches_day(ir, &calendar) {
            candidate.day += 1;
            candidate.reset_after(Field::Day, ir);
            continue;
        }

        let naive = candidate.to_naive()?;

        let dt = resolve_local(&tz, naive)?;

        if matches(ir, dt) {
            return Some(dt);
        }

        candidate.second += 1;
    }
}

fn advance_numeric_candidate(
    ir: &CronIr,
    candidate: &mut Candidate,
) {
    advance_numeric(
        candidate,
        ir,
        Field::Year,
    );

    advance_numeric(
        candidate,
        ir,
        Field::Month,
    );

    advance_numeric(
        candidate,
        ir,
        Field::Hour,
    );

    advance_numeric(
        candidate,
        ir,
        Field::Minute,
    );

    advance_numeric(
        candidate,
        ir,
        Field::Second,
    );
}
