//! Ocean-monument lower, middle, and upper rear wall shells.

use crate::generation::structure::monument_building::{b, intersects, pos};
use crate::generation::structure::monument_place::{MonumentWorld, bricks, prismarine, water_box};
use crate::generation::structure::piece::PiecePlacement;

pub(crate) fn lower_wall(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    if intersects(p, 0, 21, 6, 58) {
        b(world, p, (0, 0, 21), (6, 0, 57), prismarine());
        water_box(world, p, pos(0, 1, 21), pos(6, 7, 57));
        b(world, p, (4, 4, 21), (6, 4, 53), prismarine());
        for i in 0..4 {
            b(world, p, (i, i + 1, 21), (i, i + 1, 57 - i), bricks());
        }
        for z in (23..53).step_by(3) {
            block(world, p, 5, 5, z);
        }
        block(world, p, 5, 5, 52);
        // Source repeats these four rails; preserve the repeated write sequence.
        for i in 0..4 {
            b(world, p, (i, i + 1, 21), (i, i + 1, 57 - i), bricks());
        }
        b(world, p, (4, 1, 52), (6, 3, 52), prismarine());
        b(world, p, (5, 1, 51), (5, 3, 53), prismarine());
    }
    if intersects(p, 51, 21, 58, 58) {
        b(world, p, (51, 0, 21), (57, 0, 57), prismarine());
        water_box(world, p, pos(51, 1, 21), pos(57, 7, 57));
        b(world, p, (51, 4, 21), (53, 4, 53), prismarine());
        for i in 0..4 {
            b(
                world,
                p,
                (57 - i, i + 1, 21),
                (57 - i, i + 1, 57 - i),
                bricks(),
            );
        }
        for z in (23..53).step_by(3) {
            block(world, p, 52, 5, z);
        }
        block(world, p, 52, 5, 52);
        b(world, p, (51, 1, 52), (53, 3, 52), prismarine());
        b(world, p, (52, 1, 51), (52, 3, 53), prismarine());
    }
    if intersects(p, 0, 51, 57, 57) {
        b(world, p, (7, 0, 51), (50, 0, 57), prismarine());
        water_box(world, p, pos(7, 1, 51), pos(50, 10, 57));
        for i in 0..4 {
            b(
                world,
                p,
                (i + 1, i + 1, 57 - i),
                (56 - i, i + 1, 57 - i),
                bricks(),
            );
        }
    }
}

pub(crate) fn middle_wall(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    if intersects(p, 7, 21, 13, 50) {
        b(world, p, (7, 0, 21), (13, 0, 50), prismarine());
        water_box(world, p, pos(7, 1, 21), pos(13, 10, 50));
        b(world, p, (11, 8, 21), (13, 8, 53), prismarine());
        for i in 0..4 {
            b(world, p, (i + 7, i + 5, 21), (i + 7, i + 5, 54), bricks());
        }
        for z in (21..=45).step_by(3) {
            block(world, p, 12, 9, z);
        }
    }
    if intersects(p, 44, 21, 50, 54) {
        b(world, p, (44, 0, 21), (50, 0, 50), prismarine());
        water_box(world, p, pos(44, 1, 21), pos(50, 10, 50));
        b(world, p, (44, 8, 21), (46, 8, 53), prismarine());
        for i in 0..4 {
            b(world, p, (50 - i, i + 5, 21), (50 - i, i + 5, 54), bricks());
        }
        for z in (21..=45).step_by(3) {
            block(world, p, 45, 9, z);
        }
    }
    if intersects(p, 8, 44, 49, 54) {
        b(world, p, (14, 0, 44), (43, 0, 50), prismarine());
        water_box(world, p, pos(14, 1, 44), pos(43, 10, 50));
        for x in (12..=45).step_by(3) {
            block(world, p, x, 9, 45);
            block(world, p, x, 9, 52);
            if matches!(x, 12 | 18 | 24 | 33 | 39 | 45) {
                for (y, z) in [
                    (9, 47),
                    (9, 50),
                    (10, 45),
                    (10, 46),
                    (10, 51),
                    (10, 52),
                    (11, 47),
                    (11, 50),
                    (12, 48),
                    (12, 49),
                ] {
                    block(world, p, x, y, z);
                }
            }
        }
        for i in 0..3 {
            b(
                world,
                p,
                (8 + i, 5 + i, 54),
                (49 - i, 5 + i, 54),
                prismarine(),
            );
        }
        b(world, p, (11, 8, 54), (46, 8, 54), bricks());
        b(world, p, (14, 8, 44), (43, 8, 53), prismarine());
    }
}

pub(crate) fn upper_wall(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    if intersects(p, 14, 21, 20, 43) {
        b(world, p, (14, 0, 21), (20, 0, 43), prismarine());
        water_box(world, p, pos(14, 1, 22), pos(20, 14, 43));
        b(world, p, (18, 12, 22), (20, 12, 39), prismarine());
        b(world, p, (18, 12, 21), (20, 12, 21), bricks());
        for i in 0..4 {
            b(
                world,
                p,
                (i + 14, i + 9, 21),
                (i + 14, i + 9, 43 - i),
                bricks(),
            );
        }
        for z in (23..=39).step_by(3) {
            block(world, p, 19, 13, z);
        }
    }
    if intersects(p, 37, 21, 43, 43) {
        b(world, p, (37, 0, 21), (43, 0, 43), prismarine());
        water_box(world, p, pos(37, 1, 22), pos(43, 14, 43));
        b(world, p, (37, 12, 22), (39, 12, 39), prismarine());
        b(world, p, (37, 12, 21), (39, 12, 21), bricks());
        for i in 0..4 {
            b(
                world,
                p,
                (43 - i, i + 9, 21),
                (43 - i, i + 9, 43 - i),
                bricks(),
            );
        }
        for z in (23..=39).step_by(3) {
            block(world, p, 38, 13, z);
        }
    }
    if intersects(p, 15, 37, 42, 43) {
        b(world, p, (21, 0, 37), (36, 0, 43), prismarine());
        water_box(world, p, pos(21, 1, 37), pos(36, 14, 43));
        b(world, p, (21, 12, 37), (36, 12, 39), prismarine());
        for i in 0..4 {
            b(
                world,
                p,
                (15 + i, i + 9, 43 - i),
                (42 - i, i + 9, 43 - i),
                bricks(),
            );
        }
        for x in (21..=36).step_by(3) {
            block(world, p, x, 13, 38);
        }
    }
}

fn block(world: &mut impl MonumentWorld, p: PiecePlacement<'_>, x: i32, y: i32, z: i32) {
    p.place_block(world, pos(x, y, z), bricks());
}
