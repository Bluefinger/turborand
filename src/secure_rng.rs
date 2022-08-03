use crate::{
    entropy::generate_entropy, source::chacha::ChaCha8, Rc, SecureCore, SeededCore, TurboCore,
    TurboRand,
};

/// A Random Number generator, powered by the `ChaCha8` algorithm.
#[derive(PartialEq, Eq)]
#[cfg_attr(docsrs, doc(cfg(feature = "secure")))]
#[repr(transparent)]
pub struct SecureRng(ChaCha8);

impl SecureRng {
    /// Creates a new [`SecureRng`] with a randomised seed.
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

impl TurboCore for SecureRng {
    #[inline]
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
        self.0.rand::<SIZE>()
    }

    #[inline]
    fn fill_bytes<B: AsMut<[u8]>>(&self, buffer: B) {
        self.0.fill(buffer);
    }
}

impl SeededCore for SecureRng {
    type Seed = [u8; 40];

    #[inline]
    #[must_use]
    fn with_seed(seed: Self::Seed) -> Self {
        Self(ChaCha8::with_seed(seed))
    }

    #[inline]
    fn reseed(&self, seed: Self::Seed) {
        self.0.reseed(seed);
    }
}

impl SecureCore for SecureRng {}
impl TurboRand for SecureRng {}

impl Default for SecureRng {
    /// Initialises a default instance of [`SecureRng`]. Warning, the default is
    /// seeded with a randomly generated state, so this is **not** deterministic.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng1 = SecureRng::default();
    /// let rng2 = SecureRng::default();
    ///
    /// assert_ne!(rng1.u64(..), rng2.u64(..));
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SecureRng {
    /// Clones the [`SecureRng`] by deterministically deriving a new [`SecureRng`] based on the initial
    /// seed.
    ///
    /// # Example
    /// ```
    /// use turborand::*;
    ///
    /// let rng1 = SecureRng::with_seed([0u8; 40]);
    /// let rng2 = SecureRng::with_seed([0u8; 40]);
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
    static SECURE: Rc<SecureRng> = Rc::new(SecureRng::with_seed(generate_entropy::<40>()));
}
