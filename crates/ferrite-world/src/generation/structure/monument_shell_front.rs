//! Ocean-monument wings, entrance facade, and roof shell.

use crate::generation::structure::monument_building::{b, intersects, pos};
use crate::generation::structure::monument_place::{
    MonumentWorld, bricks, dark, lantern, prismarine, water_box,
};
use crate::generation::structure::piece::PiecePlacement;

pub(crate) fn wing(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    flipped: bool,
    x_offset: i32,
) {
    if !intersects(p, x_offset, 0, x_offset + 23, 20) {
        return;
    }
    b(
        world,
        p,
        (x_offset, 0, 0),
        (x_offset + 24, 0, 20),
        prismarine(),
    );
    water_box(world, p, pos(x_offset, 1, 0), pos(x_offset + 24, 10, 20));
    for i in 0..4 {
        for (minimum, maximum) in [
            ((x_offset + i, i + 1, i), (x_offset + i, i + 1, 20)),
            (
                (x_offset + i + 7, i + 5, i + 7),
                (x_offset + i + 7, i + 5, 20),
            ),
            (
                (x_offset + 17 - i, i + 5, i + 7),
                (x_offset + 17 - i, i + 5, 20),
            ),
            (
                (x_offset + 24 - i, i + 1, i),
                (x_offset + 24 - i, i + 1, 20),
            ),
            ((x_offset + i + 1, i + 1, i), (x_offset + 23 - i, i + 1, i)),
            (
                (x_offset + i + 8, i + 5, i + 7),
                (x_offset + 16 - i, i + 5, i + 7),
            ),
        ] {
            b(world, p, minimum, maximum, bricks());
        }
    }
    for (minimum, maximum) in [
        ((x_offset + 4, 4, 4), (x_offset + 6, 4, 20)),
        ((x_offset + 7, 4, 4), (x_offset + 17, 4, 6)),
        ((x_offset + 18, 4, 4), (x_offset + 20, 4, 20)),
        ((x_offset + 11, 8, 11), (x_offset + 13, 8, 20)),
    ] {
        b(world, p, minimum, maximum, prismarine());
    }
    for z in [12, 15, 18] {
        block(world, p, x_offset + 12, 9, z, bricks());
    }
    let left = x_offset + if flipped { 19 } else { 5 };
    let right = x_offset + if flipped { 5 } else { 19 };
    for z in (5..=20).rev().step_by(3) {
        block(world, p, left, 5, z, bricks());
    }
    for z in (7..=19).rev().step_by(3) {
        block(world, p, right, 5, z, bricks());
    }
    for i in 0..4 {
        let x = if flipped {
            x_offset + 24 - (17 - i * 3)
        } else {
            x_offset + 17 - i * 3
        };
        block(world, p, x, 5, 5, bricks());
    }
    block(world, p, right, 5, 5, bricks());
    b(
        world,
        p,
        (x_offset + 11, 1, 12),
        (x_offset + 13, 7, 12),
        prismarine(),
    );
    b(
        world,
        p,
        (x_offset + 12, 1, 11),
        (x_offset + 12, 7, 13),
        prismarine(),
    );
}

pub(crate) fn entrance_arches(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    if !intersects(p, 22, 5, 35, 17) {
        return;
    }
    water_box(world, p, pos(25, 0, 0), pos(32, 8, 20));
    for i in 0..4 {
        let z = 5 + i * 4;
        for (minimum, maximum) in [
            ((24, 2, z), (24, 4, z)),
            ((22, 4, z), (23, 4, z)),
            ((25, 5, z), (25, 5, z)),
            ((26, 6, z), (26, 6, z)),
            ((33, 2, z), (33, 4, z)),
            ((34, 4, z), (35, 4, z)),
            ((32, 5, z), (32, 5, z)),
            ((31, 6, z), (31, 6, z)),
        ] {
            b(world, p, minimum, maximum, bricks());
        }
        block(world, p, 26, 5, z, lantern());
        block(world, p, 31, 5, z, lantern());
        b(world, p, (27, 6, z), (30, 6, z), prismarine());
    }
}

pub(crate) fn entrance_wall(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    if !intersects(p, 15, 20, 42, 21) {
        return;
    }
    for (minimum, maximum) in [
        ((15, 0, 21), (42, 0, 21)),
        ((21, 12, 21), (36, 12, 21)),
        ((17, 11, 21), (40, 11, 21)),
        ((16, 10, 21), (41, 10, 21)),
        ((15, 7, 21), (42, 9, 21)),
        ((16, 6, 21), (41, 6, 21)),
        ((17, 5, 21), (40, 5, 21)),
        ((21, 4, 21), (36, 4, 21)),
        ((22, 3, 21), (26, 3, 21)),
        ((31, 3, 21), (35, 3, 21)),
        ((23, 2, 21), (25, 2, 21)),
        ((32, 2, 21), (34, 2, 21)),
    ] {
        b(world, p, minimum, maximum, prismarine());
    }
    water_box(world, p, pos(26, 1, 21), pos(31, 3, 21));
    b(world, p, (28, 4, 20), (29, 4, 21), bricks());
    for (x, y) in [(27, 3), (30, 3), (26, 2), (31, 2), (25, 1), (32, 1)] {
        block(world, p, x, y, 21, bricks());
    }
    for i in 0..7 {
        block(world, p, 28 - i, 6 + i, 21, dark());
        block(world, p, 29 + i, 6 + i, 21, dark());
    }
    for i in 0..4 {
        block(world, p, 28 - i, 9 + i, 21, dark());
        block(world, p, 29 + i, 9 + i, 21, dark());
    }
    for x in [28, 29] {
        block(world, p, x, 12, 21, dark());
    }
    for i in 0..3 {
        for y in [8, 9] {
            block(world, p, 22 - i * 2, y, 21, dark());
            block(world, p, 35 + i * 2, y, 21, dark());
        }
    }
    for (minimum, maximum) in [
        ((15, 13, 21), (42, 15, 21)),
        ((15, 1, 21), (15, 6, 21)),
        ((16, 1, 21), (16, 5, 21)),
        ((17, 1, 21), (20, 4, 21)),
        ((21, 1, 21), (21, 3, 21)),
        ((22, 1, 21), (22, 2, 21)),
        ((23, 1, 21), (24, 1, 21)),
        ((42, 1, 21), (42, 6, 21)),
        ((41, 1, 21), (41, 5, 21)),
        ((37, 1, 21), (40, 4, 21)),
        ((36, 1, 21), (36, 3, 21)),
        ((33, 1, 21), (34, 1, 21)),
        ((35, 1, 21), (35, 2, 21)),
    ] {
        water_box(world, p, pos_t(minimum), pos_t(maximum));
    }
}

pub(crate) fn roof_piece(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    if !intersects(p, 21, 21, 36, 36) {
        return;
    }
    b(world, p, (21, 0, 22), (36, 0, 36), prismarine());
    water_box(world, p, pos(21, 1, 22), pos(36, 23, 36));
    for i in 0..4 {
        for (minimum, maximum) in [
            ((21 + i, 13 + i, 21 + i), (36 - i, 13 + i, 21 + i)),
            ((21 + i, 13 + i, 36 - i), (36 - i, 13 + i, 36 - i)),
            ((21 + i, 13 + i, 22 + i), (21 + i, 13 + i, 35 - i)),
            ((36 - i, 13 + i, 22 + i), (36 - i, 13 + i, 35 - i)),
        ] {
            b(world, p, minimum, maximum, bricks());
        }
    }
    b(world, p, (25, 16, 25), (32, 16, 32), prismarine());
    for (x, z) in [(25, 25), (32, 25), (25, 32), (32, 32)] {
        b(world, p, (x, 17, z), (x, 19, z), bricks());
    }
    for (x0, z0, x1, z1) in [
        (26, 26, 27, 27),
        (26, 31, 27, 30),
        (31, 31, 30, 30),
        (31, 26, 30, 27),
    ] {
        block(world, p, x0, 20, z0, bricks());
        block(world, p, x1, 21, z1, bricks());
        block(world, p, x1, 20, z1, lantern());
    }
    for (minimum, maximum) in [
        ((28, 21, 27), (29, 21, 27)),
        ((27, 21, 28), (27, 21, 29)),
        ((28, 21, 30), (29, 21, 30)),
        ((30, 21, 28), (30, 21, 29)),
    ] {
        b(world, p, minimum, maximum, prismarine());
    }
}

fn block(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    x: i32,
    y: i32,
    z: i32,
    state: crate::generation::structure::processor::StructureState,
) {
    p.place_block(world, pos(x, y, z), state);
}

fn pos_t(value: (i32, i32, i32)) -> ferrite_foundation::coordinate::BlockPos {
    pos(value.0, value.1, value.2)
}
