//! Small configured features with fixed traversal and offer semantics.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::id::{BiomeId, BlockStateId};

pub const BASIC_FEATURE_WRITE_FLAGS: u32 = 2;
pub const STRUCTURAL_FEATURE_WRITE_FLAGS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VineFace {
    Up,
    North,
    South,
    West,
    East,
}

impl VineFace {
    pub const ORDERED: [Self; 5] = [Self::Up, Self::North, Self::South, Self::West, Self::East];

    const fn offset(self) -> (i32, i32, i32) {
        match self {
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }
}

pub trait VinesWorld {
    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn can_attach_vine_to(&mut self, neighbor: BlockPos, face: VineFace) -> bool;

    fn offer_vine(&mut self, position: BlockPos, attached: VineFace, flags: u32) -> bool;
}

pub fn place_vines<W: VinesWorld>(
    world: &mut W,
    origin: BlockPos,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError> {
    if !ensure_can_write(origin) || !world.is_empty_block(origin) {
        return Ok(false);
    }
    for face in VineFace::ORDERED {
        let (x, y, z) = face.offset();
        let neighbor = offset(origin, x, y, z)?;
        if world.can_attach_vine_to(neighbor, face) {
            let _ = world.offer_vine(origin, face, BASIC_FEATURE_WRITE_FLAGS);
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeaPickleState {
    pub count: u8,
    pub waterlogged: bool,
}

pub trait SeaPickleWorld {
    fn ocean_floor_height(&mut self, x: i32, z: i32) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_exact_water(&self, state: BlockStateId) -> bool;

    fn sea_pickle_survives(&mut self, position: BlockPos, state: SeaPickleState) -> bool;

    fn offer_sea_pickle(&mut self, position: BlockPos, state: SeaPickleState, flags: u32) -> bool;
}

pub fn place_sea_pickles<R, W>(
    world: &mut W,
    origin: BlockPos,
    count: &IntProvider,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError>
where
    R: GenerationRandom,
    W: SeaPickleWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let attempts = count.sample(random)?;
    if !(0..=256).contains(&attempts) {
        return Err(BasicFeatureError::SeaPickleCountOutOfRange { count: attempts });
    }
    let mut offers = 0_u32;
    let eight = NonZeroU32::new(8).expect("eight is nonzero");
    let four = NonZeroU32::new(4).expect("four is nonzero");
    for _ in 0..attempts {
        let dx = random.next_u32(eight) as i32 - random.next_u32(eight) as i32;
        let dz = random.next_u32(eight) as i32 - random.next_u32(eight) as i32;
        let x = origin
            .x
            .checked_add(dx)
            .ok_or(BasicFeatureError::PositionOverflow)?;
        let z = origin
            .z
            .checked_add(dz)
            .ok_or(BasicFeatureError::PositionOverflow)?;
        let y = world.ocean_floor_height(x, z);
        let state = SeaPickleState {
            count: (random.next_u32(four) + 1) as u8,
            waterlogged: true,
        };
        let position = BlockPos::new(x, y, z);
        let existing = world.block_state(position);
        if !world.is_exact_water(existing) || !world.sea_pickle_survives(position, state) {
            continue;
        }
        let _ = world.offer_sea_pickle(position, state, BASIC_FEATURE_WRITE_FLAGS);
        offers += 1;
    }
    Ok(offers != 0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IceDirection {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl IceDirection {
    const ADMISSION_ORDER: [Self; 5] = [Self::Up, Self::North, Self::South, Self::West, Self::East];

    const PROPAGATION_ORDER: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];

    const fn offset(self) -> (i32, i32, i32) {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }
}

pub trait BlueIceWorld {
    fn sea_level(&self) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_exact_water(&self, state: BlockStateId) -> bool;

    fn is_packed_ice(&self, state: BlockStateId) -> bool;

    fn is_blue_ice(&self, state: BlockStateId) -> bool;

    fn is_blue_ice_candidate(&self, state: BlockStateId) -> bool;

    fn offer_blue_ice(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_blue_ice<R, W>(
    world: &mut W,
    origin: BlockPos,
    blue_ice: BlockStateId,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError>
where
    R: GenerationRandom,
    W: BlueIceWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let maximum_y = world
        .sea_level()
        .checked_sub(1)
        .ok_or(BasicFeatureError::PositionOverflow)?;
    if origin.y > maximum_y {
        return Ok(false);
    }
    let origin_state = world.block_state(origin);
    if !world.is_exact_water(origin_state) {
        let below = offset(origin, 0, -1, 0)?;
        let below_state = world.block_state(below);
        if !world.is_exact_water(below_state) {
            return Ok(false);
        }
    }
    let mut packed_ice_neighbor = false;
    for direction in IceDirection::ADMISSION_ORDER {
        let (x, y, z) = direction.offset();
        let neighbor = offset(origin, x, y, z)?;
        let neighbor_state = world.block_state(neighbor);
        if world.is_packed_ice(neighbor_state) {
            packed_ice_neighbor = true;
            break;
        }
    }
    if !packed_ice_neighbor {
        return Ok(false);
    }
    let _ = world.offer_blue_ice(origin, blue_ice, BASIC_FEATURE_WRITE_FLAGS);

    let five = NonZeroU32::new(5).expect("five is nonzero");
    let six = NonZeroU32::new(6).expect("six is nonzero");
    for _ in 0..200 {
        let dy = random.next_u32(five) as i32 - random.next_u32(six) as i32;
        let radius = if dy < 2 { 3 + dy / 2 } else { 3 };
        let radius =
            NonZeroU32::new(radius as u32).expect("audited blue-ice radius is always positive");
        let dx = random.next_u32(radius) as i32 - random.next_u32(radius) as i32;
        let dz = random.next_u32(radius) as i32 - random.next_u32(radius) as i32;
        let candidate = offset(origin, dx, dy, dz)?;
        let candidate_state = world.block_state(candidate);
        if !world.is_blue_ice_candidate(candidate_state) {
            continue;
        }
        for direction in IceDirection::PROPAGATION_ORDER {
            let (x, y, z) = direction.offset();
            let neighbor = offset(candidate, x, y, z)?;
            let neighbor_state = world.block_state(neighbor);
            if world.is_blue_ice(neighbor_state) {
                let _ = world.offer_blue_ice(candidate, blue_ice, BASIC_FEATURE_WRITE_FLAGS);
                break;
            }
        }
    }
    Ok(true)
}

pub trait KelpWorld {
    fn ocean_floor_height(&mut self, x: i32, z: i32) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_exact_water(&self, state: BlockStateId) -> bool;

    fn kelp_body_survives(&mut self, position: BlockPos) -> bool;

    fn kelp_head_survives(&mut self, position: BlockPos) -> bool;

    fn is_kelp_head(&self, state: BlockStateId) -> bool;

    fn offer_kelp_body(&mut self, position: BlockPos, flags: u32) -> bool;

    fn offer_kelp_head(&mut self, position: BlockPos, age: u8, flags: u32) -> bool;
}

pub fn place_kelp<R, W>(
    world: &mut W,
    origin: BlockPos,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError>
where
    R: GenerationRandom,
    W: KelpWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let floor_y = world.ocean_floor_height(origin.x, origin.z);
    let floor = BlockPos::new(origin.x, floor_y, origin.z);
    let floor_state = world.block_state(floor);
    if !world.is_exact_water(floor_state) {
        return Ok(false);
    }
    let height = random.next_u32(NonZeroU32::new(10).expect("ten is nonzero")) as usize + 1;
    let mut cursor = floor;
    let mut head_offered = false;
    for index in 0..=height {
        let above = offset(cursor, 0, 1, 0)?;
        let current_state = world.block_state(cursor);
        let above_state = world.block_state(above);
        if world.is_exact_water(current_state)
            && world.is_exact_water(above_state)
            && world.kelp_body_survives(cursor)
        {
            if index == height {
                let age =
                    (random.next_u32(NonZeroU32::new(4).expect("four is nonzero")) + 20) as u8;
                let _ = world.offer_kelp_head(cursor, age, BASIC_FEATURE_WRITE_FLAGS);
                head_offered = true;
                break;
            }
            let _ = world.offer_kelp_body(cursor, BASIC_FEATURE_WRITE_FLAGS);
            cursor = above;
            continue;
        }
        if index == 0 {
            cursor = above;
            continue;
        }
        let previous = offset(cursor, 0, -1, 0)?;
        if world.kelp_head_survives(previous) {
            let below_previous = offset(previous, 0, -1, 0)?;
            let below_state = world.block_state(below_previous);
            if !world.is_kelp_head(below_state) {
                let age =
                    (random.next_u32(NonZeroU32::new(4).expect("four is nonzero")) + 20) as u8;
                let _ = world.offer_kelp_head(previous, age, BASIC_FEATURE_WRITE_FLAGS);
                head_offered = true;
            }
        }
        break;
    }
    Ok(head_offered)
}

pub trait EndIslandWorld {
    fn offer_end_stone(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_end_island<R, W>(
    world: &mut W,
    origin: BlockPos,
    end_stone: BlockStateId,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError>
where
    R: GenerationRandom,
    W: EndIslandWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let mut radius = random.next_u32(NonZeroU32::new(3).expect("three is nonzero")) as f32 + 4.0;
    let mut y_offset = 0_i32;
    while radius > 0.5 {
        let minimum = (-radius).floor() as i32;
        let maximum = radius.ceil() as i32;
        let admitted_radius = (radius + 1.0) * (radius + 1.0);
        for x_offset in minimum..=maximum {
            for z_offset in minimum..=maximum {
                let distance = x_offset * x_offset + z_offset * z_offset;
                if distance as f32 <= admitted_radius {
                    let position = offset(origin, x_offset, y_offset, z_offset)?;
                    let _ =
                        world.offer_end_stone(position, end_stone, STRUCTURAL_FEATURE_WRITE_FLAGS);
                }
            }
        }
        radius -= random.next_u32(NonZeroU32::new(2).expect("two is nonzero")) as f32 + 0.5;
        y_offset = y_offset
            .checked_sub(1)
            .ok_or(BasicFeatureError::PositionOverflow)?;
    }
    Ok(true)
}

pub trait GlowstoneWorld {
    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_glowstone_support(&self, state: BlockStateId) -> bool;

    fn is_glowstone(&self, state: BlockStateId) -> bool;

    fn offer_glowstone(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_glowstone_blob<R, W>(
    world: &mut W,
    origin: BlockPos,
    glowstone: BlockStateId,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError>
where
    R: GenerationRandom,
    W: GlowstoneWorld,
{
    if !ensure_can_write(origin) || !world.is_empty_block(origin) {
        return Ok(false);
    }
    let above = offset(origin, 0, 1, 0)?;
    let above_state = world.block_state(above);
    if !world.is_glowstone_support(above_state) {
        return Ok(false);
    }
    let _ = world.offer_glowstone(origin, glowstone, BASIC_FEATURE_WRITE_FLAGS);

    let eight = NonZeroU32::new(8).expect("eight is nonzero");
    let twelve = NonZeroU32::new(12).expect("twelve is nonzero");
    for _ in 0..1_500 {
        let dx = random.next_u32(eight) as i32 - random.next_u32(eight) as i32;
        let dy = -(random.next_u32(twelve) as i32);
        let dz = random.next_u32(eight) as i32 - random.next_u32(eight) as i32;
        let candidate = offset(origin, dx, dy, dz)?;
        if !world.is_empty_block(candidate) {
            continue;
        }
        let mut neighbors = 0_u8;
        for direction in IceDirection::PROPAGATION_ORDER {
            let (x, y, z) = direction.offset();
            let neighbor = offset(candidate, x, y, z)?;
            let neighbor_state = world.block_state(neighbor);
            if world.is_glowstone(neighbor_state) {
                neighbors += 1;
                if neighbors == 2 {
                    break;
                }
            }
        }
        if neighbors == 1 {
            let _ = world.offer_glowstone(candidate, glowstone, BASIC_FEATURE_WRITE_FLAGS);
        }
    }
    Ok(true)
}

pub trait BlockBlobWorld {
    fn minimum_y(&self) -> i32;

    fn can_place_blob_on(&mut self, position: BlockPos) -> bool;

    fn offer_blob_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_block_blob<R, W>(
    world: &mut W,
    origin: BlockPos,
    state: BlockStateId,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError>
where
    R: GenerationRandom,
    W: BlockBlobWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let minimum_center_y = world
        .minimum_y()
        .checked_add(3)
        .ok_or(BasicFeatureError::PositionOverflow)?;
    let mut center = origin;
    while center.y > minimum_center_y {
        let below = offset(center, 0, -1, 0)?;
        if world.can_place_blob_on(below) {
            break;
        }
        center = below;
    }
    if center.y <= minimum_center_y {
        return Ok(false);
    }

    let two = NonZeroU32::new(2).expect("two is nonzero");
    for _ in 0..3 {
        let x_extent = random.next_u32(two) as i32;
        let y_extent = random.next_u32(two) as i32;
        let z_extent = random.next_u32(two) as i32;
        let extent_sum = x_extent + y_extent + z_extent;
        let radius = extent_sum as f32 * 0.333_f32 + 0.5;
        let radius_squared = radius * radius;
        for z_offset in -z_extent..=z_extent {
            for y_offset in -y_extent..=y_extent {
                for x_offset in -x_extent..=x_extent {
                    let distance = x_offset * x_offset + y_offset * y_offset + z_offset * z_offset;
                    if distance as f32 <= radius_squared {
                        let position = offset(center, x_offset, y_offset, z_offset)?;
                        let _ =
                            world.offer_blob_block(position, state, STRUCTURAL_FEATURE_WRITE_FLAGS);
                    }
                }
            }
        }
        let x_shift = -1 + random.next_u32(two) as i32;
        let y_shift = -(random.next_u32(two) as i32);
        let z_shift = -1 + random.next_u32(two) as i32;
        center = offset(center, x_shift, y_shift, z_shift)?;
    }
    Ok(true)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeagrassPart {
    Short,
    TallLower,
    TallUpper,
}

pub trait SeagrassWorld {
    fn ocean_floor_height(&mut self, x: i32, z: i32) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_exact_water(&self, state: BlockStateId) -> bool;

    fn seagrass_survives(&mut self, position: BlockPos, tall: bool) -> bool;

    fn offer_seagrass(&mut self, position: BlockPos, part: SeagrassPart, flags: u32) -> bool;
}

pub fn place_seagrass<R, W>(
    world: &mut W,
    origin: BlockPos,
    tall_probability: f64,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError>
where
    R: GenerationRandom,
    W: SeagrassWorld,
{
    if !tall_probability.is_finite() || !(0.0..=1.0).contains(&tall_probability) {
        return Err(BasicFeatureError::InvalidProbability);
    }
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let eight = NonZeroU32::new(8).expect("eight is nonzero");
    let dx = random.next_u32(eight) as i32 - random.next_u32(eight) as i32;
    let dz = random.next_u32(eight) as i32 - random.next_u32(eight) as i32;
    let x = origin
        .x
        .checked_add(dx)
        .ok_or(BasicFeatureError::PositionOverflow)?;
    let z = origin
        .z
        .checked_add(dz)
        .ok_or(BasicFeatureError::PositionOverflow)?;
    let y = world.ocean_floor_height(x, z);
    let candidate = BlockPos::new(x, y, z);
    let candidate_state = world.block_state(candidate);
    if !world.is_exact_water(candidate_state) {
        return Ok(false);
    }
    let tall = random.next_f64() < tall_probability;
    if !world.seagrass_survives(candidate, tall) {
        return Ok(false);
    }
    if !tall {
        let _ = world.offer_seagrass(candidate, SeagrassPart::Short, BASIC_FEATURE_WRITE_FLAGS);
        return Ok(true);
    }
    let above = offset(candidate, 0, 1, 0)?;
    let above_state = world.block_state(above);
    if world.is_exact_water(above_state) {
        let _ = world.offer_seagrass(
            candidate,
            SeagrassPart::TallLower,
            BASIC_FEATURE_WRITE_FLAGS,
        );
        let _ = world.offer_seagrass(above, SeagrassPart::TallUpper, BASIC_FEATURE_WRITE_FLAGS);
    }
    Ok(true)
}

pub trait NetherVegetationWorld<R: GenerationRandom> {
    fn minimum_y(&self) -> i32;

    fn maximum_y(&self) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_nylium(&self, state: BlockStateId) -> bool;

    fn provide_vegetation_state(&mut self, position: BlockPos, random: &mut R) -> BlockStateId;

    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn vegetation_survives(&mut self, state: BlockStateId, position: BlockPos) -> bool;

    fn offer_vegetation(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

pub fn place_nether_forest_vegetation<R, W>(
    world: &mut W,
    origin: BlockPos,
    spread_width: NonZeroU32,
    spread_height: NonZeroU32,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError>
where
    R: GenerationRandom,
    W: NetherVegetationWorld<R>,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let below = offset(origin, 0, -1, 0)?;
    let below_state = world.block_state(below);
    if !world.is_nylium(below_state) {
        return Ok(false);
    }
    let minimum = world
        .minimum_y()
        .checked_add(1)
        .ok_or(BasicFeatureError::PositionOverflow)?;
    let above_y = origin
        .y
        .checked_add(1)
        .ok_or(BasicFeatureError::PositionOverflow)?;
    if origin.y < minimum || above_y > world.maximum_y() {
        return Ok(false);
    }
    let width = spread_width.get();
    let attempts = (width as i32).wrapping_mul(width as i32);
    if attempts <= 0 {
        return Ok(false);
    }
    let mut offers = 0_u32;
    for _ in 0..attempts {
        let dx = random.next_u32(spread_width) as i32 - random.next_u32(spread_width) as i32;
        let dy = random.next_u32(spread_height) as i32 - random.next_u32(spread_height) as i32;
        let dz = random.next_u32(spread_width) as i32 - random.next_u32(spread_width) as i32;
        let candidate = offset(origin, dx, dy, dz)?;
        let state = world.provide_vegetation_state(candidate, random);
        if !world.is_empty_block(candidate)
            || candidate.y <= world.minimum_y()
            || !world.vegetation_survives(state, candidate)
        {
            continue;
        }
        let _ = world.offer_vegetation(candidate, state, BASIC_FEATURE_WRITE_FLAGS);
        offers += 1;
    }
    Ok(offers != 0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpringDirection {
    West,
    East,
    North,
    South,
    Down,
}

impl SpringDirection {
    const ORDERED: [Self; 5] = [Self::West, Self::East, Self::North, Self::South, Self::Down];

    const fn offset(self) -> (i32, i32, i32) {
        match self {
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::Down => (0, -1, 0),
        }
    }
}

pub trait SpringWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_valid_spring_block(&self, state: BlockStateId) -> bool;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn offer_spring_fluid(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn schedule_spring_fluid(&mut self, position: BlockPos, delay: u32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpringConfiguration {
    pub fluid_legacy_block: BlockStateId,
    pub requires_block_below: bool,
    pub rock_count: i32,
    pub hole_count: i32,
}

pub fn place_spring<W: SpringWorld>(
    world: &mut W,
    origin: BlockPos,
    configuration: SpringConfiguration,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError> {
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let above = offset(origin, 0, 1, 0)?;
    let above_state = world.block_state(above);
    if !world.is_valid_spring_block(above_state) {
        return Ok(false);
    }
    if configuration.requires_block_below {
        let below = offset(origin, 0, -1, 0)?;
        let below_state = world.block_state(below);
        if !world.is_valid_spring_block(below_state) {
            return Ok(false);
        }
    }
    let origin_state = world.block_state(origin);
    if !world.is_air(origin_state) && !world.is_valid_spring_block(origin_state) {
        return Ok(false);
    }

    let mut valid_blocks = 0_i32;
    for direction in SpringDirection::ORDERED {
        let (x, y, z) = direction.offset();
        let position = offset(origin, x, y, z)?;
        let state = world.block_state(position);
        if world.is_valid_spring_block(state) {
            valid_blocks += 1;
        }
    }
    let mut empty_blocks = 0_i32;
    for direction in SpringDirection::ORDERED {
        let (x, y, z) = direction.offset();
        let position = offset(origin, x, y, z)?;
        if world.is_empty_block(position) {
            empty_blocks += 1;
        }
    }
    if valid_blocks != configuration.rock_count || empty_blocks != configuration.hole_count {
        return Ok(false);
    }
    let _ = world.offer_spring_fluid(
        origin,
        configuration.fluid_legacy_block,
        BASIC_FEATURE_WRITE_FLAGS,
    );
    world.schedule_spring_fluid(origin, 0);
    Ok(true)
}

pub trait FreezeTopLayerWorld {
    fn motion_blocking_height(&mut self, x: i32, z: i32) -> i32;

    fn biome(&mut self, position: BlockPos) -> BiomeId;

    fn should_freeze(
        &mut self,
        biome: BiomeId,
        position: BlockPos,
        require_horizontal_edge: bool,
    ) -> bool;

    fn should_snow(&mut self, biome: BiomeId, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn with_snowy_true(&self, state: BlockStateId) -> Option<BlockStateId>;

    fn offer_frozen_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreezeTopLayerStates {
    pub ice: BlockStateId,
    pub snow: BlockStateId,
}

pub fn freeze_top_layer<W: FreezeTopLayerWorld>(
    world: &mut W,
    origin: BlockPos,
    states: FreezeTopLayerStates,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, BasicFeatureError> {
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    for x_offset in 0..16 {
        let x = origin
            .x
            .checked_add(x_offset)
            .ok_or(BasicFeatureError::PositionOverflow)?;
        for z_offset in 0..16 {
            let z = origin
                .z
                .checked_add(z_offset)
                .ok_or(BasicFeatureError::PositionOverflow)?;
            let y = world.motion_blocking_height(x, z);
            let surface = BlockPos::new(x, y, z);
            let below = offset(surface, 0, -1, 0)?;
            let biome = world.biome(surface);
            if world.should_freeze(biome, below, false) {
                let _ = world.offer_frozen_block(below, states.ice, BASIC_FEATURE_WRITE_FLAGS);
            }
            if world.should_snow(biome, surface) {
                let _ = world.offer_frozen_block(surface, states.snow, BASIC_FEATURE_WRITE_FLAGS);
                let below_state = world.block_state(below);
                if let Some(snowy_state) = world.with_snowy_true(below_state) {
                    let _ = world.offer_frozen_block(below, snowy_state, BASIC_FEATURE_WRITE_FLAGS);
                }
            }
        }
    }
    Ok(true)
}

fn offset(origin: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, BasicFeatureError> {
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(BasicFeatureError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(BasicFeatureError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(BasicFeatureError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BasicFeatureError {
    #[error("basic-feature position arithmetic overflowed")]
    PositionOverflow,
    #[error("sea-pickle count {count} is outside 0..=256")]
    SeaPickleCountOutOfRange { count: i32 },
    #[error("sea-pickle count provider failed")]
    Provider(#[from] ProviderError),
    #[error("feature probability must be finite and in the inclusive range 0..=1")]
    InvalidProbability,
}
