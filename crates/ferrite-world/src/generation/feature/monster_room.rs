//! Monster-room geometry, construction, chest attempts, and spawner handoff.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

pub trait MonsterRoomWorld {
    fn minimum_y(&self) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_solid(&self, state: BlockStateId) -> bool;

    fn is_empty(&self, state: BlockStateId) -> bool;

    fn is_chest(&self, state: BlockStateId) -> bool;

    fn is_spawner(&self, state: BlockStateId) -> bool;

    fn is_protected_from_features(&self, state: BlockStateId) -> bool;

    fn cave_air(&self) -> BlockStateId;

    fn cobblestone(&self) -> BlockStateId;

    fn mossy_cobblestone(&self) -> BlockStateId;

    fn default_chest(&self) -> BlockStateId;

    fn default_spawner(&self) -> BlockStateId;

    fn reorient_chest(&mut self, position: BlockPos, default_state: BlockStateId) -> BlockStateId;

    fn offer_monster_room(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn initialize_dungeon_loot<R: GenerationRandom>(&mut self, position: BlockPos, random: &mut R);

    fn initialize_spawner<R: GenerationRandom>(&mut self, position: BlockPos, random: &mut R);
}

pub fn place_monster_room<R, W>(
    world: &mut W,
    origin: BlockPos,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, MonsterRoomError>
where
    R: GenerationRandom,
    W: MonsterRoomWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let x_radius =
        2 + random.next_u32(NonZeroU32::new(2).expect("room radius bound is nonzero")) as i32;
    let z_radius =
        2 + random.next_u32(NonZeroU32::new(2).expect("room radius bound is nonzero")) as i32;
    if !validate_room(world, origin, x_radius, z_radius)? {
        return Ok(false);
    }
    construct_room(world, origin, x_radius, z_radius, random)?;
    place_chests(world, origin, x_radius, z_radius, random)?;
    let spawner = world.default_spawner();
    safe_set(world, origin, spawner);
    world.initialize_spawner(origin, random);
    Ok(true)
}

fn validate_room<W: MonsterRoomWorld>(
    world: &mut W,
    origin: BlockPos,
    x_radius: i32,
    z_radius: i32,
) -> Result<bool, MonsterRoomError> {
    let mut openings = 0_u32;
    for x in -x_radius - 1..=x_radius + 1 {
        for y in -1..=4 {
            for z in -z_radius - 1..=z_radius + 1 {
                let position = offset_xyz(origin, x, y, z)?;
                let state = world.block_state(position);
                if (y == -1 || y == 4) && !world.is_solid(state) {
                    return Ok(false);
                }
                let outer_wall = x == -x_radius - 1
                    || x == x_radius + 1
                    || z == -z_radius - 1
                    || z == z_radius + 1;
                if y == 0 && outer_wall {
                    let reread = world.block_state(position);
                    if world.is_empty(reread) {
                        let above = offset_xyz(position, 0, 1, 0)?;
                        let above_state = world.block_state(above);
                        if world.is_empty(above_state) {
                            openings += 1;
                        }
                    }
                }
            }
        }
    }
    Ok((1..=5).contains(&openings))
}

fn construct_room<R, W>(
    world: &mut W,
    origin: BlockPos,
    x_radius: i32,
    z_radius: i32,
    random: &mut R,
) -> Result<(), MonsterRoomError>
where
    R: GenerationRandom,
    W: MonsterRoomWorld,
{
    for x in -x_radius - 1..=x_radius + 1 {
        for y in (-1..=3).rev() {
            for z in -z_radius - 1..=z_radius + 1 {
                let position = offset_xyz(origin, x, y, z)?;
                let state = world.block_state(position);
                let shell = x == -x_radius - 1
                    || x == x_radius + 1
                    || z == -z_radius - 1
                    || z == z_radius + 1
                    || y == -1;
                if shell {
                    if position.y >= world.minimum_y() {
                        let below = offset_xyz(position, 0, -1, 0)?;
                        let below_state = world.block_state(below);
                        if !world.is_solid(below_state) {
                            let _ = world.offer_monster_room(position, world.cave_air(), 2);
                            continue;
                        }
                    }
                    if world.is_solid(state) && !world.is_chest(state) {
                        let requested = if y == -1
                            && random
                                .next_u32(NonZeroU32::new(4).expect("mossy floor bound is nonzero"))
                                != 0
                        {
                            world.mossy_cobblestone()
                        } else {
                            world.cobblestone()
                        };
                        safe_set(world, position, requested);
                    }
                } else if !world.is_chest(state) && !world.is_spawner(state) {
                    safe_set(world, position, world.cave_air());
                }
            }
        }
    }
    Ok(())
}

fn place_chests<R, W>(
    world: &mut W,
    origin: BlockPos,
    x_radius: i32,
    z_radius: i32,
    random: &mut R,
) -> Result<(), MonsterRoomError>
where
    R: GenerationRandom,
    W: MonsterRoomWorld,
{
    for _ in 0..2 {
        for _ in 0..3 {
            let x = sample_symmetric(random, x_radius)?;
            let z = sample_symmetric(random, z_radius)?;
            let candidate = offset_xyz(origin, x, 0, z)?;
            let state = world.block_state(candidate);
            if !world.is_empty(state) {
                continue;
            }
            let mut solid_neighbors = 0_u8;
            for direction in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                let neighbor = offset(candidate, direction)?;
                let neighbor_state = world.block_state(neighbor);
                solid_neighbors += u8::from(world.is_solid(neighbor_state));
            }
            if solid_neighbors != 1 {
                continue;
            }
            let default_chest = world.default_chest();
            let chest = world.reorient_chest(candidate, default_chest);
            safe_set(world, candidate, chest);
            world.initialize_dungeon_loot(candidate, random);
            break;
        }
    }
    Ok(())
}

fn safe_set<W: MonsterRoomWorld>(world: &mut W, position: BlockPos, state: BlockStateId) {
    let current = world.block_state(position);
    if !world.is_protected_from_features(current) {
        let _ = world.offer_monster_room(position, state, 2);
    }
}

fn sample_symmetric(
    random: &mut impl GenerationRandom,
    radius: i32,
) -> Result<i32, MonsterRoomError> {
    let width = radius
        .checked_mul(2)
        .and_then(|value| value.checked_add(1))
        .and_then(|value| u32::try_from(value).ok())
        .and_then(NonZeroU32::new)
        .ok_or(MonsterRoomError::RadiusOverflow)?;
    Ok(random.next_u32(width) as i32 - radius)
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, MonsterRoomError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x, y, z)
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, MonsterRoomError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(MonsterRoomError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(MonsterRoomError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(MonsterRoomError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MonsterRoomError {
    #[error("monster-room position overflow")]
    PositionOverflow,
    #[error("monster-room random radius cannot be represented")]
    RadiusOverflow,
}
