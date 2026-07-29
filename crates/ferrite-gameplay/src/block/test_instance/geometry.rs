//! Signed geometry and ordered structure-placement transactions.

use crate::block::test_instance::TEMPLATE_WRITE_FLAGS;
use crate::block::test_instance::data::{IntVector, QuarterRotation, TestInstanceData};
use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockBox {
    pub minimum: BlockPos,
    pub maximum: BlockPos,
}

impl BlockBox {
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

    pub const fn inflate(self, amount: i32) -> Self {
        Self {
            minimum: BlockPos::new(
                self.minimum.x.wrapping_sub(amount),
                self.minimum.y.wrapping_sub(amount),
                self.minimum.z.wrapping_sub(amount),
            ),
            maximum: BlockPos::new(
                self.maximum.x.wrapping_add(amount),
                self.maximum.y.wrapping_add(amount),
                self.maximum.z.wrapping_add(amount),
            ),
        }
    }
}

const fn min_i32(left: i32, right: i32) -> i32 {
    if left < right { left } else { right }
}

const fn max_i32(left: i32, right: i32) -> i32 {
    if left > right { left } else { right }
}

pub const fn effective_rotation(
    intrinsic: QuarterRotation,
    extra: QuarterRotation,
) -> QuarterRotation {
    intrinsic.compose(extra)
}

pub const fn transformed_size(size: IntVector, rotation: QuarterRotation) -> IntVector {
    match rotation {
        QuarterRotation::Clockwise90 | QuarterRotation::Counterclockwise90 => {
            IntVector::new(size.z, size.y, size.x)
        }
        QuarterRotation::None | QuarterRotation::Clockwise180 => size,
    }
}

pub const fn structure_position(block: BlockPos, padding: i32) -> BlockPos {
    BlockPos::new(
        block.x.wrapping_add(padding),
        block.y.wrapping_add(padding.wrapping_add(1)),
        block.z.wrapping_add(padding.wrapping_add(1)),
    )
}

pub const fn structure_box(
    block: BlockPos,
    stored_size: IntVector,
    rotation: QuarterRotation,
    padding: i32,
) -> BlockBox {
    let start = structure_position(block, padding);
    let size = transformed_size(stored_size, rotation);
    let end = BlockPos::new(
        start.x.wrapping_add(size.x.wrapping_sub(1)),
        start.y.wrapping_add(size.y.wrapping_sub(1)),
        start.z.wrapping_add(size.z.wrapping_sub(1)),
    );
    BlockBox::from_corners(start, end)
}

pub const fn test_box(
    block: BlockPos,
    stored_size: IntVector,
    rotation: QuarterRotation,
    padding: i32,
) -> BlockBox {
    structure_box(block, stored_size, rotation, padding).inflate(padding)
}

pub const fn placement_start_corner(
    structure_position: BlockPos,
    stored_size: IntVector,
    rotation: QuarterRotation,
) -> BlockPos {
    match rotation {
        QuarterRotation::None => structure_position,
        QuarterRotation::Clockwise90 => BlockPos::new(
            structure_position
                .x
                .wrapping_add(stored_size.z.wrapping_sub(1)),
            structure_position.y,
            structure_position.z,
        ),
        QuarterRotation::Clockwise180 => BlockPos::new(
            structure_position
                .x
                .wrapping_add(stored_size.x.wrapping_sub(1)),
            structure_position.y,
            structure_position
                .z
                .wrapping_add(stored_size.z.wrapping_sub(1)),
        ),
        QuarterRotation::Counterclockwise90 => BlockPos::new(
            structure_position.x,
            structure_position.y,
            structure_position
                .z
                .wrapping_add(stored_size.x.wrapping_sub(1)),
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRectangle {
    pub minimum: ChunkPos,
    pub maximum: ChunkPos,
}

impl ChunkRectangle {
    pub const fn from_block_box(area: BlockBox) -> Self {
        Self {
            minimum: area.minimum.chunk(),
            maximum: area.maximum.chunk(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryShell {
    pub structure: BlockBox,
    pub outside_distance: u8,
    pub include_four_walls: bool,
    pub include_floor: bool,
    pub include_ceiling: bool,
}

pub const fn boundary_shell(structure: BlockBox, sky_access: bool) -> BoundaryShell {
    BoundaryShell {
        structure,
        outside_distance: 1,
        include_four_walls: true,
        include_floor: true,
        include_ceiling: !sky_access,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementEffect {
    PermanentlyForceChunks(ChunkRectangle),
    ClearBlocksToAir {
        area: BlockBox,
        flags: u16,
        explicit_neighbor_update_per_cell: bool,
    },
    ClearScheduledBlockTicks(BlockBox),
    ClearBlockEvents(BlockBox),
    DiscardNonPlayerEntities(BlockBox),
    RepeatDiscardNonPlayerEntities(BlockBox),
    PlaceTemplate {
        template: ResourceId,
        origin: BlockPos,
        pivot: BlockPos,
        rotation: QuarterRotation,
        ignore_entities: bool,
        known_shape: bool,
        use_level_rng: bool,
        flags: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementPlan {
    pub structure_box: BlockBox,
    pub test_box: BlockBox,
    pub effects: Vec<PlacementEffect>,
}

pub fn plan_placement(
    block: BlockPos,
    data: &TestInstanceData,
    template: ResourceId,
    intrinsic_rotation: QuarterRotation,
    padding: i32,
) -> PlacementPlan {
    let rotation = effective_rotation(intrinsic_rotation, data.extra_rotation);
    let structure = structure_box(block, data.size, rotation, padding);
    let test = structure.inflate(padding);
    let start = placement_start_corner(structure_position(block, padding), data.size, rotation);
    PlacementPlan {
        structure_box: structure,
        test_box: test,
        effects: vec![
            PlacementEffect::PermanentlyForceChunks(ChunkRectangle::from_block_box(structure)),
            PlacementEffect::ClearBlocksToAir {
                area: test,
                flags: TEMPLATE_WRITE_FLAGS,
                explicit_neighbor_update_per_cell: true,
            },
            PlacementEffect::ClearScheduledBlockTicks(test),
            PlacementEffect::ClearBlockEvents(test),
            PlacementEffect::DiscardNonPlayerEntities(test),
            PlacementEffect::RepeatDiscardNonPlayerEntities(test),
            PlacementEffect::PlaceTemplate {
                template,
                origin: start,
                pivot: start,
                rotation,
                ignore_entities: data.ignore_entities,
                known_shape: true,
                use_level_rng: true,
                flags: TEMPLATE_WRITE_FLAGS,
            },
        ],
    }
}

pub const fn render_geometry_admitted(size: IntVector) -> bool {
    size.x >= 1 && size.y >= 1 && size.z >= 1
}
