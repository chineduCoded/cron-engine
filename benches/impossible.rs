use criterion::{Criterion, criterion_group, criterion_main};

use chrono::TimeZone;
use chrono_tz::UTC;

use cron_engine::cron::scheduler::scheduler::CronSchedule;

fn impossible(c: &mut Criterion) {
    let schedule = CronSchedule::parse("0 0 0 * 2 1#5").unwrap();

    let start = UTC.with_ymd_and_hms(2021, 2, 1, 0, 0, 0).unwrap();

    c.bench_function("impossible schedule", |b| {
        b.iter(|| {
            schedule.next_after(start);
        });
    });
}

criterion_group!(benches, impossible);
criterion_main!(benches);
