use std::io::Write;

use flate2::Compression as FlateCompression;
use flate2::write::ZlibEncoder;
use flate2::{Decompress, FlushDecompress, Status};

use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::frame::{FrameLimits, encode_frame};
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub const MAX_INFLATED_PACKET_LENGTH: usize = 8_388_608;

/// Connection-local compression state installed after the negotiation frame is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMode {
    Disabled,
    Enabled(CompressionThreshold),
}

/// A validated nonnegative compression threshold representable by a protocol VarInt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompressionThreshold(usize);

impl CompressionThreshold {
    pub fn new(threshold: usize) -> Result<Self, WireError> {
        if threshold > i32::MAX as usize {
            Err(WireError::CompressionThresholdOutOfRange { threshold })
        } else {
            Ok(Self(threshold))
        }
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl CompressionMode {
    pub fn enabled(threshold: usize) -> Result<Self, WireError> {
        CompressionThreshold::new(threshold).map(Self::Enabled)
    }

    #[must_use]
    pub const fn threshold(self) -> Option<usize> {
        match self {
            Self::Disabled => None,
            Self::Enabled(threshold) => Some(threshold.get()),
        }
    }
}

pub fn encode_packet(
    packet_body: &[u8],
    mode: CompressionMode,
    frame_limits: FrameLimits,
) -> Result<Vec<u8>, WireError> {
    if packet_body.is_empty() {
        return Err(WireError::EmptyPacketBody);
    }
    match mode {
        CompressionMode::Disabled => encode_frame(packet_body, frame_limits),
        CompressionMode::Enabled(threshold) => {
            encode_compression_frame(packet_body, threshold.get(), frame_limits)
        }
    }
}

pub fn decode_packet(frame_body: &[u8], mode: CompressionMode) -> Result<Vec<u8>, WireError> {
    match mode {
        CompressionMode::Disabled => {
            if frame_body.is_empty() {
                Err(WireError::EmptyPacketBody)
            } else {
                Ok(frame_body.to_vec())
            }
        }
        CompressionMode::Enabled(threshold) => {
            decode_compression_frame(frame_body, threshold.get())
        }
    }
}

fn encode_compression_frame(
    packet_body: &[u8],
    threshold: usize,
    frame_limits: FrameLimits,
) -> Result<Vec<u8>, WireError> {
    if packet_body.len() > MAX_INFLATED_PACKET_LENGTH {
        return Err(WireError::InflatedPacketTooLarge {
            length: packet_body.len(),
            maximum: MAX_INFLATED_PACKET_LENGTH,
        });
    }

    let mut envelope = WireWriter::new(frame_limits.maximum_frame_length());
    if packet_body.len() < threshold {
        envelope.write_var_i32(0)?;
        envelope.write_bytes(packet_body)?;
    } else {
        let declared =
            i32::try_from(packet_body.len()).map_err(|_| WireError::InflatedPacketTooLarge {
                length: packet_body.len(),
                maximum: MAX_INFLATED_PACKET_LENGTH,
            })?;
        envelope.write_var_i32(declared)?;
        let compressed = compress(packet_body)?;
        envelope.write_bytes(&compressed)?;
    }
    encode_frame(envelope.as_slice(), frame_limits)
}

fn decode_compression_frame(frame_body: &[u8], threshold: usize) -> Result<Vec<u8>, WireError> {
    let mut reader = WireReader::new(frame_body);
    let declared = reader.read_var_i32()?;
    if declared == 0 {
        let packet = reader.take_remaining();
        return if packet.is_empty() {
            Err(WireError::EmptyPacketBody)
        } else {
            Ok(packet.to_vec())
        };
    }
    let declared = usize::try_from(declared).map_err(|_| WireError::NegativeLength {
        field: "inflated packet",
        value: declared,
    })?;
    if declared > MAX_INFLATED_PACKET_LENGTH {
        return Err(WireError::InflatedPacketTooLarge {
            length: declared,
            maximum: MAX_INFLATED_PACKET_LENGTH,
        });
    }
    if declared < threshold {
        return Err(WireError::CompressedBelowThreshold {
            length: declared,
            threshold,
        });
    }
    let compressed = reader.take_remaining();
    if compressed.is_empty() {
        return Err(WireError::EmptyCompressedPayload);
    }
    inflate_exact(compressed, declared)
}

fn compress(input: &[u8]) -> Result<Vec<u8>, WireError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), FlateCompression::default());
    encoder
        .write_all(input)
        .map_err(|_| WireError::InvalidCompressedData)?;
    encoder
        .finish()
        .map_err(|_| WireError::InvalidCompressedData)
}

fn inflate_exact(input: &[u8], declared: usize) -> Result<Vec<u8>, WireError> {
    let mut decoder = Decompress::new(true);
    let mut output = vec![0; declared.saturating_add(1)];
    let status = decoder
        .decompress(input, &mut output, FlushDecompress::Finish)
        .map_err(|_| WireError::InvalidCompressedData)?;
    let actual = usize::try_from(decoder.total_out()).unwrap_or(usize::MAX);
    if actual != declared {
        return Err(WireError::InflatedLengthMismatch { declared, actual });
    }
    let consumed = decoder.total_in();
    if status != Status::StreamEnd || consumed != input.len() as u64 {
        return Err(WireError::InvalidCompressedData);
    }
    output.truncate(declared);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::java_26_2::wire::compression::{
        CompressionMode, MAX_INFLATED_PACKET_LENGTH, decode_packet, encode_packet,
    };
    use crate::java_26_2::wire::error::WireError;
    use crate::java_26_2::wire::frame::{FrameDecoder, FrameLimits, encode_frame};
    use crate::java_26_2::wire::primitive::WireWriter;

    fn unwrap_frame(encoded: &[u8]) -> Vec<u8> {
        let mut decoder = FrameDecoder::new(FrameLimits::default());
        decoder.push(encoded).unwrap();
        decoder.next_frame().unwrap().unwrap()
    }

    #[test]
    fn emits_raw_below_threshold_and_zlib_at_threshold() {
        let mode = CompressionMode::enabled(4).unwrap();
        let raw = encode_packet(&[1, 2, 3], mode, FrameLimits::default()).unwrap();
        assert_eq!(unwrap_frame(&raw), vec![0, 1, 2, 3]);
        let compressed = encode_packet(&[1, 2, 3, 4], mode, FrameLimits::default()).unwrap();
        let envelope = unwrap_frame(&compressed);
        assert_eq!(envelope[0], 4);
        assert_eq!(decode_packet(&envelope, mode).unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn threshold_zero_compresses_every_nonempty_body() {
        let mode = CompressionMode::enabled(0).unwrap();
        let encoded = encode_packet(&[0], mode, FrameLimits::default()).unwrap();
        let envelope = unwrap_frame(&encoded);
        assert_eq!(envelope[0], 1);
        assert_eq!(decode_packet(&envelope, mode).unwrap(), vec![0]);
    }

    #[test]
    fn matches_the_locked_256_byte_threshold_boundary() {
        let mode = CompressionMode::enabled(256).unwrap();
        let below = encode_packet(&vec![7; 255], mode, FrameLimits::default()).unwrap();
        assert_eq!(unwrap_frame(&below)[0], 0);

        let boundary = encode_packet(&vec![7; 256], mode, FrameLimits::default()).unwrap();
        assert_eq!(&unwrap_frame(&boundary)[..2], &[0x80, 0x02]);
    }

    #[test]
    fn accepts_raw_form_even_when_it_meets_threshold() {
        let mode = CompressionMode::enabled(2).unwrap();
        assert_eq!(decode_packet(&[0, 1, 2], mode).unwrap(), vec![1, 2]);
    }

    #[test]
    fn rejects_nonzero_declarations_outside_locked_bounds() {
        let mode = CompressionMode::enabled(4).unwrap();
        assert_eq!(
            decode_packet(&[3, 1], mode),
            Err(WireError::CompressedBelowThreshold {
                length: 3,
                threshold: 4,
            })
        );

        let mut writer = WireWriter::new(16);
        writer
            .write_var_i32((MAX_INFLATED_PACKET_LENGTH + 1) as i32)
            .unwrap();
        writer.write_u8(1).unwrap();
        assert!(matches!(
            decode_packet(writer.as_slice(), mode),
            Err(WireError::InflatedPacketTooLarge { .. })
        ));
    }

    #[test]
    fn rejects_corruption_declared_mismatch_and_trailing_stream_data() {
        let mode = CompressionMode::enabled(1).unwrap();
        assert_eq!(
            decode_packet(&[1, 0, 1, 2], mode),
            Err(WireError::InvalidCompressedData)
        );

        let valid = encode_packet(&[7, 8], mode, FrameLimits::default()).unwrap();
        let envelope = unwrap_frame(&valid);
        let mut wrong_length = envelope.clone();
        wrong_length[0] = 3;
        assert!(matches!(
            decode_packet(&wrong_length, mode),
            Err(WireError::InflatedLengthMismatch { .. })
        ));

        let mut trailing = envelope;
        trailing.push(0);
        assert_eq!(
            decode_packet(&trailing, mode),
            Err(WireError::InvalidCompressedData)
        );

        let mut truncated =
            unwrap_frame(&encode_packet(&[7, 8, 9], mode, FrameLimits::default()).unwrap());
        truncated.pop();
        assert_eq!(
            decode_packet(&truncated, mode),
            Err(WireError::InvalidCompressedData)
        );
    }

    #[test]
    fn outer_frame_limit_still_applies_to_raw_compression_envelopes() {
        let limits = FrameLimits::new(4, 7).unwrap();
        let error =
            encode_packet(&[1, 2, 3, 4], CompressionMode::enabled(5).unwrap(), limits).unwrap_err();
        assert!(matches!(error, WireError::OutputLimit { .. }));
    }

    #[test]
    fn disabled_mode_is_exactly_the_ordinary_frame_grammar() {
        let encoded =
            encode_packet(&[0, 1], CompressionMode::Disabled, FrameLimits::default()).unwrap();
        assert_eq!(
            encoded,
            encode_frame(&[0, 1], FrameLimits::default()).unwrap()
        );
        assert_eq!(
            decode_packet(&[0, 1], CompressionMode::Disabled).unwrap(),
            vec![0, 1]
        );
    }
}
