//! Evaluation of Quartz day-of-month and day-of-week rules.
//!
//! This module implements the semantics of advanced Quartz expressions,
//! including:
//!
//! - `L`
//! - `LW`
//! - `W`
//! - `5L`
//! - `MON#2`
//!
//! It also combines the day-of-month and day-of-week fields using either
//! logical OR or logical AND, depending on the compiled schedule.

use crate::cron::{
    evaluator::calendar::{Calendar, Weekday},
    ir::{CronIr, DayRule},
};

/// Evaluates a day-of-month rule against a calendar date.
///
/// This function implements Quartz day-of-month semantics, including:
///
/// - Exact day matches
/// - `L` (last day of month)
/// - `LW` (last business day)
/// - `W` (nearest weekday)
/// - Lists of day-of-month rules
///
/// Rules that belong exclusively to the day-of-week field (such as `5L`
/// or `MON#2`) always evaluate to `false`.
pub fn evaluate_dom_rule(rule: &DayRule, calendar: &Calendar) -> bool {
    match rule {
        DayRule::Any => true,

        DayRule::Bits(bits) => bits.contains(calendar.day),

        DayRule::LastDay => calendar.is_last_day(),

        DayRule::LastBusinessDay => {
            calendar.day == Calendar::last_business_day(calendar.year, calendar.month)
        }

        DayRule::NearestWeekday(day) => {
            calendar.day == Calendar::nearest_weekday(calendar.year, calendar.month, *day)
        }

        DayRule::List(rules) => rules.iter().any(|rule| evaluate_dom_rule(rule, calendar)),

        _ => false,
    }
}

/// Evaluates a day-of-week rule against a calendar date.
///
/// This function implements Quartz day-of-week semantics, including:
///
/// - Exact weekday matches
/// - `5L` (last Friday of the month)
/// - `MON#2` (second Monday of the month)
/// - Lists of weekday rules
///
/// Rules that belong exclusively to the day-of-month field (such as
/// `L`, `LW`, or `15W`) always evaluate to `false`.
pub fn evaluate_dow_rule(rule: &DayRule, calendar: &Calendar) -> bool {
    match rule {
        DayRule::Any => true,

        DayRule::Bits(bits) => bits.contains(calendar.weekday().0 as u32),

        DayRule::LastWeekday(weekday) => {
            let last = Calendar::last_weekday(
                calendar.year,
                calendar.month,
                Weekday::try_from(*weekday).unwrap(),
            );

            calendar.day == last
        }

        DayRule::NthWeekday { weekday, nth } => {
            Calendar::nth_weekday(calendar.year, calendar.month, *weekday, *nth)
                == Some(calendar.day)
        }

        DayRule::List(rules) => rules.iter().any(|rule| evaluate_dow_rule(rule, calendar)),

        _ => false,
    }
}

fn is_any(rule: &DayRule) -> bool {
    matches!(rule, DayRule::Any)
}

/// Evaluates whether a calendar date satisfies the schedule's day rules.
///
/// Cron expressions contain two independent day fields:
///
/// - day of month (DOM)
/// - day of week (DOW)
///
/// If only one field is constrained, that field determines the result.
///
/// If both fields are constrained, the result is combined using either
/// logical **OR** or logical **AND**, depending on the schedule's
/// `dom_dow_or` setting.
///
/// This function implements Quartz-compatible day matching semantics.
pub fn matches_day(ir: &CronIr, calendar: &Calendar) -> bool {
    let dom = evaluate_dom_rule(&ir.day_of_month, calendar);

    let dow = evaluate_dow_rule(&ir.day_of_week, calendar);

    match (is_any(&ir.day_of_month), is_any(&ir.day_of_week)) {
        (true, true) => true,

        (false, true) => dom,

        (true, false) => dow,

        (false, false) => {
            if ir.dom_dow_or {
                dom || dow
            } else {
                dom && dow
            }
        }
    }
}
