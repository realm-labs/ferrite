use ferrite_foundation::coordinate::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountScreenOpen {
    pub container_id: i32,
    pub inventory_columns: i32,
    pub entity_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionHand {
    Main,
    Off,
}

impl InteractionHand {
    #[must_use]
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::Main => 0,
            Self::Off => 1,
        }
    }

    #[must_use]
    pub const fn from_ordinal(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Main),
            1 => Some(Self::Off),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenSignEditor {
    pub position: BlockPos,
    pub front_text: bool,
}
