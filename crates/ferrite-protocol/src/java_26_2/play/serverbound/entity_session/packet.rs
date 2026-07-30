use crate::java_26_2::play::serverbound::packet::Hand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attack {
    pub target_entity_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientCommand {
    pub action: ClientCommandKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientCommandKind {
    PerformRespawn,
    RequestStats,
    RequestGameruleValues,
}

impl ClientCommandKind {
    pub const fn from_index(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::PerformRespawn),
            1 => Some(Self::RequestStats),
            2 => Some(Self::RequestGameruleValues),
            _ => None,
        }
    }

    #[must_use]
    pub const fn index(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LowPrecisionVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl LowPrecisionVector {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interact {
    pub target_entity_id: i32,
    pub hand: Hand,
    pub location: LowPrecisionVector,
    pub secondary_action: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickItemFromEntity {
    pub target_entity_id: i32,
    pub include_data: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpectatorAction {
    pub target_entity_id: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeleportToEntity {
    pub target_uuid: u128,
}
