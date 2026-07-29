use thiserror::Error;

/// The connection-level action required after malformed inbound wire data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MalformedInputPolicy {
    /// Stop emitting packets and close the connection without attempting to resynchronize.
    MakeReadOnlyThenClose,
}

pub const MALFORMED_INPUT_POLICY: MalformedInputPolicy =
    MalformedInputPolicy::MakeReadOnlyThenClose;

/// The source boundary at which a terminal wire failure was detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultClass {
    Framing,
    Primitive,
    Compression,
    Buffer,
}

/// A terminal Minecraft wire-codec failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WireError {
    #[error("the decoder is terminally faulted")]
    DecoderFaulted,
    #[error("buffer would contain {attempted} bytes, above its {maximum}-byte limit")]
    BufferLimit { attempted: usize, maximum: usize },
    #[error("frame length prefix exceeds the three-byte VarInt21 boundary")]
    InvalidFramePrefix,
    #[error("frame length {length} is outside 1..={maximum}")]
    FrameLengthOutOfRange { length: usize, maximum: usize },
    #[error("stream ended with {buffered} bytes of an incomplete frame")]
    IncompleteFrame { buffered: usize },
    #[error("input ended while reading {field}: needed {needed} bytes, found {remaining}")]
    UnexpectedEnd {
        field: &'static str,
        needed: usize,
        remaining: usize,
    },
    #[error("{kind} exceeds its {maximum_bytes}-byte encoded width")]
    VariableIntegerTooLong {
        kind: &'static str,
        maximum_bytes: usize,
    },
    #[error("{field} length is negative: {value}")]
    NegativeLength { field: &'static str, value: i32 },
    #[error("{field} length {length} exceeds its limit {maximum}")]
    LengthLimit {
        field: &'static str,
        length: usize,
        maximum: usize,
    },
    #[error("UTF value contains {actual} UTF-16 code units, above its {maximum}-unit limit")]
    UtfCodeUnitLimit { actual: usize, maximum: usize },
    #[error("writer output would contain {attempted} bytes, above its {maximum}-byte limit")]
    OutputLimit { attempted: usize, maximum: usize },
    #[error("compression threshold {threshold} is outside the signed VarInt range")]
    CompressionThresholdOutOfRange { threshold: usize },
    #[error("uncompressed packet body is empty")]
    EmptyPacketBody,
    #[error("inflated packet length {length} exceeds the {maximum}-byte limit")]
    InflatedPacketTooLarge { length: usize, maximum: usize },
    #[error("inflated packet length {length} is below compression threshold {threshold}")]
    CompressedBelowThreshold { length: usize, threshold: usize },
    #[error("compressed packet contains no zlib stream")]
    EmptyCompressedPayload,
    #[error("compressed packet is not one complete zlib stream")]
    InvalidCompressedData,
    #[error("zlib stream inflated to {actual} bytes, but declared {declared}")]
    InflatedLengthMismatch { declared: usize, actual: usize },
}

impl WireError {
    #[must_use]
    pub const fn class(&self) -> FaultClass {
        match self {
            Self::BufferLimit { .. } | Self::OutputLimit { .. } => FaultClass::Buffer,
            Self::InvalidFramePrefix
            | Self::FrameLengthOutOfRange { .. }
            | Self::IncompleteFrame { .. }
            | Self::DecoderFaulted => FaultClass::Framing,
            Self::UnexpectedEnd { .. }
            | Self::VariableIntegerTooLong { .. }
            | Self::NegativeLength { .. }
            | Self::LengthLimit { .. }
            | Self::UtfCodeUnitLimit { .. } => FaultClass::Primitive,
            Self::CompressionThresholdOutOfRange { .. }
            | Self::EmptyPacketBody
            | Self::InflatedPacketTooLarge { .. }
            | Self::CompressedBelowThreshold { .. }
            | Self::EmptyCompressedPayload
            | Self::InvalidCompressedData
            | Self::InflatedLengthMismatch { .. } => FaultClass::Compression,
        }
    }
}
