//! Bounded replay headers, frames, and canonical log encoding.

use crate::codec::{
    CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError, Encoder, encode_to_vec,
};
use crate::envelope::{CommandEnvelope, CommandSource, EventEnvelope, TickNumber};
use crate::hash::{RegionHashRecord, StateHash};
use ferrite_foundation::identity::WorldId;
use ferrite_foundation::region::{RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::random::RandomAlgorithm;
use thiserror::Error;

const HEADER_MAGIC: &[u8; 4] = b"FRHD";
const FRAME_MAGIC: &[u8; 4] = b"FRFM";
const LOG_MAGIC: &[u8; 4] = b"FRLG";
const REPLAY_SCHEMA_V1: u16 = 1;
pub const MAX_REPLAY_FRAMES: usize = 10_000_000;
pub const MAX_FRAME_COMMANDS: usize = 65_536;
pub const MAX_FRAME_EVENTS: usize = 65_536;
pub const MAX_FRAME_REGIONS: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayHeader {
    implementation: ResourceId,
    world: WorldId,
    content_manifest: StateHash,
    mapping_version: RegionMappingVersion,
    random_algorithm: RandomAlgorithm,
    initial_tick: TickNumber,
}

impl ReplayHeader {
    pub const fn new(
        implementation: ResourceId,
        world: WorldId,
        content_manifest: StateHash,
        mapping_version: RegionMappingVersion,
        random_algorithm: RandomAlgorithm,
        initial_tick: TickNumber,
    ) -> Self {
        Self {
            implementation,
            world,
            content_manifest,
            mapping_version,
            random_algorithm,
            initial_tick,
        }
    }

    pub const fn implementation(&self) -> &ResourceId {
        &self.implementation
    }

    pub const fn world(&self) -> WorldId {
        self.world
    }

    pub const fn content_manifest(&self) -> StateHash {
        self.content_manifest
    }

    pub const fn mapping_version(&self) -> RegionMappingVersion {
        self.mapping_version
    }

    pub const fn random_algorithm(&self) -> RandomAlgorithm {
        self.random_algorithm
    }

    pub const fn initial_tick(&self) -> TickNumber {
        self.initial_tick
    }
}

impl CanonicalEncode for ReplayHeader {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_magic(HEADER_MAGIC);
        encoder.write_u16(REPLAY_SCHEMA_V1);
        self.implementation.encode(encoder)?;
        self.world.encode(encoder)?;
        self.content_manifest.encode(encoder)?;
        self.mapping_version.encode(encoder)?;
        self.random_algorithm.encode(encoder)?;
        self.initial_tick.encode(encoder)
    }
}

impl CanonicalDecode for ReplayHeader {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        decoder.expect_magic(HEADER_MAGIC)?;
        expect_schema(decoder, "replay header")?;
        Ok(Self::new(
            ResourceId::decode(decoder)?,
            WorldId::decode(decoder)?,
            StateHash::decode(decoder)?,
            RegionMappingVersion::decode(decoder)?,
            RandomAlgorithm::decode(decoder)?,
            TickNumber::decode(decoder)?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayFrame {
    tick: TickNumber,
    commands: Vec<CommandEnvelope>,
    events: Vec<EventEnvelope>,
    region_hashes: Vec<RegionHashRecord>,
    world_hash: StateHash,
}

impl ReplayFrame {
    pub fn new(
        tick: TickNumber,
        commands: Vec<CommandEnvelope>,
        events: Vec<EventEnvelope>,
        mut region_hashes: Vec<RegionHashRecord>,
        world_hash: StateHash,
    ) -> Result<Self, ReplayLogError> {
        ensure_limit(commands.len(), MAX_FRAME_COMMANDS, "commands")?;
        ensure_limit(events.len(), MAX_FRAME_EVENTS, "events")?;
        ensure_limit(region_hashes.len(), MAX_FRAME_REGIONS, "Region hashes")?;
        ensure_tick(tick, commands.iter().map(CommandEnvelope::tick), "command")?;
        ensure_tick(tick, events.iter().map(EventEnvelope::tick), "event")?;
        ensure_sequences(
            commands.iter().map(|command| command.sequence().get()),
            "command",
        )?;
        ensure_sequences(events.iter().map(|event| event.sequence().get()), "event")?;

        let mut keyed = region_hashes
            .drain(..)
            .map(|record| Ok((encode_to_vec(record.region())?, record)))
            .collect::<Result<Vec<_>, ReplayLogError>>()?;
        keyed.sort_by(|left, right| left.0.cmp(&right.0));
        if keyed.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(ReplayLogError::DuplicateRegionHash);
        }
        region_hashes = keyed.into_iter().map(|(_, record)| record).collect();
        Ok(Self {
            tick,
            commands,
            events,
            region_hashes,
            world_hash,
        })
    }

    pub const fn tick(&self) -> TickNumber {
        self.tick
    }

    pub fn commands(&self) -> impl ExactSizeIterator<Item = &CommandEnvelope> {
        self.commands.iter()
    }

    pub fn command_slice(&self) -> &[CommandEnvelope] {
        &self.commands
    }

    pub fn events(&self) -> impl ExactSizeIterator<Item = &EventEnvelope> {
        self.events.iter()
    }

    pub fn event_slice(&self) -> &[EventEnvelope] {
        &self.events
    }

    pub fn region_hashes(&self) -> impl ExactSizeIterator<Item = &RegionHashRecord> {
        self.region_hashes.iter()
    }

    pub fn region_hash_slice(&self) -> &[RegionHashRecord] {
        &self.region_hashes
    }

    pub const fn world_hash(&self) -> StateHash {
        self.world_hash
    }
}

impl CanonicalEncode for ReplayFrame {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_magic(FRAME_MAGIC);
        encoder.write_u16(REPLAY_SCHEMA_V1);
        self.tick.encode(encoder)?;
        encode_sequence(encoder, &self.commands)?;
        encode_sequence(encoder, &self.events)?;
        encode_sequence(encoder, &self.region_hashes)?;
        self.world_hash.encode(encoder)
    }
}

impl CanonicalDecode for ReplayFrame {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        decoder.expect_magic(FRAME_MAGIC)?;
        expect_schema(decoder, "replay frame")?;
        let tick = TickNumber::decode(decoder)?;
        let commands = decode_sequence(decoder, MAX_FRAME_COMMANDS)?;
        let events = decode_sequence(decoder, MAX_FRAME_EVENTS)?;
        let region_hashes = decode_sequence(decoder, MAX_FRAME_REGIONS)?;
        let world_hash = StateHash::decode(decoder)?;
        Self::new(tick, commands, events, region_hashes, world_hash).map_err(|_| {
            DecodeError::InvalidSemantic {
                kind: "replay frame",
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayLog {
    header: ReplayHeader,
    frames: Vec<ReplayFrame>,
}

impl ReplayLog {
    pub fn new(header: ReplayHeader, frames: Vec<ReplayFrame>) -> Result<Self, ReplayLogError> {
        ensure_limit(frames.len(), MAX_REPLAY_FRAMES, "frames")?;
        let mut previous_tick = None;
        for frame in &frames {
            if frame.tick.get() < header.initial_tick.get()
                || previous_tick.is_some_and(|previous| frame.tick.get() <= previous)
            {
                return Err(ReplayLogError::FrameOrder {
                    tick: frame.tick.get(),
                });
            }
            validate_frame_identity(&header, frame)?;
            previous_tick = Some(frame.tick.get());
        }
        Ok(Self { header, frames })
    }

    pub const fn header(&self) -> &ReplayHeader {
        &self.header
    }

    pub fn frames(&self) -> impl ExactSizeIterator<Item = &ReplayFrame> {
        self.frames.iter()
    }
}

impl CanonicalEncode for ReplayLog {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
        encoder.write_magic(LOG_MAGIC);
        encoder.write_u16(REPLAY_SCHEMA_V1);
        self.header.encode(encoder)?;
        encode_sequence(encoder, &self.frames)
    }
}

impl CanonicalDecode for ReplayLog {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        decoder.expect_magic(LOG_MAGIC)?;
        expect_schema(decoder, "replay log")?;
        let header = ReplayHeader::decode(decoder)?;
        let frames = decode_sequence(decoder, MAX_REPLAY_FRAMES)?;
        Self::new(header, frames).map_err(|_| DecodeError::InvalidSemantic { kind: "replay log" })
    }
}

fn validate_frame_identity(
    header: &ReplayHeader,
    frame: &ReplayFrame,
) -> Result<(), ReplayLogError> {
    for command in &frame.commands {
        validate_region(header, command.target())?;
        if let CommandSource::Region(source) = command.source() {
            validate_region(header, source)?;
        }
    }
    for event in &frame.events {
        validate_region(header, event.source())?;
    }
    for region_hash in &frame.region_hashes {
        validate_region(header, region_hash.region())?;
    }
    Ok(())
}

fn validate_region(
    header: &ReplayHeader,
    region: &SimulationRegionKey,
) -> Result<(), ReplayLogError> {
    if region.world() != header.world {
        return Err(ReplayLogError::WrongWorld {
            expected: header.world,
            actual: region.world(),
        });
    }
    if region.mapping_version() != header.mapping_version {
        return Err(ReplayLogError::WrongMappingVersion {
            expected: header.mapping_version,
            actual: region.mapping_version(),
        });
    }
    Ok(())
}

fn ensure_tick(
    expected: TickNumber,
    ticks: impl IntoIterator<Item = TickNumber>,
    kind: &'static str,
) -> Result<(), ReplayLogError> {
    for actual in ticks {
        if actual != expected {
            return Err(ReplayLogError::EnvelopeTick {
                kind,
                expected: expected.get(),
                actual: actual.get(),
            });
        }
    }
    Ok(())
}

fn ensure_sequences(
    sequences: impl IntoIterator<Item = u64>,
    kind: &'static str,
) -> Result<(), ReplayLogError> {
    let mut previous = None;
    for sequence in sequences {
        if previous.is_some_and(|previous| sequence <= previous) {
            return Err(ReplayLogError::SequenceOrder { kind, sequence });
        }
        previous = Some(sequence);
    }
    Ok(())
}

fn ensure_limit(actual: usize, maximum: usize, kind: &'static str) -> Result<(), ReplayLogError> {
    if actual > maximum {
        return Err(ReplayLogError::CountLimit {
            kind,
            actual,
            maximum,
        });
    }
    Ok(())
}

fn encode_sequence<T: CanonicalEncode>(
    encoder: &mut Encoder,
    values: &[T],
) -> Result<(), EncodeError> {
    encoder.write_var_u64(values.len() as u64);
    for value in values {
        value.encode(encoder)?;
    }
    Ok(())
}

fn decode_sequence<T: CanonicalDecode>(
    decoder: &mut Decoder<'_>,
    maximum: usize,
) -> Result<Vec<T>, DecodeError> {
    let length = decoder.read_length(maximum)?;
    // Decode before growing so a short hostile input cannot reserve its declared maximum count.
    let mut values = Vec::new();
    for _ in 0..length {
        values.push(T::decode(decoder)?);
    }
    Ok(values)
}

fn expect_schema(decoder: &mut Decoder<'_>, kind: &'static str) -> Result<(), DecodeError> {
    let schema = decoder.read_u16()?;
    if schema != REPLAY_SCHEMA_V1 {
        return Err(DecodeError::InvalidEnumTag {
            kind,
            tag: u64::from(schema),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReplayLogError {
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error("replay {kind} count {actual} exceeds {maximum}")]
    CountLimit {
        kind: &'static str,
        actual: usize,
        maximum: usize,
    },
    #[error("replay {kind} tick {actual} differs from frame tick {expected}")]
    EnvelopeTick {
        kind: &'static str,
        expected: u64,
        actual: u64,
    },
    #[error("replay {kind} sequence {sequence} is not strictly increasing")]
    SequenceOrder { kind: &'static str, sequence: u64 },
    #[error("replay frame repeats a Region hash")]
    DuplicateRegionHash,
    #[error("replay frame tick {tick} is before or not after its predecessor")]
    FrameOrder { tick: u64 },
    #[error("replay Region belongs to world {actual}, expected {expected}")]
    WrongWorld { expected: WorldId, actual: WorldId },
    #[error("replay Region mapping version {actual:?} differs from {expected:?}")]
    WrongMappingVersion {
        expected: RegionMappingVersion,
        actual: RegionMappingVersion,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{decode_exact, encode_to_vec};
    use crate::envelope::{CommandSource, EnvelopePayload, SequenceNumber};
    use ferrite_foundation::identity::DimensionId;
    use ferrite_foundation::region::RegionCoord;

    fn region(world: WorldId) -> SimulationRegionKey {
        SimulationRegionKey::new(
            world,
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        )
    }

    fn fixture_log() -> ReplayLog {
        let world = WorldId::new(1).unwrap();
        let region = region(world);
        let command = CommandEnvelope::new(
            TickNumber::new(2),
            SequenceNumber::new(1),
            CommandSource::System,
            region.clone(),
            ResourceId::new("ferrite", "command/tick").unwrap(),
            EnvelopePayload::new(vec![1]).unwrap(),
        );
        let frame = ReplayFrame::new(
            TickNumber::new(2),
            vec![command],
            Vec::new(),
            vec![RegionHashRecord::new(
                region,
                StateHash::from_bytes([2; 32]),
            )],
            StateHash::from_bytes([3; 32]),
        )
        .unwrap();
        ReplayLog::new(
            ReplayHeader::new(
                ResourceId::new("ferrite", "server").unwrap(),
                world,
                StateHash::from_bytes([1; 32]),
                RegionMappingVersion::V1,
                RandomAlgorithm::Xoshiro256StarStarV1,
                TickNumber::new(1),
            ),
            vec![frame],
        )
        .unwrap()
    }

    #[test]
    fn bounded_replay_log_round_trips_canonically() {
        let log = fixture_log();
        let bytes = encode_to_vec(&log).unwrap();
        assert_eq!(
            StateHash::from_bytes(*blake3::hash(&bytes).as_bytes()).to_string(),
            "b319b3ce9d9e3f5fb64b17ec39c25b759589eac1fcfd0a3dca5187e02b470c79"
        );
        assert_eq!(decode_exact::<ReplayLog>(&bytes).unwrap(), log);
    }

    #[test]
    fn frame_order_and_identity_are_validated() {
        let log = fixture_log();
        let frame = log.frames.into_iter().next().unwrap();
        let wrong_world = ReplayHeader::new(
            ResourceId::new("ferrite", "server").unwrap(),
            WorldId::new(2).unwrap(),
            StateHash::from_bytes([1; 32]),
            RegionMappingVersion::V1,
            RandomAlgorithm::Xoshiro256StarStarV1,
            TickNumber::new(1),
        );
        assert!(ReplayLog::new(wrong_world, vec![frame]).is_err());
    }

    #[test]
    fn truncated_declared_maximum_sequence_fails_before_bulk_allocation() {
        let mut encoder = Encoder::new();
        encoder.write_var_u64(MAX_REPLAY_FRAMES as u64);
        let bytes = encoder.into_bytes();
        let mut decoder = Decoder::new(&bytes);

        assert!(decode_sequence::<ReplayFrame>(&mut decoder, MAX_REPLAY_FRAMES).is_err());
    }
}
