use crate::{Cell, Debug};

#[cfg(feature = "atomic")]
use crate::{AtomicU64, Ordering};

#[cfg(feature = "serialize")]
use crate::{Deserialize, Serialize};

/// Trait for implementing [`State`] to be used in a `Rng`.
///
/// Those implementing [`State`] should also ensure to implement
/// a custom [`Debug`] formatter on the structs in order to prevent
/// leaking the Rng's state via debug, which could have security
/// implications if one wishes to obfuscate the Rng's state.
pub trait State: Sized {
    type Seed: Sized + Default;
    /// Initialise a state with a seed value.
    fn with_seed(seed: Self::Seed) -> Self
    where
        Self: Sized;
    /// Return the current state.
    fn get(&self) -> Self::Seed;
    /// Set the state with a new value.
    fn set(&self, value: Self::Seed);
}

/// Non-[`Send`] and [`Sync`] state for `Rng`. Stores the current
/// state of the PRNG in a [`Cell`].
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct CellState<S: Copy + Default>(Cell<S>);

impl<S: Copy + Default> State for CellState<S> {
    type Seed = S;

    #[inline]
    fn with_seed(seed: S) -> Self {
        Self(Cell::new(seed))
    }

    #[inline]
    fn get(&self) -> S {
        self.0.get()
    }

    #[inline]
    fn set(&self, value: S) {
        self.0.set(value);
    }
}

impl<S: Copy + Default> Debug for CellState<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CellState").finish()
    }
}

/// [`Send`] and [`Sync`] state for `Rng`. Stores the current
/// state of the PRNG in a [`AtomicU64`].
///
/// ```
/// use turborand::*;
/// use std::sync::Arc;
/// use std::thread;
///
/// let rand = Arc::new(atomic_rng!()); // Will not compile with `CellState`
/// let rand2 = rand.clone();
///
/// let thread_01 = thread::spawn(move || {
///     rand.u64(..)
/// });
///
/// let thread_02 = thread::spawn(move || {
///     rand2.u64(..)
/// });
///
/// let res1 = thread_01.join();
/// let res2 = thread_02.join();
/// ```
#[cfg(feature = "atomic")]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(docsrs, doc(cfg(feature = "atomic")))]
#[repr(transparent)]
pub struct AtomicState(AtomicU64);

#[cfg(feature = "atomic")]
impl State for AtomicState {
    type Seed = u64;
    #[inline]
    fn with_seed(seed: u64) -> Self
    where
        Self: Sized,
    {
        Self(AtomicU64::new(seed))
    }

    #[inline]
    fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    #[inline]
    fn set(&self, value: u64) {
        self.0.store(value, Ordering::Relaxed);
    }
}

#[cfg(feature = "atomic")]
impl PartialEq for AtomicState {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

#[cfg(feature = "atomic")]
impl Eq for AtomicState {}

#[cfg(feature = "atomic")]
impl Debug for AtomicState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AtomicState").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_state() {
        let state = CellState::with_seed(1);

        assert_eq!(state.get(), 1);

        state.set(5);

        assert_eq!(state.get(), 5);
    }

    #[test]
    fn cell_state_no_leaking_debug() {
        let state = CellState::<u64>::with_seed(Default::default());

        assert_eq!(format!("{state:?}"), "CellState");
    }

    #[cfg(feature = "atomic")]
    #[test]
    fn atomic_state() {
        let state = AtomicState::with_seed(1);

        assert_eq!(state.get(), 1);

        state.set(5);

        assert_eq!(state.get(), 5);
    }

    #[cfg(feature = "atomic")]
    #[test]
    fn atomic_state_no_leaking_debug() {
        let state = AtomicState::with_seed(Default::default());

        assert_eq!(format!("{state:?}"), "AtomicState");
    }
}
