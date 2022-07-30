//! A simple and fast random number generator.
//!
//! The implementation uses [Wyrand](https://github.com/wangyi-fudan/wyhash), a simple and fast
//! generator but **not** cryptographically secure.
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
//! * `atomic` - Enables [`AtomicState`] variants & [`atomic_rng`] macros, so
//!   to provide a thread-safe variation of [`Rng`].
//! * `rand` - Provides [`RandCompat`], which implements [`RngCore`] and [`SeedableRng`]
//!   so to allow for compatibility with `rand` ecosystem of crates
//! * `serialize` - Enables [`Serialize`] and [`Deserialize`] derives on [`Rng`].
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
use rand_core::{RngCore, SeedableRng};

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[macro_use]
mod methods;

mod buffer;
mod entropy;
mod internal;
mod rng;
mod secure_rng;
mod source;
mod traits;

use crate::source::wyrand::WyRand;
pub use crate::{internal::*, rng::*, secure_rng::*, traits::*};

/// Initialises an [`Rng`] instance with a [`CellState`]. Not thread safe.
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
        Rng::<CellState<u64>>::default()
    };
    ($seed:expr) => {
        Rng::<CellState<u64>>::with_seed($seed)
    };
}

/// Initialises an [`Rng`] instance with an [`AtomicState`]. Thread safe.
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
/// Else, pass in a `u64` value to get an [`Rng`] instance with the seed
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
        Rng::<AtomicState>::default()
    };
    ($seed:expr) => {
        Rng::<AtomicState>::with_seed($seed)
    };
}

/// A wrapper struct around [`Rng<CellState>`] to allow implementing
/// [`RngCore`] and [`SeedableRng`] traits in a compatible manner.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct RandCompat(Rng<CellState<u64>>);

#[cfg(feature = "rand")]
impl RandCompat {
    /// Creates a new [`RandCompat`] with a randomised seed.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Rng::default())
    }
}

#[cfg(feature = "rand")]
impl Default for RandCompat {
    /// Initialises a default instance of [`RandCompat`]. Warning, the default is
    /// seeded with a randomly generated state, so this is **not** deterministic.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    /// use rand_core::RngCore;
    ///
    /// let mut rng1 = RandCompat::default();
    /// let mut rng2 = RandCompat::default();
    ///
    /// assert_ne!(rng1.next_u64(), rng2.next_u64());
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rand")]
impl RngCore for RandCompat {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.0.gen_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.0.gen_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.0.fill_bytes(dest);
        Ok(())
    }
}

#[cfg(feature = "rand")]
impl SeedableRng for RandCompat {
    type Seed = [u8; core::mem::size_of::<u64>()];

    #[inline]
    #[must_use]
    fn from_seed(seed: Self::Seed) -> Self {
        Self(Rng::with_seed(u64::from_be_bytes(seed)))
    }
}

#[cfg(feature = "rand")]
impl From<Rng<CellState<u64>>> for RandCompat {
    #[inline]
    fn from(rng: Rng<CellState<u64>>) -> Self {
        Self(rng)
    }
}

#[cfg(feature = "rand")]
impl From<RandCompat> for Rng<CellState<u64>> {
    #[inline]
    fn from(rand: RandCompat) -> Self {
        rand.0
    }
}
