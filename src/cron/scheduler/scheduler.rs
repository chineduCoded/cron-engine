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
