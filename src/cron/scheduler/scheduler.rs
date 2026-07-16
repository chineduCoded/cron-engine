use std::fmt::Display;
use std::str::FromStr;

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use proptest::prelude::*;

use crate::cron::{
    CronError,
    compiler::CronCompiler,
    ir::CronIr,
    parser::CronParser,
    scheduler::{iterator::CronIterator, next, prev},
};

/// Direction used by scheduler navigation.
///
/// This determines whether the scheduler searches for occurrences forward
/// or backward in time.
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Search for occurrences after the current datetime.
    Forward,

    /// Search for occurences before the current datetime.
    Backward,
}

/// Immutable compiled cron schedule.
///
/// A `CronSchedule` stores the compiled representation of a cron
/// expression and provides efficient occurrence calculations.
///
/// Instances are immutable and therefore cheap to share across
/// threads.
///
/// # Example
///
/// ```
/// use cron_engine::CronSchedule;
///
/// let schedule = CronSchedule::parse("0 */5 * * * *")?;
/// # Ok::<(), cron_engine::CronError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CronSchedule {
    /// Compiled intermediate representation of the cron expression.
    ///
    /// This contains the optimized field matchers and day rules used by the
    /// scheduler. It is produced during parsing and compilation and is reused
    /// for every scheduling operation.
    ir: CronIr,

    /// Timezone used when evaluating this schedule.
    ///
    /// All occurrence calculations, matching operations, and iterators are
    /// performed relative to this timezone.
    tz: Tz,
}

impl CronSchedule {
    /// Returns the compiled intermediate representation.
    #[inline]
    pub const fn ir(&self) -> &CronIr {
        &self.ir
    }

    /// Returns the schedule timezone.
    #[inline]
    pub const fn timezone(&self) -> Tz {
        self.tz
    }
    /// Creates a schedule from a compiled [`CronIr`] and timezone.
    ///
    /// Most users should prefer [`CronSchedule::parse`], which performs parsing
    /// and compilation automatically.
    ///
    /// This constructor is primarily intended for advanced use cases where a
    /// compiled schedule is already available.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let ir = ...;
    /// let schedule = CronSchedule::new(ir, chrono_tz::UTC);
    /// ```
    pub fn new(ir: CronIr, tz: Tz) -> Self {
        Self { ir, tz }
    }

    /// Parses a cron expression into a [`CronSchedule`].
    ///
    /// This is the primary entry point for creating schedules. The expression
    /// is parsed, validated, compiled into an immutable intermediate
    /// representation, and configured to use the UTC timezone.
    ///
    /// This is equivalent to calling
    /// [`CronSchedule::parse_with_tz`] with [`chrono_tz::UTC`].
    ///
    /// Supported syntax includes:
    ///
    /// - Wildcards (`*`)
    /// - Lists (`1,2,3`)
    /// - Ranges (`1-5`)
    /// - Steps (`*/15`)
    /// - Named months (`JAN`, `FEB`, …)
    /// - Named weekdays (`MON`, `TUE`, …)
    /// - Quartz extensions (`L`, `LW`, `W`, `#`)
    ///
    /// # Examples
    ///
    /// ```
    /// use cron_engine::CronSchedule;
    ///
    /// let schedule = CronSchedule::parse("0 */5 * * * *")?;
    ///
    /// # Ok::<(), cron_engine::CronError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`CronError`] if the expression is invalid.
    pub fn parse(expr: &str) -> Result<Self, CronError> {
        expr.parse()
    }

    /// Parses a cron expression using the specified timezone.
    ///
    /// The returned [`CronSchedule`] evaluates all matching, iteration, and
    /// occurrence calculations relative to `tz`.
    ///
    /// This is useful when scheduling jobs for a timezone other than UTC.
    ///
    /// # Arguments
    ///
    /// * `expr` - The cron expression to parse.
    /// * `tz` - The timezone used for schedule evaluation.
    ///
    /// # Errors
    ///
    /// Returns [`CronError`] if the expression is syntactically invalid or
    /// contains unsupported or invalid field values.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono_tz::America::New_York;
    /// use cron_engine::CronSchedule;
    ///
    /// let schedule = CronSchedule::parse_with_tz(
    ///     "0 0 9 * * MON-FRI",
    ///     New_York,
    /// )?;
    ///
    /// assert_eq!(schedule.timezone(), New_York);
    ///
    /// # Ok::<(), cron_engine::CronError>(())
    /// ```
    pub fn parse_with_tz(expr: &str, tz: Tz) -> Result<Self, CronError> {
        let ast = CronParser::new().parse(expr)?;

        let ir = CronCompiler::compile(ast)?;

        Ok(Self::new(ir, tz))
    }

    /// Returns `true` if the given datetime satisfies this schedule.
    ///
    /// The datetime is evaluated using the schedule's configured timezone.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::TimeZone;
    /// use chrono_tz::UTC;
    /// use cron_engine::CronSchedule;
    ///
    /// let schedule = CronSchedule::parse("0 * * * * *")?;
    ///
    /// let dt = UTC.with_ymd_and_hms(2025, 1, 1, 12, 30, 0).unwrap();
    ///
    /// assert!(schedule.matches(dt));
    ///
    /// # Ok::<(), cron_engine::CronError>(())
    /// ```
    pub fn matches(&self, dt: DateTime<Tz>) -> bool {
        next::matches(&self.ir, dt)
    }

    /// Returns the first occurrence strictly after `datetime`.
    ///
    /// The supplied datetime itself is never returned, even if it matches the
    /// schedule.
    ///
    /// Returns `None` if no future occurrence exists, for example when the
    /// schedule is restricted to past years.
    ///
    /// This method correctly handles:
    ///
    /// - leap years
    /// - month boundaries
    /// - daylight saving transitions
    /// - Quartz day rules (`L`, `LW`, `W`, `#`)
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{TimeZone, Timelike};
    /// use chrono_tz::UTC;
    /// use cron_engine::CronSchedule;
    ///
    /// let schedule = CronSchedule::parse("0 */5 * * * *")?;
    ///
    /// let start = UTC.with_ymd_and_hms(2025,1,1,0,0,0).unwrap();
    ///
    /// let next = schedule.next_after(start).unwrap();
    ///
    /// assert_eq!(next.minute(), 5);
    ///
    /// # Ok::<(), cron_engine::CronError>(())
    /// ```
    ///
    /// # Returns
    ///
    /// `None` if no valid future occurrence exists.
    pub fn next_after(&self, after: DateTime<Tz>) -> Option<DateTime<Tz>> {
        next::next_after_tz(&self.ir, after)
    }

    /// Returns the last occurrence strictly before `datetime`.
    ///
    /// The supplied datetime itself is never returned.
    ///
    /// Returns `None` if no earlier occurrence exists.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let previous = schedule.previous_before(now);
    /// ```
    pub fn prev_before(&self, before: DateTime<Tz>) -> Option<DateTime<Tz>> {
        prev::prev_before_tz(&self.ir, before)
    }

    /// Converts a UTC datetime into this schedule's configured timezone.
    ///
    /// This changes only the timezone representation of the datetime; it does not
    /// change the underlying instant in time.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{TimeZone, Timelike};
    /// use chrono::Utc;
    /// use chrono_tz::America::New_York;
    /// use cron_engine::CronSchedule;
    ///
    /// let schedule = CronSchedule::parse_with_tz(
    ///     "* * * * * *",
    ///     New_York,
    /// )?;
    ///
    /// let utc = Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
    ///
    /// let local = schedule.to_local(utc);
    ///
    /// assert_eq!(local.timezone(), New_York);
    ///
    /// # Ok::<(), cron_engine::CronError>(())
    /// ```
    pub fn to_local(&self, dt: DateTime<Utc>) -> DateTime<Tz> {
        dt.with_timezone(&self.tz)
    }

    /// Converts a timezone-aware datetime into UTC.
    ///
    /// This changes only the timezone representation of the datetime; it does not
    /// change the underlying instant in time.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::TimeZone;
    /// use chrono_tz::Europe::London;
    /// use cron_engine::CronSchedule;
    ///
    /// let local = London
    ///     .with_ymd_and_hms(2025, 6, 1, 12, 0, 0)
    ///     .unwrap();
    ///
    /// let utc = CronSchedule::to_utc(local);
    ///
    /// assert_eq!(utc.timezone(), chrono::Utc);
    /// ```
    pub fn to_utc(dt: DateTime<Tz>) -> DateTime<Utc> {
        dt.with_timezone(&Utc)
    }

    /// Returns an iterator over future occurrences.
    ///
    /// The iterator yields occurrences strictly after the supplied datetime.
    ///
    /// The iterator performs lazy scheduling and computes each occurrence on
    /// demand.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::TimeZone;
    /// use chrono_tz::UTC;
    /// use cron_engine::CronSchedule;
    ///
    /// let schedule = CronSchedule::parse("0 * * * * *")?;
    ///
    /// let now = UTC.with_ymd_and_hms(2025,1,1,0,0,0).unwrap();
    ///
    /// let mut iter = schedule.upcoming(now);
    ///
    /// assert!(iter.next().is_some());
    ///
    /// # Ok::<(), cron_engine::CronError>(())
    /// ```
    pub fn upcoming<T>(&self, start: DateTime<T>) -> CronIterator
    where
        T: TimeZone,
        T::Offset: Display,
    {
        let local = start.with_timezone(&self.tz);

        CronIterator::new(self.clone(), local, false, Direction::Forward)
    }

    /// Returns an iterator beginning at `datetime`.
    ///
    /// If `datetime` already satisfies the schedule, it is yielded as the first
    /// occurrence. Otherwise, iteration begins with the next matching time.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let iter = schedule.upcoming_inclusive(now);
    /// ```
    pub fn upcoming_inclusive<T>(&self, start: DateTime<T>) -> CronIterator
    where
        T: TimeZone,
        T::Offset: Display,
    {
        let local = start.with_timezone(&self.tz);

        CronIterator::new(self.clone(), local, true, Direction::Forward)
    }

    /// Returns an iterator over previous occurrences.
    ///
    /// The iterator lazily computes matching datetimes before the supplied
    /// starting point.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut iter = schedule.previous(now);
    ///
    /// while let Some(dt) = iter.next() {
    ///     println!("{dt}");
    /// }
    /// ```
    pub fn previous<T>(&self, start: DateTime<T>) -> CronIterator
    where
        T: TimeZone,
        T::Offset: Display,
    {
        let local = start.with_timezone(&self.tz);

        CronIterator::new(self.clone(), local, false, Direction::Backward)
    }
}

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
    use chrono::{Datelike, TimeZone, Timelike};
    use chrono_tz::America::New_York;

    #[test]
    fn crosses_minute_boundary() {
        let schedule = CronSchedule::parse("0 * * * * *").unwrap();

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

        let start = schedule.tz.with_ymd_and_hms(2025, 6, 15, 12, 0, 5).unwrap();

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

        let start = schedule.tz.with_ymd_and_hms(2025, 6, 15, 15, 0, 0).unwrap();

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

        let start = schedule.tz.with_ymd_and_hms(2025, 4, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 31);
    }

    #[test]
    fn skips_february_day_30() {
        let schedule = CronSchedule::parse("0 0 0 30 * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 3);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_day_of_month_28() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 28);
    }

    #[test]
    fn leap_year_last_day() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.day(), 29);
    }

    #[test]
    fn skips_to_next_allowed_month() {
        let schedule = CronSchedule::parse("0 0 0 1 6 *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 2, 15, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 6);
        assert_eq!(next.day(), 1);
    }

    #[test]
    fn month_wraps_to_next_year() {
        let schedule = CronSchedule::parse("0 0 0 1 2 *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap();

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

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2028);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 29);
    }

    #[test]
    fn last_day_of_month_31() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 1);
        assert_eq!(next.day(), 31);
    }

    #[test]
    fn last_day_of_month_30() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 4, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 4);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_day_of_february_non_leap() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 28);
    }

    #[test]
    fn last_day_of_february_leap() {
        let schedule = CronSchedule::parse("0 0 0 L * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2024);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 29);
    }

    #[test]
    fn weekday_expression_already_weekday() {
        let schedule = CronSchedule::parse("0 0 0 15W * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 4, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.day(), 15);
    }

    #[test]
    fn weekday_moves_forward_from_saturday() {
        // June 1 2024 = Saturday -> Monday 3rd
        let schedule = CronSchedule::parse("0 0 0 1W * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2024, 5, 15, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2024);
        assert_eq!(next.month(), 6);
        assert_eq!(next.day(), 3);
    }

    #[test]
    fn weekday_moves_forward_from_sunday() {
        // September 1 2024 = Sunday -> Monday 2nd
        let schedule = CronSchedule::parse("0 0 0 1W * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2024, 8, 15, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2024);
        assert_eq!(next.month(), 9);
        assert_eq!(next.day(), 2);
    }

    #[test]
    fn weekday_moves_backward_from_month_end() {
        // May 31 2025 = Saturday -> Friday 30th
        let schedule = CronSchedule::parse("0 0 0 31W * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 5, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_weekday_of_month() {
        let schedule = CronSchedule::parse("0 0 0 LW * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 5, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_weekday_crosses_month() {
        let schedule = CronSchedule::parse("0 0 0 LW * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 5, 31, 0, 0, 1).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 6);
    }

    #[test]
    fn last_friday_of_month() {
        let schedule = CronSchedule::parse("0 0 0 * * 5L").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 5, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn last_monday_of_month() {
        let schedule = CronSchedule::parse("0 0 0 * * 1L").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.weekday().num_days_from_sunday(), 1);
    }

    #[test]
    fn third_monday() {
        let schedule = CronSchedule::parse("0 0 0 * * 1#3").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();

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

        let start = schedule.tz.with_ymd_and_hms(2025, 4, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.month(), 6);
    }

    #[test]
    fn regression_february_29() {
        let schedule = CronSchedule::parse("0 0 0 29 2 *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2028);
        assert_eq!(next.month(), 2);
        assert_eq!(next.day(), 29);
    }

    #[test]
    fn regression_day_31_skips_short_months() {
        let schedule = CronSchedule::parse("0 0 0 31 * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 4, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 5);
        assert_eq!(next.day(), 31);
    }

    #[test]
    fn regression_day_30_skips_february() {
        let schedule = CronSchedule::parse("0 0 0 30 * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 3);
        assert_eq!(next.day(), 30);
    }

    #[test]
    fn regression_year_limit_returns_none() {
        let schedule = CronSchedule::parse("0 0 0 1 1 * 2025").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 1).unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn regression_next_is_always_greater() {
        let schedule = CronSchedule::parse("*/10 * * * * *").unwrap();

        let mut current = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        for _ in 0..1000 {
            let next = schedule.next_after(current).unwrap();
            assert!(next > current);
            current = next;
        }
    }

    #[test]
    fn february_30_returns_none() {
        let schedule = CronSchedule::parse("0 0 0 30 2 *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn february_31_returns_none() {
        let schedule = CronSchedule::parse("0 0 0 31 2 *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn impossible_nth_weekday_with_year_limit() {
        let schedule = CronSchedule::parse("0 0 0 * 2 1#5 2021").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn skips_month_without_fifth_monday() {
        let schedule = CronSchedule::parse("0 0 0 * * 1#5").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2021, 2, 1, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        // Should not return any date in February 2021.
        assert!(next.month() != 2 || next.year() != 2021);
    }

    #[test]
    fn returns_none_after_max_year() {
        let schedule = CronSchedule::parse("0 0 0 * * * 2025").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

        assert!(schedule.next_after(start).is_none());
    }

    #[test]
    fn spring_forward_skips_nonexistent_time() {
        let mut schedule = CronSchedule::parse("0 30 2 * * *").unwrap();

        schedule.tz = New_York;

        let start = schedule.tz.with_ymd_and_hms(2025, 3, 9, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.year(), 2025);
        assert_eq!(next.month(), 3);
        assert_eq!(next.day(), 10);
        assert_eq!(next.hour(), 2);
        assert_eq!(next.minute(), 30);
    }

    #[test]
    fn hourly_schedule_skips_missing_hour() {
        let mut schedule = CronSchedule::parse("0 0 * * * *").unwrap();

        schedule.tz = New_York;

        let start = schedule.tz.with_ymd_and_hms(2025, 3, 9, 1, 30, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 3);
    }

    #[test]
    fn fall_back_returns_first_occurrence() {
        let mut schedule = CronSchedule::parse("0 30 1 * * *").unwrap();

        schedule.tz = New_York;

        let start = schedule.tz.with_ymd_and_hms(2025, 11, 2, 0, 0, 0).unwrap();

        let next = schedule.next_after(start).unwrap();

        assert_eq!(next.hour(), 1);
        assert_eq!(next.minute(), 30);
    }

    #[test]
    fn midnight_schedule_not_affected_by_dst() {
        let mut schedule = CronSchedule::parse("0 0 0 * * *").unwrap();

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

        let start = schedule.tz.with_ymd_and_hms(2025, 6, 10, 0, 0, 0).unwrap();

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

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let values: Vec<_> = schedule
            .upcoming(start)
            .map(Result::unwrap)
            .take(100)
            .collect();

        let unique: std::collections::HashSet<_> = values.iter().cloned().collect();

        assert_eq!(values.len(), unique.len());
    }

    #[test]
    fn every_item_matches_schedule() {
        let schedule = CronSchedule::parse("0 */10 9-17 * * MON-FRI").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();

        for dt in schedule.upcoming(start).take(500) {
            let dt = dt.unwrap();
            assert!(schedule.matches(dt));
        }
    }

    #[test]
    fn hourly_iterator_spacing() {
        let schedule = CronSchedule::parse("0 0 * * * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();

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
        let schedule = CronSchedule::parse("0 0 0 * * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();

        let values: Vec<_> = schedule
            .upcoming(start)
            .map(Result::unwrap)
            .take(10)
            .collect();

        for pair in values.windows(2) {
            assert_eq!(pair[1] - pair[0], chrono::Duration::days(1),);
        }
    }

    #[test]
    fn iterator_crosses_month_boundary() {
        let schedule = CronSchedule::parse("0 0 0 * * *").unwrap();

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
        let schedule = CronSchedule::parse("0 0 0 29 2 *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let next = schedule.upcoming(start).next().unwrap().unwrap();

        assert_eq!(next.year(), 2028);
    }

    #[test]
    fn iterator_ends_when_schedule_exhausted() {
        let schedule = CronSchedule::parse("0 0 0 * * * 2025").unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(2025, 12, 31, 23, 59, 59)
            .unwrap();

        let mut iter = schedule.upcoming(start);

        assert!(iter.next().is_none());
    }

    #[test]
    fn iterator_take() {
        let schedule = CronSchedule::parse("0 * * * * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        assert_eq!(schedule.upcoming(start).take(10).count(), 10,);
    }

    #[test]
    fn iterator_nth() {
        let schedule = CronSchedule::parse("0 * * * * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let fifth = schedule.upcoming(start).nth(4).unwrap().unwrap();

        assert_eq!(fifth.minute(), 5);
    }

    #[test]
    fn iterator_survives_many_iterations() {
        let schedule = CronSchedule::parse("0 * * * * *").unwrap();

        let start = schedule.tz.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let mut previous = start;

        for dt in schedule.upcoming(start).take(10_000) {
            let dt = dt.unwrap();
            assert!(dt > previous);
            assert!(schedule.matches(dt));

            previous = dt;
        }
    }
}

proptest! {

    #[test]
    fn next_after_always_matches(
        year in 2025i32..2028,
        month in 1u32..12,
        day in 1u32..28,
        hour in 0u32..23,
        minute in 0u32..59,
        second in 0u32..59,
    ) {

        let schedule =
            CronSchedule::parse("0 */10 * * * *")
                .unwrap();

        let start = schedule
            .tz
            .with_ymd_and_hms(
                year,
                month,
                day,
                hour,
                minute,
                second,
            )
            .unwrap();

        let next =
            schedule.next_after(start)
                .unwrap();

        prop_assert!(
            schedule.matches(next)
        );

        prop_assert!(next > start);
    }
}
