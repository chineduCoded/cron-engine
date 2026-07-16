use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use chrono::TimeZone;
use chrono_tz::UTC;

use cron_engine::cron::{
    scheduler::candidate::{Candidate, advance_numeric},
    scheduler::field::Field,
    scheduler::scheduler::CronSchedule,
};

fn candidate(c: &mut Criterion) {
    let schedule = CronSchedule::parse("*/5 * * * * *").unwrap();

    let ir = schedule.ir();

    let mut group = c.benchmark_group("candidate");

    group.bench_function("hour", |b| {
        b.iter(|| {
            let mut candidate =
                Candidate::from_datetime(UTC.with_ymd_and_hms(2025, 6, 15, 3, 27, 19).unwrap());

            advance_numeric(black_box(&mut candidate), ir, Field::Hour);
        });
    });

    group.bench_function("minute", |b| {
        b.iter(|| {
            let mut candidate =
                Candidate::from_datetime(UTC.with_ymd_and_hms(2025, 6, 15, 3, 27, 19).unwrap());

            advance_numeric(black_box(&mut candidate), ir, Field::Minute);
        });
    });

    group.bench_function("reset_after_day", |b| {
        b.iter(|| {
            let mut candidate =
                Candidate::from_datetime(UTC.with_ymd_and_hms(2025, 6, 15, 3, 27, 19).unwrap());

            candidate.reset_after(Field::Day, ir);
        });
    });

    group.finish();
}

criterion_group!(benches, candidate);
criterion_main!(benches);
