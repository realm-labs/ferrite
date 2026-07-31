//! Ocean-monument room lattice, connectivity pruning, fitting, and child boxes.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::{HorizontalDirection, OrientedPiece};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonumentPieceKind {
    Building,
    Entry,
    Core,
    DoubleX,
    DoubleXY,
    DoubleY,
    DoubleYZ,
    DoubleZ,
    Simple,
    SimpleTop,
    Wing,
    Penthouse,
}

impl MonumentPieceKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Building => "minecraft:omb",
            Self::Entry => "minecraft:omentry",
            Self::Core => "minecraft:omcr",
            Self::DoubleX => "minecraft:omdxr",
            Self::DoubleXY => "minecraft:omdxyr",
            Self::DoubleY => "minecraft:omdyr",
            Self::DoubleYZ => "minecraft:omdyzr",
            Self::DoubleZ => "minecraft:omdzr",
            Self::Simple => "minecraft:omsimple",
            Self::SimpleTop => "minecraft:omsimplet",
            Self::Wing => "minecraft:omwr",
            Self::Penthouse => "minecraft:ompenthouse",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonumentRoom {
    pub index: i32,
    pub connections: [Option<usize>; 6],
    pub openings: [bool; 6],
    pub claimed: bool,
    pub is_source: bool,
    scan_index: i32,
}

impl MonumentRoom {
    pub fn opening(&self, direction: MonumentDirection) -> bool {
        self.openings[direction.index()]
    }

    pub fn opening_count(&self) -> usize {
        self.openings.iter().filter(|opening| **opening).count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonumentChild {
    pub kind: MonumentPieceKind,
    pub bounding_box: BlockBox,
    pub orientation: HorizontalDirection,
    pub room: Option<usize>,
    pub design: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonumentGraph {
    pub stub_position: BlockPos,
    pub building: MonumentChild,
    pub rooms: Vec<MonumentRoom>,
    pub children: Vec<MonumentChild>,
    pub source_room: usize,
    pub core_room: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonumentDirection {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl MonumentDirection {
    pub const ALL: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    const fn step(self) -> [i32; 3] {
        match self {
            Self::Down => [0, -1, 0],
            Self::Up => [0, 1, 0],
            Self::North => [0, 0, -1],
            Self::South => [0, 0, 1],
            Self::West => [-1, 0, 0],
            Self::East => [1, 0, 0],
        }
    }
}

pub fn generate_monument(
    chunk_x: i32,
    chunk_z: i32,
    ocean_floor_height: i32,
    random: &mut impl GenerationRandom,
) -> MonumentGraph {
    let orientation = HorizontalDirection::ALL[bounded(random, 4) as usize];
    let west = chunk_x.wrapping_mul(16).wrapping_sub(29);
    let north = chunk_z.wrapping_mul(16).wrapping_sub(29);
    let building_box = BlockBox::new(
        BlockPos::new(west, 39, north),
        BlockPos::new(west + 57, 61, north + 57),
    )
    .expect("monument dimensions are positive");
    let building = MonumentChild {
        kind: MonumentPieceKind::Building,
        bounding_box: building_box,
        orientation,
        room: None,
        design: 0,
    };
    let (mut rooms, ordinary, source_room, core_room) = room_graph(random);
    rooms[source_room].claimed = true;
    let building_piece = OrientedPiece {
        bounds: building_box,
        orientation,
    };
    let offset = building_piece.world_position(BlockPos::new(9, 0, 22));
    let mut children = vec![room_child(
        MonumentPieceKind::Entry,
        source_room,
        [1, 1, 1],
        orientation,
        offset,
        &rooms,
        0,
    )];
    children.push(room_child(
        MonumentPieceKind::Core,
        core_room,
        [2, 2, 2],
        orientation,
        offset,
        &rooms,
        0,
    ));
    for room in ordinary {
        if rooms[room].claimed || rooms[room].index >= 75 {
            continue;
        }
        let (kind, size, claimed, design) = fit_room(room, &rooms, random);
        for slot in claimed {
            rooms[slot].claimed = true;
        }
        children.push(room_child(
            kind,
            room,
            size,
            orientation,
            offset,
            &rooms,
            design,
        ));
    }
    let wing_seed = random.next_i32();
    children.push(special_child(
        MonumentPieceKind::Wing,
        building_piece,
        BlockPos::new(1, 1, 1),
        BlockPos::new(23, 8, 21),
        orientation,
        wing_seed,
    ));
    children.push(special_child(
        MonumentPieceKind::Wing,
        building_piece,
        BlockPos::new(34, 1, 1),
        BlockPos::new(56, 8, 21),
        orientation,
        wing_seed.wrapping_add(1),
    ));
    children.push(special_child(
        MonumentPieceKind::Penthouse,
        building_piece,
        BlockPos::new(22, 13, 22),
        BlockPos::new(35, 17, 35),
        orientation,
        0,
    ));
    MonumentGraph {
        stub_position: BlockPos::new(
            chunk_x.wrapping_mul(16).wrapping_add(8),
            ocean_floor_height,
            chunk_z.wrapping_mul(16).wrapping_add(8),
        ),
        building,
        rooms,
        children,
        source_room,
        core_room,
    }
}

fn room_graph(random: &mut impl GenerationRandom) -> (Vec<MonumentRoom>, Vec<usize>, usize, usize) {
    let mut rooms = Vec::with_capacity(49);
    let mut grid = [None; 75];
    for y in 0..=1 {
        for x in 0..5 {
            for z in 0..4 {
                add_room(&mut rooms, &mut grid, room_index(x, y, z));
            }
        }
    }
    for x in 1..4 {
        for z in 0..2 {
            add_room(&mut rooms, &mut grid, room_index(x, 2, z));
        }
    }
    for x in 0..5 {
        for z in 0..5 {
            for y in 0..3 {
                let Some(current) = grid[room_index(x, y, z) as usize] else {
                    continue;
                };
                for direction in MonumentDirection::ALL {
                    let step = direction.step();
                    let [nx, ny, nz] = [x + step[0], y + step[1], z + step[2]];
                    if !(0..5).contains(&nx) || !(0..3).contains(&ny) || !(0..5).contains(&nz) {
                        continue;
                    }
                    let Some(neighbor) = grid[room_index(nx, ny, nz) as usize] else {
                        continue;
                    };
                    let logical = if nz == z {
                        direction
                    } else {
                        direction.opposite()
                    };
                    connect(&mut rooms, current, logical, neighbor);
                }
            }
        }
    }
    let roof = add_special(&mut rooms, 1003);
    let left = add_special(&mut rooms, 1001);
    let right = add_special(&mut rooms, 1002);
    let top = grid[room_index(2, 2, 0) as usize].expect("top connector exists");
    let left_connector = grid[room_index(0, 1, 0) as usize].expect("left connector exists");
    let right_connector = grid[room_index(4, 1, 0) as usize].expect("right connector exists");
    connect(&mut rooms, top, MonumentDirection::Up, roof);
    connect(&mut rooms, left_connector, MonumentDirection::South, left);
    connect(&mut rooms, right_connector, MonumentDirection::South, right);
    rooms[roof].claimed = true;
    rooms[left].claimed = true;
    rooms[right].claimed = true;
    let source = grid[room_index(2, 0, 0) as usize].expect("source exists");
    rooms[source].is_source = true;
    let core =
        grid[room_index(bounded(random, 4) as i32, 0, 2) as usize].expect("core base exists");
    let east = connection(&rooms, core, MonumentDirection::East);
    let north = connection(&rooms, core, MonumentDirection::North);
    let up = connection(&rooms, core, MonumentDirection::Up);
    let east_north = connection(&rooms, east, MonumentDirection::North);
    let east_up = connection(&rooms, east, MonumentDirection::Up);
    let north_up = connection(&rooms, north, MonumentDirection::Up);
    let east_north_up = connection(&rooms, east_north, MonumentDirection::Up);
    for slot in [
        core,
        east,
        north,
        east_north,
        up,
        east_up,
        north_up,
        east_north_up,
    ] {
        rooms[slot].claimed = true;
    }
    for room in &mut rooms {
        for direction in MonumentDirection::ALL {
            room.openings[direction.index()] = room.connections[direction.index()].is_some();
        }
    }
    let mut ordinary = (0..46).collect::<Vec<_>>();
    ordinary.sort_by_key(|room| rooms[*room].index);
    for remaining in (2..=ordinary.len()).rev() {
        let selected = bounded(random, remaining as u32) as usize;
        ordinary.swap(remaining - 1, selected);
    }
    let mut scan = 1;
    for &room in &ordinary {
        let mut closed = 0;
        for _ in 0..5 {
            if closed >= 2 {
                break;
            }
            let direction = bounded(random, 6) as usize;
            if !rooms[room].openings[direction] {
                continue;
            }
            let other = rooms[room].connections[direction].expect("open edge is connected");
            let opposite = MonumentDirection::ALL[direction].opposite().index();
            rooms[room].openings[direction] = false;
            rooms[other].openings[opposite] = false;
            let first_scan = scan;
            scan += 1;
            let first = find_source(&mut rooms, room, first_scan);
            let second = if first {
                let second_scan = scan;
                scan += 1;
                find_source(&mut rooms, other, second_scan)
            } else {
                false
            };
            if first && second {
                closed += 1;
            } else {
                rooms[room].openings[direction] = true;
                rooms[other].openings[opposite] = true;
            }
        }
    }
    ordinary.extend([roof, left, right]);
    (rooms, ordinary, source, core)
}

fn fit_room(
    room: usize,
    rooms: &[MonumentRoom],
    random: &mut impl GenerationRandom,
) -> (MonumentPieceKind, [i32; 3], Vec<usize>, i32) {
    if let Some(east) = open_unclaimed(rooms, room, MonumentDirection::East)
        && let Some(up) = open_unclaimed(rooms, room, MonumentDirection::Up)
        && let Some(east_up) = open_unclaimed(rooms, east, MonumentDirection::Up)
    {
        return (
            MonumentPieceKind::DoubleXY,
            [2, 2, 1],
            vec![room, east, up, east_up],
            0,
        );
    }
    if let Some(north) = open_unclaimed(rooms, room, MonumentDirection::North)
        && let Some(up) = open_unclaimed(rooms, room, MonumentDirection::Up)
        && let Some(north_up) = open_unclaimed(rooms, north, MonumentDirection::Up)
    {
        return (
            MonumentPieceKind::DoubleYZ,
            [1, 2, 2],
            vec![room, north, up, north_up],
            0,
        );
    }
    if let Some(north) = open_unclaimed(rooms, room, MonumentDirection::North) {
        return (MonumentPieceKind::DoubleZ, [1, 1, 2], vec![room, north], 0);
    }
    if let Some(east) = open_unclaimed(rooms, room, MonumentDirection::East) {
        return (MonumentPieceKind::DoubleX, [2, 1, 1], vec![room, east], 0);
    }
    if let Some(up) = open_unclaimed(rooms, room, MonumentDirection::Up) {
        return (MonumentPieceKind::DoubleY, [1, 2, 1], vec![room, up], 0);
    }
    if [
        MonumentDirection::West,
        MonumentDirection::East,
        MonumentDirection::North,
        MonumentDirection::South,
        MonumentDirection::Up,
    ]
    .into_iter()
    .all(|direction| !rooms[room].opening(direction))
    {
        return (MonumentPieceKind::SimpleTop, [1, 1, 1], vec![room], 0);
    }
    (
        MonumentPieceKind::Simple,
        [1, 1, 1],
        vec![room],
        bounded(random, 3) as i32,
    )
}

fn open_unclaimed(
    rooms: &[MonumentRoom],
    room: usize,
    direction: MonumentDirection,
) -> Option<usize> {
    rooms[room]
        .opening(direction)
        .then(|| rooms[room].connections[direction.index()])
        .flatten()
        .filter(|connected| !rooms[*connected].claimed)
}

fn room_child(
    kind: MonumentPieceKind,
    room: usize,
    size: [i32; 3],
    orientation: HorizontalDirection,
    offset: BlockPos,
    rooms: &[MonumentRoom],
    design: i32,
) -> MonumentChild {
    MonumentChild {
        kind,
        bounding_box: room_box(rooms[room].index, size, orientation)
            .moved([offset.x, offset.y, offset.z]),
        orientation,
        room: Some(room),
        design,
    }
}

fn room_box(index: i32, size: [i32; 3], orientation: HorizontalDirection) -> BlockBox {
    let [room_width, room_height, room_depth] = size;
    let x = index % 5;
    let z = index / 5 % 5;
    let y = index / 25;
    let (width, depth) = match orientation {
        HorizontalDirection::North | HorizontalDirection::South => (room_width * 8, room_depth * 8),
        HorizontalDirection::West | HorizontalDirection::East => (room_depth * 8, room_width * 8),
    };
    let (minimum_x, minimum_z) = match orientation {
        HorizontalDirection::North => (x * 8, -(z + room_depth) * 8 + 1),
        HorizontalDirection::South => (x * 8, z * 8),
        HorizontalDirection::West => (-(z + room_depth) * 8 + 1, x * 8),
        HorizontalDirection::East => (z * 8, x * 8),
    };
    BlockBox::new(
        BlockPos::new(minimum_x, y * 4, minimum_z),
        BlockPos::new(
            minimum_x + width - 1,
            y * 4 + room_height * 4 - 1,
            minimum_z + depth - 1,
        ),
    )
    .expect("room dimensions are positive")
}

fn special_child(
    kind: MonumentPieceKind,
    building: OrientedPiece,
    left: BlockPos,
    right: BlockPos,
    orientation: HorizontalDirection,
    design: i32,
) -> MonumentChild {
    let left = building.world_position(left);
    let right = building.world_position(right);
    MonumentChild {
        kind,
        bounding_box: encompassing(left, right),
        orientation,
        room: None,
        design,
    }
}

fn encompassing(left: BlockPos, right: BlockPos) -> BlockBox {
    BlockBox::new(
        BlockPos::new(
            left.x.min(right.x),
            left.y.min(right.y),
            left.z.min(right.z),
        ),
        BlockPos::new(
            left.x.max(right.x),
            left.y.max(right.y),
            left.z.max(right.z),
        ),
    )
    .expect("ordered transformed corners")
}

fn add_room(rooms: &mut Vec<MonumentRoom>, grid: &mut [Option<usize>; 75], index: i32) {
    let slot = add_special(rooms, index);
    grid[index as usize] = Some(slot);
}

fn add_special(rooms: &mut Vec<MonumentRoom>, index: i32) -> usize {
    let slot = rooms.len();
    rooms.push(MonumentRoom {
        index,
        connections: [None; 6],
        openings: [false; 6],
        claimed: false,
        is_source: false,
        scan_index: 0,
    });
    slot
}

fn connect(rooms: &mut [MonumentRoom], left: usize, direction: MonumentDirection, right: usize) {
    rooms[left].connections[direction.index()] = Some(right);
    rooms[right].connections[direction.opposite().index()] = Some(left);
}

fn connection(rooms: &[MonumentRoom], room: usize, direction: MonumentDirection) -> usize {
    rooms[room].connections[direction.index()].expect("canonical lattice connection exists")
}

fn find_source(rooms: &mut [MonumentRoom], room: usize, scan: i32) -> bool {
    if rooms[room].is_source {
        return true;
    }
    rooms[room].scan_index = scan;
    for direction in 0..6 {
        let Some(next) = rooms[room].connections[direction] else {
            continue;
        };
        if !rooms[room].openings[direction] || rooms[next].scan_index == scan {
            continue;
        }
        if find_source(rooms, next, scan) {
            return true;
        }
    }
    false
}

const fn room_index(x: i32, y: i32, z: i32) -> i32 {
    y * 25 + z * 5 + x
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("positive monument bound"))
}
