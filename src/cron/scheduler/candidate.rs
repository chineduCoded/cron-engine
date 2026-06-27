use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use chrono_tz::Tz;

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
