use core::ops::{Bound, RangeBounds};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{boxed::Box, vec::Vec};

use crate::internal::uniform::IncreasingUniformIter;

/// Enum for determining the kind of PRNG, whether a fast one,
/// or a slow, possibly crypographically secure one.
pub enum TurboKind {
    /// Variant for fast PRNGs, like Wyrand.
    FAST,
    /// Variant for slower PRNGs, like ChaCha8.
    SLOW,
}

/// Base trait for implementing a PRNG. Only one method must be
/// implemented: [`TurboCore::fill_bytes`], which provides the basis
/// for any PRNG, to fill a buffer of bytes with random data.
///
/// This trait is object-safe.
///
/// # General Notes
///
/// When implementing on top of [`TurboCore`], the following considerations
/// should be made:
///
/// * [`Default`] - should be implemented for `std` platforms, but defaults should
///   be non-deterministic. It should initialise with a randomised seed as
///   a default, with the intent being quick and simple but random
///   number generation.
/// * [`core::fmt::Debug`] - should be implemented, but with care so to not leak
///   the internal state of the PRNG.
/// * [`PartialEq`] - should be implemented along with [`Eq`], so that
///   easy comparisons can be made with PRNGs to see if they are in the
///   same or different internal state.
/// * [`Clone`] - should be implemented, but with deterministically derived
///   new internal states for the cloned instances. The cloned instance
///   should not equal the original, but given a set seed on the original,
///   the cloned instance should derive a new state in a deterministic fashion.
/// * [`Copy`] - Do **not** implement [`Copy`], as it makes it too implicit
///   when handling references and passing around the instance. When a
///   copy is made, this modifies the state of the original in
///   producing the new state of the copied instance, which is not
///   something you want to happen implicitly.
pub trait TurboCore {
    /// Fills a mutable buffer with random bytes.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rand = Rng::with_seed(Default::default());
    ///
    /// let mut bytes = [0u8; 10];
    ///
    /// rand.fill_bytes(&mut bytes);
    ///
    /// assert_ne!(&bytes, &[0u8; 10], "output should not match a zeroed array");
    /// ```
    fn fill_bytes(&self, buffer: &mut [u8]);
}

/// This trait provides the means to easily generate all integer types, provided
/// the main method underpinning this is implemented: [`GenCore::gen`].
/// Once implemented, the rest of the trait provides default
/// implementations for generating all integer types, though it is not
/// recommended to override these.
///
/// The underlying implementation of [`GenCore::gen`] does not have to rely on
/// [`TurboCore::fill_bytes`] if the PRNG implementation provides a means to
/// output directly an array of const size.
pub trait GenCore: TurboCore {
    /// Determines the kind of PRNG. [`TurboKind::FAST`] RNGs are meant to be very
    /// quick, non-cryptographic PRNGs, while [`TurboKind::SLOW`] are slower,
    /// more expensive PRNGs, usually CSPRNGs but not always. Setting this constant
    /// allows for certain algorithms to be toggled for tuning performance of certain
    /// methods.
    const GEN_KIND: TurboKind;

    /// Returns an array of constant `SIZE` containing random `u8` values.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rand = Rng::with_seed(Default::default());
    ///
    /// let bytes = rand.gen::<10>();
    ///
    /// assert_ne!(&bytes, &[0u8; 10], "output should not match a zeroed array");
    /// ```
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE];

    gen_int_const!(gen_u128, u128, "Returns a random `u128` value.");
    gen_int_const!(gen_i128, i128, "Returns a random `i128` value.");
    gen_int_const!(gen_u64, u64, "Returns a random `u64` value.");
    gen_int_const!(gen_i64, i64, "Returns a random `i64` value.");
    gen_int_const!(gen_u32, u32, "Returns a random `u32` value.");
    gen_int_const!(gen_i32, i32, "Returns a random `i32` value.");
    gen_int_const!(gen_u16, u16, "Returns a random `u16` value.");
    gen_int_const!(gen_i16, i16, "Returns a random `i16` value.");
    gen_int_const!(gen_u8, u8, "Returns a random `u8` value.");
    gen_int_const!(gen_i8, i8, "Returns a random `i8` value.");
    gen_int_const!(gen_usize, usize, "Returns a random `usize` value.");
    gen_int_const!(gen_isize, isize, "Returns a random `isize` value.");
}

/// Trait for implementing Seedable PRNGs, requiring that the PRNG
/// implements [`TurboCore`] as a baseline. Seeds must be `Sized` in
/// order to be used as the internal state of a PRNG.
pub trait SeededCore: TurboCore {
    /// Associated type for accepting valid Seed values. Must be `Sized`.
    type Seed: Sized;

    /// Creates a new [`SeededCore`] with a specific seed value.
    fn with_seed(seed: Self::Seed) -> Self;

    /// Reseeds the [`SeededCore`] with a new seed/state.
    fn reseed(&self, seed: Self::Seed);
}

/// A marker trait to be applied to anything that implements [`TurboCore`]
/// in order to indicate that a PRNG source is cryptographically secure, so
/// being a CSPRNG.
///
/// This trait is provided as guidance only, and it is for the implementor to
/// ensure that their PRNG source qualifies as cryptographically secure. Must
/// be manually applied and is not an auto-trait.
pub trait SecureCore: TurboCore {}

/// Extension trait for automatically implementing all [`TurboRand`] methods,
/// as long as the struct implements [`TurboCore`] & [`GenCore`]. All methods
/// are provided as default implementations that build on top of [`TurboCore`]
/// and [`GenCore`], and thus are not recommended to be overridden, lest you
/// potentially change the expected outcome of the methods.
pub trait TurboRand: TurboCore + GenCore {
    /// Returns a random `u128` within a given range bound.
    ///
    /// # Panics
    ///
    /// Panics if the range is empty or invalid.
    #[inline]
    fn u128(&self, bounds: impl RangeBounds<u128>) -> u128 {
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
    fn i128(&self, bounds: impl RangeBounds<i128>) -> i128 {
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

    trait_range_int!(u64, u64, u128, gen_u64, "Returns a random `u64` value.");
    trait_range_int!(i64, u64, u128, gen_i64, "Returns a random `i64` value.");
    trait_range_int!(u32, u32, u64, gen_u32, "Returns a random `u32` value.");
    trait_range_int!(i32, u32, u64, gen_i32, "Returns a random `i32` value.");
    trait_range_int!(u16, u16, u32, gen_u16, "Returns a random `u16` value.");
    trait_range_int!(i16, u16, u32, gen_i16, "Returns a random `i16` value.");
    trait_range_int!(u8, u8, u16, gen_u8, "Returns a random `u8` value.");
    trait_range_int!(i8, u8, u16, gen_i8, "Returns a random `i8` value.");
    #[cfg(target_pointer_width = "16")]
    trait_range_int!(
        usize,
        u16,
        u32,
        gen_usize,
        "Returns a random `usize` within a given range bound."
    );
    #[cfg(target_pointer_width = "32")]
    trait_range_int!(
        usize,
        u32,
        u64,
        gen_usize,
        "Returns a random `usize` within a given range bound."
    );
    #[cfg(target_pointer_width = "64")]
    trait_range_int!(
        usize,
        u64,
        u128,
        gen_usize,
        "Returns a random `usize` within a given range bound."
    );
    #[cfg(target_pointer_width = "16")]
    trait_range_int!(
        isize,
        u16,
        u32,
        gen_isize,
        "Returns a random `isize` within a given range bound."
    );
    #[cfg(target_pointer_width = "32")]
    trait_range_int!(
        isize,
        u32,
        u64,
        gen_isize,
        "Returns a random `isize` within a given range bound."
    );
    #[cfg(target_pointer_width = "64")]
    trait_range_int!(
        isize,
        u64,
        u128,
        gen_isize,
        "Returns a random `isize` within a given range bound."
    );

    trait_float_gen!(
        f32,
        f32,
        u32,
        1.0,
        gen_u32,
        "Returns a random `f32` value between `0.0` and `1.0`."
    );
    trait_float_gen!(
        f32_normalized,
        f32,
        i32,
        2.0,
        gen_i32,
        "Returns a random `f32` value between `-1.0` and `1.0`."
    );
    trait_float_gen!(
        f64,
        f64,
        u64,
        1.0,
        gen_u64,
        "Returns a random `f32` value between `0.0` and `1.0`."
    );
    trait_float_gen!(
        f64_normalized,
        f64,
        i64,
        2.0,
        gen_i64,
        "Returns a random `f32` value between `-1.0` and `1.0`."
    );

    /// Returns a `usize` value for stable indexing across different
    /// word size platforms.
    #[inline]
    fn index(&self, bound: impl RangeBounds<usize>) -> usize {
        let lower = match bound.start_bound() {
            Bound::Included(&val) => val as u64,
            Bound::Excluded(&val) => val.saturating_add(1) as u64,
            Bound::Unbounded => 0,
        };
        let upper = match bound.end_bound() {
            Bound::Included(&val) => val as u64,
            Bound::Excluded(&val) => val.saturating_sub(1) as u64,
            Bound::Unbounded => usize::MAX as u64,
        };
        // `bound` will always either be u64 or smaller as `usize`, so it will
        // never overflow on a u64 bound.
        self.u64(lower..=upper) as usize
    }

    /// Returns a random boolean value.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// assert_eq!(rng.bool(), true);
    /// ```
    #[inline]
    fn bool(&self) -> bool {
        self.gen_u8() % 2 == 0
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
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// assert_eq!(rng.chance(1.0), true);
    /// ```
    #[inline]
    fn chance(&self, rate: f64) -> bool {
        const SCALE: f64 = 2.0 * (1u64 << 63) as f64;

        assert!(
            (0.0..=1.0).contains(&rate),
            "rate value is not between 0.0 and 1.0, received {rate}",
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
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5, 6];
    ///
    /// assert_eq!(rng.sample(&values), Some(&5));
    /// ```
    #[inline]
    fn sample<'a, T>(&self, list: &'a [T]) -> Option<&'a T> {
        self.sample_iter(list.iter())
    }

    /// Samples a random item from an iterator of values. `O(1)` if the
    /// iterator provides an accurate [`Iterator::size_hint`] to allow
    /// for optimisations to kick in, else `O(n)` where `n` is the size
    /// of the iterator.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let mut values = [1, 2, 3, 4, 5, 6];
    ///
    /// assert_eq!(rng.sample_iter(values.iter_mut()), Some(&mut 5));
    /// ```
    #[inline]
    fn sample_iter<T: Iterator>(&self, mut list: T) -> Option<T::Item> {
        // Adapted from: https://docs.rs/rand/latest/rand/seq/trait.IteratorRandom.html#method.choose
        let (mut lower, mut upper) = list.size_hint();

        if upper == Some(lower) {
            return match lower {
                0 => None,
                1 => list.next(),
                _ => list.nth(self.index(..lower)),
            };
        }

        let mut result = None;
        let mut consumed = 0;

        // Continue until the iterator is exhausted
        loop {
            if lower > 1 {
                let index = self.index(..(lower + consumed));
                let skip = if index < lower {
                    result = list.nth(index);
                    lower - (index + 1)
                } else {
                    lower
                };

                if upper == Some(lower) {
                    return result;
                }

                consumed += lower;

                if skip > 0 {
                    list.nth(skip - 1);
                }
            } else {
                let elem = list.next();

                if elem.is_none() {
                    return result;
                }

                consumed += 1;

                match consumed {
                    1 => {
                        result = elem;
                    }
                    _ => {
                        if self.index(..consumed) == 0 {
                            result = elem;
                        }
                    }
                };
            }

            let hint = list.size_hint();
            lower = hint.0;
            upper = hint.1;
        }
    }

    /// Samples a random `&mut` item from a slice of values.
    ///
    /// **NOTE**: Mutable references must be dropped before more can be
    /// sampled from the source slice. If a sampling tries to yield a mutable
    /// reference that already exists, the compiler will complain.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let mut values = [1, 2, 3, 4, 5, 6];
    ///
    /// let result1 = rng.sample_mut(&mut values);
    ///
    /// assert_eq!(result1, Some(&mut 5));
    ///
    /// let result2 = rng.sample_mut(&mut values);
    ///
    /// assert_eq!(result2, Some(&mut 3));
    /// ```
    #[inline]
    fn sample_mut<'a, T>(&self, list: &'a mut [T]) -> Option<&'a mut T> {
        self.sample_iter(list.iter_mut())
    }

    /// Samples multiple unique items from a slice of values.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5, 6];
    ///
    /// assert_eq!(rng.sample_multiple(&values, 2), vec![&6, &4]);
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    fn sample_multiple<'a, T>(&self, list: &'a [T], amount: usize) -> Vec<&'a T> {
        self.sample_multiple_iter(list.iter(), amount)
    }

    /// Samples multiple unique items from a mutable slice of values.
    ///
    /// **NOTE**: Mutable references must be dropped before more can be
    /// sampled from the source slice. If a sampling tries to yield a mutable
    /// reference that already exists, the compiler will complain.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let mut values = [1, 2, 3, 4, 5, 6];
    ///
    /// assert_eq!(rng.sample_multiple_mut(&mut values, 2), vec![&mut 6, &mut 4]);
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    fn sample_multiple_mut<'a, T>(&self, list: &'a mut [T], amount: usize) -> Vec<&'a mut T> {
        self.sample_multiple_iter(list.iter_mut(), amount)
    }

    /// Samples multiple unique items from an iterator of values.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5, 6];
    ///
    /// assert_eq!(rng.sample_multiple_iter(values.iter(), 2), vec![&6, &4]);
    /// ```
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[inline]
    fn sample_multiple_iter<T: Iterator>(&self, mut list: T, amount: usize) -> Vec<T::Item> {
        // Adapted from: https://docs.rs/rand/latest/rand/seq/trait.IteratorRandom.html#method.choose_multiple
        let mut sampled = Vec::with_capacity(amount);

        sampled.extend(list.by_ref().take(amount));

        // Continue unless the iterator was exhausted.
        // Note: this prevents iterators that "restart" from causing problems.
        // If the iterator stops once, then so do we.
        if sampled.len() == amount {
            list.enumerate()
                .map(|(index, elem)| {
                    let len = index + amount;

                    (self.index(..=len), elem)
                })
                .for_each(|(slot_index, elem)| {
                    if let Some(slot) = sampled.get_mut(slot_index) {
                        *slot = elem;
                    }
                });
        } else {
            // Shrink sampled vector if the available amount from the iterator
            // is less than the requested amount.
            sampled.shrink_to_fit();
        }

        sampled
    }

    /// [Stochastic Acceptance](https://arxiv.org/abs/1109.3627) implementation of Roulette Wheel
    /// weighted selection. Uses a closure to return a `rate` value for each randomly sampled item
    /// to decide whether to return it or not. The returned `f64` value must be between `0.0` and `1.0`.
    ///
    /// Returns `None` if given an empty list to sample from. For a list containing 1 item, it'll always
    /// return that item regardless. Otherwise, operations are O(1) in complexity, and require no allocations.
    ///
    /// # Panics
    ///
    /// If the returned value of the `weight_sampler` closure is not between `0.0` and `1.0`.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5, 6];
    ///
    /// let total = f64::from(values.iter().sum::<i32>());
    ///
    /// assert_eq!(rng.weighted_sample(&values, |(&item, _)| item as f64 / total), Some(&4));
    /// ```
    #[inline]
    fn weighted_sample<'a, T, F>(&self, list: &'a [T], weight_sampler: F) -> Option<&'a T>
    where
        F: Fn((&T, usize)) -> f64,
    {
        // Check how many items are in the list
        match list.len() {
            // No values in list, therefore return None.
            0 => None,
            // Only a single value in list, therefore sampling will always yield that value.
            1 => list.first(),
            // Sample the list, and then check if it passes the weighted chance.
            // Keep repeating until a value succeds and return that.
            len => loop {
                let index = self.index(..len);

                if let Some(item) = list
                    .get(index)
                    .filter(|&item| self.chance(weight_sampler((item, index))))
                {
                    return Some(item);
                }
            },
        }
    }

    /// [Stochastic Acceptance](https://arxiv.org/abs/1109.3627) implementation of Roulette Wheel
    /// weighted selection. Uses a closure to return a `rate` value for each randomly sampled item
    /// to decide whether to return it or not. The returned `f64` value must be between `0.0` and `1.0`.
    ///
    /// Returns `None` if given an empty iterator to sample from. For an iterator containing 1 item, it'll always
    /// return that item regardless. Otherwise, operations are O(1) in complexity, and require no allocations.
    ///
    /// Note, this method requires an `Iterator` that implements `Clone`, as it needs to reset and re-run the
    /// `Iterator` every time a sample is unsuccessful in picking an item.
    ///
    /// # Panics
    ///
    /// If the returned value of the `weight_sampler` closure is not between `0.0` and `1.0`.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5, 6];
    ///
    /// let total = f64::from(values.iter().sum::<i32>());
    ///
    /// assert_eq!(rng.weighted_sample_iter(values.into_iter(), |(&item, _)| item as f64 / total), Some(4));
    /// ```
    #[inline]
    fn weighted_sample_iter<T, F>(&self, mut list: T, weight_sampler: F) -> Option<T::Item>
    where
        T: Iterator + Clone,
        F: Fn((&T::Item, usize)) -> f64,
    {
        let (lower, _) = list.size_hint();

        match lower {
            0 => None,
            1 => list.next(),
            _ => {
                let mut sampler = list.clone();

                loop {
                    let selected = self.index(..lower);

                    match sampler
                        .by_ref()
                        .nth(selected)
                        .filter(|item| self.chance(weight_sampler((item, selected))))
                    {
                        None => {
                            sampler = list.clone();
                        }
                        Some(item) => return Some(item),
                    };
                }
            }
        }
    }

    /// [Stochastic Acceptance](https://arxiv.org/abs/1109.3627) implementation of Roulette Wheel
    /// weighted selection. Uses a closure to return a `rate` value for each randomly sampled item
    /// to decide whether to return it or not. The returned `f64` value must be between `0.0` and `1.0`.
    ///
    /// Returns `None` if given an empty list to sample from. For a list containing 1 item, it'll always
    /// return that item regardless. Otherwise, operations are O(1) in complexity, and require no allocations.
    ///
    /// **NOTE**: Mutable references must be dropped before more can be
    /// sampled from the source slice. If a sampling tries to yield a mutable
    /// reference that already exists, the compiler will complain.
    ///
    /// # Panics
    ///
    /// If the returned value of the `weight_sampler` closure is not between `0.0` and `1.0`.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let mut values = [1, 2, 3, 4, 5, 6];
    ///
    /// let total = f64::from(values.iter().sum::<i32>());
    ///
    /// let result1 = rng.weighted_sample_mut(&mut values, |(&item, _)| item as f64 / total);
    ///
    /// assert_eq!(result1, Some(&mut 4));
    ///
    /// let result2 = rng.weighted_sample_mut(&mut values, |(&item, _)| item as f64 / total);
    ///
    /// assert_eq!(result2, Some(&mut 5));
    ///
    /// // We have to drop `result2` because the next sample will try to return the same
    /// // value again. And rust won't allow two mutable references to exist at the same
    /// // time
    /// drop(result2);
    ///
    /// let result3 = rng.weighted_sample_mut(&mut values, |(&item, _)| item as f64 / total);
    ///
    /// // Weighted sample in this example will favour larger numbers, so 5 gets picked
    /// // again.
    /// assert_eq!(result3, Some(&mut 5));
    /// ```
    #[inline]
    fn weighted_sample_mut<'a, T, F>(
        &self,
        list: &'a mut [T],
        weight_sampler: F,
    ) -> Option<&'a mut T>
    where
        F: Fn((&T, usize)) -> f64,
    {
        // Check how many items are in the list
        match list.len() {
            // No values in list, therefore return None.
            0 => None,
            // Only a single value in list, therefore sampling will always yield that value.
            1 => list.first_mut(),
            // Sample the list, and then check if it passes the weighted chance.
            // Keep repeating until a value succeds and return that.
            len => loop {
                let index = self.index(..len);

                if list
                    .get(index)
                    .filter(|&item| self.chance(weight_sampler((item, index))))
                    .is_some()
                {
                    return list.get_mut(index);
                }
            },
        }
    }

    /// Shuffles a slice randomly in O(n) time.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let values = [1, 2, 3, 4, 5];
    /// let mut shuffled = values.clone();
    ///
    /// rng.shuffle(&mut shuffled);
    ///
    /// assert_ne!(&shuffled, &values);
    /// ```
    #[inline]
    fn shuffle<T>(&self, slice: &mut [T]) {
        match slice.len() {
            // Don't bother trying to shuffle an empty list or a single item list.
            len if len < 2 => (),
            len => {
                self.partial_shuffle(slice, len);
            }
        }
    }

    /// Partially shuffles a slice by a given amount and returns the shuffled part
    /// and non-shuffled part.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng = Rng::with_seed(Default::default());
    ///
    /// let mut values = [1, 2, 3, 4, 5];
    ///
    /// let (shuffled, rest) = rng.partial_shuffle(&mut values, 2);
    ///
    /// assert_eq!(shuffled, &mut [2, 5]);
    /// assert_eq!(rest, &mut [1, 4, 3]);
    /// ```
    #[inline]
    fn partial_shuffle<'a, T>(
        &self,
        slice: &'a mut [T],
        amount: usize,
    ) -> (&'a mut [T], &'a mut [T]) {
        let len = slice.len();

        assert!(len > 1);

        let n = len.saturating_sub(amount);

        match Self::GEN_KIND {
            // Some algorithms are just much faster with the naive approach than with the
            // increasing uniform approach. Wyrand's algorithm is consistently faster than
            // the increasing uniform algorithm, so don't bother trying to optimise. This does
            // make the shuffling output different for different algorithms, but that
            // is expected anyway.
            TurboKind::FAST => {
                ((n.max(1))..len)
                    .rev()
                    .for_each(|index| slice.swap(index, self.index(..=index)));
            }
            // The Increasing Uniform approach differ's from `rand`'s in that here we are
            // optimising for the 64-bit platforms, not 32-bit. Nowadays, 64-bit platforms
            // are more common, and 32-bit is more on the embedded side. Plus, with Wyrand
            // using the fast, naive approach, this approach below would be used for ChaCha8
            // which is slower, but more secure.
            TurboKind::SLOW => {
                IncreasingUniformIter::new(self, n as u64, len)
                    .for_each(|(current_index, swap_index)| slice.swap(current_index, swap_index));
            }
        };

        let res = slice.split_at_mut(n);
        (res.1, res.0)
    }

    trait_rand_chars!(
        alphabetic,
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
        "Generates a random `char` in ranges a-z and A-Z."
    );
    trait_rand_chars!(
        alphanumeric,
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
        "Generates a random `char` in ranges a-z, A-Z and 0-9."
    );
    trait_rand_chars!(
        lowercase,
        b"abcdefghijklmnopqrstuvwxyz",
        "Generates a random `char` in the range a-z."
    );
    trait_rand_chars!(
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
    /// use turborand::prelude::*;
    ///
    /// let rand = Rng::with_seed(Default::default());
    ///
    /// let digit = rand.digit(16);
    ///
    /// assert_eq!(&digit, &'2');
    /// ```
    /// # Panics
    ///
    /// Panics if the `radix` is zero or greater than 36.
    #[inline]
    fn digit(&self, radix: u8) -> char {
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
    /// use turborand::prelude::*;
    ///
    /// let rand = Rng::with_seed(Default::default());
    ///
    /// let character = rand.char('a'..'Ç');
    ///
    /// assert_eq!(character, '»');
    /// ```
    /// # Panics
    ///
    /// Panics if the range is empty.
    #[inline]
    fn char(&self, bounds: impl RangeBounds<char>) -> char {
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
                char::from_u32(scalar).expect("Invalid exclusive lower character bound")
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
                char::from_u32(scalar).expect("Invalid exclusive upper character bound")
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

        let range = upper_scalar - gap;
        let mut val = self.u32(lower_scalar..=range);

        if val >= SURROGATE_START {
            val += gap;
        }

        char::from_u32(val).unwrap()
    }
}

/// Trait for enabling creating new [`TurboCore`] instances from an original instance.
/// Similar to cloning, except forking modifies the state of the original instance in order
/// to provide a new, random state for the forked instance. This allows for creating many randomised
/// instances from a single seed in a deterministic manner.
pub trait ForkableCore: TurboCore {
    /// Forks a [`TurboCore`] instance by deterministically deriving a new instance based on the initial
    /// seed.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng1 = Rng::with_seed(Default::default());
    /// let rng2 = Rng::with_seed(Default::default());
    ///
    /// // Use the RNGs once each.
    /// rng1.bool();
    /// rng2.bool();
    ///
    /// let forked1 = rng1.fork();
    /// let forked2 = rng2.fork();
    ///
    /// // Forked instances should not be equal to the originals
    /// assert_ne!(forked1, rng1);
    /// assert_ne!(forked2, rng2);
    /// // If they derived from the same initial seed, forked instances
    /// // should be equal to each other...
    /// assert_eq!(forked1, forked2);
    /// // ...and thus yield the same outputs.
    /// assert_eq!(forked1.u64(..), forked2.u64(..));
    /// ```
    fn fork(&self) -> Self;
}

impl<T: TurboCore + GenCore + ?Sized> TurboRand for T {}

#[cfg(feature = "alloc")]
impl<T: TurboCore + ?Sized> TurboCore for Box<T> {
    #[inline(always)]
    fn fill_bytes(&self, buffer: &mut [u8]) {
        (**self).fill_bytes(buffer);
    }
}

#[cfg(feature = "alloc")]
impl<T: GenCore + ?Sized> GenCore for Box<T> {
    const GEN_KIND: TurboKind = T::GEN_KIND;

    #[inline(always)]
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
        (**self).gen()
    }
}

impl<'a, T: TurboCore + ?Sized> TurboCore for &'a T {
    #[inline(always)]
    fn fill_bytes(&self, buffer: &mut [u8]) {
        (**self).fill_bytes(buffer);
    }
}

impl<'a, T: GenCore + ?Sized> GenCore for &'a T {
    const GEN_KIND: TurboKind = T::GEN_KIND;

    #[inline(always)]
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
        (**self).gen()
    }
}

impl<'a, T: TurboCore + ?Sized> TurboCore for &'a mut T {
    #[inline(always)]
    fn fill_bytes(&self, buffer: &mut [u8]) {
        (**self).fill_bytes(buffer);
    }
}

impl<'a, T: GenCore + ?Sized> GenCore for &'a mut T {
    const GEN_KIND: TurboKind = T::GEN_KIND;

    #[inline(always)]
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
        (**self).gen()
    }
}

#[cfg(feature = "alloc")]
impl<T: TurboCore + SecureCore + ?Sized> SecureCore for Box<T> {}

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
    use core::cell::Cell;

    use super::*;

    struct TestRng(Cell<u8>);

    impl TestRng {
        fn new() -> Self {
            Self(Cell::new(0))
        }

        fn next(&self) -> u8 {
            let value = self.0.get();

            self.0.set(value.wrapping_add(1));

            value
        }
    }

    impl TurboCore for TestRng {
        fn fill_bytes(&self, buffer: &mut [u8]) {
            buffer.iter_mut().for_each(|slot| *slot = self.next());
        }
    }

    impl GenCore for TestRng {
        const GEN_KIND: TurboKind = TurboKind::FAST;

        fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
            core::array::from_fn(|_| self.next())
        }
    }

    impl SeededCore for TestRng {
        type Seed = u8;

        fn with_seed(seed: Self::Seed) -> Self {
            Self(Cell::new(seed))
        }

        fn reseed(&self, seed: Self::Seed) {
            self.0.set(seed);
        }
    }

    #[test]
    fn auto_trait_application() {
        let rng = TestRng::new();

        fn use_rng<T: TurboRand>(source: &T) -> u8 {
            source.u8(..)
        }

        let value = use_rng(&rng);

        assert_eq!(value, 0);
    }

    #[test]
    fn seeded_methods() {
        let rng = TestRng::with_seed(5);

        fn test_seeded_methods<T: GenCore + SeededCore>(source: &T)
        where
            T: SeededCore<Seed = u8>,
        {
            let values = source.gen();

            assert_eq!(&values, &[5, 6, 7]);

            source.reseed(3);

            let values = source.gen();

            assert_eq!(&values, &[3, 4, 5]);
        }

        test_seeded_methods(&rng);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn object_safe_core() {
        let rng = Box::new(TestRng::with_seed(1));

        fn test_dyn_rng(rng: Box<dyn TurboCore>) {
            let mut buffer = [0u8; 3];

            rng.fill_bytes(&mut buffer);

            assert_eq!(&buffer, &[1, 2, 3]);
        }

        test_dyn_rng(rng);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn boxed_methods() {
        let rng = Box::new(TestRng::with_seed(1));

        assert_eq!(&rng.gen(), &[1, 2]);

        fn test_boxed_turborand<T: TurboRand>(boxed: T) {
            assert_eq!(boxed.u8(..), 3);
        }

        test_boxed_turborand(rng);
    }

    #[test]
    fn ref_methods() {
        let mut rng = TestRng::with_seed(1);

        fn test_ref_methods<T: GenCore>(reffed: T, expected: [u8; 3]) {
            assert_eq!(reffed.gen(), expected);
        }

        test_ref_methods(&rng, [1, 2, 3]);
        test_ref_methods(&mut rng, [4, 5, 6]);
    }
}
