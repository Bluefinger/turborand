use crate::{thread, DefaultHasher, Hash, Hasher, Instant};

/// Generates a random number from some quick sources
/// of entropy. Always a non-zero and odd value being returned.
#[inline]
pub(crate) fn generate_entropy() -> u64 {
    let mut hasher = DefaultHasher::new();
    Instant::now().hash(&mut hasher);
    thread::current().id().hash(&mut hasher);
    let hash = hasher.finish();
    (hash << 1) | 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_source() {
        let result = generate_entropy();

        assert_ne!(
            &result, &0,
            "generated entropy should always be a non-zero value"
        );
        assert_eq!(
            result % 2,
            1,
            "generated entropy should always be an odd value"
        );
    }
}
