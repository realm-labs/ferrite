//! Shared carver ellipsoid traversal, mask ordering, and material kernels.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarverEllipsoid {
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub horizontal_radius: f64,
    pub vertical_radius: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarverMaterialConfig {
    pub lava_level: i32,
    pub lava: BlockStateId,
    pub debug_mode: bool,
    pub debug_barrier: BlockStateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherCarverStates {
    pub lava: BlockStateId,
    pub cave_air: BlockStateId,
}

pub trait CarvingMask {
    fn contains(&self, local_x: u8, y: i32, local_z: u8) -> bool;

    fn set(&mut self, local_x: u8, y: i32, local_z: u8);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OldGenerationCuboid {
    pub center: [f64; 3],
    pub half_extents: [f64; 3],
}

impl OldGenerationCuboid {
    pub fn from_blending_data(minimum_y: i32, height: i32, offset_x: i32, offset_z: i32) -> Self {
        Self {
            center: [
                f64::from(8 + offset_x),
                f64::from(minimum_y) + f64::from(height) / 2.0,
                f64::from(8 + offset_z),
            ],
            half_extents: [8.0, f64::from(height) / 2.0, 8.0],
        }
    }
}

pub trait CarverShiftNoise {
    fn sample(&mut self, x: i32, y: i32, z: i32) -> f64;
}

pub fn old_generation_excludes(
    cuboids: &[OldGenerationCuboid],
    noise: &mut impl CarverShiftNoise,
    x: i32,
    y: i32,
    z: i32,
) -> bool {
    let shifted = [
        f64::from(x) + 0.5 + 4.0 * noise.sample(x, y, z),
        f64::from(y) + 0.5 + 4.0 * noise.sample(y, z, x),
        f64::from(z) + 0.5 + 4.0 * noise.sample(z, x, y),
    ];
    cuboids.iter().any(|cuboid| {
        let squared = (0..3)
            .map(|axis| {
                let outside = ((shifted[axis] - cuboid.center[axis]).abs()
                    - cuboid.half_extents[axis])
                    .max(0.0);
                outside * outside
            })
            .sum::<f64>();
        squared.sqrt() < 4.0
    })
}

pub trait CarverWorld {
    fn minimum_y(&self) -> i32;

    fn generation_depth(&self) -> i32;

    fn upgrading_chunk(&self) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_grass_or_mycelium(&self, state: BlockStateId) -> bool;

    fn is_carver_replaceable(&self, state: BlockStateId) -> bool;

    fn aquifer_substance(&mut self, position: BlockPos, density: f64) -> Option<BlockStateId>;

    fn debug_marker_for(&self, state: BlockStateId) -> BlockStateId;

    fn aquifer_should_schedule_fluid_update(&mut self) -> bool;

    fn has_nonempty_fluid(&self, state: BlockStateId) -> bool;

    fn offer_carved_block(&mut self, position: BlockPos, state: BlockStateId);

    fn mark_for_postprocessing(&mut self, position: BlockPos);

    fn is_dirt(&self, state: BlockStateId) -> bool;

    fn surface_top_material(
        &mut self,
        position: BlockPos,
        stone_depth_above: i32,
        stone_depth_below: i32,
        water_height: i32,
    ) -> Option<BlockStateId>;
}

pub fn carve_ellipsoid<W, M, S>(
    world: &mut W,
    mask: &mut M,
    chunk_minimum_x: i32,
    chunk_minimum_z: i32,
    ellipsoid: CarverEllipsoid,
    config: CarverMaterialConfig,
    mut skip: S,
) -> Result<bool, CarverError>
where
    W: CarverWorld,
    M: CarvingMask,
    S: FnMut(f64, f64, f64, i32) -> bool,
{
    validate_ellipsoid(ellipsoid)?;
    let chunk_middle_x = chunk_minimum_x
        .checked_add(8)
        .ok_or(CarverError::PositionOverflow)?;
    let chunk_middle_z = chunk_minimum_z
        .checked_add(8)
        .ok_or(CarverError::PositionOverflow)?;
    let reach = 16.0 + ellipsoid.horizontal_radius * 2.0;
    if (ellipsoid.center_x - f64::from(chunk_middle_x)).abs() > reach
        || (ellipsoid.center_z - f64::from(chunk_middle_z)).abs() > reach
    {
        return Ok(false);
    }

    let minimum_local_x =
        ((ellipsoid.center_x - ellipsoid.horizontal_radius).floor() as i32 - chunk_minimum_x - 1)
            .max(0);
    let maximum_local_x = ((ellipsoid.center_x + ellipsoid.horizontal_radius).floor() as i32
        - chunk_minimum_x)
        .min(15);
    let minimum_local_z =
        ((ellipsoid.center_z - ellipsoid.horizontal_radius).floor() as i32 - chunk_minimum_z - 1)
            .max(0);
    let maximum_local_z = ((ellipsoid.center_z + ellipsoid.horizontal_radius).floor() as i32
        - chunk_minimum_z)
        .min(15);
    let top_guard = if world.upgrading_chunk() { 0 } else { 7 };
    let world_top = world
        .minimum_y()
        .checked_add(world.generation_depth())
        .and_then(|value| value.checked_sub(1 + top_guard))
        .ok_or(CarverError::PositionOverflow)?;
    let maximum_y =
        ((ellipsoid.center_y + ellipsoid.vertical_radius).floor() as i32 + 1).min(world_top);
    let minimum_y = ((ellipsoid.center_y - ellipsoid.vertical_radius).floor() as i32 - 1)
        .max(world.minimum_y() + 1);

    let mut carved = false;
    for local_x in minimum_local_x..=maximum_local_x {
        let world_x = chunk_minimum_x + local_x;
        let normalized_x =
            (f64::from(world_x) + 0.5 - ellipsoid.center_x) / ellipsoid.horizontal_radius;
        for local_z in minimum_local_z..=maximum_local_z {
            let world_z = chunk_minimum_z + local_z;
            let normalized_z =
                (f64::from(world_z) + 0.5 - ellipsoid.center_z) / ellipsoid.horizontal_radius;
            if normalized_x * normalized_x + normalized_z * normalized_z >= 1.0 {
                continue;
            }
            let mut found_surface = false;
            for y in (minimum_y + 1..=maximum_y).rev() {
                let normalized_y =
                    (f64::from(y) - 0.5 - ellipsoid.center_y) / ellipsoid.vertical_radius;
                if skip(normalized_x, normalized_y, normalized_z, y) {
                    continue;
                }
                let local_x = local_x as u8;
                let local_z = local_z as u8;
                if mask.contains(local_x, y, local_z) && !config.debug_mode {
                    continue;
                }
                mask.set(local_x, y, local_z);
                let position = BlockPos::new(world_x, y, world_z);
                if carve_material(world, position, config, &mut found_surface)? {
                    carved = true;
                }
            }
        }
    }
    Ok(carved)
}

pub fn carve_nether_ellipsoid<W, M, S>(
    world: &mut W,
    mask: &mut M,
    chunk_minimum: [i32; 2],
    ellipsoid: CarverEllipsoid,
    states: NetherCarverStates,
    skip: S,
) -> Result<bool, CarverError>
where
    W: CarverWorld,
    M: CarvingMask,
    S: FnMut(f64, f64, f64, i32) -> bool,
{
    let lava_cutoff = world
        .minimum_y()
        .checked_add(31)
        .ok_or(CarverError::PositionOverflow)?;
    let config = CarverMaterialConfig {
        lava_level: i32::MIN,
        lava: states.lava,
        debug_mode: false,
        debug_barrier: states.cave_air,
    };
    carve_ellipsoid(
        &mut NetherWorld {
            inner: world,
            lava_cutoff,
            lava: states.lava,
            cave_air: states.cave_air,
        },
        mask,
        chunk_minimum[0],
        chunk_minimum[1],
        ellipsoid,
        config,
        skip,
    )
}

fn carve_material(
    world: &mut impl CarverWorld,
    position: BlockPos,
    config: CarverMaterialConfig,
    found_surface: &mut bool,
) -> Result<bool, CarverError> {
    let old_state = world.block_state(position);
    *found_surface |= world.is_grass_or_mycelium(old_state);
    if !config.debug_mode && !world.is_carver_replaceable(old_state) {
        return Ok(false);
    }
    let state = if position.y <= config.lava_level {
        config.lava
    } else {
        match world.aquifer_substance(position, 0.0) {
            Some(state) if config.debug_mode => world.debug_marker_for(state),
            Some(state) => state,
            None if config.debug_mode => config.debug_barrier,
            None => return Ok(false),
        }
    };
    world.offer_carved_block(position, state);
    if world.aquifer_should_schedule_fluid_update() && world.has_nonempty_fluid(state) {
        world.mark_for_postprocessing(position);
    }
    if *found_surface {
        let below = offset_y(position, -1)?;
        let below_state = world.block_state(below);
        if world.is_dirt(below_state) {
            let water_height = if world.has_nonempty_fluid(state) {
                position
                    .y
                    .checked_add(1)
                    .ok_or(CarverError::PositionOverflow)?
            } else {
                i32::MIN
            };
            if let Some(top) = world.surface_top_material(below, 1, 1, water_height) {
                world.offer_carved_block(below, top);
            }
        }
    }
    Ok(true)
}

struct NetherWorld<'a, W> {
    inner: &'a mut W,
    lava_cutoff: i32,
    lava: BlockStateId,
    cave_air: BlockStateId,
}

impl<W: CarverWorld> CarverWorld for NetherWorld<'_, W> {
    fn minimum_y(&self) -> i32 {
        self.inner.minimum_y()
    }

    fn generation_depth(&self) -> i32 {
        self.inner.generation_depth()
    }

    fn upgrading_chunk(&self) -> bool {
        self.inner.upgrading_chunk()
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.inner.block_state(position)
    }

    fn is_grass_or_mycelium(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_carver_replaceable(&self, state: BlockStateId) -> bool {
        self.inner.is_carver_replaceable(state)
    }

    fn aquifer_substance(&mut self, position: BlockPos, _density: f64) -> Option<BlockStateId> {
        Some(if position.y <= self.lava_cutoff {
            self.lava
        } else {
            self.cave_air
        })
    }

    fn debug_marker_for(&self, state: BlockStateId) -> BlockStateId {
        state
    }

    fn aquifer_should_schedule_fluid_update(&mut self) -> bool {
        false
    }

    fn has_nonempty_fluid(&self, _state: BlockStateId) -> bool {
        false
    }

    fn offer_carved_block(&mut self, position: BlockPos, state: BlockStateId) {
        self.inner.offer_carved_block(position, state);
    }

    fn mark_for_postprocessing(&mut self, _position: BlockPos) {}

    fn is_dirt(&self, _state: BlockStateId) -> bool {
        false
    }

    fn surface_top_material(
        &mut self,
        _position: BlockPos,
        _stone_depth_above: i32,
        _stone_depth_below: i32,
        _water_height: i32,
    ) -> Option<BlockStateId> {
        None
    }
}

fn validate_ellipsoid(ellipsoid: CarverEllipsoid) -> Result<(), CarverError> {
    if !ellipsoid.center_x.is_finite()
        || !ellipsoid.center_y.is_finite()
        || !ellipsoid.center_z.is_finite()
        || !ellipsoid.horizontal_radius.is_finite()
        || !ellipsoid.vertical_radius.is_finite()
        || ellipsoid.horizontal_radius <= 0.0
        || ellipsoid.vertical_radius <= 0.0
    {
        Err(CarverError::InvalidEllipsoid)
    } else {
        Ok(())
    }
}

fn offset_y(position: BlockPos, y: i32) -> Result<BlockPos, CarverError> {
    Ok(BlockPos::new(
        position.x,
        position
            .y
            .checked_add(y)
            .ok_or(CarverError::PositionOverflow)?,
        position.z,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CarverError {
    #[error("carver ellipsoid must have finite centers and positive finite radii")]
    InvalidEllipsoid,
    #[error("carver position overflow")]
    PositionOverflow,
}
