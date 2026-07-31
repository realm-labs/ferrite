//! Ordered woodland-mansion template-piece scheduling.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::mansion_graph::{
    CORRIDOR, MansionLayout, ROOM, ROOM_CORRIDOR, ROOM_DOOR, ROOM_ID_MASK, ROOM_ORIGIN,
    ROOM_STAIRS, ROOM_TYPE_MASK, START_ROOM,
};
use crate::generation::structure::mansion_roof::create_roof;
use crate::generation::structure::mansion_rooms::{RoomCollection, add_room};
use crate::generation::structure::template_place::TemplateMirror;

pub(crate) const HORIZONTAL: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MansionPieceSpec {
    pub template: String,
    pub position: BlockPos,
    pub rotation: Rotation,
    pub mirror: TemplateMirror,
}

impl MansionPieceSpec {
    pub(crate) fn new(template: &str, position: BlockPos, rotation: Rotation) -> Self {
        Self {
            template: template.into(),
            position,
            rotation,
            mirror: TemplateMirror::None,
        }
    }

    pub(crate) fn mirrored(
        template: &str,
        position: BlockPos,
        rotation: Rotation,
        mirror: TemplateMirror,
    ) -> Self {
        Self {
            template: template.into(),
            position,
            rotation,
            mirror,
        }
    }
}

pub fn generate_mansion_specs(
    origin: BlockPos,
    rotation: Rotation,
    random: &mut impl GenerationRandom,
) -> (MansionLayout, Vec<MansionPieceSpec>) {
    let layout = MansionLayout::generate(random);
    let pieces = create_mansion(origin, rotation, &layout, random);
    (layout, pieces)
}

pub fn create_mansion(
    origin: BlockPos,
    rotation: Rotation,
    layout: &MansionLayout,
    random: &mut impl GenerationRandom,
) -> Vec<MansionPieceSpec> {
    let mut placer = MansionPiecePlacer {
        random,
        start_x: layout.entrance_x + 1,
        start_y: layout.entrance_y + 1,
    };
    let mut pieces = Vec::new();
    let mut first = PlacementData {
        position: origin,
        rotation,
        wall: "wall_flat",
    };
    entrance(&mut pieces, &mut first);
    let mut second = PlacementData {
        position: above(first.position, 8),
        rotation: first.rotation,
        wall: "wall_window",
    };
    let end_x = layout.entrance_x + 1;
    let end_y = layout.entrance_y;
    traverse_outer_walls(
        &mut pieces,
        &mut first,
        &layout.base,
        Direction::South,
        placer.start_x,
        placer.start_y,
        end_x,
        end_y,
    );
    traverse_outer_walls(
        &mut pieces,
        &mut second,
        &layout.base,
        Direction::South,
        placer.start_x,
        placer.start_y,
        end_x,
        end_y,
    );
    let mut third_data = PlacementData {
        position: above(first.position, 19),
        rotation: first.rotation,
        wall: "wall_window",
    };
    'third: for y in 0..layout.third.height() as i32 {
        for x in (0..layout.third.width() as i32).rev() {
            if !MansionLayout::is_house(&layout.third, x, y) {
                continue;
            }
            third_data.position = relative_rotated(
                third_data.position,
                rotation,
                Direction::South,
                8 + (y - placer.start_y) * 8,
            );
            third_data.position = relative_rotated(
                third_data.position,
                rotation,
                Direction::East,
                (x - placer.start_x) * 8,
            );
            traverse_wall_piece(&mut pieces, &mut third_data);
            traverse_outer_walls(
                &mut pieces,
                &mut third_data,
                &layout.third,
                Direction::South,
                x,
                y,
                x,
                y,
            );
            break 'third;
        }
    }
    create_roof(
        &mut pieces,
        above(origin, 16),
        rotation,
        &layout.base,
        Some(&layout.third),
        placer.start_x,
        placer.start_y,
    );
    create_roof(
        &mut pieces,
        above(origin, 27),
        rotation,
        &layout.third,
        None,
        placer.start_x,
        placer.start_y,
    );
    placer.create_floors(&mut pieces, origin, rotation, layout);
    pieces
}

struct MansionPiecePlacer<'a, R> {
    random: &'a mut R,
    start_x: i32,
    start_y: i32,
}

impl<R: GenerationRandom> MansionPiecePlacer<'_, R> {
    fn create_floors(
        &mut self,
        pieces: &mut Vec<MansionPieceSpec>,
        origin: BlockPos,
        rotation: Rotation,
        layout: &MansionLayout,
    ) {
        for floor in 0..3 {
            let floor_origin = above(origin, 8 * floor as i32 + i32::from(floor == 2) * 3);
            let rooms = &layout.floor_rooms[floor];
            let grid = if floor == 2 {
                &layout.third
            } else {
                &layout.base
            };
            self.create_corridors(pieces, floor_origin, rotation, floor, grid, rooms);
            self.create_rooms(pieces, floor_origin, rotation, floor, layout);
        }
    }

    fn create_corridors(
        &mut self,
        pieces: &mut Vec<MansionPieceSpec>,
        floor_origin: BlockPos,
        rotation: Rotation,
        floor: usize,
        grid: &crate::generation::structure::mansion_graph::SimpleGrid,
        rooms: &crate::generation::structure::mansion_graph::SimpleGrid,
    ) {
        let south = if floor == 0 {
            "carpet_south_1"
        } else {
            "carpet_south_2"
        };
        let west = if floor == 0 {
            "carpet_west_1"
        } else {
            "carpet_west_2"
        };
        for y in 0..grid.height() as i32 {
            for x in 0..grid.width() as i32 {
                if grid.get(x, y) != CORRIDOR {
                    continue;
                }
                let mut position = relative_rotated(
                    floor_origin,
                    rotation,
                    Direction::South,
                    8 + (y - self.start_y) * 8,
                );
                position =
                    relative_rotated(position, rotation, Direction::East, (x - self.start_x) * 8);
                push(pieces, "corridor_floor", position, rotation);
                if connected(grid, rooms, x, y - 1) {
                    let position =
                        above(relative_rotated(position, rotation, Direction::East, 1), 1);
                    push(pieces, "carpet_north", position, rotation);
                }
                if connected(grid, rooms, x + 1, y) {
                    let position = relative_rotated(
                        relative_rotated(position, rotation, Direction::South, 1),
                        rotation,
                        Direction::East,
                        5,
                    );
                    push(pieces, "carpet_east", above(position, 1), rotation);
                }
                if connected(grid, rooms, x, y + 1) {
                    let position = relative_rotated(
                        relative_rotated(position, rotation, Direction::South, 5),
                        rotation,
                        Direction::West,
                        1,
                    );
                    push(pieces, south, position, rotation);
                }
                if connected(grid, rooms, x - 1, y) {
                    let position = relative_rotated(
                        relative_rotated(position, rotation, Direction::West, 1),
                        rotation,
                        Direction::North,
                        1,
                    );
                    push(pieces, west, position, rotation);
                }
            }
        }
    }

    fn create_rooms(
        &mut self,
        pieces: &mut Vec<MansionPieceSpec>,
        floor_origin: BlockPos,
        rotation: Rotation,
        floor: usize,
        layout: &MansionLayout,
    ) {
        let rooms = &layout.floor_rooms[floor];
        let grid = if floor == 2 {
            &layout.third
        } else {
            &layout.base
        };
        let wall = if floor == 0 {
            "indoors_wall_1"
        } else {
            "indoors_wall_2"
        };
        let door = if floor == 0 {
            "indoors_door_1"
        } else {
            "indoors_door_2"
        };
        let collection = if floor == 0 {
            RoomCollection::First
        } else {
            RoomCollection::Upper
        };
        for y in 0..grid.height() as i32 {
            for x in 0..grid.width() as i32 {
                let is_third_start = floor == 2 && grid.get(x, y) == START_ROOM;
                if grid.get(x, y) != ROOM && !is_third_start {
                    continue;
                }
                let data = rooms.get(x, y);
                let room_type = data & ROOM_TYPE_MASK;
                let room_id = data & ROOM_ID_MASK;
                let stair_endpoint = is_third_start && data & ROOM_CORRIDOR == ROOM_CORRIDOR;
                let mut door_directions = Vec::new();
                if data & ROOM_DOOR == ROOM_DOOR {
                    for direction in HORIZONTAL {
                        let [dx, _, dz] = direction.step();
                        if grid.get(x + dx, y + dz) == CORRIDOR {
                            door_directions.push(direction);
                        }
                    }
                }
                let door_direction = if door_directions.is_empty() {
                    (data & ROOM_ORIGIN == ROOM_ORIGIN).then_some(Direction::Up)
                } else {
                    let index = random_index(self.random, door_directions.len());
                    Some(door_directions[index])
                };
                let mut room_position = relative_rotated(
                    floor_origin,
                    rotation,
                    Direction::South,
                    8 + (y - self.start_y) * 8,
                );
                room_position = relative_rotated(
                    room_position,
                    rotation,
                    Direction::East,
                    -1 + (x - self.start_x) * 8,
                );
                add_boundaries(
                    pieces,
                    room_position,
                    rotation,
                    x,
                    y,
                    floor,
                    room_id,
                    grid,
                    layout,
                    stair_endpoint,
                    door_direction,
                    wall,
                    door,
                );
                let room_direction =
                    if room_type == crate::generation::structure::mansion_graph::ROOM_2X2 {
                        door_direction.and_then(|door| {
                            if door == Direction::Up {
                                return None;
                            }
                            let side = crate::generation::structure::mansion_graph::clockwise(door);
                            let [dx, _, dz] = side.step();
                            Some(if layout.is_room_id(x + dx, y + dz, floor, room_id) {
                                side
                            } else {
                                side.opposite()
                            })
                        })
                    } else {
                        layout.room_direction(x, y, floor, room_id)
                    };
                add_room(
                    pieces,
                    room_position,
                    rotation,
                    room_type,
                    room_direction,
                    door_direction,
                    collection,
                    data & ROOM_STAIRS == ROOM_STAIRS,
                    self.random,
                );
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the four source boundary directions share one room-cell transaction"
)]
fn add_boundaries(
    pieces: &mut Vec<MansionPieceSpec>,
    room_position: BlockPos,
    rotation: Rotation,
    x: i32,
    y: i32,
    floor: usize,
    room_id: i32,
    grid: &crate::generation::structure::mansion_graph::SimpleGrid,
    layout: &MansionLayout,
    stair_endpoint: bool,
    door_direction: Option<Direction>,
    wall: &str,
    door: &str,
) {
    if MansionLayout::is_house(grid, x - 1, y) && !layout.is_room_id(x - 1, y, floor, room_id) {
        push(
            pieces,
            if door_direction == Some(Direction::West) {
                door
            } else {
                wall
            },
            room_position,
            rotation,
        );
    }
    if grid.get(x + 1, y) == CORRIDOR && !stair_endpoint {
        push(
            pieces,
            if door_direction == Some(Direction::East) {
                door
            } else {
                wall
            },
            relative_rotated(room_position, rotation, Direction::East, 8),
            rotation,
        );
    }
    if MansionLayout::is_house(grid, x, y + 1) && !layout.is_room_id(x, y + 1, floor, room_id) {
        let position = relative_rotated(
            relative_rotated(room_position, rotation, Direction::South, 7),
            rotation,
            Direction::East,
            7,
        );
        push(
            pieces,
            if door_direction == Some(Direction::South) {
                door
            } else {
                wall
            },
            position,
            rotate(rotation, Rotation::Clockwise90),
        );
    }
    if grid.get(x, y - 1) == CORRIDOR && !stair_endpoint {
        let position = relative_rotated(
            relative_rotated(room_position, rotation, Direction::North, 1),
            rotation,
            Direction::East,
            7,
        );
        push(
            pieces,
            if door_direction == Some(Direction::North) {
                door
            } else {
                wall
            },
            position,
            rotate(rotation, Rotation::Clockwise90),
        );
    }
}

#[derive(Clone, Copy)]
struct PlacementData {
    position: BlockPos,
    rotation: Rotation,
    wall: &'static str,
}

fn entrance(pieces: &mut Vec<MansionPieceSpec>, data: &mut PlacementData) {
    push(
        pieces,
        "entrance",
        relative_rotated(data.position, data.rotation, Direction::West, 9),
        data.rotation,
    );
    data.position = relative_rotated(data.position, data.rotation, Direction::South, 16);
}

#[expect(
    clippy::too_many_arguments,
    reason = "the perimeter cursor requires both start and terminal grid states"
)]
fn traverse_outer_walls(
    pieces: &mut Vec<MansionPieceSpec>,
    data: &mut PlacementData,
    grid: &crate::generation::structure::mansion_graph::SimpleGrid,
    mut direction: Direction,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) {
    let (mut x, mut y) = (start_x, start_y);
    let start_direction = direction;
    loop {
        let [dx, _, dz] = direction.step();
        if !MansionLayout::is_house(grid, x + dx, y + dz) {
            traverse_turn(pieces, data);
            direction = crate::generation::structure::mansion_graph::clockwise(direction);
            if (x, y, direction) != (end_x, end_y, start_direction) {
                traverse_wall_piece(pieces, data);
            }
        } else {
            let counter = crate::generation::structure::mansion_graph::counterclockwise(direction);
            let [cx, _, cz] = counter.step();
            if MansionLayout::is_house(grid, x + dx + cx, y + dz + cz) {
                traverse_inner_turn(data);
                x += dx;
                y += dz;
                direction = counter;
            } else {
                x += dx;
                y += dz;
                if (x, y, direction) != (end_x, end_y, start_direction) {
                    traverse_wall_piece(pieces, data);
                }
            }
        }
        if (x, y, direction) == (end_x, end_y, start_direction) {
            break;
        }
    }
}

fn traverse_wall_piece(pieces: &mut Vec<MansionPieceSpec>, data: &mut PlacementData) {
    push(
        pieces,
        data.wall,
        relative_rotated(data.position, data.rotation, Direction::East, 7),
        data.rotation,
    );
    data.position = relative_rotated(data.position, data.rotation, Direction::South, 8);
}

fn traverse_turn(pieces: &mut Vec<MansionPieceSpec>, data: &mut PlacementData) {
    data.position = relative_rotated(data.position, data.rotation, Direction::South, -1);
    push(pieces, "wall_corner", data.position, data.rotation);
    data.position = relative_rotated(data.position, data.rotation, Direction::South, -7);
    data.position = relative_rotated(data.position, data.rotation, Direction::West, -6);
    data.rotation = rotate(data.rotation, Rotation::Clockwise90);
}

fn traverse_inner_turn(data: &mut PlacementData) {
    data.position = relative_rotated(data.position, data.rotation, Direction::South, 6);
    data.position = relative_rotated(data.position, data.rotation, Direction::East, 8);
    data.rotation = rotate(data.rotation, Rotation::CounterClockwise90);
}

fn connected(
    grid: &crate::generation::structure::mansion_graph::SimpleGrid,
    rooms: &crate::generation::structure::mansion_graph::SimpleGrid,
    x: i32,
    y: i32,
) -> bool {
    grid.get(x, y) == CORRIDOR || rooms.get(x, y) & ROOM_CORRIDOR == ROOM_CORRIDOR
}

pub(crate) fn push(
    pieces: &mut Vec<MansionPieceSpec>,
    template: &str,
    position: BlockPos,
    rotation: Rotation,
) {
    pieces.push(MansionPieceSpec::new(template, position, rotation));
}

pub(crate) fn above(mut position: BlockPos, distance: i32) -> BlockPos {
    position.y = position.y.wrapping_add(distance);
    position
}

pub(crate) fn relative_rotated(
    position: BlockPos,
    rotation: Rotation,
    direction: Direction,
    distance: i32,
) -> BlockPos {
    relative(position, rotation.rotate_direction(direction), distance)
}

fn relative(mut position: BlockPos, direction: Direction, distance: i32) -> BlockPos {
    let [dx, dy, dz] = direction.step();
    position.x = position.x.wrapping_add(dx.wrapping_mul(distance));
    position.y = position.y.wrapping_add(dy.wrapping_mul(distance));
    position.z = position.z.wrapping_add(dz.wrapping_mul(distance));
    position
}

pub(crate) fn rotate(rotation: Rotation, added: Rotation) -> Rotation {
    let quarter = |value| match value {
        Rotation::None => 0,
        Rotation::Clockwise90 => 1,
        Rotation::Clockwise180 => 2,
        Rotation::CounterClockwise90 => 3,
    };
    match (quarter(rotation) + quarter(added)) & 3 {
        0 => Rotation::None,
        1 => Rotation::Clockwise90,
        2 => Rotation::Clockwise180,
        _ => Rotation::CounterClockwise90,
    }
}

pub(crate) fn random_index(random: &mut impl GenerationRandom, bound: usize) -> usize {
    use std::num::NonZeroU32;

    random.next_u32(NonZeroU32::new(bound as u32).expect("mansion selector is nonempty")) as usize
}
