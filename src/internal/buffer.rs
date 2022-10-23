use std::cell::UnsafeCell;

#[cfg(feature = "serialize")]
use crate::{Deserialize, Serialize, SerializeTuple, Visitor};

#[derive(Debug)]
pub(crate) struct EntropyBuffer<const SIZE: usize> {
    buffer: UnsafeCell<[u64; SIZE]>,
    cursor: UnsafeCell<usize>,
}

impl<const SIZE: usize> EntropyBuffer<SIZE> {
    #[cfg(feature = "serialize")]
    #[inline]
    fn from_serde(buffer: [u64; SIZE], cursor: usize) -> Self {
        Self {
            buffer: UnsafeCell::new(buffer),
            cursor: UnsafeCell::new(cursor),
        }
    }

    /// Create a new [`EntropyBuffer`].
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            buffer: UnsafeCell::new([0; SIZE]),
            cursor: UnsafeCell::new(Self::total_bytes()),
        }
    }

    /// Returns the total byte size of the [`EntropyBuffer`], indicating
    /// how much entropy it can store.
    #[inline(always)]
    const fn total_bytes() -> usize {
        SIZE * core::mem::size_of::<u64>()
    }

    /// Returns a reference to the buffer. Meant for avoiding extra copies
    /// from the underlying data.
    ///
    /// **WARNING**: No references should live while an update to the buffer
    /// is made.
    #[inline]
    #[must_use]
    fn get_buffer(&self) -> &[u64; SIZE] {
        // SAFETY: Data is always initialised and no mutable references
        // will exist during the liftime of the returned reference. This
        // can also cause data races if called from different threads, but
        // EntropyBuffer is not Sync, so this won't happen.
        unsafe { &*self.buffer.get() }
    }

    #[inline]
    fn get_cursor(&self) -> usize {
        // SAFETY: Data is always initialised and no mutable references
        // will exist during the liftime of the returned reference. This
        // can also cause data races if called from different threads, but
        // EntropyBuffer is not Sync, so this won't happen.
        unsafe { *self.cursor.get() }
    }

    /// Updates the buffer with a new array value.
    ///
    /// **Warning**: Must not be used while a reference to the buffer lives, else
    /// it won't be sound.
    #[inline]
    fn update_buffer(&self, buffer: [u64; SIZE]) {
        // SAFETY: Data is writable and does not need to be dropped, and
        // the pointer is always valid as it will never point to an allocation
        // nor will it be null. The pointer only lives long enough to perform
        // the write operation and is not exposed from this point. This can also
        // cause data races if called from different threads, but EntropyBuffer
        // is not Sync, so this won't happen.
        unsafe {
            self.buffer.get().write(buffer);
        }
    }

    #[inline]
    fn update_cursor(&self, val: usize) {
        // SAFETY: Data is writable and does not need to be dropped, and
        // the pointer is always valid as it will never point to an allocation
        // nor will it be null. The pointer only lives long enough to perform
        // the write operation and is not exposed from this point. This can also
        // cause data races if called from different threads, but EntropyBuffer
        // is not Sync, so this won't happen. There are no references of the
        // underlying value ever, only returned/copied values, so this is always
        // safe to do.
        unsafe {
            self.cursor.get().write(val);
        }
    }

    /// Checks if the stored entropy has been exhausted, by
    /// seeing if the cursor is the same value as the total
    /// number of bytes available in the buffer.
    #[inline]
    fn is_empty(&self) -> bool {
        Self::total_bytes() == self.get_cursor()
    }

    /// Returns the remaining amount of entropy left in the
    /// buffer, by subtracting the total amount of bytes in
    /// the buffer by the value of the cursor. A zero value
    /// indicates an empty buffer.
    #[inline]
    fn remaining_buffer(&self) -> usize {
        Self::total_bytes() - self.get_cursor()
    }

    /// Updates the [`EntropyBuffer`] with a new buffer state, and
    /// reset the cursor to 0.
    ///
    /// **WARNING**: Must not be used while a reference to buffer is
    /// alive, else this operation will be unsound.
    #[inline]
    fn update_entropy(&self, buffer: [u64; SIZE]) {
        self.update_buffer(buffer);
        self.update_cursor(0);
    }

    /// Fills the incoming mutable byte slice with the available
    /// stored entropy in the internal buffer. Returns the filled
    /// length, which can either be the entire length of the mutable
    /// slice, or the amount filled by the remaining buffer.
    #[inline]
    fn fill(&self, output: &mut [u8]) -> usize {
        let length = output.len().min(self.remaining_buffer());
        let cursor = self.get_cursor();
        let to = cursor + length;
        let buffer = bytemuck::cast_slice(self.get_buffer());

        output[..length].copy_from_slice(&buffer[cursor..to]);

        self.update_cursor(to);

        length
    }

    /// Resets the internal buffer and cursor state, clearing any entropy
    /// stored.
    #[inline]
    pub(crate) fn empty_buffer(&self) {
        self.update_buffer([0; SIZE]);
        self.update_cursor(Self::total_bytes());
    }

    /// Fills the incoming mutable byte source with available entropy, consuming
    /// the entropy stored in the buffer until it is exhausted and then pulling in
    /// more entropy when required to refill the buffer and finish filling the input
    /// byte slice.
    #[inline]
    pub(crate) fn fill_bytes_with_source<B: AsMut<[u8]>, S: Fn() -> [u64; SIZE]>(
        &self,
        mut output: B,
        source: S,
    ) {
        let mut output = output.as_mut();
        let mut remaining = output.len();

        while remaining > 0 {
            if self.is_empty() {
                self.update_entropy(source());
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

impl<const SIZE: usize> Clone for EntropyBuffer<SIZE> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            buffer: UnsafeCell::new(*self.get_buffer()),
            cursor: UnsafeCell::new(self.get_cursor()),
        }
    }
}

impl<const SIZE: usize> PartialEq for EntropyBuffer<SIZE> {
    fn eq(&self, other: &Self) -> bool {
        self.get_buffer() == other.get_buffer() && self.get_cursor() == other.get_cursor()
    }
}

impl<const SIZE: usize> Eq for EntropyBuffer<SIZE> {}

#[cfg(feature = "serialize")]
impl<const SIZE: usize> Serialize for EntropyBuffer<SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = serializer.serialize_tuple(SIZE + 1)?;

        // Insert the buffer as tuple elements
        for val in self.get_buffer().iter() {
            buf.serialize_element(val)?;
        }

        // Add the cursor as the last element of the tuple
        buf.serialize_element(&self.get_cursor())?;

        buf.end()
    }
}

#[cfg(feature = "serialize")]
impl<'de, const SIZE: usize> Deserialize<'de> for EntropyBuffer<SIZE> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EntropyVisitor<const LENGTH: usize>;

        impl<'de, const LENGTH: usize> Visitor<'de> for EntropyVisitor<LENGTH> {
            type Value = EntropyBuffer<LENGTH>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "struct EntropyBuffer<{LENGTH}>")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut buf = [0; LENGTH];
                let mut len: usize = 0;

                for slot in buf.iter_mut() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(len, &self))?;
                    len += 1;
                }

                let cursor = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(len, &self))?;

                Ok(EntropyBuffer::from_serde(buf, cursor))
            }
        }

        deserializer.deserialize_tuple(SIZE + 1, EntropyVisitor::<SIZE>)
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
        let buffer = EntropyBuffer::<1>::new();

        buffer.update_entropy([(2 << 32) | 1]);

        assert!(!buffer.is_empty());

        let mut output = [0u8; 4];

        let filled = buffer.fill(&mut output);

        assert_eq!(&output, &[1, 0, 0, 0]);
        assert_eq!(&filled, &4);
        assert_eq!(&buffer.get_cursor(), &4);
        assert!(!buffer.is_empty());

        let mut output = [0u8; 6];

        let filled = buffer.fill(&mut output);

        assert_eq!(&output, &[2, 0, 0, 0, 0, 0]);
        assert_eq!(&filled, &4);
        assert_eq!(&buffer.get_cursor(), &8);
        assert!(buffer.is_empty());

        buffer.update_entropy([(2 << 32) | 1]);

        assert!(!buffer.is_empty());

        let filled = buffer.fill(&mut output[filled..]);

        assert_eq!(&output, &[2, 0, 0, 0, 1, 0]);
        assert_eq!(&filled, &2);
        assert_eq!(&buffer.get_cursor(), &2);
    }

    #[test]
    fn clone_buffer() {
        let buffer = EntropyBuffer::<1>::new();

        buffer.update_entropy([(2 << 32) | 1]);

        let mut output = [0u8; 4];

        // Modify the buffer to have a new state.
        buffer.fill(&mut output);

        // Clone the buffer
        let cloned = buffer.clone();

        // Check if the cloned buffer has the same state as the original
        assert_eq!(&buffer, &cloned);
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn serde_tokens() {
        use serde_test::{assert_tokens, Token};

        let buffer = EntropyBuffer::<8>::new();

        assert_tokens(
            &buffer,
            &[
                Token::Tuple { len: 9 },
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(0),
                Token::U64(64),
                Token::TupleEnd,
            ],
        );

        buffer.update_entropy([1, 2, 3, 4, 5, 6, 7, 8]);

        assert_tokens(
            &buffer,
            &[
                Token::Tuple { len: 9 },
                Token::U64(1),
                Token::U64(2),
                Token::U64(3),
                Token::U64(4),
                Token::U64(5),
                Token::U64(6),
                Token::U64(7),
                Token::U64(8),
                Token::U64(0),
                Token::TupleEnd,
            ],
        );
    }
}
