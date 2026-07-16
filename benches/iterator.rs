use criterion::{Criterion, criterion_group, criterion_main};

use chrono::TimeZone;
use chrono_tz::UTC;

use cron_engine::cron::scheduler::scheduler;

fn iterator(c: &mut Criterion) {
    let schedule = scheduler::CronSchedule::parse("0 * * * * *").unwrap();

    let start = UTC.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

    let mut group = c.benchmark_group("iterator");

    group.bench_function("1000 upcoming", |b| {
        b.iter(|| {
            schedule
                .upcoming(start)
                .take(1000)
                .map(Result::unwrap)
                .count();
        });
    });

    group.finish();
}

criterion_group!(benches, iterator);
criterion_main!(benches);
