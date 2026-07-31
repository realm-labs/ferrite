//! Stronghold crossings, stairs, chests, library, and portal-room transactions.

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::piece::PiecePlacement;
use crate::generation::structure::processor::StructureState;
use crate::generation::structure::stronghold_graph::{StrongholdPiece, StrongholdPieceKind};
use crate::generation::structure::stronghold_place::{
    StrongholdWorld, air, bars, box_, create_chest, facing, pos, selector_box, shell, small_door,
    state, stone_bricks,
};

pub(crate) fn place_special_piece(
    world: &mut impl StrongholdWorld,
    piece: &mut StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    match piece.kind {
        StrongholdPieceKind::RoomCrossing => room_crossing(world, piece, p, random),
        StrongholdPieceKind::StraightStairsDown => straight_stairs(world, piece, p, random),
        StrongholdPieceKind::StairsDown => {
            super::stronghold_rooms_basic::stairs_down(world, piece, p, random)
        }
        StrongholdPieceKind::FiveCrossing => five_crossing(world, piece, p, random),
        StrongholdPieceKind::ChestCorridor => chest_corridor(world, piece, p, random),
        StrongholdPieceKind::Library => library(world, piece, p, random),
        StrongholdPieceKind::PortalRoom => portal_room(world, piece, p, random),
        _ => unreachable!("basic stronghold room dispatched as special"),
    }
}

fn room_crossing(
    world: &mut impl StrongholdWorld,
    piece: &StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    shell(world, p, pos(10, 6, 10), true, random);
    small_door(world, p, piece.entry_door, pos(4, 1, 0));
    box_(world, p, pos(4, 1, 10), pos(6, 3, 10), air());
    box_(world, p, pos(0, 1, 4), pos(0, 3, 6), air());
    box_(world, p, pos(10, 1, 4), pos(10, 3, 6), air());
    match piece.room_type {
        0 => crossing_pillar(world, p),
        1 => crossing_fountain(world, p),
        2 => crossing_loft(world, p, random),
        3 | 4 => {}
        _ => unreachable!("crossing room type is bounded to five"),
    }
}

fn crossing_pillar(world: &mut impl StrongholdWorld, p: PiecePlacement<'_>) {
    box_(world, p, pos(5, 1, 5), pos(5, 3, 5), stone_bricks());
    for (local, direction) in [
        (pos(4, 3, 5), "west"),
        (pos(6, 3, 5), "east"),
        (pos(5, 3, 4), "south"),
        (pos(5, 3, 6), "north"),
    ] {
        p.place_block(world, local, facing("minecraft:wall_torch", direction));
    }
    for (x, z) in [
        (4, 4),
        (5, 4),
        (6, 4),
        (4, 5),
        (6, 5),
        (4, 6),
        (5, 6),
        (6, 6),
    ] {
        p.place_block(
            world,
            pos(x, 1, z),
            StructureState::new("minecraft:smooth_stone_slab"),
        );
    }
}

fn crossing_fountain(world: &mut impl StrongholdWorld, p: PiecePlacement<'_>) {
    for (minimum, maximum) in [
        (pos(3, 1, 3), pos(3, 1, 7)),
        (pos(7, 1, 3), pos(7, 1, 7)),
        (pos(4, 1, 3), pos(6, 1, 3)),
        (pos(4, 1, 7), pos(6, 1, 7)),
        (pos(5, 1, 5), pos(5, 3, 5)),
    ] {
        box_(world, p, minimum, maximum, stone_bricks());
    }
    p.place_block(world, pos(5, 4, 5), StructureState::new("minecraft:water"));
}

fn crossing_loft(
    world: &mut impl StrongholdWorld,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    for z in 1..=9 {
        for x in [1, 9] {
            p.place_block(
                world,
                pos(x, 3, z),
                StructureState::new("minecraft:cobblestone"),
            );
        }
    }
    for x in 1..=9 {
        for z in [1, 9] {
            p.place_block(
                world,
                pos(x, 3, z),
                StructureState::new("minecraft:cobblestone"),
            );
        }
    }
    for local in [
        pos(5, 1, 4),
        pos(5, 1, 6),
        pos(5, 3, 4),
        pos(5, 3, 6),
        pos(4, 1, 5),
        pos(6, 1, 5),
        pos(4, 3, 5),
        pos(6, 3, 5),
    ] {
        p.place_block(world, local, StructureState::new("minecraft:cobblestone"));
    }
    for y in 1..=3 {
        for (x, z) in [(4, 4), (6, 4), (4, 6), (6, 6)] {
            p.place_block(
                world,
                pos(x, y, z),
                StructureState::new("minecraft:cobblestone"),
            );
        }
    }
    p.place_block(
        world,
        pos(5, 3, 5),
        StructureState::new("minecraft:wall_torch"),
    );
    for z in 2..=8 {
        for x in [2, 3, 7, 8] {
            p.place_block(
                world,
                pos(x, 3, z),
                StructureState::new("minecraft:oak_planks"),
            );
        }
        if z <= 3 || z >= 7 {
            for x in 4..=6 {
                p.place_block(
                    world,
                    pos(x, 3, z),
                    StructureState::new("minecraft:oak_planks"),
                );
            }
        }
    }
    for y in 1..=3 {
        p.place_block(world, pos(9, y, 3), facing("minecraft:ladder", "west"));
    }
    create_chest(
        world,
        p,
        pos(3, 4, 8),
        "minecraft:chests/stronghold_crossing",
        random,
    );
}

fn straight_stairs(
    world: &mut impl StrongholdWorld,
    piece: &StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    shell(world, p, pos(4, 10, 7), true, random);
    small_door(world, p, piece.entry_door, pos(1, 7, 0));
    box_(world, p, pos(1, 1, 7), pos(3, 3, 7), air());
    for i in 0..6 {
        for x in 1..=3 {
            p.place_block(
                world,
                pos(x, 6 - i, 1 + i),
                facing("minecraft:cobblestone_stairs", "south"),
            );
            if i < 5 {
                p.place_block(world, pos(x, 5 - i, 1 + i), stone_bricks());
            }
        }
    }
}

fn five_crossing(
    world: &mut impl StrongholdWorld,
    piece: &StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    shell(world, p, pos(9, 8, 10), true, random);
    small_door(world, p, piece.entry_door, pos(4, 3, 0));
    if piece.low_left {
        box_(world, p, pos(0, 3, 1), pos(0, 5, 3), air());
    }
    if piece.low_right {
        box_(world, p, pos(9, 3, 1), pos(9, 5, 3), air());
    }
    if piece.high_left {
        box_(world, p, pos(0, 5, 7), pos(0, 7, 9), air());
    }
    if piece.high_right {
        box_(world, p, pos(9, 5, 7), pos(9, 7, 9), air());
    }
    box_(world, p, pos(5, 1, 10), pos(7, 3, 10), air());
    for (minimum, maximum) in [
        (pos(1, 2, 1), pos(8, 2, 6)),
        (pos(4, 1, 5), pos(4, 4, 9)),
        (pos(8, 1, 5), pos(8, 4, 9)),
        (pos(1, 4, 7), pos(3, 4, 9)),
        (pos(1, 3, 5), pos(3, 3, 6)),
    ] {
        selector_box(world, p, minimum, maximum, false, random);
    }
    box_(
        world,
        p,
        pos(1, 3, 4),
        pos(3, 3, 4),
        StructureState::new("minecraft:smooth_stone_slab"),
    );
    box_(
        world,
        p,
        pos(1, 4, 6),
        pos(3, 4, 6),
        StructureState::new("minecraft:smooth_stone_slab"),
    );
    selector_box(world, p, pos(5, 1, 7), pos(7, 1, 8), false, random);
    for (minimum, maximum) in [
        (pos(5, 1, 9), pos(7, 1, 9)),
        (pos(5, 2, 7), pos(7, 2, 7)),
        (pos(4, 5, 7), pos(4, 5, 9)),
        (pos(8, 5, 7), pos(8, 5, 9)),
    ] {
        box_(
            world,
            p,
            minimum,
            maximum,
            StructureState::new("minecraft:smooth_stone_slab"),
        );
    }
    box_(world, p, pos(5, 5, 7), pos(7, 5, 9), double_slab());
    p.place_block(world, pos(6, 5, 6), facing("minecraft:wall_torch", "south"));
}

fn chest_corridor(
    world: &mut impl StrongholdWorld,
    piece: &mut StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    shell(world, p, pos(4, 4, 6), true, random);
    small_door(world, p, piece.entry_door, pos(1, 1, 0));
    box_(world, p, pos(1, 1, 6), pos(3, 3, 6), air());
    box_(world, p, pos(3, 1, 2), pos(3, 1, 4), stone_bricks());
    for local in [pos(3, 1, 1), pos(3, 1, 5), pos(3, 2, 2), pos(3, 2, 4)] {
        p.place_block(
            world,
            local,
            StructureState::new("minecraft:stone_brick_slab"),
        );
    }
    box_(
        world,
        p,
        pos(2, 1, 2),
        pos(2, 1, 4),
        StructureState::new("minecraft:stone_brick_slab"),
    );
    let chest_position = p.piece.world_position(pos(3, 2, 3));
    if piece.chest_pending && p.clip.contains(chest_position) {
        piece.chest_pending = false;
        create_chest(
            world,
            p,
            pos(3, 2, 3),
            "minecraft:chests/stronghold_corridor",
            random,
        );
    }
}

fn library(
    world: &mut impl StrongholdWorld,
    piece: &StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    let height = if piece.tall_library { 11 } else { 6 };
    shell(world, p, pos(13, height - 1, 14), true, random);
    small_door(world, p, piece.entry_door, pos(4, 1, 0));
    for y in 1..=4 {
        for x in 2..=11 {
            for z in 1..=13 {
                if random.next_f32() <= 0.07 {
                    p.place_block(world, pos(x, y, z), StructureState::new("minecraft:cobweb"));
                }
            }
        }
    }
    for z in 1..=13 {
        let side = if matches!(z, 1 | 5 | 9 | 13) {
            "minecraft:oak_planks"
        } else {
            "minecraft:bookshelf"
        };
        box_(
            world,
            p,
            pos(1, 1, z),
            pos(1, 4, z),
            StructureState::new(side),
        );
        if piece.tall_library {
            box_(
                world,
                p,
                pos(1, 6, z),
                pos(1, 9, z),
                StructureState::new(side),
            );
            box_(
                world,
                p,
                pos(12, 6, z),
                pos(12, 9, z),
                StructureState::new(side),
            );
        }
        box_(
            world,
            p,
            pos(12, 1, z),
            pos(12, 4, z),
            StructureState::new(side),
        );
        if side == "minecraft:oak_planks" {
            p.place_block(world, pos(2, 3, z), facing("minecraft:wall_torch", "east"));
            p.place_block(world, pos(11, 3, z), facing("minecraft:wall_torch", "west"));
        }
    }
    for z in (3..=11).step_by(2) {
        box_(
            world,
            p,
            pos(3, 1, z),
            pos(4, 3, z),
            StructureState::new("minecraft:bookshelf"),
        );
        box_(
            world,
            p,
            pos(6, 1, z),
            pos(7, 3, z),
            StructureState::new("minecraft:bookshelf"),
        );
        box_(
            world,
            p,
            pos(9, 1, z),
            pos(10, 3, z),
            StructureState::new("minecraft:bookshelf"),
        );
    }
    if piece.tall_library {
        tall_library(world, p);
    }
    create_chest(
        world,
        p,
        pos(3, 3, 5),
        "minecraft:chests/stronghold_library",
        random,
    );
    if piece.tall_library {
        p.place_block(world, pos(12, 9, 1), air());
        create_chest(
            world,
            p,
            pos(12, 8, 1),
            "minecraft:chests/stronghold_library",
            random,
        );
    }
}

fn tall_library(world: &mut impl StrongholdWorld, p: PiecePlacement<'_>) {
    for (minimum, maximum) in [
        (pos(1, 5, 1), pos(3, 5, 13)),
        (pos(10, 5, 1), pos(12, 5, 13)),
        (pos(4, 5, 1), pos(9, 5, 2)),
        (pos(4, 5, 12), pos(9, 5, 13)),
    ] {
        box_(
            world,
            p,
            minimum,
            maximum,
            StructureState::new("minecraft:oak_planks"),
        );
    }
    for local in [pos(9, 5, 11), pos(8, 5, 11), pos(9, 5, 10)] {
        p.place_block(world, local, StructureState::new("minecraft:oak_planks"));
    }
    box_(
        world,
        p,
        pos(3, 6, 3),
        pos(3, 6, 11),
        fence(&["north", "south"]),
    );
    box_(
        world,
        p,
        pos(10, 6, 3),
        pos(10, 6, 9),
        fence(&["north", "south"]),
    );
    box_(
        world,
        p,
        pos(4, 6, 2),
        pos(9, 6, 2),
        fence(&["west", "east"]),
    );
    box_(
        world,
        p,
        pos(4, 6, 12),
        pos(7, 6, 12),
        fence(&["west", "east"]),
    );
    p.place_block(world, pos(3, 6, 2), fence(&["north", "east"]));
    p.place_block(world, pos(3, 6, 12), fence(&["south", "east"]));
    p.place_block(world, pos(10, 6, 2), fence(&["north", "west"]));
    for i in 0..=2 {
        p.place_block(world, pos(8 + i, 6, 12 - i), fence(&["south", "west"]));
        if i < 2 {
            p.place_block(world, pos(8 + i, 6, 11 - i), fence(&["north", "east"]));
        }
    }
    for y in 1..=7 {
        p.place_block(world, pos(10, y, 13), facing("minecraft:ladder", "south"));
    }
    for y in [8, 9] {
        p.place_block(world, pos(6, y, 7), fence(&["east"]));
        p.place_block(world, pos(7, y, 7), fence(&["west"]));
    }
    for (local, connections) in [
        (pos(6, 7, 7), &["north", "south", "west", "east"][..]),
        (pos(7, 7, 7), &["north", "south", "west", "east"][..]),
        (pos(5, 7, 7), &["east"][..]),
        (pos(8, 7, 7), &["west"][..]),
        (pos(6, 7, 6), &["north", "east"][..]),
        (pos(6, 7, 8), &["south", "east"][..]),
        (pos(7, 7, 6), &["north", "west"][..]),
        (pos(7, 7, 8), &["south", "west"][..]),
    ] {
        p.place_block(world, local, fence(connections));
    }
    for local in [
        pos(5, 8, 7),
        pos(8, 8, 7),
        pos(6, 8, 6),
        pos(6, 8, 8),
        pos(7, 8, 6),
        pos(7, 8, 8),
    ] {
        p.place_block(world, local, StructureState::new("minecraft:torch"));
    }
}

fn portal_room(
    world: &mut impl StrongholdWorld,
    piece: &mut StrongholdPiece,
    p: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    shell(world, p, pos(10, 7, 15), false, random);
    small_door(
        world,
        p,
        crate::generation::structure::stronghold_graph::StrongholdDoor::Grates,
        pos(4, 1, 0),
    );
    for (minimum, maximum) in [
        (pos(1, 6, 1), pos(1, 6, 14)),
        (pos(9, 6, 1), pos(9, 6, 14)),
        (pos(2, 6, 1), pos(8, 6, 2)),
        (pos(2, 6, 14), pos(8, 6, 14)),
        (pos(1, 1, 1), pos(2, 1, 4)),
        (pos(8, 1, 1), pos(9, 1, 4)),
        (pos(3, 1, 8), pos(7, 1, 12)),
        (pos(4, 1, 5), pos(6, 1, 7)),
        (pos(4, 2, 6), pos(6, 2, 7)),
        (pos(4, 3, 7), pos(6, 3, 7)),
    ] {
        selector_box(world, p, minimum, maximum, false, random);
    }
    box_(
        world,
        p,
        pos(1, 1, 1),
        pos(1, 1, 3),
        StructureState::new("minecraft:lava"),
    );
    box_(
        world,
        p,
        pos(9, 1, 1),
        pos(9, 1, 3),
        StructureState::new("minecraft:lava"),
    );
    box_(
        world,
        p,
        pos(4, 1, 9),
        pos(6, 1, 11),
        StructureState::new("minecraft:lava"),
    );
    let mut north_south = bars();
    north_south.properties.insert("north".into(), "true".into());
    north_south.properties.insert("south".into(), "true".into());
    let mut west_east = bars();
    west_east.properties.insert("west".into(), "true".into());
    west_east.properties.insert("east".into(), "true".into());
    for z in (3..14).step_by(2) {
        box_(world, p, pos(0, 3, z), pos(0, 4, z), north_south.clone());
        box_(world, p, pos(10, 3, z), pos(10, 4, z), north_south.clone());
    }
    for x in (2..9).step_by(2) {
        box_(world, p, pos(x, 3, 15), pos(x, 4, 15), west_east.clone());
    }
    for (y, z) in [(1, 4), (2, 5), (3, 6)] {
        for x in 4..=6 {
            p.place_block(
                world,
                pos(x, y, z),
                facing("minecraft:stone_brick_stairs", "north"),
            );
        }
    }
    let frames = [
        (pos(4, 3, 8), "north"),
        (pos(5, 3, 8), "north"),
        (pos(6, 3, 8), "north"),
        (pos(4, 3, 12), "south"),
        (pos(5, 3, 12), "south"),
        (pos(6, 3, 12), "south"),
        (pos(3, 3, 9), "east"),
        (pos(3, 3, 10), "east"),
        (pos(3, 3, 11), "east"),
        (pos(7, 3, 9), "west"),
        (pos(7, 3, 10), "west"),
        (pos(7, 3, 11), "west"),
    ];
    let eyes = std::array::from_fn::<_, 12, _>(|_| random.next_f32() > 0.9);
    for ((local, direction), eye) in frames.into_iter().zip(eyes) {
        p.place_block(
            world,
            local,
            state(
                "minecraft:end_portal_frame",
                &[('f', direction), ('e', if eye { "true" } else { "false" })],
            ),
        );
    }
    if eyes.into_iter().all(|eye| eye) {
        box_(
            world,
            p,
            pos(4, 3, 9),
            pos(6, 3, 11),
            StructureState::new("minecraft:end_portal"),
        );
    }
    let spawner = p.piece.world_position(pos(5, 3, 6));
    if !piece.spawner_placed && p.clip.contains(spawner) {
        piece.spawner_placed = true;
        crate::generation::structure::piece::PieceWorld::set_state(
            world,
            spawner,
            StructureState::new("minecraft:spawner"),
            2,
        );
        if world.is_silverfish_spawner_block_entity(spawner) {
            world.configure_silverfish_spawner(spawner, random);
        }
    }
}

fn double_slab() -> StructureState {
    let mut slab = StructureState::new("minecraft:smooth_stone_slab");
    slab.properties.insert("type".into(), "double".into());
    slab
}

fn fence(connections: &[&str]) -> StructureState {
    let mut state = StructureState::new("minecraft:oak_fence");
    for direction in connections {
        state.properties.insert((*direction).into(), "true".into());
    }
    state
}
