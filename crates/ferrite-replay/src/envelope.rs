//! Topology-independent command and event envelopes.

use crate::codec::{CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError, Encoder};
use ferrite_foundation::identity::StableEntityId;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use thiserror::Error;

const COMMAND_MAGIC: &[u8; 4] = b"FRCM";
const EVENT_MAGIC: &[u8; 4] = b"FREV";
const ENVELOPE_SCHEMA_V1: u16 = 1;
pub const MAX_ENVELOPE_PAYLOAD_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TickNumber(u64);

impl TickNumber {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl CanonicalEncode for TickNumber {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u64(self.0);
        Ok(())
    }
}

impl CanonicalDecode for TickNumber {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self::new(decoder.read_u64()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl CanonicalEncode for SequenceNumber {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_u64(self.0);
        Ok(())
    }
}

impl CanonicalDecode for SequenceNumber {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self::new(decoder.read_u64()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopePayload(Vec<u8>);

impl EnvelopePayload {
    pub fn new(bytes: Vec<u8>) -> Result<Self, EnvelopeError> {
        if bytes.len() > MAX_ENVELOPE_PAYLOAD_BYTES {
            return Err(EnvelopeError::PayloadTooLarge {
                actual: bytes.len(),
                maximum: MAX_ENVELOPE_PAYLOAD_BYTES,
            });
        }
        Ok(Self(bytes))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl CanonicalEncode for EnvelopePayload {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_bytes(&self.0, MAX_ENVELOPE_PAYLOAD_BYTES)
    }
}

impl CanonicalDecode for EnvelopePayload {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self(
            decoder.read_bytes(MAX_ENVELOPE_PAYLOAD_BYTES)?.to_vec(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandSource {
    System,
    Player(StableEntityId),
    Region(SimulationRegionKey),
}

impl CanonicalEncode for CommandSource {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        match self {
            Self::System => encoder.write_u8(0),
            Self::Player(player) => {
                encoder.write_u8(1);
                player.encode(encoder)?;
            }
            Self::Region(region) => {
                encoder.write_u8(2);
                region.encode(encoder)?;
            }
        }
        Ok(())
    }
}

impl CanonicalDecode for CommandSource {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        match decoder.read_u8()? {
            0 => Ok(Self::System),
            1 => Ok(Self::Player(StableEntityId::decode(decoder)?)),
            2 => Ok(Self::Region(SimulationRegionKey::decode(decoder)?)),
            tag => Err(DecodeError::InvalidEnumTag {
                kind: "command source",
                tag: u64::from(tag),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEnvelope {
    tick: TickNumber,
    sequence: SequenceNumber,
    source: CommandSource,
    target: SimulationRegionKey,
    kind: ResourceId,
    payload: EnvelopePayload,
}

impl CommandEnvelope {
    pub const fn new(
        tick: TickNumber,
        sequence: SequenceNumber,
        source: CommandSource,
        target: SimulationRegionKey,
        kind: ResourceId,
        payload: EnvelopePayload,
    ) -> Self {
        Self {
            tick,
            sequence,
            source,
            target,
            kind,
            payload,
        }
    }

    pub const fn tick(&self) -> TickNumber {
        self.tick
    }

    pub const fn sequence(&self) -> SequenceNumber {
        self.sequence
    }

    pub const fn source(&self) -> &CommandSource {
        &self.source
    }

    pub const fn target(&self) -> &SimulationRegionKey {
        &self.target
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub const fn payload(&self) -> &EnvelopePayload {
        &self.payload
    }
}

impl CanonicalEncode for CommandEnvelope {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_magic(COMMAND_MAGIC);
        encoder.write_u16(ENVELOPE_SCHEMA_V1);
        self.tick.encode(encoder)?;
        self.sequence.encode(encoder)?;
        self.source.encode(encoder)?;
        self.target.encode(encoder)?;
        self.kind.encode(encoder)?;
        self.payload.encode(encoder)
    }
}

impl CanonicalDecode for CommandEnvelope {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        decoder.expect_magic(COMMAND_MAGIC)?;
        expect_envelope_schema(decoder)?;
        Ok(Self::new(
            TickNumber::decode(decoder)?,
            SequenceNumber::decode(decoder)?,
            CommandSource::decode(decoder)?,
            SimulationRegionKey::decode(decoder)?,
            ResourceId::decode(decoder)?,
            EnvelopePayload::decode(decoder)?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventEnvelope {
    tick: TickNumber,
    sequence: SequenceNumber,
    source: SimulationRegionKey,
    kind: ResourceId,
    payload: EnvelopePayload,
}

impl EventEnvelope {
    pub const fn new(
        tick: TickNumber,
        sequence: SequenceNumber,
        source: SimulationRegionKey,
        kind: ResourceId,
        payload: EnvelopePayload,
    ) -> Self {
        Self {
            tick,
            sequence,
            source,
            kind,
            payload,
        }
    }

    pub const fn tick(&self) -> TickNumber {
        self.tick
    }

    pub const fn sequence(&self) -> SequenceNumber {
        self.sequence
    }

    pub const fn source(&self) -> &SimulationRegionKey {
        &self.source
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub const fn payload(&self) -> &EnvelopePayload {
        &self.payload
    }
}

impl CanonicalEncode for EventEnvelope {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_magic(EVENT_MAGIC);
        encoder.write_u16(ENVELOPE_SCHEMA_V1);
        self.tick.encode(encoder)?;
        self.sequence.encode(encoder)?;
        self.source.encode(encoder)?;
        self.kind.encode(encoder)?;
        self.payload.encode(encoder)
    }
}

impl CanonicalDecode for EventEnvelope {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        decoder.expect_magic(EVENT_MAGIC)?;
        expect_envelope_schema(decoder)?;
        Ok(Self::new(
            TickNumber::decode(decoder)?,
            SequenceNumber::decode(decoder)?,
            SimulationRegionKey::decode(decoder)?,
            ResourceId::decode(decoder)?,
            EnvelopePayload::decode(decoder)?,
        ))
    }
}

fn expect_envelope_schema(decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
    let schema = decoder.read_u16()?;
    if schema != ENVELOPE_SCHEMA_V1 {
        return Err(DecodeError::InvalidEnumTag {
            kind: "envelope schema",
            tag: u64::from(schema),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EnvelopeError {
    #[error("envelope payload is {actual} bytes; maximum is {maximum}")]
    PayloadTooLarge { actual: usize, maximum: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{decode_exact, encode_to_vec};
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};

    fn region() -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(-1, 2),
            RegionMappingVersion::V1,
        )
    }

    #[test]
    fn command_and_event_envelopes_round_trip_exactly() {
        let command = CommandEnvelope::new(
            TickNumber::new(4),
            SequenceNumber::new(9),
            CommandSource::Player(StableEntityId::new(2).unwrap()),
            region(),
            ResourceId::new("ferrite", "command/place_block").unwrap(),
            EnvelopePayload::new(vec![1, 2, 3]).unwrap(),
        );
        let bytes = encode_to_vec(&command).unwrap();
        assert_eq!(decode_exact::<CommandEnvelope>(&bytes).unwrap(), command);

        let event = EventEnvelope::new(
            TickNumber::new(4),
            SequenceNumber::new(10),
            region(),
            ResourceId::new("ferrite", "event/block_changed").unwrap(),
            EnvelopePayload::new(vec![4, 5]).unwrap(),
        );
        let bytes = encode_to_vec(&event).unwrap();
        assert_eq!(decode_exact::<EventEnvelope>(&bytes).unwrap(), event);
    }

    #[test]
    fn envelope_magic_schema_and_payload_are_bounded() {
        assert!(EnvelopePayload::new(vec![0; MAX_ENVELOPE_PAYLOAD_BYTES + 1]).is_err());
        let mut bytes = vec![0; 8];
        bytes[..4].copy_from_slice(b"NOPE");
        assert!(decode_exact::<CommandEnvelope>(&bytes).is_err());
    }
}
