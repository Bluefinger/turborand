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
//! * `atomic` - Enables [`AtomicRng`] & [`atomic_rng`] macros, so
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
use rand_core::RngCore;

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
#[macro_export]
macro_rules! secure_rng {
    () => {
        SecureRng::default()
    };
    ($seed:expr) => {
        SecureRng::with_seed($seed)
    };
}

/// A wrapper struct around [`TurboCore`] to allow implementing
/// [`RngCore`] trait in a compatible manner.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct RandCompat<T: TurboCore + Default>(T);

#[cfg(feature = "rand")]
impl<T: TurboCore + Default> RandCompat<T> {
    /// Creates a new [`RandCompat`] with a randomised seed.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    /// use rand_core::RngCore;
    ///
    /// let mut rng = RandCompat::<SecureRng>::new();
    /// let mut buffer = [0u8; 32];
    ///
    /// rng.fill_bytes(&mut buffer);
    ///
    /// assert_ne!(&buffer, &[0u8; 32]);
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(T::default())
    }
}

#[cfg(feature = "rand")]
impl<T: TurboCore + Default> Default for RandCompat<T> {
    /// Initialises a default instance of [`RandCompat`]. Warning, the default is
    /// seeded with a randomly generated state, so this is **not** deterministic.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    /// use rand_core::RngCore;
    ///
    /// let mut rng1 = RandCompat::<SecureRng>::default();
    /// let mut rng2 = RandCompat::<SecureRng>::default();
    ///
    /// assert_ne!(rng1.next_u64(), rng2.next_u64());
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "rand")]
impl<T: TurboCore + Default> RngCore for RandCompat<T> {
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
impl<T: TurboCore + Default> From<T> for RandCompat<T> {
    #[inline]
    fn from(rng: T) -> Self {
        Self(rng)
    }
}

#[cfg(feature = "rand")]
impl From<RandCompat<Rng>> for Rng {
    #[inline]
    fn from(rand: RandCompat<Rng>) -> Self {
        rand.0
    }
}

#[cfg(all(feature = "rand", feature = "atomic"))]
impl From<RandCompat<AtomicRng>> for AtomicRng {
    #[inline]
    fn from(rand: RandCompat<AtomicRng>) -> Self {
        rand.0
    }
}

#[cfg(feature = "rand")]
impl From<RandCompat<SecureRng>> for SecureRng {
    #[inline]
    fn from(rand: RandCompat<SecureRng>) -> Self {
        rand.0
    }
}
