use crate::{
    entropy::generate_entropy, source::wyrand::WyRand, CellState, Debug, Rc, SeededCore, TurboCore,
    TurboRand,
};

#[cfg(feature = "atomic")]
use crate::AtomicState;

#[cfg(feature = "serialize")]
use crate::{Deserialize, Serialize};

/// A Random Number generator, powered by the `WyRand` algorithm.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(docsrs, doc(cfg(feature = "wyrand")))]
#[repr(transparent)]
pub struct Rng(WyRand<CellState>);

impl Rng {
    /// Creates a new [`Rng`] with a randomised seed.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(WyRand::with_seed(RNG.with(|rng| rng.gen_u64())))
    }

    /// Reseeds the current thread-local generator.
    #[inline]
    pub fn reseed_local(seed: u64) {
        RNG.with(|rng| rng.reseed(seed));
    }
}

impl TurboCore for Rng {
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
        self.0.rand::<SIZE>()
    }

    fn fill_bytes<B: AsMut<[u8]>>(&self, buffer: B) {
        self.0.fill(buffer);
    }
}

impl SeededCore for Rng {
    type Seed = u64;

    fn with_seed(seed: Self::Seed) -> Self {
        Self(WyRand::with_seed(seed << 1 | 1))
    }

    fn reseed(&self, seed: Self::Seed) {
        self.0.reseed(seed);
    }
}

impl TurboRand for Rng {}

impl Default for Rng {
    /// Initialises a default instance of [`Rng`]. Warning, the default is
    /// seeded with a randomly generated state, so this is **not** deterministic.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng1 = Rng::default();
    /// let rng2 = Rng::default();
    ///
    /// assert_ne!(rng1.u64(..), rng2.u64(..));
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Rng {
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

/// A Random Number generator, powered by the `WyRand` algorithm, but with
/// thread-safe internal state.
#[cfg(feature = "atomic")]
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "wyrand", feature = "atomic"))))]
#[repr(transparent)]
pub struct AtomicRng(WyRand<AtomicState>);

#[cfg(feature = "atomic")]
impl AtomicRng {
    /// Creates a new [`AtomicRng`] with a randomised seed.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(WyRand::with_seed(RNG.with(|rng| rng.gen_u64())))
    }
}

#[cfg(feature = "atomic")]
impl Default for AtomicRng {
    /// Initialises a default instance of [`AtomicRng`]. Warning, the default is
    /// seeded with a randomly generated state, so this is **not** deterministic.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng1 = AtomicRng::default();
    /// let rng2 = AtomicRng::default();
    ///
    /// assert_ne!(rng1.u64(..), rng2.u64(..));
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "atomic")]
impl Clone for AtomicRng {
    /// Clones the [`AtomicRng`] by deterministically deriving a new [`AtomicRng`] based on the initial
    /// seed.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng1 = atomic_rng!(Default::default());
    /// let rng2 = atomic_rng!(Default::default());
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

#[cfg(feature = "atomic")]
impl TurboCore for AtomicRng {
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
        self.0.rand::<SIZE>()
    }

    fn fill_bytes<B: AsMut<[u8]>>(&self, buffer: B) {
        self.0.fill(buffer);
    }
}

#[cfg(feature = "atomic")]
impl SeededCore for AtomicRng {
    type Seed = u64;

    fn with_seed(seed: Self::Seed) -> Self {
        Self(WyRand::with_seed(seed << 1 | 1))
    }

    fn reseed(&self, seed: Self::Seed) {
        self.0.reseed(seed);
    }
}

#[cfg(feature = "atomic")]
impl TurboRand for AtomicRng {}

thread_local! {
    static RNG: Rc<Rng> = Rc::new(Rng(WyRand::with_seed(
        u64::from_ne_bytes(generate_entropy::<{ core::mem::size_of::<u64>() }>()),
    )));
}

#[cfg(test)]
mod tests {
    use crate::rng;

    #[cfg(feature = "atomic")]
    use crate::atomic_rng;

    #[cfg(feature = "serialize")]
    use serde_test::{assert_tokens, Token};

    use super::*;

    #[test]
    fn rng_no_leaking_debug() {
        let rng = rng!(Default::default());

        assert_eq!(format!("{:?}", rng), "Rng(WyRand(CellState))");
    }

    #[cfg(feature = "atomic")]
    #[test]
    fn atomic_no_leaking_debug() {
        let rng = atomic_rng!(Default::default());

        assert_eq!(format!("{:?}", rng), "AtomicRng(WyRand(AtomicState))");
    }

    #[cfg(feature = "rand")]
    #[test]
    fn rand_compatibility() {
        use rand_core::RngCore;

        use crate::{RandBorrowed, RandCompat};

        fn get_rand_num<R: RngCore>(rng: &mut R) -> u64 {
            rng.next_u64()
        }

        let rng = rng!(Default::default());

        let mut rand = RandCompat::from(rng);

        let result = get_rand_num(&mut rand);

        assert_eq!(
            result, 14_839_104_130_206_199_084,
            "Should receive expect random u64 output, got {} instead",
            result
        );

        let mut rng = rng!(Default::default());

        let mut rand = RandBorrowed::from(&mut rng);

        let result = get_rand_num(&mut rand);

        assert_eq!(
            result, 14_839_104_130_206_199_084,
            "Should receive expect random u64 output, got {} instead",
            result
        );
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn rng_serde_tokens() {
        let rng = rng!(12345);

        assert_tokens(
            &rng,
            &[
                Token::NewtypeStruct { name: "Rng" },
                Token::Struct {
                    name: "WyRand",
                    len: 1,
                },
                Token::BorrowedStr("state"),
                Token::NewtypeStruct { name: "CellState" },
                Token::U64(24691),
                Token::StructEnd,
            ],
        );
    }

    #[cfg(all(feature = "serialize", feature = "atomic"))]
    #[test]
    fn atomic_serde_tokens() {
        let rng = atomic_rng!(12345);

        assert_tokens(
            &rng,
            &[
                Token::NewtypeStruct { name: "AtomicRng" },
                Token::Struct {
                    name: "WyRand",
                    len: 1,
                },
                Token::BorrowedStr("state"),
                Token::NewtypeStruct {
                    name: "AtomicState",
                },
                Token::U64(24691),
                Token::StructEnd,
            ],
        );
    }
}
