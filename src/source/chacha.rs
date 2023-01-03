use core::cell::UnsafeCell;

use self::utils::{calculate_block, increment_counter, init_state, AlignedSeed};
use crate::internal::buffer::EntropyBuffer;

#[cfg(feature = "fmt")]
use crate::Debug;

#[cfg(feature = "serialize")]
use crate::{Deserialize, Serialize, SerializeStruct, Visitor};

mod constants;
pub(crate) mod utils;

/// A ChaCha8 based Random Number Generator
pub(crate) struct ChaCha8 {
    state: UnsafeCell<[u32; 16]>,
    cache: EntropyBuffer<8>,
}

impl ChaCha8 {
    #[cfg(feature = "serialize")]
    #[inline]
    #[must_use]
    fn from_serde(state: [u32; 16], cache: EntropyBuffer<8>) -> Self {
        Self {
            state: UnsafeCell::new(state),
            cache,
        }
    }

    #[inline]
    #[must_use]
    fn get_state(&self) -> &[u32; 16] {
        // SAFETY: The memory being read will always be initialised,
        // therefore this is safe. This reference is used in only three cases,
        // in which all will never exist for long enough to overlap with a write.
        // This can also cause data races if called from different threads,
        // but ChaCha8 is not Sync, so this won't happen.
        unsafe { &*self.state.get() }
    }

    #[inline]
    fn update_state(&self, state: [u32; 16]) {
        // SAFETY: Pointer is kept here only for as long as the write happens,
        // with the array of data not needing to be dropped and instead it being
        // fine for being overwritten. This can also cause data races if called
        // from different threads, but ChaCha8 is not Sync, so this won't happen.
        unsafe {
            self.state.get().write(state);
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn with_seed(seed: AlignedSeed) -> Self {
        Self {
            state: UnsafeCell::new(init_state(seed)),
            cache: EntropyBuffer::new(),
        }
    }

    #[inline]
    pub(crate) fn reseed(&self, seed: AlignedSeed) {
        let state = init_state(seed);

        self.update_state(state);
        self.cache.empty_buffer();
    }

    fn generate(&self) -> [u32; 16] {
        let new_state = calculate_block::<4>(self.get_state());

        self.update_state(increment_counter(new_state));

        new_state
    }

    #[inline]
    pub(crate) fn rand<const OUTPUT: usize>(&self) -> [u8; OUTPUT] {
        let mut value = [0u8; OUTPUT];

        self.fill(&mut value);

        value
    }

    #[inline]
    pub(crate) fn fill<B: AsMut<[u8]>>(&self, buffer: B) {
        self.cache
            .fill_bytes_with_source(buffer, || bytemuck::cast(self.generate()))
    }
}

impl Clone for ChaCha8 {
    fn clone(&self) -> Self {
        Self {
            state: UnsafeCell::new(*self.get_state()),
            cache: self.cache.clone(),
        }
    }
}

#[cfg(feature = "fmt")]
impl Debug for ChaCha8 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ChaCha8").finish()
    }
}

impl PartialEq for ChaCha8 {
    fn eq(&self, other: &Self) -> bool {
        self.get_state() == other.get_state() && self.cache == other.cache
    }
}

impl Eq for ChaCha8 {}

#[cfg(feature = "serialize")]
impl Serialize for ChaCha8 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ChaCha8", 2)?;
        s.serialize_field("state", self.get_state())?;
        s.serialize_field("cache", &self.cache)?;

        s.end()
    }
}

#[cfg(feature = "serialize")]
impl<'de> Deserialize<'de> for ChaCha8 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["state", "cache"];

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            State,
            Cache,
        }

        struct ChaChaVisitor;

        impl<'de> Visitor<'de> for ChaChaVisitor {
            type Value = ChaCha8;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ChaCha8")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let state = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let cache = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                Ok(ChaCha8::from_serde(state, cache))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut state = None;
                let mut cache = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::State => {
                            if state.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state = Some(map.next_value()?);
                        }
                        Field::Cache => {
                            if cache.is_some() {
                                return Err(serde::de::Error::duplicate_field("cache"));
                            }
                            cache = Some(map.next_value()?);
                        }
                    }
                }

                let state = state.ok_or_else(|| serde::de::Error::missing_field("state"))?;
                let cache = cache.ok_or_else(|| serde::de::Error::missing_field("cache"))?;

                Ok(ChaCha8::from_serde(state, cache))
            }
        }

        deserializer.deserialize_struct("ChaCha8", FIELDS, ChaChaVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_vector {
        ($test:ident, $seed:tt, $output1:tt) => {
            #[test]
            fn $test() {
                let source = ChaCha8::with_seed($seed.into());

                let expected_output: [u8; 64] = $output1;
                let output = source.rand::<64>();

                assert_eq!(&output, &expected_output);
            }
        };
    }

    #[cfg(feature = "fmt")]
    #[test]
    fn no_leaking_debug() {
        let source = ChaCha8::with_seed([0u8; 40].into());

        assert_eq!(format!("{:?}", source), "ChaCha8");
    }

    #[test]
    fn clone_chacha_source() {
        let source = ChaCha8::with_seed([0u8; 40].into());

        let cloned = source.clone();

        assert_eq!(&source, &cloned);
    }

    #[test]
    fn equality_check() {
        let source = ChaCha8::with_seed([0u8; 40].into());
        let source2 = ChaCha8::with_seed([0u8; 40].into());

        assert_eq!(
            source, source2,
            "Sources should match with same seed & buffer states"
        );

        source.rand::<10>();

        assert_ne!(
            source, source2,
            "Sources should not match when buffer & state are different"
        );

        source2.rand::<10>();

        assert_eq!(
            source, source2,
            "Sources should match again when buffer & states are the same again"
        );
    }

    #[test]
    fn reseed() {
        let source = ChaCha8::with_seed([0u8; 40].into());

        let value1 = source.rand::<4>();

        source.reseed([0u8; 40].into());

        let value2 = source.rand::<4>();

        assert_eq!(
            value1, value2,
            "Output values should match after source is reseeded with the same state"
        );
    }

    #[test]
    fn buffered_rand() {
        let source = ChaCha8::with_seed([0u8; 40].into());

        let output = source.rand::<40>();

        assert_eq!(
            &output,
            &[
                62, 0, 239, 47, 137, 95, 64, 214, 127, 91, 184, 232, 31, 9, 165, 161, 44, 132, 14,
                195, 206, 154, 127, 59, 24, 27, 225, 136, 239, 113, 26, 30, 152, 76, 225, 114, 185,
                33, 111, 65
            ]
        );

        let output = source.rand::<40>();

        assert_eq!(
            &output,
            &[
                159, 68, 83, 103, 69, 109, 86, 25, 49, 74, 66, 163, 218, 134, 176, 1, 56, 123, 253,
                184, 14, 12, 254, 66, 73, 85, 195, 125, 191, 44, 172, 1, 16, 111, 141, 49, 230,
                177, 3, 19
            ]
        );
    }

    #[test]
    fn stress_rand_buffer() {
        let source = ChaCha8::with_seed([0u8; 40].into());

        let mut output = [0u8; 40];

        for _ in 0..32 {
            let generated = source.rand::<40>();

            assert_ne!(&output, &generated);

            output = generated;
        }
    }

    test_vector!(
        zeroed_vector,
        [0u8; 40],
        [
            0x3e, 0x00, 0xef, 0x2f, 0x89, 0x5f, 0x40, 0xd6, 0x7f, 0x5b, 0xb8, 0xe8, 0x1f, 0x09,
            0xa5, 0xa1, 0x2c, 0x84, 0x0e, 0xc3, 0xce, 0x9a, 0x7f, 0x3b, 0x18, 0x1b, 0xe1, 0x88,
            0xef, 0x71, 0x1a, 0x1e, 0x98, 0x4c, 0xe1, 0x72, 0xb9, 0x21, 0x6f, 0x41, 0x9f, 0x44,
            0x53, 0x67, 0x45, 0x6d, 0x56, 0x19, 0x31, 0x4a, 0x42, 0xa3, 0xda, 0x86, 0xb0, 0x01,
            0x38, 0x7b, 0xfd, 0xb8, 0x0e, 0x0c, 0xfe, 0x42,
        ]
    );

    test_vector!(
        key_vector_one,
        [
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        [
            0xcf, 0x5e, 0xe9, 0xa0, 0x49, 0x4a, 0xa9, 0x61, 0x3e, 0x05, 0xd5, 0xed, 0x72, 0x5b,
            0x80, 0x4b, 0x12, 0xf4, 0xa4, 0x65, 0xee, 0x63, 0x5a, 0xcc, 0x3a, 0x31, 0x1d, 0xe8,
            0x74, 0x04, 0x89, 0xea, 0x28, 0x9d, 0x04, 0xf4, 0x3c, 0x75, 0x18, 0xdb, 0x56, 0xeb,
            0x44, 0x33, 0xe4, 0x98, 0xa1, 0x23, 0x8c, 0xd8, 0x46, 0x4d, 0x37, 0x63, 0xdd, 0xbb,
            0x92, 0x22, 0xee, 0x3b, 0xd8, 0xfa, 0xe3, 0xc8,
        ]
    );

    test_vector!(
        iv_vector_one,
        [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        [
            0x2b, 0x8f, 0x4b, 0xb3, 0x79, 0x83, 0x06, 0xca, 0x51, 0x30, 0xd4, 0x7c, 0x4f, 0x8d,
            0x4e, 0xd1, 0x3a, 0xa0, 0xed, 0xcc, 0xc1, 0xbe, 0x69, 0x42, 0x09, 0x0f, 0xae, 0xec,
            0xa0, 0xd7, 0x59, 0x9b, 0x7f, 0xf0, 0xfe, 0x61, 0x6b, 0xb2, 0x5a, 0xa0, 0x15, 0x3a,
            0xd6, 0xfd, 0xc8, 0x8b, 0x95, 0x49, 0x03, 0xc2, 0x24, 0x26, 0xd4, 0x78, 0xb9, 0x7b,
            0x22, 0xb8, 0xf9, 0xb1, 0xdb, 0x00, 0xcf, 0x06,
        ]
    );

    test_vector!(
        filled_vector,
        [0xff; 40],
        [
            0xe1, 0x63, 0xbb, 0xf8, 0xc9, 0xa7, 0x39, 0xd1, 0x89, 0x25, 0xee, 0x83, 0x62, 0xda,
            0xd2, 0xcd, 0xc9, 0x73, 0xdf, 0x05, 0x22, 0x5a, 0xfb, 0x2a, 0xa2, 0x63, 0x96, 0xf2,
            0xa9, 0x84, 0x9a, 0x4a, 0x44, 0x5e, 0x05, 0x47, 0xd3, 0x1c, 0x16, 0x23, 0xc5, 0x37,
            0xdf, 0x4b, 0xa8, 0x5c, 0x70, 0xa9, 0x88, 0x4a, 0x35, 0xbc, 0xbf, 0x3d, 0xfa, 0xb0,
            0x77, 0xe9, 0x8b, 0x0f, 0x68, 0x13, 0x5f, 0x54,
        ]
    );

    test_vector!(
        every_even_bit_vector,
        [0x55; 40],
        [
            0x7c, 0xb7, 0x82, 0x14, 0xe4, 0xd3, 0x46, 0x5b, 0x6d, 0xc6, 0x2c, 0xf7, 0xa1, 0x53,
            0x8c, 0x88, 0x99, 0x69, 0x52, 0xb4, 0xfb, 0x72, 0xcb, 0x61, 0x05, 0xf1, 0x24, 0x3c,
            0xe3, 0x44, 0x2e, 0x29, 0x75, 0xa5, 0x9e, 0xbc, 0xd2, 0xb2, 0xa5, 0x98, 0x29, 0x0d,
            0x75, 0x38, 0x49, 0x1f, 0xe6, 0x5b, 0xdb, 0xfe, 0xfd, 0x06, 0x0d, 0x88, 0x79, 0x81,
            0x20, 0xa7, 0x0d, 0x04, 0x9d, 0xc2, 0x67, 0x7d,
        ]
    );

    test_vector!(
        every_odd_bit_vector,
        [0xaa; 40],
        [
            0x40, 0xf9, 0xab, 0x86, 0xc8, 0xf9, 0xa1, 0xa0, 0xcd, 0xc0, 0x5a, 0x75, 0xe5, 0x53,
            0x1b, 0x61, 0x2d, 0x71, 0xef, 0x7f, 0x0c, 0xf9, 0xe3, 0x87, 0xdf, 0x6e, 0xd6, 0x97,
            0x2f, 0x0a, 0xae, 0x21, 0x31, 0x1a, 0xa5, 0x81, 0xf8, 0x16, 0xc9, 0x0e, 0x8a, 0x99,
            0xde, 0x99, 0x0b, 0x6b, 0x95, 0xaa, 0xc9, 0x24, 0x50, 0xf4, 0xe1, 0x12, 0x71, 0x26,
            0x67, 0xb8, 0x04, 0xc9, 0x9e, 0x9c, 0x6e, 0xda,
        ]
    );

    test_vector!(
        sequence_vector,
        [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44,
            0x33, 0x22, 0x11, 0x00, 0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78,
        ],
        [
            0xdb, 0x43, 0xad, 0x9d, 0x1e, 0x84, 0x2d, 0x12, 0x72, 0xe4, 0x53, 0x0e, 0x27, 0x6b,
            0x3f, 0x56, 0x8f, 0x88, 0x59, 0xb3, 0xf7, 0xcf, 0x6d, 0x9d, 0x2c, 0x74, 0xfa, 0x53,
            0x80, 0x8c, 0xb5, 0x15, 0x7a, 0x8e, 0xbf, 0x46, 0xad, 0x3d, 0xcc, 0x4b, 0x6c, 0x7d,
            0xad, 0xde, 0x13, 0x17, 0x84, 0xb0, 0x12, 0x0e, 0x0e, 0x22, 0xf6, 0xd5, 0xf9, 0xff,
            0xa7, 0x40, 0x7d, 0x4a, 0x21, 0xb6, 0x95, 0xd9,
        ]
    );
}
