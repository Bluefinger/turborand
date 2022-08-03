use criterion::{black_box, criterion_main, Criterion};
use turborand::*;

fn turborand_cell_benchmark(c: &mut Criterion) {
    c.bench_function("CellRng new", |b| b.iter(|| black_box(rng!())));
    c.bench_function("CellRng clone", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.clone()))
    });
    c.bench_function("CellRng fill_bytes", |b| {
        let rand = rng!();

        let data = [0u8; 24];

        b.iter_batched(
            || data,
            |mut data| rand.fill_bytes(&mut data),
            criterion::BatchSize::SmallInput,
        )
    });
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
    c.bench_function("CellRng u128 bounded range", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.u128(..20)));
    });
    c.bench_function("CellRng i128 bounded range", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.i128(-20..20)));
    });
    c.bench_function("CellRng u64 bounded range", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.u64(..20)));
    });
    c.bench_function("CellRng i32 bounded range", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.i32(-20..20)));
    });
    c.bench_function("CellRng f64", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.f64()));
    });
    c.bench_function("CellRng f32", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.f32()));
    });
    c.bench_function("CellRng f64 normalized", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.f64_normalized()));
    });
    c.bench_function("CellRng f32 normalized", |b| {
        let rand = rng!();
        b.iter(|| black_box(rand.f32_normalized()));
    });
}

#[cfg(feature = "atomic")]
fn turborand_atomic_benchmark(c: &mut Criterion) {
    c.bench_function("AtomicRng new", |b| b.iter(|| black_box(atomic_rng!())));
    c.bench_function("AtomicRng clone", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.clone()))
    });
    c.bench_function("AtomicRng fill_bytes", |b| {
        let rand = atomic_rng!();

        let data = [0u8; 24];

        b.iter_batched(
            || data,
            |mut data| rand.fill_bytes(&mut data),
            criterion::BatchSize::SmallInput,
        )
    });
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
    c.bench_function("AtomicRng u128 bounded range", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.u128(..20)));
    });
    c.bench_function("AtomicRng i128 bounded range", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.i128(-20..20)));
    });
    c.bench_function("AtomicRng u64 bounded range", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.u64(..20)));
    });
    c.bench_function("AtomicRng i32 bounded range", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.i32(-20..20)));
    });
    c.bench_function("AtomicRng f64", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.f64()));
    });
    c.bench_function("AtomicRng f32", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.f32()));
    });
    c.bench_function("AtomicRng f64 normalized", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.f64_normalized()));
    });
    c.bench_function("AtomicRng f32 normalized", |b| {
        let rand = atomic_rng!();
        b.iter(|| black_box(rand.f32_normalized()));
    });
}

#[cfg(feature = "secure")]
fn turborand_secure_benchmark(c: &mut Criterion) {
    c.bench_function("SecureRng new", |b| {
        b.iter(|| black_box(secure_rng!()));
    });
    c.bench_function("SecureRng clone", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.clone()));
    });
    c.bench_function("SecureRng fill_bytes", |b| {
        let rand = secure_rng!();

        let data = [0u8; 24];

        b.iter_batched(
            || data,
            |mut data| rand.fill_bytes(&mut data),
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("SecureRng gen_u128", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.gen_u128()));
    });
    c.bench_function("SecureRng gen_u64", |b| {
        let rand = secure_rng!();

        b.iter(|| black_box(rand.gen_u64()));
    });
    c.bench_function("SecureRng gen_u32", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.gen_u32()));
    });
    c.bench_function("SecureRng gen_u16", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.gen_u16()));
    });
    c.bench_function("SecureRng gen_u8", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.gen_u8()));
    });
    c.bench_function("SecureRng bool", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.bool()));
    });
    c.bench_function("SecureRng usize range", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.usize(..20)));
    });
    c.bench_function("SecureRng isize range", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.isize(-10..10)));
    });
    c.bench_function("SecureRng u128 bounded range", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.u128(..20)));
    });
    c.bench_function("SecureRng i128 bounded range", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.i128(-20..20)));
    });
    c.bench_function("SecureRng u64 bounded range", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.u64(..20)));
    });
    c.bench_function("SecureRng i32 bounded range", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.i32(-20..20)));
    });
    c.bench_function("SecureRng f64", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.f64()));
    });
    c.bench_function("SecureRng f32", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.f32()));
    });
    c.bench_function("SecureRng f64 normalized", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.f64_normalized()));
    });
    c.bench_function("SecureRng f32 normalized", |b| {
        let rand = secure_rng!();
        b.iter(|| black_box(rand.f32_normalized()));
    });
}

pub fn benches() {
    let mut criterion: Criterion<_> = Criterion::default().configure_from_args();
    turborand_cell_benchmark(&mut criterion);
    #[cfg(feature = "atomic")]
    turborand_atomic_benchmark(&mut criterion);
    #[cfg(feature = "secure")]
    turborand_secure_benchmark(&mut criterion);
}

criterion_main!(benches);
