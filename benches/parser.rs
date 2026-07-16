use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use cron_engine::cron::parser;

fn parser(c: &mut Criterion) {
    let parser = parser::CronParser::new();

    let mut group = c.benchmark_group("parser");

    group.bench_function("wildcard", |b| {
        b.iter(|| {
            parser.parse(black_box("* * * * * *")).unwrap();
        });
    });

    group.bench_function("lists", |b| {
        b.iter(|| parser.parse("1,2,3,4 * * * * *").unwrap());
    });

    group.bench_function("ranges", |b| {
        b.iter(|| {
            parser.parse(black_box("0 0-30/5 9-17 * * *")).unwrap();
        });
    });

    group.bench_function("Names", |b| {
        b.iter(|| {
            parser
                .parse(black_box("0 0 12 * JAN,MAR,JUN MON-FRI"))
                .unwrap();
        });
    });

    group.bench_function("L", |b| {
        b.iter(|| parser.parse("0 0 0 L * *").unwrap());
    });

    group.bench_function("LW", |b| {
        b.iter(|| {
            parser.parse(black_box("0 0 0 LW * *")).unwrap();
        });
    });

    group.bench_function("W", |b| {
        b.iter(|| parser.parse("0 0 0 15W * *").unwrap());
    });

    group.bench_function("Nth weekday", |b| {
        b.iter(|| {
            parser.parse(black_box("0 0 0 * * 5#3")).unwrap();
        });
    });

    group.bench_function("Year", |b| {
        b.iter(|| parser.parse("0 0 0 * * * 2025").unwrap());
    });

    group.finish();
}

criterion_group!(benches, parser);
criterion_main!(benches);
