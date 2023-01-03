# turborand

[![CI](https://github.com/Bluefinger/turborand/actions/workflows/ci.yml/badge.svg)](https://github.com/Bluefinger/turborand/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](https://github.com/Bluefinger/turborand)
[![Cargo](https://img.shields.io/crates/v/turborand.svg)](https://crates.io/crates/turborand)
[![Documentation](https://docs.rs/turborand/badge.svg)](https://docs.rs/turborand)

Fast random number generators.

`turborand`'s internal implementations use [Wyrand](https://github.com/wangyi-fudan/wyhash), a simple and fast
generator but **not** cryptographically secure, and also [ChaCha8](https://cr.yp.to/chacha.html), a cryptographically
secure generator tuned to 8 rounds of the ChaCha algorithm in order to increase throughput considerably without sacrificing
too much security, as per the recommendations set out in the [Too Much Crypto](https://eprint.iacr.org/2019/1492.pdf) paper.

## Examples

```rust
use turborand::prelude::*;

let rand = Rng::new();

if rand.bool() {
    println!("Success! :D");
} else {
    println!("Failure... :(");
}
```

Sample a value from a list:

```rust
use turborand::prelude::*;

let rand = Rng::new();

let values = [1, 2, 3, 4, 5];

let value = rand.sample(&values);
```

Generate a vector with random values:

```rust
use turborand::prelude::*;
use std::iter::repeat_with;

let rand = Rng::new();

let values: Vec<_> = repeat_with(|| rand.f32()).take(10).collect();
```

## `no-std` Compatibility

`turborand` can be exposed to `no-std` environments, however only with reduced capability and feature sets. There'll be no `Default` implementations, and no `new()` constructors, so `Rng`/`ChaChaRng` seeds must be provided by the user from whatever source available on the platform. Some `TurboRand` methods will also not be available unless the `alloc` feature is enabled, which necessitates having a global allocator.

## Performance

`Wyrand` is a pretty fast PRNG, and is a good choice when speed is needed while still having decent statistical properties. Currently, the `turborand` implementation benches extremely well against similar `rand` algorithms. Below is a chart of the `fill_bytes` method performance, tested on Windows 10 x64 on an AMD Ryzen 1700 clocked at 3.7Ghz with 32GB RAM at 3066Mhz.

![fill_bytes benchmark](./assets/fill_bytes_violin.svg)

For filling 2048 byte array buffers, `turborand`'s `Rng` is able to do so in around 170-180ns, whereas `SmallRng` does it between 260-268ns, and `Pcg64Mcg` (the fastest PCG impl on 64bit systems) does it in 305-312ns.

![u64 gen benchmark](./assets/u64_violin.svg)

For generating unbound `u64` values, `turborand` and `fastrand` are equal in performance, which is expected given they both implement the `Wyrand` algorithm, consistently performing around 820-830ps for generating a `u64` value. `SmallRng` performs around 1.16ns, while `Pcg64Mcg` is at 1.35ns.

## Migration from 0.5 to 0.6

Version 0.6 introduces a major reworking of the crate, with code reorganised and also exposed more granularly via features. First things to note:

- All major exports for the crate are now in the `prelude` module. Top level only exports the new traits for `turborand`.
- `Rng` is now split into `Rng` and `AtomicRng`, no more top level generics that required exporting internal traits. `State` trait is now made private and no longer available to be implemented, as this was an internal implementation detail for `WyRand`.
- All previous methods for `Rng` are now implemented in `TurboCore`, `GenCore`, `SeededCore` and `TurboRand` traits. These are part of the `prelude` so as long as they are included, all existing methods will work as expected.
- `Rng` is now under a feature flag, `wyrand`. This is enabled by default however, unless `default-features = false` is applied on the dependency declaration in Cargo.toml.
- _Yeet_ the `rng!`, `atomic_rng!` macros, as these are no longer needed to manage the generics spam that has since been refactored out. Instead, use `::new()`, `::default()` or `::with_seed(seed)` methods instead.

## Migration from 0.6 to 0.7

Version 0.7 hasn't changed much except that the internals module is now fully private (so the `State` traits and `CellState`/`AtomicState` structs are no longer public). They are not accessible from the prelude any more. The removal of these from the public API thus constitutes a breaking change, leading to a new major version.

Also, the serialisation format of `ChaChaRng` has changed, so 0.7 is not compatible with older serialised structs. The plus side is also a flatter serialised format for `ChaChaRng`. Also, `ChaChaRng` is no longer backed by a `Vec` for caching generated entropy, now preferring to use an aligned array for better random number generation at the slight cost of initialisation/cloning performance and increased struct size. This means that the single heap allocation `ChaChaRng` needed is now reduced to zero.

## Migration from 0.7 to 0.8

Version 0.8 seperates the old `Clone` behaviour into two: standard `Clone` which maintains the original state and clones it to the new instance as is (and so both old and new equal to each other), and `ForkableCore` which mutates the state of the original to _fork_ a new instance with a random state generated from the original. **Previous usage of `.clone()` now should make use of `.fork()` instead**. Cloning now should be used where preserving the state of the original to the cloned instance is required.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
