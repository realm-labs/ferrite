//! Versioned Simulation scheduled-work, runtime-stream, and boundary-fence records.

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::RegionCoord;
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{SnapshotError, SnapshotRecord, SnapshotRecordKind};
use ferrite_simulation::random::{DeterministicRng, RandomAlgorithm, RandomError, RandomState};
use ferrite_simulation::random_tick::position::RandomPositionStream;
use ferrite_simulation::scheduled_tick::level::ScheduledTickQueue;
use ferrite_simulation::scheduled_tick::record::{SavedTick, SubTickCounter, TickPriority};
use std::collections::BTreeSet;
use std::str::FromStr;
use thiserror::Error;

const RUNTIME_MAGIC: &[u8; 4] = b"F5R1";
const SCHEDULE_MAGIC: &[u8; 4] = b"F5S1";
const MAX_TICKS_PER_CHUNK: usize = 1_000_000;
const MAX_IDENTITY_BYTES: usize = u16::MAX as usize;

// These Goal 01 identities are persisted compatibility surfaces. G03-P1-B3 owns their migration.
const LEGACY_SCHEDULED_BLOCK_DOMAIN: &str = "phase5/scheduled_block_v1";
const LEGACY_SCHEDULED_FLUID_DOMAIN: &str = "phase5/scheduled_fluid_v1";
const LEGACY_RUNTIME_DOMAIN: &str = "ferrite:phase5/runtime_v1";
const LEGACY_RECEIPT_DOMAIN: &str = "ferrite:phase5/boundary_receipt_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScheduledQueueKind {
    Block,
    Fluid,
}

impl ScheduledQueueKind {
    fn domain(self) -> ResourceId {
        let path = match self {
            Self::Block => LEGACY_SCHEDULED_BLOCK_DOMAIN,
            Self::Fluid => LEGACY_SCHEDULED_FLUID_DOMAIN,
        };
        ResourceId::new("ferrite", path).expect("static Simulation domain is valid")
    }

    fn from_domain(domain: &ResourceId) -> Option<Self> {
        match (domain.namespace(), domain.path()) {
            ("ferrite", LEGACY_SCHEDULED_BLOCK_DOMAIN) => Some(Self::Block),
            ("ferrite", LEGACY_SCHEDULED_FLUID_DOMAIN) => Some(Self::Fluid),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppliedBoundaryReceipt {
    pub source: RegionCoord,
    pub source_generation: ActivationGeneration,
    pub source_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledChunkContinuity {
    pub kind: ScheduledQueueKind,
    pub chunk: ChunkPos,
    pub ticks: Vec<SavedTick<ResourceId>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationContinuity {
    pub next_sub_tick: i64,
    pub random_position_value: i32,
    pub gameplay_random_algorithm: RandomAlgorithm,
    pub gameplay_random_state: RandomState,
    pub scheduled: Vec<ScheduledChunkContinuity>,
    pub applied_boundaries: BTreeSet<AppliedBoundaryReceipt>,
}

impl SimulationContinuity {
    pub fn capture(
        blocks: &ScheduledTickQueue<ResourceId>,
        fluids: &ScheduledTickQueue<ResourceId>,
        current_tick: i64,
        sub_tick_counter: SubTickCounter,
        random_position: RandomPositionStream,
        gameplay_random: &DeterministicRng,
        applied_boundaries: BTreeSet<AppliedBoundaryReceipt>,
    ) -> Self {
        let mut scheduled = Vec::new();
        capture_queue(
            blocks,
            ScheduledQueueKind::Block,
            current_tick,
            &mut scheduled,
        );
        capture_queue(
            fluids,
            ScheduledQueueKind::Fluid,
            current_tick,
            &mut scheduled,
        );
        scheduled.sort_by_key(|entry| (entry.kind, entry.chunk));
        Self {
            next_sub_tick: sub_tick_counter.value(),
            random_position_value: random_position.value(),
            gameplay_random_algorithm: gameplay_random.algorithm(),
            gameplay_random_state: gameplay_random.state(),
            scheduled,
            applied_boundaries,
        }
    }

    pub fn to_records(&self) -> Result<Vec<SnapshotRecord>, ContinuityError> {
        let mut records =
            Vec::with_capacity(1 + self.scheduled.len() + self.applied_boundaries.len());
        let mut runtime = Vec::with_capacity(50);
        runtime.extend_from_slice(RUNTIME_MAGIC);
        runtime.extend_from_slice(&self.next_sub_tick.to_be_bytes());
        runtime.extend_from_slice(&self.random_position_value.to_be_bytes());
        runtime.extend_from_slice(&self.gameplay_random_algorithm.stable_tag().to_be_bytes());
        for word in self.gameplay_random_state.words() {
            runtime.extend_from_slice(&word.to_be_bytes());
        }
        records.push(SnapshotRecord::new(
            SnapshotRecordKind::Extension,
            runtime_domain(),
            Vec::new(),
            runtime,
        )?);
        for chunk in &self.scheduled {
            records.push(SnapshotRecord::new(
                SnapshotRecordKind::ScheduledWork,
                chunk.kind.domain(),
                encode_chunk_key(chunk.chunk),
                encode_ticks(&chunk.ticks)?,
            )?);
        }
        for receipt in &self.applied_boundaries {
            records.push(SnapshotRecord::new(
                SnapshotRecordKind::AppliedBoundary,
                receipt_domain(),
                encode_receipt(receipt),
                Vec::new(),
            )?);
        }
        Ok(records)
    }

    pub fn from_records(records: &[SnapshotRecord]) -> Result<Self, ContinuityError> {
        let mut runtime = None;
        let mut scheduled = Vec::new();
        let mut applied_boundaries = BTreeSet::new();
        for record in records {
            if record.kind() == SnapshotRecordKind::Extension
                && record.domain() == &runtime_domain()
            {
                if runtime.is_some() || !record.key().is_empty() {
                    return Err(ContinuityError::DuplicateRuntimeRecord);
                }
                runtime = Some(decode_runtime(record.value())?);
            } else if record.kind() == SnapshotRecordKind::ScheduledWork
                && let Some(kind) = ScheduledQueueKind::from_domain(record.domain())
            {
                let chunk = decode_chunk_key(record.key())?;
                let ticks = decode_ticks(record.value(), chunk)?;
                scheduled.push(ScheduledChunkContinuity { kind, chunk, ticks });
            } else if record.kind() == SnapshotRecordKind::AppliedBoundary
                && record.domain() == &receipt_domain()
            {
                if !record.value().is_empty() {
                    return Err(ContinuityError::InvalidReceipt);
                }
                if !applied_boundaries.insert(decode_receipt(record.key())?) {
                    return Err(ContinuityError::DuplicateReceipt);
                }
            }
        }
        let (
            next_sub_tick,
            random_position_value,
            gameplay_random_algorithm,
            gameplay_random_state,
        ) = runtime.ok_or(ContinuityError::MissingRuntimeRecord)?;
        scheduled.sort_by_key(|entry| (entry.kind, entry.chunk));
        if scheduled
            .windows(2)
            .any(|pair| pair[0].kind == pair[1].kind && pair[0].chunk == pair[1].chunk)
        {
            return Err(ContinuityError::DuplicateScheduledChunk);
        }
        Ok(Self {
            next_sub_tick,
            random_position_value,
            gameplay_random_algorithm,
            gameplay_random_state,
            scheduled,
            applied_boundaries,
        })
    }
}

fn capture_queue(
    queue: &ScheduledTickQueue<ResourceId>,
    kind: ScheduledQueueKind,
    current_tick: i64,
    output: &mut Vec<ScheduledChunkContinuity>,
) {
    output.extend(queue.registered_chunks().map(|chunk| {
        let ticks = queue
            .pack_container(chunk, current_tick)
            .expect("registered chunk remains present during immutable capture");
        ScheduledChunkContinuity { kind, chunk, ticks }
    }));
}

fn encode_ticks(ticks: &[SavedTick<ResourceId>]) -> Result<Vec<u8>, ContinuityError> {
    if ticks.len() > MAX_TICKS_PER_CHUNK || ticks.len() > u32::MAX as usize {
        return Err(ContinuityError::TooManyTicks {
            actual: ticks.len(),
            maximum: MAX_TICKS_PER_CHUNK,
        });
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(SCHEDULE_MAGIC);
    bytes.extend_from_slice(&(ticks.len() as u32).to_be_bytes());
    for tick in ticks {
        let identity = tick.type_identity.to_string();
        if identity.len() > MAX_IDENTITY_BYTES {
            return Err(ContinuityError::IdentityTooLong {
                actual: identity.len(),
                maximum: MAX_IDENTITY_BYTES,
            });
        }
        bytes.extend_from_slice(&(identity.len() as u16).to_be_bytes());
        bytes.extend_from_slice(identity.as_bytes());
        bytes.extend_from_slice(&tick.position.x.to_be_bytes());
        bytes.extend_from_slice(&tick.position.y.to_be_bytes());
        bytes.extend_from_slice(&tick.position.z.to_be_bytes());
        bytes.extend_from_slice(&tick.delay.to_be_bytes());
        bytes.push(tick.priority.value() as u8);
    }
    Ok(bytes)
}

fn decode_ticks(
    bytes: &[u8],
    chunk: ChunkPos,
) -> Result<Vec<SavedTick<ResourceId>>, ContinuityError> {
    let mut cursor = Cursor::new(bytes);
    cursor.expect(SCHEDULE_MAGIC)?;
    let count = cursor.u32()? as usize;
    if count > MAX_TICKS_PER_CHUNK {
        return Err(ContinuityError::TooManyTicks {
            actual: count,
            maximum: MAX_TICKS_PER_CHUNK,
        });
    }
    let mut ticks = Vec::with_capacity(count);
    for _ in 0..count {
        let identity_length = usize::from(cursor.u16()?);
        let identity = std::str::from_utf8(cursor.take(identity_length)?)
            .map_err(|_| ContinuityError::InvalidIdentity)?
            .parse()
            .map_err(|_| ContinuityError::InvalidIdentity)?;
        let position = BlockPos::new(cursor.i32()?, cursor.i32()?, cursor.i32()?);
        if position.chunk() != chunk {
            return Err(ContinuityError::TickOutsideChunk { position, chunk });
        }
        ticks.push(SavedTick::new(
            identity,
            position,
            cursor.i32()?,
            TickPriority::from_value(i32::from(cursor.i8()?)),
        ));
    }
    cursor.finish()?;
    Ok(ticks)
}

fn encode_chunk_key(chunk: ChunkPos) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8);
    bytes.extend_from_slice(&chunk.x.to_be_bytes());
    bytes.extend_from_slice(&chunk.z.to_be_bytes());
    bytes
}

fn decode_chunk_key(bytes: &[u8]) -> Result<ChunkPos, ContinuityError> {
    let mut cursor = Cursor::new(bytes);
    let chunk = ChunkPos::new(cursor.i32()?, cursor.i32()?);
    cursor.finish()?;
    Ok(chunk)
}

fn encode_receipt(receipt: &AppliedBoundaryReceipt) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(24);
    bytes.extend_from_slice(&receipt.source.x().to_be_bytes());
    bytes.extend_from_slice(&receipt.source.z().to_be_bytes());
    bytes.extend_from_slice(&receipt.source_generation.get().to_be_bytes());
    bytes.extend_from_slice(&receipt.source_sequence.to_be_bytes());
    bytes
}

fn decode_receipt(bytes: &[u8]) -> Result<AppliedBoundaryReceipt, ContinuityError> {
    let mut cursor = Cursor::new(bytes);
    let source = RegionCoord::new(cursor.i32()?, cursor.i32()?);
    let source_generation =
        ActivationGeneration::new(cursor.u64()?).map_err(|_| ContinuityError::InvalidReceipt)?;
    let source_sequence = cursor.u64()?;
    cursor.finish()?;
    Ok(AppliedBoundaryReceipt {
        source,
        source_generation,
        source_sequence,
    })
}

fn decode_runtime(
    bytes: &[u8],
) -> Result<(i64, i32, RandomAlgorithm, RandomState), ContinuityError> {
    let mut cursor = Cursor::new(bytes);
    cursor.expect(RUNTIME_MAGIC)?;
    let next_sub_tick = cursor.i64()?;
    let random_position = cursor.i32()?;
    let algorithm = match cursor.u16()? {
        1 => RandomAlgorithm::Xoshiro256StarStarV1,
        tag => return Err(ContinuityError::UnknownRandomAlgorithm(tag)),
    };
    let state = RandomState::new([cursor.u64()?, cursor.u64()?, cursor.u64()?, cursor.u64()?])?;
    cursor.finish()?;
    Ok((next_sub_tick, random_position, algorithm, state))
}

fn runtime_domain() -> ResourceId {
    ResourceId::from_str(LEGACY_RUNTIME_DOMAIN).expect("static Simulation runtime domain is valid")
}

fn receipt_domain() -> ResourceId {
    ResourceId::from_str(LEGACY_RECEIPT_DOMAIN).expect("static Simulation receipt domain is valid")
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn expect(&mut self, expected: &[u8]) -> Result<(), ContinuityError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(ContinuityError::WrongMagic)
        }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], ContinuityError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(ContinuityError::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(ContinuityError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn u16(&mut self) -> Result<u16, ContinuityError> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, ContinuityError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, ContinuityError> {
        Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i8(&mut self) -> Result<i8, ContinuityError> {
        Ok(self.take(1)?[0] as i8)
    }

    fn u64(&mut self) -> Result<u64, ContinuityError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn i64(&mut self) -> Result<i64, ContinuityError> {
        Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn finish(self) -> Result<(), ContinuityError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(ContinuityError::TrailingBytes)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContinuityError {
    #[error("Simulation continuity record has the wrong magic")]
    WrongMagic,
    #[error("Simulation continuity record is truncated")]
    Truncated,
    #[error("Simulation continuity record has trailing bytes")]
    TrailingBytes,
    #[error("Simulation continuity runtime record is missing")]
    MissingRuntimeRecord,
    #[error("Simulation continuity runtime record is duplicated")]
    DuplicateRuntimeRecord,
    #[error("Simulation scheduled chunk record is duplicated")]
    DuplicateScheduledChunk,
    #[error("Simulation applied-boundary receipt is duplicated")]
    DuplicateReceipt,
    #[error("Simulation applied-boundary receipt is invalid")]
    InvalidReceipt,
    #[error("Simulation scheduled identity is invalid")]
    InvalidIdentity,
    #[error("Simulation gameplay random algorithm tag {0} is unknown")]
    UnknownRandomAlgorithm(u16),
    #[error("Simulation scheduled identity has {actual} bytes, exceeding {maximum}")]
    IdentityTooLong { actual: usize, maximum: usize },
    #[error("Simulation chunk has {actual} ticks, exceeding {maximum}")]
    TooManyTicks { actual: usize, maximum: usize },
    #[error("scheduled tick {position:?} is outside record chunk {chunk:?}")]
    TickOutsideChunk { position: BlockPos, chunk: ChunkPos },
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Random(#[from] RandomError),
}
