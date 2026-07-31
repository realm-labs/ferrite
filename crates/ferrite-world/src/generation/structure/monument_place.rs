//! Shared ocean-monument placement primitives and child dispatch.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::monument_building::place_monument_building;
use crate::generation::structure::monument_graph::{
    MonumentChild, MonumentGraph, MonumentPieceKind, MonumentRoom,
};
use crate::generation::structure::monument_rooms::place_room_piece;
use crate::generation::structure::monument_special::place_special_piece;
use crate::generation::structure::piece::{OrientedPiece, PiecePlacement, PieceWorld};
use crate::generation::structure::processor::StructureState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonumentElderSpawn {
    pub position: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub healed_to_maximum: bool,
    pub reason_structure: bool,
    pub finalize_with_local_difficulty: bool,
    pub include_passengers: bool,
}

pub trait MonumentWorld: PieceWorld {
    fn sea_level(&self) -> i32;

    fn minimum_y(&self) -> i32;

    fn monument_support_replaceable(&mut self, position: BlockPos, state: &StructureState) -> bool;

    fn spawn_elder_guardian(&mut self, request: MonumentElderSpawn);
}

pub fn place_monument_child(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    clip: &BlockBox,
    random: &mut impl GenerationRandom,
) {
    if child.kind == MonumentPieceKind::Building {
        place_monument_building(world, graph, clip, random);
        return;
    }
    match child.kind {
        MonumentPieceKind::Entry
        | MonumentPieceKind::Core
        | MonumentPieceKind::Wing
        | MonumentPieceKind::Penthouse => place_special_piece(world, graph, child, clip),
        MonumentPieceKind::DoubleX
        | MonumentPieceKind::DoubleXY
        | MonumentPieceKind::DoubleY
        | MonumentPieceKind::DoubleYZ
        | MonumentPieceKind::DoubleZ
        | MonumentPieceKind::Simple
        | MonumentPieceKind::SimpleTop => place_room_piece(world, graph, child, clip, random),
        MonumentPieceKind::Building => unreachable!("building dispatched before child match"),
    }
}

pub(crate) fn placement<'a>(child: &MonumentChild, clip: &'a BlockBox) -> PiecePlacement<'a> {
    PiecePlacement {
        piece: OrientedPiece {
            bounds: child.bounding_box,
            orientation: child.orientation,
        },
        clip,
    }
}

pub(crate) fn solid_box(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    minimum: BlockPos,
    maximum: BlockPos,
    state: StructureState,
) {
    for y in minimum.y..=maximum.y {
        for x in minimum.x..=maximum.x {
            for z in minimum.z..=maximum.z {
                p.place_block(world, BlockPos::new(x, y, z), state.clone());
            }
        }
    }
}

pub(crate) fn water_box(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    minimum: BlockPos,
    maximum: BlockPos,
) {
    for y in minimum.y..=maximum.y {
        for x in minimum.x..=maximum.x {
            for z in minimum.z..=maximum.z {
                let local = BlockPos::new(x, y, z);
                let position = p.piece.world_position(local);
                let state = if p.clip.contains(position) {
                    PieceWorld::state_at(world, position)
                } else {
                    air()
                };
                if keep_during_water_fill(&state) {
                    continue;
                }
                let replacement = if position.y >= world.sea_level() {
                    air()
                } else {
                    water()
                };
                p.place_block(world, local, replacement);
            }
        }
    }
}

pub(crate) fn fill_only_box(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    minimum: BlockPos,
    maximum: BlockPos,
    state: StructureState,
) {
    for y in minimum.y..=maximum.y {
        for x in minimum.x..=maximum.x {
            for z in minimum.z..=maximum.z {
                let local = BlockPos::new(x, y, z);
                let position = p.piece.world_position(local);
                let existing = if p.clip.contains(position) {
                    PieceWorld::state_at(world, position)
                } else {
                    air()
                };
                if canonical_water(&existing) {
                    p.place_block(world, local, state.clone());
                }
            }
        }
    }
}

pub(crate) fn default_floor(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    x: i32,
    z: i32,
    down_opening: bool,
) {
    if !down_opening {
        solid_box(
            world,
            p,
            BlockPos::new(x, 0, z),
            BlockPos::new(x + 7, 0, z + 7),
            prismarine(),
        );
        return;
    }
    for (minimum, maximum, state) in [
        ((x, 0, z), (x + 2, 0, z + 7), prismarine()),
        ((x + 5, 0, z), (x + 7, 0, z + 7), prismarine()),
        ((x + 3, 0, z), (x + 4, 0, z + 2), prismarine()),
        ((x + 3, 0, z + 5), (x + 4, 0, z + 7), prismarine()),
        ((x + 3, 0, z + 2), (x + 4, 0, z + 2), bricks()),
        ((x + 3, 0, z + 5), (x + 4, 0, z + 5), bricks()),
        ((x + 2, 0, z + 3), (x + 2, 0, z + 4), bricks()),
        ((x + 5, 0, z + 3), (x + 5, 0, z + 4), bricks()),
    ] {
        box_tuple(world, p, minimum, maximum, state);
    }
}

pub(crate) fn fill_column_down(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    x: i32,
    z: i32,
) {
    let mut position = p.piece.world_position(BlockPos::new(x, -1, z));
    if !p.clip.contains(position) {
        return;
    }
    loop {
        let state = PieceWorld::state_at(world, position);
        if position.y <= world.minimum_y() + 1
            || !world.monument_support_replaceable(position, &state)
        {
            break;
        }
        PieceWorld::set_state(world, position, bricks(), 2);
        position.y -= 1;
    }
}

pub(crate) fn room<'a>(graph: &'a MonumentGraph, child: &MonumentChild) -> &'a MonumentRoom {
    &graph.rooms[child.room.expect("room piece has a room")]
}

pub(crate) fn connected_room<'a>(
    graph: &'a MonumentGraph,
    room: &MonumentRoom,
    direction: usize,
) -> Option<&'a MonumentRoom> {
    room.connections[direction].map(|slot| &graph.rooms[slot])
}

pub(crate) fn spawn_elder(world: &mut impl MonumentWorld, p: PiecePlacement<'_>, local: BlockPos) {
    let position = p.piece.world_position(local);
    if !p.clip.contains(position) {
        return;
    }
    world.spawn_elder_guardian(MonumentElderSpawn {
        position: [
            f64::from(position.x) + 0.5,
            f64::from(position.y),
            f64::from(position.z) + 0.5,
        ],
        yaw: 0.0,
        pitch: 0.0,
        healed_to_maximum: true,
        reason_structure: true,
        finalize_with_local_difficulty: true,
        include_passengers: true,
    });
}

pub(crate) fn prismarine() -> StructureState {
    StructureState::new("minecraft:prismarine")
}

pub(crate) fn bricks() -> StructureState {
    StructureState::new("minecraft:prismarine_bricks")
}

pub(crate) fn dark() -> StructureState {
    StructureState::new("minecraft:dark_prismarine")
}

pub(crate) fn lantern() -> StructureState {
    StructureState::new("minecraft:sea_lantern")
}

pub(crate) fn water() -> StructureState {
    StructureState::new("minecraft:water")
}

pub(crate) fn air() -> StructureState {
    StructureState::new("minecraft:air")
}

pub(crate) fn wet_sponge() -> StructureState {
    StructureState::new("minecraft:wet_sponge")
}

pub(crate) fn gold() -> StructureState {
    StructureState::new("minecraft:gold_block")
}

pub(crate) fn box_tuple(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    minimum: (i32, i32, i32),
    maximum: (i32, i32, i32),
    state: StructureState,
) {
    solid_box(world, p, tuple_pos(minimum), tuple_pos(maximum), state);
}

pub(crate) fn tuple_pos(value: (i32, i32, i32)) -> BlockPos {
    BlockPos::new(value.0, value.1, value.2)
}

fn keep_during_water_fill(state: &StructureState) -> bool {
    matches!(
        state.block.as_str(),
        "minecraft:ice" | "minecraft:packed_ice" | "minecraft:blue_ice" | "minecraft:water"
    )
}

fn canonical_water(state: &StructureState) -> bool {
    state.block == "minecraft:water"
        && state
            .properties
            .get("level")
            .is_none_or(|level| level == "0")
}
