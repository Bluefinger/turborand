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
use turborand::*;

let rand = rng!();

if rand.bool() {
    println!("Success! :D");
} else {
    println!("Failure... :(");
}
```

Sample a value from a list:

```rust
use turborand::*;

let rand = rng!();

let values = [1, 2, 3, 4, 5];

let value = rand.sample(&values);
```

Generate a vector with random values:

```rust
use turborand::*;
use std::iter::repeat_with;

let rand = rng!();

let values: Vec<_> = repeat_with(|| rand.f32()).take(10).collect();
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
