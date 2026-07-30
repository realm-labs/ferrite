use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

pub mod input;
pub mod mutation;
pub mod prediction;
pub mod session;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakingItem {
    pub object_id: u64,
    pub item_id: u32,
    pub components_fingerprint: u64,
    pub count: u32,
}

impl BreakingItem {
    #[must_use]
    pub const fn same_item_and_components(self, other: Self) -> bool {
        self.object_id == other.object_id
            || (self.item_id == other.item_id
                && self.components_fingerprint == other.components_fingerprint)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetState {
    pub is_air: bool,
    pub destroy_progress: f32,
    pub sound_volume: f32,
    pub sound_pitch: f32,
}

impl TargetState {
    pub const AIR: Self = Self {
        is_air: true,
        destroy_progress: 0.0,
        sound_volume: 0.0,
        sound_pitch: 0.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    StartDestroyBlock,
    StopDestroyBlock,
    AbortDestroyBlock,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientBreakEffect {
    SendCarriedSlot,
    TutorialProgress {
        position: BlockPos,
        progress: f32,
    },
    AttackBlock(BlockPos),
    BeginPrediction(i32),
    AttemptLocalDestroy(BlockPos),
    SendAction {
        action: PlayerAction,
        position: BlockPos,
        face: Direction,
        sequence: i32,
    },
    EndPrediction(i32),
    PublishCrack {
        position: BlockPos,
        stage: i32,
    },
    PlayHitSound {
        position: BlockPos,
        volume: f32,
        pitch: f32,
    },
    BreakingEffect(BlockPos),
    SwingMainHand,
    ResetAttackStrength,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientBreakOutcome {
    pub continued: bool,
    pub effects: Vec<ClientBreakEffect>,
}

impl ClientBreakOutcome {
    pub(crate) const fn rejected() -> Self {
        Self {
            continued: false,
            effects: Vec::new(),
        }
    }
}
