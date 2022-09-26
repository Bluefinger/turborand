//! A cryptographically secure PRNG (CSPRNG) based on [ChaCha8](https://cr.yp.to/chacha.html).
use crate::{
    entropy::generate_entropy, source::chacha::ChaCha8, GenCore, Rc, SecureCore, SeededCore,
    TurboCore,
};

use crate::source::chacha::utils::AlignedSeed;

#[cfg(feature = "serialize")]
use crate::{Deserialize, Serialize};

/// A Random Number generator, powered by the `ChaCha8` algorithm.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(docsrs, doc(cfg(feature = "chacha")))]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct ChaChaRng(ChaCha8);

impl ChaChaRng {
    /// Creates a new [`ChaChaRng`] with a randomised seed.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        SECURE.with(|rng| rng.as_ref().clone())
    }

    /// Reseeds the current thread-local generator.
    #[inline]
    pub fn reseed_local(seed: [u8; 40]) {
        SECURE.with(|rng| rng.reseed(seed));
    }
}

impl TurboCore for ChaChaRng {
    #[inline]
    fn fill_bytes(&self, buffer: &mut [u8]) {
        self.0.fill(buffer);
    }
}

impl GenCore for ChaChaRng {
    #[inline]
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
        self.0.rand()
    }
}

impl SeededCore for ChaChaRng {
    type Seed = [u8; 40];

    #[inline]
    #[must_use]
    fn with_seed(seed: Self::Seed) -> Self {
        Self(ChaCha8::with_seed(AlignedSeed::from(seed)))
    }

    #[inline]
    fn reseed(&self, seed: Self::Seed) {
        self.0.reseed(AlignedSeed::from(seed));
    }
}

impl SecureCore for ChaChaRng {}

impl Default for ChaChaRng {
    /// Initialises a default instance of [`ChaChaRng`]. Warning, the default is
    /// seeded with a randomly generated state, so this is **not** deterministic.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng1 = ChaChaRng::default();
    /// let rng2 = ChaChaRng::default();
    ///
    /// assert_ne!(rng1.u64(..), rng2.u64(..));
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ChaChaRng {
    /// Clones the [`ChaChaRng`] by deterministically deriving a new [`ChaChaRng`] based on the initial
    /// seed.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    ///
    /// let rng1 = ChaChaRng::with_seed([0u8; 40]);
    /// let rng2 = ChaChaRng::with_seed([0u8; 40]);
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

thread_local! {
    static SECURE: Rc<ChaChaRng> = Rc::new(ChaChaRng::with_seed(generate_entropy()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_leaking_debug() {
        let rng = ChaChaRng::with_seed([0u8; 40]);

        assert_eq!(format!("{:?}", rng), "ChaChaRng(ChaCha8)");
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn serde_tokens() {
        use serde_test::{assert_tokens, Token};

        let rng = ChaChaRng::with_seed([0u8; 40]);

        assert_tokens(
            &rng,
            &[
                Token::NewtypeStruct { name: "ChaChaRng" },
                Token::Struct {
                    name: "ChaCha8",
                    len: 2,
                },
                Token::BorrowedStr("state"),
                Token::Tuple { len: 16 },
                Token::U32(1634760805),
                Token::U32(857760878),
                Token::U32(2036477234),
                Token::U32(1797285236),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::U32(0),
                Token::TupleEnd,
                Token::BorrowedStr("cache"),
                Token::Tuple { len: 9 },
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(64),
                Token::TupleEnd,
                Token::StructEnd,
            ],
        );

        rng.gen::<16>();

        assert_tokens(
            &rng,
            &[
                Token::NewtypeStruct { name: "ChaChaRng" },
                Token::Struct {
                    name: "ChaCha8",
                    len: 2,
                },
                Token::BorrowedStr("state"),
                Token::Tuple { len: 16 },
                Token::U32(804192318),
                Token::U32(3594542985),
                Token::U32(3904396159),
                Token::U32(2711947551),
                Token::U32(3272508460),
                Token::U32(998218446),
                Token::U32(2296453912),
                Token::U32(505049583),
                Token::U32(1927367832),
                Token::U32(1097802169),
                Token::U32(1733510303),
                Token::U32(425094469),
                Token::U32(2739030578),
                Token::U32(28346074),
                Token::U32(3103619896),
                Token::U32(1123945486),
                Token::TupleEnd,
                Token::BorrowedStr("cache"),
                Token::Tuple { len: 9 },
                Token::U64(15438444565445410878),
                Token::U64(11647726043916688255),
                Token::U64(4287315583106450476),
                Token::U64(2169171444139891480),
                Token::U64(4715024415260232856),
                Token::U64(1825766843798996127),
                Token::U64(121745463539026481),
                Token::U64(4827309107960445752),
                Token::U64(16),
                Token::TupleEnd,
                Token::StructEnd,
            ],
        );
    }
}