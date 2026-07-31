//! Stronghold straight, prison, turn, source-stairs, and filler geometry.

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::piece::{HorizontalDirection, PiecePlacement};
use crate::generation::structure::stronghold_graph::{StrongholdPiece, StrongholdPieceKind};
use crate::generation::structure::stronghold_place::{
    StrongholdWorld, air, bars, box_, facing, pos, selector_box, shell, small_door, stone_bricks,
};

pub(crate) fn place_basic_piece(
    world: &mut impl StrongholdWorld,
    piece: &StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    match piece.kind {
        StrongholdPieceKind::Start => stairs_down(world, piece, p, random),
        StrongholdPieceKind::Straight => straight(world, piece, p, random),
        StrongholdPieceKind::PrisonHall => prison(world, piece, p, random),
        StrongholdPieceKind::LeftTurn | StrongholdPieceKind::RightTurn => {
            turn(world, piece, p, random)
        }
        StrongholdPieceKind::FillerCorridor => filler(world, piece, p),
        _ => unreachable!("special stronghold room dispatched as basic"),
    }
}

fn straight(
    world: &mut impl StrongholdWorld,
    piece: &StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    shell(world, p, pos(4, 4, 6), true, random);
    small_door(world, p, piece.entry_door, pos(1, 1, 0));
    box_(world, p, pos(1, 1, 6), pos(3, 3, 6), air());
    for (local, direction) in [
        (pos(1, 2, 1), "east"),
        (pos(3, 2, 1), "west"),
        (pos(1, 2, 5), "east"),
        (pos(3, 2, 5), "west"),
    ] {
        if random.next_f32() < 0.1 {
            p.place_block(world, local, facing("minecraft:wall_torch", direction));
        }
    }
    if piece.left_child {
        box_(world, p, pos(0, 1, 2), pos(0, 3, 4), air());
    }
    if piece.right_child {
        box_(world, p, pos(4, 1, 2), pos(4, 3, 4), air());
    }
}

fn prison(
    world: &mut impl StrongholdWorld,
    piece: &StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    shell(world, p, pos(8, 4, 10), true, random);
    small_door(world, p, piece.entry_door, pos(1, 1, 0));
    box_(world, p, pos(1, 1, 10), pos(3, 3, 10), air());
    for z in [1, 3, 7, 9] {
        selector_box(world, p, pos(4, 1, z), pos(4, 3, z), false, random);
    }
    let mut north_south = bars();
    north_south.properties.insert("north".into(), "true".into());
    north_south.properties.insert("south".into(), "true".into());
    let mut west_east = bars();
    west_east.properties.insert("west".into(), "true".into());
    west_east.properties.insert("east".into(), "true".into());
    let mut junction = north_south.clone();
    junction.properties.insert("east".into(), "true".into());
    for y in 1..=3 {
        p.place_block(world, pos(4, y, 4), north_south.clone());
        p.place_block(world, pos(4, y, 5), junction.clone());
        p.place_block(world, pos(4, y, 6), north_south.clone());
        for x in 5..=7 {
            p.place_block(world, pos(x, y, 5), west_east.clone());
        }
    }
    for z in [2, 8] {
        p.place_block(world, pos(4, 3, z), north_south.clone());
        p.place_block(world, pos(4, 1, z), facing("minecraft:iron_door", "west"));
        let mut upper = facing("minecraft:iron_door", "west");
        upper.properties.insert("half".into(), "upper".into());
        p.place_block(world, pos(4, 2, z), upper);
    }
}

fn turn(
    world: &mut impl StrongholdWorld,
    piece: &StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    shell(world, p, pos(4, 4, 4), true, random);
    small_door(world, p, piece.entry_door, pos(1, 1, 0));
    let opposite = matches!(
        piece.orientation,
        HorizontalDirection::South | HorizontalDirection::West
    );
    let open_left = (piece.kind == StrongholdPieceKind::LeftTurn) ^ opposite;
    if open_left {
        box_(world, p, pos(0, 1, 1), pos(0, 3, 3), air());
    } else {
        box_(world, p, pos(4, 1, 1), pos(4, 3, 3), air());
    }
}

pub(crate) fn stairs_down(
    world: &mut impl StrongholdWorld,
    piece: &StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    shell(world, p, pos(4, 10, 4), true, random);
    small_door(world, p, piece.entry_door, pos(1, 7, 0));
    box_(world, p, pos(1, 1, 4), pos(3, 3, 4), air());
    for (local, block) in [
        (pos(2, 6, 1), "minecraft:stone_bricks"),
        (pos(1, 5, 1), "minecraft:stone_bricks"),
        (pos(1, 6, 1), "minecraft:smooth_stone_slab"),
        (pos(1, 5, 2), "minecraft:stone_bricks"),
        (pos(1, 4, 3), "minecraft:stone_bricks"),
        (pos(1, 5, 3), "minecraft:smooth_stone_slab"),
        (pos(2, 4, 3), "minecraft:stone_bricks"),
        (pos(3, 3, 3), "minecraft:stone_bricks"),
        (pos(3, 4, 3), "minecraft:smooth_stone_slab"),
        (pos(3, 3, 2), "minecraft:stone_bricks"),
        (pos(3, 2, 1), "minecraft:stone_bricks"),
        (pos(3, 3, 1), "minecraft:smooth_stone_slab"),
        (pos(2, 2, 1), "minecraft:stone_bricks"),
        (pos(1, 1, 1), "minecraft:stone_bricks"),
        (pos(1, 2, 1), "minecraft:smooth_stone_slab"),
        (pos(1, 1, 2), "minecraft:stone_bricks"),
        (pos(1, 1, 3), "minecraft:smooth_stone_slab"),
    ] {
        p.place_block(
            world,
            local,
            crate::generation::structure::processor::StructureState::new(block),
        );
    }
}

fn filler(world: &mut impl StrongholdWorld, piece: &StrongholdPiece, p: PiecePlacement<'_>) {
    let steps = piece.filler_steps;
    for y in 0..=4 {
        for x in 0..=4 {
            for z in 0..steps {
                let edge = y == 0 || y == 4 || x == 0 || x == 4;
                p.place_block(
                    world,
                    pos(x, y, z),
                    if edge { stone_bricks() } else { air() },
                );
            }
        }
    }
}
