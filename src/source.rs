use crate::{
    internal::{CellState, State},
    Debug,
};

#[cfg(feature = "serialize")]
use crate::{Deserialize, Serialize};

/// A Wyrand Random Number Generator
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub(crate) struct WyRand<S: Debug + State = CellState> {
    state: S,
}

impl<S: State + Debug> WyRand<S> {
    /// Creates a new [`WyRand`] source with seeded value.
    #[inline]
    pub(crate) fn with_seed(seed: u64) -> Self {
        Self {
            state: S::with_seed(seed),
        }
    }

    /// Reseeds an existing [`WyRand`] source with a new seed value.
    #[inline]
    pub(crate) fn reseed(&self, seed: u64) {
        self.state.set(seed);
    }

    /// Generates random bytes from the RNG source.
    #[inline]
    pub(crate) fn rand(&self) -> [u8; core::mem::size_of::<u64>()] {
        let state = self.state.get().wrapping_add(0xa076_1d64_78bd_642f);
        self.state.set(state);
        let t = u128::from(state).wrapping_mul(u128::from(state ^ 0xe703_7ed1_a0b4_28db));
        let ret = (t.wrapping_shr(64) ^ t) as u64;
        ret.to_le_bytes()
    }
}

impl<S: State + Debug> Clone for WyRand<S> {
    /// Deterministically clones the [`WyRand`] source.
    fn clone(&self) -> Self {
        Self {
            state: S::with_seed(u64::from_le_bytes(self.rand())),
        }
    }
}

impl<S: State + Debug> Debug for WyRand<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WyRand").field(&self.state).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed() {
        let rng = WyRand::<CellState>::with_seed(Default::default());

        assert_eq!(rng.state.get(), 0);
    }

    #[test]
    fn reseed() {
        let rng = WyRand::<CellState>::with_seed(Default::default());

        rng.reseed(5);

        assert_eq!(
            rng.state.get(),
            5,
            "reseeds should always force the state to the given value"
        );
    }

    #[test]
    fn rand() {
        let rng = WyRand::<CellState>::with_seed(1);

        let output = rng.rand();

        assert_eq!(
            output.len(),
            core::mem::size_of::<u64>(),
            "output should be the same amount of bytes for an u64 value"
        );
        assert_eq!(
            &output,
            &[44, 237, 248, 225, 149, 22, 239, 205],
            "seeded output should match expected array values"
        );
    }

    #[test]
    fn clone() {
        let rng1 = WyRand::<CellState>::with_seed(1);
        let rng2 = WyRand::<CellState>::with_seed(1);

        let cloned1 = rng1.clone();
        let cloned2 = rng2.clone();

        assert_ne!(
            &rng1.rand(),
            &cloned1.rand(),
            "cloned rngs should not match against the original"
        );
        assert_ne!(
            &rng2.rand(),
            &cloned2.rand(),
            "cloned rngs should not match against the original"
        );
        assert_eq!(
            &cloned1.rand(),
            &cloned2.rand(),
            "cloning should be deterministic"
        );
    }

    #[test]
    fn no_leaking_debug() {
        let rng = WyRand::<CellState>::with_seed(Default::default());

        assert_eq!(format!("{:?}", rng), "WyRand(CellState)");
    }
}
