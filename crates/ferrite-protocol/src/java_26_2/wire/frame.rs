use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::varint::encode_i32;

pub const MAX_FRAME_LENGTH: usize = 2_097_151;
pub const MAX_FRAME_PREFIX_BYTES: usize = 3;

/// Limits for one incremental TCP frame decoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameLimits {
    maximum_frame_length: usize,
    maximum_buffered_bytes: usize,
}

impl FrameLimits {
    pub fn new(
        maximum_frame_length: usize,
        maximum_buffered_bytes: usize,
    ) -> Result<Self, WireError> {
        if !(1..=MAX_FRAME_LENGTH).contains(&maximum_frame_length) {
            return Err(WireError::FrameLengthOutOfRange {
                length: maximum_frame_length,
                maximum: MAX_FRAME_LENGTH,
            });
        }
        let minimum_buffer = maximum_frame_length + MAX_FRAME_PREFIX_BYTES;
        if maximum_buffered_bytes < minimum_buffer {
            return Err(WireError::BufferLimit {
                attempted: minimum_buffer,
                maximum: maximum_buffered_bytes,
            });
        }
        Ok(Self {
            maximum_frame_length,
            maximum_buffered_bytes,
        })
    }

    #[must_use]
    pub const fn maximum_frame_length(self) -> usize {
        self.maximum_frame_length
    }

    #[must_use]
    pub const fn maximum_buffered_bytes(self) -> usize {
        self.maximum_buffered_bytes
    }
}

impl Default for FrameLimits {
    fn default() -> Self {
        Self {
            maximum_frame_length: MAX_FRAME_LENGTH,
            maximum_buffered_bytes: (MAX_FRAME_LENGTH + MAX_FRAME_PREFIX_BYTES) * 2,
        }
    }
}

/// Incrementally extracts bounded Minecraft frames from arbitrary TCP chunks.
#[derive(Debug, Clone)]
pub struct FrameDecoder {
    limits: FrameLimits,
    buffer: Vec<u8>,
    start: usize,
    faulted: bool,
}

impl FrameDecoder {
    #[must_use]
    pub fn new(limits: FrameLimits) -> Self {
        Self {
            limits,
            buffer: Vec::new(),
            start: 0,
            faulted: false,
        }
    }

    #[must_use]
    pub const fn limits(&self) -> FrameLimits {
        self.limits
    }

    #[must_use]
    pub const fn is_faulted(&self) -> bool {
        self.faulted
    }

    #[must_use]
    pub fn buffered_bytes(&self) -> usize {
        self.buffer.len() - self.start
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<(), WireError> {
        self.require_live()?;
        let attempted = self.buffered_bytes().saturating_add(bytes.len());
        if attempted > self.limits.maximum_buffered_bytes {
            return self.fail(WireError::BufferLimit {
                attempted,
                maximum: self.limits.maximum_buffered_bytes,
            });
        }
        self.compact_if_needed(bytes.len());
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>, WireError> {
        self.require_live()?;
        let available = &self.buffer[self.start..];
        let decoded = match decode_frame_length(available) {
            Ok(decoded) => decoded,
            Err(error) => return self.fail(error),
        };
        let Some((length, prefix_length)) = decoded else {
            return Ok(None);
        };
        if length > self.limits.maximum_frame_length {
            return self.fail(WireError::FrameLengthOutOfRange {
                length,
                maximum: self.limits.maximum_frame_length,
            });
        }
        let complete_length = prefix_length + length;
        if available.len() < complete_length {
            return Ok(None);
        }
        let body_start = self.start + prefix_length;
        let body_end = self.start + complete_length;
        let frame = self.buffer[body_start..body_end].to_vec();
        self.start = body_end;
        if self.start == self.buffer.len() {
            self.buffer.clear();
            self.start = 0;
        }
        Ok(Some(frame))
    }

    pub fn finish(&mut self) -> Result<(), WireError> {
        self.require_live()?;
        let buffered = self.buffered_bytes();
        if buffered == 0 {
            Ok(())
        } else {
            self.fail(WireError::IncompleteFrame { buffered })
        }
    }

    pub(crate) fn mark_faulted(&mut self) {
        self.faulted = true;
    }

    fn require_live(&self) -> Result<(), WireError> {
        if self.faulted {
            Err(WireError::DecoderFaulted)
        } else {
            Ok(())
        }
    }

    fn fail<T>(&mut self, error: WireError) -> Result<T, WireError> {
        self.faulted = true;
        Err(error)
    }

    fn compact_if_needed(&mut self, incoming: usize) {
        if self.start > 0
            && (self.start >= self.buffer.len() / 2
                || self.buffer.len() + incoming > self.buffer.capacity())
        {
            self.buffer.drain(..self.start);
            self.start = 0;
        }
    }
}

pub fn encode_frame(body: &[u8], limits: FrameLimits) -> Result<Vec<u8>, WireError> {
    if body.is_empty() || body.len() > limits.maximum_frame_length {
        return Err(WireError::FrameLengthOutOfRange {
            length: body.len(),
            maximum: limits.maximum_frame_length,
        });
    }
    let length = i32::try_from(body.len()).map_err(|_| WireError::FrameLengthOutOfRange {
        length: body.len(),
        maximum: limits.maximum_frame_length,
    })?;
    let prefix = encode_i32(length);
    let mut encoded = Vec::with_capacity(prefix.as_slice().len() + body.len());
    encoded.extend_from_slice(prefix.as_slice());
    encoded.extend_from_slice(body);
    Ok(encoded)
}

fn decode_frame_length(input: &[u8]) -> Result<Option<(usize, usize)>, WireError> {
    let mut value = 0usize;
    for index in 0..MAX_FRAME_PREFIX_BYTES {
        let Some(byte) = input.get(index).copied() else {
            return Ok(None);
        };
        value |= usize::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            if value == 0 {
                return Err(WireError::FrameLengthOutOfRange {
                    length: 0,
                    maximum: MAX_FRAME_LENGTH,
                });
            }
            return Ok(Some((value, index + 1)));
        }
    }
    Err(WireError::InvalidFramePrefix)
}

#[cfg(test)]
mod tests {
    use crate::java_26_2::wire::error::WireError;
    use crate::java_26_2::wire::frame::{
        FrameDecoder, FrameLimits, MAX_FRAME_LENGTH, encode_frame,
    };

    #[test]
    fn decodes_every_fragmentation_of_a_frame() {
        let encoded = encode_frame(&[0, 1, 2, 3, 4], FrameLimits::default()).unwrap();
        for split in 0..=encoded.len() {
            let mut decoder = FrameDecoder::new(FrameLimits::default());
            decoder.push(&encoded[..split]).unwrap();
            if split < encoded.len() {
                assert_eq!(decoder.next_frame().unwrap(), None);
            }
            decoder.push(&encoded[split..]).unwrap();
            assert_eq!(decoder.next_frame().unwrap(), Some(vec![0, 1, 2, 3, 4]));
            decoder.finish().unwrap();
        }
    }

    #[test]
    fn preserves_multiple_frames_and_compacts_consumed_storage() {
        let first = encode_frame(&[1], FrameLimits::default()).unwrap();
        let second = encode_frame(&[2, 3], FrameLimits::default()).unwrap();
        let mut decoder = FrameDecoder::new(FrameLimits::default());
        decoder.push(&[first, second].concat()).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), Some(vec![1]));
        assert_eq!(decoder.next_frame().unwrap(), Some(vec![2, 3]));
        assert_eq!(decoder.next_frame().unwrap(), None);
        assert_eq!(decoder.buffered_bytes(), 0);
    }

    #[test]
    fn rejects_zero_overlong_and_configured_oversize_frames_terminally() {
        for prefix in [&[0][..], &[0x80, 0x80, 0x80][..], &[4, 1, 2, 3, 4][..]] {
            let limits = FrameLimits::new(3, 6).unwrap();
            let mut decoder = FrameDecoder::new(limits);
            decoder.push(prefix).unwrap();
            assert!(decoder.next_frame().is_err());
            assert!(decoder.is_faulted());
            assert_eq!(decoder.push(&[1]), Err(WireError::DecoderFaulted));
        }
    }

    #[test]
    fn enforces_receive_budget_before_mutating() {
        let limits = FrameLimits::new(1, 4).unwrap();
        let mut decoder = FrameDecoder::new(limits);
        let error = decoder.push(&[1, 0, 1, 0, 1]).unwrap_err();
        assert!(matches!(error, WireError::BufferLimit { .. }));
        assert_eq!(decoder.buffered_bytes(), 0);
        assert!(decoder.is_faulted());
    }

    #[test]
    fn supports_the_locked_maximum_frame_length() {
        let body = vec![0; MAX_FRAME_LENGTH];
        let encoded = encode_frame(&body, FrameLimits::default()).unwrap();
        assert_eq!(&encoded[..3], &[0xff, 0xff, 0x7f]);
    }

    #[test]
    fn consumes_independent_c0_golden_and_nonminimal_frames() {
        const INTENTION: &[u8] = &[
            0x10, 0x00, 0x88, 0x06, 0x09, b'l', b'o', b'c', b'a', b'l', b'h', b'o', b's', b't',
            0x63, 0xdd, 0x01,
        ];
        let mut decoder = FrameDecoder::new(FrameLimits::default());
        decoder.push(INTENTION).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), Some(INTENTION[1..].to_vec()));

        decoder.push(&[0x81, 0x00, 0x00]).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), Some(vec![0]));
    }

    #[test]
    fn eof_with_a_partial_frame_terminally_faults() {
        let mut decoder = FrameDecoder::new(FrameLimits::default());
        decoder.push(&[9, 1, 1]).unwrap();
        assert_eq!(
            decoder.finish(),
            Err(WireError::IncompleteFrame { buffered: 3 })
        );
        assert_eq!(decoder.next_frame(), Err(WireError::DecoderFaulted));
    }
}
