//! Woodland-mansion room-family selection and transform tables.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::mansion_graph::{ROOM_1X1, ROOM_1X2, ROOM_2X2};
use crate::generation::structure::mansion_pieces::{
    MansionPieceSpec, random_index, relative_rotated, rotate,
};
use crate::generation::structure::template_place::TemplateMirror;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RoomCollection {
    First,
    Upper,
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source room dispatch is keyed by all seven independent graph fields"
)]
pub(crate) fn add_room(
    pieces: &mut Vec<MansionPieceSpec>,
    room_position: BlockPos,
    rotation: Rotation,
    room_type: i32,
    room_direction: Option<Direction>,
    door_direction: Option<Direction>,
    collection: RoomCollection,
    stairs: bool,
    random: &mut impl GenerationRandom,
) {
    match room_type {
        ROOM_1X1 => add_1x1(
            pieces,
            room_position,
            rotation,
            door_direction,
            collection,
            random,
        ),
        ROOM_1X2 => {
            if let (Some(room_direction), Some(door_direction)) = (room_direction, door_direction) {
                add_1x2(
                    pieces,
                    room_position,
                    rotation,
                    room_direction,
                    door_direction,
                    collection,
                    stairs,
                    random,
                );
            }
        }
        ROOM_2X2 => match door_direction {
            Some(Direction::Up) => {
                add_2x2_secret(pieces, room_position, rotation, collection, random)
            }
            Some(door_direction) => {
                let Some(room_direction) = room_direction else {
                    return;
                };
                add_2x2(
                    pieces,
                    room_position,
                    rotation,
                    room_direction,
                    door_direction,
                    collection,
                    random,
                );
            }
            None => {}
        },
        _ => {}
    }
}

fn add_1x1(
    pieces: &mut Vec<MansionPieceSpec>,
    room_position: BlockPos,
    mansion_rotation: Rotation,
    door: Option<Direction>,
    collection: RoomCollection,
    random: &mut impl GenerationRandom,
) {
    let mut piece_rotation = Rotation::None;
    let mut template = collection.one_by_one(random);
    match door {
        Some(Direction::East) => {}
        Some(Direction::North) => piece_rotation = Rotation::CounterClockwise90,
        Some(Direction::West) => piece_rotation = Rotation::Clockwise180,
        Some(Direction::South) => piece_rotation = Rotation::Clockwise90,
        _ => template = collection.one_by_one_secret(random),
    }
    let (offset_x, offset_z) = match piece_rotation {
        Rotation::None => (1, 0),
        Rotation::Clockwise90 => (8, 0),
        Rotation::Clockwise180 => (8, 7),
        Rotation::CounterClockwise90 => (1, 7),
    };
    let position = local_offset(room_position, mansion_rotation, offset_x, offset_z);
    pieces.push(MansionPieceSpec::new(
        &template,
        position,
        rotate(piece_rotation, mansion_rotation),
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "the source 1x2 transform table has independent door, axis, floor, and stairs inputs"
)]
fn add_1x2(
    pieces: &mut Vec<MansionPieceSpec>,
    room_position: BlockPos,
    rotation: Rotation,
    room: Direction,
    door: Direction,
    collection: RoomCollection,
    stairs: bool,
    random: &mut impl GenerationRandom,
) {
    let (family, east, south, piece_rotation, mirror) = match (door, room) {
        (Direction::East, Direction::South) => {
            (RoomFamily::Side, 1, 0, Rotation::None, TemplateMirror::None)
        }
        (Direction::East, Direction::North) => (
            RoomFamily::Side,
            1,
            6,
            Rotation::None,
            TemplateMirror::LeftRight,
        ),
        (Direction::West, Direction::North) => (
            RoomFamily::Side,
            7,
            6,
            Rotation::Clockwise180,
            TemplateMirror::None,
        ),
        (Direction::West, Direction::South) => (
            RoomFamily::Side,
            7,
            0,
            Rotation::None,
            TemplateMirror::FrontBack,
        ),
        (Direction::South, Direction::East) => (
            RoomFamily::Side,
            1,
            0,
            Rotation::Clockwise90,
            TemplateMirror::LeftRight,
        ),
        (Direction::South, Direction::West) => (
            RoomFamily::Side,
            7,
            0,
            Rotation::Clockwise90,
            TemplateMirror::None,
        ),
        (Direction::North, Direction::West) => (
            RoomFamily::Side,
            7,
            6,
            Rotation::Clockwise90,
            TemplateMirror::FrontBack,
        ),
        (Direction::North, Direction::East) => (
            RoomFamily::Side,
            1,
            6,
            Rotation::CounterClockwise90,
            TemplateMirror::None,
        ),
        (Direction::South, Direction::North) => (
            RoomFamily::Front,
            1,
            -8,
            Rotation::None,
            TemplateMirror::None,
        ),
        (Direction::North, Direction::South) => (
            RoomFamily::Front,
            7,
            14,
            Rotation::Clockwise180,
            TemplateMirror::None,
        ),
        (Direction::West, Direction::East) => (
            RoomFamily::Front,
            15,
            0,
            Rotation::Clockwise90,
            TemplateMirror::None,
        ),
        (Direction::East, Direction::West) => (
            RoomFamily::Front,
            -7,
            6,
            Rotation::CounterClockwise90,
            TemplateMirror::None,
        ),
        (Direction::Up, Direction::East) => (
            RoomFamily::Secret,
            15,
            0,
            Rotation::Clockwise90,
            TemplateMirror::None,
        ),
        (Direction::Up, Direction::South) => (
            RoomFamily::Secret,
            1,
            0,
            Rotation::None,
            TemplateMirror::None,
        ),
        _ => return,
    };
    let template = match family {
        RoomFamily::Side => collection.one_by_two_side(random, stairs),
        RoomFamily::Front => collection.one_by_two_front(random, stairs),
        RoomFamily::Secret => collection.one_by_two_secret(random),
    };
    pieces.push(transformed(
        template,
        room_position,
        rotation,
        east,
        south,
        piece_rotation,
        mirror,
    ));
}

enum RoomFamily {
    Side,
    Front,
    Secret,
}

fn add_2x2(
    pieces: &mut Vec<MansionPieceSpec>,
    room_position: BlockPos,
    rotation: Rotation,
    room: Direction,
    door: Direction,
    collection: RoomCollection,
    random: &mut impl GenerationRandom,
) {
    let Some((east, south, piece_rotation, mirror)) = (match (door, room) {
        (Direction::East, Direction::South) => Some((-7, 0, Rotation::None, TemplateMirror::None)),
        (Direction::East, Direction::North) => {
            Some((-7, 6, Rotation::None, TemplateMirror::LeftRight))
        }
        (Direction::North, Direction::East) => {
            Some((1, 14, Rotation::CounterClockwise90, TemplateMirror::None))
        }
        (Direction::North, Direction::West) => Some((
            7,
            14,
            Rotation::CounterClockwise90,
            TemplateMirror::LeftRight,
        )),
        (Direction::South, Direction::West) => {
            Some((7, -8, Rotation::Clockwise90, TemplateMirror::None))
        }
        (Direction::South, Direction::East) => {
            Some((1, -8, Rotation::Clockwise90, TemplateMirror::LeftRight))
        }
        (Direction::West, Direction::North) => {
            Some((15, 6, Rotation::Clockwise180, TemplateMirror::None))
        }
        (Direction::West, Direction::South) => {
            Some((15, 0, Rotation::None, TemplateMirror::FrontBack))
        }
        _ => None,
    }) else {
        return;
    };
    pieces.push(transformed(
        collection.two_by_two(random),
        room_position,
        rotation,
        east,
        south,
        piece_rotation,
        mirror,
    ));
}

fn add_2x2_secret(
    pieces: &mut Vec<MansionPieceSpec>,
    room_position: BlockPos,
    rotation: Rotation,
    collection: RoomCollection,
    random: &mut impl GenerationRandom,
) {
    pieces.push(transformed(
        collection.two_by_two_secret(random),
        room_position,
        rotation,
        1,
        0,
        Rotation::None,
        TemplateMirror::None,
    ));
}

fn transformed(
    template: String,
    room_position: BlockPos,
    mansion_rotation: Rotation,
    east: i32,
    south: i32,
    piece_rotation: Rotation,
    mirror: TemplateMirror,
) -> MansionPieceSpec {
    MansionPieceSpec::mirrored(
        &template,
        local_offset(room_position, mansion_rotation, east, south),
        rotate(mansion_rotation, piece_rotation),
        mirror,
    )
}

fn local_offset(position: BlockPos, rotation: Rotation, east: i32, south: i32) -> BlockPos {
    relative_rotated(
        relative_rotated(position, rotation, Direction::East, east),
        rotation,
        Direction::South,
        south,
    )
}

impl RoomCollection {
    fn one_by_one(self, random: &mut impl GenerationRandom) -> String {
        let family = if self == Self::First { 'a' } else { 'b' };
        format!("1x1_{family}{}", random_index(random, 5) + 1)
    }

    fn one_by_one_secret(self, random: &mut impl GenerationRandom) -> String {
        format!("1x1_as{}", random_index(random, 4) + 1)
    }

    fn one_by_two_side(self, random: &mut impl GenerationRandom, stairs: bool) -> String {
        if self == Self::Upper && stairs {
            "1x2_c_stairs".into()
        } else if self == Self::First {
            format!("1x2_a{}", random_index(random, 9) + 1)
        } else {
            format!("1x2_c{}", random_index(random, 4) + 1)
        }
    }

    fn one_by_two_front(self, random: &mut impl GenerationRandom, stairs: bool) -> String {
        if self == Self::Upper && stairs {
            "1x2_d_stairs".into()
        } else if self == Self::First {
            format!("1x2_b{}", random_index(random, 5) + 1)
        } else {
            format!("1x2_d{}", random_index(random, 5) + 1)
        }
    }

    fn one_by_two_secret(self, random: &mut impl GenerationRandom) -> String {
        if self == Self::First {
            format!("1x2_s{}", random_index(random, 2) + 1)
        } else {
            let _ = random_index(random, 1);
            "1x2_se1".into()
        }
    }

    fn two_by_two(self, random: &mut impl GenerationRandom) -> String {
        let (family, count) = if self == Self::First {
            ('a', 4)
        } else {
            ('b', 5)
        };
        format!("2x2_{family}{}", random_index(random, count) + 1)
    }

    fn two_by_two_secret(self, _random: &mut impl GenerationRandom) -> String {
        "2x2_s1".into()
    }
}
