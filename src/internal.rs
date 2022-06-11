use crate::{Cell, Debug};

/// Trait for implementing `State` to be used in a Rng.
///
/// Those implementing `State` should also ensure to implement
/// a custom `Debug` formatter on the structs in order to prevent
/// leaking the Rng's state via debug, which could have security
/// implications if one wishes to obfuscate the Rng's state.
pub trait State {
    /// Initialise a state with a seed value.
    fn with_seed(seed: u64) -> Self
    where
        Self: Sized;
    /// Return the current state.
    fn get(&self) -> u64;
    /// Set the state with a new value.
    fn set(&self, value: u64);
}

/// Non-`Send` and `Sync` state for Rng. Stores the current
/// state of the PRNG in a `Cell`.
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct CellState(Cell<u64>);

impl State for CellState {
    fn with_seed(seed: u64) -> Self
    where
        Self: Sized,
    {
        Self(Cell::new(seed))
    }

    fn get(&self) -> u64 {
        self.0.get()
    }

    fn set(&self, value: u64) {
        self.0.set(value);
    }
}

impl Debug for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CellState").finish()
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
        let state = CellState::with_seed(Default::default());

        assert_eq!(format!("{:?}", state), "CellState");
    }
}
