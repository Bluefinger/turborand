//! Compatibility shims for the `rand` crate ecosystem.

use crate::{
    traits::{GenCore, TurboCore},
    RngCore,
};

#[cfg(feature = "wyrand")]
use crate::rng::Rng;

#[cfg(feature = "chacha")]
use crate::chacha_rng::ChaChaRng;

#[cfg(feature = "atomic")]
use crate::rng::AtomicRng;

/// A wrapper struct around [`TurboCore`] to allow implementing
/// [`RngCore`] trait in a compatible manner.
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct RandCompat<T: TurboCore + GenCore>(T);

#[cfg(feature = "std")]
impl<T: TurboCore + GenCore + Default> RandCompat<T> {
    /// Creates a new [`RandCompat`] with a randomised seed.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    /// use rand_core::RngCore;
    ///
    /// let mut rng = RandCompat::<Rng>::new();
    /// let mut buffer = [0u8; 32];
    ///
    /// rng.fill_bytes(&mut buffer);
    ///
    /// assert_ne!(&buffer, &[0u8; 32]);
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(T::default())
    }
}

#[cfg(feature = "std")]
impl<T: TurboCore + GenCore + Default> Default for RandCompat<T> {
    /// Initialises a default instance of [`RandCompat`]. Warning, the default is
    /// seeded with a randomly generated state, so this is **not** deterministic.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    /// use rand_core::RngCore;
    ///
    /// let mut rng1 = RandCompat::<Rng>::default();
    /// let mut rng2 = RandCompat::<Rng>::default();
    ///
    /// assert_ne!(rng1.next_u64(), rng2.next_u64());
    /// ```
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TurboCore + GenCore> RngCore for RandCompat<T> {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.0.gen_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.0.gen_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.0.fill_bytes(dest);
        Ok(())
    }
}

impl<T: TurboCore + GenCore> From<T> for RandCompat<T> {
    #[inline]
    fn from(rng: T) -> Self {
        Self(rng)
    }
}

#[cfg(feature = "wyrand")]
impl From<RandCompat<Rng>> for Rng {
    #[inline]
    fn from(rand: RandCompat<Rng>) -> Self {
        rand.0
    }
}

#[cfg(feature = "atomic")]
impl From<RandCompat<AtomicRng>> for AtomicRng {
    #[inline]
    fn from(rand: RandCompat<AtomicRng>) -> Self {
        rand.0
    }
}

#[cfg(feature = "chacha")]
impl From<RandCompat<ChaChaRng>> for ChaChaRng {
    #[inline]
    fn from(rand: RandCompat<ChaChaRng>) -> Self {
        rand.0
    }
}

/// A wrapper struct around a borrowed [`TurboCore`] instance to allow
/// implementing [`RngCore`] trait in a compatible manner. Uses a mutable
/// reference to gain exclusive control over the [`TurboCore`] instance.
/// [`RngCore`] uses `&mut self` for its methods, so [`RandBorrowed`] should
/// impose the same requirements onto [`TurboCore`] in needing a `&mut`
/// reference.
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct RandBorrowed<'a, T: TurboCore + GenCore>(&'a mut T);

impl<'a, T: TurboCore + GenCore> From<&'a mut T> for RandBorrowed<'a, T> {
    /// Convert a [`TurboCore`] reference into a [`RandBorrowed`] struct,
    /// allowing a borrowed reference to be used with the `rand` crate
    /// ecosystem.
    ///
    /// # Example
    /// ```
    /// use turborand::prelude::*;
    /// use rand_core::RngCore;
    ///
    /// let mut turbo = Rng::with_seed(Default::default());
    ///
    /// let mut rng = RandBorrowed::from(&mut turbo);
    ///
    /// assert_eq!(rng.next_u32(), 3791187244);
    /// ```
    #[inline]
    fn from(rng: &'a mut T) -> Self {
        Self(rng)
    }
}

impl<'a, T: TurboCore + GenCore + Default> RngCore for RandBorrowed<'a, T> {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.0.gen_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.0.gen_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.0.fill_bytes(dest);
        Ok(())
    }
}
