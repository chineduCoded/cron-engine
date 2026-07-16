use chrono::DateTime;
use chrono_tz::Tz;

use crate::cron::{
    CronError,
    scheduler::scheduler::{CronSchedule, Direction},
};

/// Lazy iterator over future schedule occurrences.
///
/// Created by [`CronSchedule::upcoming`] and
/// [`CronSchedule::upcoming_inclusive`].
///
/// Occurrences are computed on demand rather than being precomputed,
/// allowing efficient iteration over large or unbounded schedules.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CronIterator {
    schedule: CronSchedule,
    current: DateTime<Tz>,
    inclusive: bool,
    direction: Direction,
}

impl CronIterator {
    pub fn new(
        schedule: CronSchedule,
        start: DateTime<Tz>,
        inclusive: bool,
        direction: Direction,
    ) -> Self {
        Self {
            schedule,
            current: start,
            inclusive,
            direction,
        }
    }
}

impl Iterator for CronIterator {
    type Item = Result<DateTime<Tz>, CronError>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = if self.inclusive {
            self.inclusive = false;
            self.current
        } else {
            self.current
        };

        let next = match self.direction {
            Direction::Forward => self.schedule.next_after(start),

            Direction::Backward => self.schedule.prev_before(start),
        };

        match next {
            Some(dt) => {
                self.current = dt;
                Some(Ok(dt))
            }
            None => None,
        }
    }
}
