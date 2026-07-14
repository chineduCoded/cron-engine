use criterion::{criterion_group, criterion_main, Criterion};
use chrono::TimeZone;
use chrono_tz::UTC;
use dhat::Alloc;

use cron_engine::cron::scheduler::scheduler::CronSchedule;

#[global_allocator]
static ALLOCATOR: dhat::Alloc = Alloc;

fn scheduler_allocations(c: &mut Criterion) {
    let _profiler = dhat::Profiler::new_heap();

    let schedule =
        CronSchedule::parse("0 */5 * * * *").unwrap();

    let start = UTC
        .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
        .unwrap();

    c.bench_function("next_after", |b| {
        b.iter(|| {
            schedule.next_after(start)
        });
    });
}

criterion_group!(benches, scheduler_allocations);
criterion_main!(benches);
