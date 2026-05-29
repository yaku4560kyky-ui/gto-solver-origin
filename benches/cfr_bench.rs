use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[path = "../src/cfr.rs"]
mod cfr;
#[path = "../src/cfr_plus.rs"]
mod cfr_plus;

fn vanilla_cfr_10k(c: &mut Criterion) {
    c.bench_function("vanilla_cfr_10k", |b| {
        b.iter(|| black_box(cfr::train(10_000)));
    });
}

fn cfr_plus_10k(c: &mut Criterion) {
    c.bench_function("cfr_plus_10k", |b| {
        b.iter(|| black_box(cfr_plus::train(10_000)));
    });
}

criterion_group!(benches, vanilla_cfr_10k, cfr_plus_10k);
criterion_main!(benches);
