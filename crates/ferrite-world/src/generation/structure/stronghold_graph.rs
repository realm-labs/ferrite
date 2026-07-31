//! Stronghold retry loop, weighted frontier graph, boxes, and relocation.

use std::num::NonZeroU32;
use std::sync::atomic::{AtomicI32, Ordering};

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};
use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::{HorizontalDirection, OrientedPiece};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrongholdPieceKind {
    Start,
    Straight,
    PrisonHall,
    LeftTurn,
    RightTurn,
    RoomCrossing,
    StraightStairsDown,
    StairsDown,
    FiveCrossing,
    ChestCorridor,
    Library,
    PortalRoom,
    FillerCorridor,
}

impl StrongholdPieceKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Start => "minecraft:shstart",
            Self::Straight => "minecraft:shs",
            Self::PrisonHall => "minecraft:shph",
            Self::LeftTurn => "minecraft:shlt",
            Self::RightTurn => "minecraft:shrt",
            Self::RoomCrossing => "minecraft:shrc",
            Self::StraightStairsDown => "minecraft:shssd",
            Self::StairsDown => "minecraft:shsd",
            Self::FiveCrossing => "minecraft:sh5c",
            Self::ChestCorridor => "minecraft:shcc",
            Self::Library => "minecraft:shli",
            Self::PortalRoom => "minecraft:shpr",
            Self::FillerCorridor => "minecraft:shfc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrongholdDoor {
    Opening,
    Wood,
    Grates,
    Iron,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrongholdPiece {
    pub kind: StrongholdPieceKind,
    pub bounding_box: BlockBox,
    pub generation_depth: i32,
    pub orientation: HorizontalDirection,
    pub entry_door: StrongholdDoor,
    pub source: bool,
    pub left_child: bool,
    pub right_child: bool,
    pub room_type: i32,
    pub low_left: bool,
    pub high_left: bool,
    pub low_right: bool,
    pub high_right: bool,
    pub chest_pending: bool,
    pub tall_library: bool,
    pub spawner_placed: bool,
    pub filler_steps: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrongholdGraph {
    pub stub_position: BlockPos,
    pub pieces: Vec<StrongholdPiece>,
    pub portal_room: usize,
    pub locator_position: BlockPos,
    pub attempts: u32,
    pub vertical_offset: i32,
}

#[derive(Clone, Copy)]
struct PieceWeight {
    kind: StrongholdPieceKind,
    weight: i32,
    maximum: i32,
    minimum_depth: i32,
}

const WEIGHTS: [PieceWeight; 11] = [
    weight(StrongholdPieceKind::Straight, 40, 0, 0),
    weight(StrongholdPieceKind::PrisonHall, 5, 5, 0),
    weight(StrongholdPieceKind::LeftTurn, 20, 0, 0),
    weight(StrongholdPieceKind::RightTurn, 20, 0, 0),
    weight(StrongholdPieceKind::RoomCrossing, 10, 6, 0),
    weight(StrongholdPieceKind::StraightStairsDown, 5, 5, 0),
    weight(StrongholdPieceKind::StairsDown, 5, 5, 0),
    weight(StrongholdPieceKind::FiveCrossing, 5, 4, 0),
    weight(StrongholdPieceKind::ChestCorridor, 5, 4, 0),
    weight(StrongholdPieceKind::Library, 10, 2, 4),
    weight(StrongholdPieceKind::PortalRoom, 20, 1, 5),
];

// The Java implementation mutates shared PieceWeight instances. Relaxed atomics retain those
// reset/increment interleavings without importing a data race into Rust.
static PLACEMENT_COUNTS: [AtomicI32; 11] = [
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
    AtomicI32::new(0),
];

struct GraphBuilder<'a> {
    random: &'a mut LegacyRandom,
    pieces: Vec<StrongholdPiece>,
    pending: Vec<usize>,
    available: Vec<usize>,
    previous: Option<usize>,
    imposed: Option<StrongholdPieceKind>,
    portal_room: Option<usize>,
    start_box: BlockBox,
}

pub fn generate_stronghold(
    world_seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    sea_level: i32,
    minimum_build_y: i32,
) -> StrongholdGraph {
    let mut attempt = 0_u32;
    loop {
        let mut random = LegacyRandom::new(0);
        random.set_large_feature_seed(
            world_seed.wrapping_add(i64::from(attempt)),
            chunk_x,
            chunk_z,
        );
        let mut builder = GraphBuilder::new(chunk_x, chunk_z, &mut random);
        builder.expand_piece(0);
        while !builder.pending.is_empty() {
            let selected = bounded(builder.random, builder.pending.len() as u32) as usize;
            let piece = builder.pending.remove(selected);
            builder.expand_piece(piece);
        }
        let Some(portal_room) = builder.portal_room else {
            attempt = attempt.wrapping_add(1);
            continue;
        };
        let union = builder
            .pieces
            .iter()
            .map(|piece| piece.bounding_box)
            .reduce(BlockBox::union)
            .expect("stronghold contains its start");
        let target = sea_level - 10;
        let mut destination = union.size()[1] + minimum_build_y + 1;
        if destination < target {
            destination += bounded(builder.random, (target - destination) as u32) as i32;
        }
        let vertical_offset = destination - union.maximum.y;
        for piece in &mut builder.pieces {
            piece.bounding_box = piece.bounding_box.moved([0, vertical_offset, 0]);
        }
        let locator_position = builder.pieces[portal_room].bounding_box.center();
        return StrongholdGraph {
            stub_position: BlockPos::new(chunk_x.wrapping_mul(16), 0, chunk_z.wrapping_mul(16)),
            pieces: builder.pieces,
            portal_room,
            locator_position,
            attempts: attempt + 1,
            vertical_offset,
        };
    }
}

impl<'a> GraphBuilder<'a> {
    fn new(chunk_x: i32, chunk_z: i32, random: &'a mut LegacyRandom) -> Self {
        for count in &PLACEMENT_COUNTS {
            count.store(0, Ordering::Relaxed);
        }
        let orientation = HorizontalDirection::ALL[bounded(random, 4) as usize];
        let anchor = BlockPos::new(
            chunk_x.wrapping_mul(16).wrapping_add(2),
            64,
            chunk_z.wrapping_mul(16).wrapping_add(2),
        );
        let start_box =
            OrientedPiece::from_anchor(anchor, BlockPos::new(0, 0, 0), [5, 11, 5], orientation)
                .bounds;
        Self {
            random,
            pieces: vec![StrongholdPiece {
                kind: StrongholdPieceKind::Start,
                bounding_box: start_box,
                generation_depth: 0,
                orientation,
                entry_door: StrongholdDoor::Opening,
                source: true,
                left_child: false,
                right_child: false,
                room_type: 0,
                low_left: false,
                high_left: false,
                low_right: false,
                high_right: false,
                chest_pending: false,
                tall_library: false,
                spawner_placed: false,
                filler_steps: 0,
            }],
            pending: Vec::new(),
            available: (0..WEIGHTS.len()).collect(),
            previous: None,
            imposed: Some(StrongholdPieceKind::FiveCrossing),
            portal_room: None,
            start_box,
        }
    }

    fn expand_piece(&mut self, index: usize) {
        let piece = self.pieces[index].clone();
        match piece.kind {
            StrongholdPieceKind::Start => self.forward(&piece, 1, 1),
            StrongholdPieceKind::Straight => {
                self.forward(&piece, 1, 1);
                if piece.left_child {
                    self.left(&piece, 1, 2);
                }
                if piece.right_child {
                    self.right(&piece, 1, 2);
                }
            }
            StrongholdPieceKind::PrisonHall
            | StrongholdPieceKind::StraightStairsDown
            | StrongholdPieceKind::StairsDown
            | StrongholdPieceKind::ChestCorridor => self.forward(&piece, 1, 1),
            StrongholdPieceKind::LeftTurn => match piece.orientation {
                HorizontalDirection::North | HorizontalDirection::East => self.left(&piece, 1, 1),
                HorizontalDirection::South | HorizontalDirection::West => self.right(&piece, 1, 1),
            },
            StrongholdPieceKind::RightTurn => match piece.orientation {
                HorizontalDirection::North | HorizontalDirection::East => self.right(&piece, 1, 1),
                HorizontalDirection::South | HorizontalDirection::West => self.left(&piece, 1, 1),
            },
            StrongholdPieceKind::RoomCrossing => {
                self.forward(&piece, 4, 1);
                self.left(&piece, 1, 4);
                self.right(&piece, 1, 4);
            }
            StrongholdPieceKind::FiveCrossing => {
                let (low_y, high_y) = if matches!(
                    piece.orientation,
                    HorizontalDirection::West | HorizontalDirection::North
                ) {
                    (5, 3)
                } else {
                    (3, 5)
                };
                self.forward(&piece, 5, 1);
                if piece.low_left {
                    self.left(&piece, low_y, 1);
                }
                if piece.low_right {
                    self.right(&piece, low_y, 1);
                }
                if piece.high_left {
                    self.left(&piece, high_y, 7);
                }
                if piece.high_right {
                    self.right(&piece, high_y, 7);
                }
            }
            StrongholdPieceKind::Library
            | StrongholdPieceKind::PortalRoom
            | StrongholdPieceKind::FillerCorridor => {}
        }
    }

    fn forward(&mut self, parent: &StrongholdPiece, x_offset: i32, y_offset: i32) {
        let box_ = parent.bounding_box;
        let (anchor, direction) = match parent.orientation {
            HorizontalDirection::North => (
                BlockPos::new(
                    box_.minimum.x + x_offset,
                    box_.minimum.y + y_offset,
                    box_.minimum.z - 1,
                ),
                HorizontalDirection::North,
            ),
            HorizontalDirection::South => (
                BlockPos::new(
                    box_.minimum.x + x_offset,
                    box_.minimum.y + y_offset,
                    box_.maximum.z + 1,
                ),
                HorizontalDirection::South,
            ),
            HorizontalDirection::West => (
                BlockPos::new(
                    box_.minimum.x - 1,
                    box_.minimum.y + y_offset,
                    box_.minimum.z + x_offset,
                ),
                HorizontalDirection::West,
            ),
            HorizontalDirection::East => (
                BlockPos::new(
                    box_.maximum.x + 1,
                    box_.minimum.y + y_offset,
                    box_.minimum.z + x_offset,
                ),
                HorizontalDirection::East,
            ),
        };
        self.request(parent.generation_depth, anchor, direction);
    }

    fn left(&mut self, parent: &StrongholdPiece, y_offset: i32, z_offset: i32) {
        let box_ = parent.bounding_box;
        let (anchor, direction) = match parent.orientation {
            HorizontalDirection::North | HorizontalDirection::South => (
                BlockPos::new(
                    box_.minimum.x - 1,
                    box_.minimum.y + y_offset,
                    box_.minimum.z + z_offset,
                ),
                HorizontalDirection::West,
            ),
            HorizontalDirection::West | HorizontalDirection::East => (
                BlockPos::new(
                    box_.minimum.x + z_offset,
                    box_.minimum.y + y_offset,
                    box_.minimum.z - 1,
                ),
                HorizontalDirection::North,
            ),
        };
        self.request(parent.generation_depth, anchor, direction);
    }

    fn right(&mut self, parent: &StrongholdPiece, y_offset: i32, z_offset: i32) {
        let box_ = parent.bounding_box;
        let (anchor, direction) = match parent.orientation {
            HorizontalDirection::North | HorizontalDirection::South => (
                BlockPos::new(
                    box_.maximum.x + 1,
                    box_.minimum.y + y_offset,
                    box_.minimum.z + z_offset,
                ),
                HorizontalDirection::East,
            ),
            HorizontalDirection::West | HorizontalDirection::East => (
                BlockPos::new(
                    box_.minimum.x + z_offset,
                    box_.minimum.y + y_offset,
                    box_.maximum.z + 1,
                ),
                HorizontalDirection::South,
            ),
        };
        self.request(parent.generation_depth, anchor, direction);
    }

    fn request(&mut self, parent_depth: i32, anchor: BlockPos, direction: HorizontalDirection) {
        if parent_depth > 50
            || anchor
                .x
                .wrapping_sub(self.start_box.minimum.x)
                .wrapping_abs()
                > 112
            || anchor
                .z
                .wrapping_sub(self.start_box.minimum.z)
                .wrapping_abs()
                > 112
        {
            return;
        }
        let depth = parent_depth + 1;
        let piece = self
            .imposed
            .take()
            .and_then(|kind| self.factory(kind, anchor, direction, depth))
            .or_else(|| self.select(anchor, direction, depth));
        if let Some(piece) = piece {
            let index = self.pieces.len();
            if piece.kind == StrongholdPieceKind::PortalRoom {
                self.portal_room = Some(index);
            }
            self.pieces.push(piece);
            self.pending.push(index);
        }
    }

    fn select(
        &mut self,
        anchor: BlockPos,
        direction: HorizontalDirection,
        depth: i32,
    ) -> Option<StrongholdPiece> {
        let (total, has_finite) = self.update_weights();
        if !has_finite {
            return None;
        }
        for _ in 0..5 {
            let mut selected = bounded(self.random, total as u32) as i32;
            let available = self.available.clone();
            for weight_index in available {
                let weight = WEIGHTS[weight_index];
                selected -= weight.weight;
                if selected >= 0 {
                    continue;
                }
                let count = PLACEMENT_COUNTS[weight_index].load(Ordering::Relaxed);
                if depth <= weight.minimum_depth
                    || (weight.maximum > 0 && count >= weight.maximum)
                    || self.previous == Some(weight_index)
                {
                    break;
                }
                if let Some(piece) = self.factory(weight.kind, anchor, direction, depth) {
                    let next = PLACEMENT_COUNTS[weight_index].fetch_add(1, Ordering::Relaxed) + 1;
                    self.previous = Some(weight_index);
                    if weight.maximum > 0 && next >= weight.maximum {
                        self.available.retain(|index| *index != weight_index);
                    }
                    return Some(piece);
                }
            }
        }
        self.filler(anchor, direction, depth)
    }

    fn update_weights(&self) -> (i32, bool) {
        let mut total = 0;
        let mut has_finite = false;
        for &index in &self.available {
            let weight = WEIGHTS[index];
            total += weight.weight;
            if weight.maximum > 0
                && PLACEMENT_COUNTS[index].load(Ordering::Relaxed) < weight.maximum
            {
                has_finite = true;
            }
        }
        (total, has_finite)
    }

    fn factory(
        &mut self,
        kind: StrongholdPieceKind,
        anchor: BlockPos,
        direction: HorizontalDirection,
        depth: i32,
    ) -> Option<StrongholdPiece> {
        let (offset, size) = dimensions(kind);
        let mut box_ = OrientedPiece::from_anchor(anchor, offset, size, direction).bounds;
        let mut tall = false;
        if kind == StrongholdPieceKind::Library && !self.admitted(box_) {
            box_ = OrientedPiece::from_anchor(anchor, offset, [14, 6, 15], direction).bounds;
        } else if kind == StrongholdPieceKind::Library {
            tall = true;
        }
        if !self.admitted(box_) {
            return None;
        }
        let door = match kind {
            StrongholdPieceKind::PortalRoom => StrongholdDoor::Opening,
            _ => random_door(self.random),
        };
        let mut piece = blank_piece(kind, box_, depth, direction, door);
        match kind {
            StrongholdPieceKind::Straight => {
                piece.left_child = bounded(self.random, 2) == 0;
                piece.right_child = bounded(self.random, 2) == 0;
            }
            StrongholdPieceKind::RoomCrossing => piece.room_type = bounded(self.random, 5) as i32,
            StrongholdPieceKind::FiveCrossing => {
                piece.low_left = self.random.next_bool();
                piece.high_left = self.random.next_bool();
                piece.low_right = self.random.next_bool();
                piece.high_right = bounded(self.random, 3) > 0;
            }
            StrongholdPieceKind::ChestCorridor => piece.chest_pending = true,
            StrongholdPieceKind::Library => piece.tall_library = tall,
            _ => {}
        }
        Some(piece)
    }

    fn filler(
        &self,
        anchor: BlockPos,
        direction: HorizontalDirection,
        depth: i32,
    ) -> Option<StrongholdPiece> {
        let full =
            OrientedPiece::from_anchor(anchor, BlockPos::new(-1, -1, 0), [5, 5, 4], direction)
                .bounds;
        let blocker = self
            .pieces
            .iter()
            .find(|piece| piece.bounding_box.intersects(full))?;
        if blocker.bounding_box.minimum.y != full.minimum.y {
            return None;
        }
        for length in (1..=2).rev() {
            let short = OrientedPiece::from_anchor(
                anchor,
                BlockPos::new(-1, -1, 0),
                [5, 5, length],
                direction,
            )
            .bounds;
            if !blocker.bounding_box.intersects(short) {
                let steps = length + 1;
                let box_ = OrientedPiece::from_anchor(
                    anchor,
                    BlockPos::new(-1, -1, 0),
                    [5, 5, steps],
                    direction,
                )
                .bounds;
                if box_.minimum.y <= 1 {
                    return None;
                }
                let mut piece = blank_piece(
                    StrongholdPieceKind::FillerCorridor,
                    box_,
                    depth,
                    direction,
                    StrongholdDoor::Opening,
                );
                piece.filler_steps = steps;
                return Some(piece);
            }
        }
        None
    }

    fn admitted(&self, box_: BlockBox) -> bool {
        box_.minimum.y > 10
            && self
                .pieces
                .iter()
                .all(|piece| !piece.bounding_box.intersects(box_))
    }
}

const fn dimensions(kind: StrongholdPieceKind) -> (BlockPos, [i32; 3]) {
    match kind {
        StrongholdPieceKind::Straight | StrongholdPieceKind::ChestCorridor => {
            (BlockPos::new(-1, -1, 0), [5, 5, 7])
        }
        StrongholdPieceKind::PrisonHall => (BlockPos::new(-1, -1, 0), [9, 5, 11]),
        StrongholdPieceKind::LeftTurn | StrongholdPieceKind::RightTurn => {
            (BlockPos::new(-1, -1, 0), [5, 5, 5])
        }
        StrongholdPieceKind::RoomCrossing => (BlockPos::new(-4, -1, 0), [11, 7, 11]),
        StrongholdPieceKind::StraightStairsDown => (BlockPos::new(-1, -7, 0), [5, 11, 8]),
        StrongholdPieceKind::StairsDown => (BlockPos::new(-1, -7, 0), [5, 11, 5]),
        StrongholdPieceKind::FiveCrossing => (BlockPos::new(-4, -3, 0), [10, 9, 11]),
        StrongholdPieceKind::Library => (BlockPos::new(-4, -1, 0), [14, 11, 15]),
        StrongholdPieceKind::PortalRoom => (BlockPos::new(-4, -1, 0), [11, 8, 16]),
        StrongholdPieceKind::Start | StrongholdPieceKind::FillerCorridor => {
            (BlockPos::new(0, 0, 0), [1, 1, 1])
        }
    }
}

fn blank_piece(
    kind: StrongholdPieceKind,
    bounding_box: BlockBox,
    generation_depth: i32,
    orientation: HorizontalDirection,
    entry_door: StrongholdDoor,
) -> StrongholdPiece {
    StrongholdPiece {
        kind,
        bounding_box,
        generation_depth,
        orientation,
        entry_door,
        source: false,
        left_child: false,
        right_child: false,
        room_type: 0,
        low_left: false,
        high_left: false,
        low_right: false,
        high_right: false,
        chest_pending: false,
        tall_library: false,
        spawner_placed: false,
        filler_steps: 0,
    }
}

fn random_door(random: &mut impl GenerationRandom) -> StrongholdDoor {
    match bounded(random, 5) {
        0 | 1 => StrongholdDoor::Opening,
        2 => StrongholdDoor::Wood,
        3 => StrongholdDoor::Grates,
        4 => StrongholdDoor::Iron,
        _ => unreachable!("door draw is bounded to five"),
    }
}

const fn weight(
    kind: StrongholdPieceKind,
    weight: i32,
    maximum: i32,
    minimum_depth: i32,
) -> PieceWeight {
    PieceWeight {
        kind,
        weight,
        maximum,
        minimum_depth,
    }
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("positive stronghold bound"))
}
