//! Source-ordered woodland-mansion floor graph generation.

use std::num::NonZeroU32;

use ferrite_foundation::direction::Direction;

use crate::generation::feature::random::GenerationRandom;

pub const CLEAR: i32 = 0;
pub const CORRIDOR: i32 = 1;
pub const ROOM: i32 = 2;
pub const START_ROOM: i32 = 3;
pub const TEST_ROOM: i32 = 4;
pub const BLOCKED: i32 = 5;

pub const ROOM_1X1: i32 = 0x1_0000;
pub const ROOM_1X2: i32 = 0x2_0000;
pub const ROOM_2X2: i32 = 0x4_0000;
pub const ROOM_ORIGIN: i32 = 0x10_0000;
pub const ROOM_DOOR: i32 = 0x20_0000;
pub const ROOM_STAIRS: i32 = 0x40_0000;
pub const ROOM_CORRIDOR: i32 = 0x80_0000;
pub const ROOM_TYPE_MASK: i32 = 0x0f_0000;
pub const ROOM_ID_MASK: i32 = 0x00_ffff;

const SIZE: usize = 11;
const HORIZONTAL: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];
const DATA_2D: [Direction; 4] = [
    Direction::South,
    Direction::West,
    Direction::North,
    Direction::East,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleGrid {
    width: usize,
    height: usize,
    outside: i32,
    cells: Vec<i32>,
}

impl SimpleGrid {
    pub fn new(width: usize, height: usize, outside: i32) -> Self {
        Self {
            width,
            height,
            outside,
            cells: vec![0; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: i32, y: i32) -> i32 {
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            return self.outside;
        };
        if x >= self.width || y >= self.height {
            self.outside
        } else {
            self.cells[y * self.width + x]
        }
    }

    pub fn cells(&self) -> &[i32] {
        &self.cells
    }

    fn set(&mut self, x: i32, y: i32, value: i32) {
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            return;
        };
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = value;
        }
    }

    fn fill(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, value: i32) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.set(x, y, value);
            }
        }
    }

    fn set_if(&mut self, x: i32, y: i32, expected: i32, value: i32) {
        if self.get(x, y) == expected {
            self.set(x, y, value);
        }
    }

    fn edges_to(&self, x: i32, y: i32, value: i32) -> bool {
        self.get(x - 1, y) == value
            || self.get(x + 1, y) == value
            || self.get(x, y + 1) == value
            || self.get(x, y - 1) == value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MansionLayout {
    pub base: SimpleGrid,
    pub third: SimpleGrid,
    pub floor_rooms: [SimpleGrid; 3],
    pub entrance_x: i32,
    pub entrance_y: i32,
}

impl MansionLayout {
    pub fn generate(random: &mut impl GenerationRandom) -> Self {
        let mut base = SimpleGrid::new(SIZE, SIZE, BLOCKED);
        let entrance_x = 7;
        let entrance_y = 4;
        base.fill(
            entrance_x,
            entrance_y,
            entrance_x + 1,
            entrance_y + 1,
            START_ROOM,
        );
        base.fill(
            entrance_x - 1,
            entrance_y,
            entrance_x - 1,
            entrance_y + 1,
            ROOM,
        );
        base.fill(
            entrance_x + 2,
            entrance_y - 2,
            entrance_x + 3,
            entrance_y + 3,
            BLOCKED,
        );
        base.fill(
            entrance_x + 1,
            entrance_y - 2,
            entrance_x + 1,
            entrance_y - 1,
            CORRIDOR,
        );
        base.fill(
            entrance_x + 1,
            entrance_y + 2,
            entrance_x + 1,
            entrance_y + 3,
            CORRIDOR,
        );
        base.set(entrance_x - 1, entrance_y - 1, CORRIDOR);
        base.set(entrance_x - 1, entrance_y + 2, CORRIDOR);
        base.fill(0, 0, 11, 1, BLOCKED);
        base.fill(0, 9, 11, 11, BLOCKED);

        recursive_corridor(
            &mut base,
            entrance_x,
            entrance_y - 2,
            Direction::West,
            6,
            random,
        );
        recursive_corridor(
            &mut base,
            entrance_x,
            entrance_y + 3,
            Direction::West,
            6,
            random,
        );
        recursive_corridor(
            &mut base,
            entrance_x - 2,
            entrance_y - 1,
            Direction::West,
            3,
            random,
        );
        recursive_corridor(
            &mut base,
            entrance_x - 2,
            entrance_y + 2,
            Direction::West,
            3,
            random,
        );
        while clean_edges(&mut base) {}

        let mut floor_rooms = [
            SimpleGrid::new(SIZE, SIZE, BLOCKED),
            SimpleGrid::new(SIZE, SIZE, BLOCKED),
            SimpleGrid::new(SIZE, SIZE, BLOCKED),
        ];
        identify_rooms(&base, &mut floor_rooms[0], random);
        identify_rooms(&base, &mut floor_rooms[1], random);
        floor_rooms[0].fill(
            entrance_x + 1,
            entrance_y,
            entrance_x + 1,
            entrance_y + 1,
            ROOM_CORRIDOR,
        );
        floor_rooms[1].fill(
            entrance_x + 1,
            entrance_y,
            entrance_x + 1,
            entrance_y + 1,
            ROOM_CORRIDOR,
        );

        let mut third = SimpleGrid::new(SIZE, SIZE, BLOCKED);
        setup_third_floor(&base, &mut third, &mut floor_rooms, random);
        identify_rooms(&third, &mut floor_rooms[2], random);
        Self {
            base,
            third,
            floor_rooms,
            entrance_x,
            entrance_y,
        }
    }

    pub fn is_house(grid: &SimpleGrid, x: i32, y: i32) -> bool {
        matches!(grid.get(x, y), CORRIDOR | ROOM | START_ROOM | TEST_ROOM)
    }

    pub fn is_room_id(&self, x: i32, y: i32, floor: usize, room_id: i32) -> bool {
        self.floor_rooms[floor].get(x, y) & ROOM_ID_MASK == room_id
    }

    pub fn room_direction(&self, x: i32, y: i32, floor: usize, room_id: i32) -> Option<Direction> {
        HORIZONTAL.into_iter().find(|direction| {
            let [dx, _, dz] = direction.step();
            self.is_room_id(x + dx, y + dz, floor, room_id)
        })
    }
}

fn recursive_corridor(
    grid: &mut SimpleGrid,
    x: i32,
    y: i32,
    heading: Direction,
    depth: i32,
    random: &mut impl GenerationRandom,
) {
    if depth <= 0 {
        return;
    }
    let [hx, _, hz] = heading.step();
    grid.set(x, y, CORRIDOR);
    grid.set_if(x + hx, y + hz, CLEAR, CORRIDOR);
    for _ in 0..8 {
        let next = DATA_2D[random_index(random, 4)];
        if next == heading.opposite() || next == Direction::East && random.next_bool() {
            continue;
        }
        let [nx, _, nz] = next.step();
        let ahead_x = x + hx;
        let ahead_y = y + hz;
        if grid.get(ahead_x + nx, ahead_y + nz) != CLEAR
            || grid.get(ahead_x + 2 * nx, ahead_y + 2 * nz) != CLEAR
        {
            continue;
        }
        recursive_corridor(grid, ahead_x + nx, ahead_y + nz, next, depth - 1, random);
        break;
    }
    let clockwise = clockwise(heading);
    let counterclockwise = counterclockwise(heading);
    let [cx, _, cz] = clockwise.step();
    let [ccx, _, ccz] = counterclockwise.step();
    for (room_x, room_y) in [
        (x + cx, y + cz),
        (x + ccx, y + ccz),
        (x + hx + cx, y + hz + cz),
        (x + hx + ccx, y + hz + ccz),
        (x + 2 * hx, y + 2 * hz),
        (x + 2 * cx, y + 2 * cz),
        (x + 2 * ccx, y + 2 * ccz),
    ] {
        grid.set_if(room_x, room_y, CLEAR, ROOM);
    }
}

fn clean_edges(grid: &mut SimpleGrid) -> bool {
    let mut touched = false;
    for y in 0..grid.height as i32 {
        for x in 0..grid.width as i32 {
            if grid.get(x, y) != CLEAR {
                continue;
            }
            let direct = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .into_iter()
                .filter(|(dx, dy)| MansionLayout::is_house(grid, x + dx, y + dy))
                .count();
            if direct >= 3 {
                grid.set(x, y, ROOM);
                touched = true;
                continue;
            }
            if direct == 2 {
                let diagonal = [(1, 1), (-1, 1), (1, -1), (-1, -1)]
                    .into_iter()
                    .filter(|(dx, dy)| MansionLayout::is_house(grid, x + dx, y + dy))
                    .count();
                if diagonal <= 1 {
                    grid.set(x, y, ROOM);
                    touched = true;
                }
            }
        }
    }
    touched
}

fn setup_third_floor(
    base: &SimpleGrid,
    third: &mut SimpleGrid,
    floor_rooms: &mut [SimpleGrid; 3],
    random: &mut impl GenerationRandom,
) {
    let mut candidates = Vec::new();
    for y in 0..SIZE as i32 {
        for x in 0..SIZE as i32 {
            let data = floor_rooms[1].get(x, y);
            if data & ROOM_TYPE_MASK == ROOM_1X2 && data & ROOM_DOOR == ROOM_DOOR {
                candidates.push((x, y));
            }
        }
    }
    if candidates.is_empty() {
        third.fill(0, 0, SIZE as i32, SIZE as i32, BLOCKED);
        return;
    }
    let (room_x, room_y) = candidates[random_index(random, candidates.len())];
    let room_data = floor_rooms[1].get(room_x, room_y);
    floor_rooms[1].set(room_x, room_y, room_data | ROOM_STAIRS);
    let room_id = room_data & ROOM_ID_MASK;
    let room_direction = HORIZONTAL
        .into_iter()
        .find(|direction| {
            let [dx, _, dz] = direction.step();
            floor_rooms[1].get(room_x + dx, room_y + dz) & ROOM_ID_MASK == room_id
        })
        .expect("a 1x2 room has a second cell");
    let [room_dx, _, room_dz] = room_direction.step();
    let end_x = room_x + room_dx;
    let end_y = room_y + room_dz;
    for y in 0..SIZE as i32 {
        for x in 0..SIZE as i32 {
            if !MansionLayout::is_house(base, x, y) {
                third.set(x, y, BLOCKED);
            } else if (x, y) == (room_x, room_y) {
                third.set(x, y, START_ROOM);
            } else if (x, y) == (end_x, end_y) {
                third.set(x, y, START_ROOM);
                floor_rooms[2].set(x, y, ROOM_CORRIDOR);
            }
        }
    }
    let corridors = HORIZONTAL
        .into_iter()
        .filter(|direction| {
            let [dx, _, dz] = direction.step();
            third.get(end_x + dx, end_y + dz) == CLEAR
        })
        .collect::<Vec<_>>();
    if corridors.is_empty() {
        third.fill(0, 0, SIZE as i32, SIZE as i32, BLOCKED);
        floor_rooms[1].set(room_x, room_y, room_data);
        return;
    }
    let direction = corridors[random_index(random, corridors.len())];
    let [dx, _, dz] = direction.step();
    recursive_corridor(third, end_x + dx, end_y + dz, direction, 4, random);
    while clean_edges(third) {}
}

fn identify_rooms(source: &SimpleGrid, rooms: &mut SimpleGrid, random: &mut impl GenerationRandom) {
    let mut cells = Vec::new();
    for y in 0..source.height as i32 {
        for x in 0..source.width as i32 {
            if source.get(x, y) == ROOM {
                cells.push((x, y));
            }
        }
    }
    shuffle(&mut cells, random);
    let mut room_id = 10;
    for (x, y) in cells {
        if rooms.get(x, y) != CLEAR {
            continue;
        }
        let (mut x0, mut x1, mut y0, mut y1) = (x, x, y, y);
        let room_type;
        if available_2x2(source, rooms, x, y, 1, 1) {
            x1 += 1;
            y1 += 1;
            room_type = ROOM_2X2;
        } else if available_2x2(source, rooms, x, y, -1, 1) {
            x0 -= 1;
            y1 += 1;
            room_type = ROOM_2X2;
        } else if available_2x2(source, rooms, x, y, -1, -1) {
            x0 -= 1;
            y0 -= 1;
            room_type = ROOM_2X2;
        } else if available(source, rooms, x + 1, y) {
            x1 += 1;
            room_type = ROOM_1X2;
        } else if available(source, rooms, x, y + 1) {
            y1 += 1;
            room_type = ROOM_1X2;
        } else if available(source, rooms, x - 1, y) {
            x0 -= 1;
            room_type = ROOM_1X2;
        } else if available(source, rooms, x, y - 1) {
            y0 -= 1;
            room_type = ROOM_1X2;
        } else {
            room_type = ROOM_1X1;
        }
        let mut door_x = if random.next_bool() { x0 } else { x1 };
        let mut door_y = if random.next_bool() { y0 } else { y1 };
        let mut door_flag = ROOM_DOOR;
        if !source.edges_to(door_x, door_y, CORRIDOR) {
            door_x = toggle(door_x, x0, x1);
            door_y = toggle(door_y, y0, y1);
            if !source.edges_to(door_x, door_y, CORRIDOR) {
                door_y = toggle(door_y, y0, y1);
                if !source.edges_to(door_x, door_y, CORRIDOR) {
                    door_x = toggle(door_x, x0, x1);
                    door_y = toggle(door_y, y0, y1);
                    if !source.edges_to(door_x, door_y, CORRIDOR) {
                        door_flag = 0;
                        door_x = x0;
                        door_y = y0;
                    }
                }
            }
        }
        for room_y in y0..=y1 {
            for room_x in x0..=x1 {
                let flags = if (room_x, room_y) == (door_x, door_y) {
                    ROOM_ORIGIN | door_flag
                } else {
                    0
                };
                rooms.set(room_x, room_y, flags | room_type | room_id);
            }
        }
        room_id += 1;
    }
}

fn available(source: &SimpleGrid, rooms: &SimpleGrid, x: i32, y: i32) -> bool {
    rooms.get(x, y) == CLEAR && source.get(x, y) == ROOM
}

fn available_2x2(
    source: &SimpleGrid,
    rooms: &SimpleGrid,
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
) -> bool {
    available(source, rooms, x + dx, y)
        && available(source, rooms, x, y + dy)
        && available(source, rooms, x + dx, y + dy)
}

fn toggle(value: i32, low: i32, high: i32) -> i32 {
    if value == low { high } else { low }
}

fn shuffle<T>(values: &mut [T], random: &mut impl GenerationRandom) {
    for remaining in (2..=values.len()).rev() {
        values.swap(remaining - 1, random_index(random, remaining));
    }
}

fn random_index(random: &mut impl GenerationRandom, bound: usize) -> usize {
    let bound = u32::try_from(bound).expect("mansion collection bound fits u32");
    random.next_u32(NonZeroU32::new(bound).expect("mansion collection is nonempty")) as usize
}

pub(crate) fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        _ => direction,
    }
}

pub(crate) fn counterclockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::West,
        Direction::West => Direction::South,
        Direction::South => Direction::East,
        Direction::East => Direction::North,
        _ => direction,
    }
}

#[cfg(test)]
mod tests {
    use crate::generation::feature::random::LegacyRandom;

    use super::*;

    #[test]
    fn layout_preserves_fixed_start_and_partition_invariants() {
        let layout = MansionLayout::generate(&mut LegacyRandom::new(1));
        assert_eq!(layout.base.get(7, 4), START_ROOM);
        assert_eq!(layout.base.get(8, 5), START_ROOM);
        assert_eq!(layout.floor_rooms[0].get(8, 4), ROOM_CORRIDOR);
        assert_eq!(layout.floor_rooms[1].get(8, 5), ROOM_CORRIDOR);
        assert_eq!(layout.base.get(-1, 4), BLOCKED);
        assert!(layout.base.cells().contains(&CORRIDOR));
    }
}
