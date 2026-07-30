use std::collections::BTreeMap;

use ferrite_foundation::coordinate::{BlockPos, SectionPos};
use ferrite_foundation::identity::StableEntityId;
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    BlockUpdate, PlayClientboundPacket, SectionBlockChange, SectionBlocksUpdate,
};
use ferrite_region_runtime::local::LocalTickReport;
use ferrite_simulation::journal::{JournalDomain, JournalEntry};
use ferrite_world::id::BlockStateId;
use thiserror::Error;

use crate::chunk::projection::{JavaTerrainRegistryMap, TerrainProjectionError};
use crate::player::block::command::{BLOCK_RESULT_PATH, BLOCK_UPDATE_PATH};

const RESULT_MAGIC: &[u8; 4] = b"FBR1";
const UPDATE_MAGIC: &[u8; 4] = b"FBU1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthoritativeBlockUpdate {
    pub position: BlockPos,
    pub state: BlockStateId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockCommandResult {
    pub player: StableEntityId,
    pub command_sequence: u64,
    pub outcome: BlockCommandOutcome,
    pub corrections: Vec<AuthoritativeBlockUpdate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCommandOutcome {
    Applied,
    Rejected,
    Tracking,
    Cleared,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommittedBlockProjection {
    pub results: Vec<BlockCommandResult>,
    pub packets: Vec<PlayClientboundPacket>,
}

impl BlockCommandResult {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(30 + self.corrections.len() * 16);
        bytes.extend_from_slice(RESULT_MAGIC);
        bytes.extend_from_slice(&self.player.to_be_bytes());
        bytes.extend_from_slice(&self.command_sequence.to_be_bytes());
        bytes.push(outcome_tag(self.outcome));
        bytes.push(self.corrections.len() as u8);
        for update in &self.corrections {
            encode_update(&mut bytes, *update);
        }
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlockReplicationError> {
        if bytes.len() < 30 || &bytes[..4] != RESULT_MAGIC {
            return Err(BlockReplicationError::InvalidResult);
        }
        let player = StableEntityId::new(read_u128(bytes, 4)?)
            .map_err(|_| BlockReplicationError::InvalidResult)?;
        let command_sequence = read_u64(bytes, 20)?;
        let outcome = decode_outcome(bytes[28])?;
        let count = usize::from(bytes[29]);
        if bytes.len() != 30 + count * 16 {
            return Err(BlockReplicationError::InvalidResult);
        }
        let corrections = (0..count)
            .map(|index| decode_update(bytes, 30 + index * 16))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            player,
            command_sequence,
            outcome,
            corrections,
        })
    }
}

pub fn encode_replication(update: AuthoritativeBlockUpdate) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(20);
    bytes.extend_from_slice(UPDATE_MAGIC);
    encode_update(&mut bytes, update);
    bytes
}

pub fn project_committed_blocks(
    report: &LocalTickReport,
    player: StableEntityId,
    registries: Option<&JavaTerrainRegistryMap>,
) -> Result<CommittedBlockProjection, BlockReplicationError> {
    let mut results = Vec::new();
    let mut replicated = BTreeMap::new();
    for commit in report.commits() {
        for entry in commit.journal().entries() {
            if is_entry(entry, JournalDomain::Replication, BLOCK_UPDATE_PATH) {
                let update = decode_replication(entry.payload())?;
                replicated.insert(update.position, update.state);
            } else if is_entry(entry, JournalDomain::Mutation, BLOCK_RESULT_PATH) {
                let result = BlockCommandResult::decode(entry.payload())?;
                if result.player == player {
                    results.push(result);
                }
            }
        }
    }
    let packets = aggregate_updates(replicated, registries)?;
    Ok(CommittedBlockProjection { results, packets })
}

pub fn project_authoritative_updates(
    updates: impl IntoIterator<Item = AuthoritativeBlockUpdate>,
    registries: &JavaTerrainRegistryMap,
) -> Result<Vec<PlayClientboundPacket>, BlockReplicationError> {
    let updates = updates
        .into_iter()
        .map(|update| (update.position, update.state))
        .collect();
    aggregate_updates(updates, Some(registries))
}

fn aggregate_updates(
    updates: BTreeMap<BlockPos, BlockStateId>,
    registries: Option<&JavaTerrainRegistryMap>,
) -> Result<Vec<PlayClientboundPacket>, BlockReplicationError> {
    if updates.is_empty() {
        return Ok(Vec::new());
    }
    let registries = registries.ok_or(BlockReplicationError::MissingRegistryMap)?;
    let mut sections = BTreeMap::<SectionPos, Vec<AuthoritativeBlockUpdate>>::new();
    for (position, state) in updates {
        sections
            .entry(position.section())
            .or_default()
            .push(AuthoritativeBlockUpdate { position, state });
    }
    let mut packets = Vec::new();
    for (section, updates) in sections {
        if let [update] = updates.as_slice() {
            packets.push(PlayClientboundPacket::BlockUpdate(BlockUpdate {
                position: update.position,
                state: registries.block_state(update.state)?,
            }));
            continue;
        }
        let changes = updates
            .into_iter()
            .map(
                |update| -> Result<SectionBlockChange, TerrainProjectionError> {
                    let local = update.position.local();
                    Ok(SectionBlockChange {
                        relative_position: (u16::from(local.x()) << 8)
                            | (u16::from(local.z()) << 4)
                            | u16::from(local.y()),
                        state: Some(registries.block_state(update.state)?),
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;
        packets.push(PlayClientboundPacket::SectionBlocksUpdate(
            SectionBlocksUpdate { section, changes },
        ));
    }
    Ok(packets)
}

fn decode_replication(bytes: &[u8]) -> Result<AuthoritativeBlockUpdate, BlockReplicationError> {
    if bytes.len() != 20 || &bytes[..4] != UPDATE_MAGIC {
        return Err(BlockReplicationError::InvalidUpdate);
    }
    decode_update(bytes, 4)
}

fn encode_update(bytes: &mut Vec<u8>, update: AuthoritativeBlockUpdate) {
    bytes.extend_from_slice(&update.position.x.to_be_bytes());
    bytes.extend_from_slice(&update.position.y.to_be_bytes());
    bytes.extend_from_slice(&update.position.z.to_be_bytes());
    bytes.extend_from_slice(&update.state.get().to_be_bytes());
}

fn decode_update(
    bytes: &[u8],
    offset: usize,
) -> Result<AuthoritativeBlockUpdate, BlockReplicationError> {
    Ok(AuthoritativeBlockUpdate {
        position: BlockPos::new(
            read_i32(bytes, offset)?,
            read_i32(bytes, offset + 4)?,
            read_i32(bytes, offset + 8)?,
        ),
        state: BlockStateId::new(read_u32(bytes, offset + 12)?),
    })
}

fn is_entry(entry: &JournalEntry, domain: JournalDomain, path: &str) -> bool {
    entry.domain() == domain && entry.kind().namespace() == "ferrite" && entry.kind().path() == path
}

const fn outcome_tag(outcome: BlockCommandOutcome) -> u8 {
    match outcome {
        BlockCommandOutcome::Applied => 0,
        BlockCommandOutcome::Rejected => 1,
        BlockCommandOutcome::Tracking => 2,
        BlockCommandOutcome::Cleared => 3,
    }
}

fn decode_outcome(value: u8) -> Result<BlockCommandOutcome, BlockReplicationError> {
    match value {
        0 => Ok(BlockCommandOutcome::Applied),
        1 => Ok(BlockCommandOutcome::Rejected),
        2 => Ok(BlockCommandOutcome::Tracking),
        3 => Ok(BlockCommandOutcome::Cleared),
        _ => Err(BlockReplicationError::InvalidResult),
    }
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32, BlockReplicationError> {
    Ok(i32::from_be_bytes(read_array(bytes, offset)?))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, BlockReplicationError> {
    Ok(u32::from_be_bytes(read_array(bytes, offset)?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, BlockReplicationError> {
    Ok(u64::from_be_bytes(read_array(bytes, offset)?))
}

fn read_u128(bytes: &[u8], offset: usize) -> Result<u128, BlockReplicationError> {
    Ok(u128::from_be_bytes(read_array(bytes, offset)?))
}

fn read_array<const N: usize>(
    bytes: &[u8],
    offset: usize,
) -> Result<[u8; N], BlockReplicationError> {
    bytes
        .get(offset..offset + N)
        .and_then(|slice| slice.try_into().ok())
        .ok_or(BlockReplicationError::InvalidResult)
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BlockReplicationError {
    #[error("committed block command result is invalid")]
    InvalidResult,
    #[error("committed block replication update is invalid")]
    InvalidUpdate,
    #[error("Java terrain registry map is required for block-state projection")]
    MissingRegistryMap,
    #[error(transparent)]
    Terrain(#[from] TerrainProjectionError),
}
