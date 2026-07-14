use criterion::{criterion_group, criterion_main, Criterion};

use chrono::TimeZone;
use chrono_tz::UTC;

use cron_engine::cron::scheduler::scheduler;

fn throughput(c: &mut Criterion) {
    let schedule =
        scheduler::CronSchedule::parse("0 * * * * *")
            .unwrap();

    let start = UTC
        .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
        .unwrap();

    let mut group = c.benchmark_group("throughput");

    group.sample_size(50);

    for count in [1_000, 10_000, 100_000, 1_000_000] {
        group.bench_function(format!("{count} occurrences"), |b| {
            b.iter(|| {
                let mut iter = schedule.upcoming(start);

                for _ in 0..count {
                    let _ = iter.next().unwrap();
                }
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, throughput);
criterion_main!(benches);
