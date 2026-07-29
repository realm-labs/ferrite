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
