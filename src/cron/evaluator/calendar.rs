//! Calendar computations independent of chrono.
//!
//! Implements:
//!
//! - leap year detection
//! - weekday calculation
//! - month lengths
//! - nearest weekday (`W`)
//! - last weekday (`5L`)
//! - nth weekday (`#`)
//! - last business day (`LW`)

use chrono::{DateTime, Datelike, TimeZone};
use proptest::prelude::*;

use crate::cron::scheduler::candidate::Candidate;

const MONTH_OFFSETS: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];

const MONTH_LENGTHS: [[u8; 12]; 2] = [
    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
];

/// Day of the week.
///
/// Weekdays are represented using Quartz and POSIX numbering:
///
/// | Value | Day |
/// |------:|-----|
/// | 0 | Sunday |
/// | 1 | Monday |
/// | 2 | Tuesday |
/// | 3 | Wednesday |
/// | 4 | Thursday |
/// | 5 | Friday |
/// | 6 | Saturday |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Weekday(pub u8);

impl Weekday {
    /// Creates a weekday.
    ///
    /// # Panics
    ///
    /// Panics if `value` is greater than `6`.
    #[inline]
    pub const fn new(value: u8) -> Self {
        assert!(value <= 6);
        Self(value)
    }

    /// Creates a weekday by wrapping the value into the range `0..=6`.
    ///
    /// This is equivalent to `value % 7`.
    pub const fn wrapping(value: u8) -> Self {
        Self(value % 7)
    }

    /// Returns the numeric representation of this weekday.
    #[inline]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Returns `true` if this is Monday through Friday.
    #[inline]
    pub const fn is_weekday(self) -> bool {
        matches!(self.0, 1..=5)
    }

    /// Returns `true` if this is Saturday or Sunday.
    #[inline]
    pub const fn is_weekend(self) -> bool {
        !self.is_weekday()
    }

    /// Returns the next weekday, wrapping from Saturday to Sunday.
    #[inline]
    pub const fn next(self) -> Self {
        Self((self.0 + 1) % 7)
    }
}

impl TryFrom<u32> for Weekday {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value <= 6 {
            Ok(Self(value as u8))
        } else {
            Err("weekday must be in range 0..=6")
        }
    }
}

impl TryFrom<u8> for Weekday {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 6 {
            Ok(Self(value))
        } else {
            Err("weekday must be in range 0..=6")
        }
    }
}

impl TryFrom<i32> for Weekday {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0..=6 => Ok(Self(value as u8)),
            _ => Err("weekday must be in range 0..=6"),
        }
    }
}

impl From<Weekday> for u8 {
    fn from(day: Weekday) -> Self {
        day.0
    }
}

impl From<Weekday> for u32 {
    fn from(day: Weekday) -> Self {
        day.0 as u32
    }
}

impl From<chrono::Weekday> for Weekday {
    fn from(day: chrono::Weekday) -> Self {
        Self(day.num_days_from_sunday() as u8)
    }
}

/// Calendar date used during schedule evaluation.
///
/// Unlike `chrono::DateTime`, this type stores only the calendar
/// components required by cron day-rule evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Calendar {
    /// Four-digit calendar year.
    pub year: i32,

    /// Month of the year (`1..-12`).
    pub month: u32,

    /// Day of month (`1..=31`).
    pub day: u32,
}

impl Calendar {
    /// Creates a calendar date.
    #[inline]
    pub const fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Returrns the day of week for this date.
    #[inline]
    pub fn weekday(&self) -> Weekday {
        Self::week_day(self.year, self.month, self.day)
    }

    /// Returrns `true` if this date is the final day of its month.
    #[inline]
    pub fn is_last_day(&self) -> bool {
        self.day == Self::days_in_month(self.year, self.month)
    }

    /// Zeller's congruence variant for day of week (0 = Sunday)
    pub fn week_day(year: i32, month: u32, day: u32) -> Weekday {
        let mut year = year;

        if month < 3 {
            year -= 1;
        }

        let weekday = (year + year / 4 - year / 100
            + year / 400
            + MONTH_OFFSETS[(month - 1) as usize]
            + day as i32)
            % 7;

        Weekday::new(weekday as u8)
    }

    /// Quartz rules for `W`
    ///
    /// Given `N W`:
    ///
    /// - If `N` is Monday–Friday → use `N`.
    /// - If `N` is Saturday:
    ///     - Normally → Friday (`N - 1`)
    ///     - If `N == 1` → Monday (`3`)
    /// - If `N` is Sunday:
    ///     - Normally → Monday (`N + 1`)
    ///     - If `N` is the last day of the month → Friday (`N - 2`)
    /// The important rule is:
    ///
    /// Never cross the month boundary.
    pub fn nearest_weekday(year: i32, month: u32, day: u32) -> u32 {
        let max = Self::days_in_month(year, month);

        debug_assert!(day >= 1);
        debug_assert!(day <= max);

        match Self::week_day(year, month, day) {
            weekday if weekday.is_weekday() => day,

            // Saturday
            Weekday(6) => {
                if day == 1 {
                    3
                } else {
                    day - 1
                }
            }

            // Sunday
            Weekday(0) => {
                if day == max {
                    day - 2
                } else {
                    day + 1
                }
            }

            _ => unreachable!(),
        }
    }

    /// Returns the day of month corresponding to the `nth` occurrence of a
    /// weekday within a month.
    ///
    /// For example, the third Friday of June 2025:
    ///
    /// ```text
    /// nth_weekday(2025, 6, 5, 3) == Some(20)
    /// ```
    ///
    /// Returns `None` if the requested occurrence does not exist.
    pub fn nth_weekday(year: i32, month: u32, weekday: u32, nth: u32) -> Option<u32> {
        let mut count = 0;

        let max = Self::days_in_month(year, month);

        for day in 1..=max {
            let dow = Self::week_day(year, month, day);

            if dow.0 as u32 == weekday {
                count += 1;

                if count == nth {
                    return Some(day);
                }
            }
        }

        None
    }

    /// Returns the last occurrence of a weekday within a month.
    ///
    /// This is used to evaluate Quartz expressions such as `5L`
    /// ("last Friday of the month").
    ///
    /// # Panics
    ///
    /// Panics only if the weekday is outside the valid range.
    /// Every month contains at least one occurrence of every weekday.
    pub fn last_weekday(year: i32, month: u32, weekday: Weekday) -> u32 {
        let max = Self::days_in_month(year, month);

        for day in (1..=max).rev() {
            if Self::week_day(year, month, day) == weekday {
                return day;
            }
        }
        unreachable!("Every month has at least one of each weekday");
    }

    /// Returns the last business day of a month.
    ///
    /// Business days are Monday through Friday.
    ///
    /// This is used to evaluate the Quartz `LW` expression.
    pub fn last_business_day(year: i32, month: u32) -> u32 {
        let mut day = Self::days_in_month(year, month);

        while Self::week_day(year, month, day).is_weekend() {
            day -= 1;
        }

        day
    }

    /// Returns the number of days in a month.
    ///
    /// Leap years are handled automatically for February.
    #[inline]
    pub fn days_in_month(year: i32, month: u32) -> u32 {
        MONTH_LENGTHS[Self::is_leap_year(year) as usize][(month - 1) as usize] as u32
    }

    /// Returns `true` if the specified year is a leap year in the
    /// Gregorian calendar.
    ///
    /// A leap year is divisible by 4, except years divisible by 100,
    /// unless they are also divisible by 400.
    #[inline]
    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
}

impl From<&Candidate> for Calendar {
    fn from(candidate: &Candidate) -> Self {
        Self::new(candidate.year, candidate.month, candidate.day)
    }
}

impl<Tz> From<&DateTime<Tz>> for Calendar
where
    Tz: TimeZone,
{
    fn from(dt: &DateTime<Tz>) -> Self {
        Self::new(dt.year(), dt.month(), dt.day())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_last_business_day() {
        assert_eq!(Calendar::last_business_day(2025, 5), 30);
    }

    // -------------------------------------------------------------------------
    // Weekday
    // -------------------------------------------------------------------------
    #[test]
    fn weekday_epoch() {
        // 1970-01-01 = Thursday
        assert_eq!(Calendar::week_day(1970, 1, 1), Weekday::new(4),);
    }

    #[test]
    fn weekday_y2k() {
        // 2000-01-01 = Saturday
        assert_eq!(Calendar::week_day(2000, 1, 1), Weekday::new(6));
    }

    #[test]
    fn weekday_leap_day() {
        // 2024-02-29 = Thursday
        assert_eq!(Calendar::week_day(2024, 2, 29), Weekday::new(4));
    }

    #[test]
    fn weekday_christmas_2025() {
        // Thursday
        assert_eq!(Calendar::week_day(2025, 12, 25), Weekday::new(4));
    }

    #[test]
    fn weekday_try_from_rejects_invalid_values() {
        assert!(Weekday::try_from(8u32).is_err());
    }

    #[test]
    fn weekday_wraps() {
        assert_eq!(Weekday::wrapping(8), Weekday::new(1));
    }

    // -------------------------------------------------------------------------
    // Leap years
    // -------------------------------------------------------------------------
    #[test]
    fn leap_year_divisible_by_400() {
        assert!(Calendar::is_leap_year(2000));
    }

    #[test]
    fn century_not_divisible_by_400() {
        assert!(!Calendar::is_leap_year(1900));
    }

    #[test]
    fn divisible_by_four() {
        assert!(Calendar::is_leap_year(2024));
    }

    #[test]
    fn ordinary_year() {
        assert!(!Calendar::is_leap_year(2023));
    }

    // -------------------------------------------------------------------------
    // Days in month
    // -------------------------------------------------------------------------
    #[test]
    fn january_has_31_days() {
        assert_eq!(Calendar::days_in_month(2025, 1), 31,);
    }

    #[test]
    fn april_has_30_days() {
        assert_eq!(Calendar::days_in_month(2025, 4), 30,);
    }

    #[test]
    fn february_normal_year() {
        assert_eq!(Calendar::days_in_month(2025, 2), 28,);
    }

    #[test]
    fn february_leap_year() {
        assert_eq!(Calendar::days_in_month(2024, 2), 29,);
    }

    // -------------------------------------------------------------------------
    // Nearest weekday
    // -------------------------------------------------------------------------
    #[test]
    fn nearest_weekday_returns_same_day() {
        // Already weekday
        assert_eq!(Calendar::nearest_weekday(2025, 7, 16,), 16,);
    }

    #[test]
    fn saturday_moves_to_friday() {
        /*12(Sat)
        ↓
        11(Fri)*/
        assert_eq!(Calendar::nearest_weekday(2026, 9, 12,), 11,);
    }

    #[test]
    fn sunday_moves_to_monday() {
        /*
        13(Sun)
        ↓
        14(Mon)
        */
        assert_eq!(Calendar::nearest_weekday(2026, 9, 13,), 14,);
    }

    #[test]
    fn first_day_saturday_moves_forward() {
        /*
        1 Sat
        ↓
        2 Mon*/
        assert_eq!(Calendar::nearest_weekday(2026, 8, 1,), 3,);
    }

    #[test]
    fn last_day_sunday_moves_backward() {
        /*
        31 Sun
        ↓
        30 Fri
        */
        assert_eq!(Calendar::nearest_weekday(2021, 1, 31,), 29,);
    }

    // -------------------------------------------------------------------------
    // Nth weekday
    // -------------------------------------------------------------------------
    #[test]
    fn first_monday() {
        assert_eq!(Calendar::nth_weekday(2025, 7, 1, 1,), Some(7),);
    }

    #[test]
    fn second_monday() {
        assert_eq!(Calendar::nth_weekday(2025, 7, 1, 2,), Some(14),);
    }

    #[test]
    fn third_monday() {
        assert_eq!(Calendar::nth_weekday(2025, 7, 1, 3,), Some(21),);
    }

    #[test]
    fn fourth_monday() {
        assert_eq!(Calendar::nth_weekday(2025, 7, 1, 4,), Some(28),);
    }

    #[test]
    fn fifth_monday_missing() {
        assert_eq!(Calendar::nth_weekday(2025, 2, 1, 5,), None,);
    }

    // -------------------------------------------------------------------------
    // Last weekday
    // -------------------------------------------------------------------------
    #[test]
    fn last_sunday_january_2025() {
        // Sundays: 5,12,19,26
        assert_eq!(Calendar::last_weekday(2025, 1, Weekday(0)), 26,);
    }

    #[test]
    fn last_monday_january_2025() {
        // Mondays: 6,13,20,27
        assert_eq!(Calendar::last_weekday(2025, 1, Weekday(1)), 27,);
    }

    #[test]
    fn last_tuesday_january_2025() {
        assert_eq!(Calendar::last_weekday(2025, 1, Weekday(2)), 28,);
    }

    #[test]
    fn last_wednesday_january_2025() {
        assert_eq!(Calendar::last_weekday(2025, 1, Weekday(3)), 29,);
    }

    #[test]
    fn last_thursday_january_2025() {
        assert_eq!(Calendar::last_weekday(2025, 1, Weekday(4)), 30,);
    }

    #[test]
    fn last_friday_january_2025() {
        assert_eq!(Calendar::last_weekday(2025, 1, Weekday(5)), 31,);
    }

    #[test]
    fn last_saturday_january_2025() {
        assert_eq!(Calendar::last_weekday(2025, 1, Weekday(6)), 25,);
    }

    #[test]
    fn last_thursday_february_leap_year() {
        // Feb 2024 ends on Thursday (29th)
        assert_eq!(Calendar::last_weekday(2024, 2, Weekday(4)), 29,);
    }

    #[test]
    fn last_wednesday_february_leap_year() {
        assert_eq!(Calendar::last_weekday(2024, 2, Weekday(3)), 28,);
    }

    #[test]
    fn last_friday_february_normal_year() {
        // Feb 2025 Fridays: 7,14,21,28
        assert_eq!(Calendar::last_weekday(2025, 2, Weekday(5)), 28,);
    }

    #[test]
    fn last_saturday_february_normal_year() {
        // Saturdays: 1,8,15,22
        assert_eq!(Calendar::last_weekday(2025, 2, Weekday(6)), 22,);
    }

    #[test]
    fn last_wednesday_april_2025() {
        // Wednesdays: 2,9,16,23,30
        assert_eq!(Calendar::last_weekday(2025, 4, Weekday(3)), 30,);
    }

    #[test]
    fn last_sunday_april_2025() {
        // Sundays: 6,13,20,27
        assert_eq!(Calendar::last_weekday(2025, 4, Weekday(0)), 27,);
    }

    #[test]
    fn last_weekday_is_always_correct_weekday() {
        for weekday in 0..7 {
            let day = Calendar::last_weekday(2025, 7, Weekday::try_from(weekday).unwrap());

            assert_eq!(Calendar::week_day(2025, 7, day).0, weekday,);
        }
    }

    #[test]
    fn last_weekday_is_within_month() {
        for weekday in 0..7 {
            let day = Calendar::last_weekday(2025, 12, Weekday::try_from(weekday).unwrap());

            assert!(day >= 1);
            assert!(day <= 31);
        }
    }

    #[test]
    fn returned_day_is_the_last_occurrence() {
        for weekday in 0..7 {
            let day = Calendar::last_weekday(2025, 5, Weekday::try_from(weekday).unwrap());

            let max = Calendar::days_in_month(2025, 5);

            for later in (day + 1)..=max {
                assert_ne!(Calendar::week_day(2025, 5, later).0, weekday,);
            }
        }
    }

    #[test]
    fn last_weekday_is_valid_day() {
        for year in 1990..=2035 {
            for month in 1..=12 {
                for weekday in 0..=6 {
                    let day =
                        Calendar::last_weekday(year, month, Weekday::try_from(weekday).unwrap());

                    assert!(day >= 1);
                    assert!(day <= Calendar::days_in_month(year, month));
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Calendar helpers
    // -------------------------------------------------------------------------
    #[test]
    fn calendar_is_last_day_true() {
        let cal = Calendar::new(2025, 1, 31);

        assert!(cal.is_last_day());
    }

    #[test]
    fn calendar_is_last_day_false() {
        let cal = Calendar::new(2025, 1, 30);

        assert!(!cal.is_last_day());
    }

    #[test]
    fn calendar_weekday_matches_static_function() {
        let cal = Calendar::new(2025, 7, 20);

        assert_eq!(cal.weekday(), Calendar::week_day(2025, 7, 20),);
    }
}

proptest! {
    #[test]
    fn days_in_month_is_valid(
        year in 1900i32..2400,
        month in 1u32..12,
    ) {

        let days =
            Calendar::days_in_month(year,month);

        prop_assert!((28..=31).contains(&days));
    }
}

proptest! {
    #[test]
    fn weekday_is_valid(
        year in 1900i32..2400,
        month in 1u32..12,
        day in 1u32..28,
    ) {

        let weekday =
            Calendar::week_day(year,month,day);

        prop_assert!(weekday.0 <= 6);
    }
}

proptest! {
    #[test]
    fn last_weekday_is_correct(
        year in 2000i32..2100,
        month in 1u32..12,
        weekday in 0u8..7,
    ) {

        let wd = Weekday::new(weekday);

        let day =
            Calendar::last_weekday(
                year,
                month,
                wd,
            );

        prop_assert_eq!(
            Calendar::week_day(year,month,day),
            wd,
        );
    }
}
