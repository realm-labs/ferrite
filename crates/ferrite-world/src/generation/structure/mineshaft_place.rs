//! Shared mineshaft cancellation and room, crossing, and stairs placement.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::mineshaft_graph::{
    MineshaftCrossing, MineshaftPiece, MineshaftRoom, MineshaftStairs, MineshaftType,
};
use crate::generation::structure::piece::{FluidState, OrientedPiece, PiecePlacement, PieceWorld};
use crate::generation::structure::processor::StructureState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MineshaftChestCartSpawn {
    pub position: [f64; 3],
    pub rail_north_south: bool,
    pub creation_reason_chunk_generation: bool,
    pub loot_table: &'static str,
    pub loot_seed: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineshaftFace {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

pub trait MineshaftWorld: PieceWorld {
    fn mineshaft_blocking_biome(&mut self, position: BlockPos) -> bool;

    fn ocean_floor_height(&mut self, x: i32, z: i32) -> i32;

    fn structure_replaceable(&mut self, position: BlockPos, state: &StructureState) -> bool;

    fn sturdy_top(&mut self, position: BlockPos) -> bool;

    fn sturdy_face(&mut self, position: BlockPos, face: MineshaftFace) -> bool;

    fn supports_center_down(&mut self, position: BlockPos) -> bool;

    fn falling_block(&mut self, position: BlockPos) -> bool;

    fn minimum_y(&self) -> i32;

    fn maximum_y(&self) -> i32;

    fn create_mineshaft_chest_cart(&mut self, position: [f64; 3]) -> bool;

    fn spawn_mineshaft_chest_cart(&mut self, request: MineshaftChestCartSpawn);

    fn is_spawner_block_entity(&mut self, position: BlockPos) -> bool;

    fn configure_cave_spider_spawner(&mut self, position: BlockPos);
}

pub fn place_non_corridor(
    world: &mut impl MineshaftWorld,
    piece: &MineshaftPiece,
    kind: MineshaftType,
    clip: &BlockBox,
    _random: &mut impl GenerationRandom,
) -> bool {
    if invalid_location(world, piece.bounding_box(), clip) {
        return false;
    }
    match piece {
        MineshaftPiece::Room(room) => place_room(world, room, kind, clip),
        MineshaftPiece::Crossing(crossing) => place_crossing(world, crossing, kind, clip),
        MineshaftPiece::Stairs(stairs) => place_stairs(world, stairs, kind, clip),
        MineshaftPiece::Corridor(_) => return false,
    }
    true
}

pub fn invalid_location(
    world: &mut impl MineshaftWorld,
    bounding_box: BlockBox,
    clip: &BlockBox,
) -> bool {
    let expanded = BlockBox::new(
        BlockPos::new(
            bounding_box.minimum.x - 1,
            bounding_box.minimum.y - 1,
            bounding_box.minimum.z - 1,
        ),
        BlockPos::new(
            bounding_box.maximum.x + 1,
            bounding_box.maximum.y + 1,
            bounding_box.maximum.z + 1,
        ),
    )
    .expect("expanded mineshaft box is ordered");
    let Some(sample) = intersection(expanded, *clip) else {
        return true;
    };
    if world.mineshaft_blocking_biome(sample.center()) {
        return true;
    }
    for x in sample.minimum.x..=sample.maximum.x {
        for z in sample.minimum.z..=sample.maximum.z {
            if liquid(world, BlockPos::new(x, sample.minimum.y, z))
                || liquid(world, BlockPos::new(x, sample.maximum.y, z))
            {
                return true;
            }
        }
    }
    for x in sample.minimum.x..=sample.maximum.x {
        for y in sample.minimum.y..=sample.maximum.y {
            if liquid(world, BlockPos::new(x, y, sample.minimum.z))
                || liquid(world, BlockPos::new(x, y, sample.maximum.z))
            {
                return true;
            }
        }
    }
    for z in sample.minimum.z..=sample.maximum.z {
        for y in sample.minimum.y..=sample.maximum.y {
            if liquid(world, BlockPos::new(sample.minimum.x, y, z))
                || liquid(world, BlockPos::new(sample.maximum.x, y, z))
            {
                return true;
            }
        }
    }
    false
}

pub fn can_replace(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    position: BlockPos,
) -> bool {
    let state = PieceWorld::state_at(world, position);
    state.block != kind.planks()
        && state.block != kind.log()
        && state.block != kind.fence()
        && state.block != "minecraft:chain"
}

pub fn is_air(state: &StructureState) -> bool {
    matches!(
        state.block.as_str(),
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

pub fn interior(
    world: &mut impl MineshaftWorld,
    piece: OrientedPiece,
    clip: &BlockBox,
    local: BlockPos,
) -> bool {
    let above = piece.world_position(BlockPos::new(local.x, local.y + 1, local.z));
    clip.contains(above) && above.y < world.ocean_floor_height(above.x, above.z)
}

pub fn place_replacing(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    position: BlockPos,
    state: StructureState,
    clip: &BlockBox,
) -> bool {
    if !clip.contains(position) || !can_replace(world, kind, position) {
        return false;
    }
    let written = PieceWorld::set_state(world, position, state, 2);
    let fluid = PieceWorld::fluid_at(world, position);
    if !fluid.is_empty() {
        PieceWorld::schedule_fluid_tick(world, position, fluid, 0);
    }
    written
}

fn place_room(
    world: &mut impl MineshaftWorld,
    room: &MineshaftRoom,
    kind: MineshaftType,
    clip: &BlockBox,
) {
    let box_ = room.bounding_box;
    carve_box(
        world,
        kind,
        BlockBox::new(
            BlockPos::new(box_.minimum.x, box_.minimum.y + 1, box_.minimum.z),
            BlockPos::new(
                box_.maximum.x,
                (box_.minimum.y + 3).min(box_.maximum.y),
                box_.maximum.z,
            ),
        )
        .expect("room lower box is ordered"),
        clip,
    );
    for entrance in &room.entrances {
        carve_box(
            world,
            kind,
            BlockBox::new(
                BlockPos::new(
                    entrance.minimum.x,
                    entrance.maximum.y - 2,
                    entrance.minimum.z,
                ),
                entrance.maximum,
            )
            .expect("entrance top is ordered"),
            clip,
        );
    }
    let lower_y = box_.minimum.y + 4;
    if lower_y > box_.maximum.y {
        return;
    }
    let width = box_.size()[0] as f32;
    let height = (box_.maximum.y - lower_y + 1) as f32;
    let depth = box_.size()[2] as f32;
    let center_x = box_.minimum.x as f32 + width / 2.0;
    let center_z = box_.minimum.z as f32 + depth / 2.0;
    for y in lower_y..=box_.maximum.y {
        let normalized_y = (y - lower_y) as f32 / height;
        for x in box_.minimum.x..=box_.maximum.x {
            let normalized_x = (x as f32 - center_x) / (width * 0.5);
            for z in box_.minimum.z..=box_.maximum.z {
                let normalized_z = (z as f32 - center_z) / (depth * 0.5);
                if normalized_x * normalized_x
                    + normalized_y * normalized_y
                    + normalized_z * normalized_z
                    <= 1.05
                {
                    place_replacing(
                        world,
                        kind,
                        BlockPos::new(x, y, z),
                        StructureState::new("minecraft:cave_air"),
                        clip,
                    );
                }
            }
        }
    }
}

fn place_crossing(
    world: &mut impl MineshaftWorld,
    crossing: &MineshaftCrossing,
    kind: MineshaftType,
    clip: &BlockBox,
) {
    let box_ = crossing.bounding_box;
    if crossing.two_floored {
        for (minimum_y, maximum_y) in [
            (box_.minimum.y, box_.minimum.y + 2),
            (box_.maximum.y - 2, box_.maximum.y),
        ] {
            carve_cross(world, kind, box_, minimum_y, maximum_y, clip);
        }
        carve_box(
            world,
            kind,
            BlockBox::new(
                BlockPos::new(box_.minimum.x + 1, box_.minimum.y + 3, box_.minimum.z + 1),
                BlockPos::new(box_.maximum.x - 1, box_.minimum.y + 3, box_.maximum.z - 1),
            )
            .expect("crossing separator is ordered"),
            clip,
        );
    } else {
        carve_cross(world, kind, box_, box_.minimum.y, box_.maximum.y, clip);
    }
    for (x, z) in [
        (box_.minimum.x + 1, box_.minimum.z + 1),
        (box_.minimum.x + 1, box_.maximum.z - 1),
        (box_.maximum.x - 1, box_.minimum.z + 1),
        (box_.maximum.x - 1, box_.maximum.z - 1),
    ] {
        let above = BlockPos::new(x, box_.maximum.y + 1, z);
        if clip.contains(above) && !is_air(&PieceWorld::state_at(world, above)) {
            for y in box_.minimum.y..=box_.maximum.y {
                place_replacing(
                    world,
                    kind,
                    BlockPos::new(x, y, z),
                    StructureState::new(kind.planks()),
                    clip,
                );
            }
        }
    }
    for x in box_.minimum.x..=box_.maximum.x {
        for z in box_.minimum.z..=box_.maximum.z {
            let floor = BlockPos::new(x, box_.minimum.y - 1, z);
            if clip.contains(BlockPos::new(x, box_.minimum.y, z)) && !world.sturdy_top(floor) {
                PieceWorld::set_state(world, floor, StructureState::new(kind.planks()), 2);
            }
        }
    }
}

fn carve_cross(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    box_: BlockBox,
    minimum_y: i32,
    maximum_y: i32,
    clip: &BlockBox,
) {
    carve_box(
        world,
        kind,
        BlockBox::new(
            BlockPos::new(box_.minimum.x, minimum_y, box_.minimum.z + 1),
            BlockPos::new(box_.maximum.x, maximum_y, box_.maximum.z - 1),
        )
        .expect("east-west crossing box is ordered"),
        clip,
    );
    carve_box(
        world,
        kind,
        BlockBox::new(
            BlockPos::new(box_.minimum.x + 1, minimum_y, box_.minimum.z),
            BlockPos::new(box_.maximum.x - 1, maximum_y, box_.maximum.z),
        )
        .expect("north-south crossing box is ordered"),
        clip,
    );
}

fn place_stairs(
    world: &mut impl MineshaftWorld,
    stairs: &MineshaftStairs,
    kind: MineshaftType,
    clip: &BlockBox,
) {
    let piece = OrientedPiece {
        bounds: stairs.bounding_box,
        orientation: stairs.orientation,
    };
    let placement = PiecePlacement { piece, clip };
    carve_local_box(
        world,
        kind,
        placement,
        BlockPos::new(0, 5, 0),
        BlockPos::new(2, 7, 1),
    );
    carve_local_box(
        world,
        kind,
        placement,
        BlockPos::new(0, 0, 7),
        BlockPos::new(2, 2, 8),
    );
    for offset in 0..5 {
        carve_local_box(
            world,
            kind,
            placement,
            BlockPos::new(0, 5 - offset - i32::from(offset < 4), 2 + offset),
            BlockPos::new(2, 7 - offset, 2 + offset),
        );
    }
}

fn carve_local_box(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    placement: PiecePlacement<'_>,
    minimum: BlockPos,
    maximum: BlockPos,
) {
    for y in minimum.y..=maximum.y {
        for x in minimum.x..=maximum.x {
            for z in minimum.z..=maximum.z {
                let position = placement.piece.world_position(BlockPos::new(x, y, z));
                place_replacing(
                    world,
                    kind,
                    position,
                    StructureState::new("minecraft:cave_air"),
                    placement.clip,
                );
            }
        }
    }
}

fn carve_box(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    box_: BlockBox,
    clip: &BlockBox,
) {
    for y in box_.minimum.y..=box_.maximum.y {
        for x in box_.minimum.x..=box_.maximum.x {
            for z in box_.minimum.z..=box_.maximum.z {
                place_replacing(
                    world,
                    kind,
                    BlockPos::new(x, y, z),
                    StructureState::new("minecraft:cave_air"),
                    clip,
                );
            }
        }
    }
}

fn liquid(world: &mut impl MineshaftWorld, position: BlockPos) -> bool {
    PieceWorld::fluid_at(world, position) != FluidState::Empty
}

fn intersection(left: BlockBox, right: BlockBox) -> Option<BlockBox> {
    BlockBox::new(
        BlockPos::new(
            left.minimum.x.max(right.minimum.x),
            left.minimum.y.max(right.minimum.y),
            left.minimum.z.max(right.minimum.z),
        ),
        BlockPos::new(
            left.maximum.x.min(right.maximum.x),
            left.maximum.y.min(right.maximum.y),
            left.maximum.z.min(right.maximum.z),
        ),
    )
}
