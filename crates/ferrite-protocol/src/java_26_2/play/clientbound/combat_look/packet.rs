use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnchor {
    Feet,
    Eyes,
}

impl EntityAnchor {
    #[must_use]
    pub const fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            0 => Some(Self::Feet),
            1 => Some(Self::Eyes),
            _ => None,
        }
    }

    #[must_use]
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::Feet => 0,
            Self::Eyes => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerCombatEnd {
    pub duration: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerCombatKill {
    pub player_entity_id: i32,
    pub message: TextComponentNbt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LookPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookEntity {
    pub entity_id: i32,
    pub anchor: EntityAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerLookAt {
    pub from_anchor: EntityAnchor,
    pub fallback: LookPosition,
    pub entity: Option<LookEntity>,
}
