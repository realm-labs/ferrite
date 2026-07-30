//! Protocol-neutral player target selection, attack, and use transactions.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

use crate::player::state::Vec3;

pub mod attack;
pub mod targeting;
pub mod use_action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Main,
    Off,
}

impl Hand {
    pub const ORDERED: [Self; 2] = [Self::Main, Self::Off];
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockHit {
    pub position: BlockPos,
    pub location: Vec3,
    pub face: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityHit {
    pub entity_id: u64,
    pub location: Vec3,
    pub relative_location: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitTarget {
    Miss { location: Vec3 },
    Block(BlockHit),
    Entity(EntityHit),
}

impl HitTarget {
    #[must_use]
    pub const fn location(self) -> Vec3 {
        match self {
            Self::Miss { location } => location,
            Self::Block(hit) => hit.location,
            Self::Entity(hit) => hit.location,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwingSource {
    None,
    Client,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackState {
    pub object_id: u64,
    pub item_id: u32,
    pub count: u32,
    pub damage: u32,
    pub use_duration: i32,
    pub feature_enabled: bool,
    pub on_cooldown: bool,
}

impl StackState {
    pub const EMPTY: Self = Self {
        object_id: 0,
        item_id: 0,
        count: 0,
        damage: 0,
        use_duration: 0,
        feature_enabled: true,
        on_cooldown: false,
    };

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.count == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemContext {
    None,
    ItemUsed { transformed: Option<StackState> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionResult {
    Success {
        swing: SwingSource,
        item: ItemContext,
    },
    Fail,
    Pass,
    TryEmptyHandInteraction,
}

impl InteractionResult {
    #[must_use]
    pub const fn consumes(self) -> bool {
        matches!(self, Self::Success { .. })
    }

    #[must_use]
    pub const fn swing_source(self) -> SwingSource {
        match self {
            Self::Success { swing, .. } => swing,
            Self::Fail | Self::Pass | Self::TryEmptyHandInteraction => SwingSource::None,
        }
    }

    #[must_use]
    pub const fn item_context(self) -> ItemContext {
        match self {
            Self::Success { item, .. } => item,
            Self::Fail | Self::Pass | Self::TryEmptyHandInteraction => ItemContext::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackMutation {
    Retain,
    Replace(StackState),
    Clear,
    RestoreCount(u32),
}
