#![allow(unused)]

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use chrono_tz::Tz;
use proptest::prelude::*;

use crate::cron::{ir::CronIr, scheduler::{field::Field, navigator::{AdvanceResult, FieldNavigator, NumericNavigator}}, timezone::resolve_local};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    pub year: i32,
    pub month: u32,
    pub day: u32,

    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl Candidate {
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) ->Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    pub fn normalize(&mut self) {
        let Some(naive) = self.to_naive() else {
            return;
        };

        *self = Candidate::from_naive(naive);
    }

    pub fn from_datetime(
        dt: DateTime<Tz>,
    ) -> Self {
        Self {
            year: dt.year(),
            month: dt.month(),
            day: dt.day(),
            hour: dt.hour(),
            minute: dt.minute(),
            second: dt.second(),
        }
    }

    pub fn from_naive(
        naive: NaiveDateTime,
    ) -> Self {
        Self {
            year: naive.year(),
            month: naive.month(),
            day: naive.day(),
            hour: naive.hour(),
            minute: naive.minute(),
            second: naive.second(),
        }
    }

    pub fn to_naive(self) -> Option<NaiveDateTime> {
        let date = NaiveDate::from_ymd_opt(
            self.year, 
            self.month, 
            self.day,
        )?;

        date.and_hms_opt(
            self.hour,
            self.minute,
            self.second,
        )
    }

    pub fn to_datetime(
        &self,
        tz: &Tz,
    ) -> Option<DateTime<Tz>> {
        let naive = self.to_naive()?;
        resolve_local(tz, naive)
    }

    pub fn reset_after(
        &mut self,
        field: Field,
        ir: &CronIr,
    ) {
        match field {
            Field::Year => {
                self.month = ir.min_month();
                self.day = 1;

                self.reset_time(ir);
            }

            Field::Month => {
                self.day = 1;

                self.reset_time(ir);
            }

            Field::Day => {
                self.reset_time(ir);
            }

            Field::Hour => {
                self.minute = ir.min_minute();
                self.second = ir.min_second();
            }

            Field::Minute => {
                self.second = ir.min_second();
            }

            Field::Second => {}
        }
    }

    fn reset_time(
        &mut self,
        ir: &CronIr,
    ) {
        self.hour = ir.min_hour();
        self.minute = ir.min_minute();
        self.second = ir.min_second();
    }

    pub fn get(
        &self,
        field: Field,
    ) -> u32 {
        match field {
            Field::Year => self.year as u32,
            Field::Month => self.month,
            Field::Day => self.day,
            Field::Hour => self.hour,
            Field::Minute => self.minute,
            Field::Second => self.second,
        }
    }

    pub fn set(
        &mut self,
        field: Field,
        value: u32,
    ) {
        match field {
            Field::Year => self.year = value as i32,
            Field::Month => self.month = value,
            Field::Day => self.day = value,
            Field::Hour => self.hour = value,
            Field::Minute => self.minute = value,
            Field::Second => self.second = value,
        }
    }

    /// Advances the parent field.
    pub fn advance_parent(
        &mut self,
        field: Field,
    ) {
        match field {
            Field::Year => {
                return;
            }

            Field::Month => {
                self.year += 1
            }

            Field::Day => self.next_month(),

            Field::Hour => {
                self.map_datetime(|dt| dt + Duration::days(1))
            }

            Field::Minute => {
                self.map_datetime(|dt| dt + Duration::hours(1))
            }

            Field::Second => {
                self.map_datetime(|dt| dt + Duration::minutes(1))
            }
        }
    }

    pub fn next_valid_day(&mut self) {
        self.map_datetime(|dt| dt + Duration::days(1));
    }

    pub fn advance_smallest(&mut self) {
        self.map_datetime(|dt| dt + Duration::seconds(1));
    }

    fn map_datetime(
        &mut self,
        f: impl FnOnce(NaiveDateTime) -> NaiveDateTime,
    ) {
        let naive = self.to_naive()
            .expect("candidate should always be valid");

        *self = Candidate::from_naive(f(naive))
    }

    fn next_month(&mut self) {
        self.map_datetime(|dt| {
            let first = dt.with_day(1).unwrap();

            if first.month() == 12 {
                first
                    .with_year(first.year() + 1)
                    .unwrap()
                    .with_month(1)
                    .unwrap()
            } else {
                first
                    .with_month(first.month() + 1)
                    .unwrap()
            }
        });
    }
}

pub fn advance_numeric(
    candidate: &mut Candidate,
    ir: &CronIr,
    field: Field,
) -> bool {
    let Some(matcher) = ir.matcher(field) else {
        panic!("advance_numeric called for non-numeric field")
    };

    let nav = NumericNavigator::new(matcher);

    match nav.advance(candidate.get(field)) {
        AdvanceResult::Unchanged => false,

        AdvanceResult::Changed(next) => {
            candidate.set(field, next);
            candidate.reset_after(field, ir);
            true
        }

        AdvanceResult::Wrapped(min) => {
            candidate.advance_parent(field);
            candidate.set(field, min);
            candidate.reset_after(field, ir);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::NaiveDate;

    use crate::cron::{
        field::BitField, ir::{CronIr, DayRule, FieldMatcher}, scheduler::field::Field,
    };

    fn matcher(
        value: u32,
        min: u32,
        max: u32,
    ) ->FieldMatcher {
        let mut bf = BitField::empty(min, max - min + 1);
        bf.set(value);

        FieldMatcher::from(bf)
    }

    fn matcher_values(
        values: &[u32],
        min: u32,
        max: u32,
    ) ->FieldMatcher {
        let mut bf = BitField::empty(min, max - min + 1);

        for &value in values {
            bf.set(value);
        }

        FieldMatcher::from(bf)
    }

    fn ir() -> CronIr {
        CronIr {
            second: matcher_values(&[5, 30, 45], 0, 59),
            minute: matcher_values(&[10, 20, 40], 0, 59),
            hour: matcher_values(&[3, 8, 16], 0, 23),

            day_of_month: DayRule::Any,
            day_of_week: DayRule::Any,

            month: matcher_values(&[2, 6, 11], 1, 12),

            year: None,

            dom_dow_or: true,
        }
    }

    #[test]
    fn from_naive_extracts_all_fields() {
        let naive = NaiveDate::from_ymd_opt(2025, 8, 14)
            .unwrap()
            .and_hms_opt(17, 30, 45)
            .unwrap();

        let c = Candidate::from_naive(naive);

        assert_eq!(c.year, 2025);
        assert_eq!(c.month, 8);
        assert_eq!(c.day, 14);

        assert_eq!(c.hour, 17);
        assert_eq!(c.minute, 30);
        assert_eq!(c.second, 45);
    }

    #[test]
    fn to_naive_round_trip() {
        let original = Candidate::new(
            2025,
            9,
            8,
            15,
            20,
            33,
        );

        let naive = original.to_naive().unwrap();

        let copy = Candidate::from_naive(naive);

        assert_eq!(copy, original);
    }

    #[test]
    fn invalid_date_returns_none() {
        let c = Candidate::new(
            2025,
            2,
            31,
            0,
            0,
            0,
        );

        assert!(c.to_naive().is_none());
    }

        #[test]
    fn get_returns_correct_values() {
        let c = Candidate {
            year: 2030,
            month: 5,
            day: 7,

            hour: 12,
            minute: 15,
            second: 20,
        };

        assert_eq!(c.get(Field::Year), 2030);
        assert_eq!(c.get(Field::Month), 5);
        assert_eq!(c.get(Field::Hour), 12);
        assert_eq!(c.get(Field::Minute), 15);
        assert_eq!(c.get(Field::Second), 20);
    }

        #[test]
    fn set_updates_field() {
        let mut c = Candidate {
            year: 2025,
            month: 1,
            day: 1,

            hour: 0,
            minute: 0,
            second: 0,
        };

        c.set(Field::Hour, 18);

        assert_eq!(c.hour, 18);

        c.set(Field::Minute, 42);

        assert_eq!(c.minute, 42);

        c.set(Field::Second, 59);

        assert_eq!(c.second, 59);
    }

    #[test]
    fn reset_after_year() {
        let mut c = Candidate {
            year: 2030,
            month: 12,
            day: 25,

            hour: 23,
            minute: 59,
            second: 58,
        };

        c.reset_after(Field::Year, &ir());

        assert_eq!(c.month, 2);
        assert_eq!(c.day, 1);

        assert_eq!(c.hour, 3);
        assert_eq!(c.minute, 10);
        assert_eq!(c.second, 5);
    }

    #[test]
    fn reset_after_month() {
        let mut c = Candidate {
            year: 2025,
            month: 12,
            day: 20,

            hour: 18,
            minute: 30,
            second: 50,
        };

        c.reset_after(Field::Month, &ir());

        assert_eq!(c.day, 1);

        assert_eq!(c.hour, 3);
        assert_eq!(c.minute, 10);
        assert_eq!(c.second, 5);
    }

    #[test]
    fn reset_after_day() {
        let mut c = Candidate {
            year: 2025,
            month: 7,
            day: 12,

            hour: 20,
            minute: 55,
            second: 59,
        };

        c.reset_after(Field::Day, &ir());

        assert_eq!(c.hour, 3);
        assert_eq!(c.minute, 10);
        assert_eq!(c.second, 5);
    }

    #[test]
    fn reset_after_hour() {
        let mut c = Candidate {
            year: 2025,
            month: 7,
            day: 12,

            hour: 18,
            minute: 44,
            second: 33,
        };

        c.reset_after(Field::Hour, &ir());

        assert_eq!(c.minute, 10);
        assert_eq!(c.second, 5);
    }

    #[test]
    fn reset_after_minute() {
        let mut c = Candidate {
            year: 2025,
            month: 7,
            day: 12,

            hour: 18,
            minute: 44,
            second: 33,
        };

        c.reset_after(Field::Minute, &ir());

        assert_eq!(c.second, 5);
    }

        #[test]
    fn reset_after_second_does_nothing() {
        let mut c = Candidate {
            year: 2025,
            month: 7,
            day: 12,

            hour: 18,
            minute: 44,
            second: 33,
        };

        let before = c;

        c.reset_after(Field::Second, &ir());

        assert_eq!(c, before);
    }

        #[test]
    fn map_datetime_can_add_day() {
        let mut c = Candidate {
            year: 2025,
            month: 5,
            day: 31,

            hour: 12,
            minute: 0,
            second: 0,
        };

        c.map_datetime(|dt| dt + chrono::Duration::days(1));

        assert_eq!(c.year, 2025);
        assert_eq!(c.month, 6);
        assert_eq!(c.day, 1);
    }

    #[test]
    fn map_datetime_can_add_month() {
        let mut c = Candidate {
            year: 2025,
            month: 12,
            day: 1,

            hour: 0,
            minute: 0,
            second: 0,
        };

        c.map_datetime(|dt| dt + chrono::Duration::days(31));

        assert_eq!(c.year, 2026);
    }

    #[test]
    fn advance_parent_second() {
        let mut candidate = Candidate {
            year: 2025,
            month: 6,
            day: 15,
            hour: 10,
            minute: 20,
            second: 30,
        };

        candidate.advance_parent(Field::Second);

        assert_eq!(candidate.second, 30);
        assert_eq!(candidate.minute, 21);
        assert_eq!(candidate.hour, 10);
        assert_eq!(candidate.day, 15);
    }

    #[test]
    fn advance_parent_second_wraps_minute() {
        let mut candidate = Candidate {
            year: 2025,
            month: 6,
            day: 15,
            hour: 10,
            minute: 59,
            second: 30,
        };

        candidate.advance_parent(Field::Second);

        assert_eq!(candidate.minute, 0);
        assert_eq!(candidate.hour, 11);
    }

    #[test]
    fn advance_parent_minute() {
        let mut candidate = Candidate {
            year: 2025,
            month: 6,
            day: 15,
            hour: 10,
            minute: 20,
            second: 40,
        };

        candidate.advance_parent(Field::Minute);

        assert_eq!(candidate.hour, 11);
        assert_eq!(candidate.minute, 20);
        assert_eq!(candidate.second, 40);
    }

    #[test]
    fn advance_parent_minute_wraps_day() {
        let mut candidate = Candidate {
            year: 2025,
            month: 6,
            day: 15,
            hour: 23,
            minute: 20,
            second: 40,
        };

        candidate.advance_parent(Field::Minute);

        assert_eq!(candidate.day, 16);
        assert_eq!(candidate.hour, 0);
    }

    #[test]
    fn advance_parent_hour() {
        let mut candidate = Candidate {
            year: 2025,
            month: 6,
            day: 15,
            hour: 10,
            minute: 20,
            second: 40,
        };

        candidate.advance_parent(Field::Hour);

        assert_eq!(candidate.day, 16);
        assert_eq!(candidate.hour, 10);
    }

    #[test]
    fn advance_parent_hour_wraps_month() {
        let mut candidate = Candidate {
            year: 2025,
            month: 6,
            day: 30,
            hour: 10,
            minute: 20,
            second: 40,
        };

        candidate.advance_parent(Field::Hour);

        assert_eq!(candidate.year, 2025);
        assert_eq!(candidate.month, 7);
        assert_eq!(candidate.day, 1);
    }

    #[test]
    fn advance_parent_day() {
        let mut candidate = Candidate {
            year: 2025,
            month: 3,
            day: 15,
            hour: 12,
            minute: 30,
            second: 45,
        };

        candidate.advance_parent(Field::Day);

        assert_eq!(candidate.year, 2025);
        assert_eq!(candidate.month, 4);
        assert_eq!(candidate.day, 1);
    }

    #[test]
    fn advance_parent_day_wraps_year() {
        let mut candidate = Candidate {
            year: 2025,
            month: 12,
            day: 15,
            hour: 12,
            minute: 30,
            second: 45,
        };

        candidate.advance_parent(Field::Day);

        assert_eq!(candidate.year, 2026);
        assert_eq!(candidate.month, 1);
        assert_eq!(candidate.day, 1);
    }

    #[test]
    fn advance_parent_month() {
        let mut candidate = Candidate {
            year: 2025,
            month: 6,
            day: 15,
            hour: 10,
            minute: 20,
            second: 40,
        };

        candidate.advance_parent(Field::Month);

        assert_eq!(candidate.year, 2026);
        assert_eq!(candidate.month, 6);
        assert_eq!(candidate.day, 15);
    }

    #[test]
    fn advance_parent_year_is_noop() {
        let mut candidate = Candidate {
            year: 2025,
            month: 6,
            day: 15,
            hour: 10,
            minute: 20,
            second: 40,
        };

        let before = candidate;

        candidate.advance_parent(Field::Year);

        assert_eq!(candidate, before);
    }

    #[test]
    fn advance_parent_hour_handles_leap_year() {
        let mut candidate = Candidate {
            year: 2024,
            month: 2,
            day: 29,
            hour: 12,
            minute: 0,
            second: 0,
        };

        candidate.advance_parent(Field::Hour);

        assert_eq!(candidate.year, 2024);
        assert_eq!(candidate.month, 3);
        assert_eq!(candidate.day, 1);
    }

    #[test]
    fn advance_parent_hour_handles_non_leap_year() {
        let mut candidate = Candidate {
            year: 2025,
            month: 2,
            day: 28,
            hour: 12,
            minute: 0,
            second: 0,
        };

        candidate.advance_parent(Field::Hour);

        assert_eq!(candidate.month, 3);
        assert_eq!(candidate.day, 1);
    }

    #[test]
    fn advance_parent_hour_handles_new_year() {
        let mut candidate = Candidate {
            year: 2025,
            month: 12,
            day: 31,
            hour: 8,
            minute: 0,
            second: 0,
        };

        candidate.advance_parent(Field::Hour);

        assert_eq!(candidate.year, 2026);
        assert_eq!(candidate.month, 1);
        assert_eq!(candidate.day, 1);
    }

    #[test]
    fn advance_second_unchanged() {
        let ir = ir();

        let mut candidate = Candidate {
            year: 2025,
            month: 2,
            day: 10,
            hour: 3,
            minute: 10,
            second: 5,
        };

        assert!(!advance_numeric(
            &mut candidate,
            &ir,
            Field::Second,
        ));
    }

    #[test]
    fn advance_minute_unchanged() {
        let ir = ir();

        let mut candidate = Candidate {
            year: 2025, 
            month: 2, 
            day: 10,
            hour: 3, 
            minute: 10, 
            second: 50,
        };

        assert!(!advance_numeric(
            &mut candidate,
            &ir,
            Field::Minute,
        ));

        assert_eq!(candidate.minute, 10);
    }

    #[test]
    fn advance_hour_unchanged() {
        let ir = ir();

        let mut candidate = Candidate::new(
            2025, 2, 10,
            3, 15, 50,
        );

        assert!(!advance_numeric(
            &mut candidate,
            &ir,
            Field::Hour,
        ));

        assert_eq!(candidate.hour, 3);
    }

    #[test]
    fn advance_second_changed() {
        let mut ir = ir();

        ir.second = matcher_values(&[5, 20, 40], 0, 59);

        let mut candidate = Candidate::new(
            2025, 2, 10,
            3, 10, 17,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Second,
        ));

        assert_eq!(candidate.second, 20);
    }

    #[test]
    fn advance_minute_changed() {
        let mut ir = ir();

        ir.minute = matcher_values(&[10, 20, 45], 0, 59);

        let mut candidate = Candidate::new(
            2025, 2, 10,
            3, 17, 55,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Minute,
        ));

        assert_eq!(candidate.minute, 20);

        // second reset
        assert_eq!(candidate.second, 5);
    }

    #[test]
    fn advance_hour_changed() {
        let mut ir = ir();

        ir.hour = matcher_values(&[3, 8, 16], 0, 23);

        let mut candidate = Candidate::new(
            2025, 2, 10,
            5, 30, 40,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Hour,
        ));

        assert_eq!(candidate.hour, 8);

        assert_eq!(candidate.minute, 10);
        assert_eq!(candidate.second, 5);
    }

    #[test]
    fn advance_month_changed() {
        let mut ir = ir();

        ir.month = matcher_values(&[2, 5, 9], 1, 12);

        let mut candidate = Candidate::new(
            2025, 3, 17,
            8, 20, 30,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Month,
        ));

        assert_eq!(candidate.month, 5);

        assert_eq!(candidate.day, 1);

        assert_eq!(candidate.hour, 3);
        assert_eq!(candidate.minute, 10);
        assert_eq!(candidate.second, 5);
    }

    #[test]
    fn advance_year_changed() {
        let mut ir = ir();

        ir.year = Some(
            matcher_values(&[2025, 2027], 2025, 2030),
        );

        let mut candidate = Candidate::new(
            2026, 8, 20,
            9, 40, 50,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Year,
        ));

        assert_eq!(candidate.year, 2027);

        assert_eq!(candidate.month, 2);
        assert_eq!(candidate.day, 1);

        assert_eq!(candidate.hour, 3);
        assert_eq!(candidate.minute, 10);
        assert_eq!(candidate.second, 5);
    }

    #[test]
    fn advance_second_wraps() {
        let ir = ir();

        let mut candidate = Candidate::new(
            2025, 2, 10,
            3, 10, 59,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Second,
        ));

        assert_eq!(candidate.minute, 11);
        assert_eq!(candidate.second, 5);
    }

    #[test]
    fn advance_minute_wraps() {
        let ir = ir();

        let mut candidate = Candidate::new(
            2025, 2, 10,
            3, 59, 20,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Minute,
        ));

        assert_eq!(candidate.hour, 4);
        assert_eq!(candidate.minute, 10);
        assert_eq!(candidate.second, 5);
    }

    #[test]
    fn advance_hour_wraps() {
        let ir = ir();

        let mut candidate = Candidate::new(
            2025, 2, 10,
            23, 40, 50,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Hour,
        ));

        assert_eq!(candidate.day, 11);

        assert_eq!(candidate.hour, 3);

        assert_eq!(candidate.minute, 10);
        assert_eq!(candidate.second, 5);
    }

    #[test]
    fn advance_month_wraps() {
        let ir = ir();

        let mut candidate = Candidate::new(
            2025, 12, 25,
            12, 20, 30,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Month,
        ));

        assert_eq!(candidate.year, 2026);
        assert_eq!(candidate.month, 2);

        assert_eq!(candidate.day, 1);

        assert_eq!(candidate.hour, 3);
        assert_eq!(candidate.minute, 10);
        assert_eq!(candidate.second, 5);
    }

    #[test]
    fn advance_year_wraps_to_minimum() {
        let mut ir = ir();

        ir.year = Some(
            matcher_values(&[2025, 2027], 2025, 2030),
        );

        let mut candidate = Candidate::new(
            2028, 8, 20,
            9, 40, 50,
        );

        assert!(advance_numeric(
            &mut candidate,
            &ir,
            Field::Year,
        ));

        assert_eq!(candidate.year, 2025);

        assert_eq!(candidate.month, 2);
        assert_eq!(candidate.day, 1);

        assert_eq!(candidate.hour, 3);
        assert_eq!(candidate.minute, 10);
        assert_eq!(candidate.second, 5);
    }

    #[test]
    #[should_panic(expected = "advance_numeric called for non-numeric field")]
    fn advance_day_panics() {
        let ir = ir();

        let mut candidate = Candidate::new(
            2025, 2, 10,
            3, 10, 5,
        );

        advance_numeric(
            &mut candidate,
            &ir,
            Field::Day,
        );
    }
}

proptest! {
    #[test]
    fn next_day_never_goes_backwards(
        year in 2020i32..2030,
        month in 1u32..12,
        day in 1u32..28,
    ) {

        let mut candidate =
            Candidate::new(
                year,
                month,
                day,
                0,
                0,
                0,
            );

        let before =
            candidate.to_naive().unwrap();

        candidate.next_valid_day();

        let after =
            candidate.to_naive().unwrap();

        prop_assert!(after > before);
    }
}





















