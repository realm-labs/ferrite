//! Nether-fortress weighted frontier graph construction and relocation.

use std::num::NonZeroU32;
use std::sync::atomic::{AtomicI32, Ordering};

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::{HorizontalDirection, OrientedPiece};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FortressPieceKind {
    Start,
    BridgeStraight,
    BridgeCrossing,
    RoomCrossing,
    StairsRoom,
    MonsterThrone,
    CastleEntrance,
    CastleSmallCorridor,
    CastleSmallCrossing,
    CastleRightTurn,
    CastleLeftTurn,
    CastleCorridorStairs,
    CastleTBalcony,
    CastleStalkRoom,
    BridgeEndFiller,
}

impl FortressPieceKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Start => "minecraft:nestart",
            Self::BridgeStraight => "minecraft:nebs",
            Self::BridgeCrossing => "minecraft:nebcr",
            Self::RoomCrossing => "minecraft:nerc",
            Self::StairsRoom => "minecraft:nesr",
            Self::MonsterThrone => "minecraft:nemt",
            Self::CastleEntrance => "minecraft:nece",
            Self::CastleSmallCorridor => "minecraft:nesc",
            Self::CastleSmallCrossing => "minecraft:nescsc",
            Self::CastleRightTurn => "minecraft:nescrt",
            Self::CastleLeftTurn => "minecraft:nesclt",
            Self::CastleCorridorStairs => "minecraft:neccs",
            Self::CastleTBalcony => "minecraft:nectb",
            Self::CastleStalkRoom => "minecraft:necsr",
            Self::BridgeEndFiller => "minecraft:nebef",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FortressPiece {
    pub kind: FortressPieceKind,
    pub bounding_box: BlockBox,
    pub generation_depth: i32,
    pub orientation: HorizontalDirection,
    pub chest_pending: bool,
    pub spawner_placed: bool,
    pub filler_seed: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FortressGraph {
    pub stub_position: BlockPos,
    pub pieces: Vec<FortressPiece>,
    pub vertical_offset: i32,
}

#[derive(Clone, Copy)]
struct PieceWeight {
    kind: FortressPieceKind,
    weight: i32,
    maximum: i32,
    allow_in_row: bool,
}

const BRIDGE_WEIGHTS: [usize; 6] = [0, 1, 2, 3, 4, 5];
const CASTLE_WEIGHTS: [usize; 7] = [6, 7, 8, 9, 10, 11, 12];
const WEIGHTS: [PieceWeight; 13] = [
    PieceWeight {
        kind: FortressPieceKind::BridgeStraight,
        weight: 30,
        maximum: 0,
        allow_in_row: true,
    },
    PieceWeight {
        kind: FortressPieceKind::BridgeCrossing,
        weight: 10,
        maximum: 4,
        allow_in_row: false,
    },
    PieceWeight {
        kind: FortressPieceKind::RoomCrossing,
        weight: 10,
        maximum: 4,
        allow_in_row: false,
    },
    PieceWeight {
        kind: FortressPieceKind::StairsRoom,
        weight: 10,
        maximum: 3,
        allow_in_row: false,
    },
    PieceWeight {
        kind: FortressPieceKind::MonsterThrone,
        weight: 5,
        maximum: 2,
        allow_in_row: false,
    },
    PieceWeight {
        kind: FortressPieceKind::CastleEntrance,
        weight: 5,
        maximum: 1,
        allow_in_row: false,
    },
    PieceWeight {
        kind: FortressPieceKind::CastleSmallCorridor,
        weight: 25,
        maximum: 0,
        allow_in_row: true,
    },
    PieceWeight {
        kind: FortressPieceKind::CastleSmallCrossing,
        weight: 15,
        maximum: 5,
        allow_in_row: false,
    },
    PieceWeight {
        kind: FortressPieceKind::CastleRightTurn,
        weight: 5,
        maximum: 10,
        allow_in_row: false,
    },
    PieceWeight {
        kind: FortressPieceKind::CastleLeftTurn,
        weight: 5,
        maximum: 10,
        allow_in_row: false,
    },
    PieceWeight {
        kind: FortressPieceKind::CastleCorridorStairs,
        weight: 10,
        maximum: 3,
        allow_in_row: true,
    },
    PieceWeight {
        kind: FortressPieceKind::CastleTBalcony,
        weight: 7,
        maximum: 2,
        allow_in_row: false,
    },
    PieceWeight {
        kind: FortressPieceKind::CastleStalkRoom,
        weight: 5,
        maximum: 2,
        allow_in_row: false,
    },
];

// Mojang stores these counters in shared mutable PieceWeight singletons. Relaxed atomics preserve
// the observable reset/increment interleavings without introducing undefined behavior in Rust.
static PLACEMENT_COUNTS: [AtomicI32; 13] = [
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
    AtomicI32::new(0),
    AtomicI32::new(0),
];

struct GraphBuilder<'a, R> {
    random: &'a mut R,
    pieces: Vec<FortressPiece>,
    pending: Vec<usize>,
    bridge: Vec<usize>,
    castle: Vec<usize>,
    previous: Option<usize>,
    start_box: BlockBox,
}

pub fn generate_fortress(
    chunk_x: i32,
    chunk_z: i32,
    random: &mut impl GenerationRandom,
) -> FortressGraph {
    let start_x = chunk_x.wrapping_mul(16).wrapping_add(2);
    let start_z = chunk_z.wrapping_mul(16).wrapping_add(2);
    let orientation = HorizontalDirection::ALL[bounded(random, 4) as usize];
    for count in &PLACEMENT_COUNTS {
        count.store(0, Ordering::Relaxed);
    }
    let start_box = BlockBox::new(
        BlockPos::new(start_x, 64, start_z),
        BlockPos::new(start_x + 18, 73, start_z + 18),
    )
    .expect("fortress start dimensions are positive");
    let start = FortressPiece {
        kind: FortressPieceKind::Start,
        bounding_box: start_box,
        generation_depth: 0,
        orientation,
        chest_pending: false,
        spawner_placed: false,
        filler_seed: 0,
    };
    let mut builder = GraphBuilder {
        random,
        pieces: vec![start],
        pending: Vec::new(),
        bridge: BRIDGE_WEIGHTS.to_vec(),
        castle: CASTLE_WEIGHTS.to_vec(),
        previous: None,
        start_box,
    };
    builder.expand_piece(0);
    while !builder.pending.is_empty() {
        let selected = bounded(builder.random, builder.pending.len() as u32) as usize;
        let piece = builder.pending.remove(selected);
        builder.expand_piece(piece);
    }
    let union = builder
        .pieces
        .iter()
        .map(|piece| piece.bounding_box)
        .reduce(BlockBox::union)
        .expect("fortress contains its start");
    let range = 23 - union.size()[1];
    let target_minimum = if range > 1 {
        48 + bounded(builder.random, range as u32) as i32
    } else {
        48
    };
    let vertical_offset = target_minimum.wrapping_sub(union.minimum.y);
    for piece in &mut builder.pieces {
        piece.bounding_box = piece.bounding_box.moved([0, vertical_offset, 0]);
    }
    FortressGraph {
        stub_position: BlockPos::new(chunk_x.wrapping_mul(16), 64, chunk_z.wrapping_mul(16)),
        pieces: builder.pieces,
        vertical_offset,
    }
}

impl<R: GenerationRandom> GraphBuilder<'_, R> {
    fn expand_piece(&mut self, index: usize) {
        let piece = self.pieces[index].clone();
        match piece.kind {
            FortressPieceKind::Start | FortressPieceKind::BridgeCrossing => {
                self.forward(&piece, 8, 3, false);
                self.left(&piece, 3, 8, false);
                self.right(&piece, 3, 8, false);
            }
            FortressPieceKind::BridgeStraight => self.forward(&piece, 1, 3, false),
            FortressPieceKind::RoomCrossing => {
                self.forward(&piece, 2, 0, false);
                self.left(&piece, 0, 2, false);
                self.right(&piece, 0, 2, false);
            }
            FortressPieceKind::StairsRoom => self.right(&piece, 6, 2, false),
            FortressPieceKind::CastleEntrance => self.forward(&piece, 5, 3, true),
            FortressPieceKind::CastleSmallCorridor => self.forward(&piece, 1, 0, true),
            FortressPieceKind::CastleRightTurn => self.right(&piece, 0, 1, true),
            FortressPieceKind::CastleLeftTurn => self.left(&piece, 0, 1, true),
            FortressPieceKind::CastleCorridorStairs => self.forward(&piece, 1, 0, true),
            FortressPieceKind::CastleTBalcony => {
                let z_offset = match piece.orientation {
                    HorizontalDirection::North | HorizontalDirection::West => 5,
                    HorizontalDirection::South | HorizontalDirection::East => 1,
                };
                let castle = bounded(self.random, 8) > 0;
                self.left(&piece, 0, z_offset, castle);
                let castle = bounded(self.random, 8) > 0;
                self.right(&piece, 0, z_offset, castle);
            }
            FortressPieceKind::CastleSmallCrossing => {
                self.forward(&piece, 1, 0, true);
                self.left(&piece, 0, 1, true);
                self.right(&piece, 0, 1, true);
            }
            FortressPieceKind::CastleStalkRoom => {
                self.forward(&piece, 5, 3, true);
                self.forward(&piece, 5, 11, true);
            }
            FortressPieceKind::MonsterThrone | FortressPieceKind::BridgeEndFiller => {}
        }
    }

    fn forward(&mut self, parent: &FortressPiece, x_offset: i32, y_offset: i32, castle: bool) {
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
        self.request(parent.generation_depth, anchor, direction, castle);
    }

    fn left(&mut self, parent: &FortressPiece, y_offset: i32, z_offset: i32, castle: bool) {
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
        self.request(parent.generation_depth, anchor, direction, castle);
    }

    fn right(&mut self, parent: &FortressPiece, y_offset: i32, z_offset: i32, castle: bool) {
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
        self.request(parent.generation_depth, anchor, direction, castle);
    }

    fn request(
        &mut self,
        parent_depth: i32,
        anchor: BlockPos,
        direction: HorizontalDirection,
        castle: bool,
    ) {
        if anchor
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
            // The source returns this proposal directly and does not retain it in the builder.
            let _ = self.factory(
                FortressPieceKind::BridgeEndFiller,
                anchor,
                direction,
                parent_depth,
            );
            return;
        }
        let depth = parent_depth + 1;
        let mut available = if castle {
            std::mem::take(&mut self.castle)
        } else {
            std::mem::take(&mut self.bridge)
        };
        let piece = self.select(&mut available, anchor, direction, depth);
        if castle {
            self.castle = available;
        } else {
            self.bridge = available;
        }
        if let Some(piece) = piece {
            let index = self.pieces.len();
            self.pieces.push(piece);
            self.pending.push(index);
        }
    }

    fn select(
        &mut self,
        available: &mut Vec<usize>,
        anchor: BlockPos,
        direction: HorizontalDirection,
        depth: i32,
    ) -> Option<FortressPiece> {
        let total = total_weight(available);
        if total > 0 && depth <= 30 {
            for _ in 0..5 {
                let mut selection = bounded(self.random, total as u32) as i32;
                for &weight_index in available.iter() {
                    let weight = WEIGHTS[weight_index];
                    selection -= weight.weight;
                    if selection >= 0 {
                        continue;
                    }
                    let count = PLACEMENT_COUNTS[weight_index].load(Ordering::Relaxed);
                    if (weight.maximum != 0 && count >= weight.maximum)
                        || (self.previous == Some(weight_index) && !weight.allow_in_row)
                    {
                        break;
                    }
                    let Some(piece) = self.factory(weight.kind, anchor, direction, depth) else {
                        continue;
                    };
                    let placed = PLACEMENT_COUNTS[weight_index].fetch_add(1, Ordering::Relaxed) + 1;
                    self.previous = Some(weight_index);
                    if weight.maximum != 0 && placed >= weight.maximum {
                        available.retain(|candidate| *candidate != weight_index);
                    }
                    return Some(piece);
                }
            }
        }
        self.factory(FortressPieceKind::BridgeEndFiller, anchor, direction, depth)
    }

    fn factory(
        &mut self,
        kind: FortressPieceKind,
        anchor: BlockPos,
        direction: HorizontalDirection,
        depth: i32,
    ) -> Option<FortressPiece> {
        let (offset, size) = piece_shape(kind)?;
        let bounding_box = OrientedPiece::from_anchor(anchor, offset, size, direction).bounds;
        if bounding_box.minimum.y <= 10
            || self
                .pieces
                .iter()
                .any(|piece| piece.bounding_box.intersects(bounding_box))
        {
            return None;
        }
        let chest_pending = matches!(
            kind,
            FortressPieceKind::CastleRightTurn | FortressPieceKind::CastleLeftTurn
        ) && bounded(self.random, 3) == 0;
        let filler_seed = if kind == FortressPieceKind::BridgeEndFiller {
            self.random.next_i32()
        } else {
            0
        };
        Some(FortressPiece {
            kind,
            bounding_box,
            generation_depth: depth,
            orientation: direction,
            chest_pending,
            spawner_placed: false,
            filler_seed,
        })
    }
}

fn total_weight(available: &[usize]) -> i32 {
    let mut has_finite = false;
    let mut total = 0;
    for &index in available {
        let weight = WEIGHTS[index];
        let count = PLACEMENT_COUNTS[index].load(Ordering::Relaxed);
        has_finite |= weight.maximum > 0 && count < weight.maximum;
        total += weight.weight;
    }
    if has_finite { total } else { -1 }
}

fn piece_shape(kind: FortressPieceKind) -> Option<(BlockPos, [i32; 3])> {
    let shape = match kind {
        FortressPieceKind::Start => return None,
        FortressPieceKind::BridgeStraight => (BlockPos::new(-1, -3, 0), [5, 10, 19]),
        FortressPieceKind::BridgeCrossing => (BlockPos::new(-8, -3, 0), [19, 10, 19]),
        FortressPieceKind::RoomCrossing => (BlockPos::new(-2, 0, 0), [7, 9, 7]),
        FortressPieceKind::StairsRoom => (BlockPos::new(-2, 0, 0), [7, 11, 7]),
        FortressPieceKind::MonsterThrone => (BlockPos::new(-2, 0, 0), [7, 8, 9]),
        FortressPieceKind::CastleEntrance | FortressPieceKind::CastleStalkRoom => {
            (BlockPos::new(-5, -3, 0), [13, 14, 13])
        }
        FortressPieceKind::CastleSmallCorridor
        | FortressPieceKind::CastleSmallCrossing
        | FortressPieceKind::CastleRightTurn
        | FortressPieceKind::CastleLeftTurn => (BlockPos::new(-1, 0, 0), [5, 7, 5]),
        FortressPieceKind::CastleCorridorStairs => (BlockPos::new(-1, -7, 0), [5, 14, 10]),
        FortressPieceKind::CastleTBalcony => (BlockPos::new(-3, 0, 0), [9, 7, 9]),
        FortressPieceKind::BridgeEndFiller => (BlockPos::new(-1, -3, 0), [5, 10, 8]),
    };
    Some(shape)
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("positive fortress bound"))
}
