macro_rules! gen_int_const {
    ($func:ident, $int:ty, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        fn $func(&self) -> $int {
            <$int>::from_le_bytes(self.gen())
        }
    };
}

pub(crate) use gen_int_const;

macro_rules! trait_range_int {
    ($value:tt, $unsigned:tt, $bigger:ty, $source:ident, $doc:tt) => {
        #[doc = $doc]
        ///
        /// # Panics
        ///
        /// Panics if the range is empty or invalid.
        #[inline]
        fn $value(&self, bounds: impl RangeBounds<$value>) -> $value {
            const BITS: $bigger = $value::BITS as $bigger;

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

            assert!(lower <= upper, "Range should not be zero sized or invalid");

            match (lower, upper) {
                ($value::MIN, $value::MAX) => self.$source(),
                (_, _) => {
                    let range = upper.wrapping_sub(lower).wrapping_add(1) as $unsigned;
                    let mut generated = self.$source() as $unsigned;
                    let mut high = (generated as $bigger).wrapping_mul(range as $bigger);
                    let mut low = high as $unsigned;
                    if low < range {
                        let threshold = range.wrapping_neg() % range;
                        while low < threshold {
                            generated = self.$source() as $unsigned;
                            high = (generated as $bigger).wrapping_mul(range as $bigger);
                            low = high as $unsigned;
                        }
                    }
                    let value = (high >> BITS) as $value;
                    lower.wrapping_add(value)
                }
            }
        }
    };
}

pub(crate) use trait_range_int;

macro_rules! trait_float_gen {
    ($name:ident, $value:tt, $int:ty, $scale:expr, $source:ident, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        fn $name(&self) -> $value {
            const FLOAT_SIZE: u32 = (core::mem::size_of::<$value>() as u32) * 8;
            const SCALE: $value = $scale / ((1 as $int << <$value>::MANTISSA_DIGITS) as $value);

            let value = self.$source() >> (FLOAT_SIZE - <$value>::MANTISSA_DIGITS);

            SCALE * (value as $value)
        }
    };
}

pub(crate) use trait_float_gen;

macro_rules! trait_rand_chars {
    ($func:ident, $chars:expr, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        fn $func(&self) -> char {
            const CHARS: &[u8] = $chars;

            self.sample(CHARS).map(|&value| value as char).unwrap()
        }
    };
}

pub(crate) use trait_rand_chars;

macro_rules! trait_fillable_gen {
    () => {};
    ($t:ty) => {
        impl Fillable for [$t] {
            #[inline]
            fn fill_random<R: GenCore + ?Sized>(&mut self, rng: &R) {
                if !self.is_empty() {
                    // SAFETY: The slice is not empty, therefore it is properly
                    // initialised and aligned, so constructing a [u8] slice from
                    // it is safe.
                    rng.fill_bytes(unsafe {
                        core::slice::from_raw_parts_mut(
                            self.as_mut_ptr() as *mut u8,
                            core::mem::size_of_val(self),
                        )
                    });
                }
            }
        }

        impl Fillable for [core::num::Wrapping<$t>] {
            #[inline]
            fn fill_random<R: GenCore + ?Sized>(&mut self, rng: &R) {
                if !self.is_empty() {
                    // SAFETY: The slice is not empty, therefore it is properly
                    // initialised and aligned, so constructing a [u8] slice from
                    // it is safe.
                    rng.fill_bytes(unsafe {
                        core::slice::from_raw_parts_mut(
                            self.as_mut_ptr() as *mut u8,
                            core::mem::size_of_val(self),
                        )
                    });
                }
            }
        }
    };
}

pub(crate) use trait_fillable_gen;
