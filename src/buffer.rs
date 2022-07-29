#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct EntropyBuffer<const SIZE: usize> {
    buffer: Option<[u8; SIZE]>,
    cursor: usize,
}

impl<const SIZE: usize> EntropyBuffer<SIZE> {
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            buffer: None,
            cursor: 0,
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        if let Some(buffer) = self.buffer {
            buffer.len() == self.cursor
        } else {
            true
        }
    }

    #[inline]
    fn remaining_buffer(&self) -> usize {
        if let Some(buffer) = self.buffer {
            buffer.len() - self.cursor
        } else {
            0
        }
    }

    #[inline]
    fn reset_buffer(&mut self, buffer: [u8; SIZE]) {
        self.buffer = Some(buffer);
        self.cursor = 0;
    }

    #[inline]
    fn fill(&mut self, output: &mut [u8]) -> Option<usize> {
        self.buffer.map(|buffer| {
            let length = output.len().min(self.remaining_buffer());

            output[..length].copy_from_slice(&buffer[self.cursor..(self.cursor + length)]);

            self.cursor += length;

            length
        })
    }

    #[inline]
    pub(crate) fn fill_bytes_with_source<B: AsMut<[u8]>, S: Fn() -> [u8; SIZE]>(
        &mut self,
        mut output: B,
        source: S,
    ) {
        let mut output = output.as_mut();
        let mut remaining = output.len();
        
        while remaining > 0 {
            if self.is_empty() {
                self.reset_buffer(source());
            }

            // Unwrap here is fine, because we check before whether
            // the buffer is empty and refill it, so here will always
            // return Some(usize).
            let filled = self.fill(output).unwrap();

            output = &mut output[filled..];

            remaining -= filled;
        }
    }
}

impl<const SIZE: usize> Default for EntropyBuffer<SIZE> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialises_as_empty() {
        let buffer = EntropyBuffer::<8>::new();

        assert!(buffer.is_empty(), "Buffer should be empty on init");
    }

    #[test]
    fn fills_byte_slices() {
        let mut buffer = EntropyBuffer::<8>::new();

        buffer.reset_buffer([1, 2, 3, 4, 5, 6, 7, 8]);

        assert!(!buffer.is_empty());

        let mut output = [0u8; 4];

        let filled = buffer.fill(&mut output).unwrap();

        assert_eq!(&output, &[1, 2, 3, 4]);
        assert_eq!(&filled, &4);
        assert_eq!(&buffer.cursor, &4);
        assert!(!buffer.is_empty());

        let mut output = [0u8; 6];

        let filled = buffer.fill(&mut output).unwrap();

        assert_eq!(&output, &[5, 6, 7, 8, 0, 0]);
        assert_eq!(&filled, &4);
        assert_eq!(&buffer.cursor, &8);
        assert!(buffer.is_empty());

        buffer.reset_buffer([1, 2, 3, 4, 5, 6, 7, 8]);

        assert!(!buffer.is_empty());

        let filled = buffer.fill(&mut output[filled..]).unwrap();

        assert_eq!(&output, &[5, 6, 7, 8, 1, 2]);
        assert_eq!(&filled, &2);
        assert_eq!(&buffer.cursor, &2);
    }
}
