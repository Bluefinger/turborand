use crate::{
    internal::{CellState, State},
    Debug,
};

/// A ChaCha8 based Random Number Generator
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct ChaCha8<S: Debug + State = CellState<[u8; 32]>>
where
    S: State<Seed = [u8; 32]>,
{
    state: S,
}
