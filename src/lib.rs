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
//! use turborand::*;
//!
//! let rand = rng!();
//!
//! let value = rand.bool();
//! ```
//!
//! Sample a value from a list:
//!
//! ```
//! use turborand::*;
//!
//! let rand = rng!();
//!
//! let values = [1, 2, 3, 4, 5];
//!
//! let value = rand.sample(&values);
//! ```
//!
//! Generate a vector with random values:
//!
//! ```
//! use turborand::*;
//! use std::iter::repeat_with;
//!
//! let rand = rng!();
//!
//! let values: Vec<_> = repeat_with(|| rand.f32()).take(10).collect();
//! ```
//!
//! # Features
//! 
//! The base crate will always export the [`TurboCore`], [`SeededCore`],
//! [`TurboRand`] and [`SecureCore`] traits, and will do so when set as
//! `default-features = false` in the Cargo.toml. By default, it will
//! have `wyrand` feature enabled as the basic PRNG exposed.
//!
//! * `wyrand` - Enables [`Rng`] & [`rng!`] macros, so to provide a
//!   basic, non-threadsafe PRNG. Enabled by default.
//! * `atomic` - Enables [`AtomicRng`] & [`atomic_rng!`] macros, so
//!   to provide a thread-safe variation of [`Rng`]. Enables `wyrand`
//!   feature implicitly. **Note**, this is slower than [`Rng`].
//! * `rand` - Provides [`RandCompat`], which implements [`RngCore`]
//!   so to allow for compatibility with `rand` ecosystem of crates
//! * `serialize` - Enables [`Serialize`] and [`Deserialize`] derives on [`Rng`],
//!   [`AtomicRng`] and [`SecureRng`], provided they have their
//!   respective features activated as well.
//! * `chacha` - Enables [`SecureRng`] for providing a more cryptographically
//!   secure source of Rng. Note, this will be slower than [`Rng`] in
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
mod compatibility;
#[cfg(any(feature = "wyrand", feature = "chacha"))]
mod entropy;
#[cfg(feature = "wyrand")]
mod internal;
#[cfg(feature = "wyrand")]
mod rng;
#[cfg(feature = "chacha")]
mod secure_rng;
mod source;
mod traits;

pub use crate::traits::*;

#[cfg(feature = "wyrand")]
pub use crate::{internal::*, rng::*};

#[cfg(feature = "chacha")]
pub use crate::secure_rng::*;

#[cfg(feature = "rand")]
pub use crate::compatibility::*;

/// Initialises an [`Rng`] instance. Not thread safe.
/// Can be used with and without a seed value. If invoked without
/// a seed value, it will initialise a default instance with a generated
/// seed.
///
/// # Example
///
/// ```
/// use turborand::*;
///
/// let rand = rng!();
///
/// let value = rand.bool();
/// ```
///
/// Else, pass in a `u64` value to get an [`Rng`] instance with the seed
/// initialised to that value.
///
/// ```
/// use turborand::*;
///
/// let rand = rng!(128u64);
///
/// let value = rand.bool();
/// ```
#[cfg(feature = "wyrand")]
#[cfg_attr(docsrs, doc(cfg(feature = "wyrand")))]
#[macro_export]
macro_rules! rng {
    () => {
        Rng::default()
    };
    ($seed:expr) => {
        Rng::with_seed($seed)
    };
}

/// Initialises an [`AtomicRng`] instance. Thread safe.
/// Can be used with and without a seed value. If invoked without
/// a seed value, it will initialise a default instance with a generated
/// seed.
///
/// # Example
///
/// ```
/// use turborand::*;
/// use std::sync::Arc;
///
/// let rand = Arc::new(atomic_rng!());
///
/// let value = rand.bool();
/// ```
///
/// Else, pass in a `u64` value to get an [`AtomicRng`] instance with the seed
/// initialised to that value.
///
/// ```
/// use turborand::*;
/// use std::sync::Arc;
///
/// let rand = Arc::new(atomic_rng!(128u64));
///
/// let value = rand.bool();
/// ```
#[cfg(all(feature = "wyrand", feature = "atomic"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "wyrand", feature = "atomic"))))]
#[macro_export]
macro_rules! atomic_rng {
    () => {
        AtomicRng::default()
    };
    ($seed:expr) => {
        AtomicRng::with_seed($seed)
    };
}

/// Initialises a [`SecureRng`] instance. Not thread safe.
/// Can be used with and without a seed value. If invoked without
/// a seed value, it will initialise a default instance with a generated
/// seed.
///
/// # Example
///
/// ```
/// use turborand::*;
///
/// let rand = secure_rng!();
///
/// let value = rand.bool();
/// ```
///
/// Else, pass in a `[u8; 40]` array to get an [`SecureRng`] instance with the seed
/// initialised to that value.
///
/// ```
/// use turborand::*;
///
/// let rand = secure_rng!([1u8; 40]);
///
/// let value = rand.bool();
/// ```
#[cfg(feature = "chacha")]
#[cfg_attr(docsrs, doc(cfg(feature = "chacha")))]
#[macro_export]
macro_rules! secure_rng {
    () => {
        SecureRng::default()
    };
    ($seed:expr) => {
        SecureRng::with_seed($seed)
    };
}
