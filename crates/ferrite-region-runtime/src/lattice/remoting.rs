//! Bounded Ferrite Region envelopes carried by Lattice remoting frames.

use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::tick::GameTick;
use lattice_remoting::wire::{Frame, FrameKind};
use thiserror::Error;

const REMOTE_MAGIC: &[u8; 4] = b"FREM";
const REMOTE_SCHEMA_V1: u16 = 1;
const MAX_DIMENSION_BYTES: usize = 32 * 1024;
pub const MAX_REMOTE_REGION_PAYLOAD_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RemoteRegionMessageKind {
    Command = 0,
    Boundary = 1,
    ImmediateEffect = 2,
    EntityTransfer = 3,
    Acknowledgement = 4,
}

impl RemoteRegionMessageKind {
    fn from_tag(tag: u8) -> Result<Self, RemotingAdapterError> {
        match tag {
            0 => Ok(Self::Command),
            1 => Ok(Self::Boundary),
            2 => Ok(Self::ImmediateEffect),
            3 => Ok(Self::EntityTransfer),
            4 => Ok(Self::Acknowledgement),
            _ => Err(RemotingAdapterError::UnknownMessageKind(tag)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRegionEnvelope {
    kind: RemoteRegionMessageKind,
    tick: GameTick,
    source: SimulationRegionKey,
    target: SimulationRegionKey,
    source_generation: ActivationGeneration,
    target_generation: ActivationGeneration,
    source_sequence: u64,
    payload: Vec<u8>,
}

impl RemoteRegionEnvelope {
    pub fn new(
        header: RemoteRegionEnvelopeHeader,
        payload: Vec<u8>,
    ) -> Result<Self, RemotingAdapterError> {
        validate_endpoints(&header.source, &header.target)?;
        if payload.len() > MAX_REMOTE_REGION_PAYLOAD_BYTES {
            return Err(RemotingAdapterError::PayloadTooLarge {
                actual: payload.len(),
                maximum: MAX_REMOTE_REGION_PAYLOAD_BYTES,
            });
        }
        Ok(Self {
            kind: header.kind,
            tick: header.tick,
            source: header.source,
            target: header.target,
            source_generation: header.source_generation,
            target_generation: header.target_generation,
            source_sequence: header.source_sequence,
            payload,
        })
    }

    pub const fn kind(&self) -> RemoteRegionMessageKind {
        self.kind
    }

    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub const fn source(&self) -> &SimulationRegionKey {
        &self.source
    }

    pub const fn target(&self) -> &SimulationRegionKey {
        &self.target
    }

    pub const fn source_generation(&self) -> ActivationGeneration {
        self.source_generation
    }

    pub const fn target_generation(&self) -> ActivationGeneration {
        self.target_generation
    }

    pub const fn source_sequence(&self) -> u64 {
        self.source_sequence
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRegionEnvelopeHeader {
    pub kind: RemoteRegionMessageKind,
    pub tick: GameTick,
    pub source: SimulationRegionKey,
    pub target: SimulationRegionKey,
    pub source_generation: ActivationGeneration,
    pub target_generation: ActivationGeneration,
    pub source_sequence: u64,
}

#[derive(Debug, Clone)]
pub struct LatticeRemotingAdapter {
    maximum_frame_bytes: usize,
}

impl LatticeRemotingAdapter {
    pub fn new(maximum_frame_bytes: usize) -> Result<Self, RemotingAdapterError> {
        if maximum_frame_bytes == 0 {
            return Err(RemotingAdapterError::ZeroFrameLimit);
        }
        Ok(Self {
            maximum_frame_bytes,
        })
    }

    pub fn encode(
        &self,
        envelope: &RemoteRegionEnvelope,
    ) -> Result<LatticeTransportFrame, RemotingAdapterError> {
        let bytes = encode_envelope(envelope)?;
        if bytes.len() > self.maximum_frame_bytes {
            return Err(RemotingAdapterError::FrameTooLarge {
                actual: bytes.len(),
                maximum: self.maximum_frame_bytes,
            });
        }
        Ok(LatticeTransportFrame {
            frame: Frame::new(FrameKind::EntityTell, bytes.into()),
        })
    }

    pub fn decode(
        &self,
        frame: &LatticeTransportFrame,
    ) -> Result<RemoteRegionEnvelope, RemotingAdapterError> {
        if frame.frame.kind != FrameKind::EntityTell {
            return Err(RemotingAdapterError::WrongFrameKind);
        }
        if frame.frame.payload_len() > self.maximum_frame_bytes {
            return Err(RemotingAdapterError::FrameTooLarge {
                actual: frame.frame.payload_len(),
                maximum: self.maximum_frame_bytes,
            });
        }
        decode_envelope(frame.frame.payload())
    }
}

#[derive(Debug, Clone)]
pub struct LatticeTransportFrame {
    frame: Frame,
}

impl LatticeTransportFrame {
    pub fn payload_len(&self) -> usize {
        self.frame.payload_len()
    }

    pub fn transport_payload(&self) -> &[u8] {
        self.frame.payload()
    }

    pub fn from_transport_payload(payload: Vec<u8>) -> Self {
        Self {
            frame: Frame::new(FrameKind::EntityTell, payload.into()),
        }
    }
}

fn encode_envelope(envelope: &RemoteRegionEnvelope) -> Result<Vec<u8>, RemotingAdapterError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(REMOTE_MAGIC);
    bytes.extend_from_slice(&REMOTE_SCHEMA_V1.to_le_bytes());
    bytes.push(envelope.kind as u8);
    bytes.extend_from_slice(&envelope.tick.get().to_le_bytes());
    encode_region(&mut bytes, &envelope.source)?;
    encode_region(&mut bytes, &envelope.target)?;
    bytes.extend_from_slice(&envelope.source_generation.get().to_le_bytes());
    bytes.extend_from_slice(&envelope.target_generation.get().to_le_bytes());
    bytes.extend_from_slice(&envelope.source_sequence.to_le_bytes());
    bytes.extend_from_slice(&(envelope.payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&envelope.payload);
    Ok(bytes)
}

fn decode_envelope(bytes: &[u8]) -> Result<RemoteRegionEnvelope, RemotingAdapterError> {
    let mut decoder = TransportDecoder::new(bytes);
    if decoder.take(REMOTE_MAGIC.len())? != REMOTE_MAGIC {
        return Err(RemotingAdapterError::WrongMagic);
    }
    let schema = decoder.u16()?;
    if schema != REMOTE_SCHEMA_V1 {
        return Err(RemotingAdapterError::UnsupportedSchema(schema));
    }
    let kind = RemoteRegionMessageKind::from_tag(decoder.u8()?)?;
    let tick = GameTick::new(decoder.u64()?);
    let source = decode_region(&mut decoder)?;
    let target = decode_region(&mut decoder)?;
    let source_generation = ActivationGeneration::new(decoder.u64()?)
        .map_err(|_| RemotingAdapterError::InvalidIdentity)?;
    let target_generation = ActivationGeneration::new(decoder.u64()?)
        .map_err(|_| RemotingAdapterError::InvalidIdentity)?;
    let source_sequence = decoder.u64()?;
    let payload_length = decoder.u32()? as usize;
    if payload_length > MAX_REMOTE_REGION_PAYLOAD_BYTES {
        return Err(RemotingAdapterError::PayloadTooLarge {
            actual: payload_length,
            maximum: MAX_REMOTE_REGION_PAYLOAD_BYTES,
        });
    }
    let payload = decoder.take(payload_length)?.to_vec();
    decoder.finish()?;
    RemoteRegionEnvelope::new(
        RemoteRegionEnvelopeHeader {
            kind,
            tick,
            source,
            target,
            source_generation,
            target_generation,
            source_sequence,
        },
        payload,
    )
}

fn encode_region(
    bytes: &mut Vec<u8>,
    region: &SimulationRegionKey,
) -> Result<(), RemotingAdapterError> {
    let dimension = region.dimension().resource().to_string();
    let length =
        u16::try_from(dimension.len()).map_err(|_| RemotingAdapterError::DimensionTooLarge)?;
    bytes.extend_from_slice(&region.world().get().to_le_bytes());
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(dimension.as_bytes());
    bytes.extend_from_slice(&region.coordinate().x().to_le_bytes());
    bytes.extend_from_slice(&region.coordinate().z().to_le_bytes());
    bytes.extend_from_slice(&region.mapping_version().get().to_le_bytes());
    Ok(())
}

fn decode_region(
    decoder: &mut TransportDecoder<'_>,
) -> Result<SimulationRegionKey, RemotingAdapterError> {
    let world = WorldId::new(decoder.u128()?).map_err(|_| RemotingAdapterError::InvalidIdentity)?;
    let length = usize::from(decoder.u16()?);
    if length > MAX_DIMENSION_BYTES {
        return Err(RemotingAdapterError::DimensionTooLarge);
    }
    let dimension = std::str::from_utf8(decoder.take(length)?)
        .map_err(|_| RemotingAdapterError::InvalidIdentity)?
        .parse::<ResourceId>()
        .map(DimensionId::new)
        .map_err(|_| RemotingAdapterError::InvalidIdentity)?;
    let coordinate = RegionCoord::new(decoder.i32()?, decoder.i32()?);
    let version = RegionMappingVersion::new(decoder.u16()?)
        .map_err(|_| RemotingAdapterError::InvalidIdentity)?;
    Ok(SimulationRegionKey::new(
        world, dimension, coordinate, version,
    ))
}

fn validate_endpoints(
    source: &SimulationRegionKey,
    target: &SimulationRegionKey,
) -> Result<(), RemotingAdapterError> {
    if source == target {
        return Err(RemotingAdapterError::SelfTarget);
    }
    if source.world() != target.world() || source.mapping_version() != target.mapping_version() {
        return Err(RemotingAdapterError::IncompatibleEndpoints);
    }
    Ok(())
}

struct TransportDecoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> TransportDecoder<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn u8(&mut self) -> Result<u8, RemotingAdapterError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, RemotingAdapterError> {
        Ok(u16::from_le_bytes(self.fixed()?))
    }

    fn u32(&mut self) -> Result<u32, RemotingAdapterError> {
        Ok(u32::from_le_bytes(self.fixed()?))
    }

    fn i32(&mut self) -> Result<i32, RemotingAdapterError> {
        Ok(i32::from_le_bytes(self.fixed()?))
    }

    fn u64(&mut self) -> Result<u64, RemotingAdapterError> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }

    fn u128(&mut self) -> Result<u128, RemotingAdapterError> {
        Ok(u128::from_le_bytes(self.fixed()?))
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], RemotingAdapterError> {
        self.take(N)?
            .try_into()
            .map_err(|_| RemotingAdapterError::Truncated)
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], RemotingAdapterError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(RemotingAdapterError::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(RemotingAdapterError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn finish(self) -> Result<(), RemotingAdapterError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(RemotingAdapterError::TrailingBytes)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RemotingAdapterError {
    #[error("Lattice remoting frame limit cannot be zero")]
    ZeroFrameLimit,
    #[error("remote Region payload has {actual} bytes, exceeding limit {maximum}")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error("Lattice frame has {actual} bytes, exceeding limit {maximum}")]
    FrameTooLarge { actual: usize, maximum: usize },
    #[error("remote Region envelope cannot target its source")]
    SelfTarget,
    #[error("remote Region endpoints are in different ownership domains")]
    IncompatibleEndpoints,
    #[error("remote Region envelope has the wrong magic")]
    WrongMagic,
    #[error("remote Region envelope schema {0} is unsupported")]
    UnsupportedSchema(u16),
    #[error("remote Region message kind {0} is unknown")]
    UnknownMessageKind(u8),
    #[error("remote Region envelope is truncated")]
    Truncated,
    #[error("remote Region envelope has trailing bytes")]
    TrailingBytes,
    #[error("remote Region envelope contains an invalid identity")]
    InvalidIdentity,
    #[error("remote Region dimension identity is too large")]
    DimensionTooLarge,
    #[error("Lattice frame kind is not an entity message")]
    WrongFrameKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(x: i32) -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(x, 0),
            RegionMappingVersion::V1,
        )
    }

    #[test]
    fn ferrite_envelope_round_trips_through_a_lattice_frame() {
        let envelope = RemoteRegionEnvelope::new(
            RemoteRegionEnvelopeHeader {
                kind: RemoteRegionMessageKind::Boundary,
                tick: GameTick::new(9),
                source: region(0),
                target: region(1),
                source_generation: ActivationGeneration::INITIAL,
                target_generation: ActivationGeneration::new(2).unwrap(),
                source_sequence: 7,
            },
            vec![1, 2, 3],
        )
        .unwrap();
        let adapter = LatticeRemotingAdapter::new(4096).unwrap();
        let frame = adapter.encode(&envelope).unwrap();
        assert_eq!(frame.frame.kind, FrameKind::EntityTell);
        assert_eq!(adapter.decode(&frame).unwrap(), envelope);
    }

    #[test]
    fn bounds_and_endpoint_identity_fail_closed() {
        assert!(
            RemoteRegionEnvelope::new(
                RemoteRegionEnvelopeHeader {
                    kind: RemoteRegionMessageKind::Command,
                    tick: GameTick::ZERO,
                    source: region(0),
                    target: region(0),
                    source_generation: ActivationGeneration::INITIAL,
                    target_generation: ActivationGeneration::INITIAL,
                    source_sequence: 0,
                },
                vec![],
            )
            .is_err()
        );
        let adapter = LatticeRemotingAdapter::new(1).unwrap();
        let envelope = RemoteRegionEnvelope::new(
            RemoteRegionEnvelopeHeader {
                kind: RemoteRegionMessageKind::Command,
                tick: GameTick::new(1),
                source: region(0),
                target: region(1),
                source_generation: ActivationGeneration::INITIAL,
                target_generation: ActivationGeneration::INITIAL,
                source_sequence: 0,
            },
            vec![],
        )
        .unwrap();
        assert!(adapter.encode(&envelope).is_err());
    }
}
