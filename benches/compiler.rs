use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use cron_engine::cron::{compiler::CronCompiler, parser::CronParser};

fn compiler(c: &mut Criterion) {
    let parser = CronParser::new();

    let simple = parser.parse("0 * * * * *").unwrap();

    let complex = parser
        .parse("*/15 0-30/5 9-17 LW JAN,MAR,JUN MON-FRI 2025-2035")
        .unwrap();

    let mut group = c.benchmark_group("compiler");

    group.bench_function("simple", |b| {
        b.iter(|| {
            CronCompiler::compile(black_box(simple.clone())).unwrap();
        });
    });

    group.bench_function("complex", |b| {
        b.iter(|| {
            CronCompiler::compile(black_box(complex.clone())).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, compiler);
criterion_main!(benches);
