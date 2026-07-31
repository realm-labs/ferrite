//! Nether-fortress castle-family geometry and its chest/lava transactions.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::fortress_graph::{FortressPiece, FortressPieceKind};
use crate::generation::structure::fortress_place::{
    FortressWorld, air, create_turn_chest, fence, fill_column_down, generate_box, lava,
    nether_bricks, nether_wart, place, placement, schedule_explicit_lava_tick, soul_sand, stairs,
};
use crate::generation::structure::piece::PiecePlacement;
use crate::generation::structure::processor::StructureState;

pub(crate) fn place_castle_piece(
    world: &mut impl FortressWorld,
    piece: &mut FortressPiece,
    clip: &BlockBox,
    _random: &mut impl GenerationRandom,
    loot_seed: &mut impl FnMut() -> i64,
) {
    match piece.kind {
        FortressPieceKind::CastleEntrance => castle_entrance(world, piece, clip),
        FortressPieceKind::CastleSmallCorridor => small_corridor(world, placement(piece, clip)),
        FortressPieceKind::CastleSmallCrossing => small_crossing(world, placement(piece, clip)),
        FortressPieceKind::CastleRightTurn => right_turn(world, piece, clip, loot_seed),
        FortressPieceKind::CastleLeftTurn => left_turn(world, piece, clip, loot_seed),
        FortressPieceKind::CastleCorridorStairs => corridor_stairs(world, placement(piece, clip)),
        FortressPieceKind::CastleTBalcony => t_balcony(world, placement(piece, clip)),
        FortressPieceKind::CastleStalkRoom => stalk_room(world, placement(piece, clip)),
        _ => unreachable!("bridge pieces use fortress_bridge"),
    }
}

fn castle_entrance(world: &mut impl FortressWorld, piece: &FortressPiece, clip: &BlockBox) {
    let p = placement(piece, clip);
    castle_shell(world, p);
    b(world, p, (5, 8, 0), (7, 8, 0), fence(&[]));
    battlements_and_windows(world, p);
    castle_foundation(world, p);
    b(world, p, (5, 5, 5), (7, 5, 7), nether_bricks());
    b(world, p, (6, 1, 6), (6, 4, 6), air());
    place(world, p, BlockPos::new(6, 0, 6), nether_bricks());
    place(world, p, BlockPos::new(6, 5, 6), lava());
    schedule_explicit_lava_tick(world, p, BlockPos::new(6, 5, 6));
}

fn small_corridor(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    small_base(world, p);
    b(world, p, (0, 2, 0), (0, 5, 4), nether_bricks());
    b(world, p, (4, 2, 0), (4, 5, 4), nether_bricks());
    let ns = fence(&["north", "south"]);
    for (x, z) in [(0, 1), (0, 3), (4, 1), (4, 3)] {
        b(world, p, (x, 3, z), (x, 4, z), ns.clone());
    }
    small_roof_and_supports(world, p);
}

fn right_turn(
    world: &mut impl FortressWorld,
    piece: &mut FortressPiece,
    clip: &BlockBox,
    loot_seed: &mut impl FnMut() -> i64,
) {
    let p = placement(piece, clip);
    small_base(world, p);
    b(world, p, (0, 2, 0), (0, 5, 4), nether_bricks());
    b(world, p, (0, 3, 1), (0, 4, 1), fence(&["north", "south"]));
    b(world, p, (0, 3, 3), (0, 4, 3), fence(&["north", "south"]));
    b(world, p, (4, 2, 0), (4, 5, 0), nether_bricks());
    b(world, p, (1, 2, 4), (4, 5, 4), nether_bricks());
    b(world, p, (1, 3, 4), (1, 4, 4), fence(&["west", "east"]));
    b(world, p, (3, 3, 4), (3, 4, 4), fence(&["west", "east"]));
    create_turn_chest(world, piece, clip, BlockPos::new(1, 2, 3), loot_seed);
    small_roof_and_supports(world, p);
}

fn left_turn(
    world: &mut impl FortressWorld,
    piece: &mut FortressPiece,
    clip: &BlockBox,
    loot_seed: &mut impl FnMut() -> i64,
) {
    let p = placement(piece, clip);
    small_base(world, p);
    b(world, p, (4, 2, 0), (4, 5, 4), nether_bricks());
    b(world, p, (4, 3, 1), (4, 4, 1), fence(&["north", "south"]));
    b(world, p, (4, 3, 3), (4, 4, 3), fence(&["north", "south"]));
    b(world, p, (0, 2, 0), (0, 5, 0), nether_bricks());
    b(world, p, (0, 2, 4), (3, 5, 4), nether_bricks());
    b(world, p, (1, 3, 4), (1, 4, 4), fence(&["west", "east"]));
    b(world, p, (3, 3, 4), (3, 4, 4), fence(&["west", "east"]));
    create_turn_chest(world, piece, clip, BlockPos::new(3, 2, 3), loot_seed);
    small_roof_and_supports(world, p);
}

fn small_crossing(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    small_base(world, p);
    for (x, z) in [(0, 0), (4, 0), (0, 4), (4, 4)] {
        b(world, p, (x, 2, z), (x, 5, z), nether_bricks());
    }
    small_roof_and_supports(world, p);
}

fn small_base(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    b(world, p, (0, 0, 0), (4, 1, 4), nether_bricks());
    b(world, p, (0, 2, 0), (4, 5, 4), air());
}

fn small_roof_and_supports(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    b(world, p, (0, 6, 0), (4, 6, 4), nether_bricks());
    for x in 0..=4 {
        for z in 0..=4 {
            fill_column_down(world, p, x, z);
        }
    }
}

fn corridor_stairs(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    let stair = stairs("south");
    let ns = fence(&["north", "south"]);
    for step in 0..=9 {
        let floor = 1.max(7 - step);
        let roof = (floor + 5).max(14 - step).min(13);
        b(world, p, (0, 0, step), (4, floor, step), nether_bricks());
        b(world, p, (1, floor + 1, step), (3, roof - 1, step), air());
        if step <= 6 {
            for x in 1..=3 {
                place(world, p, BlockPos::new(x, floor + 1, step), stair.clone());
            }
        }
        b(world, p, (0, roof, step), (4, roof, step), nether_bricks());
        b(
            world,
            p,
            (0, floor + 1, step),
            (0, roof - 1, step),
            nether_bricks(),
        );
        b(
            world,
            p,
            (4, floor + 1, step),
            (4, roof - 1, step),
            nether_bricks(),
        );
        if step & 1 == 0 {
            b(
                world,
                p,
                (0, floor + 2, step),
                (0, floor + 3, step),
                ns.clone(),
            );
            b(
                world,
                p,
                (4, floor + 2, step),
                (4, floor + 3, step),
                ns.clone(),
            );
        }
        for x in 0..=4 {
            fill_column_down(world, p, x, step);
        }
    }
}

fn t_balcony(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    b(world, p, (0, 0, 0), (8, 1, 8), nether_bricks());
    b(world, p, (0, 2, 0), (8, 5, 8), air());
    b(world, p, (0, 6, 0), (8, 6, 5), nether_bricks());
    b(world, p, (0, 2, 0), (2, 5, 0), nether_bricks());
    b(world, p, (6, 2, 0), (8, 5, 0), nether_bricks());
    b(world, p, (1, 3, 0), (1, 4, 0), fence(&["west", "east"]));
    b(world, p, (7, 3, 0), (7, 4, 0), fence(&["west", "east"]));
    b(world, p, (0, 2, 4), (8, 2, 8), nether_bricks());
    b(world, p, (1, 1, 4), (2, 2, 4), air());
    b(world, p, (6, 1, 4), (7, 2, 4), air());
    b(world, p, (1, 3, 8), (7, 3, 8), fence(&["west", "east"]));
    place(world, p, BlockPos::new(0, 3, 8), fence(&["east", "south"]));
    place(world, p, BlockPos::new(8, 3, 8), fence(&["west", "south"]));
    b(world, p, (0, 3, 6), (0, 3, 7), fence(&["north", "south"]));
    b(world, p, (8, 3, 6), (8, 3, 7), fence(&["north", "south"]));
    for (minimum, maximum) in [
        ((0, 3, 4), (0, 5, 5)),
        ((8, 3, 4), (8, 5, 5)),
        ((1, 3, 5), (2, 5, 5)),
        ((6, 3, 5), (7, 5, 5)),
    ] {
        b(world, p, minimum, maximum, nether_bricks());
    }
    b(world, p, (1, 4, 5), (1, 5, 5), fence(&["west", "east"]));
    b(world, p, (7, 4, 5), (7, 5, 5), fence(&["west", "east"]));
    for z in 0..=5 {
        for x in 0..=8 {
            fill_column_down(world, p, x, z);
        }
    }
}

fn stalk_room(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    castle_shell(world, p);
    battlements_and_windows(world, p);
    let north_stair = stairs("north");
    for i in 0..=6 {
        let z = i + 4;
        for x in 5..=7 {
            place(world, p, BlockPos::new(x, 5 + i, z), north_stair.clone());
        }
        if (5..=8).contains(&z) {
            b(world, p, (5, 5, z), (7, i + 4, z), nether_bricks());
        } else if (9..=10).contains(&z) {
            b(world, p, (5, 8, z), (7, i + 4, z), nether_bricks());
        }
        if i >= 1 {
            b(world, p, (5, 6 + i, z), (7, 9 + i, z), air());
        }
    }
    for x in 5..=7 {
        place(world, p, BlockPos::new(x, 12, 11), north_stair.clone());
    }
    b(
        world,
        p,
        (5, 6, 7),
        (5, 7, 7),
        fence(&["north", "south", "east"]),
    );
    b(
        world,
        p,
        (7, 6, 7),
        (7, 7, 7),
        fence(&["north", "south", "west"]),
    );
    b(world, p, (5, 13, 12), (7, 13, 12), air());
    for (minimum, maximum) in [
        ((2, 5, 2), (3, 5, 3)),
        ((2, 5, 9), (3, 5, 10)),
        ((2, 5, 4), (2, 5, 8)),
        ((9, 5, 2), (10, 5, 3)),
        ((9, 5, 9), (10, 5, 10)),
        ((10, 5, 4), (10, 5, 8)),
    ] {
        b(world, p, minimum, maximum, nether_bricks());
    }
    for (x, z, facing) in [
        (4, 2, "west"),
        (4, 3, "west"),
        (4, 9, "west"),
        (4, 10, "west"),
        (8, 2, "east"),
        (8, 3, "east"),
        (8, 9, "east"),
        (8, 10, "east"),
    ] {
        place(world, p, BlockPos::new(x, 5, z), stairs(facing));
    }
    b(world, p, (3, 4, 4), (4, 4, 8), soul_sand());
    b(world, p, (8, 4, 4), (9, 4, 8), soul_sand());
    b(world, p, (3, 5, 4), (4, 5, 8), nether_wart());
    b(world, p, (8, 5, 4), (9, 5, 8), nether_wart());
    castle_foundation(world, p);
}

fn castle_shell(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    b(world, p, (0, 3, 0), (12, 4, 12), nether_bricks());
    b(world, p, (0, 5, 0), (12, 13, 12), air());
    for (minimum, maximum) in [
        ((0, 5, 0), (1, 12, 12)),
        ((11, 5, 0), (12, 12, 12)),
        ((2, 5, 11), (4, 12, 12)),
        ((8, 5, 11), (10, 12, 12)),
        ((5, 9, 11), (7, 12, 12)),
        ((2, 5, 0), (4, 12, 1)),
        ((8, 5, 0), (10, 12, 1)),
        ((5, 9, 0), (7, 12, 1)),
        ((2, 11, 2), (10, 12, 10)),
    ] {
        b(world, p, minimum, maximum, nether_bricks());
    }
}

fn battlements_and_windows(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    let we = fence(&["west", "east"]);
    let ns = fence(&["north", "south"]);
    for i in (1..=11).step_by(2) {
        b(world, p, (i, 10, 0), (i, 11, 0), we.clone());
        b(world, p, (i, 10, 12), (i, 11, 12), we.clone());
        b(world, p, (0, 10, i), (0, 11, i), ns.clone());
        b(world, p, (12, 10, i), (12, 11, i), ns.clone());
        for local in [(i, 13, 0), (i, 13, 12), (0, 13, i), (12, 13, i)] {
            place(world, p, tuple_pos(local), nether_bricks());
        }
        if i != 11 {
            place(world, p, BlockPos::new(i + 1, 13, 0), we.clone());
            place(world, p, BlockPos::new(i + 1, 13, 12), we.clone());
            place(world, p, BlockPos::new(0, 13, i + 1), ns.clone());
            place(world, p, BlockPos::new(12, 13, i + 1), ns.clone());
        }
    }
    for (local, state) in [
        ((0, 13, 0), fence(&["north", "east"])),
        ((0, 13, 12), fence(&["south", "east"])),
        ((12, 13, 12), fence(&["south", "west"])),
        ((12, 13, 0), fence(&["north", "west"])),
    ] {
        place(world, p, tuple_pos(local), state);
    }
    for z in (3..=9).step_by(2) {
        b(
            world,
            p,
            (1, 7, z),
            (1, 8, z),
            fence(&["north", "south", "west"]),
        );
        b(
            world,
            p,
            (11, 7, z),
            (11, 8, z),
            fence(&["north", "south", "east"]),
        );
    }
}

fn castle_foundation(world: &mut impl FortressWorld, p: PiecePlacement<'_>) {
    for (minimum, maximum) in [
        ((4, 2, 0), (8, 2, 12)),
        ((0, 2, 4), (12, 2, 8)),
        ((4, 0, 0), (8, 1, 3)),
        ((4, 0, 9), (8, 1, 12)),
        ((0, 0, 4), (3, 1, 8)),
        ((9, 0, 4), (12, 1, 8)),
    ] {
        b(world, p, minimum, maximum, nether_bricks());
    }
    for x in 4..=8 {
        for z in 0..=2 {
            fill_column_down(world, p, x, z);
            fill_column_down(world, p, x, 12 - z);
        }
    }
    for x in 0..=2 {
        for z in 4..=8 {
            fill_column_down(world, p, x, z);
            fill_column_down(world, p, 12 - x, z);
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
