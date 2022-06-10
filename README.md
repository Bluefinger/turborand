# turborand

[![License](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](
https://github.com/Bluefinger/turborand)

A fast random number generator.

`turborand`'s internal implementation uses [Wyrand](https://github.com/wangyi-fudan/wyhash), a simple and fast
generator but **not** cryptographically secure.

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

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
