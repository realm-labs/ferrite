//! Terrain-oriented configured features with bounded deterministic traversal.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

pub const BLOCK_PILE_WRITE_FLAGS: u32 = 260;

pub trait BlockPileWorld<R: GenerationRandom> {
    fn minimum_y(&self) -> i32;

    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_dirt_path(&self, state: BlockStateId) -> bool;

    fn is_sturdy_up(&self, state: BlockStateId) -> bool;

    fn provide_pile_state(&mut self, position: BlockPos, random: &mut R) -> BlockStateId;

    fn offer_pile_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_block_pile<R, W>(
    world: &mut W,
    origin: BlockPos,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TerrainFeatureError>
where
    R: GenerationRandom,
    W: BlockPileWorld<R>,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let minimum_y = world
        .minimum_y()
        .checked_add(5)
        .ok_or(TerrainFeatureError::PositionOverflow)?;
    if origin.y < minimum_y {
        return Ok(false);
    }
    let two = NonZeroU32::new(2).expect("two is nonzero");
    let x_radius = random.next_u32(two) as i32 + 2;
    let z_radius = random.next_u32(two) as i32 + 2;
    for z_offset in -z_radius..=z_radius {
        for y_offset in 0..=1 {
            for x_offset in -x_radius..=x_radius {
                let distance = x_offset * x_offset + z_offset * z_offset;
                let primary_threshold = random.next_f32() * 10.0 - random.next_f32() * 6.0;
                let admitted = distance as f32 <= primary_threshold || random.next_f32() < 0.031;
                if !admitted {
                    continue;
                }
                let candidate = offset(origin, x_offset, y_offset, z_offset)?;
                if !world.is_empty_block(candidate) {
                    continue;
                }
                let below = offset(candidate, 0, -1, 0)?;
                let support = world.block_state(below);
                let supported = if world.is_dirt_path(support) {
                    random.next_bool()
                } else {
                    world.is_sturdy_up(support)
                };
                if !supported {
                    continue;
                }
                let state = world.provide_pile_state(candidate, random);
                let _ = world.offer_pile_block(candidate, state, BLOCK_PILE_WRITE_FLAGS);
            }
        }
    }
    Ok(true)
}

pub trait DiskWorld<R: GenerationRandom> {
    fn test_disk_target(&mut self, position: BlockPos, random: &mut R) -> bool;

    fn provide_disk_state(&mut self, position: BlockPos, random: &mut R) -> Option<BlockStateId>;

    fn offer_disk_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn mark_for_postprocessing(&mut self, position: BlockPos);
}

pub fn place_disk<R, W>(
    world: &mut W,
    origin: BlockPos,
    radius: &IntProvider,
    half_height: u8,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TerrainFeatureError>
where
    R: GenerationRandom,
    W: DiskWorld<R>,
{
    if half_height > 4 {
        return Err(TerrainFeatureError::DiskHalfHeightOutOfRange { half_height });
    }
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let radius = radius.sample(random)?;
    if !(0..=8).contains(&radius) {
        return Err(TerrainFeatureError::DiskRadiusOutOfRange { radius });
    }
    let top = origin
        .y
        .checked_add(i32::from(half_height))
        .ok_or(TerrainFeatureError::PositionOverflow)?;
    let bottom = origin
        .y
        .checked_sub(i32::from(half_height))
        .ok_or(TerrainFeatureError::PositionOverflow)?;
    let radius_squared = radius * radius;
    let mut any_offered = false;
    for z_offset in -radius..=radius {
        for x_offset in -radius..=radius {
            if x_offset * x_offset + z_offset * z_offset > radius_squared {
                continue;
            }
            let x = origin
                .x
                .checked_add(x_offset)
                .ok_or(TerrainFeatureError::PositionOverflow)?;
            let z = origin
                .z
                .checked_add(z_offset)
                .ok_or(TerrainFeatureError::PositionOverflow)?;
            let mut active_target_run = false;
            for y in (bottom..=top).rev() {
                let position = BlockPos::new(x, y, z);
                if !world.test_disk_target(position, random) {
                    active_target_run = false;
                    continue;
                }
                let Some(state) = world.provide_disk_state(position, random) else {
                    continue;
                };
                let _ = world.offer_disk_block(position, state, 2);
                any_offered = true;
                if !active_target_run {
                    mark_above_for_postprocessing(world, position)?;
                    active_target_run = true;
                }
            }
        }
    }
    Ok(any_offered)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalDirection {
    North,
    South,
    West,
    East,
}

impl HorizontalDirection {
    const ORDERED: [Self; 4] = [Self::North, Self::South, Self::West, Self::East];

    const fn offset(self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::South => (0, 1),
            Self::West => (-1, 0),
            Self::East => (1, 0),
        }
    }
}

pub trait BasaltPillarWorld {
    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn is_outside_build_height(&self, position: BlockPos) -> bool;

    fn offer_basalt(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_basalt_pillar<R, W>(
    world: &mut W,
    origin: BlockPos,
    basalt: BlockStateId,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TerrainFeatureError>
where
    R: GenerationRandom,
    W: BasaltPillarWorld,
{
    if !ensure_can_write(origin) || !world.is_empty_block(origin) {
        return Ok(false);
    }
    let above = offset(origin, 0, 1, 0)?;
    if world.is_empty_block(above) {
        return Ok(false);
    }

    let ten = NonZeroU32::new(10).expect("ten is nonzero");
    let mut side_enabled = [true; 4];
    let mut cursor = origin;
    while world.is_empty_block(cursor) {
        if world.is_outside_build_height(cursor) {
            return Ok(true);
        }
        let _ = world.offer_basalt(cursor, basalt, 2);
        for (index, direction) in HorizontalDirection::ORDERED.into_iter().enumerate() {
            if !side_enabled[index] {
                continue;
            }
            if random.next_u32(ten) == 0 {
                side_enabled[index] = false;
                continue;
            }
            let (x, z) = direction.offset();
            let side = offset(cursor, x, 0, z)?;
            let _ = world.offer_basalt(side, basalt, 2);
        }
        cursor = offset(cursor, 0, -1, 0)?;
    }

    let bottom_pillar = offset(cursor, 0, 1, 0)?;
    for direction in HorizontalDirection::ORDERED {
        if random.next_bool() {
            let (x, z) = direction.offset();
            let side = offset(bottom_pillar, x, 0, z)?;
            let _ = world.offer_basalt(side, basalt, 2);
        }
    }
    let support_y = cursor.y;
    for x_offset in -3_i32..=3 {
        for z_offset in -3_i32..=3 {
            let threshold = 10 - x_offset.abs() * z_offset.abs();
            if random.next_u32(ten) as i32 >= threshold {
                continue;
            }
            let x = cursor
                .x
                .checked_add(x_offset)
                .ok_or(TerrainFeatureError::PositionOverflow)?;
            let z = cursor
                .z
                .checked_add(z_offset)
                .ok_or(TerrainFeatureError::PositionOverflow)?;
            let mut root = BlockPos::new(x, support_y, z);
            for _ in 0..3 {
                let below = offset(root, 0, -1, 0)?;
                if !world.is_empty_block(below) {
                    break;
                }
                root = below;
            }
            let below = offset(root, 0, -1, 0)?;
            if !world.is_empty_block(below) {
                let _ = world.offer_basalt(root, basalt, 2);
            }
        }
    }
    Ok(true)
}

pub trait ReplacementBlobWorld {
    fn minimum_y(&self) -> i32;

    fn maximum_y(&self) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn same_block_identity(&self, state: BlockStateId, target: BlockStateId) -> bool;

    fn offer_replacement(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_replacement_blob<R, W>(
    world: &mut W,
    origin: BlockPos,
    target: BlockStateId,
    replacement: BlockStateId,
    radius: &IntProvider,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TerrainFeatureError>
where
    R: GenerationRandom,
    W: ReplacementBlobWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let lowest_search_y = world
        .minimum_y()
        .checked_add(1)
        .ok_or(TerrainFeatureError::PositionOverflow)?;
    if lowest_search_y > world.maximum_y() {
        return Ok(false);
    }
    let mut cursor = BlockPos::new(
        origin.x,
        origin.y.clamp(lowest_search_y, world.maximum_y()),
        origin.z,
    );
    let center = loop {
        if cursor.y <= lowest_search_y {
            return Ok(false);
        }
        let state = world.block_state(cursor);
        if world.same_block_identity(state, target) {
            break cursor;
        }
        cursor = offset(cursor, 0, -1, 0)?;
    };

    let x_radius = sample_replacement_radius(radius, random)?;
    let y_radius = sample_replacement_radius(radius, random)?;
    let z_radius = sample_replacement_radius(radius, random)?;
    let maximum_radius = x_radius.max(y_radius).max(z_radius);
    let mut any_offered = false;
    for distance in 0..=maximum_radius {
        for x_offset in -x_radius..=x_radius {
            for y_offset in -y_radius..=y_radius {
                let z_offset =
                    distance - x_offset.unsigned_abs() as i32 - y_offset.unsigned_abs() as i32;
                if z_offset < 0 || z_offset > z_radius {
                    continue;
                }
                for z_offset in [Some(z_offset), (z_offset != 0).then_some(-z_offset)]
                    .into_iter()
                    .flatten()
                {
                    let position = offset(center, x_offset, y_offset, z_offset)?;
                    let state = world.block_state(position);
                    if world.same_block_identity(state, target) {
                        let _ = world.offer_replacement(position, replacement, 3);
                        any_offered = true;
                    }
                }
            }
        }
    }
    Ok(any_offered)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockFace {
    Up,
    North,
    East,
    South,
    West,
}

pub trait UnderwaterMagmaWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_exact_water(&self, state: BlockStateId) -> bool;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn has_full_face(&self, state: BlockStateId, position: BlockPos, face: BlockFace) -> bool;

    fn offer_magma(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnderwaterMagmaConfig {
    pub floor_search_range: u32,
    pub placement_radius: u32,
    pub placement_probability: f32,
    pub magma: BlockStateId,
}

pub fn place_underwater_magma<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: UnderwaterMagmaConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TerrainFeatureError>
where
    R: GenerationRandom,
    W: UnderwaterMagmaWorld,
{
    if config.floor_search_range > 512 {
        return Err(TerrainFeatureError::FloorSearchRangeOutOfRange {
            range: config.floor_search_range,
        });
    }
    if config.placement_radius > 64 {
        return Err(TerrainFeatureError::MagmaRadiusOutOfRange {
            radius: config.placement_radius,
        });
    }
    if !config.placement_probability.is_finite()
        || !(0.0..=1.0).contains(&config.placement_probability)
    {
        return Err(TerrainFeatureError::InvalidProbability);
    }
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let origin_state = world.block_state(origin);
    if !world.is_exact_water(origin_state) {
        return Ok(false);
    }
    let Some(_) = scan_water_edge(world, origin, 1, config.floor_search_range)? else {
        return Ok(false);
    };
    let Some(floor_y) = scan_water_edge(world, origin, -1, config.floor_search_range)? else {
        return Ok(false);
    };
    let radius = config.placement_radius as i32;
    let mut offers = 0_u32;
    for z_offset in -radius..=radius {
        for y_offset in -radius..=radius {
            for x_offset in -radius..=radius {
                if random.next_f32() >= config.placement_probability {
                    continue;
                }
                let candidate = offset(
                    BlockPos::new(origin.x, floor_y, origin.z),
                    x_offset,
                    y_offset,
                    z_offset,
                )?;
                if !valid_magma_position(world, candidate)? {
                    continue;
                }
                let _ = world.offer_magma(candidate, config.magma, 2);
                offers += 1;
            }
        }
    }
    Ok(offers != 0)
}

fn scan_water_edge<W: UnderwaterMagmaWorld>(
    world: &mut W,
    origin: BlockPos,
    y_step: i32,
    range: u32,
) -> Result<Option<i32>, TerrainFeatureError> {
    let mut cursor = origin;
    let mut distance = 1_u32;
    while distance < range {
        let state = world.block_state(cursor);
        if !world.is_exact_water(state) {
            break;
        }
        cursor = offset(cursor, 0, y_step, 0)?;
        distance += 1;
    }
    let state = world.block_state(cursor);
    Ok((!world.is_exact_water(state)).then_some(cursor.y))
}

fn valid_magma_position<W: UnderwaterMagmaWorld>(
    world: &mut W,
    candidate: BlockPos,
) -> Result<bool, TerrainFeatureError> {
    let state = world.block_state(candidate);
    if world.is_exact_water(state) || world.is_air(state) {
        return Ok(false);
    }
    let below = offset(candidate, 0, -1, 0)?;
    let below_state = world.block_state(below);
    if !world.has_full_face(below_state, below, BlockFace::Up) {
        return Ok(false);
    }
    for (x, z, face) in [
        (0, -1, BlockFace::South),
        (1, 0, BlockFace::West),
        (0, 1, BlockFace::North),
        (-1, 0, BlockFace::East),
    ] {
        let neighbor = offset(candidate, x, 0, z)?;
        let neighbor_state = world.block_state(neighbor);
        if !world.has_full_face(neighbor_state, neighbor, face) {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaFeatureConfig {
    pub contents: BlockStateId,
    pub rim: BlockStateId,
    pub size: IntProvider,
    pub rim_size: IntProvider,
}

pub trait DeltaFeatureWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn same_block_identity(&self, state: BlockStateId, target: BlockStateId) -> bool;

    fn is_protected_delta_block(&self, state: BlockStateId) -> bool;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn offer_delta_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_delta_feature<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &DeltaFeatureConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TerrainFeatureError>
where
    R: GenerationRandom,
    W: DeltaFeatureWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let sample_rim = random.next_f64() < 0.9;
    let (rim_x, rim_z) = if sample_rim {
        (
            sample_delta_reach(&config.rim_size, random)?,
            sample_delta_reach(&config.rim_size, random)?,
        )
    } else {
        (0, 0)
    };
    let rim_active = sample_rim && rim_x != 0 && rim_z != 0;
    let size_x = sample_delta_reach(&config.size, random)?;
    let size_z = sample_delta_reach(&config.size, random)?;
    let maximum_distance = size_x.max(size_z);
    let mut any_offered = false;
    for distance in 0..=maximum_distance {
        for x_offset in -distance..=distance {
            if x_offset.unsigned_abs() as i32 > size_x {
                continue;
            }
            let z_offset = distance - x_offset.unsigned_abs() as i32;
            if z_offset > size_z {
                continue;
            }
            for z_offset in [Some(z_offset), (z_offset != 0).then_some(-z_offset)]
                .into_iter()
                .flatten()
            {
                let rim_position = offset(origin, x_offset, 0, z_offset)?;
                if !delta_position_is_clear(world, rim_position, config.contents)? {
                    continue;
                }
                if rim_active {
                    let _ = world.offer_delta_block(rim_position, config.rim, 3);
                    any_offered = true;
                }
                let contents_position = offset(rim_position, rim_x, 0, rim_z)?;
                if delta_position_is_clear(world, contents_position, config.contents)? {
                    let _ = world.offer_delta_block(contents_position, config.contents, 3);
                    any_offered = true;
                }
            }
        }
    }
    Ok(any_offered)
}

fn sample_delta_reach(
    provider: &IntProvider,
    random: &mut impl GenerationRandom,
) -> Result<i32, TerrainFeatureError> {
    let reach = provider.sample(random)?;
    if (0..=16).contains(&reach) {
        Ok(reach)
    } else {
        Err(TerrainFeatureError::DeltaReachOutOfRange { reach })
    }
}

fn delta_position_is_clear<W: DeltaFeatureWorld>(
    world: &mut W,
    position: BlockPos,
    contents: BlockStateId,
) -> Result<bool, TerrainFeatureError> {
    let state = world.block_state(position);
    if world.same_block_identity(state, contents) || world.is_protected_delta_block(state) {
        return Ok(false);
    }
    for (x, y, z, must_be_air) in [
        (0, -1, 0, false),
        (0, 1, 0, true),
        (0, 0, -1, false),
        (0, 0, 1, false),
        (-1, 0, 0, false),
        (1, 0, 0, false),
    ] {
        let neighbor = offset(position, x, y, z)?;
        let neighbor_state = world.block_state(neighbor);
        let neighbor_is_air = world.is_air(neighbor_state);
        if neighbor_is_air != must_be_air {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesertWellBlocks {
    pub sandstone: BlockStateId,
    pub water: BlockStateId,
    pub sand: BlockStateId,
    pub sandstone_slab: BlockStateId,
    pub suspicious_sand: BlockStateId,
}

pub trait DesertWellWorld {
    fn minimum_y(&self) -> i32;

    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_sand_block(&self, state: BlockStateId) -> bool;

    fn offer_well_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn assign_desert_well_loot(&mut self, position: BlockPos, seed: i64) -> bool;
}

pub fn place_desert_well<R, W>(
    world: &mut W,
    origin: BlockPos,
    blocks: DesertWellBlocks,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TerrainFeatureError>
where
    R: GenerationRandom,
    W: DesertWellWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let mut center = offset(origin, 0, 1, 0)?;
    while world.is_empty_block(center)
        && center.y
            > world
                .minimum_y()
                .checked_add(2)
                .ok_or(TerrainFeatureError::PositionOverflow)?
    {
        center = offset(center, 0, -1, 0)?;
    }
    let center_state = world.block_state(center);
    if !world.is_sand_block(center_state) {
        return Ok(false);
    }
    for x_offset in -2..=2 {
        for z_offset in -2..=2 {
            let below = offset(center, x_offset, -1, z_offset)?;
            if world.is_empty_block(below)
                && world.is_empty_block(offset(center, x_offset, -2, z_offset)?)
            {
                return Ok(false);
            }
        }
    }

    for y_offset in -2..=0 {
        for x_offset in -2..=2 {
            for z_offset in -2..=2 {
                offer_well(
                    world,
                    offset(center, x_offset, y_offset, z_offset)?,
                    blocks.sandstone,
                    2,
                );
            }
        }
    }
    offer_center_and_cardinals(world, center, 0, blocks.water)?;
    offer_center_and_cardinals(world, center, -1, blocks.sand)?;

    for x_offset in -2_i32..=2 {
        for z_offset in -2_i32..=2 {
            if x_offset.abs() == 2 || z_offset.abs() == 2 {
                offer_well(
                    world,
                    offset(center, x_offset, 1, z_offset)?,
                    blocks.sandstone,
                    2,
                );
            }
        }
    }
    for (x_offset, z_offset) in [(2, 0), (-2, 0), (0, 2), (0, -2)] {
        offer_well(
            world,
            offset(center, x_offset, 1, z_offset)?,
            blocks.sandstone_slab,
            2,
        );
    }
    for x_offset in -1_i32..=1 {
        for z_offset in -1_i32..=1 {
            let state = if x_offset == 0 && z_offset == 0 {
                blocks.sandstone
            } else {
                blocks.sandstone_slab
            };
            offer_well(world, offset(center, x_offset, 4, z_offset)?, state, 2);
        }
    }
    for y_offset in 1..=3 {
        for (x_offset, z_offset) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
            offer_well(
                world,
                offset(center, x_offset, y_offset, z_offset)?,
                blocks.sandstone,
                2,
            );
        }
    }

    let archaeology_offsets = [(0, 0), (1, 0), (0, 1), (-1, 0), (0, -1)];
    for depth in [1, 2] {
        let index = random.next_u32(
            NonZeroU32::new(archaeology_offsets.len() as u32)
                .expect("archaeology choices are nonempty"),
        ) as usize;
        let (x_offset, z_offset) = archaeology_offsets[index];
        let position = offset(center, x_offset, -depth, z_offset)?;
        offer_well(world, position, blocks.suspicious_sand, 3);
        let _ = world.assign_desert_well_loot(position, block_position_as_long(position));
    }
    Ok(true)
}

fn offer_center_and_cardinals<W: DesertWellWorld>(
    world: &mut W,
    center: BlockPos,
    y_offset: i32,
    state: BlockStateId,
) -> Result<(), TerrainFeatureError> {
    for (x_offset, z_offset) in [(0, 0), (0, -1), (1, 0), (0, 1), (-1, 0)] {
        offer_well(
            world,
            offset(center, x_offset, y_offset, z_offset)?,
            state,
            2,
        );
    }
    Ok(())
}

fn offer_well<W: DesertWellWorld>(
    world: &mut W,
    position: BlockPos,
    state: BlockStateId,
    flags: u32,
) {
    let _ = world.offer_well_block(position, state, flags);
}

const fn block_position_as_long(position: BlockPos) -> i64 {
    ((position.x as i64 & 0x3ff_ffff) << 38)
        | ((position.z as i64 & 0x3ff_ffff) << 12)
        | (position.y as i64 & 0xfff)
}

pub trait SpikeWorld {
    fn minimum_y(&self) -> i32;

    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn can_place_spike_on(&self, state: BlockStateId) -> bool;

    fn can_replace_with_spike(&self, state: BlockStateId) -> bool;

    fn same_block_identity(&self, left: BlockStateId, right: BlockStateId) -> bool;

    fn offer_spike(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_spike<R, W>(
    world: &mut W,
    origin: BlockPos,
    spike: BlockStateId,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TerrainFeatureError>
where
    R: GenerationRandom,
    W: SpikeWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let minimum_stop_y = world
        .minimum_y()
        .checked_add(2)
        .ok_or(TerrainFeatureError::PositionOverflow)?;
    let mut center = origin;
    while world.is_empty_block(center) && center.y > minimum_stop_y {
        center = offset(center, 0, -1, 0)?;
    }
    let support = world.block_state(center);
    if !world.can_place_spike_on(support) {
        return Ok(false);
    }
    center = offset(
        center,
        0,
        random.next_u32(NonZeroU32::new(4).expect("spike center bound is nonzero")) as i32,
        0,
    )?;
    let height =
        7 + random.next_u32(NonZeroU32::new(4).expect("spike height bound is nonzero")) as i32;
    let base_radius = height / 4
        + random.next_u32(NonZeroU32::new(2).expect("spike radius bound is nonzero")) as i32;
    if base_radius > 1
        && random.next_u32(NonZeroU32::new(60).expect("spike rarity bound is nonzero")) == 0
    {
        let extra_height =
            10 + random.next_u32(NonZeroU32::new(30).expect("spike lift bound is nonzero")) as i32;
        center = offset(center, 0, extra_height, 0)?;
    }

    place_spike_body(world, center, height, base_radius, spike, random)?;
    place_spike_roots(world, center, base_radius, spike, random)?;
    Ok(true)
}

fn place_spike_body<R, W>(
    world: &mut W,
    center: BlockPos,
    height: i32,
    base_radius: i32,
    spike: BlockStateId,
    random: &mut R,
) -> Result<(), TerrainFeatureError>
where
    R: GenerationRandom,
    W: SpikeWorld,
{
    for layer in 0..height {
        let radius = (1.0_f32 - layer as f32 / height as f32) * base_radius as f32;
        let extent = radius.ceil() as i32;
        for x_offset in -extent..=extent {
            for z_offset in -extent..=extent {
                if (x_offset != 0 || z_offset != 0)
                    && radial_distance(x_offset, z_offset) > radius * radius
                {
                    continue;
                }
                let perimeter = x_offset == -extent
                    || x_offset == extent
                    || z_offset == -extent
                    || z_offset == extent;
                if perimeter && random.next_f32() > 0.75 {
                    continue;
                }
                let upper = offset(center, x_offset, layer, z_offset)?;
                offer_spike_if_replaceable(world, upper, spike);
                if layer != 0 && extent > 1 {
                    let lower = offset(center, x_offset, -layer, z_offset)?;
                    offer_spike_if_replaceable(world, lower, spike);
                }
            }
        }
    }
    Ok(())
}

fn radial_distance(x_offset: i32, z_offset: i32) -> f32 {
    let x = x_offset.unsigned_abs() as f32 - 0.25;
    let z = z_offset.unsigned_abs() as f32 - 0.25;
    x * x + z * z
}

fn offer_spike_if_replaceable<W: SpikeWorld>(
    world: &mut W,
    position: BlockPos,
    spike: BlockStateId,
) {
    let state = world.block_state(position);
    if world.is_air(state) || world.can_replace_with_spike(state) {
        let _ = world.offer_spike(position, spike, 3);
    }
}

fn place_spike_roots<R, W>(
    world: &mut W,
    center: BlockPos,
    base_radius: i32,
    spike: BlockStateId,
    random: &mut R,
) -> Result<(), TerrainFeatureError>
where
    R: GenerationRandom,
    W: SpikeWorld,
{
    let extent = (base_radius - 1).clamp(0, 1);
    for x_offset in -extent..=extent {
        for z_offset in -extent..=extent {
            let mut cursor = offset(center, x_offset, -1, z_offset)?;
            let mut segment = if x_offset != 0 && z_offset != 0 {
                random.next_u32(NonZeroU32::new(5).expect("spike root segment bound is nonzero"))
                    as i32
            } else {
                50
            };
            while cursor.y > 50 {
                let state = world.block_state(cursor);
                if !(world.is_air(state)
                    || world.can_replace_with_spike(state)
                    || world.same_block_identity(state, spike))
                {
                    break;
                }
                let _ = world.offer_spike(cursor, spike, 3);
                cursor = offset(cursor, 0, -1, 0)?;
                segment -= 1;
                if segment <= 0 {
                    let gap = 1 + random
                        .next_u32(NonZeroU32::new(5).expect("spike root gap bound is nonzero"))
                        as i32;
                    cursor = offset(cursor, 0, -gap, 0)?;
                    segment = random
                        .next_u32(NonZeroU32::new(5).expect("spike root reset bound is nonzero"))
                        as i32;
                }
            }
        }
    }
    Ok(())
}

fn sample_replacement_radius(
    provider: &IntProvider,
    random: &mut impl GenerationRandom,
) -> Result<i32, TerrainFeatureError> {
    let radius = provider.sample(random)?;
    if (0..=12).contains(&radius) {
        Ok(radius)
    } else {
        Err(TerrainFeatureError::ReplacementRadiusOutOfRange { radius })
    }
}

fn mark_above_for_postprocessing<R, W>(
    world: &mut W,
    position: BlockPos,
) -> Result<(), TerrainFeatureError>
where
    R: GenerationRandom,
    W: DiskWorld<R>,
{
    let mut cursor = position;
    for _ in 0..2 {
        cursor = offset(cursor, 0, 1, 0)?;
        let state = world.block_state(cursor);
        if world.is_air(state) {
            break;
        }
        world.mark_for_postprocessing(cursor);
    }
    Ok(())
}

fn offset(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, TerrainFeatureError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(TerrainFeatureError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(TerrainFeatureError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(TerrainFeatureError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TerrainFeatureError {
    #[error("terrain-feature position arithmetic overflowed")]
    PositionOverflow,
    #[error("disk radius {radius} is outside 0..=8")]
    DiskRadiusOutOfRange { radius: i32 },
    #[error("disk half-height {half_height} is outside 0..=4")]
    DiskHalfHeightOutOfRange { half_height: u8 },
    #[error("terrain-feature integer provider failed")]
    Provider(#[from] ProviderError),
    #[error("replacement-blob radius {radius} is outside 0..=12")]
    ReplacementRadiusOutOfRange { radius: i32 },
    #[error("delta-feature reach {reach} is outside 0..=16")]
    DeltaReachOutOfRange { reach: i32 },
    #[error("underwater-magma floor search range {range} is outside 0..=512")]
    FloorSearchRangeOutOfRange { range: u32 },
    #[error("underwater-magma placement radius {radius} is outside 0..=64")]
    MagmaRadiusOutOfRange { radius: u32 },
    #[error("terrain-feature probability must be finite and in the inclusive range 0..=1")]
    InvalidProbability,
}
