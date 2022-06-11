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
#![warn(missing_docs, rust_2018_idioms)]

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

mod entropy;
mod internal;
mod source;

pub use crate::internal::*;
use crate::{entropy::generate_entropy, source::WyRand};

/// A Random Number generator, powered by the `WyRand` algorithm.
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct Rng<S: State + Debug>(WyRand<S>);

macro_rules! range_unsigned {
    ($value:tt, $bigger:tt, $source:ident, $doc:tt) => {
        #[doc = $doc]
        ///
        /// Panics if the range is empty.
        #[inline]
        pub fn $value(&self, bounds: impl RangeBounds<$value>) -> $value {
            const BITS: $bigger = $value::BITS as $bigger;

            let lower = match bounds.start_bound() {
                Bound::Included(lower) => *lower,
                Bound::Excluded(lower) => lower.saturating_add(1),
                Bound::Unbounded => $value::MIN,
            };
            let upper = match bounds.end_bound() {
                Bound::Included(upper) => upper.saturating_add(1),
                Bound::Excluded(upper) => *upper,
                Bound::Unbounded => $value::MAX,
            };

            assert!(upper > lower, "Range should not be zero sized or invalid");

            let upper = upper.saturating_sub(lower);
            let mut value = self.$source();
            let mut m = (upper as $bigger).wrapping_mul(value as $bigger);
            if (m as $value) < upper {
                let t = (!upper).wrapping_sub(1) % upper;
                while (m as $value) < t {
                    value = self.$source();
                    m = (upper as $bigger).wrapping_mul(value as $bigger);
                }
            }
            (m >> BITS) as $value + lower
        }
    };
}

macro_rules! range_signed {
    ($value:tt, $unsigned:tt, $bigger:tt, $doc:tt) => {
        #[doc = $doc]
        ///
        /// Panics if the range is empty.
        #[inline]
        pub fn $value(&self, bounds: impl RangeBounds<$value>) -> $value {
            let lower = match bounds.start_bound() {
                Bound::Included(lower) => *lower,
                Bound::Excluded(lower) => lower.saturating_add(1),
                Bound::Unbounded => $value::MIN,
            };
            let upper = match bounds.end_bound() {
                Bound::Included(upper) => *upper,
                Bound::Excluded(upper) => upper.saturating_sub(1),
                Bound::Unbounded => $value::MAX,
            };

            assert!(upper > lower, "Range should not be zero sized or invalid");

            let lower = lower.wrapping_sub($value::MIN) as $unsigned;
            let upper = upper.wrapping_sub($value::MIN) as $unsigned;
            self.$unsigned(lower..=upper)
                .wrapping_add($value::MAX as $unsigned) as $value
        }
    };
}

macro_rules! rand_int {
    ($func:ident, $int:ty, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        pub fn $func(&self) -> $int {
            const SIZE: usize = core::mem::size_of::<$int>();
            let mut bytes = [0u8; SIZE];
            let random = self.0.rand();
            bytes.copy_from_slice(&random[..SIZE]);
            <$int>::from_le_bytes(bytes)
        }
    };
}

macro_rules! rand_int_from_bytes {
    ($func:ident, $int:ty, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        pub fn $func(&self) -> $int {
            <$int>::from_le_bytes(self.0.rand())
        }
    };
}

/// Initialises an `Rng` instance with a `CellState`. Not thread safe.
#[macro_export]
macro_rules! rng {
    () => {
        Rng::<CellState>::default()
    };
}

impl<S: State + Debug> Rng<S> {
    /// Creates a new RNG with a randomised seed.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(WyRand::<S>::with_seed(RNG.with(|rng| rng.gen_u64())))
    }

    /// Creates a new RNG with a specific seed value.
    #[inline]
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self(WyRand::<S>::with_seed(seed << 1 | 1))
    }

    /// Reseeds the current thread-local generator.
    #[inline]
    pub fn reseed_local(seed: u64) {
        RNG.with(|rng| rng.reseed(seed));
    }

    /// Reseeds the RNG with a new seed/state.
    #[inline]
    pub fn reseed(&self, seed: u64) {
        self.0.reseed(seed << 1 | 1);
    }

    /// Returns a random `u128` value.
    #[inline]
    pub fn gen_u128(&self) -> u128 {
        let low = self.0.rand();
        let high = self.0.rand();
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&low);
        bytes[8..].copy_from_slice(&high);
        u128::from_le_bytes(bytes)
    }

    /// Returns a random `i128` value.
    #[inline]
    pub fn gen_i128(&self) -> i128 {
        let low = self.0.rand();
        let high = self.0.rand();
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&low);
        bytes[8..].copy_from_slice(&high);
        i128::from_le_bytes(bytes)
    }

    rand_int_from_bytes!(gen_u64, u64, "Returns a random `u64` value.");
    rand_int_from_bytes!(gen_i64, i64, "Returns a random `i64` value.");
    rand_int!(gen_u32, u32, "Returns a random `u32` value.");
    rand_int!(gen_i32, i32, "Returns a random `i32` value.");
    rand_int!(gen_u16, u16, "Returns a random `u16` value.");
    rand_int!(gen_i16, i16, "Returns a random `i16` value.");
    rand_int!(gen_u8, u8, "Returns a random `u8` value.");
    rand_int!(gen_i8, i8, "Returns a random `i8` value.");

    #[cfg(target_pointer_width = "64")]
    rand_int_from_bytes!(gen_usize, usize, "Returns a random `usize` value.");
    #[cfg(not(target_pointer_width = "64"))]
    rand_int!(gen_usize, usize, "Returns a random `usize` value.");

    #[cfg(target_pointer_width = "64")]
    rand_int_from_bytes!(gen_isize, isize, "Returns a random `isize` value.");
    #[cfg(not(target_pointer_width = "64"))]
    rand_int!(gen_isize, isize, "Returns a random `isize` value.");

    range_unsigned!(
        u64,
        u128,
        gen_u64,
        "Returns a random `u64` within a given range bound."
    );
    range_unsigned!(
        u32,
        u64,
        gen_u32,
        "Returns a random `u32` within a given range bound."
    );
    range_unsigned!(
        u16,
        u32,
        gen_u16,
        "Returns a random `u16` within a given range bound."
    );
    range_unsigned!(
        u8,
        u16,
        gen_u8,
        "Returns a random `u8` within a given range bound."
    );

    range_signed!(
        i64,
        u64,
        u128,
        "Returns a random `i64` within a given range bound."
    );
    range_signed!(
        i32,
        u32,
        u64,
        "Returns a random `i32` within a given range bound."
    );
    range_signed!(
        i16,
        u16,
        u32,
        "Returns a random `i16` within a given range bound."
    );
    range_signed!(
        i8,
        u8,
        u16,
        "Returns a random `i8` within a given range bound."
    );

    #[cfg(target_pointer_width = "16")]
    range_unsigned!(
        usize,
        u32,
        gen_usize,
        "Returns a random `usize` within a given range bound."
    );
    #[cfg(target_pointer_width = "32")]
    range_unsigned!(
        usize,
        u64,
        gen_usize,
        "Returns a random `usize` within a given range bound."
    );
    #[cfg(target_pointer_width = "64")]
    range_unsigned!(
        usize,
        u128,
        gen_usize,
        "Returns a random `usize` within a given range bound."
    );

    #[cfg(target_pointer_width = "16")]
    range_signed!(
        isize,
        usize,
        u32,
        "Returns a random `isize` within a given range bound."
    );
    #[cfg(target_pointer_width = "32")]
    range_signed!(
        isize,
        usize,
        u64,
        "Returns a random `isize` within a given range bound."
    );
    #[cfg(target_pointer_width = "64")]
    range_signed!(
        isize,
        usize,
        u128,
        "Returns a random `isize` within a given range bound."
    );

    /// Returns a random `f32` value between `0.0` and `1.0`.
    #[inline]
    pub fn f32(&self) -> f32 {
        (self.gen_u32() as f32) / (u32::MAX as f32)
    }

    /// Returns a random `f32` value between `-1.0` and `1.0`.
    #[inline]
    pub fn f32_normalized(&self) -> f32 {
        (self.gen_i32() as f32) / (i32::MAX as f32)
    }

    /// Returns a random `f64` value between `0.0` and `1.0`.
    #[inline]
    pub fn f64(&self) -> f64 {
        (self.gen_u64() as f64) / (u64::MAX as f64)
    }

    /// Returns a random `f64` value between `-1.0` and `1.0`.
    #[inline]
    pub fn f64_normalized(&self) -> f64 {
        (self.gen_i64() as f64) / (i64::MAX as f64)
    }

    /// Returns a random boolean value.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng = Rng::<CellState>::with_seed(Default::default());
    ///
    /// assert_eq!(rng.bool(), true);
    /// ```
    #[inline]
    pub fn bool(&self) -> bool {
        self.0.rand()[0] % 2 == 0
    }

    /// Returns a boolean value based on a rate. `rate` represents
    /// the chance to return a `true` value, with `0.0` being no
    /// chance and `1.0` will always return true.
    ///
    /// # Panics
    ///
    /// Will panic if `rate` is *not* a value between 0.0 and 1.0.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng = Rng::<CellState>::with_seed(Default::default());
    ///
    /// assert_eq!(rng.chance(1.0), true);
    /// ```
    #[inline]
    pub fn chance(&self, rate: f64) -> bool {
        const SCALE: f64 = 2.0 * (1u64 << 63) as f64;

        assert!((0.0..=1.0).contains(&rate));

        let rate_int = (rate * SCALE) as u64;

        match rate_int {
            u64::MAX => true,
            0 => false,
            _ => self.gen_u64() < rate_int,
        }
    }

    /// Samples a random item from a slice of values.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng = Rng::<CellState>::with_seed(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5, 6];
    ///
    /// assert_eq!(rng.sample(&values), Some(&5));
    /// ```
    #[inline]
    pub fn sample<'a, T>(&self, list: &'a [T]) -> Option<&'a T> {
        match list.len() {
            0 => None,
            // SOUND: Length already known to be 1, therefore index 0 will yield an item
            1 => unsafe { Some(list.get_unchecked(0)) },
            // SOUND: Range is exclusive, so yielded random values will always be a valid index and within bounds
            _ => unsafe { Some(list.get_unchecked(self.usize(..list.len()))) },
        }
    }

    /// [Stochastic Acceptance](https://arxiv.org/abs/1109.3627) implementation of Roulette Wheel
    /// weighted selection. Uses a closure to return a `rate` value for each randomly sampled item
    /// to decide whether to return it or not. The returned `f64` value must be between `0.0` and `1.0`.
    ///
    /// Returns `None` if given an empty list to sample from. For a list containing 1 item, it'll always
    /// return that item regardless.
    ///
    /// # Panics
    ///
    /// If the returned value of the `weight_sampler` closure is not between `0.0` and `1.0`.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng = Rng::<CellState>::with_seed(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5, 6];
    ///
    /// let total = f64::from(values.iter().sum::<i32>());
    ///
    /// assert_eq!(rng.weighted_sample(&values, |&item| item as f64 / total), Some(&4));
    /// ```
    #[inline]
    pub fn weighted_sample<'a, T, F>(&self, list: &'a [T], weight_sampler: F) -> Option<&'a T>
    where
        F: Fn(&'a T) -> f64,
    {
        // Check how many items are in the list
        match list.len() {
            // No values in list, therefore return None.
            0 => None,
            // Only a single value in list, therefore sampling will always yield that value.
            // SOUND: Length already known to be 1, therefore index 0 will yield an item
            1 => unsafe { Some(list.get_unchecked(0)) },
            // Sample the list, flatten the `Option<&T>` and then check if it passes the
            // weighted chance. Keep repeating until `.find` yields a value.
            _ => repeat_with(|| self.sample(list))
                .flatten()
                .find(|&item| self.chance(weight_sampler(item))),
        }
    }
}

impl<S: State + Debug> Default for Rng<S> {
    /// Initialises a default instance of `Rng`. Warning, the default is
    /// seeded with a randomly generated state, so this is **not** deterministic.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng1 = Rng::<CellState>::default();
    /// let rng2 = Rng::<CellState>::default();
    ///
    /// assert_ne!(rng1.u64(..), rng2.u64(..));
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl<S: State + Debug> Clone for Rng<S> {
    /// Clones the RNG by deterministically deriving a new RNG based on the initial
    /// seed.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng1 = Rng::<CellState>::with_seed(Default::default());
    /// let rng2 = Rng::<CellState>::with_seed(Default::default());
    ///
    /// // Use the RNGs once each.
    /// rng1.bool();
    /// rng2.bool();
    ///
    /// let cloned1 = rng1.clone();
    /// let cloned2 = rng2.clone();
    ///
    /// assert_eq!(cloned1.u64(..), cloned2.u64(..));
    /// ```
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: State + Debug> Debug for Rng<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Rng").field(&self.0).finish()
    }
}

thread_local! {
    static RNG: Rc<Rng<CellState>> = Rc::new(Rng(WyRand::<CellState>::with_seed(generate_entropy())));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn no_leaking_debug() {
        let rng = Rng::<CellState>::with_seed(Default::default());

        assert_eq!(format!("{:?}", rng), "Rng(WyRand(CellState))");
    }

    #[test]
    fn index_smoke_testing() {
        let rng = Rng::<CellState>::with_seed(Default::default());

        for _ in 0..1000 {
            let index = rng.usize(4..10);

            assert!(
                (4..10).contains(&index),
                "Must generate a number within 4 and 10, received: {}",
                index
            );
        }

        for _ in 0..1000 {
            let index = rng.usize(..20);

            assert!(
                (..20).contains(&index),
                "Must generate a number within 0 and 20, received: {}",
                index
            );
        }

        for _ in 0..1000 {
            let index = rng.usize(4..=15);

            assert!(
                (4..=15).contains(&index),
                "Must generate a number within 4 and inclusively 15, received: {}",
                index
            );
        }
    }

    #[test]
    fn unbounded_range_smoke_testing() {
        let rng = Rng::<CellState>::with_seed(Default::default());

        for _ in 0..1000 {
            let index = rng.u8(..);

            assert!((..).contains(&index));
        }

        for _ in 0..1000 {
            let index = rng.u64(..);

            assert!((..).contains(&index));
        }

        for _ in 0..1000 {
            let index = rng.usize(..);

            assert!((..).contains(&index));
        }
    }

    #[test]
    fn sample_smoke_testing() {
        let rng = Rng::<CellState>::with_seed(Default::default());

        let indexes: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut sampled = [0; 8];

        for _ in 0..2000 {
            let index = rng.sample(&indexes).unwrap();

            sampled[*index] += 1;
        }

        assert_eq!(
            &sampled,
            &[214, 238, 267, 241, 237, 276, 261, 266],
            "samples will occur across all array items at statistically equal chance"
        );
    }

    #[test]
    fn weighted_sample_smoke_testing() {
        let rng = Rng::<CellState>::with_seed(Default::default());

        let samples: [u32; 5] = [0, 1, 2, 3, 4];

        let sample_total_weight = f64::from(samples.iter().sum::<u32>());

        let actual_histogram: BTreeMap<u32, _> = repeat_with(|| {
            // Select items from the array based on their value divided by the total sum to
            // form their weighting.
            rng.weighted_sample(&samples, |&item| f64::from(item) / sample_total_weight)
        })
        .take(1000)
        .flatten()
        .fold(
            BTreeMap::from_iter(vec![(0, 0)]),
            |mut histogram, &individual| {
                *histogram.entry(individual).or_default() += 1;

                histogram
            },
        );

        // Larger values are expected to be selected more often. 0 should never be
        // selected ever.
        let expected_histogram =
            BTreeMap::from_iter(vec![(0, 0), (1, 92), (2, 207), (3, 294), (4, 407)]);

        assert_eq!(
            actual_histogram, expected_histogram,
            "weighted samples should match in frequency to the expected histogram"
        );
    }
}
