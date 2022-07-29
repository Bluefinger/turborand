macro_rules! range_int {
    ($value:tt, $unsigned:tt, $source:ident, $modulus:ident, $doc:tt) => {
        #[doc = $doc]
        ///
        /// # Panics
        ///
        /// Panics if the range is empty or invalid.
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

            assert!(lower <= upper, "Range should not be zero sized or invalid");

            match (lower, upper) {
                ($value::MIN, $value::MAX) => self.$source(),
                (_, _) => {
                    let range = upper.wrapping_sub(lower).wrapping_add(1);
                    lower.wrapping_add(self.$modulus(range as $unsigned) as $value)
                }
            }
        }
    };
}

macro_rules! modulus_int {
    ($name:ident, $value:tt, $bigger:tt, $source:ident) => {
        #[inline]
        fn $name(&self, range: $value) -> $value {
            const BITS: $bigger = $value::BITS as $bigger;

            let mut generated = self.$source();
            let mut high = (generated as $bigger).wrapping_mul(range as $bigger);
            let mut low = high as $value;
            if low < range {
                let threshold = range.wrapping_neg() % range;
                while low < threshold {
                    generated = self.$source();
                    high = (generated as $bigger).wrapping_mul(range as $bigger);
                    low = high as $value;
                }
            }
            (high >> BITS) as $value
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
            self.fill_bytes(&mut bytes);
            <$int>::from_le_bytes(bytes)
        }
    };
}

macro_rules! rand_int_const {
    ($func:ident, $int:ty, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        pub fn $func(&self) -> $int {
            <$int>::from_le_bytes(self.0.rand::<{ core::mem::size_of::<$int>() }>())
        }
    };
}

macro_rules! rand_int_generate {
    ($func:ident, $int:ty, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        pub fn $func(&self) -> $int {
            const SIZE: usize = core::mem::size_of::<$int>();
            let mut bytes = [0u8; SIZE];
            let mut buffer = bytes.as_mut_slice();
            let mut length = SIZE;
            while length > 0 {
                let output = self.0.generate();
                let fill = output.len().min(length);
                buffer[..fill].copy_from_slice(&output[..fill]);
                buffer = &mut buffer[fill..];
                length -= fill;
            }
            <$int>::from_le_bytes(bytes)
        }
    };
}

macro_rules! rand_characters {
    ($func:ident, $chars:expr, $doc:tt) => {
        #[doc = $doc]
        #[inline]
        pub fn $func(&self) -> char {
            const CHARS: &[u8] = $chars;

            self.sample(CHARS).map(|&value| value as char).unwrap()
        }
    };
}
