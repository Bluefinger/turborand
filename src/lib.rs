//! Fast random number generators.
//!
//! The implementations use [Wyrand](https://github.com/wangyi-fudan/wyhash), a simple and fast
//! generator but **not** cryptographically secure, and [ChaCha8](https://cr.yp.to/chacha.html),
//! a cryptographically secure generator tuned to 8 rounds of the ChaCha algorithm in order to
//! increase throughput considerably without sacrificing too much security, as per the
//! recommendations set out in the [Too Much Crypto](https://eprint.iacr.org/2019/1492.pdf) paper.
//!
//! # Examples
//!
//! Generate a random value:
//!
//! ```
//! use turborand::prelude::*;
//!
//! let rand = Rng::new();
//!
//! let value = rand.bool();
//! ```
//!
//! Sample a value from a list:
//!
//! ```
//! use turborand::prelude::*;
//!
//! let rand = Rng::new();
//!
//! let values = [1, 2, 3, 4, 5];
//!
//! let value = rand.sample(&values);
//! ```
//!
//! Generate a vector with random values:
//!
//! ```
//! use turborand::prelude::*;
//! use std::iter::repeat_with;
//!
//! let rand = Rng::new();
//!
//! let values: Vec<_> = repeat_with(|| rand.f32()).take(10).collect();
//! ```
//!
//! # Features
//!
//! The base crate will always export the [`TurboCore`], [`GenCore`],
//! [`SeededCore`], [`TurboRand`] and [`SecureCore`] traits, and will do
//! so when set as `default-features = false` in the Cargo.toml. By default,
//! it will have `wyrand` feature enabled as the basic PRNG exposed.
//!
//! * **`wyrand`** - Enables [`rng::Rng`], so to provide a
//!   basic, non-threadsafe PRNG. Enabled by default.
//! * **`atomic`** - Enables [`rng::AtomicRng`], so
//!   to provide a thread-safe variation of [`rng::Rng`]. Enables `wyrand`
//!   feature implicitly. **Note**, this is slower than [`rng::Rng`].
//! * **`rand`** - Provides [`compatibility::RandCompat`], which implements [`RngCore`]
//!   so to allow for compatibility with `rand` ecosystem of crates
//! * **`serialize`** - Enables [`Serialize`] and [`Deserialize`] derives on [`rng::Rng`],
//!   [`rng::AtomicRng`] and [`chacha_rng::ChaChaRng`], provided they have their
//!   respective features activated as well.
//! * **`chacha`** - Enables [`chacha_rng::ChaChaRng`] for providing a more cryptographically
//!   secure source of Rng. Note, this will be slower than [`rng::Rng`] in
//!   throughput, but will produce much higher quality randomness.
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[cfg(any(feature = "wyrand", feature = "chacha"))]
use std::{fmt::Debug, rc::Rc};

#[cfg(all(target_arch = "wasm32", any(feature = "wyrand", feature = "chacha")))]
use instant::Instant;
#[cfg(all(
    not(target_arch = "wasm32"),
    any(feature = "wyrand", feature = "chacha")
))]
use std::time::Instant;

#[cfg(feature = "rand")]
use rand_core::RngCore;

#[cfg(all(feature = "serialize", any(feature = "chacha", feature = "wyrand")))]
use serde::{Deserialize, Serialize};

#[cfg(all(feature = "serialize", any(feature = "chacha", feature = "atomic")))]
use serde::de::Visitor;

#[cfg(all(feature = "serialize", feature = "chacha"))]
use serde::ser::SerializeStruct;

#[macro_use]
mod methods;

#[cfg(feature = "chacha")]
mod buffer;
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub mod compatibility;
#[cfg(any(feature = "wyrand", feature = "chacha"))]
mod entropy;
#[cfg(feature = "wyrand")]
mod internal;
#[cfg(any(feature = "wyrand", feature = "atomic"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "wyrand", feature = "atomic"))))]
pub mod rng;
#[cfg(feature = "chacha")]
#[cfg_attr(docsrs, doc(cfg(feature = "chacha")))]
pub mod chacha_rng;
mod source;
mod traits;

pub use traits::{GenCore, SecureCore, SeededCore, TurboCore, TurboRand};

pub mod prelude;
