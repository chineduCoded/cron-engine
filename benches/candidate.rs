use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use chrono::TimeZone;
use chrono_tz::UTC;

use cron_engine::cron::scheduler::{candidate::{Candidate, navigate_numeric}, field::Field, scheduler::{CronSchedule, Direction}};

fn candidate(c: &mut Criterion) {
    let schedule = CronSchedule::parse("*/5 * * * * *").unwrap();

    let ir = schedule.ir();

    let mut group = c.benchmark_group("candidate");

    group.bench_function("hour forward", |b| {
        b.iter(|| {
            let mut candidate =
                Candidate::from_datetime(UTC.with_ymd_and_hms(2025, 6, 15, 3, 27, 19).unwrap());

            navigate_numeric(black_box(&mut candidate), ir, Field::Hour, Direction::Forward);
        });
    });

    group.bench_function("minute forward", |b| {
        b.iter(|| {
            let mut candidate =
                Candidate::from_datetime(UTC.with_ymd_and_hms(2025, 6, 15, 3, 27, 19).unwrap());

            navigate_numeric(black_box(&mut candidate), ir, Field::Minute, Direction::Forward);
        });
    });

    group.bench_function("reset_forward_day", |b| {
        b.iter(|| {
            let mut candidate =
                Candidate::from_datetime(UTC.with_ymd_and_hms(2025, 6, 15, 3, 27, 19).unwrap());

            candidate.reset_after(Field::Day, ir, Direction::Forward);
        });
    });

    group.finish();
}

criterion_group!(benches, candidate);
criterion_main!(benches);
