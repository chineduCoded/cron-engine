use criterion::{criterion_group, criterion_main, Criterion};

use cron_engine::cron::field::BitField;

fn bitfield_iter(c: &mut Criterion) {
    let mut bf = BitField::empty(0, 60);

    for i in 0..60 {
        bf.set(i);
    }

    c.bench_function("bitfield/iterate 60 bits", |b| {
        b.iter(|| {
            let mut iter = bf.iter();

            while iter.next().is_some() {}
        });
    });
}

criterion_group!(benches, bitfield_iter);
criterion_main!(benches);
