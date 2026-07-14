use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use cron_engine::cron::field;

fn bitfield(c: &mut Criterion) {
    let bf = field::BitField::full(0, 64);

    c.bench_function(
        "bitfield contains",
        |b| {
            b.iter(|| {
                bf.contains(
                    black_box(37),
                );
            });
        }
    );

    c.bench_function(
        "bitfield next",
        |b| {
            b.iter(|| {
                bf.next_from(
                    black_box(23),
                );
            });
        }
    );
}

criterion_group!(benches, bitfield);
criterion_main!(benches);
