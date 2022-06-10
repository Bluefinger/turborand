use crate::Cell;

/// Trait for implementing `State` to be used in a Rng.
pub trait State {
    /// Initialise a state with a seed value.
    fn with_seed(seed: u64) -> Self where Self: Sized;
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
    fn with_seed(seed: u64) -> Self where Self: Sized {
        Self(Cell::new(seed))
    }

    fn get(&self) -> u64 {
        self.0.get()
    }

    fn set(&self, value: u64) {
        self.0.set(value);
    }
}
