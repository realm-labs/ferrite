use crate::java_26_2::wire::compression::{CompressionMode, decode_packet, encode_packet};
use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::frame::{FrameDecoder, FrameLimits};

/// A bounded, terminal-on-fault TCP-to-packet-body decoder.
#[derive(Debug, Clone)]
pub struct PacketStreamDecoder {
    frames: FrameDecoder,
    compression: CompressionMode,
}

impl PacketStreamDecoder {
    #[must_use]
    pub fn new(limits: FrameLimits, compression: CompressionMode) -> Self {
        Self {
            frames: FrameDecoder::new(limits),
            compression,
        }
    }

    #[must_use]
    pub const fn compression(&self) -> CompressionMode {
        self.compression
    }

    #[must_use]
    pub const fn is_faulted(&self) -> bool {
        self.frames.is_faulted()
    }

    #[must_use]
    pub fn buffered_bytes(&self) -> usize {
        self.frames.buffered_bytes()
    }

    pub fn set_compression(&mut self, compression: CompressionMode) -> Result<(), WireError> {
        if self.frames.is_faulted() {
            Err(WireError::DecoderFaulted)
        } else {
            self.compression = compression;
            Ok(())
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<(), WireError> {
        self.frames.push(bytes)
    }

    pub fn next_packet(&mut self) -> Result<Option<Vec<u8>>, WireError> {
        let Some(frame) = self.frames.next_frame()? else {
            return Ok(None);
        };
        match decode_packet(&frame, self.compression) {
            Ok(packet) => Ok(Some(packet)),
            Err(error) => {
                self.frames.mark_faulted();
                Err(error)
            }
        }
    }

    pub fn finish(&mut self) -> Result<(), WireError> {
        self.frames.finish()
    }
}

/// Encodes packet bodies with the connection's current framing and compression state.
#[derive(Debug, Clone, Copy)]
pub struct PacketStreamEncoder {
    limits: FrameLimits,
    compression: CompressionMode,
}

impl PacketStreamEncoder {
    #[must_use]
    pub const fn new(limits: FrameLimits, compression: CompressionMode) -> Self {
        Self {
            limits,
            compression,
        }
    }

    pub fn set_compression(&mut self, compression: CompressionMode) {
        self.compression = compression;
    }

    pub fn encode(&self, packet_body: &[u8]) -> Result<Vec<u8>, WireError> {
        encode_packet(packet_body, self.compression, self.limits)
    }
}

#[cfg(test)]
mod tests {
    use crate::java_26_2::wire::compression::CompressionMode;
    use crate::java_26_2::wire::error::WireError;
    use crate::java_26_2::wire::frame::FrameLimits;
    use crate::java_26_2::wire::stream::{PacketStreamDecoder, PacketStreamEncoder};

    #[test]
    fn switches_compression_between_complete_frames() {
        let limits = FrameLimits::default();
        let compressed = CompressionMode::enabled(0).unwrap();
        let mut encoder = PacketStreamEncoder::new(limits, CompressionMode::Disabled);
        let first = encoder.encode(&[0, 1]).unwrap();
        encoder.set_compression(compressed);
        let second = encoder.encode(&[2, 3]).unwrap();

        let mut decoder = PacketStreamDecoder::new(limits, CompressionMode::Disabled);
        decoder.push(&[first, second].concat()).unwrap();
        assert_eq!(decoder.next_packet().unwrap(), Some(vec![0, 1]));
        decoder.set_compression(compressed).unwrap();
        assert_eq!(decoder.next_packet().unwrap(), Some(vec![2, 3]));
    }

    #[test]
    fn compression_failure_terminally_faults_stream() {
        let mut decoder =
            PacketStreamDecoder::new(FrameLimits::default(), CompressionMode::enabled(1).unwrap());
        decoder.push(&[2, 1, 0]).unwrap();
        assert!(decoder.next_packet().is_err());
        assert!(decoder.is_faulted());
        assert_eq!(decoder.next_packet(), Err(WireError::DecoderFaulted));
    }

    #[test]
    fn consumes_independent_c1_login_acknowledgement_goldens() {
        let limits = FrameLimits::default();
        let mut raw = PacketStreamDecoder::new(limits, CompressionMode::Disabled);
        raw.push(&[1, 3]).unwrap();
        assert_eq!(raw.next_packet().unwrap(), Some(vec![3]));

        let mut compressed =
            PacketStreamDecoder::new(limits, CompressionMode::enabled(256).unwrap());
        compressed.push(&[2, 0, 3]).unwrap();
        assert_eq!(compressed.next_packet().unwrap(), Some(vec![3]));
    }
}
