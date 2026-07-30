use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::region::SimulationRegionKey;
use thiserror::Error;

pub const MAX_PLAYER_PAYLOAD_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerPayload {
    bytes: Box<[u8]>,
    digest: [u8; 32],
}

impl PlayerPayload {
    pub fn new(bytes: Vec<u8>) -> Result<Self, PlayerPayloadError> {
        if bytes.len() > MAX_PLAYER_PAYLOAD_BYTES {
            return Err(PlayerPayloadError::TooLarge {
                actual: bytes.len(),
                maximum: MAX_PLAYER_PAYLOAD_BYTES,
            });
        }
        Ok(Self {
            digest: *blake3::hash(&bytes).as_bytes(),
            bytes: bytes.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

impl Default for PlayerPayload {
    fn default() -> Self {
        Self::new(Vec::new()).expect("empty player payload is bounded")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PlayerPayloadError {
    #[error("player payload has {actual} bytes, exceeding {maximum}")]
    TooLarge { actual: usize, maximum: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerPersistentState {
    pub inventory_revision: u64,
    pub inventory: PlayerPayload,
    pub selected_slot: u8,
    pub experience_points: u32,
    pub experience_level: u32,
    pub food_level: i32,
    pub saturation_bits: u32,
    pub exhaustion_bits: u32,
    pub progression: PlayerPayload,
    pub last_action_sequence: u64,
    pub last_session_epoch: u64,
}

impl Default for PlayerPersistentState {
    fn default() -> Self {
        Self {
            inventory_revision: 0,
            inventory: PlayerPayload::default(),
            selected_slot: 0,
            experience_points: 0,
            experience_level: 0,
            food_level: 20,
            saturation_bits: 5.0_f32.to_bits(),
            exhaustion_bits: 0.0_f32.to_bits(),
            progression: PlayerPayload::default(),
            last_action_sequence: 0,
            last_session_epoch: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerMutation {
    pub expected_inventory_revision: u64,
    pub inventory: PlayerPayload,
    pub selected_slot: u8,
    pub experience_points: u32,
    pub experience_level: u32,
    pub food_level: i32,
    pub saturation_bits: u32,
    pub exhaustion_bits: u32,
    pub progression: PlayerPayload,
}

impl PlayerMutation {
    #[must_use]
    pub fn from_state(state: &PlayerPersistentState) -> Self {
        Self {
            expected_inventory_revision: state.inventory_revision,
            inventory: state.inventory.clone(),
            selected_slot: state.selected_slot,
            experience_points: state.experience_points,
            experience_level: state.experience_level,
            food_level: state.food_level,
            saturation_bits: state.saturation_bits,
            exhaustion_bits: state.exhaustion_bits,
            progression: state.progression.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerActionHeader {
    pub region: SimulationRegionKey,
    pub generation: ActivationGeneration,
    pub player: StableEntityId,
    pub session_epoch: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuLease {
    pub container_id: u8,
    pub state_id: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResyncReason {
    Join,
    Reload,
    InventoryRevision,
    MenuState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionKind {
    InventoryDelta {
        inventory_revision: u64,
    },
    MenuDelta {
        container_id: u8,
        state_id: u16,
        inventory_revision: u64,
    },
    FullState {
        reason: ResyncReason,
        inventory_revision: u64,
        menu: Option<MenuLease>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerProjection {
    pub revision: u64,
    pub player: StableEntityId,
    pub session_epoch: u64,
    pub kind: ProjectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOutcome {
    Committed {
        projection_revision: u64,
        full_resync: bool,
    },
    RejectedAndResynchronized {
        reason: ResyncReason,
        projection_revision: u64,
    },
    IgnoredWrongContainer,
    AlreadyApplied,
}
