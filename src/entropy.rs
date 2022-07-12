use crate::{thread, DefaultHasher, Hash, Hasher, Instant};

/// Generates a random buffer from some quick sources
/// of entropy. Always a non-zero being returned.
#[inline]
pub(crate) fn generate_entropy<const SIZE: usize>() -> [u8; SIZE] {
    let mut hasher = DefaultHasher::new();
    Instant::now().hash(&mut hasher);
    thread::current().id().hash(&mut hasher);

    let mut bytes = [0u8; SIZE];
    let mut length = bytes.len();

    let mut buffer = bytes.as_mut();

    while length > 0 {
        length.hash(&mut hasher);
        let output = hasher.finish().to_ne_bytes();
        let fill = output.len().min(length);
        buffer[..fill].copy_from_slice(&output[..fill]);
        buffer = &mut buffer[fill..];
        length -= fill;
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_source() {
        let result = generate_entropy::<{ core::mem::size_of::<u64>() }>();

        assert_ne!(
            &u64::from_be_bytes(result),
            &0,
            "generated entropy should always be a non-zero value"
        );
    }

    #[test]
    fn large_entropy_source() {
        let result = generate_entropy::<{ core::mem::size_of::<u128>() }>();

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
