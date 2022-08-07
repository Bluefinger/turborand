#[derive(PartialEq, Eq)]
pub(crate) struct EntropyBuffer<const SIZE: usize> {
    buffer: Vec<u8>,
    cursor: usize,
}

impl<const SIZE: usize> EntropyBuffer<SIZE> {
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: SIZE,
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        SIZE == self.cursor
    }

    #[inline]
    fn remaining_buffer(&self) -> usize {
        SIZE - self.cursor
    }

    #[inline]
    fn reset_buffer<B: AsRef<[u32]>>(&mut self, buffer: B) {
        let buffer: &[u8] = bytemuck::cast_slice(buffer.as_ref());

        assert!(buffer.len() == SIZE);

        if self.buffer.is_empty() {
            self.buffer.extend(buffer);
        } else {
            self.buffer
                .iter_mut()
                .zip(buffer)
                .for_each(|(slot, &val)| *slot = val);
        }

        self.cursor = 0;
    }

    #[inline]
    fn fill(&mut self, output: &mut [u8]) -> usize {
        let length = output.len().min(self.remaining_buffer());

        let to = self.cursor + length;

        output[..length].copy_from_slice(&self.buffer[self.cursor..to]);

        self.cursor = to;

        length
    }

    #[inline]
    pub(crate) fn empty_buffer(&mut self) {
        self.buffer.clear();
        self.cursor = SIZE;
    }

    #[inline]
    pub(crate) fn fill_bytes_with_source<B: AsMut<[u8]>, R: AsRef<[u32]>, S: Fn() -> R>(
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

            let filled = self.fill(output);

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

        buffer.reset_buffer([1, 2]);

        assert!(!buffer.is_empty());

        let mut output = [0u8; 4];

        let filled = buffer.fill(&mut output);

        assert_eq!(&output, &[1, 0, 0, 0]);
        assert_eq!(&filled, &4);
        assert_eq!(&buffer.cursor, &4);
        assert!(!buffer.is_empty());

        let mut output = [0u8; 6];

        let filled = buffer.fill(&mut output);

        assert_eq!(&output, &[2, 0, 0, 0, 0, 0]);
        assert_eq!(&filled, &4);
        assert_eq!(&buffer.cursor, &8);
        assert!(buffer.is_empty());

        buffer.reset_buffer([1, 2]);

        assert!(!buffer.is_empty());

        let filled = buffer.fill(&mut output[filled..]);

        assert_eq!(&output, &[2, 0, 0, 0, 1, 0]);
        assert_eq!(&filled, &2);
        assert_eq!(&buffer.cursor, &2);
    }
}
