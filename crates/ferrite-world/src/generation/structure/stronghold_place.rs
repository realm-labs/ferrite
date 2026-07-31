//! Shared stronghold masonry, doors, state helpers, and placement dispatch.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::{OrientedPiece, PiecePlacement, PieceWorld};
use crate::generation::structure::processor::StructureState;
use crate::generation::structure::stronghold_graph::{
    StrongholdDoor, StrongholdPiece, StrongholdPieceKind,
};
use crate::generation::structure::stronghold_rooms_basic::place_basic_piece;
use crate::generation::structure::stronghold_rooms_special::place_special_piece;

pub trait StrongholdWorld: PieceWorld {
    fn is_silverfish_spawner_block_entity(&mut self, position: BlockPos) -> bool;

    fn configure_silverfish_spawner(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    );
}

pub fn place_stronghold_piece(
    world: &mut impl StrongholdWorld,
    piece: &mut StrongholdPiece,
    clip: &BlockBox,
    random: &mut impl GenerationRandom,
) {
    let p = placement(piece, clip);
    match piece.kind {
        StrongholdPieceKind::Start
        | StrongholdPieceKind::Straight
        | StrongholdPieceKind::PrisonHall
        | StrongholdPieceKind::LeftTurn
        | StrongholdPieceKind::RightTurn
        | StrongholdPieceKind::FillerCorridor => place_basic_piece(world, piece, p, random),
        StrongholdPieceKind::RoomCrossing
        | StrongholdPieceKind::StraightStairsDown
        | StrongholdPieceKind::StairsDown
        | StrongholdPieceKind::FiveCrossing
        | StrongholdPieceKind::ChestCorridor
        | StrongholdPieceKind::Library
        | StrongholdPieceKind::PortalRoom => place_special_piece(world, piece, p, random),
    }
}

pub(crate) fn placement<'a>(piece: &StrongholdPiece, clip: &'a BlockBox) -> PiecePlacement<'a> {
    PiecePlacement {
        piece: OrientedPiece {
            bounds: piece.bounding_box,
            orientation: piece.orientation,
        },
        clip,
    }
}

pub(crate) fn shell(
    world: &mut impl StrongholdWorld,
    p: PiecePlacement<'_>,
    maximum: BlockPos,
    skip_air: bool,
    random: &mut impl GenerationRandom,
) {
    selector_box(world, p, BlockPos::new(0, 0, 0), maximum, skip_air, random);
}

pub(crate) fn selector_box(
    world: &mut impl StrongholdWorld,
    p: PiecePlacement<'_>,
    minimum: BlockPos,
    maximum: BlockPos,
    skip_air: bool,
    random: &mut impl GenerationRandom,
) {
    for y in minimum.y..=maximum.y {
        for x in minimum.x..=maximum.x {
            for z in minimum.z..=maximum.z {
                let local = BlockPos::new(x, y, z);
                let position = p.piece.world_position(local);
                let existing = if p.clip.contains(position) {
                    world.state_at(position)
                } else {
                    air()
                };
                if skip_air
                    && matches!(
                        existing.block.as_str(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                    )
                {
                    continue;
                }
                let edge = x == minimum.x
                    || x == maximum.x
                    || y == minimum.y
                    || y == maximum.y
                    || z == minimum.z
                    || z == maximum.z;
                let state = if edge {
                    selected_stone(random.next_f32())
                } else {
                    air()
                };
                p.place_block(world, local, state);
            }
        }
    }
}

pub(crate) fn box_(
    world: &mut impl StrongholdWorld,
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

pub(crate) fn small_door(
    world: &mut impl StrongholdWorld,
    p: PiecePlacement<'_>,
    door: StrongholdDoor,
    origin: BlockPos,
) {
    let BlockPos { x, y, z } = origin;
    match door {
        StrongholdDoor::Opening => box_(world, p, pos(x, y, z), pos(x + 2, y + 2, z), air()),
        StrongholdDoor::Wood | StrongholdDoor::Iron => {
            let frame = stone_bricks();
            for (minimum, maximum) in [
                (pos(x, y, z), pos(x, y + 2, z)),
                (pos(x + 2, y, z), pos(x + 2, y + 2, z)),
                (pos(x + 1, y + 2, z), pos(x + 1, y + 2, z)),
            ] {
                box_(world, p, minimum, maximum, frame.clone());
            }
            let block = if door == StrongholdDoor::Wood {
                "minecraft:oak_door"
            } else {
                "minecraft:iron_door"
            };
            p.place_block(
                world,
                pos(x + 1, y, z),
                state(block, &[('f', "north"), ('h', "lower")]),
            );
            p.place_block(
                world,
                pos(x + 1, y + 1, z),
                state(block, &[('f', "north"), ('h', "upper")]),
            );
            if door == StrongholdDoor::Iron {
                p.place_block(
                    world,
                    pos(x + 2, y + 1, z + 1),
                    facing("minecraft:stone_button", "north"),
                );
                p.place_block(
                    world,
                    pos(x + 2, y + 1, z - 1),
                    facing("minecraft:stone_button", "south"),
                );
            }
        }
        StrongholdDoor::Grates => {
            box_(world, p, pos(x + 1, y, z), pos(x + 1, y + 1, z), air());
            let mut west = bars();
            west.properties.insert("west".into(), "true".into());
            let mut east = bars();
            east.properties.insert("east".into(), "true".into());
            box_(world, p, pos(x, y, z), pos(x, y + 1, z), west);
            box_(world, p, pos(x + 2, y, z), pos(x + 2, y + 1, z), east);
            let mut top = bars();
            top.properties.insert("west".into(), "true".into());
            top.properties.insert("east".into(), "true".into());
            box_(world, p, pos(x, y + 2, z), pos(x + 2, y + 2, z), top);
        }
    }
}

pub(crate) fn create_chest(
    world: &mut impl StrongholdWorld,
    p: PiecePlacement<'_>,
    local: BlockPos,
    table: &str,
    random: &mut impl GenerationRandom,
) -> bool {
    p.create_chest(world, local, table, || next_i64(random))
}

pub(crate) fn selected_stone(value: f32) -> StructureState {
    if value < 0.2 {
        StructureState::new("minecraft:cracked_stone_bricks")
    } else if value < 0.5 {
        StructureState::new("minecraft:mossy_stone_bricks")
    } else if value < 0.55 {
        StructureState::new("minecraft:infested_stone_bricks")
    } else {
        stone_bricks()
    }
}

pub(crate) fn state(block: &str, properties: &[(char, &str)]) -> StructureState {
    let mut state = StructureState::new(block);
    for (key, value) in properties {
        let name = match key {
            'f' => "facing",
            'h' => "half",
            't' => "type",
            'e' => "eye",
            _ => unreachable!("known compact stronghold property key"),
        };
        state.properties.insert(name.into(), (*value).into());
    }
    state
}

pub(crate) fn facing(block: &str, direction: &str) -> StructureState {
    state(block, &[('f', direction)])
}

pub(crate) fn stone_bricks() -> StructureState {
    StructureState::new("minecraft:stone_bricks")
}

pub(crate) fn air() -> StructureState {
    StructureState::new("minecraft:cave_air")
}

pub(crate) fn bars() -> StructureState {
    StructureState::new("minecraft:iron_bars")
}

pub(crate) fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos::new(x, y, z)
}

fn next_i64(random: &mut impl GenerationRandom) -> i64 {
    (i64::from(random.next_i32()) << 32).wrapping_add(i64::from(random.next_i32()))
}
