use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    thread,
};

use crate::Instant;

use getrandom::{getrandom, Error};

/// This is a fallback in case other sources are not available. It is not meant
/// to be super secure, but to provide at least something in case of absolute
/// failure.
#[inline]
fn fallback_entropy<B: AsMut<[u8]>>(mut buffer: B) -> Result<(), Error> {
    let mut hasher = DefaultHasher::new();
    Instant::now().hash(&mut hasher);
    thread::current().id().hash(&mut hasher);

    let mut buffer = buffer.as_mut();
    let mut remaining = buffer.len();

    while remaining > 0 {
        remaining.hash(&mut hasher);
        let output = hasher.finish().to_ne_bytes();
        let fill = output.len().min(remaining);
        buffer[..fill].copy_from_slice(&output[..fill]);
        buffer = &mut buffer[fill..];
        remaining -= fill;
    }

    Ok(())
}

/// Generates a random buffer from some OS/Hardware sources
/// of entropy. Fallback provided in case OS/Hardware sources fail.
#[inline]
pub(crate) fn generate_entropy<const SIZE: usize>() -> [u8; SIZE] {
    let mut bytes = [0u8; SIZE];

    getrandom(&mut bytes)
        .or_else(|_| fallback_entropy(&mut bytes))
        .expect("Entropy sources should be available and not fail in order to sample random data");

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_entropy_source() {
        let mut result = [0u8; { core::mem::size_of::<u64>() }];

        fallback_entropy(&mut result).unwrap();

        assert_ne!(
            &u64::from_be_bytes(result),
            &0,
            "generated entropy should always be a non-zero value"
        );
    }

    #[test]
    fn large_fallback_entropy_source() {
        let mut result = [0u8; { core::mem::size_of::<u128>() }];

        fallback_entropy(&mut result).unwrap();

        let split = core::mem::size_of::<u64>();

        let mut part1 = [0u8; 8];
        part1.copy_from_slice(&result[..split]);
        let part1 = u64::from_ne_bytes(part1);

        let mut part2 = [0u8; 8];
        part2.copy_from_slice(&result[split..]);
        let part2 = u64::from_ne_bytes(part2);

        assert_ne!(
            part1, part2,
            "internal hasher should not output the same values to fill out the generated buffer"
        );
    }
}
