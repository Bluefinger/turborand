use crate::{entropy::generate_entropy, source::chacha::ChaCha8, Debug, State};

/// A Random Number generator, powered by the `WyRand` algorithm.
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct SecureRng<S: State<Seed = [u32; 16]> + Debug>(ChaCha8<S>);

impl<S: State<Seed = [u32; 16]> + Debug> SecureRng<S> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::with_seed(generate_entropy::<40>())
    }

    #[inline]
    #[must_use]
    pub fn with_seed(seed: [u8; 40]) -> Self {
        Self(ChaCha8::with_seed(seed))
    }

    rand_int_const!(gen_u128, u128, "Returns a random `u128` value.");
    rand_int_const!(gen_i128, i128, "Returns a random `i128` value.");
    rand_int_const!(gen_u64, u64, "Returns a random `u64` value.");
    rand_int_const!(gen_i64, i64, "Returns a random `i64` value.");
    rand_int_const!(gen_u32, u32, "Returns a random `u32` value.");
    rand_int_const!(gen_i32, i32, "Returns a random `i32` value.");
    rand_int_const!(gen_u16, u16, "Returns a random `u16` value.");
    rand_int_const!(gen_i16, i16, "Returns a random `i16` value.");
    rand_int_const!(gen_u8, u8, "Returns a random `u8` value.");
    rand_int_const!(gen_i8, i8, "Returns a random `i8` value.");
    rand_int_const!(gen_usize, usize, "Returns a random `usize` value.");
    rand_int_const!(gen_isize, isize, "Returns a random `isize` value.");
}
