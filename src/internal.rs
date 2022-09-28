//! Internal structs and traits for the `WyRand` PRNGs.
#[cfg(feature = "chacha")]
pub(crate) mod buffer;

#[cfg(feature = "wyrand")]
pub(crate) mod state;
