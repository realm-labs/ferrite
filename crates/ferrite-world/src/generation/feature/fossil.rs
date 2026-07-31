//! Fossil feature orchestration around paired structure templates.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FossilTemplateId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FossilProcessorId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FossilRotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FossilBoundingBox {
    pub minimum: BlockPos,
    pub maximum: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FossilClip {
    pub minimum: BlockPos,
    pub maximum: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FossilConfig {
    pub primary_templates: Vec<FossilTemplateId>,
    pub overlay_templates: Vec<FossilTemplateId>,
    pub primary_processors: Vec<FossilProcessorId>,
    pub overlay_processors: Vec<FossilProcessorId>,
    pub maximum_empty_corners: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FossilPlacementSettings<'a> {
    pub rotation: FossilRotation,
    pub clip: FossilClip,
    pub processors: &'a [FossilProcessorId],
}

pub trait FossilWorld {
    fn minimum_y(&self) -> i32;

    fn maximum_y(&self) -> i32;

    fn resolve_template(&mut self, identifier: FossilTemplateId) -> bool;

    fn rotated_template_size(
        &mut self,
        template: FossilTemplateId,
        rotation: FossilRotation,
    ) -> [i32; 3];

    fn ocean_floor_wg(&mut self, x: i32, z: i32) -> i32;

    fn transformed_zero_position(
        &mut self,
        template: FossilTemplateId,
        position: BlockPos,
        rotation: FossilRotation,
    ) -> BlockPos;

    fn template_bounding_box(
        &mut self,
        template: FossilTemplateId,
        zero_position: BlockPos,
        rotation: FossilRotation,
        clip: FossilClip,
    ) -> FossilBoundingBox;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_water_or_lava_block_identity(&self, state: BlockStateId) -> bool;

    fn place_fossil_template<R: GenerationRandom>(
        &mut self,
        template: FossilTemplateId,
        position: BlockPos,
        pivot: BlockPos,
        settings: FossilPlacementSettings<'_>,
        random: &mut R,
        flags: u32,
    ) -> bool;
}

pub fn place_fossil<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &FossilConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, FossilError>
where
    R: GenerationRandom,
    W: FossilWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    validate_config(config)?;
    let rotation =
        match random.next_u32(NonZeroU32::new(4).expect("fossil rotation bound is nonzero")) {
            0 => FossilRotation::None,
            1 => FossilRotation::Clockwise90,
            2 => FossilRotation::Clockwise180,
            3 => FossilRotation::Counterclockwise90,
            _ => unreachable!("bounded rotation draw"),
        };
    let template_bound = u32::try_from(config.primary_templates.len())
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(FossilError::InvalidTemplateLists)?;
    let index = random.next_u32(template_bound) as usize;
    let primary = config.primary_templates[index];
    let overlay = config.overlay_templates[index];
    if !world.resolve_template(primary) {
        return Err(FossilError::MissingTemplate(primary));
    }
    if !world.resolve_template(overlay) {
        return Err(FossilError::MissingTemplate(overlay));
    }

    let clip = chunk_clip(world, origin)?;
    let size = world.rotated_template_size(primary, rotation);
    if size[0] < 0 || size[2] < 0 {
        return Err(FossilError::InvalidTemplateSize);
    }
    let centered_x = origin
        .x
        .checked_add((-size[0]) / 2)
        .ok_or(FossilError::PositionOverflow)?;
    let centered_z = origin
        .z
        .checked_add((-size[2]) / 2)
        .ok_or(FossilError::PositionOverflow)?;
    let mut minimum_surface = origin.y;
    for x_offset in 0..size[0] {
        for z_offset in 0..size[2] {
            let x = centered_x
                .checked_add(x_offset)
                .ok_or(FossilError::PositionOverflow)?;
            let z = centered_z
                .checked_add(z_offset)
                .ok_or(FossilError::PositionOverflow)?;
            minimum_surface = minimum_surface.min(world.ocean_floor_wg(x, z));
        }
    }
    let burial =
        random.next_u32(NonZeroU32::new(10).expect("fossil burial bound is nonzero")) as i32;
    let unclamped_y = minimum_surface
        .checked_sub(15)
        .and_then(|value| value.checked_sub(burial))
        .ok_or(FossilError::PositionOverflow)?;
    let minimum_placement_y = world
        .minimum_y()
        .checked_add(10)
        .ok_or(FossilError::PositionOverflow)?;
    let placement = BlockPos::new(centered_x, unclamped_y.max(minimum_placement_y), centered_z);
    let zero = world.transformed_zero_position(primary, placement, rotation);
    let bounds = world.template_bounding_box(primary, zero, rotation, clip);
    let empty_corners = count_empty_corners(world, bounds);
    if empty_corners > u32::from(config.maximum_empty_corners) {
        return Ok(false);
    }

    let primary_settings = FossilPlacementSettings {
        rotation,
        clip,
        processors: &config.primary_processors,
    };
    let _ = world.place_fossil_template(primary, zero, zero, primary_settings, random, 260);
    let overlay_settings = FossilPlacementSettings {
        rotation,
        clip,
        processors: &config.overlay_processors,
    };
    let _ = world.place_fossil_template(overlay, zero, zero, overlay_settings, random, 260);
    Ok(true)
}

fn count_empty_corners<W: FossilWorld>(world: &mut W, bounds: FossilBoundingBox) -> u32 {
    let min = bounds.minimum;
    let max = bounds.maximum;
    let corners = [
        BlockPos::new(max.x, max.y, max.z),
        BlockPos::new(min.x, max.y, max.z),
        BlockPos::new(max.x, min.y, max.z),
        BlockPos::new(min.x, min.y, max.z),
        BlockPos::new(max.x, max.y, min.z),
        BlockPos::new(min.x, max.y, min.z),
        BlockPos::new(max.x, min.y, min.z),
        BlockPos::new(min.x, min.y, min.z),
    ];
    let mut empty = 0_u32;
    for corner in corners {
        let state = world.block_state(corner);
        if world.is_air(state) || world.is_water_or_lava_block_identity(state) {
            empty += 1;
        }
    }
    empty
}

fn chunk_clip<W: FossilWorld>(world: &W, origin: BlockPos) -> Result<FossilClip, FossilError> {
    let chunk_x = origin.x.div_euclid(16);
    let chunk_z = origin.z.div_euclid(16);
    let chunk_min_x = chunk_x
        .checked_mul(16)
        .ok_or(FossilError::PositionOverflow)?;
    let chunk_min_z = chunk_z
        .checked_mul(16)
        .ok_or(FossilError::PositionOverflow)?;
    Ok(FossilClip {
        minimum: BlockPos::new(
            chunk_min_x
                .checked_sub(16)
                .ok_or(FossilError::PositionOverflow)?,
            world.minimum_y(),
            chunk_min_z
                .checked_sub(16)
                .ok_or(FossilError::PositionOverflow)?,
        ),
        maximum: BlockPos::new(
            chunk_min_x
                .checked_add(31)
                .ok_or(FossilError::PositionOverflow)?,
            world.maximum_y(),
            chunk_min_z
                .checked_add(31)
                .ok_or(FossilError::PositionOverflow)?,
        ),
    })
}

fn validate_config(config: &FossilConfig) -> Result<(), FossilError> {
    if config.primary_templates.is_empty()
        || config.primary_templates.len() != config.overlay_templates.len()
    {
        return Err(FossilError::InvalidTemplateLists);
    }
    if config.maximum_empty_corners > 7 {
        return Err(FossilError::InvalidMaximumEmptyCorners(
            config.maximum_empty_corners,
        ));
    }
    Ok(())
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FossilError {
    #[error("fossil primary templates must be nonempty and pair one-to-one with overlays")]
    InvalidTemplateLists,
    #[error("fossil maximum empty corners must be in 0..=7, got {0}")]
    InvalidMaximumEmptyCorners(u8),
    #[error("fossil template {0:?} could not be resolved")]
    MissingTemplate(FossilTemplateId),
    #[error("fossil template has a negative horizontal size")]
    InvalidTemplateSize,
    #[error("fossil feature position overflow")]
    PositionOverflow,
}
