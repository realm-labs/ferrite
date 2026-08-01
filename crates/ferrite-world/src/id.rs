//! Domain-specific process-local world storage identities.

use crate::palette::PaletteEntry;

macro_rules! world_runtime_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            pub const fn new(value: u32) -> Self {
                Self(value)
            }

            pub const fn get(self) -> u32 {
                self.0
            }
        }

        impl PaletteEntry for $name {
            fn to_raw(self) -> u32 {
                self.0
            }

            fn from_raw(value: u32) -> Self {
                Self(value)
            }
        }
    };
}

world_runtime_id!(BlockStateId);
world_runtime_id!(BiomeId);

pub const AIR: BlockStateId = BlockStateId::new(0);
pub const STONE: BlockStateId = BlockStateId::new(1);
pub const GRASS_BLOCK: BlockStateId = BlockStateId::new(2);
pub const WATER: BlockStateId = BlockStateId::new(3);
pub const LAVA: BlockStateId = BlockStateId::new(4);
pub const FIRE: BlockStateId = BlockStateId::new(5);
pub const NETHERRACK: BlockStateId = BlockStateId::new(6);
pub const END_STONE: BlockStateId = BlockStateId::new(7);
pub const OBSIDIAN: BlockStateId = BlockStateId::new(8);
pub const NETHER_PORTAL_X: BlockStateId = BlockStateId::new(9);
pub const NETHER_PORTAL_Z: BlockStateId = BlockStateId::new(10);
pub const END_PORTAL: BlockStateId = BlockStateId::new(11);

#[must_use]
pub const fn light_opacity(state: BlockStateId) -> u8 {
    if state.get() == AIR.get()
        || state.get() == WATER.get()
        || state.get() == LAVA.get()
        || state.get() == FIRE.get()
        || state.get() == NETHER_PORTAL_X.get()
        || state.get() == NETHER_PORTAL_Z.get()
        || state.get() == END_PORTAL.get()
    {
        0
    } else {
        15
    }
}

#[must_use]
pub const fn light_emission(state: BlockStateId) -> u8 {
    if state.get() == LAVA.get() || state.get() == FIRE.get() {
        15
    } else {
        0
    }
}

#[must_use]
pub const fn has_empty_collision(state: BlockStateId) -> bool {
    state.get() == AIR.get()
        || state.get() == WATER.get()
        || state.get() == LAVA.get()
        || state.get() == FIRE.get()
        || state.get() == NETHER_PORTAL_X.get()
        || state.get() == NETHER_PORTAL_Z.get()
        || state.get() == END_PORTAL.get()
}
