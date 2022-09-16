macro_rules! gen_int_const {
    ($func:ident, $int:ty, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        fn $func(&self) -> $int {
            <$int>::from_le_bytes(self.gen())
        }
    };
}

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

macro_rules! trait_float_gen {
    ($name:ident, $value:tt, $int:ty, $source:ident, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        fn $name(&self) -> $value {
            (self.$source() as $value) / (<$int>::MAX as $value)
        }
    };
}

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
