use core::cell::Cell;

#[cfg(feature = "fmt")]
use crate::Debug;

#[cfg(feature = "atomic")]
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "serialize")]
use crate::{Deserialize, Serialize};

#[cfg(all(feature = "serialize", feature = "atomic"))]
use crate::Visitor;

/// Trait for implementing [`State`] to be used in `WyRand`.
///
/// Those implementing [`State`] should also ensure to implement
/// a custom [`Debug`] formatter on the structs in order to prevent
/// leaking the Rng's state via debug, which could have security
/// implications if one wishes to obfuscate the Rng's state.
pub(crate) trait State: Sized {
    /// Initialise a state with a seed value.
    fn with_seed(seed: u64) -> Self
    where
        Self: Sized;
    /// Return the current state.
    fn get(&self) -> u64;
    /// Set the state with a new value.
    fn set(&self, value: u64);
    /// Update the internal state and return the new, resulting value
    #[inline(always)]
    fn update(&self, value: u64) -> u64 {
        let new_value = self.get().wrapping_add(value);
        self.set(new_value);
        new_value
    }
}

/// Non-[`Send`] and [`Sync`] state for `Rng`. Stores the current
/// state of the PRNG in a [`Cell`].
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub(crate) struct CellState(Cell<u64>);

impl State for CellState {
    #[inline]
    fn with_seed(seed: u64) -> Self {
        Self(Cell::new(seed))
    }

    #[inline]
    fn get(&self) -> u64 {
        self.0.get()
    }

    #[inline]
    fn set(&self, value: u64) {
        self.0.set(value);
    }
}

#[cfg(feature = "fmt")]
impl Debug for CellState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("CellState").finish()
    }
}

impl Clone for CellState {
    #[inline]
    fn clone(&self) -> Self {
        Self(Cell::new(self.get()))
    }
}

/// [`Send`] and [`Sync`] state for `AtomicRng`. Stores the current
/// state of the PRNG in a [`AtomicU64`].
///
/// ```
/// use turborand::prelude::*;
/// use std::sync::Arc;
/// use std::thread;
///
/// let rand = Arc::new(AtomicRng::default()); // Will not compile with `Rng`
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
#[repr(transparent)]
pub(crate) struct AtomicState(AtomicU64);

#[cfg(feature = "atomic")]
impl State for AtomicState {
    #[inline]
    fn with_seed(seed: u64) -> Self
    where
        Self: Sized,
    {
        Self(AtomicU64::new(seed))
    }

    #[inline]
    fn get(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }

    #[inline]
    fn set(&self, value: u64) {
        self.0.store(value, Ordering::SeqCst);
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

#[cfg(all(feature = "fmt", feature = "atomic"))]
impl Debug for AtomicState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("AtomicState").finish()
    }
}

#[cfg(feature = "atomic")]
impl Clone for AtomicState {
    #[inline]
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.get()))
    }
}

#[cfg(all(feature = "atomic", feature = "serialize"))]
impl Serialize for AtomicState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = self.get();
        serializer.serialize_newtype_struct("AtomicState", &value)
    }
}

#[cfg(all(feature = "atomic", feature = "serialize"))]
impl<'de> Deserialize<'de> for AtomicState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct AtomicStateVisitor;
        struct AtomicU64Visitor;

        impl Visitor<'_> for AtomicU64Visitor {
            type Value = u64;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("u64 state")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v)
            }
        }

        impl<'de> Visitor<'de> for AtomicStateVisitor {
            type Value = AtomicState;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("NewtypeStruct AtomicState")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = deserializer.deserialize_u64(AtomicU64Visitor)?;

                Ok(AtomicState::with_seed(value))
            }
        }

        deserializer.deserialize_newtype_struct("AtomicState", AtomicStateVisitor)
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

        state.update(6);

        assert_eq!(state.get(), 11);
    }

    #[cfg(all(feature = "fmt", feature = "alloc"))]
    #[test]
    fn cell_state_no_leaking_debug() {
        #[cfg(all(feature = "alloc", not(feature = "std")))]
        use alloc::format;

        let state = CellState::with_seed(Default::default());

        assert_eq!(format!("{state:?}"), "CellState");
    }

    #[cfg(feature = "atomic")]
    #[test]
    fn atomic_state() {
        let state = AtomicState::with_seed(1);

        assert_eq!(state.get(), 1);

        state.set(5);

        assert_eq!(state.get(), 5);

        state.update(6);

        assert_eq!(state.get(), 11);
    }

    #[cfg(all(feature = "fmt", feature = "atomic"))]
    #[test]
    fn atomic_state_no_leaking_debug() {
        let state = AtomicState::with_seed(Default::default());

        assert_eq!(format!("{state:?}"), "AtomicState");
    }
}
