//! Shared Nether-fortress placement transaction and special side effects.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::fortress_bridge::place_bridge_piece;
use crate::generation::structure::fortress_castle::place_castle_piece;
use crate::generation::structure::fortress_graph::{FortressPiece, FortressPieceKind};
use crate::generation::structure::piece::{
    FluidState, HorizontalDirection, OrientedPiece, PiecePlacement, PieceWorld,
};
use crate::generation::structure::processor::StructureState;

pub trait FortressWorld: PieceWorld {
    fn minimum_y(&self) -> i32;

    fn fortress_support_replaceable(&mut self, position: BlockPos, state: &StructureState) -> bool;

    fn is_blaze_spawner_block_entity(&mut self, position: BlockPos) -> bool;

    fn configure_blaze_spawner(&mut self, position: BlockPos, random: &mut impl GenerationRandom);
}

pub fn place_fortress_piece<R, F>(
    world: &mut impl FortressWorld,
    piece: &mut FortressPiece,
    clip: &BlockBox,
    random: &mut R,
    loot_seed: &mut F,
) where
    R: GenerationRandom,
    F: FnMut() -> i64,
{
    match piece.kind {
        FortressPieceKind::Start
        | FortressPieceKind::BridgeStraight
        | FortressPieceKind::BridgeCrossing
        | FortressPieceKind::RoomCrossing
        | FortressPieceKind::StairsRoom
        | FortressPieceKind::MonsterThrone
        | FortressPieceKind::BridgeEndFiller => place_bridge_piece(world, piece, clip, random),
        FortressPieceKind::CastleEntrance
        | FortressPieceKind::CastleSmallCorridor
        | FortressPieceKind::CastleSmallCrossing
        | FortressPieceKind::CastleRightTurn
        | FortressPieceKind::CastleLeftTurn
        | FortressPieceKind::CastleCorridorStairs
        | FortressPieceKind::CastleTBalcony
        | FortressPieceKind::CastleStalkRoom => {
            place_castle_piece(world, piece, clip, random, loot_seed)
        }
    }
}

pub(crate) fn placement<'a>(piece: &FortressPiece, clip: &'a BlockBox) -> PiecePlacement<'a> {
    PiecePlacement {
        piece: OrientedPiece {
            bounds: piece.bounding_box,
            orientation: piece.orientation,
        },
        clip,
    }
}

pub(crate) fn place(
    world: &mut impl FortressWorld,
    placement: PiecePlacement<'_>,
    local: BlockPos,
    mut state: StructureState,
) {
    let position = placement.piece.world_position(local);
    if !placement.clip.contains(position) {
        return;
    }
    transform_state(&mut state, placement.piece.orientation);
    let shape_sensitive = state.block == "minecraft:nether_brick_fence";
    PieceWorld::set_state(world, position, state, 2);
    let fluid = PieceWorld::fluid_at(world, position);
    if !fluid.is_empty() {
        PieceWorld::schedule_fluid_tick(world, position, fluid, 0);
    }
    if shape_sensitive {
        PieceWorld::mark_shape_postprocessing(world, position);
    }
}

pub(crate) fn generate_box(
    world: &mut impl FortressWorld,
    placement: PiecePlacement<'_>,
    minimum: BlockPos,
    maximum: BlockPos,
    state: StructureState,
) {
    for y in minimum.y..=maximum.y {
        for x in minimum.x..=maximum.x {
            for z in minimum.z..=maximum.z {
                place(world, placement, BlockPos::new(x, y, z), state.clone());
            }
        }
    }
}

pub(crate) fn fill_column_down(
    world: &mut impl FortressWorld,
    placement: PiecePlacement<'_>,
    x: i32,
    z: i32,
) {
    let mut position = placement.piece.world_position(BlockPos::new(x, -1, z));
    if !placement.clip.contains(position) {
        return;
    }
    loop {
        let state = PieceWorld::state_at(world, position);
        if position.y <= world.minimum_y() + 1
            || !world.fortress_support_replaceable(position, &state)
        {
            break;
        }
        PieceWorld::set_state(world, position, nether_bricks(), 2);
        position.y -= 1;
    }
}

pub(crate) fn create_turn_chest(
    world: &mut impl FortressWorld,
    piece: &mut FortressPiece,
    clip: &BlockBox,
    local: BlockPos,
    loot_seed: &mut impl FnMut() -> i64,
) {
    if !piece.chest_pending {
        return;
    }
    let position = placement(piece, clip).piece.world_position(local);
    if !clip.contains(position) {
        return;
    }
    piece.chest_pending = false;
    if PieceWorld::state_at(world, position).block == "minecraft:chest" {
        return;
    }
    let state = reoriented_chest(world, position);
    PieceWorld::set_state(world, position, state, 2);
    if PieceWorld::is_loot_container(world, position) {
        PieceWorld::install_loot(
            world,
            position,
            "minecraft:chests/nether_bridge",
            loot_seed(),
        );
    }
}

pub(crate) fn nether_bricks() -> StructureState {
    StructureState::new("minecraft:nether_bricks")
}

pub(crate) fn air() -> StructureState {
    StructureState::new("minecraft:air")
}

pub(crate) fn lava() -> StructureState {
    StructureState::new("minecraft:lava")
}

pub(crate) fn soul_sand() -> StructureState {
    StructureState::new("minecraft:soul_sand")
}

pub(crate) fn nether_wart() -> StructureState {
    StructureState::new("minecraft:nether_wart")
}

pub(crate) fn fence(directions: &[&str]) -> StructureState {
    let mut state = StructureState::new("minecraft:nether_brick_fence");
    for direction in directions {
        state
            .properties
            .insert((*direction).to_owned(), "true".to_owned());
    }
    state
}

pub(crate) fn stairs(facing: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:nether_brick_stairs");
    state.properties.insert("facing".into(), facing.into());
    state
}

fn reoriented_chest(world: &mut impl FortressWorld, position: BlockPos) -> StructureState {
    let directions = ["north", "east", "south", "west"];
    let mut solid_neighbor = None;
    for direction in directions {
        let neighbor = horizontal_neighbor(position, direction);
        let state = PieceWorld::state_at(world, neighbor);
        if state.block == "minecraft:chest" {
            return chest("north");
        }
        if PieceWorld::solid_render(world, neighbor) {
            if solid_neighbor.is_none() {
                solid_neighbor = Some(direction);
            } else {
                solid_neighbor = None;
                break;
            }
        }
    }
    if let Some(direction) = solid_neighbor {
        return chest(opposite(direction));
    }
    let mut facing = "north";
    if PieceWorld::solid_render(world, horizontal_neighbor(position, facing)) {
        facing = opposite(facing);
    }
    if PieceWorld::solid_render(world, horizontal_neighbor(position, facing)) {
        facing = clockwise(facing);
    }
    if PieceWorld::solid_render(world, horizontal_neighbor(position, facing)) {
        facing = opposite(facing);
    }
    chest(facing)
}

fn chest(facing: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:chest");
    state.properties.insert("facing".into(), facing.into());
    state
}

fn horizontal_neighbor(position: BlockPos, direction: &str) -> BlockPos {
    match direction {
        "north" => BlockPos::new(position.x, position.y, position.z - 1),
        "east" => BlockPos::new(position.x + 1, position.y, position.z),
        "south" => BlockPos::new(position.x, position.y, position.z + 1),
        "west" => BlockPos::new(position.x - 1, position.y, position.z),
        _ => unreachable!("horizontal direction"),
    }
}

fn opposite(direction: &str) -> &str {
    match direction {
        "north" => "south",
        "east" => "west",
        "south" => "north",
        "west" => "east",
        _ => unreachable!("horizontal direction"),
    }
}

fn clockwise(direction: &str) -> &str {
    match direction {
        "north" => "east",
        "east" => "south",
        "south" => "west",
        "west" => "north",
        _ => unreachable!("horizontal direction"),
    }
}

fn transform_state(state: &mut StructureState, orientation: HorizontalDirection) {
    if let Some(facing) = state.properties.get_mut("facing") {
        *facing = transform_direction(facing, orientation).to_owned();
    }
    let old = ["north", "east", "south", "west"]
        .map(|name| state.properties.remove(name).map(|value| (name, value)));
    for (direction, value) in old.into_iter().flatten() {
        state.properties.insert(
            transform_direction(direction, orientation).to_owned(),
            value,
        );
    }
}

fn transform_direction(direction: &str, orientation: HorizontalDirection) -> &str {
    match orientation {
        HorizontalDirection::North => direction,
        HorizontalDirection::South => match direction {
            "north" => "south",
            "south" => "north",
            _ => direction,
        },
        HorizontalDirection::West => match direction {
            "north" => "west",
            "east" => "south",
            "south" => "east",
            "west" => "north",
            _ => direction,
        },
        HorizontalDirection::East => match direction {
            "north" => "east",
            "east" => "south",
            "south" => "west",
            "west" => "north",
            _ => direction,
        },
    }
}

pub(crate) fn schedule_explicit_lava_tick(
    world: &mut impl FortressWorld,
    placement: PiecePlacement<'_>,
    local: BlockPos,
) {
    let position = placement.piece.world_position(local);
    if placement.clip.contains(position) {
        PieceWorld::schedule_fluid_tick(world, position, FluidState::Lava, 0);
    }
}
