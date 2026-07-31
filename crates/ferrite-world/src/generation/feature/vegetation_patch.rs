//! Ordinary and waterlogged vegetation patches with Java-HashSet iteration order.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::java_hash_set::JavaBlockPosSet;
use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveSurface {
    Floor,
    Ceiling,
}

impl CaveSurface {
    const fn direction(self) -> Direction {
        match self {
            Self::Floor => Direction::Down,
            Self::Ceiling => Direction::Up,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VegetationPatchConfig {
    pub surface: CaveSurface,
    pub depth: IntProvider,
    pub extra_bottom_block_chance: f32,
    pub vertical_range: u32,
    pub vegetation_chance: f32,
    pub xz_radius: IntProvider,
    pub extra_edge_column_chance: f32,
}

pub trait VegetationPatchWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_empty(&self, state: BlockStateId) -> bool;

    fn is_face_sturdy(&mut self, position: BlockPos, state: BlockStateId, face: Direction) -> bool;

    fn sample_ground<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> BlockStateId;

    fn same_block_type(&self, left: BlockStateId, right: BlockStateId) -> bool;

    fn is_ground_replaceable(&self, state: BlockStateId) -> bool;

    fn source_water(&self) -> BlockStateId;

    fn with_waterlogged_true(&self, state: BlockStateId) -> Option<BlockStateId>;

    fn offer_patch_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn place_nested_vegetation<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BonemealablePatchType {
    NeighborSpreader,
}

pub trait BonemealablePatchWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn resolve_bonemeal_patch(&mut self) -> bool;

    fn place_bonemeal_patch<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> bool;
}

pub fn is_valid_patch_bonemeal_target<W: BonemealablePatchWorld>(
    world: &mut W,
    position: BlockPos,
) -> Result<bool, VegetationPatchError> {
    let above = offset(position, Direction::Up)?;
    let state = world.block_state(above);
    Ok(world.is_air(state))
}

#[must_use]
pub const fn is_patch_bonemeal_success() -> bool {
    true
}

#[must_use]
pub const fn patch_bonemeal_type() -> BonemealablePatchType {
    BonemealablePatchType::NeighborSpreader
}

pub fn perform_patch_bonemeal<R, W>(
    world: &mut W,
    position: BlockPos,
    random: &mut R,
) -> Result<(), VegetationPatchError>
where
    R: GenerationRandom,
    W: BonemealablePatchWorld,
{
    if world.resolve_bonemeal_patch() {
        let above = offset(position, Direction::Up)?;
        let _ = world.place_bonemeal_patch(above, random);
    }
    Ok(())
}

pub fn place_vegetation_patch<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &VegetationPatchConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, VegetationPatchError>
where
    R: GenerationRandom,
    W: VegetationPatchWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    validate_config(config)?;
    let ground = place_ground_patch(world, origin, config, random)?;
    distribute_vegetation(world, &ground, config, random, false)?;
    Ok(!ground.is_empty())
}

pub fn place_waterlogged_vegetation_patch<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &VegetationPatchConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, VegetationPatchError>
where
    R: GenerationRandom,
    W: VegetationPatchWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    validate_config(config)?;
    let mut ground = place_ground_patch(world, origin, config, random)?;
    ground.retain(|position| enclosed_for_water(world, position));
    for position in ground.iter() {
        let _ = world.offer_patch_block(position, world.source_water(), 2);
    }
    distribute_vegetation(world, &ground, config, random, true)?;
    Ok(!ground.is_empty())
}

fn place_ground_patch<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &VegetationPatchConfig,
    random: &mut R,
) -> Result<JavaBlockPosSet, VegetationPatchError>
where
    R: GenerationRandom,
    W: VegetationPatchWorld,
{
    let radius_x = config
        .xz_radius
        .sample(random)?
        .checked_add(1)
        .ok_or(VegetationPatchError::RadiusOverflow)?;
    let radius_z = config
        .xz_radius
        .sample(random)?
        .checked_add(1)
        .ok_or(VegetationPatchError::RadiusOverflow)?;
    let mut successful = JavaBlockPosSet::new();
    if radius_x < 0 || radius_z < 0 {
        return Ok(successful);
    }
    for x in -radius_x..=radius_x {
        for z in -radius_z..=radius_z {
            let x_edge = x.abs() == radius_x;
            let z_edge = z.abs() == radius_z;
            if x_edge && z_edge {
                continue;
            }
            if (x_edge || z_edge)
                && (config.extra_edge_column_chance == 0.0
                    || random.next_f32() > config.extra_edge_column_chance)
            {
                continue;
            }
            let candidate = offset_xyz(origin, x, 0, z)?;
            if let Some(ground) = place_ground_column(world, candidate, config, random)? {
                successful.insert(ground);
            }
        }
    }
    Ok(successful)
}

fn place_ground_column<R, W>(
    world: &mut W,
    start: BlockPos,
    config: &VegetationPatchConfig,
    random: &mut R,
) -> Result<Option<BlockPos>, VegetationPatchError>
where
    R: GenerationRandom,
    W: VegetationPatchWorld,
{
    let surface_direction = config.surface.direction();
    let opposite = surface_direction.opposite();
    let mut cursor = start;
    for _ in 0..config.vertical_range {
        let state = world.block_state(cursor);
        if !world.is_air(state) {
            break;
        }
        cursor = offset(cursor, surface_direction)?;
    }
    for _ in 0..config.vertical_range {
        let state = world.block_state(cursor);
        if world.is_air(state) {
            break;
        }
        cursor = offset(cursor, opposite)?;
    }

    let ground = offset(cursor, surface_direction)?;
    let ground_state = world.block_state(ground);
    let cursor_state = world.block_state(cursor);
    if !world.is_empty(cursor_state) || !world.is_face_sturdy(ground, ground_state, opposite) {
        return Ok(None);
    }
    let mut depth = config.depth.sample(random)?;
    if !(1..=128).contains(&depth) {
        return Err(VegetationPatchError::InvalidDepth(depth));
    }
    if config.extra_bottom_block_chance > 0.0
        && random.next_f32() < config.extra_bottom_block_chance
    {
        depth = depth
            .checked_add(1)
            .ok_or(VegetationPatchError::InvalidDepth(depth))?;
    }
    if place_ground_layers(world, ground, surface_direction, depth, random)? {
        Ok(Some(ground))
    } else {
        Ok(None)
    }
}

fn place_ground_layers<R, W>(
    world: &mut W,
    origin: BlockPos,
    direction: Direction,
    depth: i32,
    random: &mut R,
) -> Result<bool, VegetationPatchError>
where
    R: GenerationRandom,
    W: VegetationPatchWorld,
{
    let mut cursor = origin;
    for index in 0..depth {
        let sampled = world.sample_ground(cursor, random);
        let existing = world.block_state(cursor);
        if world.same_block_type(sampled, existing) {
            continue;
        }
        if !world.is_ground_replaceable(existing) {
            return Ok(index != 0);
        }
        let _ = world.offer_patch_block(cursor, sampled, 2);
        cursor = offset(cursor, direction)?;
    }
    Ok(true)
}

fn distribute_vegetation<R, W>(
    world: &mut W,
    ground: &JavaBlockPosSet,
    config: &VegetationPatchConfig,
    random: &mut R,
    waterlogged: bool,
) -> Result<(), VegetationPatchError>
where
    R: GenerationRandom,
    W: VegetationPatchWorld,
{
    if config.vegetation_chance == 0.0 {
        return Ok(());
    }
    for ground_position in ground.iter() {
        if random.next_f32() >= config.vegetation_chance {
            continue;
        }
        let base = if waterlogged {
            offset(ground_position, Direction::Down)?
        } else {
            ground_position
        };
        let child_position = offset(base, config.surface.direction().opposite())?;
        let placed = world.place_nested_vegetation(child_position, random);
        if waterlogged && placed {
            let state = world.block_state(ground_position);
            if let Some(waterlogged_state) = world.with_waterlogged_true(state) {
                let _ = world.offer_patch_block(ground_position, waterlogged_state, 2);
            }
        }
    }
    Ok(())
}

fn enclosed_for_water<W: VegetationPatchWorld>(world: &mut W, position: BlockPos) -> bool {
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
        Direction::Down,
    ] {
        let Ok(neighbor) = offset(position, direction) else {
            return false;
        };
        let state = world.block_state(neighbor);
        if !world.is_face_sturdy(neighbor, state, direction.opposite()) {
            return false;
        }
    }
    true
}

fn validate_config(config: &VegetationPatchConfig) -> Result<(), VegetationPatchError> {
    if !(1..=256).contains(&config.vertical_range) {
        return Err(VegetationPatchError::InvalidVerticalRange(
            config.vertical_range,
        ));
    }
    for chance in [
        config.extra_bottom_block_chance,
        config.vegetation_chance,
        config.extra_edge_column_chance,
    ] {
        if !chance.is_finite() || !(0.0..=1.0).contains(&chance) {
            return Err(VegetationPatchError::InvalidChance);
        }
    }
    Ok(())
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, VegetationPatchError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x, y, z)
}

fn offset_xyz(
    position: BlockPos,
    x: i32,
    y: i32,
    z: i32,
) -> Result<BlockPos, VegetationPatchError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(VegetationPatchError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(VegetationPatchError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(VegetationPatchError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VegetationPatchError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("vegetation-patch radius overflow")]
    RadiusOverflow,
    #[error("vegetation-patch sampled invalid depth {0}")]
    InvalidDepth(i32),
    #[error("vegetation-patch vertical range must be in 1..=256, got {0}")]
    InvalidVerticalRange(u32),
    #[error("vegetation-patch chances must be finite and in 0..=1")]
    InvalidChance,
    #[error("vegetation-patch position overflow")]
    PositionOverflow,
}
