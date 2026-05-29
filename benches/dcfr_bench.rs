#![allow(dead_code, unused_imports)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};

#[path = "../src/dcfr.rs"]
mod dcfr;
#[path = "../src/dcfr_parallel.rs"]
mod dcfr_parallel;
#[path = "../src/leduc/mod.rs"]
mod leduc;

fn dcfr_1k(c: &mut Criterion) {
    c.bench_function("dcfr_1k", |b| {
        b.iter(|| black_box(dcfr::train(1_000)));
    });
}

fn dcfr_parallel_1k(c: &mut Criterion) {
    c.bench_function("dcfr_parallel_1k", |b| {
        b.iter(|| black_box(dcfr_parallel::train_parallel(1_000)));
    });
}

criterion_group!(benches, dcfr_1k, dcfr_parallel_1k);
criterion_main!(benches);
