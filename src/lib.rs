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
    convert::{TryFrom, TryInto},
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

mod entropy;
mod internal;
mod source;

pub use crate::internal::*;
use crate::{entropy::generate_entropy, source::WyRand};

/// A Random Number generator, powered by the `WyRand` algorithm.
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Rng<S: State + Debug>(WyRand<S>);

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
        Rng::<CellState>::default()
    };
    ($seed:expr) => {
        Rng::<CellState>::with_seed($seed)
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

impl<S: State + Debug> Rng<S> {
    /// Creates a new [`Rng`] with a randomised seed.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(WyRand::<S>::with_seed(RNG.with(|rng| rng.gen_u64())))
    }

    /// Creates a new [`Rng`] with a specific seed value.
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

    /// Reseeds the [`Rng`] with a new seed/state.
    #[inline]
    pub fn reseed(&self, seed: u64) {
        self.0.reseed(seed << 1 | 1);
    }

    rand_int!(gen_u128, u128, "Returns a random `u128` value.");
    rand_int!(gen_i128, i128, "Returns a random `i128` value.");
    rand_int!(gen_u64, u64, "Returns a random `u64` value.");
    rand_int!(gen_i64, i64, "Returns a random `i64` value.");
    rand_int!(gen_u32, u32, "Returns a random `u32` value.");
    rand_int!(gen_i32, i32, "Returns a random `i32` value.");
    rand_int!(gen_u16, u16, "Returns a random `u16` value.");
    rand_int!(gen_i16, i16, "Returns a random `i16` value.");
    rand_int!(gen_u8, u8, "Returns a random `u8` value.");
    rand_int!(gen_i8, i8, "Returns a random `i8` value.");

    #[cfg(target_pointer_width = "64")]
    rand_int!(gen_usize, usize, "Returns a random `usize` value.");
    #[cfg(not(target_pointer_width = "64"))]
    rand_int!(gen_usize, usize, "Returns a random `usize` value.");

    #[cfg(target_pointer_width = "64")]
    rand_int!(gen_isize, isize, "Returns a random `isize` value.");
    #[cfg(not(target_pointer_width = "64"))]
    rand_int!(gen_isize, isize, "Returns a random `isize` value.");

    /// Fills a mutable buffer with random bytes.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rand = rng!(Default::default());
    ///
    /// let mut bytes = [0u8; 10];
    ///
    /// rand.fill_bytes(&mut bytes);
    ///
    /// assert_ne!(&bytes, &[0u8; 10], "output should not match a zeroed array");
    /// ```
    #[inline]
    pub fn fill_bytes<B: AsMut<[u8]>>(&self, mut buffer: B) {
        let mut buffer = buffer.as_mut();
        let mut length: usize = buffer.len();
        while length > 0 {
            let output = self.0.rand();
            let fill = output.len().min(length);
            buffer[..fill].copy_from_slice(&output[..fill]);
            buffer = &mut buffer[fill..];
            length -= fill;
        }
    }

    /// Returns a random `u128` within a given range bound.
    ///
    /// # Panics
    ///
    /// Panics if the range is empty or invalid.
    #[inline]
    pub fn u128(&self, bounds: impl RangeBounds<u128>) -> u128 {
        let lower = match bounds.start_bound() {
            Bound::Included(lower) => *lower,
            Bound::Excluded(lower) => lower.saturating_add(1),
            Bound::Unbounded => u128::MIN,
        };
        let upper = match bounds.end_bound() {
            Bound::Included(upper) => *upper,
            Bound::Excluded(upper) => upper.saturating_sub(1),
            Bound::Unbounded => u128::MAX,
        };

        assert!(lower <= upper, "Range should not be zero sized or invalid");

        match (lower, upper) {
            (u128::MIN, u128::MAX) => self.gen_u128(),
            (_, _) => {
                let range = upper.wrapping_sub(lower).wrapping_add(1);
                let mut value = self.gen_u128();
                let mut high = multiply_high_u128(value, range);
                let mut low = value.wrapping_mul(range);
                if low < range {
                    let t = range.wrapping_neg() % range;
                    while low < t {
                        value = self.gen_u128();
                        high = multiply_high_u128(value, range);
                        low = value.wrapping_mul(range);
                    }
                }
                lower.wrapping_add(high)
            }
        }
    }

    /// Returns a random `i128` within a given range bound.
    ///
    /// # Panics
    ///
    /// Panics if the range is empty or invalid.
    #[inline]
    pub fn i128(&self, bounds: impl RangeBounds<i128>) -> i128 {
        let lower = match bounds.start_bound() {
            Bound::Included(lower) => *lower,
            Bound::Excluded(lower) => lower.saturating_add(1),
            Bound::Unbounded => i128::MIN,
        };
        let upper = match bounds.end_bound() {
            Bound::Included(upper) => *upper,
            Bound::Excluded(upper) => upper.saturating_sub(1),
            Bound::Unbounded => i128::MAX,
        };

        assert!(lower <= upper, "Range should not be zero sized or invalid");

        match (lower, upper) {
            (i128::MIN, i128::MAX) => self.gen_i128(),
            (_, _) => {
                let range = upper.wrapping_sub(lower).wrapping_add(1) as u128;
                let mut value = self.gen_u128();
                let mut high = multiply_high_u128(value, range);
                let mut low = value.wrapping_mul(range);
                if low < range {
                    let t = range.wrapping_neg() % range;
                    while low < t {
                        value = self.gen_u128();
                        high = multiply_high_u128(value, range);
                        low = value.wrapping_mul(range);
                    }
                }
                lower.wrapping_add(high as i128)
            }
        }
    }

    range_int!(
        u64,
        u64,
        gen_u64,
        mod_u64,
        "Returns a random `u64` within a given range bound."
    );
    range_int!(
        u32,
        u32,
        gen_u32,
        mod_u32,
        "Returns a random `u32` within a given range bound."
    );
    range_int!(
        u16,
        u16,
        gen_u16,
        mod_u16,
        "Returns a random `u16` within a given range bound."
    );
    range_int!(
        u8,
        u8,
        gen_u8,
        mod_u8,
        "Returns a random `u8` within a given range bound."
    );

    range_int!(
        i64,
        u64,
        gen_i64,
        mod_u64,
        "Returns a random `i64` within a given range bound."
    );
    range_int!(
        i32,
        u32,
        gen_i32,
        mod_u32,
        "Returns a random `i32` within a given range bound."
    );
    range_int!(
        i16,
        u16,
        gen_i16,
        mod_u16,
        "Returns a random `i16` within a given range bound."
    );
    range_int!(
        i8,
        u8,
        gen_i8,
        mod_u8,
        "Returns a random `i8` within a given range bound."
    );

    #[cfg(target_pointer_width = "16")]
    range_int!(
        usize,
        u16,
        gen_usize,
        mod_u16,
        "Returns a random `usize` within a given range bound."
    );
    #[cfg(target_pointer_width = "32")]
    range_int!(
        usize,
        u32,
        gen_usize,
        mod_u32,
        "Returns a random `usize` within a given range bound."
    );
    #[cfg(target_pointer_width = "64")]
    range_int!(
        usize,
        u64,
        gen_usize,
        mod_u64,
        "Returns a random `usize` within a given range bound."
    );

    #[cfg(target_pointer_width = "16")]
    range_int!(
        isize,
        u16,
        gen_isize,
        mod_u16,
        "Returns a random `isize` within a given range bound."
    );
    #[cfg(target_pointer_width = "32")]
    range_int!(
        isize,
        u32,
        gen_isize,
        mod_u32,
        "Returns a random `isize` within a given range bound."
    );
    #[cfg(target_pointer_width = "64")]
    range_int!(
        isize,
        u64,
        gen_isize,
        mod_u64,
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
    /// let rng = rng!(Default::default());
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
    /// let rng = rng!(Default::default());
    ///
    /// assert_eq!(rng.chance(1.0), true);
    /// ```
    #[inline]
    pub fn chance(&self, rate: f64) -> bool {
        const SCALE: f64 = 2.0 * (1u64 << 63) as f64;

        assert!(
            (0.0..=1.0).contains(&rate),
            "rate value is not between 0.0 and 1.0, received {}",
            rate
        );

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
    /// let rng = rng!(Default::default());
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

    /// Samples multiple unique items from a slice of values.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng = rng!(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5, 6];
    ///
    /// assert_eq!(rng.sample_multiple(&values, 2), vec![&6, &4]);
    /// ```
    #[inline]
    pub fn sample_multiple<'a, T>(&self, list: &'a [T], amount: usize) -> Vec<&'a T> {
        let draining = list.len().min(amount);

        let mut shuffled: Vec<&'a T> = list.iter().collect();

        self.shuffle(&mut shuffled);

        shuffled.drain(0..draining).collect()
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
    /// let rng = rng!(Default::default());
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

    /// Shuffles a slice randomly in O(n) time.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng = rng!(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5];
    /// let mut shuffled = values.clone();
    ///
    /// rng.shuffle(&mut shuffled);
    ///
    /// assert_ne!(&shuffled, &values);
    /// ```
    #[inline]
    pub fn shuffle<T>(&self, slice: &mut [T]) {
        (1..slice.len())
            .rev()
            .for_each(|index| slice.swap(index, self.usize(..=index)));
    }

    rand_characters!(
        alphabetic,
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
        "Generates a random `char` in ranges a-z and A-Z."
    );
    rand_characters!(
        alphanumeric,
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
        "Generates a random `char` in ranges a-z, A-Z and 0-9."
    );
    rand_characters!(
        lowercase,
        b"abcdefghijklmnopqrstuvwxyz",
        "Generates a random `char` in the range a-z."
    );
    rand_characters!(
        uppercase,
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "Generates a random `char` in the range A-Z."
    );

    /// Generate a random digit in the given `radix`.
    ///
    /// Digits are represented by `char`s in ranges 0-9 and a-z.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rand = rng!(Default::default());
    ///
    /// let digit = rand.digit(16);
    ///
    /// assert_eq!(&digit, &'2');
    /// ```
    /// # Panics
    ///
    /// Panics if the `radix` is zero or greater than 36.
    #[inline]
    pub fn digit(&self, radix: u8) -> char {
        match radix {
            0 => panic!("radix cannot be zero"),
            1..=36 => {
                let num = self.u8(..radix);

                if num < 10 {
                    (b'0' + num) as char
                } else {
                    (b'a' + num - 10) as char
                }
            }
            _ => panic!("radix cannot be greater than 36"),
        }
    }

    /// Generates a random `char` in the given range.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rand = rng!(Default::default());
    ///
    /// let character = rand.char('a'..'??');
    ///
    /// assert_eq!(character, '??');
    /// ```
    /// # Panics
    ///
    /// Panics if the range is empty.
    #[inline]
    pub fn char(&self, bounds: impl RangeBounds<char>) -> char {
        const SURROGATE_START: u32 = 0xd800u32;
        const SURROGATE_LENGTH: u32 = 0x800u32;

        let lower = match bounds.start_bound() {
            Bound::Unbounded => 0u8 as char,
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => {
                let scalar = if x as u32 == SURROGATE_START - 1 {
                    SURROGATE_START + SURROGATE_LENGTH
                } else {
                    x as u32 + 1
                };
                char::try_from(scalar)
                    .unwrap_or_else(|_| panic!("Invalid exclusive lower character bound"))
            }
        };

        let upper = match bounds.end_bound() {
            Bound::Unbounded => char::MAX,
            Bound::Included(&x) => x,
            Bound::Excluded(&x) => {
                let scalar = if x as u32 == SURROGATE_START + SURROGATE_LENGTH {
                    SURROGATE_START - 1
                } else {
                    (x as u32).wrapping_sub(1)
                };
                char::try_from(scalar)
                    .unwrap_or_else(|_| panic!("Invalid exclusive upper character bound"))
            }
        };

        assert!(upper >= lower, "Invalid character range");

        let lower_scalar = lower as u32;
        let upper_scalar = upper as u32;

        let gap = if lower_scalar < SURROGATE_START && upper_scalar >= SURROGATE_START {
            SURROGATE_LENGTH
        } else {
            0
        };

        let range = upper_scalar - lower_scalar - gap;
        let mut val = self.u32(0..=range) + lower_scalar;

        if val >= SURROGATE_START {
            val += gap;
        }

        val.try_into().unwrap()
    }

    modulus_int!(mod_u64, u64, u128, gen_u64);
    modulus_int!(mod_u32, u32, u64, gen_u32);
    modulus_int!(mod_u16, u16, u32, gen_u16);
    modulus_int!(mod_u8, u8, u16, gen_u8);
}

impl<S: State + Debug> Default for Rng<S> {
    /// Initialises a default instance of [`Rng`]. Warning, the default is
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
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<S: State + Debug> Clone for Rng<S> {
    /// Clones the [`Rng`] by deterministically deriving a new [`Rng`] based on the initial
    /// seed.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng1 = rng!(Default::default());
    /// let rng2 = rng!(Default::default());
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

/// A wrapper struct around [`Rng<CellState>`] to allow implementing
/// [`RngCore`] and [`SeedableRng`] traits in a compatible manner.
#[cfg(feature = "rand")]
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct RandCompat(Rng<CellState>);

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
impl From<Rng<CellState>> for RandCompat {
    #[inline]
    fn from(rng: Rng<CellState>) -> Self {
        Self(rng)
    }
}

#[cfg(feature = "rand")]
impl From<RandCompat> for Rng<CellState> {
    #[inline]
    fn from(rand: RandCompat) -> Self {
        rand.0
    }
}

thread_local! {
    static RNG: Rc<Rng<CellState>> = Rc::new(Rng(WyRand::<CellState>::with_seed(
        u64::from_ne_bytes(generate_entropy::<{ core::mem::size_of::<u64>() }>()),
    )));
}

/// Computes `(a * b) >> 128`. Adapted from: https://stackoverflow.com/a/28904636
#[inline]
fn multiply_high_u128(a: u128, b: u128) -> u128 {
    let a_low = a as u64 as u128;
    let a_high = (a >> 64) as u64 as u128;

    let b_low = b as u64 as u128;
    let b_high = (b >> 64) as u64 as u128;

    let carry = (a_low * b_low) >> 64;

    let a_high_x_b_low = a_high * b_low;
    let a_low_x_b_high = a_low * b_high;

    let carry = (a_high_x_b_low as u64 as u128 + a_low_x_b_high as u64 as u128 + carry) >> 64;

    a_high * b_high + (a_high_x_b_low >> 64) + (a_low_x_b_high >> 64) + carry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_leaking_debug() {
        let rng = rng!(Default::default());

        assert_eq!(format!("{:?}", rng), "Rng(WyRand(CellState))");
    }

    #[cfg(feature = "rand")]
    #[test]
    fn rand_compatibility() {
        fn get_rand_num<R: RngCore>(rng: &mut R) -> u64 {
            rng.next_u64()
        }

        let rng = rng!(Default::default());

        let mut rand: RandCompat = rng.into();

        let result = get_rand_num(&mut rand);

        assert_eq!(
            result, 14_839_104_130_206_199_084,
            "Should receive expect random u64 output, got {} instead",
            result
        );
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn serialize_rng() {
        let rng = rng!(12345);

        let json = serde_json::to_string(&rng).unwrap();

        assert_eq!(json, "{\"state\":24691}", "Serialized output not as expected");
    }
}
