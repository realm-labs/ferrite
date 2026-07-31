//! Ocean-monument entry, core, wings, penthouse, gold, and elder transactions.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::structure::BlockBox;
use crate::generation::structure::monument_graph::{
    MonumentChild, MonumentDirection, MonumentGraph, MonumentPieceKind,
};
use crate::generation::structure::monument_place::{
    MonumentWorld, box_tuple, bricks, dark, fill_only_box, gold, lantern, placement, prismarine,
    room, spawn_elder, water_box,
};
use crate::generation::structure::piece::PiecePlacement;

pub(crate) fn place_special_piece(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    clip: &BlockBox,
) {
    let p = placement(child, clip);
    match child.kind {
        MonumentPieceKind::Entry => entry(world, graph, child, p),
        MonumentPieceKind::Core => core(world, p),
        MonumentPieceKind::Wing => wing(world, p, child.design & 1),
        MonumentPieceKind::Penthouse => penthouse(world, p),
        _ => unreachable!("ordinary rooms use monument_rooms"),
    }
}

fn entry(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    p: PiecePlacement<'_>,
) {
    for (minimum, maximum) in [
        ((0, 3, 0), (2, 3, 7)),
        ((5, 3, 0), (7, 3, 7)),
        ((0, 2, 0), (1, 2, 7)),
        ((6, 2, 0), (7, 2, 7)),
        ((0, 1, 0), (0, 1, 7)),
        ((7, 1, 0), (7, 1, 7)),
        ((0, 1, 7), (7, 3, 7)),
        ((1, 1, 0), (2, 3, 0)),
        ((5, 1, 0), (6, 3, 0)),
    ] {
        box_tuple(world, p, minimum, maximum, bricks());
    }
    let room = room(graph, child);
    if room.opening(MonumentDirection::North) {
        water_box(world, p, BlockPos::new(3, 1, 7), BlockPos::new(4, 2, 7));
    }
    if room.opening(MonumentDirection::West) {
        water_box(world, p, BlockPos::new(0, 1, 3), BlockPos::new(1, 2, 4));
    }
    if room.opening(MonumentDirection::East) {
        water_box(world, p, BlockPos::new(6, 1, 3), BlockPos::new(7, 2, 4));
    }
}

fn core(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    fill_only_box(
        world,
        p,
        BlockPos::new(1, 8, 0),
        BlockPos::new(14, 8, 14),
        prismarine(),
    );
    for (minimum, maximum) in [
        ((0, 7, 0), (0, 7, 15)),
        ((15, 7, 0), (15, 7, 15)),
        ((1, 7, 0), (15, 7, 0)),
        ((1, 7, 15), (14, 7, 15)),
    ] {
        box_tuple(world, p, minimum, maximum, bricks());
    }
    for y in 1..=6 {
        let state = if y == 2 || y == 6 {
            prismarine()
        } else {
            bricks()
        };
        for x in [0, 15] {
            box_tuple(world, p, (x, y, 0), (x, y, 1), state.clone());
            box_tuple(world, p, (x, y, 6), (x, y, 9), state.clone());
            box_tuple(world, p, (x, y, 14), (x, y, 15), state.clone());
        }
        box_tuple(world, p, (1, y, 0), (1, y, 0), state.clone());
        box_tuple(world, p, (6, y, 0), (9, y, 0), state.clone());
        box_tuple(world, p, (14, y, 0), (14, y, 0), state.clone());
        box_tuple(world, p, (1, y, 15), (14, y, 15), state);
    }
    box_tuple(world, p, (6, 3, 6), (9, 6, 9), dark());
    box_tuple(world, p, (7, 4, 7), (8, 5, 8), gold());
    for y in [3, 6] {
        for x in [6, 9] {
            for z in [6, 9] {
                p.place_block(world, BlockPos::new(x, y, z), lantern());
            }
        }
    }
    for (minimum, maximum) in [
        ((5, 1, 6), (5, 2, 6)),
        ((5, 1, 9), (5, 2, 9)),
        ((10, 1, 6), (10, 2, 6)),
        ((10, 1, 9), (10, 2, 9)),
        ((6, 1, 5), (6, 2, 5)),
        ((9, 1, 5), (9, 2, 5)),
        ((6, 1, 10), (6, 2, 10)),
        ((9, 1, 10), (9, 2, 10)),
        ((5, 2, 5), (5, 6, 5)),
        ((5, 2, 10), (5, 6, 10)),
        ((10, 2, 5), (10, 6, 5)),
        ((10, 2, 10), (10, 6, 10)),
        ((5, 7, 1), (5, 7, 6)),
        ((10, 7, 1), (10, 7, 6)),
        ((5, 7, 9), (5, 7, 14)),
        ((10, 7, 9), (10, 7, 14)),
        ((1, 7, 5), (6, 7, 5)),
        ((1, 7, 10), (6, 7, 10)),
        ((9, 7, 5), (14, 7, 5)),
        ((9, 7, 10), (14, 7, 10)),
        ((2, 1, 2), (2, 1, 3)),
        ((3, 1, 2), (3, 1, 2)),
        ((13, 1, 2), (13, 1, 3)),
        ((12, 1, 2), (12, 1, 2)),
        ((2, 1, 12), (2, 1, 13)),
        ((3, 1, 13), (3, 1, 13)),
        ((13, 1, 12), (13, 1, 13)),
        ((12, 1, 13), (12, 1, 13)),
    ] {
        box_tuple(world, p, minimum, maximum, bricks());
    }
}

fn wing(world: &mut impl MonumentWorld, p: PiecePlacement<'_>, design: i32) {
    if design == 0 {
        for i in 0..4 {
            box_tuple(
                world,
                p,
                (10 - i, 3 - i, 20 - i),
                (12 + i, 3 - i, 20),
                bricks(),
            );
        }
        for (minimum, maximum) in [
            ((7, 0, 6), (15, 0, 16)),
            ((6, 0, 6), (6, 3, 20)),
            ((16, 0, 6), (16, 3, 20)),
            ((7, 1, 7), (7, 1, 20)),
            ((15, 1, 7), (15, 1, 20)),
            ((7, 1, 6), (9, 3, 6)),
            ((13, 1, 6), (15, 3, 6)),
            ((8, 1, 7), (9, 1, 7)),
            ((13, 1, 7), (14, 1, 7)),
            ((9, 0, 5), (13, 0, 5)),
        ] {
            box_tuple(world, p, minimum, maximum, bricks());
        }
        box_tuple(world, p, (10, 0, 7), (12, 0, 7), dark());
        box_tuple(world, p, (8, 0, 10), (8, 0, 12), dark());
        box_tuple(world, p, (14, 0, 10), (14, 0, 12), dark());
        for z in (7..=18).rev().step_by(3) {
            p.place_block(world, BlockPos::new(6, 3, z), lantern());
            p.place_block(world, BlockPos::new(16, 3, z), lantern());
        }
        for local in [(10, 0, 10), (12, 0, 10), (10, 0, 12), (12, 0, 12)] {
            p.place_block(world, tuple(local), lantern());
        }
        for local in [(8, 3, 6), (14, 3, 6)] {
            p.place_block(world, tuple(local), lantern());
        }
        for (x, z) in [(4, 4), (18, 4), (4, 18), (18, 18)] {
            p.place_block(world, BlockPos::new(x, 2, z), bricks());
            p.place_block(world, BlockPos::new(x, 1, z), lantern());
            p.place_block(world, BlockPos::new(x, 0, z), bricks());
        }
        for local in [(9, 7, 20), (13, 7, 20)] {
            p.place_block(world, tuple(local), bricks());
        }
        box_tuple(world, p, (6, 0, 21), (7, 4, 21), bricks());
        box_tuple(world, p, (15, 0, 21), (16, 4, 21), bricks());
        spawn_elder(world, p, BlockPos::new(11, 2, 16));
    } else {
        box_tuple(world, p, (9, 3, 18), (13, 3, 20), bricks());
        box_tuple(world, p, (9, 0, 18), (9, 2, 18), bricks());
        box_tuple(world, p, (13, 0, 18), (13, 2, 18), bricks());
        for x in [9, 13] {
            p.place_block(world, BlockPos::new(x, 6, 20), bricks());
            p.place_block(world, BlockPos::new(x, 5, 20), lantern());
            p.place_block(world, BlockPos::new(x, 4, 20), bricks());
        }
        box_tuple(world, p, (7, 3, 7), (15, 3, 14), bricks());
        for x in [10, 12] {
            box_tuple(world, p, (x, 0, 10), (x, 6, 10), bricks());
            box_tuple(world, p, (x, 0, 12), (x, 6, 12), bricks());
            for y in [0, 4] {
                p.place_block(world, BlockPos::new(x, y, 10), lantern());
                p.place_block(world, BlockPos::new(x, y, 12), lantern());
            }
        }
        for x in [8, 14] {
            box_tuple(world, p, (x, 0, 7), (x, 2, 7), bricks());
            box_tuple(world, p, (x, 0, 14), (x, 2, 14), bricks());
        }
        box_tuple(world, p, (8, 3, 8), (8, 3, 13), dark());
        box_tuple(world, p, (14, 3, 8), (14, 3, 13), dark());
        spawn_elder(world, p, BlockPos::new(11, 5, 13));
    }
}

fn penthouse(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    box_tuple(world, p, (2, -1, 2), (11, -1, 11), bricks());
    for (minimum, maximum) in [
        ((0, -1, 0), (1, -1, 11)),
        ((12, -1, 0), (13, -1, 11)),
        ((2, -1, 0), (11, -1, 1)),
        ((2, -1, 12), (11, -1, 13)),
    ] {
        box_tuple(world, p, minimum, maximum, prismarine());
    }
    for (minimum, maximum) in [
        ((0, 0, 0), (0, 0, 13)),
        ((13, 0, 0), (13, 0, 13)),
        ((1, 0, 0), (12, 0, 0)),
        ((1, 0, 13), (12, 0, 13)),
    ] {
        box_tuple(world, p, minimum, maximum, bricks());
    }
    for i in (2..=11).step_by(3) {
        for local in [(0, 0, i), (13, 0, i), (i, 0, 0)] {
            p.place_block(world, tuple(local), lantern());
        }
    }
    for (minimum, maximum) in [
        ((2, 0, 3), (4, 0, 9)),
        ((9, 0, 3), (11, 0, 9)),
        ((4, 0, 9), (9, 0, 11)),
    ] {
        box_tuple(world, p, minimum, maximum, bricks());
    }
    for local in [(5, 0, 8), (8, 0, 8), (10, 0, 10), (3, 0, 10)] {
        p.place_block(world, tuple(local), bricks());
    }
    box_tuple(world, p, (3, 0, 3), (3, 0, 7), dark());
    box_tuple(world, p, (10, 0, 3), (10, 0, 7), dark());
    box_tuple(world, p, (6, 0, 10), (7, 0, 10), dark());
    for x in [3, 10] {
        for z in (2..=8).step_by(3) {
            box_tuple(world, p, (x, 0, z), (x, 2, z), bricks());
        }
    }
    box_tuple(world, p, (5, 0, 10), (5, 2, 10), bricks());
    box_tuple(world, p, (8, 0, 10), (8, 2, 10), bricks());
    box_tuple(world, p, (6, -1, 7), (7, -1, 8), dark());
    water_box(world, p, BlockPos::new(6, -1, 3), BlockPos::new(7, -1, 4));
    spawn_elder(world, p, BlockPos::new(6, 1, 6));
}

fn tuple(value: (i32, i32, i32)) -> BlockPos {
    BlockPos::new(value.0, value.1, value.2)
}
