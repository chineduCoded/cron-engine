use std::str::FromStr;
use std::fmt::Display;

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;

use crate::cron::{
    CronError, compiler::CronCompiler, ir::CronIr, parser::CronParser, scheduler::{
        iterator::CronIterator,
        next,
        prev,
    }
};

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CronSchedule {
    ir: CronIr,
    tz: Tz,
}

impl CronSchedule {
    pub fn new(
        ir: CronIr,
        tz: Tz,
    ) -> Self {
        Self { ir, tz }
    }

    pub fn parse(expr: &str) -> Result<Self, CronError> {
        expr.parse()
    }

    pub fn parse_with_tz(
        expr: &str,
        tz: Tz,
    ) -> Result<Self, CronError> {
        let ast = CronParser::new().parse(expr)?;

        let ir = CronCompiler::compile(ast)?;

        Ok(Self::new(ir, tz))
    }

    pub fn matches(
        &self,
        dt: DateTime<Tz>,
    ) -> bool {
        next::matches(&self.ir, dt)
    }

    pub fn next_after(
        &self,
        after: DateTime<Tz>,
    ) -> Option<DateTime<Tz>> {
        next::next_after_tz(
            &self.ir,
            after,
        )
    }

    pub fn prev_before(
        &self,
        before: DateTime<Tz>,
    ) -> Option<DateTime<Tz>> {
        prev::prev_before_tz(
            &self.ir,
            before,
        )
    }

    pub fn to_local(&self, dt: DateTime<Utc>) -> DateTime<Tz> {
        dt.with_timezone(&self.tz)
    }

    pub fn to_utc(dt: DateTime<Tz>) -> DateTime<Utc> {
        dt.with_timezone(&Utc)
    }

    pub fn upcoming<T>(
        &self,
        start: DateTime<T>,
    ) -> CronIterator
    where 
        T: TimeZone,
        T::Offset: Display,
    {
        let local = start.with_timezone(&self.tz);

        CronIterator::new(
            self.clone(),
            local,
            false,
            Direction::Forward,
        )
    }

    pub fn upcoming_inclusive<T>(
        &self,
        start: DateTime<T>,
    ) -> CronIterator
    where 
        T: TimeZone,
        T::Offset: Display,
    {
        let local = start.with_timezone(&self.tz);

        CronIterator::new(
            self.clone(),
            local,
            true,
            Direction::Forward,
        )
    }

    pub fn previous<T>(
        &self,
        start: DateTime<T>,
    ) -> CronIterator
    where
        T: TimeZone,
        T::Offset: Display,
    {
        let local = start.with_timezone(&self.tz);

        CronIterator::new(
            self.clone(),
            local,
            false,
            Direction::Backward,
        )
    }
}

/*for occurrence in schedule.upcoming(Utc::now()).take(10) {
    println!("{:?}", occurrence?);
}*/

impl FromStr for CronSchedule {
    type Err = CronError;

    fn from_str(expr: &str) -> Result<Self, Self::Err> {
        let ast = CronParser::new().parse(expr)?;

        let ir = CronCompiler::compile(ast)?;

        Ok(Self::new(ir, Tz::UTC))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike, TimeZone};
    use chrono_tz::America::New_York;

    #[test]
    fn crosses_minute_boundary() {
        let schedule = CronSchedule::parse(
            "0 * * * * *"
        ).unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 0, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 6);
        assert_eq!(next.day(), 15);

        assert_eq!(next.hour(), 12);
        assert_eq!(next.minute(), 1);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn wraps_second_to_first_allowed_value() {
        let schedule = CronSchedule::parse("5 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 0, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.minute(), 1);
        assert_eq!(next.second(), 5);
    }

    #[test]
    fn advances_inside_same_minute() {
        let schedule = CronSchedule::parse("20 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 0, 17)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 20);
    }

    #[test]
    fn next_after_is_exclusive() {
        let schedule = CronSchedule::parse("5 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 0, 5)
            .unwrap();

            let next = schedule.next_after(start).unwrap();

            assert_eq!(next.minute(), 1);
            assert_eq!(next.second(), 5);
    }

    #[test]
    fn wildcard_seconds_returns_next_second() {
        let schedule = CronSchedule::parse("* * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 0, 17)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.second(), 18);
    }

    #[test]
    fn second_step_expression() {
        let schedule = CronSchedule::parse("*/15 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 0, 17)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.second(), 30);
    }

    #[test]
    fn second_step_wraps() {
        let schedule = CronSchedule::parse("*/15 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 0, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.minute(), 1);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn crosses_hour_boundary() {
        let schedule = CronSchedule::parse("0 0 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 13);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn advances_minute_inside_same_hour() {
        let schedule = CronSchedule::parse("0 20 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 17, 45)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 12);
        assert_eq!(next.minute(), 20);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn wraps_minute_to_next_hour() {
        let schedule = CronSchedule::parse("0 20 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 59, 30)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 13);
        assert_eq!(next.minute(), 20);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn minute_next_after_is_exclusive() {
        let schedule = CronSchedule::parse("0 20 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 20, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 13);
        assert_eq!(next.minute(), 20);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn wildcard_minute() {
        let schedule = CronSchedule::parse("0 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 17, 15)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 12);
        assert_eq!(next.minute(), 18);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn minute_step_expression() {
        let schedule = CronSchedule::parse("0 */15 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 17, 10)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 12);
        assert_eq!(next.minute(), 30);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn minute_step_wraps() {
        let schedule = CronSchedule::parse("0 */15 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 59, 30)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 13);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn changing_minute_resets_second_to_schedule_minimum() {
        let schedule = CronSchedule::parse("5 20 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 17, 58)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 12);
        assert_eq!(next.minute(), 20);
        assert_eq!(next.second(), 5);
    }

    #[test]
    fn crosses_midnight() {
        let schedule = CronSchedule::parse("0 0 0 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 6);
        assert_eq!(next.day(), 16);

        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn advances_hour_inside_same_day() {
        let schedule = CronSchedule::parse("0 0 15 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 20, 30)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.day(), 15);

        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn wraps_hour_to_next_day() {
        let schedule = CronSchedule::parse("0 0 15 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 23, 10, 50)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.day(), 16);

        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn hour_next_after_is_exclusive() {
        let schedule = CronSchedule::parse("0 0 15 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 15, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.day(), 16);

        assert_eq!(next.hour(), 15);
    }

    #[test]
    fn wildcard_hour() {
        let schedule = CronSchedule::parse("0 0 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 30, 15)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 13);

        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn hour_step_expression() {
        let schedule = CronSchedule::parse("0 0 */6 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 7, 15, 10)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 12);

        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn hour_step_wraps() {
        let schedule = CronSchedule::parse("0 0 */6 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 23, 10, 5)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.day(), 16);

        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn changing_hour_resets_lower_fields() {
        let schedule = CronSchedule::parse("5 10 15 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 15, 12, 40, 50)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 10);
        assert_eq!(next.second(), 5);
    }

    #[test]
    fn midnight_crosses_into_new_month() {
        let schedule = CronSchedule::parse("0 0 0 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 30, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 7);
        assert_eq!(next.day(), 1);

        assert_eq!(next.hour(), 0);
    }

    #[test]
    fn midnight_crosses_into_new_year() {
        let schedule = CronSchedule::parse("0 0 0 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 12, 31, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2026);
        assert_eq!(next.month(), 1);
        assert_eq!(next.day(), 1);

        assert_eq!(next.hour(), 0);
    }

    #[test]
    fn january_to_february() {
        let schedule = CronSchedule::parse("0 0 0 1 * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 31, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 1);

        assert_eq!(next.hour(), 0);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn april_to_may() {
        let schedule = CronSchedule::parse("0 0 0 1 * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 4, 30, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 1);
    }

    #[test]
    fn february_non_leap() {
        let schedule = CronSchedule::parse("0 0 0 1 * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 2, 28, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 3);
        assert_eq!(next.day(), 1);
    }

    #[test]
    fn february_leap_year() {
        let schedule = CronSchedule::parse("0 0 0 1 * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2024, 2, 29, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 3);
        assert_eq!(next.day(), 1);
    }

    #[test]
    fn skips_month_without_day_31() {
        let schedule = CronSchedule::parse("0 0 0 31 * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 4, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 31);
    }

    #[test]
    fn skips_february_day_30() {
        let schedule = CronSchedule::parse("0 0 0 30 * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 2, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 3);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_day_of_month_28() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 2, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 28);
    }

    #[test]
    fn leap_year_last_day() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2024, 2, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.day(), 29);
    }

    #[test]
    fn skips_to_next_allowed_month() {
        let schedule = CronSchedule::parse("0 0 0 1 6 *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 2, 15, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 6);
        assert_eq!(next.day(), 1);
    }

    #[test]
    fn month_wraps_to_next_year() {
        let schedule = CronSchedule::parse("0 0 0 1 2 *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 3, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2026);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 1);
    }

    #[test]
    fn month_change_resets_time() {
        let schedule = CronSchedule::parse("5 10 15 1 * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 30, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 7);
        assert_eq!(next.day(), 1);

        assert_eq!(next.hour(), 15);
        assert_eq!(next.minute(), 10);
        assert_eq!(next.second(), 5);
    }

    #[test]
    fn february_29_skips_non_leap_year() {
        let schedule = CronSchedule::parse("0 0 0 29 2 *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2028);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 29);
    }

    #[test]
    fn last_day_of_month_31() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 1);
        assert_eq!(next.day(), 31);
    }

    #[test]
    fn last_day_of_month_30() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 4, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 4);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_day_of_february_non_leap() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 2, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 28);
    }

    #[test]
    fn last_day_of_february_leap() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2024, 2, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2024);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 29);
    }

    #[test]
    fn weekday_expression_already_weekday() {
        let schedule = CronSchedule::parse("0 0 0 15W * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 4, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.day(), 15);
    }

    #[test]
    fn weekday_moves_forward_from_saturday() {
        // June 1 2024 = Saturday -> Monday 3rd
        let schedule = CronSchedule::parse("0 0 0 1W * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2024, 5, 15, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2024);
        assert_eq!(next.month(), 6);
        assert_eq!(next.day(), 3);
    }

    #[test]
    fn weekday_moves_forward_from_sunday() {
        // September 1 2024 = Sunday -> Monday 2nd
        let schedule = CronSchedule::parse("0 0 0 1W * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2024, 8, 15, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2024);
        assert_eq!(next.month(), 9);
        assert_eq!(next.day(), 2);
    }

    #[test]
    fn weekday_moves_backward_from_month_end() {
        // May 31 2025 = Saturday -> Friday 30th
        let schedule = CronSchedule::parse("0 0 0 31W * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 5, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_weekday_of_month() {
        let schedule = CronSchedule::parse("0 0 0 LW * *").unwrap();
        
        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 5, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_weekday_crosses_month() {
        let schedule = CronSchedule::parse("0 0 0 LW * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 5, 31, 0, 0, 1)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 6);
    }

    #[test]
    fn last_friday_of_month() {
        let schedule = CronSchedule::parse("0 0 0 * * 5L").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 5, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_monday_of_month() {
        let schedule = CronSchedule::parse("0 0 0 * * 1L").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.weekday().num_days_from_sunday(), 1);
    }

    #[test]
    fn third_monday() {
        let schedule = CronSchedule::parse("0 0 0 * * 1#3").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 6);
        assert_eq!(next.day(), 16);
    }

    #[test]
    fn first_sunday() {
        let schedule = CronSchedule::parse("0 0 0 * * 0#1").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 5, 31, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 6);
        assert_eq!(next.day(), 1);
    }

    #[test]
    fn fifth_monday_skips_month_without_occurrence() {
        let schedule = CronSchedule::parse("0 0 0 * * 1#5").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 4, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 6);
    }

    #[test]
    fn regression_february_29() {
        let schedule = CronSchedule::parse("0 0 0 29 2 *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2028);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 29);
    }

    #[test]
    fn regression_day_31_skips_short_months() {
        let schedule = CronSchedule::parse("0 0 0 31 * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 4, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 31);
    }

    #[test]
    fn regression_day_30_skips_february() {
        let schedule = CronSchedule::parse("0 0 0 30 * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 2, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 3);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn regression_year_limit_returns_none() {
        let schedule = CronSchedule::parse("0 0 0 1 1 * 2025").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 1)
            .unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn regression_next_is_always_greater() {
        let schedule = CronSchedule::parse("*/10 * * * * *").unwrap();

        let mut current = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        for _ in 0..1000 {
            let next = schedule.next_after(current).unwrap();
            assert!(next > current);
            current = next;
        }
    }

    #[test]
    fn february_30_returns_none() {
        let schedule = CronSchedule::parse("0 0 0 30 2 *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn february_31_returns_none() {
        let schedule = CronSchedule::parse("0 0 0 31 2 *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn impossible_nth_weekday_with_year_limit() {
        let schedule = CronSchedule::parse("0 0 0 * 2 1#5 2021").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2021, 1, 1, 0, 0, 0)
            .unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn skips_month_without_fifth_monday() {
        let schedule = CronSchedule::parse("0 0 0 * * 1#5").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2021, 2, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        // Should not return any date in February 2021.
        assert!(next.month() != 2 || next.year() != 2021);
    }

    #[test]
    fn returns_none_after_max_year() {
        let schedule = CronSchedule::parse("0 0 0 * * * 2025").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn spring_forward_skips_nonexistent_time() {
        let mut schedule =
            CronSchedule::parse("0 30 2 * * *").unwrap();

        schedule.tz = New_York;

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 3, 9, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 3);
        assert_eq!(next.day(), 10);
        assert_eq!(next.hour(), 2);
        assert_eq!(next.minute(), 30);
    }

    #[test]
    fn hourly_schedule_skips_missing_hour() {
        let mut schedule =
            CronSchedule::parse("0 0 * * * *").unwrap();

        schedule.tz = New_York;

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 3, 9, 1, 30, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 3);
    }

    #[test]
    fn fall_back_returns_first_occurrence() {
        let mut schedule =
            CronSchedule::parse("0 30 1 * * *").unwrap();

        schedule.tz = New_York;

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 11, 2, 0, 0, 0)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 1);
        assert_eq!(next.minute(), 30);
    }

    #[test]
    fn midnight_schedule_not_affected_by_dst() {
        let mut schedule =
            CronSchedule::parse("0 0 0 * * *").unwrap();

        schedule.tz = New_York;

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 3, 8, 23, 59, 59)
            .unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.day(), 9);
        assert_eq!(next.hour(), 0);
    }

    #[test]
    fn iterator_starts_at_first_occurrence() {
        let schedule = CronSchedule::parse("0 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 10, 12, 0, 30)
            .unwrap();

        let mut iter = schedule.upcoming(start);

        let next = iter.next().unwrap().unwrap();

        assert_eq!(next.second(), 0);
        assert_eq!(next.minute(), 1);
    }

    #[test]
    fn iterator_is_monotonic() {
        let schedule = CronSchedule::parse("0 */15 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 10, 0, 0, 0)
            .unwrap();

        let mut iter = schedule.upcoming(start);

        let mut previous = iter.next().unwrap().unwrap();

        for _ in 0..100 {
            let current = iter.next().unwrap().unwrap();

            assert!(current > previous);

            previous = current;
        }
    }

    #[test]
    fn iterator_never_repeats() {
        let schedule = CronSchedule::parse("0 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        let values: Vec<_> = schedule
            .upcoming(start)
            .map(Result::unwrap)
            .take(100)
            .collect();

        let unique: std::collections::HashSet<_> =
            values.iter().cloned().collect();

        assert_eq!(values.len(), unique.len());
    }

    #[test]
    fn every_item_matches_schedule() {
        let schedule =
            CronSchedule::parse("0 */10 9-17 * * MON-FRI").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 1, 0, 0, 0)
            .unwrap();

        for dt in schedule.upcoming(start).take(500) {
            let dt = dt.unwrap();
            assert!(schedule.matches(dt));
        }
    }

    #[test]
    fn hourly_iterator_spacing() {
        let schedule =
            CronSchedule::parse("0 0 * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 1, 0, 0, 0)
            .unwrap();

        let items: Vec<_> = schedule
            .upcoming(start)
            .map(Result::unwrap)
            .take(5)
            .collect();

        assert_eq!(items[1] - items[0], chrono::Duration::hours(1));
        assert_eq!(items[2] - items[1], chrono::Duration::hours(1));
    }

    #[test]
    fn daily_iterator_spacing() {
        let schedule =
            CronSchedule::parse("0 0 0 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 6, 1, 0, 0, 0)
            .unwrap();

        let values: Vec<_> = schedule
            .upcoming(start)
            .map(Result::unwrap)
            .take(10)
            .collect();

        for pair in values.windows(2) {
            assert_eq!(
                pair[1] - pair[0],
                chrono::Duration::days(1),
            );
        }
    }

    #[test]
    fn iterator_crosses_month_boundary() {
        let schedule =
            CronSchedule::parse("0 0 0 * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 31, 23, 59, 59)
            .unwrap();

        let next = schedule.upcoming(start).next().unwrap().unwrap();

        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 1);
    }

    #[test]
    fn iterator_crosses_leap_year() {
        let schedule =
            CronSchedule::parse("0 0 0 29 2 *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        let next = schedule.upcoming(start).next().unwrap().unwrap();

        assert_eq!(next.year(), 2028);
    }

    #[test]
    fn iterator_ends_when_schedule_exhausted() {
        let schedule =
            CronSchedule::parse("0 0 0 * * * 2025").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 12, 31, 23, 59, 59)
            .unwrap();

        let mut iter = schedule.upcoming(start);

        assert!(iter.next().is_none());
    }

    #[test]
    fn iterator_take() {
        let schedule =
            CronSchedule::parse("0 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        assert_eq!(
            schedule.upcoming(start).take(10).count(),
            10,
        );
    }

    #[test]
    fn iterator_nth() {
        let schedule =
            CronSchedule::parse("0 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        let fifth = schedule.upcoming(start).nth(4).unwrap().unwrap();

        assert_eq!(fifth.minute(), 5);
    }

    #[test]
    fn iterator_survives_many_iterations() {
        let schedule =
            CronSchedule::parse("0 * * * * *").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap();

        let mut previous = start;

        for dt in schedule.upcoming(start).take(10_000) {
            let dt = dt.unwrap();
            assert!(dt > previous);
            assert!(schedule.matches(dt));

            previous = dt;
        }
    }
}











