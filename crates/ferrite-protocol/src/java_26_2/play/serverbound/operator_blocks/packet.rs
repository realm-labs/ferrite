#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorBlockPacketKind {
    JigsawGenerate,
    SetCommandBlock,
    SetCommandMinecart,
    SetJigsawBlock,
    SetStructureBlock,
    SetTestBlock,
    TestInstanceBlockAction,
}

impl OperatorBlockPacketKind {
    pub const ALL: [Self; 7] = [
        Self::JigsawGenerate,
        Self::SetCommandBlock,
        Self::SetCommandMinecart,
        Self::SetJigsawBlock,
        Self::SetStructureBlock,
        Self::SetTestBlock,
        Self::TestInstanceBlockAction,
    ];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::JigsawGenerate => 27,
            Self::SetCommandBlock => 54,
            Self::SetCommandMinecart => 55,
            Self::SetJigsawBlock => 58,
            Self::SetStructureBlock => 59,
            Self::SetTestBlock => 60,
            Self::TestInstanceBlockAction => 65,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::JigsawGenerate => "minecraft:jigsaw_generate",
            Self::SetCommandBlock => "minecraft:set_command_block",
            Self::SetCommandMinecart => "minecraft:set_command_minecart",
            Self::SetJigsawBlock => "minecraft:set_jigsaw_block",
            Self::SetStructureBlock => "minecraft:set_structure_block",
            Self::SetTestBlock => "minecraft:set_test_block",
            Self::TestInstanceBlockAction => "minecraft:test_instance_block_action",
        }
    }

    #[must_use]
    pub const fn is_command_tool(self) -> bool {
        matches!(self, Self::SetCommandBlock | Self::SetCommandMinecart)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBlockMode {
    Sequence,
    Auto,
    Redstone,
}

impl CommandBlockMode {
    #[must_use]
    pub const fn from_strict_raw(raw_id: i32) -> Option<Self> {
        match raw_id {
            0 => Some(Self::Sequence),
            1 => Some(Self::Auto),
            2 => Some(Self::Redstone),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureUpdate {
    UpdateData,
    SaveArea,
    LoadArea,
    ScanArea,
}

impl StructureUpdate {
    #[must_use]
    pub const fn from_strict_raw(raw_id: i32) -> Option<Self> {
        match raw_id {
            0 => Some(Self::UpdateData),
            1 => Some(Self::SaveArea),
            2 => Some(Self::LoadArea),
            3 => Some(Self::ScanArea),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureMode {
    Save,
    Load,
    Corner,
    Data,
}

impl StructureMode {
    #[must_use]
    pub const fn from_strict_raw(raw_id: i32) -> Option<Self> {
        match raw_id {
            0 => Some(Self::Save),
            1 => Some(Self::Load),
            2 => Some(Self::Corner),
            3 => Some(Self::Data),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawJoint {
    Rollable,
    Aligned,
}

impl JigsawJoint {
    #[must_use]
    pub fn from_fallback_name(name: &str) -> Self {
        if name == "rollable" {
            Self::Rollable
        } else {
            Self::Aligned
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

impl TestBlockMode {
    #[must_use]
    pub const fn from_zero_fallback_raw(raw_id: i32) -> Self {
        match raw_id {
            1 => Self::Log,
            2 => Self::Fail,
            3 => Self::Accept,
            _ => Self::Start,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestInstanceAction {
    Init,
    Query,
    Set,
    Reset,
    Save,
    Export,
    Run,
}

impl TestInstanceAction {
    #[must_use]
    pub const fn from_zero_fallback_raw(raw_id: i32) -> Self {
        match raw_id {
            1 => Self::Query,
            2 => Self::Set,
            3 => Self::Reset,
            4 => Self::Save,
            5 => Self::Export,
            6 => Self::Run,
            _ => Self::Init,
        }
    }

    #[must_use]
    pub const fn installs_data(self) -> bool {
        !matches!(self, Self::Init | Self::Query)
    }
}

#[must_use]
pub const fn clamp_structure_offset(value: i8) -> i8 {
    if value < -48 {
        -48
    } else if value > 48 {
        48
    } else {
        value
    }
}

#[must_use]
pub const fn clamp_structure_size(value: i8) -> i8 {
    if value < 0 {
        0
    } else if value > 48 {
        48
    } else {
        value
    }
}

#[must_use]
pub fn clamp_structure_integrity(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorBlockRequest {
    JigsawGenerate {
        target_matches: bool,
        levels: i32,
        keep_jigsaws: bool,
    },
    SetCommandBlock {
        target_matches: bool,
        command_nonempty: bool,
        track_output: bool,
        command_blocks_enabled: bool,
    },
    SetCommandMinecart {
        target_matches: bool,
        command_nonempty: bool,
        track_output: bool,
        command_blocks_enabled: bool,
    },
    SetJigsawBlock {
        target_matches: bool,
    },
    SetStructureBlock {
        target_matches: bool,
        update: StructureUpdate,
        name_valid: bool,
        operation_succeeded: bool,
    },
    SetTestBlock {
        target_matches: bool,
    },
    TestInstanceBlockAction {
        target_matches: bool,
        action: TestInstanceAction,
        test_key_resolves: bool,
        operation_succeeded: bool,
    },
}

impl OperatorBlockRequest {
    #[must_use]
    pub const fn kind(self) -> OperatorBlockPacketKind {
        match self {
            Self::JigsawGenerate { .. } => OperatorBlockPacketKind::JigsawGenerate,
            Self::SetCommandBlock { .. } => OperatorBlockPacketKind::SetCommandBlock,
            Self::SetCommandMinecart { .. } => OperatorBlockPacketKind::SetCommandMinecart,
            Self::SetJigsawBlock { .. } => OperatorBlockPacketKind::SetJigsawBlock,
            Self::SetStructureBlock { .. } => OperatorBlockPacketKind::SetStructureBlock,
            Self::SetTestBlock { .. } => OperatorBlockPacketKind::SetTestBlock,
            Self::TestInstanceBlockAction { .. } => {
                OperatorBlockPacketKind::TestInstanceBlockAction
            }
        }
    }

    #[must_use]
    pub const fn target_matches(self) -> bool {
        match self {
            Self::JigsawGenerate { target_matches, .. }
            | Self::SetCommandBlock { target_matches, .. }
            | Self::SetCommandMinecart { target_matches, .. }
            | Self::SetJigsawBlock { target_matches }
            | Self::SetStructureBlock { target_matches, .. }
            | Self::SetTestBlock { target_matches }
            | Self::TestInstanceBlockAction { target_matches, .. } => target_matches,
        }
    }
}
