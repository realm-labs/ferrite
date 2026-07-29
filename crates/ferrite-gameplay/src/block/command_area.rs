//! Deterministic preflight and traversal for clone, fill, and fillbiome.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

pub const DEFAULT_MAX_BLOCK_MODIFICATIONS: i32 = 32_768;
pub const CLONE_FLAGS: u16 = 2;
pub const FILL_FLAGS: u16 = 258;
pub const STRICT_FLAGS: u16 = 818;
pub const MOVE_CLEAR_FLAGS: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InclusiveBox {
    pub minimum: BlockPos,
    pub maximum: BlockPos,
}

impl InclusiveBox {
    pub const fn from_corners(first: BlockPos, second: BlockPos) -> Self {
        Self {
            minimum: BlockPos::new(
                min_i32(first.x, second.x),
                min_i32(first.y, second.y),
                min_i32(first.z, second.z),
            ),
            maximum: BlockPos::new(
                max_i32(first.x, second.x),
                max_i32(first.y, second.y),
                max_i32(first.z, second.z),
            ),
        }
    }

    pub fn volume(self) -> Result<i64, AreaCommandError> {
        let x = i64::from(self.maximum.x) - i64::from(self.minimum.x) + 1;
        let y = i64::from(self.maximum.y) - i64::from(self.minimum.y) + 1;
        let z = i64::from(self.maximum.z) - i64::from(self.minimum.z) + 1;
        x.checked_mul(y)
            .and_then(|area| area.checked_mul(z))
            .ok_or(AreaCommandError::VolumeOverflow)
    }

    pub fn validate_limit(self, maximum: i32) -> Result<i64, AreaCommandError> {
        if maximum < 1 {
            return Err(AreaCommandError::InvalidMaximum(maximum));
        }
        let volume = self.volume()?;
        if volume > i64::from(maximum) {
            Err(AreaCommandError::TooLarge { maximum, volume })
        } else {
            Ok(volume)
        }
    }

    pub fn visit_x_y_z<F>(self, mut visit: F) -> Result<(), AreaCommandError>
    where
        F: FnMut(BlockPos),
    {
        let mut z = self.minimum.z;
        loop {
            let mut y = self.minimum.y;
            loop {
                let mut x = self.minimum.x;
                loop {
                    visit(BlockPos::new(x, y, z));
                    if x == self.maximum.x {
                        break;
                    }
                    x = x
                        .checked_add(1)
                        .ok_or(AreaCommandError::CoordinateOverflow)?;
                }
                if y == self.maximum.y {
                    break;
                }
                y = y
                    .checked_add(1)
                    .ok_or(AreaCommandError::CoordinateOverflow)?;
            }
            if z == self.maximum.z {
                break;
            }
            z = z
                .checked_add(1)
                .ok_or(AreaCommandError::CoordinateOverflow)?;
        }
        Ok(())
    }

    pub const fn intersects(self, other: Self) -> bool {
        self.minimum.x <= other.maximum.x
            && self.maximum.x >= other.minimum.x
            && self.minimum.y <= other.maximum.y
            && self.maximum.y >= other.minimum.y
            && self.minimum.z <= other.maximum.z
            && self.maximum.z >= other.minimum.z
    }

    pub const fn is_boundary(self, position: BlockPos) -> bool {
        position.x == self.minimum.x
            || position.x == self.maximum.x
            || position.y == self.minimum.y
            || position.y == self.maximum.y
            || position.z == self.minimum.z
            || position.z == self.maximum.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneMode {
    Normal,
    Force,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClonePreflight {
    pub source: InclusiveBox,
    pub destination: InclusiveBox,
    pub same_level: bool,
    pub mode: CloneMode,
    pub maximum: i32,
    pub source_loaded: bool,
    pub destination_loaded: bool,
    pub destination_debug: bool,
}

pub fn validate_clone_preflight(preflight: ClonePreflight) -> Result<i64, AreaCommandError> {
    if preflight.same_level
        && preflight.mode == CloneMode::Normal
        && preflight.source.intersects(preflight.destination)
    {
        return Err(AreaCommandError::CloneOverlap);
    }
    let volume = preflight.source.validate_limit(preflight.maximum)?;
    if !preflight.source_loaded || !preflight.destination_loaded {
        return Err(AreaCommandError::Unloaded);
    }
    if preflight.destination_debug {
        return Err(AreaCommandError::DebugLevel);
    }
    Ok(volume)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneBlockCategory {
    BlockEntity,
    Solid,
    NonFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloneEntry {
    pub source: BlockPos,
    pub destination: BlockPos,
    pub category: CloneBlockCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneWriteKind {
    SourceBarrier,
    SourceAir,
    DestinationBarrier,
    DestinationState,
    DestinationBlockEntityState,
    ExplicitNeighborUpdate,
    CopyScheduledTicks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloneWrite {
    pub position: BlockPos,
    pub kind: CloneWriteKind,
    pub flags: u16,
}

pub fn plan_clone(entries: &[CloneEntry], mode: CloneMode, strict: bool) -> Vec<CloneWrite> {
    let mut ordered: Vec<&CloneEntry> = Vec::with_capacity(entries.len());
    ordered.extend(
        entries
            .iter()
            .filter(|entry| entry.category == CloneBlockCategory::Solid),
    );
    ordered.extend(
        entries
            .iter()
            .filter(|entry| entry.category == CloneBlockCategory::BlockEntity),
    );
    ordered.extend(
        entries
            .iter()
            .filter(|entry| entry.category == CloneBlockCategory::NonFull),
    );

    let base_flags = if strict { STRICT_FLAGS } else { CLONE_FLAGS };
    let mut writes = Vec::new();
    if mode == CloneMode::Move {
        let clear_order = source_clear_order(entries);
        for entry in &clear_order {
            writes.push(CloneWrite {
                position: entry.source,
                kind: CloneWriteKind::SourceBarrier,
                flags: STRICT_FLAGS,
            });
        }
        for entry in clear_order {
            writes.push(CloneWrite {
                position: entry.source,
                kind: CloneWriteKind::SourceAir,
                flags: if strict {
                    STRICT_FLAGS
                } else {
                    MOVE_CLEAR_FLAGS
                },
            });
        }
    }
    for entry in ordered.iter().rev() {
        writes.push(CloneWrite {
            position: entry.destination,
            kind: CloneWriteKind::DestinationBarrier,
            flags: STRICT_FLAGS,
        });
    }
    for entry in &ordered {
        writes.push(CloneWrite {
            position: entry.destination,
            kind: CloneWriteKind::DestinationState,
            flags: base_flags,
        });
    }
    for entry in ordered
        .iter()
        .filter(|entry| entry.category == CloneBlockCategory::BlockEntity)
    {
        writes.push(CloneWrite {
            position: entry.destination,
            kind: CloneWriteKind::DestinationBlockEntityState,
            flags: base_flags,
        });
    }
    if !strict {
        for entry in ordered.iter().rev() {
            writes.push(CloneWrite {
                position: entry.destination,
                kind: CloneWriteKind::ExplicitNeighborUpdate,
                flags: 0,
            });
        }
    }
    writes.push(CloneWrite {
        position: BlockPos::default(),
        kind: CloneWriteKind::CopyScheduledTicks,
        flags: 0,
    });
    writes
}

fn source_clear_order(entries: &[CloneEntry]) -> Vec<&CloneEntry> {
    let mut order = Vec::with_capacity(entries.len());
    order.extend(
        entries
            .iter()
            .filter(|entry| entry.category == CloneBlockCategory::NonFull)
            .rev(),
    );
    order.extend(
        entries
            .iter()
            .filter(|entry| entry.category != CloneBlockCategory::NonFull),
    );
    order
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillMode {
    Replace,
    Outline,
    Hollow,
    Destroy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillDecision {
    pub destroy_first: bool,
    pub place_requested: bool,
    pub place_air: bool,
    pub flags: u16,
}

pub const fn fill_decision(
    area: InclusiveBox,
    position: BlockPos,
    mode: FillMode,
    strict: bool,
) -> FillDecision {
    let boundary = area.is_boundary(position);
    FillDecision {
        destroy_first: matches!(mode, FillMode::Destroy),
        place_requested: match mode {
            FillMode::Replace | FillMode::Destroy => true,
            FillMode::Outline | FillMode::Hollow => boundary,
        },
        place_air: matches!(mode, FillMode::Hollow) && !boundary,
        flags: if strict { STRICT_FLAGS } else { FILL_FLAGS },
    }
}

pub const fn fill_result_increment(destroyed: bool, placed: bool) -> u32 {
    if destroyed || placed { 1 } else { 0 }
}

pub const fn quantize_biome_coordinate(value: i32) -> i32 {
    value.div_euclid(4) * 4
}

pub const fn quantize_biome_box(first: BlockPos, second: BlockPos) -> InclusiveBox {
    InclusiveBox::from_corners(
        BlockPos::new(
            quantize_biome_coordinate(first.x),
            quantize_biome_coordinate(first.y),
            quantize_biome_coordinate(first.z),
        ),
        BlockPos::new(
            quantize_biome_coordinate(second.x),
            quantize_biome_coordinate(second.y),
            quantize_biome_coordinate(second.z),
        ),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillBiomeChunk {
    pub full_chunk_available: bool,
    pub matching_quart_cells: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillBiomeCommit {
    pub matching_quart_cells: u64,
    pub dirty_chunk_indices: Vec<usize>,
    pub resend_chunk_indices: Vec<usize>,
}

pub fn plan_fill_biome(
    chunks_in_z_major_order: &[FillBiomeChunk],
) -> Result<FillBiomeCommit, AreaCommandError> {
    if chunks_in_z_major_order
        .iter()
        .any(|chunk| !chunk.full_chunk_available)
    {
        return Err(AreaCommandError::Unloaded);
    }
    let matching_quart_cells = chunks_in_z_major_order
        .iter()
        .map(|chunk| u64::from(chunk.matching_quart_cells))
        .sum();
    let indices = (0..chunks_in_z_major_order.len()).collect::<Vec<_>>();
    Ok(FillBiomeCommit {
        matching_quart_cells,
        dirty_chunk_indices: indices.clone(),
        resend_chunk_indices: indices,
    })
}

const fn min_i32(first: i32, second: i32) -> i32 {
    if first < second { first } else { second }
}

const fn max_i32(first: i32, second: i32) -> i32 {
    if first > second { first } else { second }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AreaCommandError {
    #[error("max_block_modifications must be positive, got {0}")]
    InvalidMaximum(i32),
    #[error("inclusive area volume overflows signed 64-bit arithmetic")]
    VolumeOverflow,
    #[error("area volume {volume} exceeds max_block_modifications {maximum}")]
    TooLarge { maximum: i32, volume: i64 },
    #[error("inclusive traversal overflowed a block coordinate")]
    CoordinateOverflow,
    #[error("normal clone source and destination overlap in one level")]
    CloneOverlap,
    #[error("a required area chunk is not loaded")]
    Unloaded,
    #[error("destination level is a debug level")]
    DebugLevel,
}
