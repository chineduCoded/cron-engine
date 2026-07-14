use criterion::{criterion_group, criterion_main, Criterion};

use chrono::TimeZone;
use chrono_tz::UTC;

use cron_engine::cron::scheduler::scheduler::CronSchedule;

fn pathological(c: &mut Criterion) {
    let start = UTC
        .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
        .unwrap();

    let schedules = [
        ("Feb 29", "0 0 0 29 2 *"),
        ("31st", "0 0 0 31 * *"),
        ("Last weekday", "0 0 0 LW * *"),
        ("5th Monday", "0 0 0 * * 1#5"),
    ];

    for (name, expr) in schedules {
        let schedule = CronSchedule::parse(expr).unwrap();

        c.bench_function(name, |b| {
            b.iter(|| {
                schedule.next_after(start);
            });
        });
    }
}

criterion_group!(benches, pathological);
criterion_main!(benches);
