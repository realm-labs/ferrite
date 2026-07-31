//! Shared structure placement geometry and jigsaw expansion.

pub mod block_tags;
pub mod buried_treasure;
pub mod desert_pyramid;
pub mod desert_pyramid_archaeology;
pub mod end_city;
pub mod end_city_graph;
mod fortress_bridge;
mod fortress_castle;
pub mod fortress_catalog;
pub mod fortress_graph;
pub mod fortress_place;
pub mod igloo;
pub mod jigsaw;
pub mod jungle_temple;
pub mod mansion_catalog;
pub mod mansion_graph;
pub mod mansion_pieces;
mod mansion_roof;
mod mansion_rooms;
pub mod mansion_runtime;
pub mod mineshaft_corridor;
pub mod mineshaft_graph;
pub mod mineshaft_place;
mod monument_building;
pub mod monument_catalog;
pub mod monument_graph;
pub mod monument_place;
mod monument_rooms;
mod monument_shell_front;
mod monument_shell_walls;
mod monument_special;
pub mod nbt;
pub mod nether_fossil;
pub mod ocean_ruin;
pub mod payload_audit;
pub mod piece;
pub mod pool_catalog;
pub mod pool_place;
pub mod processor;
pub mod processor_catalog;
pub mod records;
pub mod ruined_portal;
pub mod shipwreck;
pub mod stronghold_catalog;
pub mod stronghold_graph;
pub mod stronghold_place;
mod stronghold_rooms_basic;
mod stronghold_rooms_special;
pub mod swamp_hut;
pub mod template;
pub mod template_manager;
pub mod template_place;

use ferrite_foundation::coordinate::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockBox {
    pub minimum: BlockPos,
    pub maximum: BlockPos,
}

impl BlockBox {
    pub fn new(minimum: BlockPos, maximum: BlockPos) -> Option<Self> {
        (minimum.x <= maximum.x && minimum.y <= maximum.y && minimum.z <= maximum.z)
            .then_some(Self { minimum, maximum })
    }

    pub fn point(position: BlockPos) -> Self {
        Self {
            minimum: position,
            maximum: position,
        }
    }

    pub fn moved(self, offset: [i32; 3]) -> Self {
        Self {
            minimum: offset_position(self.minimum, offset),
            maximum: offset_position(self.maximum, offset),
        }
    }

    pub fn contains(self, position: BlockPos) -> bool {
        position.x >= self.minimum.x
            && position.x <= self.maximum.x
            && position.y >= self.minimum.y
            && position.y <= self.maximum.y
            && position.z >= self.minimum.z
            && position.z <= self.maximum.z
    }

    pub fn contains_box(self, other: Self) -> bool {
        self.contains(other.minimum) && self.contains(other.maximum)
    }

    pub fn intersects(self, other: Self) -> bool {
        self.minimum.x <= other.maximum.x
            && self.maximum.x >= other.minimum.x
            && self.minimum.y <= other.maximum.y
            && self.maximum.y >= other.minimum.y
            && self.minimum.z <= other.maximum.z
            && self.maximum.z >= other.minimum.z
    }

    pub fn center(self) -> BlockPos {
        BlockPos::new(
            self.minimum.x.wrapping_add(self.maximum.x) / 2,
            self.minimum.y.wrapping_add(self.maximum.y) / 2,
            self.minimum.z.wrapping_add(self.maximum.z) / 2,
        )
    }

    pub fn size(self) -> [i32; 3] {
        [
            self.maximum.x.wrapping_sub(self.minimum.x).wrapping_add(1),
            self.maximum.y.wrapping_sub(self.minimum.y).wrapping_add(1),
            self.maximum.z.wrapping_sub(self.minimum.z).wrapping_add(1),
        ]
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            minimum: BlockPos::new(
                self.minimum.x.min(other.minimum.x),
                self.minimum.y.min(other.minimum.y),
                self.minimum.z.min(other.minimum.z),
            ),
            maximum: BlockPos::new(
                self.maximum.x.max(other.maximum.x),
                self.maximum.y.max(other.maximum.y),
                self.maximum.z.max(other.maximum.z),
            ),
        }
    }
}

pub(crate) fn offset_position(position: BlockPos, offset: [i32; 3]) -> BlockPos {
    BlockPos::new(
        position.x.wrapping_add(offset[0]),
        position.y.wrapping_add(offset[1]),
        position.z.wrapping_add(offset[2]),
    )
}
