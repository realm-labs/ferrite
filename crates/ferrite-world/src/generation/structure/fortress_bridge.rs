//! Nether-fortress bridge-family geometry, throne, and terminal filler.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};
use crate::generation::structure::BlockBox;
use crate::generation::structure::fortress_graph::{FortressPiece, FortressPieceKind};
use crate::generation::structure::fortress_place::{
    FortressWorld, air, fence, fill_column_down, generate_box, nether_bricks, place, placement,
};
use crate::generation::structure::piece::{PiecePlacement, PieceWorld};
use crate::generation::structure::processor::StructureState;

pub(crate) fn place_bridge_piece(
    world: &mut impl FortressWorld,
    piece: &mut FortressPiece,
    clip: &BlockBox,
    random: &mut impl GenerationRandom,
) {
    let placement = placement(piece, clip);
    match piece.kind {
        FortressPieceKind::Start | FortressPieceKind::BridgeCrossing => {
            bridge_crossing(world, placement)
        }
        FortressPieceKind::BridgeStraight => bridge_straight(world, placement),
        FortressPieceKind::RoomCrossing => room_crossing(world, placement),
        FortressPieceKind::StairsRoom => stairs_room(world, placement),
        FortressPieceKind::MonsterThrone => monster_throne(world, piece, clip, random),
        FortressPieceKind::BridgeEndFiller => {
            bridge_end_filler(world, placement, piece.filler_seed)
        }
        _ => unreachable!("castle pieces use fortress_castle"),
    }
}

fn bridge_straight(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    b(world, p, (0, 3, 0), (4, 4, 18), nether_bricks());
    b(world, p, (1, 5, 0), (3, 7, 18), air());
    b(world, p, (0, 5, 0), (0, 5, 18), nether_bricks());
    b(world, p, (4, 5, 0), (4, 5, 18), nether_bricks());
    b(world, p, (0, 2, 0), (4, 2, 5), nether_bricks());
    b(world, p, (0, 2, 13), (4, 2, 18), nether_bricks());
    b(world, p, (0, 0, 0), (4, 1, 3), nether_bricks());
    b(world, p, (0, 0, 15), (4, 1, 18), nether_bricks());
    for x in 0..=4 {
        for z in 0..=2 {
            fill_column_down(world, p, x, z);
            fill_column_down(world, p, x, 18 - z);
        }
    }
    let nse = fence(&["north", "south", "east"]);
    let nsw = fence(&["north", "south", "west"]);
    for (x, z, y0, y1, state) in [
        (0, 1, 1, 4, nse.clone()),
        (0, 4, 3, 4, nse.clone()),
        (0, 14, 3, 4, nse.clone()),
        (0, 17, 1, 4, nse),
        (4, 1, 1, 4, nsw.clone()),
        (4, 4, 3, 4, nsw.clone()),
        (4, 14, 3, 4, nsw.clone()),
        (4, 17, 1, 4, nsw),
    ] {
        b(world, p, (x, y0, z), (x, y1, z), state);
    }
}

fn bridge_crossing(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    for (minimum, maximum) in [((7, 3, 0), (11, 4, 18)), ((0, 3, 7), (18, 4, 11))] {
        b(world, p, minimum, maximum, nether_bricks());
    }
    b(world, p, (8, 5, 0), (10, 7, 18), air());
    b(world, p, (0, 5, 8), (18, 7, 10), air());
    for (minimum, maximum) in [
        ((7, 5, 0), (7, 5, 7)),
        ((7, 5, 11), (7, 5, 18)),
        ((11, 5, 0), (11, 5, 7)),
        ((11, 5, 11), (11, 5, 18)),
        ((0, 5, 7), (7, 5, 7)),
        ((11, 5, 7), (18, 5, 7)),
        ((0, 5, 11), (7, 5, 11)),
        ((11, 5, 11), (18, 5, 11)),
        ((7, 2, 0), (11, 2, 5)),
        ((7, 2, 13), (11, 2, 18)),
        ((7, 0, 0), (11, 1, 3)),
        ((7, 0, 15), (11, 1, 18)),
    ] {
        b(world, p, minimum, maximum, nether_bricks());
    }
    for x in 7..=11 {
        for z in 0..=2 {
            fill_column_down(world, p, x, z);
            fill_column_down(world, p, x, 18 - z);
        }
    }
    for (minimum, maximum) in [
        ((0, 2, 7), (5, 2, 11)),
        ((13, 2, 7), (18, 2, 11)),
        ((0, 0, 7), (3, 1, 11)),
        ((15, 0, 7), (18, 1, 11)),
    ] {
        b(world, p, minimum, maximum, nether_bricks());
    }
    for x in 0..=2 {
        for z in 7..=11 {
            fill_column_down(world, p, x, z);
            fill_column_down(world, p, 18 - x, z);
        }
    }
}

fn room_crossing(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    b(world, p, (0, 0, 0), (6, 1, 6), nether_bricks());
    b(world, p, (0, 2, 0), (6, 7, 6), air());
    for (minimum, maximum) in [
        ((0, 2, 0), (1, 6, 0)),
        ((0, 2, 6), (1, 6, 6)),
        ((5, 2, 0), (6, 6, 0)),
        ((5, 2, 6), (6, 6, 6)),
        ((0, 2, 0), (0, 6, 1)),
        ((0, 2, 5), (0, 6, 6)),
        ((6, 2, 0), (6, 6, 1)),
        ((6, 2, 5), (6, 6, 6)),
        ((2, 6, 0), (4, 6, 0)),
        ((2, 6, 6), (4, 6, 6)),
        ((0, 6, 2), (0, 6, 4)),
        ((6, 6, 2), (6, 6, 4)),
    ] {
        b(world, p, minimum, maximum, nether_bricks());
    }
    let east_west = fence(&["west", "east"]);
    let north_south = fence(&["north", "south"]);
    b(world, p, (2, 5, 0), (4, 5, 0), east_west.clone());
    b(world, p, (2, 5, 6), (4, 5, 6), east_west);
    b(world, p, (0, 5, 2), (0, 5, 4), north_south.clone());
    b(world, p, (6, 5, 2), (6, 5, 4), north_south);
    support_rectangle(world, p, 0..=6, 0..=6);
}

fn stairs_room(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    b(world, p, (0, 0, 0), (6, 1, 6), nether_bricks());
    b(world, p, (0, 2, 0), (6, 10, 6), air());
    for (minimum, maximum) in [
        ((0, 2, 0), (1, 8, 0)),
        ((5, 2, 0), (6, 8, 0)),
        ((0, 2, 1), (0, 8, 6)),
        ((6, 2, 1), (6, 8, 6)),
        ((1, 2, 6), (5, 8, 6)),
    ] {
        b(world, p, minimum, maximum, nether_bricks());
    }
    b(world, p, (0, 3, 2), (0, 5, 4), fence(&["north", "south"]));
    b(world, p, (6, 3, 2), (6, 5, 2), fence(&["north", "south"]));
    b(world, p, (6, 3, 4), (6, 5, 4), fence(&["north", "south"]));
    for (x, maximum_y) in [(5, 2), (4, 3), (3, 4), (2, 5), (1, 6)] {
        b(world, p, (x, 2, 5), (x, maximum_y, 5), nether_bricks());
    }
    b(world, p, (1, 7, 1), (5, 7, 4), nether_bricks());
    b(world, p, (6, 8, 2), (6, 8, 4), air());
    b(world, p, (2, 6, 0), (4, 8, 0), nether_bricks());
    b(world, p, (2, 5, 0), (4, 5, 0), fence(&["west", "east"]));
    support_rectangle(world, p, 0..=6, 0..=6);
}

fn monster_throne(
    world: &mut impl FortressWorld,
    piece: &mut FortressPiece,
    clip: &BlockBox,
    random: &mut impl GenerationRandom,
) {
    let p = placement(piece, clip);
    b(world, p, (0, 2, 0), (6, 7, 7), air());
    b(world, p, (1, 0, 0), (5, 1, 7), nether_bricks());
    b(world, p, (1, 2, 1), (5, 2, 7), nether_bricks());
    b(world, p, (1, 3, 2), (5, 3, 7), nether_bricks());
    b(world, p, (1, 4, 3), (5, 4, 7), nether_bricks());
    for (minimum, maximum) in [
        ((1, 2, 0), (1, 4, 2)),
        ((5, 2, 0), (5, 4, 2)),
        ((1, 5, 2), (1, 5, 3)),
        ((5, 5, 2), (5, 5, 3)),
        ((0, 5, 3), (0, 5, 8)),
        ((6, 5, 3), (6, 5, 8)),
        ((1, 5, 8), (5, 5, 8)),
    ] {
        b(world, p, minimum, maximum, nether_bricks());
    }
    for (local, state) in [
        ((1, 6, 3), fence(&["west"])),
        ((5, 6, 3), fence(&["east"])),
        ((0, 6, 3), fence(&["east", "north"])),
        ((6, 6, 3), fence(&["west", "north"])),
        ((0, 6, 8), fence(&["east", "south"])),
        ((6, 6, 8), fence(&["west", "south"])),
        ((1, 7, 8), fence(&["east"])),
        ((5, 7, 8), fence(&["west"])),
        ((2, 8, 8), fence(&["east"])),
        ((3, 8, 8), fence(&["west", "east"])),
        ((4, 8, 8), fence(&["west"])),
    ] {
        place(world, p, tuple_pos(local), state);
    }
    b(world, p, (0, 6, 4), (0, 6, 7), fence(&["north", "south"]));
    b(world, p, (6, 6, 4), (6, 6, 7), fence(&["north", "south"]));
    b(world, p, (1, 6, 8), (5, 6, 8), fence(&["west", "east"]));
    b(world, p, (2, 7, 8), (4, 7, 8), fence(&["west", "east"]));
    let spawner = p.piece.world_position(BlockPos::new(3, 5, 5));
    if !piece.spawner_placed && clip.contains(spawner) {
        piece.spawner_placed = true;
        PieceWorld::set_state(world, spawner, StructureState::new("minecraft:spawner"), 2);
        if world.is_blaze_spawner_block_entity(spawner) {
            world.configure_blaze_spawner(spawner, random);
        }
    }
    support_rectangle(world, p, 0..=6, 0..=6);
}

fn bridge_end_filler(world: &mut impl FortressWorld, p: PiecePlacement<'_>, seed: i32) {
    let mut random = LegacyRandom::new(i64::from(seed));
    for x in 0..=4 {
        for y in 3..=4 {
            let z = bounded(&mut random, 8) as i32;
            b(world, p, (x, y, 0), (x, y, z), nether_bricks());
        }
    }
    let z = bounded(&mut random, 8) as i32;
    b(world, p, (0, 5, 0), (0, 5, z), nether_bricks());
    let z = bounded(&mut random, 8) as i32;
    b(world, p, (4, 5, 0), (4, 5, z), nether_bricks());
    for x in 0..=4 {
        let z = bounded(&mut random, 5) as i32;
        b(world, p, (x, 2, 0), (x, 2, z), nether_bricks());
    }
    for x in 0..=4 {
        for y in 0..=1 {
            let z = bounded(&mut random, 3) as i32;
            b(world, p, (x, y, 0), (x, y, z), nether_bricks());
        }
    }
}

fn support_rectangle(
    world: &mut impl FortressWorld,
    p: PiecePlacement<'_>,
    xs: impl Iterator<Item = i32> + Clone,
    zs: impl Iterator<Item = i32> + Clone,
) {
    for x in xs {
        for z in zs.clone() {
            fill_column_down(world, p, x, z);
        }
    }
}

fn b(
    world: &mut impl FortressWorld,
    p: PiecePlacement<'_>,
    minimum: (i32, i32, i32),
    maximum: (i32, i32, i32),
    state: StructureState,
) {
    generate_box(world, p, tuple_pos(minimum), tuple_pos(maximum), state);
}

fn tuple_pos(value: (i32, i32, i32)) -> BlockPos {
    BlockPos::new(value.0, value.1, value.2)
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("positive filler bound"))
}
