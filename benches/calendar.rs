use std::hint::black_box;

use chrono::TimeZone;
use chrono_tz::UTC;
use criterion::{Criterion, criterion_group, criterion_main};

use cron_engine::cron::evaluator::calendar::{Calendar, Weekday};

fn calendar(c: &mut Criterion) {
    let dt = UTC.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();

    let mut group = c.benchmark_group("calendar");

    group.bench_function("from datetime", |b| {
        b.iter(|| Calendar::from(black_box(&dt)))
    });

    group.bench_function("weekday", |b| {
        b.iter(|| Calendar::week_day(black_box(2025), black_box(6), black_box(15)))
    });

    group.bench_function("days in month", |b| {
        b.iter(|| Calendar::days_in_month(black_box(2025), black_box(2)))
    });

    group.bench_function("leap year", |b| {
        b.iter(|| Calendar::is_leap_year(black_box(2024)))
    });

    group.bench_function("nearest weekday", |b| {
        b.iter(|| Calendar::nearest_weekday(black_box(2025), black_box(6), black_box(15)))
    });

    group.bench_function("nth weekday", |b| {
        b.iter(|| {
            Calendar::nth_weekday(
                black_box(2025),
                black_box(6),
                black_box(1), // Monday
                black_box(3), // 3rd Monday
            )
        })
    });

    group.bench_function("last weekday", |b| {
        b.iter(|| {
            Calendar::last_weekday(
                black_box(2025),
                black_box(6),
                black_box(Weekday::new(5)), // Friday
            )
        })
    });

    group.bench_function("last business day", |b| {
        b.iter(|| Calendar::last_business_day(black_box(2025), black_box(6)))
    });

    group.finish();
}

criterion_group!(benches, calendar);
criterion_main!(benches);
