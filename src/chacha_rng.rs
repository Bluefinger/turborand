//! A cryptographically secure PRNG (CSPRNG) based on [ChaCha8](https://cr.yp.to/chacha.html).
use crate::{
    source::chacha::{utils::AlignedSeed, ChaCha8},
    ForkableCore, GenCore, SecureCore, SeededCore, TurboCore,
};

#[cfg(feature = "std")]
use crate::{entropy::generate_entropy, Rc};

#[cfg(feature = "fmt")]
use crate::Debug;

#[cfg(feature = "serialize")]
use crate::{Deserialize, Serialize};

/// A Random Number generator, powered by the `ChaCha8` algorithm.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fmt", derive(Debug))]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(docsrs, doc(cfg(feature = "chacha")))]
#[repr(transparent)]
pub struct ChaChaRng(ChaCha8);

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl ChaChaRng {
    /// Creates a new [`ChaChaRng`] with a randomised seed.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        SECURE.with(|rng| rng.fork())
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

impl ForkableCore for ChaChaRng {
    #[inline]
    #[must_use]
    fn fork(&self) -> Self {
        Self(ChaCha8::with_seed(AlignedSeed::from(self.0.rand())))
    }
}

impl SecureCore for ChaChaRng {}

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
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

#[cfg(feature = "std")]
thread_local! {
    static SECURE: Rc<ChaChaRng> = Rc::new(ChaChaRng::with_seed(generate_entropy()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "fmt")]
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
