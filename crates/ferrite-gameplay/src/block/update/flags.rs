//! Minecraft 26.2 block update bit meanings.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockUpdateFlags(u16);

impl BlockUpdateFlags {
    pub const UPDATE_NEIGHBORS: Self = Self(1);
    pub const UPDATE_CLIENTS: Self = Self(2);
    pub const UPDATE_INVISIBLE: Self = Self(4);
    pub const UPDATE_IMMEDIATE: Self = Self(8);
    pub const UPDATE_KNOWN_SHAPE: Self = Self(16);
    pub const UPDATE_SUPPRESS_DROPS: Self = Self(32);
    pub const UPDATE_MOVE_BY_PISTON: Self = Self(64);
    pub const UPDATE_SKIP_SHAPE_UPDATE_ON_WIRE: Self = Self(128);
    pub const UPDATE_SKIP_BLOCK_ENTITY_SIDE_EFFECTS: Self = Self(256);
    pub const UPDATE_SKIP_ON_PLACE: Self = Self(512);

    pub const UPDATE_NONE: Self = Self(260);
    pub const UPDATE_ALL: Self = Self(3);
    pub const UPDATE_ALL_IMMEDIATE: Self = Self(11);
    pub const UPDATE_SKIP_ALL_SIDE_EFFECTS: Self = Self(816);

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub const fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }

    pub const fn without(self, flags: Self) -> Self {
        Self(self.0 & !flags.0)
    }

    pub const fn nested_shape_flags(self) -> Self {
        self.without(Self(
            Self::UPDATE_NEIGHBORS.0 | Self::UPDATE_SUPPRESS_DROPS.0,
        ))
    }
}
