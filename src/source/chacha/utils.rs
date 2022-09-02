use super::constants::INITIAL_STATE;

#[repr(align(16))]
pub(crate) struct AlignedSeed([u32; 10]);

impl From<[u8; 40]> for AlignedSeed {
    #[inline]
    fn from(seed: [u8; 40]) -> Self {
        Self(bytemuck::cast(seed))
    }
}

impl std::ops::Deref for AlignedSeed {
    type Target = [u32; 10];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[inline]
pub(super) fn increment_counter(mut state: [u32; 16]) -> Option<[u32; 16]> {
    let counter = ((state[13] as u64) << 32) | (state[12] as u64);

    counter.checked_add(1).map(|updated_counter| {
        state[12] = (updated_counter & 0xFFFF_FFFF) as u32;
        state[13] = ((updated_counter >> 32) & 0xFFFF_FFFF) as u32;
        state
    })
}

#[inline]
const fn pack_into_u32(input: &[u8]) -> u32 {
    assert!(input.len() == 4);

    (input[0] as u32)
        | ((input[1] as u32) << 8)
        | ((input[2] as u32) << 16)
        | ((input[3] as u32) << 24)
}

#[inline]
pub(super) fn init_state(seed: AlignedSeed) -> [u32; 16] {
    [
        pack_into_u32(&INITIAL_STATE[..4]),
        pack_into_u32(&INITIAL_STATE[4..8]),
        pack_into_u32(&INITIAL_STATE[8..12]),
        pack_into_u32(&INITIAL_STATE[12..]),
        seed[0],
        seed[1],
        seed[2],
        seed[3],
        seed[4],
        seed[5],
        seed[6],
        seed[7],
        0,
        0,
        seed[8],
        seed[9],
    ]
}

#[inline]
fn add_xor_rotate<const A: usize, const B: usize, const C: usize, const LEFT: u32>(
    input: &mut [u32; 16],
) {
    input[A] = input[A].wrapping_add(input[B]);
    input[C] ^= input[A];
    input[C] = input[C].rotate_left(LEFT);
}

#[inline]
fn quarter_round<const A: usize, const B: usize, const C: usize, const D: usize>(
    input: &mut [u32; 16],
) {
    add_xor_rotate::<A, B, D, 16>(input);
    add_xor_rotate::<C, D, B, 12>(input);
    add_xor_rotate::<A, B, D, 8>(input);
    add_xor_rotate::<C, D, B, 7>(input);
}

#[inline]
pub(super) fn calculate_block<const DOUBLE_ROUNDS: usize>(state: &[u32; 16]) -> [u32; 16] {
    assert!(DOUBLE_ROUNDS % 2 == 0, "DOUBLE_ROUNDS must be even number");

    let mut new_state = *state;

    // 8 Rounds of ChaCha, 4 loops * 2 rounds per loop = 8 Rounds
    for _ in 0..DOUBLE_ROUNDS {
        // Odd Rounds
        quarter_round::<0, 4, 8, 12>(&mut new_state);
        quarter_round::<1, 5, 9, 13>(&mut new_state);
        quarter_round::<2, 6, 10, 14>(&mut new_state);
        quarter_round::<3, 7, 11, 15>(&mut new_state);
        // Even Rounds
        quarter_round::<0, 5, 10, 15>(&mut new_state);
        quarter_round::<1, 6, 11, 12>(&mut new_state);
        quarter_round::<2, 7, 8, 13>(&mut new_state);
        quarter_round::<3, 4, 9, 14>(&mut new_state);
    }

    new_state
        .iter_mut()
        .zip(state.iter())
        .for_each(|(new, &old)| *new = new.wrapping_add(old));

    new_state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_quarter_round_state() {
        let mut state: [u32; 16] = [
            0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
            0x2a5f714c, 0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0x3d631689,
            0x2098d9d6, 0x91dbd320,
        ];

        quarter_round::<2, 7, 8, 13>(&mut state);

        let expected_state = [
            0x879531e0, 0xc5ecf37d, 0xbdb886dc, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
            0xcfacafd2, 0xe46bea80, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0xccc07c79,
            0x2098d9d6, 0x91dbd320,
        ];

        assert_eq!(&state, &expected_state);
    }

    #[test]
    fn calculate_block_state() {
        let state: [u32; 16] = [
            0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908,
            0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x00000000,
            0x4a000000, 0x00000000,
        ];

        let state = calculate_block::<10>(&state);

        let expected_state: [u32; 16] = [
            0xf3514f22, 0xe1d91b40, 0x6f27de2f, 0xed1d63b8, 0x821f138c, 0xe2062c3d, 0xecca4f7e,
            0x78cff39e, 0xa30a3b8a, 0x920a6072, 0xcd7479b5, 0x34932bed, 0x40ba4c79, 0xcd343ec6,
            0x4c2c21ea, 0xb7417df0,
        ];

        assert_eq!(&state, &expected_state);
    }
}
