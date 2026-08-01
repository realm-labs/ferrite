#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundReconfigurationPacketKind {
    ConfigurationAcknowledged,
}

impl ServerboundReconfigurationPacketKind {
    pub const ALL: [Self; 1] = [Self::ConfigurationAcknowledged];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        16
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        "minecraft:configuration_acknowledged"
    }
}
