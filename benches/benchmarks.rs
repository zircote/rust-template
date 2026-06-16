//! Criterion micro-benchmarks for the `rust_template` example API.
//!
//! Run with `cargo bench` (or `just bench`). Results land in
//! `target/criterion/`; CI archives them and `benchmark-regression.yml`
//! compares them against the `main` baseline.
//!
//! NOTE: these benchmark the placeholder `add` / `divide` / `process`
//! functions — replace them when you replace the example API.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use rust_template::{add, divide, process};

fn benchmarks(c: &mut Criterion) {
    c.bench_function("add", |b| b.iter(|| add(black_box(2), black_box(40))));
    c.bench_function("divide", |b| b.iter(|| divide(black_box(84), black_box(2))));
    c.bench_function("process", |b| b.iter(|| process(black_box("42"))));
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
