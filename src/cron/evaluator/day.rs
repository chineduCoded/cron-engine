use crate::cron::{evaluator::calendar::{Calendar, Weekday}, ir::{DayRule, CronIr}};


pub fn evaluate_dom_rule(
    rule: &DayRule,
    calendar: &Calendar,
) -> bool {
    match rule {
        DayRule::Any => true,

        DayRule::Bits(bits) => {
            bits.contains(calendar.day)
        }

        DayRule::LastDay => {
            calendar.is_last_day()
        }

        DayRule::NearestWeekday(day) => {
            calendar.day
                == Calendar::nearest_weekday(
                    calendar.year,
                    calendar.month,
                    *day,
                )
        }

        DayRule::List(rules) => {
            rules.iter()
                .any(|rule| evaluate_dom_rule(rule, calendar))
        }

        _ => false,
    }
}

pub fn evaluate_dow_rule(
    rule: &DayRule,
    calendar: &Calendar,
) -> bool {
    match rule {
        DayRule::Any => true,

        DayRule::Bits(bits) => {
            bits.contains(calendar.weekday().0 as u32)
        }

        DayRule::LastWeekday(weekday) => {
            let last = Calendar::last_weekday(
                calendar.year,
                calendar.month,
                Weekday::try_from(*weekday).unwrap(),
            );

            calendar.day == last
        }

        DayRule::NthWeekday {
            weekday,
            nth,
        } => {
            Calendar::nth_weekday(
                calendar.year,
                calendar.month,
                *weekday,
                *nth,
            ) == Some(calendar.day)
        }

        DayRule::List(rules) => {
            rules.iter()
                .any(|rule| evaluate_dow_rule(rule, calendar))
        }

        _ => false,
    }
}

fn is_any(rule: &DayRule) -> bool {
    matches!(rule, DayRule::Any)
}

pub fn matches_day(
    ir: &CronIr,
    calendar: &Calendar,
) -> bool {
    let dom =
        evaluate_dom_rule(
            &ir.day_of_month,
            calendar,
        );

    let dow =
        evaluate_dow_rule(
            &ir.day_of_week,
            calendar,
        );

    match (
        is_any(&ir.day_of_month),
        is_any(&ir.day_of_week),
    ) {
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
