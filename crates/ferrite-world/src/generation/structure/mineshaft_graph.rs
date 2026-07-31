//! Depth-first mineshaft graph construction and final vertical relocation.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::HorizontalDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineshaftType {
    Normal,
    Mesa,
}

impl MineshaftType {
    pub const fn log(self) -> &'static str {
        match self {
            Self::Normal => "minecraft:oak_log",
            Self::Mesa => "minecraft:dark_oak_log",
        }
    }

    pub const fn planks(self) -> &'static str {
        match self {
            Self::Normal => "minecraft:oak_planks",
            Self::Mesa => "minecraft:dark_oak_planks",
        }
    }

    pub const fn fence(self) -> &'static str {
        match self {
            Self::Normal => "minecraft:oak_fence",
            Self::Mesa => "minecraft:dark_oak_fence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MineshaftRoom {
    pub bounding_box: BlockBox,
    pub generation_depth: i32,
    pub entrances: Vec<BlockBox>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MineshaftCorridor {
    pub bounding_box: BlockBox,
    pub generation_depth: i32,
    pub orientation: HorizontalDirection,
    pub has_rails: bool,
    pub spider_corridor: bool,
    pub has_placed_spider: bool,
    pub sections: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MineshaftCrossing {
    pub bounding_box: BlockBox,
    pub generation_depth: i32,
    pub direction: HorizontalDirection,
    pub two_floored: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MineshaftStairs {
    pub bounding_box: BlockBox,
    pub generation_depth: i32,
    pub orientation: HorizontalDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MineshaftPiece {
    Room(MineshaftRoom),
    Corridor(MineshaftCorridor),
    Crossing(MineshaftCrossing),
    Stairs(MineshaftStairs),
}

impl MineshaftPiece {
    pub fn bounding_box(&self) -> BlockBox {
        match self {
            Self::Room(piece) => piece.bounding_box,
            Self::Corridor(piece) => piece.bounding_box,
            Self::Crossing(piece) => piece.bounding_box,
            Self::Stairs(piece) => piece.bounding_box,
        }
    }

    pub fn generation_depth(&self) -> i32 {
        match self {
            Self::Room(piece) => piece.generation_depth,
            Self::Corridor(piece) => piece.generation_depth,
            Self::Crossing(piece) => piece.generation_depth,
            Self::Stairs(piece) => piece.generation_depth,
        }
    }

    fn move_vertical(&mut self, offset: i32) {
        match self {
            Self::Room(piece) => {
                piece.bounding_box = piece.bounding_box.moved([0, offset, 0]);
                for entrance in &mut piece.entrances {
                    *entrance = entrance.moved([0, offset, 0]);
                }
            }
            Self::Corridor(piece) => {
                piece.bounding_box = piece.bounding_box.moved([0, offset, 0]);
            }
            Self::Crossing(piece) => {
                piece.bounding_box = piece.bounding_box.moved([0, offset, 0]);
            }
            Self::Stairs(piece) => {
                piece.bounding_box = piece.bounding_box.moved([0, offset, 0]);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MineshaftGraph {
    pub kind: MineshaftType,
    pub pieces: Vec<MineshaftPiece>,
    pub stub_position: BlockPos,
    pub vertical_offset: i32,
}

pub fn generate_mineshaft(
    chunk_x: i32,
    chunk_z: i32,
    kind: MineshaftType,
    sea_level: i32,
    minimum_y: i32,
    world_surface: &mut impl FnMut(i32, i32) -> i32,
    random: &mut impl GenerationRandom,
) -> MineshaftGraph {
    random.next_f64();
    let start_x = chunk_x.wrapping_mul(16).wrapping_add(2);
    let start_z = chunk_z.wrapping_mul(16).wrapping_add(2);
    let room = MineshaftRoom {
        bounding_box: BlockBox::new(
            BlockPos::new(start_x, 50, start_z),
            BlockPos::new(
                start_x + 7 + bounded(random, 6) as i32,
                54 + bounded(random, 6) as i32,
                start_z + 7 + bounded(random, 6) as i32,
            ),
        )
        .expect("room dimensions are positive"),
        generation_depth: 0,
        entrances: Vec::new(),
    };
    let root_box = room.bounding_box;
    let mut pieces = vec![MineshaftPiece::Room(room)];
    expand_room(0, root_box, kind, &mut pieces, random);
    let union = union_boxes(&pieces);
    let offset = match kind {
        MineshaftType::Mesa => {
            let center = union.center();
            let surface = world_surface(center.x, center.z);
            let target = if surface <= sea_level {
                sea_level
            } else {
                sea_level + bounded(random, (surface - sea_level + 1) as u32) as i32
            };
            target.wrapping_sub(center.y)
        }
        MineshaftType::Normal => below_sea_offset(union, sea_level, minimum_y, random),
    };
    for piece in &mut pieces {
        piece.move_vertical(offset);
    }
    MineshaftGraph {
        kind,
        pieces,
        stub_position: BlockPos::new(
            chunk_x.wrapping_mul(16).wrapping_add(8),
            50_i32.wrapping_add(offset),
            chunk_z.wrapping_mul(16),
        ),
        vertical_offset: offset,
    }
}

fn expand_room(
    room_index: usize,
    root_box: BlockBox,
    kind: MineshaftType,
    pieces: &mut Vec<MineshaftPiece>,
    random: &mut impl GenerationRandom,
) {
    let room_box = pieces[room_index].bounding_box();
    let height_space = (room_box.size()[1] - 4).max(1);
    for direction in HorizontalDirection::ALL {
        let span = match direction {
            HorizontalDirection::North | HorizontalDirection::South => room_box.size()[0],
            HorizontalDirection::West | HorizontalDirection::East => room_box.size()[2],
        };
        let mut position = 0;
        while position < span {
            position += bounded(random, span as u32) as i32;
            if position + 3 > span {
                break;
            }
            let y = room_box.minimum.y + 1 + bounded(random, height_space as u32) as i32;
            let (x, z) = match direction {
                HorizontalDirection::North => {
                    (room_box.minimum.x + position, room_box.minimum.z - 1)
                }
                HorizontalDirection::South => {
                    (room_box.minimum.x + position, room_box.maximum.z + 1)
                }
                HorizontalDirection::West => {
                    (room_box.minimum.x - 1, room_box.minimum.z + position)
                }
                HorizontalDirection::East => {
                    (room_box.maximum.x + 1, room_box.minimum.z + position)
                }
            };
            if let Some(child_index) =
                add_child(root_box, kind, pieces, random, (x, y, z), direction, 0)
            {
                let child = pieces[child_index].bounding_box();
                let entrance = match direction {
                    HorizontalDirection::North => BlockBox::new(
                        BlockPos::new(child.minimum.x, child.minimum.y, room_box.minimum.z),
                        BlockPos::new(child.maximum.x, child.maximum.y, room_box.minimum.z + 1),
                    ),
                    HorizontalDirection::South => BlockBox::new(
                        BlockPos::new(child.minimum.x, child.minimum.y, room_box.maximum.z - 1),
                        BlockPos::new(child.maximum.x, child.maximum.y, room_box.maximum.z),
                    ),
                    HorizontalDirection::West => BlockBox::new(
                        BlockPos::new(room_box.minimum.x, child.minimum.y, child.minimum.z),
                        BlockPos::new(room_box.minimum.x + 1, child.maximum.y, child.maximum.z),
                    ),
                    HorizontalDirection::East => BlockBox::new(
                        BlockPos::new(room_box.maximum.x - 1, child.minimum.y, child.minimum.z),
                        BlockPos::new(room_box.maximum.x, child.maximum.y, child.maximum.z),
                    ),
                }
                .expect("entrance dimensions are ordered");
                let MineshaftPiece::Room(room) = &mut pieces[room_index] else {
                    unreachable!("root room index remains a room");
                };
                room.entrances.push(entrance);
            }
            position += 4;
        }
    }
}

fn add_child(
    root_box: BlockBox,
    kind: MineshaftType,
    pieces: &mut Vec<MineshaftPiece>,
    random: &mut impl GenerationRandom,
    anchor: (i32, i32, i32),
    direction: HorizontalDirection,
    input_depth: i32,
) -> Option<usize> {
    let (x, y, z) = anchor;
    if input_depth > 8 || x.abs_diff(root_box.minimum.x) > 80 || z.abs_diff(root_box.minimum.z) > 80
    {
        return None;
    }
    let depth = input_depth + 1;
    let selection = bounded(random, 100);
    let piece = if selection < 70 {
        make_corridor(pieces, random, x, y, z, direction, depth)?
    } else if selection < 80 {
        make_stairs(pieces, x, y, z, direction, depth)?
    } else {
        make_crossing(pieces, random, x, y, z, direction, depth)?
    };
    pieces.push(piece);
    let index = pieces.len() - 1;
    expand_piece(index, root_box, kind, pieces, random);
    Some(index)
}

fn make_corridor(
    pieces: &[MineshaftPiece],
    random: &mut impl GenerationRandom,
    x: i32,
    y: i32,
    z: i32,
    direction: HorizontalDirection,
    depth: i32,
) -> Option<MineshaftPiece> {
    let mut sections = 2 + bounded(random, 3) as i32;
    let bounding_box = loop {
        if sections == 0 {
            return None;
        }
        let length = sections * 5;
        let relative = match direction {
            HorizontalDirection::North => ((0, 0, -(length - 1)), (2, 2, 0)),
            HorizontalDirection::South => ((0, 0, 0), (2, 2, length - 1)),
            HorizontalDirection::West => ((-(length - 1), 0, 0), (0, 2, 2)),
            HorizontalDirection::East => ((0, 0, 0), (length - 1, 2, 2)),
        };
        let candidate = moved_box(relative, x, y, z);
        if !collides(pieces, candidate) {
            break candidate;
        }
        sections -= 1;
    };
    let has_rails = bounded(random, 3) == 0;
    let spider_corridor = !has_rails && bounded(random, 23) == 0;
    Some(MineshaftPiece::Corridor(MineshaftCorridor {
        bounding_box,
        generation_depth: depth,
        orientation: direction,
        has_rails,
        spider_corridor,
        has_placed_spider: false,
        sections,
    }))
}

fn make_stairs(
    pieces: &[MineshaftPiece],
    x: i32,
    y: i32,
    z: i32,
    direction: HorizontalDirection,
    depth: i32,
) -> Option<MineshaftPiece> {
    let relative = match direction {
        HorizontalDirection::North => ((0, -5, -8), (2, 2, 0)),
        HorizontalDirection::South => ((0, -5, 0), (2, 2, 8)),
        HorizontalDirection::West => ((-8, -5, 0), (0, 2, 2)),
        HorizontalDirection::East => ((0, -5, 0), (8, 2, 2)),
    };
    let bounding_box = moved_box(relative, x, y, z);
    (!collides(pieces, bounding_box)).then_some(MineshaftPiece::Stairs(MineshaftStairs {
        bounding_box,
        generation_depth: depth,
        orientation: direction,
    }))
}

fn make_crossing(
    pieces: &[MineshaftPiece],
    random: &mut impl GenerationRandom,
    x: i32,
    y: i32,
    z: i32,
    direction: HorizontalDirection,
    depth: i32,
) -> Option<MineshaftPiece> {
    let maximum_y = if bounded(random, 4) == 0 { 6 } else { 2 };
    let relative = match direction {
        HorizontalDirection::North => ((-1, 0, -4), (3, maximum_y, 0)),
        HorizontalDirection::South => ((-1, 0, 0), (3, maximum_y, 4)),
        HorizontalDirection::West => ((-4, 0, -1), (0, maximum_y, 3)),
        HorizontalDirection::East => ((0, 0, -1), (4, maximum_y, 3)),
    };
    let bounding_box = moved_box(relative, x, y, z);
    (!collides(pieces, bounding_box)).then_some(MineshaftPiece::Crossing(MineshaftCrossing {
        bounding_box,
        generation_depth: depth,
        direction,
        two_floored: maximum_y > 2,
    }))
}

fn expand_piece(
    index: usize,
    root_box: BlockBox,
    kind: MineshaftType,
    pieces: &mut Vec<MineshaftPiece>,
    random: &mut impl GenerationRandom,
) {
    match pieces[index].clone() {
        MineshaftPiece::Corridor(piece) => expand_corridor(piece, root_box, kind, pieces, random),
        MineshaftPiece::Crossing(piece) => expand_crossing(piece, root_box, kind, pieces, random),
        MineshaftPiece::Stairs(piece) => {
            let (x, z) = forward_anchor(piece.bounding_box, piece.orientation);
            let _ = add_child(
                root_box,
                kind,
                pieces,
                random,
                (x, piece.bounding_box.minimum.y, z),
                piece.orientation,
                piece.generation_depth,
            );
        }
        MineshaftPiece::Room(_) => {}
    }
}

fn expand_corridor(
    piece: MineshaftCorridor,
    root_box: BlockBox,
    kind: MineshaftType,
    pieces: &mut Vec<MineshaftPiece>,
    random: &mut impl GenerationRandom,
) {
    let end = bounded(random, 4);
    let y = piece.bounding_box.minimum.y - 1 + bounded(random, 3) as i32;
    let (x, z, direction) = corridor_end(piece.bounding_box, piece.orientation, end);
    let _ = add_child(
        root_box,
        kind,
        pieces,
        random,
        (x, y, z),
        direction,
        piece.generation_depth,
    );
    if piece.generation_depth >= 8 {
        return;
    }
    match piece.orientation {
        HorizontalDirection::North | HorizontalDirection::South => {
            let mut z = piece.bounding_box.minimum.z + 3;
            while z + 3 <= piece.bounding_box.maximum.z {
                let selection = bounded(random, 5);
                let side = if selection == 0 {
                    Some((piece.bounding_box.minimum.x - 1, HorizontalDirection::West))
                } else if selection == 1 {
                    Some((piece.bounding_box.maximum.x + 1, HorizontalDirection::East))
                } else {
                    None
                };
                if let Some((x, direction)) = side {
                    let _ = add_child(
                        root_box,
                        kind,
                        pieces,
                        random,
                        (x, piece.bounding_box.minimum.y, z),
                        direction,
                        piece.generation_depth + 1,
                    );
                }
                z += 5;
            }
        }
        HorizontalDirection::West | HorizontalDirection::East => {
            let mut x = piece.bounding_box.minimum.x + 3;
            while x + 3 <= piece.bounding_box.maximum.x {
                let selection = bounded(random, 5);
                let side = if selection == 0 {
                    Some((piece.bounding_box.minimum.z - 1, HorizontalDirection::North))
                } else if selection == 1 {
                    Some((piece.bounding_box.maximum.z + 1, HorizontalDirection::South))
                } else {
                    None
                };
                if let Some((z, direction)) = side {
                    let _ = add_child(
                        root_box,
                        kind,
                        pieces,
                        random,
                        (x, piece.bounding_box.minimum.y, z),
                        direction,
                        piece.generation_depth + 1,
                    );
                }
                x += 5;
            }
        }
    }
}

fn expand_crossing(
    piece: MineshaftCrossing,
    root_box: BlockBox,
    kind: MineshaftType,
    pieces: &mut Vec<MineshaftPiece>,
    random: &mut impl GenerationRandom,
) {
    for direction in lower_crossing_exits(piece.direction) {
        let (x, z) = crossing_anchor(piece.bounding_box, direction);
        let _ = add_child(
            root_box,
            kind,
            pieces,
            random,
            (x, piece.bounding_box.minimum.y, z),
            direction,
            piece.generation_depth,
        );
    }
    if piece.two_floored {
        for direction in [
            HorizontalDirection::North,
            HorizontalDirection::West,
            HorizontalDirection::East,
            HorizontalDirection::South,
        ] {
            if random.next_bool() {
                let (x, z) = crossing_anchor(piece.bounding_box, direction);
                let _ = add_child(
                    root_box,
                    kind,
                    pieces,
                    random,
                    (x, piece.bounding_box.minimum.y + 4, z),
                    direction,
                    piece.generation_depth,
                );
            }
        }
    }
}

fn lower_crossing_exits(entry: HorizontalDirection) -> [HorizontalDirection; 3] {
    match entry {
        HorizontalDirection::North => [
            HorizontalDirection::North,
            HorizontalDirection::West,
            HorizontalDirection::East,
        ],
        HorizontalDirection::South => [
            HorizontalDirection::South,
            HorizontalDirection::West,
            HorizontalDirection::East,
        ],
        HorizontalDirection::West => [
            HorizontalDirection::North,
            HorizontalDirection::South,
            HorizontalDirection::West,
        ],
        HorizontalDirection::East => [
            HorizontalDirection::North,
            HorizontalDirection::South,
            HorizontalDirection::East,
        ],
    }
}

fn corridor_end(
    bounding_box: BlockBox,
    orientation: HorizontalDirection,
    selection: u32,
) -> (i32, i32, HorizontalDirection) {
    match orientation {
        HorizontalDirection::North if selection <= 1 => (
            bounding_box.minimum.x,
            bounding_box.minimum.z - 1,
            orientation,
        ),
        HorizontalDirection::North if selection == 2 => (
            bounding_box.minimum.x - 1,
            bounding_box.minimum.z,
            HorizontalDirection::West,
        ),
        HorizontalDirection::North => (
            bounding_box.maximum.x + 1,
            bounding_box.minimum.z,
            HorizontalDirection::East,
        ),
        HorizontalDirection::South if selection <= 1 => (
            bounding_box.minimum.x,
            bounding_box.maximum.z + 1,
            orientation,
        ),
        HorizontalDirection::South if selection == 2 => (
            bounding_box.minimum.x - 1,
            bounding_box.maximum.z - 3,
            HorizontalDirection::West,
        ),
        HorizontalDirection::South => (
            bounding_box.maximum.x + 1,
            bounding_box.maximum.z - 3,
            HorizontalDirection::East,
        ),
        HorizontalDirection::West if selection <= 1 => (
            bounding_box.minimum.x - 1,
            bounding_box.minimum.z,
            orientation,
        ),
        HorizontalDirection::West if selection == 2 => (
            bounding_box.minimum.x,
            bounding_box.minimum.z - 1,
            HorizontalDirection::North,
        ),
        HorizontalDirection::West => (
            bounding_box.minimum.x,
            bounding_box.maximum.z + 1,
            HorizontalDirection::South,
        ),
        HorizontalDirection::East if selection <= 1 => (
            bounding_box.maximum.x + 1,
            bounding_box.minimum.z,
            orientation,
        ),
        HorizontalDirection::East if selection == 2 => (
            bounding_box.maximum.x - 3,
            bounding_box.minimum.z - 1,
            HorizontalDirection::North,
        ),
        HorizontalDirection::East => (
            bounding_box.maximum.x - 3,
            bounding_box.maximum.z + 1,
            HorizontalDirection::South,
        ),
    }
}

fn forward_anchor(box_: BlockBox, direction: HorizontalDirection) -> (i32, i32) {
    match direction {
        HorizontalDirection::North => (box_.minimum.x, box_.minimum.z - 1),
        HorizontalDirection::South => (box_.minimum.x, box_.maximum.z + 1),
        HorizontalDirection::West => (box_.minimum.x - 1, box_.minimum.z),
        HorizontalDirection::East => (box_.maximum.x + 1, box_.minimum.z),
    }
}

fn crossing_anchor(box_: BlockBox, direction: HorizontalDirection) -> (i32, i32) {
    match direction {
        HorizontalDirection::North => (box_.minimum.x + 1, box_.minimum.z - 1),
        HorizontalDirection::South => (box_.minimum.x + 1, box_.maximum.z + 1),
        HorizontalDirection::West => (box_.minimum.x - 1, box_.minimum.z + 1),
        HorizontalDirection::East => (box_.maximum.x + 1, box_.minimum.z + 1),
    }
}

fn moved_box(
    (minimum, maximum): ((i32, i32, i32), (i32, i32, i32)),
    x: i32,
    y: i32,
    z: i32,
) -> BlockBox {
    BlockBox::new(
        BlockPos::new(x + minimum.0, y + minimum.1, z + minimum.2),
        BlockPos::new(x + maximum.0, y + maximum.1, z + maximum.2),
    )
    .expect("factory boxes are ordered")
}

fn collides(pieces: &[MineshaftPiece], candidate: BlockBox) -> bool {
    pieces
        .iter()
        .any(|piece| piece.bounding_box().intersects(candidate))
}

fn union_boxes(pieces: &[MineshaftPiece]) -> BlockBox {
    pieces
        .iter()
        .map(MineshaftPiece::bounding_box)
        .reduce(BlockBox::union)
        .expect("a mineshaft always has its room")
}

fn below_sea_offset(
    union: BlockBox,
    sea_level: i32,
    minimum_y: i32,
    random: &mut impl GenerationRandom,
) -> i32 {
    let target = sea_level - 10;
    let lower = union.size()[1] + minimum_y + 1;
    let selected = if lower < target {
        lower + bounded(random, (target - lower) as u32) as i32
    } else {
        lower
    };
    selected - union.maximum.y
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("positive mineshaft bound"))
}
