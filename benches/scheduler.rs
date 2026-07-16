use std::hint::black_box;

use chrono::TimeZone;
use chrono_tz::UTC;
use criterion::{Criterion, criterion_group, criterion_main};

use cron_engine::cron::scheduler::scheduler;

fn scheduler(c: &mut Criterion) {
    let start = UTC.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();

    let mut group = c.benchmark_group("scheduler");

    group.bench_function("every 5 minutes", |b| {
        let schedule = scheduler::CronSchedule::parse("0 */5 * * * *").unwrap();

        b.iter(|| {
            schedule.next_after(black_box(start)).unwrap();
        });
    });

    group.bench_function("every second", |b| {
        let schedule = scheduler::CronSchedule::parse("* * * * * *").unwrap();

        b.iter(|| schedule.next_after(black_box(start)));
    });

    group.bench_function("hourly", |b| {
        let schedule = scheduler::CronSchedule::parse("0 0 * * * *").unwrap();

        b.iter(|| schedule.next_after(black_box(start)));
    });

    group.bench_function("daily", |b| {
        let schedule = scheduler::CronSchedule::parse("0 0 0 * * *").unwrap();

        b.iter(|| schedule.next_after(black_box(start)));
    });

    group.bench_function("monthly", |b| {
        let schedule = scheduler::CronSchedule::parse("0 0 0 1 * *").unwrap();

        b.iter(|| schedule.next_after(black_box(start)));
    });

    group.bench_function("last weekday", |b| {
        let schedule = scheduler::CronSchedule::parse("0 0 0 LW * *").unwrap();

        b.iter(|| schedule.next_after(black_box(start)));
    });

    group.finish();
}

criterion_group!(benches, scheduler);
criterion_main!(benches);
