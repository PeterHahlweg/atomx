use std::sync::{Arc, atomic::AtomicU64};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use haphazard::HazardPointer;


fn hazard_pointer_new(c: &mut Criterion) {
    c.bench_function("HazardPointer::new()", |b| b.iter(|| {
        black_box(HazardPointer::new())
    }));
}

fn atomic_pointer_clone(c: &mut Criterion) {
    let atomic = Arc::new(AtomicU64::new(0));
    c.bench_function("AtomicPointer::clone()", |b| b.iter(|| {
        black_box(black_box(atomic.clone()))
    }));
}

criterion_group!(benches, hazard_pointer_new, atomic_pointer_clone);
criterion_main!(benches);