use ferrite_foundation::identity::{StableEntityId, StableIdError};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{SnapshotError, SnapshotRecord, SnapshotRecordKind};
use thiserror::Error;

use crate::player_service::model::{PlayerPayload, PlayerPayloadError, PlayerPersistentState};

const PLAYER_MAGIC: &[u8; 4] = b"F6P1";
// This Goal 01 identity is persisted. G03-P1-B3 owns its versioned migration.
const LEGACY_PLAYER_DOMAIN: &str = "phase6/player_v1";

#[must_use]
pub fn player_domain() -> ResourceId {
    ResourceId::new("ferrite", LEGACY_PLAYER_DOMAIN)
        .expect("static legacy player continuity domain is valid")
}

pub fn encode_player(
    player: StableEntityId,
    state: &PlayerPersistentState,
) -> Result<SnapshotRecord, ContinuityError> {
    let mut value =
        Vec::with_capacity(57 + state.inventory.bytes().len() + state.progression.bytes().len());
    value.extend_from_slice(PLAYER_MAGIC);
    value.extend_from_slice(&state.inventory_revision.to_be_bytes());
    encode_payload(&mut value, &state.inventory)?;
    value.push(state.selected_slot);
    value.extend_from_slice(&state.experience_points.to_be_bytes());
    value.extend_from_slice(&state.experience_level.to_be_bytes());
    value.extend_from_slice(&state.food_level.to_be_bytes());
    value.extend_from_slice(&state.saturation_bits.to_be_bytes());
    value.extend_from_slice(&state.exhaustion_bits.to_be_bytes());
    encode_payload(&mut value, &state.progression)?;
    value.extend_from_slice(&state.last_action_sequence.to_be_bytes());
    value.extend_from_slice(&state.last_session_epoch.to_be_bytes());
    SnapshotRecord::new(
        SnapshotRecordKind::Entity,
        player_domain(),
        player.to_be_bytes().to_vec(),
        value,
    )
    .map_err(ContinuityError::Snapshot)
}

pub fn decode_player(
    record: &SnapshotRecord,
) -> Result<Option<(StableEntityId, PlayerPersistentState)>, ContinuityError> {
    if record.kind() != SnapshotRecordKind::Entity || record.domain() != &player_domain() {
        return Ok(None);
    }
    let player_bytes: [u8; 16] = record
        .key()
        .try_into()
        .map_err(|_| ContinuityError::InvalidPlayerKey)?;
    let player = StableEntityId::new(u128::from_be_bytes(player_bytes))?;
    let mut cursor = Cursor::new(record.value());
    cursor.expect(PLAYER_MAGIC)?;
    let state = PlayerPersistentState {
        inventory_revision: cursor.u64()?,
        inventory: cursor.payload()?,
        selected_slot: cursor.u8()?,
        experience_points: cursor.u32()?,
        experience_level: cursor.u32()?,
        food_level: cursor.i32()?,
        saturation_bits: cursor.u32()?,
        exhaustion_bits: cursor.u32()?,
        progression: cursor.payload()?,
        last_action_sequence: cursor.u64()?,
        last_session_epoch: cursor.u64()?,
    };
    cursor.finish()?;
    validate_state(&state)?;
    Ok(Some((player, state)))
}

fn encode_payload(output: &mut Vec<u8>, payload: &PlayerPayload) -> Result<(), ContinuityError> {
    let length =
        u32::try_from(payload.bytes().len()).map_err(|_| ContinuityError::PayloadLengthOverflow)?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(payload.bytes());
    Ok(())
}

pub fn validate_state(state: &PlayerPersistentState) -> Result<(), ContinuityError> {
    if state.selected_slot > 8 {
        return Err(ContinuityError::InvalidSelectedSlot(state.selected_slot));
    }
    if !(0..=20).contains(&state.food_level) {
        return Err(ContinuityError::InvalidFoodLevel(state.food_level));
    }
    let saturation = f32::from_bits(state.saturation_bits);
    let exhaustion = f32::from_bits(state.exhaustion_bits);
    if !saturation.is_finite() || saturation < 0.0 {
        return Err(ContinuityError::InvalidSaturation);
    }
    if !exhaustion.is_finite() || exhaustion < 0.0 {
        return Err(ContinuityError::InvalidExhaustion);
    }
    Ok(())
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

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], ContinuityError> {
        self.take(N)?
            .try_into()
            .map_err(|_| ContinuityError::Truncated)
    }

    fn u8(&mut self) -> Result<u8, ContinuityError> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, ContinuityError> {
        Ok(u32::from_be_bytes(self.fixed()?))
    }

    fn i32(&mut self) -> Result<i32, ContinuityError> {
        Ok(i32::from_be_bytes(self.fixed()?))
    }

    fn u64(&mut self) -> Result<u64, ContinuityError> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    fn payload(&mut self) -> Result<PlayerPayload, ContinuityError> {
        let length = self.u32()? as usize;
        PlayerPayload::new(self.take(length)?.to_vec()).map_err(Into::into)
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
    #[error("player-service continuity has the wrong magic")]
    WrongMagic,
    #[error("player-service continuity is truncated")]
    Truncated,
    #[error("player-service continuity has trailing bytes")]
    TrailingBytes,
    #[error("player-service continuity has an invalid player key")]
    InvalidPlayerKey,
    #[error("player-service payload length exceeds the encoded integer range")]
    PayloadLengthOverflow,
    #[error("player-service selected slot {0} is outside 0..=8")]
    InvalidSelectedSlot(u8),
    #[error("player-service food level {0} is outside 0..=20")]
    InvalidFoodLevel(i32),
    #[error("player-service saturation is negative or non-finite")]
    InvalidSaturation,
    #[error("player-service exhaustion is negative or non-finite")]
    InvalidExhaustion,
    #[error(transparent)]
    StableId(#[from] StableIdError),
    #[error(transparent)]
    Payload(#[from] PlayerPayloadError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
}
