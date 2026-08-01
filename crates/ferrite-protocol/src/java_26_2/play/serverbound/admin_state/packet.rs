#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminStatePacketKind {
    BlockEntityTagQuery,
    ChangeDifficulty,
    ChangeGameMode,
    EntityTagQuery,
    LockDifficulty,
    SetCreativeModeSlot,
    SetGameRule,
}

impl AdminStatePacketKind {
    pub const ALL: [Self; 7] = [
        Self::BlockEntityTagQuery,
        Self::ChangeDifficulty,
        Self::ChangeGameMode,
        Self::EntityTagQuery,
        Self::LockDifficulty,
        Self::SetCreativeModeSlot,
        Self::SetGameRule,
    ];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::BlockEntityTagQuery => 2,
            Self::ChangeDifficulty => 4,
            Self::ChangeGameMode => 5,
            Self::EntityTagQuery => 25,
            Self::LockDifficulty => 29,
            Self::SetCreativeModeSlot => 56,
            Self::SetGameRule => 57,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::BlockEntityTagQuery => "minecraft:block_entity_tag_query",
            Self::ChangeDifficulty => "minecraft:change_difficulty",
            Self::ChangeGameMode => "minecraft:change_game_mode",
            Self::EntityTagQuery => "minecraft:entity_tag_query",
            Self::LockDifficulty => "minecraft:lock_difficulty",
            Self::SetCreativeModeSlot => "minecraft:set_creative_mode_slot",
            Self::SetGameRule => "minecraft:set_game_rule",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    #[must_use]
    pub const fn from_wrapping_raw(raw_id: i32) -> Self {
        match raw_id.rem_euclid(4) {
            0 => Self::Peaceful,
            1 => Self::Easy,
            2 => Self::Normal,
            _ => Self::Hard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameMode {
    #[must_use]
    pub const fn from_zero_fallback_raw(raw_id: i32) -> Self {
        match raw_id {
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            _ => Self::Survival,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreativeStackClass {
    EmptyOrAir,
    Item,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminStateRequest {
    BlockEntityTagQuery {
        target_exists: bool,
    },
    ChangeDifficulty {
        requested: Difficulty,
    },
    ChangeGameMode {
        requested: GameMode,
    },
    EntityTagQuery {
        target_exists: bool,
    },
    LockDifficulty,
    SetCreativeModeSlot {
        slot: i16,
        stack: CreativeStackClass,
        feature_enabled: bool,
        count_within_maximum: bool,
        drop_throttle: i32,
    },
    SetGameRule,
}

impl AdminStateRequest {
    #[must_use]
    pub const fn kind(self) -> AdminStatePacketKind {
        match self {
            Self::BlockEntityTagQuery { .. } => AdminStatePacketKind::BlockEntityTagQuery,
            Self::ChangeDifficulty { .. } => AdminStatePacketKind::ChangeDifficulty,
            Self::ChangeGameMode { .. } => AdminStatePacketKind::ChangeGameMode,
            Self::EntityTagQuery { .. } => AdminStatePacketKind::EntityTagQuery,
            Self::LockDifficulty => AdminStatePacketKind::LockDifficulty,
            Self::SetCreativeModeSlot { .. } => AdminStatePacketKind::SetCreativeModeSlot,
            Self::SetGameRule => AdminStatePacketKind::SetGameRule,
        }
    }
}
