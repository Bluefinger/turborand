use crate::{
    entropy::generate_entropy, CellState, Debug, Rc, SeededCore, State,
    TurboCore, TurboRand, WyRand,
};

#[cfg(feature = "serialize")]
use crate::{Serialize, Deserialize};

/// A Random Number generator, powered by the `WyRand` algorithm.
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Rng<S: State<Seed = u64> + Debug>(WyRand<S>);

impl<S: State<Seed = u64> + Debug> Rng<S> {
    /// Creates a new [`Rng`] with a randomised seed.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(WyRand::<S>::with_seed(RNG.with(|rng| rng.gen_u64())))
    }

    /// Reseeds the current thread-local generator.
    #[inline]
    pub fn reseed_local(seed: u64) {
        RNG.with(|rng| rng.reseed(seed));
    }
}

impl<S: State<Seed = u64> + Debug> TurboCore for Rng<S> {
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
        self.0.rand::<SIZE>()
    }

    fn fill_bytes<B: AsMut<[u8]>>(&self, buffer: B) {
        self.0.fill(buffer);
    }
}

impl<S: State<Seed = u64> + Debug> SeededCore for Rng<S> {
    type Seed = u64;

    fn with_seed(seed: Self::Seed) -> Self {
        Self(WyRand::with_seed(seed << 1 | 1))
    }

    fn reseed(&self, seed: Self::Seed) {
        self.0.reseed(seed);
    }
}

impl<S: State<Seed = u64> + Debug> TurboRand for Rng<S> {}

impl<S: State<Seed = u64> + Debug> Default for Rng<S> {
    /// Initialises a default instance of [`Rng`]. Warning, the default is
    /// seeded with a randomly generated state, so this is **not** deterministic.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng1 = Rng::<CellState<u64>>::default();
    /// let rng2 = Rng::<CellState<u64>>::default();
    ///
    /// assert_ne!(rng1.u64(..), rng2.u64(..));
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<S: State<Seed = u64> + Debug> Clone for Rng<S> {
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

impl<S: State<Seed = u64> + Debug> Debug for Rng<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Rng").field(&self.0).finish()
    }
}

thread_local! {
    static RNG: Rc<Rng<CellState<u64>>> = Rc::new(Rng(WyRand::<CellState<u64>>::with_seed(
        u64::from_ne_bytes(generate_entropy::<{ core::mem::size_of::<u64>() }>()),
    )));
}

#[cfg(test)]
mod tests {
    use crate::rng;

    use super::*;

    #[test]
    fn no_leaking_debug() {
        let rng = rng!(Default::default());

        assert_eq!(format!("{:?}", rng), "Rng(WyRand(CellState))");
    }

    #[cfg(feature = "rand")]
    #[test]
    fn rand_compatibility() {
        use rand_core::RngCore;

        use crate::RandCompat;

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
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn serialize_rng() {
        let rng = rng!(12345);

        let json = serde_json::to_string(&rng).unwrap();

        assert_eq!(json, "{\"state\":24691}", "Serialized output not as expected");
    }
}
