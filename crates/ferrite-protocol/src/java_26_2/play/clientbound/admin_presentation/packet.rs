use std::collections::BTreeMap;

use ferrite_foundation::coordinate::BlockPos;

use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminPresentationPacketKind {
    GameRuleValues,
    GameTestHighlightPosition,
    LowDiskSpaceWarning,
    TestInstanceBlockStatus,
}

impl AdminPresentationPacketKind {
    pub const ALL: [Self; 4] = [
        Self::GameRuleValues,
        Self::GameTestHighlightPosition,
        Self::LowDiskSpaceWarning,
        Self::TestInstanceBlockStatus,
    ];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::GameRuleValues => 39,
            Self::GameTestHighlightPosition => 40,
            Self::LowDiskSpaceWarning => 50,
            Self::TestInstanceBlockStatus => 126,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::GameRuleValues => "minecraft:game_rule_values",
            Self::GameTestHighlightPosition => "minecraft:game_test_highlight_pos",
            Self::LowDiskSpaceWarning => "minecraft:low_disk_space_warning",
            Self::TestInstanceBlockStatus => "minecraft:test_instance_block_status",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminPresentationPacket {
    GameRuleValues(BTreeMap<Identifier, String>),
    GameTestHighlightPosition {
        absolute: BlockPos,
        relative: BlockPos,
    },
    LowDiskSpaceWarning,
    TestInstanceBlockStatus {
        status: TextComponentNbt,
        size: Option<Vec3i>,
    },
}

impl AdminPresentationPacket {
    #[must_use]
    pub const fn kind(&self) -> AdminPresentationPacketKind {
        match self {
            Self::GameRuleValues(_) => AdminPresentationPacketKind::GameRuleValues,
            Self::GameTestHighlightPosition { .. } => {
                AdminPresentationPacketKind::GameTestHighlightPosition
            }
            Self::LowDiskSpaceWarning => AdminPresentationPacketKind::LowDiskSpaceWarning,
            Self::TestInstanceBlockStatus { .. } => {
                AdminPresentationPacketKind::TestInstanceBlockStatus
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
