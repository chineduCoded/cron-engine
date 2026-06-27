#![allow(unused)]
use chrono::{DateTime, TimeZone, Utc};

use chrono_tz::{Africa::Lagos, America::New_York, Tz};
use cron_engine::cron::CronSchedule;

fn get_upcoming_datetimes(
    schedule: &CronSchedule,
    after: DateTime<Tz>,
    count: usize,
) -> Vec<DateTime<Tz>> {
    schedule
        .upcoming(after)
        .take(count)
        .filter_map(|res| res.ok())
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let expressions = vec![
        //"*/5 * * * *",
        //"0 9 * * MON-FRI",
        //"0 0 1 JAN *",
        //"15 14 1-5 JAN-MAR MON-FRI",
        //"*/10 8-18 * * *",
        //"@daily",
        //"@weekly",
        //"@monthly",
      //  "@yearly",
    //];

    let schedule = CronSchedule::parse(
        "15 14 1-5 JAN-MAR MON-FRI"
    )?;

    let start = Lagos
        .with_ymd_and_hms(2026, 6, 20, 2, 55, 0)
        .single()
        .unwrap();

    for  occurrence in schedule.upcoming(Utc::now()).take(10) {
        println!("{}", occurrence?);
    }

    Ok(())
}
