use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use cron_engine::cron::{
    field::BitField, ir::FieldMatcher, scheduler::navigator::NumericNavigator,
};

fn matcher() -> FieldMatcher {
    let mut bf = BitField::empty(0, 60);

    for i in (0..60).step_by(5) {
        bf.set(i);
    }

    FieldMatcher::from(bf)
}

fn navigator(c: &mut Criterion) {
    let matcher = matcher();
    let nav = NumericNavigator::new(&matcher);

    let mut group = c.benchmark_group("navigator");

    group.bench_function("min", |b| {
        b.iter(|| nav.min());
    });

    group.bench_function("max", |b| {
        b.iter(|| nav.max());
    });

    group.bench_function("next", |b| {
        b.iter(|| nav.next(black_box(37)));
    });

    group.finish();
}

criterion_group!(benches, navigator);
criterion_main!(benches);
