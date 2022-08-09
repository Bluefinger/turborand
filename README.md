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

## Migration from 0.5 to 0.6

Version 0.6 introduces a major reworking of the crate, with code reorganised and also exposed more granularly via features. First things to note:

- All major exports for the crate are now in the `prelude` module. Top level only exports the new traits for `turborand`.
- `Rng` is now split into `Rng` and `AtomicRng`, no more top level generics that required exporting internal traits. `State` trait is now made private and no longer available to be implemented, as this was an internal implementation detail for `WyRand`.
- All previous methods for `Rng` are now implemented in `TurboCore`, `SeededCore` and `TurboRand` traits. These are part of the `prelude` so as long as they are included, all existing methods will work as expected.
- `Rng` is now under a feature flag, `wyrand`. This is enabled by default however, unless `default-features = false` is applied on the dependency declaration in Cargo.toml.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
