use crate::TurboRand;

/// Similar to a Uniform distribution, after returning a number in the range [0,n], n is increased by 1.
/// Adapted from https://github.com/rust-random/rand/blob/master/src/seq/increasing_uniform.rs.
pub(crate) struct IncreasingUniformIter<R: TurboRand> {
    rng: R,
    n: u64,
    // Chunk is a random number in [0, (n + 1) * (n + 2) *..* (n + chunk_remaining) )
    chunk: u64,
    chunk_remaining: u8,
    len: usize,
}

impl<R: TurboRand> IncreasingUniformIter<R> {
    /// Create a dice roller.
    /// The next item returned will be a random number in the range [0,n]
    #[inline]
    pub(crate) fn new(rng: R, n: u64, len: usize) -> Self {
        // If n = 0, the first number returned will always be 0
        // so we don't need to generate a random number
        let chunk_remaining = u8::from(n == 0);

        Self {
            rng,
            n,
            chunk: 0,
            chunk_remaining,
            len,
        }
    }

    /// Returns a number in [0,n] and increments n by 1.
    /// Generates new random bits as needed
    /// Panics if `n >= u64::MAX`
    #[inline]
    fn next_swap_index(&mut self) -> usize {
        let next_n = self.n + 1;

        let next_chunk_remaining = self.chunk_remaining.checked_sub(1).unwrap_or_else(|| {
            // If the chunk is empty, generate a new chunk
            let (bound, remaining) = calculate_bound_u64(next_n);
            // bound = (n + 1) * (n + 2) *..* (n + remaining)
            self.chunk = self.rng.u64(..bound);
            // Chunk is a random number in
            // [0, (n + 1) * (n + 2) *..* (n + remaining) )
            remaining - 1
        });

        let result = if next_chunk_remaining == 0 {
            // `chunk` is a random number in the range [0..n+1)
            // Because `chunk_remaining` is about to be set to zero
            // we do not need to clear the chunk here
            self.chunk as usize
        } else {
            // `chunk` is a random number in a range that is a multiple of n+1
            // so r will be a random number in [0..n+1)
            let random = self.chunk % next_n;
            self.chunk /= next_n;
            random as usize
        };

        self.chunk_remaining = next_chunk_remaining;
        self.n = next_n;
        result
    }
}

impl<R: TurboRand> Iterator for IncreasingUniformIter<R> {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == (self.n as usize) {
            None
        } else {
            Some((self.n as usize, self.next_swap_index()))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len - self.n as usize;
        (len, Some(len))
    }
}

impl<R: TurboRand> ExactSizeIterator for IncreasingUniformIter<R> {
    #[inline]
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

#[inline]
/// Calculates `bound`, `count` such that bound (m)*(m+1)*..*(m + remaining - 1)
fn calculate_bound_u64(min: u64) -> (u64, u8) {
    debug_assert!(min > 1);

    #[inline]
    const fn inner(min: u64) -> (u64, u8) {
        let mut product = min;
        let mut current = min + 1;

        while let Some(p) = product.checked_mul(current) {
            product = p;
            current += 1;
        }

        let count = (current - min) as u8;
        (product, count)
    }

    const RESULT2: (u64, u8) = inner(2);

    match min {
        // Making this value a constant instead of recalculating it
        // gives a significant (~50%) performance boost for small shuffles
        2 => RESULT2,
        min => inner(min),
    }
}
