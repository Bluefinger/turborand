use std::{
    iter::repeat_with,
    ops::{Bound, RangeBounds},
};

/// Base trait for implementing a PRNG. Two methods must be
/// implemented: [`TurboCore::gen`] and [`TurboCore::fill_bytes`].
/// Once implemented, the rest of the trait provides default
/// implementations for generating all integer type, though it is not
/// recommended to override these.
pub trait TurboCore: Sized {
    /// Returns an array of constant `SIZE` containing random `u8` values.
    /// 
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rand = rng!(Default::default());
    ///
    /// let bytes = rand.gen::<10>();
    ///
    /// assert_ne!(&bytes, &[0u8; 10], "output should not match a zeroed array");
    /// ```
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE];

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
    fn fill_bytes<B: AsMut<[u8]>>(&self, buffer: B);

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
/// ensure that their PRNG source qualifies as cryptographically secure.
pub trait SecureCore: TurboCore {}

/// Extension trait for automatically implementing all [`TurboRand`] methods,
/// as long as the struct implements [`TurboCore`]. All methods are provided
/// as default implementations that build on top of [`TurboCore`] as well as
/// some methods defined within [`TurboRand`] and thus are not recommended to
/// be overridden, lest you potentially change the expected outcome of the
/// methods.
pub trait TurboRand: TurboCore {
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
        gen_u32,
        "Returns a random `f32` value between `0.0` and `1.0`."
    );
    trait_float_gen!(
        f32_normalized,
        f32,
        i32,
        gen_i32,
        "Returns a random `f32` value between `-1.0` and `1.0`."
    );
    trait_float_gen!(
        f64,
        f64,
        u64,
        gen_u64,
        "Returns a random `f32` value between `0.0` and `1.0`."
    );
    trait_float_gen!(
        f64_normalized,
        f64,
        i64,
        gen_i64,
        "Returns a random `f32` value between `-1.0` and `1.0`."
    );

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
    /// use turborand::*;
    ///
    /// let rng = rng!(Default::default());
    ///
    /// assert_eq!(rng.chance(1.0), true);
    /// ```
    #[inline]
    fn chance(&self, rate: f64) -> bool {
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
    fn sample<'a, T>(&self, list: &'a [T]) -> Option<&'a T> {
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
    fn sample_multiple<'a, T>(&self, list: &'a [T], amount: usize) -> Vec<&'a T> {
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
    fn weighted_sample<'a, T, F>(&self, list: &'a [T], weight_sampler: F) -> Option<&'a T>
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
    fn shuffle<T>(&self, slice: &mut [T]) {
        (1..slice.len())
            .rev()
            .for_each(|index| slice.swap(index, self.usize(..=index)));
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
    /// use turborand::*;
    ///
    /// let rand = rng!(Default::default());
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
