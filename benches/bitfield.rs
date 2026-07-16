use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use cron_engine::cron::field;

fn bitfield(c: &mut Criterion) {
    let bf = field::BitField::full(0, 64);

    let mut group = c.benchmark_group("bitfield");

    group.bench_function("contains", |b| {
        b.iter(|| {
            bf.contains(black_box(37));
        });
    });

    group.bench_function("next", |b| {
        b.iter(|| {
            bf.next_from(black_box(23));
        });
    });

    group.finish();
}

criterion_group!(benches, bitfield);
criterion_main!(benches);
