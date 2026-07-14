use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use chrono::TimeZone;
use chrono_tz::UTC;

use cron_engine::cron::{
    scheduler::candidate::{advance_numeric, Candidate},
    scheduler::field::Field,
    scheduler::scheduler::CronSchedule,
};

fn candidate(c: &mut Criterion) {
    let schedule = CronSchedule::parse("*/5 * * * * *").unwrap();

    let ir = &schedule.ir;

    let mut group = c.benchmark_group("candidate");

    group.bench_function("candidate/hour", |b| {
        b.iter(|| {
            let mut candidate = Candidate::from_datetime(
                UTC.with_ymd_and_hms(2025, 6, 15, 3, 27, 19)
                    .unwrap(),
            );

            advance_numeric(
                black_box(&mut candidate), 
                ir,
                Field::Hour,
            );
        });
    });

    group.bench_function("candidate/minute", |b| {
        b.iter(|| {
            let mut candidate = Candidate::from_datetime(
                UTC.with_ymd_and_hms(2025, 6, 15, 3, 27, 19)
                    .unwrap(),
            );

            advance_numeric(
                black_box(&mut candidate),
                ir,
                Field::Minute,
            );
        });
    });

    group.bench_function("candidate/reset_after_day", |b| {
        b.iter(|| {
            let mut candidate = Candidate::from_datetime(
                UTC.with_ymd_and_hms(2025, 6, 15, 3, 27, 19)
                    .unwrap(),
            );

            candidate.reset_after(Field::Day, ir);
        });
    });

    group.finish();
}

criterion_group!(benches, candidate);
criterion_main!(benches);
