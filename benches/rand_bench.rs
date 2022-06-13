use criterion::{black_box, criterion_group, criterion_main, Criterion};
use turborand::*;

fn turborand_benchmark(c: &mut Criterion) {
    c.bench_function("CellRng new", |b| b.iter(|| black_box(rng!())));
    c.bench_function("CellRng gen_u128", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.gen_u128()));
    });
    c.bench_function("CellRng gen_u64", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.gen_u64()));
    });
    c.bench_function("CellRng gen_u32", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.gen_u32()));
    });
    c.bench_function("CellRng gen_u16", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.gen_u16()));
    });
    c.bench_function("CellRng gen_u8", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.gen_u8()));
    });
    c.bench_function("CellRng bool", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.bool()));
    });
    c.bench_function("CellRng usize range", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.usize(..20)));
    });
    c.bench_function("CellRng isize range", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.isize(-10..10)));
    });
    c.bench_function("CellRng u64 unbounded range", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.u64(..)));
    });
    c.bench_function("CellRng i32 unbounded range", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.i32(..)));
    });
}

#[cfg(feature = "atomic")]
fn turborand_atomic_benchmark(c: &mut Criterion) {
    c.bench_function("AtomicRng new", |b| b.iter(|| black_box(atomic_rng!())));
    c.bench_function("AtomicRng gen_u128", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.gen_u128()));
    });
    c.bench_function("AtomicRng gen_u64", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.gen_u64()));
    });
    c.bench_function("AtomicRng gen_u32", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.gen_u32()));
    });
    c.bench_function("AtomicRng gen_u16", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.gen_u16()));
    });
    c.bench_function("AtomicRng gen_u8", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.gen_u8()));
    });
    c.bench_function("AtomicRng bool", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.bool()));
    });
    c.bench_function("AtomicRng usize range", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.usize(..20)));
    });
    c.bench_function("AtomicRng isize range", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.isize(-10..10)));
    });
    c.bench_function("AtomicRng u64 unbounded range", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.u64(..)));
    });
    c.bench_function("AtomicRng i32 unbounded range", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.i32(..)));
    });
}

criterion_group!(benches, turborand_benchmark, turborand_atomic_benchmark);
criterion_main!(benches);
