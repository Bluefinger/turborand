use criterion::{black_box, criterion_main, Criterion};
use turborand::prelude::*;

#[cfg(feature = "wyrand")]
fn turborand_cell_benchmark(c: &mut Criterion) {
    c.bench_function("CellRng new", |b| b.iter(|| black_box(Rng::default())));
    c.bench_function("CellRng clone", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.clone()))
    });
    c.bench_function("CellRng fork", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.fork()))
    });
    c.bench_function("CellRng fill_bytes", |b| {
        let rand = Rng::default();

        let data = [0u8; 24];

        b.iter_batched(
            || data,
            |mut data| rand.fill_bytes(&mut data),
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("CellRng fill_bytes large", |b| {
        let rand = Rng::default();

        let data = [0u8; 2048];

        b.iter_batched_ref(
            || data,
            |data| rand.fill_bytes(data),
            criterion::BatchSize::LargeInput,
        )
    });
    c.bench_function("CellRng gen_u128", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.gen_u128()));
    });
    c.bench_function("CellRng gen_u64", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.gen_u64()));
    });
    c.bench_function("CellRng gen_u32", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.gen_u32()));
    });
    c.bench_function("CellRng gen_u16", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.gen_u16()));
    });
    c.bench_function("CellRng gen_u8", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.gen_u8()));
    });
    c.bench_function("CellRng bool", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.bool()));
    });
    c.bench_function("CellRng usize range", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.usize(..20)));
    });
    c.bench_function("CellRng index", |b| {
        let rand = Rng::default();
        let bound = 20;
        b.iter(|| black_box(rand.index(bound)));
    });
    c.bench_function("CellRng isize range", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.isize(-10..10)));
    });
    c.bench_function("CellRng u128 bounded range", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.u128(..20)));
    });
    c.bench_function("CellRng i128 bounded range", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.i128(-20..20)));
    });
    c.bench_function("CellRng u64 bounded range", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.u64(..20)));
    });
    c.bench_function("CellRng i32 bounded range", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.i32(-20..20)));
    });
    c.bench_function("CellRng f64", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.f64()));
    });
    c.bench_function("CellRng f32", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.f32()));
    });
    c.bench_function("CellRng f64 normalized", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.f64_normalized()));
    });
    c.bench_function("CellRng f32 normalized", |b| {
        let rand = Rng::default();
        b.iter(|| black_box(rand.f32_normalized()));
    });
    c.bench_function("CellRng sample", |b| {
        let rand = Rng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample(&data)))
    });
    c.bench_function("CellRng sample one", |b| {
        let rand = Rng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample(&data[0..1])))
    });
    c.bench_function("CellRng sample iter", |b| {
        let rand = Rng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample_iter(data.iter())))
    });
    c.bench_function("CellRng sample iter one", |b| {
        let rand = Rng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample_iter(data[0..1].iter())))
    });
    c.bench_function("CellRng weighted sample", |b| {
        let rand = Rng::default();

        let mut data = [0.0; 2048];

        data.iter_mut().for_each(|slot| *slot = rand.f64());

        b.iter(|| black_box(rand.weighted_sample(&data, |(&item, _)| item)))
    });
    c.bench_function("CellRng weighted sample mut", |b| {
        let rand = Rng::default();

        let mut data = [0.0; 2048];

        data.iter_mut().for_each(|slot| *slot = rand.f64());

        b.iter(|| {
            let _ = rand.weighted_sample_mut(&mut data, |(&item, _)| item);
        })
    });
}

#[cfg(feature = "atomic")]
fn turborand_atomic_benchmark(c: &mut Criterion) {
    c.bench_function("AtomicRng new", |b| {
        b.iter(|| black_box(AtomicRng::default()))
    });
    c.bench_function("AtomicRng clone", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.clone()))
    });
    c.bench_function("AtomicRng fork", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.fork()))
    });
    c.bench_function("AtomicRng fill_bytes", |b| {
        let rand = AtomicRng::default();

        let data = [0u8; 24];

        b.iter_batched(
            || data,
            |mut data| rand.fill_bytes(&mut data),
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("AtomicRng fill_bytes large", |b| {
        let rand = AtomicRng::default();

        let data = [0u8; 2048];

        b.iter_batched_ref(
            || data,
            |data| rand.fill_bytes(data),
            criterion::BatchSize::LargeInput,
        )
    });
    c.bench_function("AtomicRng gen_u128", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.gen_u128()));
    });
    c.bench_function("AtomicRng gen_u64", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.gen_u64()));
    });
    c.bench_function("AtomicRng gen_u32", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.gen_u32()));
    });
    c.bench_function("AtomicRng gen_u16", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.gen_u16()));
    });
    c.bench_function("AtomicRng gen_u8", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.gen_u8()));
    });
    c.bench_function("AtomicRng bool", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.bool()));
    });
    c.bench_function("AtomicRng usize range", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.usize(..20)));
    });
    c.bench_function("AtomicRng index", |b| {
        let rand = AtomicRng::default();
        let bound = 20;
        b.iter(|| black_box(rand.index(bound)));
    });
    c.bench_function("AtomicRng isize range", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.isize(-10..10)));
    });
    c.bench_function("AtomicRng u128 bounded range", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.u128(..20)));
    });
    c.bench_function("AtomicRng i128 bounded range", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.i128(-20..20)));
    });
    c.bench_function("AtomicRng u64 bounded range", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.u64(..20)));
    });
    c.bench_function("AtomicRng i32 bounded range", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.i32(-20..20)));
    });
    c.bench_function("AtomicRng f64", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.f64()));
    });
    c.bench_function("AtomicRng f32", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.f32()));
    });
    c.bench_function("AtomicRng f64 normalized", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.f64_normalized()));
    });
    c.bench_function("AtomicRng f32 normalized", |b| {
        let rand = AtomicRng::default();
        b.iter(|| black_box(rand.f32_normalized()));
    });
    c.bench_function("AtomicRng sample", |b| {
        let rand = AtomicRng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample(&data)))
    });
    c.bench_function("AtomicRng sample one", |b| {
        let rand = AtomicRng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample(&data[0..1])))
    });
    c.bench_function("AtomicRng sample iter", |b| {
        let rand = AtomicRng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample_iter(data.iter())))
    });
    c.bench_function("AtomicRng sample iter one", |b| {
        let rand = AtomicRng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample_iter(data[0..1].iter())))
    });
    c.bench_function("AtomicRng weighted sample", |b| {
        let rand = AtomicRng::default();

        let mut data = [0.0; 2048];

        data.iter_mut().for_each(|slot| *slot = rand.f64());

        b.iter(|| black_box(rand.weighted_sample(&data, |(&item, _)| item)))
    });
    c.bench_function("AtomicRng weighted sample mut", |b| {
        let rand = AtomicRng::default();

        let mut data = [0.0; 2048];

        data.iter_mut().for_each(|slot| *slot = rand.f64());

        b.iter(|| {
            let _ = rand.weighted_sample_mut(&mut data, |(&item, _)| item);
        })
    });
}

#[cfg(feature = "chacha")]
fn turborand_chacha_benchmark(c: &mut Criterion) {
    c.bench_function("ChaChaRng new", |b| {
        b.iter(|| black_box(ChaChaRng::default()));
    });
    c.bench_function("ChaChaRng clone", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.clone()));
    });
    c.bench_function("ChaChaRng fork", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.fork()));
    });
    c.bench_function("ChaChaRng fill_bytes", |b| {
        let rand = ChaChaRng::default();

        let data = [0u8; 24];

        b.iter_batched(
            || data,
            |mut data| rand.fill_bytes(&mut data),
            criterion::BatchSize::SmallInput,
        )
    });
    c.bench_function("ChaChaRng fill_bytes large", |b| {
        let rand = ChaChaRng::default();

        let data = [0u8; 2048];

        b.iter_batched_ref(
            || data,
            |data| rand.fill_bytes(data),
            criterion::BatchSize::LargeInput,
        )
    });
    c.bench_function("ChaChaRng gen_u128", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.gen_u128()));
    });
    c.bench_function("ChaChaRng gen_u64", |b| {
        let rand = ChaChaRng::default();

        b.iter(|| black_box(rand.gen_u64()));
    });
    c.bench_function("ChaChaRng gen_u32", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.gen_u32()));
    });
    c.bench_function("ChaChaRng gen_u16", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.gen_u16()));
    });
    c.bench_function("ChaChaRng gen_u8", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.gen_u8()));
    });
    c.bench_function("ChaChaRng bool", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.bool()));
    });
    c.bench_function("ChaChaRng usize range", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.usize(..20)));
    });
    c.bench_function("ChaChaRng index", |b| {
        let rand = ChaChaRng::default();
        let bound = 20;
        b.iter(|| black_box(rand.index(bound)));
    });
    c.bench_function("ChaChaRng isize range", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.isize(-10..10)));
    });
    c.bench_function("ChaChaRng u128 bounded range", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.u128(..20)));
    });
    c.bench_function("ChaChaRng i128 bounded range", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.i128(-20..20)));
    });
    c.bench_function("ChaChaRng u64 bounded range", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.u64(..20)));
    });
    c.bench_function("ChaChaRng i32 bounded range", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.i32(-20..20)));
    });
    c.bench_function("ChaChaRng f64", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.f64()));
    });
    c.bench_function("ChaChaRng f32", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.f32()));
    });
    c.bench_function("ChaChaRng f64 normalized", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.f64_normalized()));
    });
    c.bench_function("ChaChaRng f32 normalized", |b| {
        let rand = ChaChaRng::default();
        b.iter(|| black_box(rand.f32_normalized()));
    });
    c.bench_function("ChaChaRng sample", |b| {
        let rand = ChaChaRng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample(&data)))
    });
    c.bench_function("ChaChaRng sample one", |b| {
        let rand = ChaChaRng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample(&data[0..1])))
    });
    c.bench_function("ChaChaRng sample iter", |b| {
        let rand = ChaChaRng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample_iter(data.iter())))
    });
    c.bench_function("ChaChaRng sample iter one", |b| {
        let rand = ChaChaRng::default();

        let mut data = [0u8; 2048];

        rand.fill_bytes(&mut data);

        b.iter(|| black_box(rand.sample_iter(data[0..1].iter())))
    });
    c.bench_function("ChaChaRng weighted sample", |b| {
        let rand = ChaChaRng::default();

        let mut data = [0.0; 2048];

        data.iter_mut().for_each(|slot| *slot = rand.f64());

        b.iter(|| black_box(rand.weighted_sample(&data, |(&item, _)| item)))
    });
    c.bench_function("ChaChaRng weighted sample mut", |b| {
        let rand = ChaChaRng::default();

        let mut data = [0.0; 2048];

        data.iter_mut().for_each(|slot| *slot = rand.f64());

        b.iter(|| {
            let _ = rand.weighted_sample_mut(&mut data, |(&item, _)| item);
        })
    });
}

pub fn benches() {
    let mut criterion: Criterion<_> = Criterion::default().configure_from_args();
    #[cfg(feature = "wyrand")]
    turborand_cell_benchmark(&mut criterion);
    #[cfg(feature = "atomic")]
    turborand_atomic_benchmark(&mut criterion);
    #[cfg(feature = "chacha")]
    turborand_chacha_benchmark(&mut criterion);
}

criterion_main!(benches);
