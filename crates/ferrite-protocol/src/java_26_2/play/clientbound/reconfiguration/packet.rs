#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconfigurationPacketKind {
    StartConfiguration,
}

impl ReconfigurationPacketKind {
    pub const ALL: [Self; 1] = [Self::StartConfiguration];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::StartConfiguration => 118,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::StartConfiguration => "minecraft:start_configuration",
        }
    }
}
