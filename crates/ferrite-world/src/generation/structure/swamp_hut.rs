//! Terrain-aligned swamp-hut geometry, supports, and occupant latches.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::{
    HorizontalDirection, OrientedPiece, PiecePlacement, PieceWorld,
};
use crate::generation::structure::processor::StructureState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwampHutOccupant {
    Witch,
    Cat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwampHutSpawn {
    pub occupant: SwampHutOccupant,
    pub position: BlockPos,
    pub persistent: bool,
    pub finalize_structure_spawn: bool,
    pub force_black_cat: bool,
}

pub trait SwampHutWorld: PieceWorld {
    fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32;

    fn minimum_y(&self) -> i32;

    fn spawn_swamp_hut_occupant(&mut self, request: SwampHutSpawn);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwampHutPiece {
    pub piece: OrientedPiece,
    pub average_ground_height: i32,
    pub witch_spawned: bool,
    pub cat_spawned: bool,
}

impl SwampHutPiece {
    pub fn new(chunk_minimum: BlockPos, orientation: HorizontalDirection) -> Self {
        Self {
            piece: OrientedPiece::from_anchor(
                BlockPos::new(chunk_minimum.x, 64, chunk_minimum.z),
                BlockPos::new(0, 0, 0),
                [7, 7, 9],
                orientation,
            ),
            average_ground_height: -1,
            witch_spawned: false,
            cat_spawned: false,
        }
    }

    pub fn place(&mut self, world: &mut impl SwampHutWorld, clip: &BlockBox) -> bool {
        if self.average_ground_height < 0 && !self.align_to_terrain(world, clip) {
            return false;
        }
        let placement = PiecePlacement {
            piece: self.piece,
            clip,
        };
        place_cabin(world, placement);
        let minimum_y = world.minimum_y();
        for z in [2, 7] {
            for x in [1, 5] {
                placement.fill_column_down(
                    world,
                    BlockPos::new(x, -1, z),
                    StructureState::new("minecraft:oak_log"),
                    minimum_y,
                    |state, fluid| {
                        !fluid.is_empty()
                            || matches!(
                                state.block.as_str(),
                                "minecraft:air"
                                    | "minecraft:glow_lichen"
                                    | "minecraft:seagrass"
                                    | "minecraft:tall_seagrass"
                            )
                    },
                );
            }
        }
        let spawn_position = self.piece.world_position(BlockPos::new(2, 2, 5));
        if clip.contains(spawn_position) {
            if !self.witch_spawned {
                self.witch_spawned = true;
                world.spawn_swamp_hut_occupant(spawn_request(
                    SwampHutOccupant::Witch,
                    spawn_position,
                ));
            }
            if !self.cat_spawned {
                self.cat_spawned = true;
                world
                    .spawn_swamp_hut_occupant(spawn_request(SwampHutOccupant::Cat, spawn_position));
            }
        }
        true
    }

    fn align_to_terrain(&mut self, world: &mut impl SwampHutWorld, clip: &BlockBox) -> bool {
        let mut sum = 0_i64;
        let mut count = 0_i64;
        for z in self.piece.bounds.minimum.z..=self.piece.bounds.maximum.z {
            for x in self.piece.bounds.minimum.x..=self.piece.bounds.maximum.x {
                let probe = BlockPos::new(x, 64, z);
                if clip.contains(probe) {
                    sum += i64::from(world.motion_blocking_no_leaves_height(x, z));
                    count += 1;
                }
            }
        }
        if count == 0 {
            return false;
        }
        let average = i32::try_from(sum / count).expect("mean of i32 heights fits i32");
        self.average_ground_height = average;
        self.piece.bounds =
            self.piece
                .bounds
                .moved([0, average.wrapping_sub(self.piece.bounds.minimum.y), 0]);
        true
    }
}

fn place_cabin(world: &mut impl PieceWorld, placement: PiecePlacement<'_>) {
    let planks = StructureState::new("minecraft:spruce_planks");
    for (minimum, maximum) in [
        ((1, 1, 1), (5, 1, 7)),
        ((1, 4, 2), (5, 4, 7)),
        ((2, 1, 0), (4, 1, 0)),
        ((2, 2, 2), (3, 3, 2)),
        ((1, 2, 3), (1, 3, 6)),
        ((5, 2, 3), (5, 3, 6)),
        ((2, 2, 7), (4, 3, 7)),
    ] {
        placement.fill_box(
            world,
            tuple_position(minimum),
            tuple_position(maximum),
            false,
            |_, _| planks.clone(),
        );
    }
    let logs = StructureState::new("minecraft:oak_log");
    for (minimum, maximum) in [
        ((1, 0, 2), (1, 3, 2)),
        ((5, 0, 2), (5, 3, 2)),
        ((1, 0, 7), (1, 3, 7)),
        ((5, 0, 7), (5, 3, 7)),
    ] {
        placement.fill_box(
            world,
            tuple_position(minimum),
            tuple_position(maximum),
            false,
            |_, _| logs.clone(),
        );
    }
    for (position, block) in [
        ((2, 3, 2), "minecraft:oak_fence"),
        ((3, 3, 7), "minecraft:oak_fence"),
        ((1, 3, 4), "minecraft:air"),
        ((5, 3, 4), "minecraft:air"),
        ((5, 3, 5), "minecraft:air"),
        ((1, 3, 5), "minecraft:potted_red_mushroom"),
        ((3, 2, 6), "minecraft:crafting_table"),
        ((4, 2, 6), "minecraft:cauldron"),
        ((1, 2, 1), "minecraft:oak_fence"),
        ((5, 2, 1), "minecraft:oak_fence"),
    ] {
        placement.place_block(world, tuple_position(position), StructureState::new(block));
    }
    for x in 0..=6 {
        placement.place_block(world, BlockPos::new(x, 4, 1), stair("north", "straight"));
    }
    for z in 2..=7 {
        placement.place_block(world, BlockPos::new(0, 4, z), stair("east", "straight"));
        placement.place_block(world, BlockPos::new(6, 4, z), stair("west", "straight"));
    }
    for x in 0..=6 {
        placement.place_block(world, BlockPos::new(x, 4, 8), stair("south", "straight"));
    }
    for (position, facing, shape) in [
        ((0, 4, 1), "north", "outer_right"),
        ((6, 4, 1), "north", "outer_left"),
        ((0, 4, 8), "south", "outer_left"),
        ((6, 4, 8), "south", "outer_right"),
    ] {
        placement.place_block(world, tuple_position(position), stair(facing, shape));
    }
}

fn stair(facing: &str, shape: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:spruce_stairs");
    state.properties.insert("facing".into(), facing.into());
    state.properties.insert("shape".into(), shape.into());
    state
}

fn spawn_request(occupant: SwampHutOccupant, position: BlockPos) -> SwampHutSpawn {
    SwampHutSpawn {
        occupant,
        position,
        persistent: true,
        finalize_structure_spawn: true,
        force_black_cat: occupant == SwampHutOccupant::Cat,
    }
}

fn tuple_position((x, y, z): (i32, i32, i32)) -> BlockPos {
    BlockPos::new(x, y, z)
}
