use crate::internal::state::{CellState, State};

#[cfg(feature = "fmt")]
use crate::Debug;

#[cfg(feature = "serialize")]
use crate::{Deserialize, Serialize};

/// A Wyrand Random Number Generator
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub(crate) struct WyRand<S: State = CellState> {
    state: S,
}

impl<S: State> WyRand<S> {
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

    #[inline(always)]
    fn generate(&self) -> [u8; core::mem::size_of::<u64>()] {
        let state = self.state.update(0xa076_1d64_78bd_642f);
        let t = u128::from(state).wrapping_mul(u128::from(state ^ 0xe703_7ed1_a0b4_28db));
        let ret = (t.wrapping_shr(64) ^ t) as u64;
        ret.to_le_bytes()
    }

    /// Generates random bytes from the RNG source.
    #[inline]
    pub(crate) fn rand<const SIZE: usize>(&self) -> [u8; SIZE] {
        let mut output = [0u8; SIZE];

        self.fill(&mut output);

        output
    }

    #[inline]
    pub fn fill<B: AsMut<[u8]>>(&self, mut buffer: B) {
        let mut output = buffer.as_mut();

        while output.len() >= 8 {
            let (target, remainder) = output.split_at_mut(8);

            target.copy_from_slice(&self.generate());

            output = remainder;
        }

        if !output.is_empty() {
            let input = self.generate();

            let fill = output.len().min(input.len());

            output.copy_from_slice(&input[..fill]);
        }
    }
}

#[cfg(feature = "fmt")]
impl<S: State + Debug> Debug for WyRand<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

    #[cfg(feature = "fmt")]
    #[test]
    fn clone() {
        let rng1 = WyRand::<CellState>::with_seed(1);
        let rng2 = WyRand::<CellState>::with_seed(1);

        let cloned1 = rng1.clone();
        let cloned2 = rng2.clone();

        assert_eq!(
            &rng1, &cloned1,
            "cloned rngs should match against the original"
        );
        assert_eq!(
            &rng2, &cloned2,
            "cloned rngs should match against the original"
        );
        assert_eq!(
            &cloned1.generate(),
            &cloned2.generate(),
            "cloning should be deterministic"
        );
    }

    #[cfg(feature = "fmt")]
    #[test]
    fn no_leaking_debug() {
        let rng = WyRand::<CellState>::with_seed(Default::default());

        assert_eq!(format!("{:?}", rng), "WyRand(CellState)");
    }
}
