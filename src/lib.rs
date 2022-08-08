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
//! * `atomic` - Enables [`AtomicRng`] & [`atomic_rng`] macros, so
//!   to provide a thread-safe variation of [`Rng`].
//! * `rand` - Provides [`RandCompat`], which implements [`RngCore`]
//!   so to allow for compatibility with `rand` ecosystem of crates
//! * `serialize` - Enables [`Serialize`] and [`Deserialize`] derives on [`Rng`].
//! * `secure` - Enables [`SecureRng`] for providing a more cryptographically
//!   secure source of Rng. Note, this will be slower than [`Rng`] in
//!   throughput, but will produce much higher quality randomness.
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

use std::{
    cell::Cell,
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
    iter::repeat_with,
    ops::{Bound, RangeBounds},
    rc::Rc,
    thread,
};

#[cfg(target_arch = "wasm32")]
use instant::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(feature = "atomic")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "rand")]
use rand_core::RngCore;

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[cfg(all(feature = "serialize", feature = "secure"))]
use serde::{de::Visitor, ser::SerializeStruct, Deserializer};

#[macro_use]
mod methods;

#[cfg(feature = "secure")]
mod buffer;
#[cfg(feature = "rand")]
mod compatibility;
mod entropy;
mod internal;
mod rng;
#[cfg(feature = "secure")]
mod secure_rng;
mod source;
mod traits;

use crate::source::wyrand::WyRand;
pub use crate::{internal::*, rng::*, traits::*};

#[cfg(feature = "secure")]
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
#[cfg(feature = "atomic")]
#[cfg_attr(docsrs, doc(cfg(feature = "atomic")))]
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
/// Else, pass in a `u64` value to get an [`SecureRng`] instance with the seed
/// initialised to that value.
///
/// ```
/// use turborand::*;
///
/// let rand = secure_rng!([1u8; 40]);
///
/// let value = rand.bool();
/// ```
#[cfg(feature = "secure")]
#[cfg_attr(docsrs, doc(cfg(feature = "secure")))]
#[macro_export]
macro_rules! secure_rng {
    () => {
        SecureRng::default()
    };
    ($seed:expr) => {
        SecureRng::with_seed($seed)
    };
}
