//! Ocean-monument ordinary fitted room geometry.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::monument_graph::{
    MonumentChild, MonumentDirection, MonumentGraph, MonumentPieceKind, MonumentRoom,
};
use crate::generation::structure::monument_place::{
    MonumentWorld, box_tuple, bricks, connected_room, dark, default_floor, fill_only_box, lantern,
    placement, prismarine, room, water_box, wet_sponge,
};
use crate::generation::structure::piece::PiecePlacement;
use crate::generation::structure::processor::StructureState;

pub(crate) fn place_room_piece(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    clip: &BlockBox,
    random: &mut impl GenerationRandom,
) {
    let p = placement(child, clip);
    match child.kind {
        MonumentPieceKind::DoubleYZ => double_yz(world, graph, child, p),
        MonumentPieceKind::DoubleXY => double_xy(world, graph, child, p),
        MonumentPieceKind::DoubleZ => double_z(world, graph, child, p),
        MonumentPieceKind::DoubleX => double_x(world, graph, child, p),
        MonumentPieceKind::DoubleY => double_y(world, graph, child, p),
        MonumentPieceKind::SimpleTop => simple_top(world, graph, child, p, random),
        MonumentPieceKind::Simple => simple(world, graph, child, p, random),
        _ => unreachable!("special monument piece dispatched as ordinary room"),
    }
}

fn double_yz(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    p: PiecePlacement<'_>,
) {
    let south = room(graph, child);
    let north = connection(graph, south, MonumentDirection::North);
    let south_up = connection(graph, south, MonumentDirection::Up);
    let north_up = connection(graph, north, MonumentDirection::Up);
    if south.index / 25 > 0 {
        default_floor(world, p, 0, 8, north.opening(MonumentDirection::Down));
        default_floor(world, p, 0, 0, south.opening(MonumentDirection::Down));
    }
    roof_if_top(world, p, south_up, (1, 8, 1), (6, 8, 7));
    roof_if_top(world, p, north_up, (1, 8, 8), (6, 8, 14));
    for y in 1..=7 {
        let state = band(y);
        for (minimum, maximum) in [
            ((0, y, 0), (0, y, 15)),
            ((7, y, 0), (7, y, 15)),
            ((1, y, 0), (6, y, 0)),
            ((1, y, 15), (6, y, 15)),
        ] {
            b(world, p, minimum, maximum, state.clone());
        }
        b(
            world,
            p,
            (3, y, 7),
            (4, y, 8),
            if y == 2 || y == 6 { lantern() } else { dark() },
        );
    }
    portal_z(world, p, south, MonumentDirection::South, 0, 1);
    portal_x(world, p, south, MonumentDirection::West, 0, 3, 1);
    portal_x(world, p, south, MonumentDirection::East, 7, 3, 1);
    portal_z(world, p, north, MonumentDirection::North, 15, 1);
    portal_x(world, p, north, MonumentDirection::West, 0, 11, 1);
    portal_x(world, p, north, MonumentDirection::East, 7, 11, 1);
    portal_z(world, p, south_up, MonumentDirection::South, 0, 5);
    upper_side_portal(world, p, south_up, MonumentDirection::East, (7, 3, 5, 2));
    upper_side_portal(world, p, south_up, MonumentDirection::West, (0, 3, 5, 2));
    portal_z(world, p, north_up, MonumentDirection::North, 15, 5);
    upper_side_portal(world, p, north_up, MonumentDirection::West, (0, 11, 5, 10));
    upper_side_portal(world, p, north_up, MonumentDirection::East, (7, 11, 5, 10));
}

fn upper_side_portal(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    geometry: (i32, i32, i32, i32),
) {
    if !room.opening(direction) {
        return;
    }
    let (x, z, y, support_z) = geometry;
    water_box(
        world,
        p,
        BlockPos::new(x, y, z),
        BlockPos::new(x, y + 1, z + 1),
    );
    let west = direction == MonumentDirection::West;
    let (x0, x1, column_x) = if west { (1, 2, 1) } else { (5, 6, 6) };
    b(
        world,
        p,
        (x0, 4, support_z),
        (x1, 4, support_z + 3),
        bricks(),
    );
    for column_z in [support_z, support_z + 3] {
        b(
            world,
            p,
            (column_x, 1, column_z),
            (column_x, 3, column_z),
            bricks(),
        );
    }
}

fn double_xy(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    p: PiecePlacement<'_>,
) {
    let west = room(graph, child);
    let east = connection(graph, west, MonumentDirection::East);
    let west_up = connection(graph, west, MonumentDirection::Up);
    let east_up = connection(graph, east, MonumentDirection::Up);
    if west.index / 25 > 0 {
        default_floor(world, p, 8, 0, east.opening(MonumentDirection::Down));
        default_floor(world, p, 0, 0, west.opening(MonumentDirection::Down));
    }
    roof_if_top(world, p, west_up, (1, 8, 1), (7, 8, 6));
    roof_if_top(world, p, east_up, (8, 8, 1), (14, 8, 6));
    for y in 1..=7 {
        let state = band(y);
        for (minimum, maximum) in [
            ((0, y, 0), (0, y, 7)),
            ((15, y, 0), (15, y, 7)),
            ((1, y, 0), (15, y, 0)),
            ((1, y, 7), (14, y, 7)),
        ] {
            b(world, p, minimum, maximum, state.clone());
        }
    }
    for (minimum, maximum) in [
        ((2, 1, 3), (2, 7, 4)),
        ((3, 1, 2), (4, 7, 2)),
        ((3, 1, 5), (4, 7, 5)),
        ((13, 1, 3), (13, 7, 4)),
        ((11, 1, 2), (12, 7, 2)),
        ((11, 1, 5), (12, 7, 5)),
        ((5, 1, 3), (5, 3, 4)),
        ((10, 1, 3), (10, 3, 4)),
        ((5, 7, 2), (10, 7, 5)),
        ((5, 5, 2), (5, 7, 2)),
        ((10, 5, 2), (10, 7, 2)),
        ((5, 5, 5), (5, 7, 5)),
        ((10, 5, 5), (10, 7, 5)),
        ((6, 6, 2), (6, 6, 2)),
        ((9, 6, 2), (9, 6, 2)),
        ((6, 6, 5), (6, 6, 5)),
        ((9, 6, 5), (9, 6, 5)),
        ((5, 4, 3), (6, 4, 4)),
        ((9, 4, 3), (10, 4, 4)),
    ] {
        b(world, p, minimum, maximum, bricks());
    }
    for position in [(5, 4, 2), (5, 4, 5), (10, 4, 2), (10, 4, 5)] {
        b(world, p, position, position, lantern());
    }
    portals_xy(world, p, west, 0, 1);
    portals_xy(world, p, east, 8, 1);
    portals_xy(world, p, west_up, 0, 5);
    portals_xy(world, p, east_up, 8, 5);
}

fn portals_xy(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    x_offset: i32,
    y: i32,
) {
    portal_z_offset(world, p, room, MonumentDirection::South, x_offset + 3, 0, y);
    portal_z_offset(world, p, room, MonumentDirection::North, x_offset + 3, 7, y);
    if x_offset == 0 {
        portal_x(world, p, room, MonumentDirection::West, 0, 3, y);
    } else {
        portal_x(world, p, room, MonumentDirection::East, 15, 3, y);
    }
}

fn double_z(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    p: PiecePlacement<'_>,
) {
    let south = room(graph, child);
    let north = connection(graph, south, MonumentDirection::North);
    if south.index / 25 > 0 {
        default_floor(world, p, 0, 8, north.opening(MonumentDirection::Down));
        default_floor(world, p, 0, 0, south.opening(MonumentDirection::Down));
    }
    roof_if_top(world, p, south, (1, 4, 1), (6, 4, 7));
    roof_if_top(world, p, north, (1, 4, 8), (6, 4, 14));
    perimeter(world, p, 7, 15, 3);
    for (minimum, maximum) in [
        ((1, 1, 1), (1, 1, 2)),
        ((6, 1, 1), (6, 1, 2)),
        ((1, 3, 1), (1, 3, 2)),
        ((6, 3, 1), (6, 3, 2)),
        ((1, 1, 13), (1, 1, 14)),
        ((6, 1, 13), (6, 1, 14)),
        ((1, 3, 13), (1, 3, 14)),
        ((6, 3, 13), (6, 3, 14)),
        ((2, 1, 6), (2, 3, 6)),
        ((5, 1, 6), (5, 3, 6)),
        ((2, 1, 9), (2, 3, 9)),
        ((5, 1, 9), (5, 3, 9)),
        ((3, 2, 6), (4, 2, 6)),
        ((3, 2, 9), (4, 2, 9)),
        ((2, 2, 7), (2, 2, 8)),
        ((5, 2, 7), (5, 2, 8)),
        ((2, 3, 5), (2, 3, 5)),
        ((5, 3, 5), (5, 3, 5)),
        ((2, 3, 10), (2, 3, 10)),
        ((5, 3, 10), (5, 3, 10)),
    ] {
        b(world, p, minimum, maximum, bricks());
    }
    for position in [(2, 2, 5), (5, 2, 5), (2, 2, 10), (5, 2, 10)] {
        b(world, p, position, position, lantern());
    }
    portal_z(world, p, south, MonumentDirection::South, 0, 1);
    portal_x(world, p, south, MonumentDirection::East, 7, 3, 1);
    portal_x(world, p, south, MonumentDirection::West, 0, 3, 1);
    portal_z(world, p, north, MonumentDirection::North, 15, 1);
    portal_x(world, p, north, MonumentDirection::West, 0, 11, 1);
    portal_x(world, p, north, MonumentDirection::East, 7, 11, 1);
}

fn double_x(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    p: PiecePlacement<'_>,
) {
    let west = room(graph, child);
    let east = connection(graph, west, MonumentDirection::East);
    if west.index / 25 > 0 {
        default_floor(world, p, 8, 0, east.opening(MonumentDirection::Down));
        default_floor(world, p, 0, 0, west.opening(MonumentDirection::Down));
    }
    roof_if_top(world, p, west, (1, 4, 1), (7, 4, 6));
    roof_if_top(world, p, east, (8, 4, 1), (14, 4, 6));
    perimeter(world, p, 15, 7, 3);
    for (minimum, maximum, state) in [
        ((5, 1, 0), (10, 1, 4), bricks()),
        ((6, 2, 0), (9, 2, 3), prismarine()),
        ((5, 3, 0), (10, 3, 4), bricks()),
    ] {
        b(world, p, minimum, maximum, state);
    }
    for position in [(6, 2, 3), (9, 2, 3)] {
        b(world, p, position, position, lantern());
    }
    portals_xy(world, p, west, 0, 1);
    portals_xy(world, p, east, 8, 1);
}

fn double_y(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    p: PiecePlacement<'_>,
) {
    let lower = room(graph, child);
    let upper = connection(graph, lower, MonumentDirection::Up);
    if lower.index / 25 > 0 {
        default_floor(world, p, 0, 0, lower.opening(MonumentDirection::Down));
    }
    roof_if_top(world, p, upper, (1, 8, 1), (6, 8, 6));
    for (minimum, maximum) in [
        ((0, 4, 0), (0, 4, 7)),
        ((7, 4, 0), (7, 4, 7)),
        ((1, 4, 0), (6, 4, 0)),
        ((1, 4, 7), (6, 4, 7)),
        ((2, 4, 1), (2, 4, 2)),
        ((1, 4, 2), (1, 4, 2)),
        ((5, 4, 1), (5, 4, 2)),
        ((6, 4, 2), (6, 4, 2)),
        ((2, 4, 5), (2, 4, 6)),
        ((1, 4, 5), (1, 4, 5)),
        ((5, 4, 5), (5, 4, 6)),
        ((6, 4, 5), (6, 4, 5)),
    ] {
        b(world, p, minimum, maximum, bricks());
    }
    for (level, room) in [(1, lower), (5, upper)] {
        framed_wall_z(world, p, room, MonumentDirection::South, 0, level);
        framed_wall_z(world, p, room, MonumentDirection::North, 7, level);
        framed_wall_x(world, p, room, MonumentDirection::West, 0, level);
        framed_wall_x(world, p, room, MonumentDirection::East, 7, level);
    }
}

fn simple_top(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    let room = room(graph, child);
    common_floor_and_roof(world, p, room);
    let three = NonZeroU32::new(3).expect("three is nonzero");
    let four = NonZeroU32::new(4).expect("four is nonzero");
    for x in 1..=6 {
        for z in 1..=6 {
            if random.next_u32(three) == 0 {
                continue;
            }
            let y = 2 + i32::from(random.next_u32(four) != 0);
            b(world, p, (x, y, z), (x, 3, z), wet_sponge());
        }
    }
    simple_ring(world, p);
    if room.opening(MonumentDirection::South) {
        water_box(world, p, BlockPos::new(3, 1, 0), BlockPos::new(4, 2, 0));
    }
}

fn simple(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    child: &MonumentChild,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    let room = room(graph, child);
    common_floor_and_roof(world, p, room);
    let center_pillar = child.design != 0
        && random.next_bool()
        && !room.opening(MonumentDirection::Down)
        && !room.opening(MonumentDirection::Up)
        && room.opening_count() > 1;
    match child.design {
        0 => simple_design_zero(world, p, room),
        1 => simple_design_one(world, p, room),
        2 => simple_design_two(world, p, room),
        _ => unreachable!("simple monument design is bounded to three"),
    }
    if center_pillar {
        b(world, p, (3, 1, 3), (4, 1, 4), bricks());
        b(world, p, (3, 2, 3), (4, 2, 4), prismarine());
        b(world, p, (3, 3, 3), (4, 3, 4), bricks());
    }
}

fn simple_design_zero(world: &mut impl MonumentWorld, p: PiecePlacement<'_>, room: &MonumentRoom) {
    for (x, z, gray_minimum, gray_maximum, lamp) in [
        (0, 0, (0, 2, 0), (2, 2, 0), (1, 2, 1)),
        (5, 0, (5, 2, 0), (7, 2, 0), (6, 2, 1)),
        (0, 5, (0, 2, 7), (2, 2, 7), (1, 2, 6)),
        (5, 5, (5, 2, 7), (7, 2, 7), (6, 2, 6)),
    ] {
        b(world, p, (x, 1, z), (x + 2, 1, z + 2), bricks());
        b(world, p, (x, 3, z), (x + 2, 3, z + 2), bricks());
        b(world, p, gray_minimum, gray_maximum, prismarine());
        let side = if x == 0 {
            ((0, 2, z), (0, 2, z + 2))
        } else {
            ((7, 2, z), (7, 2, z + 2))
        };
        b(world, p, side.0, side.1, prismarine());
        b(world, p, lamp, lamp, lantern());
    }
    design_zero_wall_z(world, p, room, MonumentDirection::South, 0, 0);
    design_zero_wall_z(world, p, room, MonumentDirection::North, 7, 6);
    design_zero_wall_x(world, p, room, MonumentDirection::West, 0, 0);
    design_zero_wall_x(world, p, room, MonumentDirection::East, 7, 6);
}

fn design_zero_wall_z(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    z: i32,
    inner_z: i32,
) {
    if room.opening(direction) {
        b(world, p, (3, 3, z), (4, 3, z), bricks());
    } else {
        b(world, p, (3, 3, inner_z), (4, 3, inner_z + 1), bricks());
        b(world, p, (3, 2, z), (4, 2, z), prismarine());
        b(world, p, (3, 1, inner_z), (4, 1, inner_z + 1), bricks());
    }
}

fn design_zero_wall_x(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    x: i32,
    inner_x: i32,
) {
    if room.opening(direction) {
        b(world, p, (x, 3, 3), (x, 3, 4), bricks());
    } else {
        b(world, p, (inner_x, 3, 3), (inner_x + 1, 3, 4), bricks());
        b(world, p, (x, 2, 3), (x, 2, 4), prismarine());
        b(world, p, (inner_x, 1, 3), (inner_x + 1, 1, 4), bricks());
    }
}

fn simple_design_one(world: &mut impl MonumentWorld, p: PiecePlacement<'_>, room: &MonumentRoom) {
    for (x, z) in [(2, 2), (2, 5), (5, 5), (5, 2)] {
        b(world, p, (x, 1, z), (x, 3, z), bricks());
        b(world, p, (x, 2, z), (x, 2, z), lantern());
    }
    for (minimum, maximum) in [
        ((0, 1, 0), (1, 3, 0)),
        ((0, 1, 1), (0, 3, 1)),
        ((0, 1, 7), (1, 3, 7)),
        ((0, 1, 6), (0, 3, 6)),
        ((6, 1, 7), (7, 3, 7)),
        ((7, 1, 6), (7, 3, 6)),
        ((6, 1, 0), (7, 3, 0)),
        ((7, 1, 1), (7, 3, 1)),
    ] {
        b(world, p, minimum, maximum, bricks());
    }
    for position in [
        (1, 2, 0),
        (0, 2, 1),
        (1, 2, 7),
        (0, 2, 6),
        (6, 2, 7),
        (7, 2, 6),
        (6, 2, 0),
        (7, 2, 1),
    ] {
        b(world, p, position, position, prismarine());
    }
    closed_wall_z(world, p, room, MonumentDirection::South, 0);
    closed_wall_z(world, p, room, MonumentDirection::North, 7);
    closed_wall_x(world, p, room, MonumentDirection::West, 0);
    closed_wall_x(world, p, room, MonumentDirection::East, 7);
}

fn simple_design_two(world: &mut impl MonumentWorld, p: PiecePlacement<'_>, room: &MonumentRoom) {
    simple_ring(world, p);
    portal_z(world, p, room, MonumentDirection::South, 0, 1);
    portal_z(world, p, room, MonumentDirection::North, 7, 1);
    portal_x(world, p, room, MonumentDirection::West, 0, 3, 1);
    portal_x(world, p, room, MonumentDirection::East, 7, 3, 1);
}

fn simple_ring(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    for (y, state) in [(1, bricks()), (2, dark()), (3, bricks())] {
        for (minimum, maximum) in [
            ((0, y, 0), (0, y, 7)),
            ((7, y, 0), (7, y, 7)),
            ((1, y, 0), (6, y, 0)),
            ((1, y, 7), (6, y, 7)),
        ] {
            b(world, p, minimum, maximum, state.clone());
        }
    }
    for (minimum, maximum) in [
        ((0, 1, 3), (0, 2, 4)),
        ((7, 1, 3), (7, 2, 4)),
        ((3, 1, 0), (4, 2, 0)),
        ((3, 1, 7), (4, 2, 7)),
    ] {
        b(world, p, minimum, maximum, dark());
    }
}

fn common_floor_and_roof(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
) {
    if room.index / 25 > 0 {
        default_floor(world, p, 0, 0, room.opening(MonumentDirection::Down));
    }
    roof_if_top(world, p, room, (1, 4, 1), (6, 4, 6));
}

fn roof_if_top(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    minimum: (i32, i32, i32),
    maximum: (i32, i32, i32),
) {
    if room.connections[MonumentDirection::Up.index()].is_none() {
        fill_only_box(world, p, pos(minimum), pos(maximum), prismarine());
    }
}

fn perimeter(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    maximum_x: i32,
    maximum_z: i32,
    maximum_y: i32,
) {
    for y in 1..=maximum_y {
        let state = if y == 2 { prismarine() } else { bricks() };
        for (minimum, maximum) in [
            ((0, y, 0), (0, y, maximum_z)),
            ((maximum_x, y, 0), (maximum_x, y, maximum_z)),
            ((1, y, 0), (maximum_x, y, 0)),
            ((1, y, maximum_z), (maximum_x - 1, y, maximum_z)),
        ] {
            b(world, p, minimum, maximum, state.clone());
        }
    }
}

fn framed_wall_z(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    z: i32,
    y: i32,
) {
    if room.opening(direction) {
        b(world, p, (2, y, z), (2, y + 2, z), bricks());
        b(world, p, (5, y, z), (5, y + 2, z), bricks());
        b(world, p, (3, y + 2, z), (4, y + 2, z), bricks());
    } else {
        b(world, p, (0, y, z), (7, y + 2, z), bricks());
        b(world, p, (0, y + 1, z), (7, y + 1, z), prismarine());
    }
}

fn framed_wall_x(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    x: i32,
    y: i32,
) {
    if room.opening(direction) {
        b(world, p, (x, y, 2), (x, y + 2, 2), bricks());
        b(world, p, (x, y, 5), (x, y + 2, 5), bricks());
        b(world, p, (x, y + 2, 3), (x, y + 2, 4), bricks());
    } else {
        b(world, p, (x, y, 0), (x, y + 2, 7), bricks());
        b(world, p, (x, y + 1, 0), (x, y + 1, 7), prismarine());
    }
}

fn closed_wall_z(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    z: i32,
) {
    if !room.opening(direction) {
        b(world, p, (1, 3, z), (6, 3, z), bricks());
        b(world, p, (1, 2, z), (6, 2, z), prismarine());
        b(world, p, (1, 1, z), (6, 1, z), bricks());
    }
}

fn closed_wall_x(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    x: i32,
) {
    if !room.opening(direction) {
        b(world, p, (x, 3, 1), (x, 3, 6), bricks());
        b(world, p, (x, 2, 1), (x, 2, 6), prismarine());
        b(world, p, (x, 1, 1), (x, 1, 6), bricks());
    }
}

fn portal_z(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    z: i32,
    y: i32,
) {
    portal_z_offset(world, p, room, direction, 3, z, y);
}

fn portal_z_offset(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    x: i32,
    z: i32,
    y: i32,
) {
    if room.opening(direction) {
        water_box(
            world,
            p,
            BlockPos::new(x, y, z),
            BlockPos::new(x + 1, y + 1, z),
        );
    }
}

fn portal_x(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    room: &MonumentRoom,
    direction: MonumentDirection,
    x: i32,
    z: i32,
    y: i32,
) {
    if room.opening(direction) {
        water_box(
            world,
            p,
            BlockPos::new(x, y, z),
            BlockPos::new(x, y + 1, z + 1),
        );
    }
}

fn connection<'a>(
    graph: &'a MonumentGraph,
    room: &MonumentRoom,
    direction: MonumentDirection,
) -> &'a MonumentRoom {
    connected_room(graph, room, direction.index()).expect("fitted monument room connection exists")
}

fn band(y: i32) -> StructureState {
    if y == 2 || y == 6 {
        prismarine()
    } else {
        bricks()
    }
}

fn b(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    minimum: (i32, i32, i32),
    maximum: (i32, i32, i32),
    state: StructureState,
) {
    box_tuple(world, p, minimum, maximum, state);
}

fn pos(value: (i32, i32, i32)) -> BlockPos {
    BlockPos::new(value.0, value.1, value.2)
}
