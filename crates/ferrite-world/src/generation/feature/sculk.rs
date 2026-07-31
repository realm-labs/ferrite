//! World-generation sculk patch and charge-cursor runtime.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, PartialEq)]
pub struct SculkPatchConfig {
    pub charge_count: u32,
    pub amount_per_charge: i32,
    pub spread_attempts: u32,
    pub growth_rounds: u32,
    pub spread_rounds: u32,
    pub extra_rare_growths: IntProvider,
    pub catalyst_chance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SculkBehavior {
    Default,
    Sculk,
    Vein,
}

pub trait SculkWorld {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_water_block(&self, state: BlockStateId) -> bool;

    fn fluid_is_source(&mut self, position: BlockPos) -> bool;

    fn has_nonempty_fluid(&mut self, position: BlockPos) -> bool;

    fn has_full_collision(&self, state: BlockStateId, position: BlockPos) -> bool;

    fn is_face_sturdy(&self, state: BlockStateId, position: BlockPos, face: Direction) -> bool;

    fn behavior(&self, state: BlockStateId) -> SculkBehavior;

    fn is_sculk_behavior(&self, state: BlockStateId) -> bool {
        self.behavior(state) != SculkBehavior::Default
    }

    fn available_vein_faces(&self, state: BlockStateId) -> u8;

    fn is_worldgen_replaceable(&self, state: BlockStateId) -> bool;

    fn is_ordinary_sculk_replaceable(&self, state: BlockStateId) -> bool;

    fn is_sensor_or_shrieker(&self, state: BlockStateId) -> bool;

    fn spread_vein_same_space(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        postprocess: bool,
    ) -> u64;

    fn spread_vein_all(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        postprocess: bool,
    ) -> u64;

    fn regrow_vein(&mut self, position: BlockPos, replaced: BlockStateId, faces: u8) -> bool;

    fn discharge_vein(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        random: &mut impl GenerationRandom,
    );

    fn sculk_state(&self) -> BlockStateId;

    fn sensor_state(&self, waterlogged: bool) -> BlockStateId;

    fn shrieker_state(&self, can_summon: bool, waterlogged: bool) -> BlockStateId;

    fn catalyst_state(&self) -> BlockStateId;

    fn offer_sculk(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn push_entities_up(
        &mut self,
        position: BlockPos,
        old_state: BlockStateId,
        new_state: BlockStateId,
    );

    fn play_spread_sound(&mut self, position: BlockPos);

    fn play_placement_sound(&mut self, position: BlockPos, state: BlockStateId);

    fn level_event_3006(&mut self, position: BlockPos, data: i32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChargeCursor {
    position: BlockPos,
    charge: i32,
    update_delay: u8,
    decay_delay: i32,
    faces: Option<u8>,
}

impl ChargeCursor {
    fn new(position: BlockPos, charge: i32) -> Self {
        Self {
            position,
            charge,
            update_delay: 0,
            decay_delay: 1,
            faces: None,
        }
    }
}

pub fn place_sculk_patch<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &SculkPatchConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, SculkError>
where
    R: GenerationRandom,
    W: SculkWorld,
{
    if !ensure_can_write(origin) || !can_spread_from(world, origin)? {
        return Ok(false);
    }
    validate_config(config)?;
    let total_rounds = config
        .spread_rounds
        .checked_add(config.growth_rounds)
        .ok_or(SculkError::InvalidConfiguration)?;
    let mut cursors = Vec::new();
    for round in 0..total_rounds {
        add_cursors(
            &mut cursors,
            origin,
            config.charge_count,
            config.amount_per_charge,
        );
        let growth_enabled = round < config.spread_rounds;
        for _ in 0..config.spread_attempts {
            update_cursors(world, origin, &mut cursors, growth_enabled, random)?;
        }
        cursors.clear();
    }

    let below = offset(origin, Direction::Down)?;
    if random.next_f32() <= config.catalyst_chance {
        let state = world.block_state(below);
        if world.has_full_collision(state, below) {
            let catalyst = world.catalyst_state();
            let _ = world.offer_sculk(origin, catalyst, 3);
        }
    }
    let extra = config.extra_rare_growths.sample(random)?;
    for _ in 0..extra {
        let x = bounded(random, 5) - 2;
        let z = bounded(random, 5) - 2;
        let candidate = offset_xyz(origin, x, 0, z)?;
        let candidate_state = world.block_state(candidate);
        if !world.is_air(candidate_state) {
            continue;
        }
        let support = offset(candidate, Direction::Down)?;
        let support_state = world.block_state(support);
        if world.is_face_sturdy(support_state, support, Direction::Up) {
            let shrieker = world.shrieker_state(true, false);
            let _ = world.offer_sculk(candidate, shrieker, 3);
        }
    }
    Ok(true)
}

fn update_cursors<R, W>(
    world: &mut W,
    origin: BlockPos,
    cursors: &mut Vec<ChargeCursor>,
    growth_enabled: bool,
    random: &mut R,
) -> Result<(), SculkError>
where
    R: GenerationRandom,
    W: SculkWorld,
{
    let mut retained = Vec::new();
    for mut cursor in cursors.drain(..) {
        if chebyshev(cursor.position, origin) > 1_024 {
            continue;
        }
        update_cursor(world, origin, &mut cursor, growth_enabled, random)?;
        if cursor.charge <= 0 {
            world.level_event_3006(cursor.position, 0);
        } else {
            retained.push(cursor);
        }
    }
    emit_charge_events(world, &retained);
    *cursors = retained;
    Ok(())
}

fn update_cursor<R, W>(
    world: &mut W,
    origin: BlockPos,
    cursor: &mut ChargeCursor,
    growth_enabled: bool,
    random: &mut R,
) -> Result<(), SculkError>
where
    R: GenerationRandom,
    W: SculkWorld,
{
    if cursor.update_delay > 0 {
        cursor.update_delay -= 1;
        return Ok(());
    }
    let mut state = world.block_state(cursor.position);
    let mut behavior = world.behavior(state);
    if growth_enabled && attempt_spread_vein(world, cursor.position, state, cursor.faces, behavior)
    {
        if behavior != SculkBehavior::Sculk {
            state = world.block_state(cursor.position);
            behavior = world.behavior(state);
        }
        world.play_spread_sound(cursor.position);
    }
    cursor.charge = use_charge(
        world,
        origin,
        cursor,
        state,
        behavior,
        growth_enabled,
        random,
    )?;
    if cursor.charge <= 0 {
        discharge(world, cursor.position, state, behavior, random);
        return Ok(());
    }
    if let Some(destination) = movement_destination(world, cursor.position, random)? {
        discharge(world, cursor.position, state, behavior, random);
        cursor.position = destination;
        if horizontal_distance(cursor.position, origin) >= 15.0 {
            cursor.charge = 0;
            return Ok(());
        }
        state = world.block_state(destination);
    }
    if world.is_sculk_behavior(state) {
        cursor.faces = Some(world.available_vein_faces(state));
    }
    cursor.decay_delay = match behavior {
        SculkBehavior::Default => (cursor.decay_delay - 1).max(0),
        SculkBehavior::Sculk | SculkBehavior::Vein => 1,
    };
    cursor.update_delay = 1;
    Ok(())
}

fn attempt_spread_vein(
    world: &mut impl SculkWorld,
    position: BlockPos,
    state: BlockStateId,
    faces: Option<u8>,
    behavior: SculkBehavior,
) -> bool {
    if behavior == SculkBehavior::Default {
        match faces {
            None => world.spread_vein_same_space(position, state, true) > 0,
            Some(faces) if faces != 0 && (world.is_air(state) || world.is_water_block(state)) => {
                world.regrow_vein(position, state, faces)
            }
            Some(faces) if faces != 0 => false,
            Some(_) => world.spread_vein_all(position, state, true) > 0,
        }
    } else {
        world.spread_vein_all(position, state, true) > 0
    }
}

fn use_charge<R, W>(
    world: &mut W,
    origin: BlockPos,
    cursor: &mut ChargeCursor,
    state: BlockStateId,
    behavior: SculkBehavior,
    growth_enabled: bool,
    random: &mut R,
) -> Result<i32, SculkError>
where
    R: GenerationRandom,
    W: SculkWorld,
{
    match behavior {
        SculkBehavior::Default => Ok(if cursor.decay_delay > 0 {
            cursor.charge
        } else {
            0
        }),
        SculkBehavior::Sculk => use_sculk_charge(world, origin, cursor, random),
        SculkBehavior::Vein => {
            if growth_enabled && convert_vein(world, cursor.position, state, random)? {
                Ok(cursor.charge - 1)
            } else if bounded(random, 5) == 0 {
                Ok((cursor.charge as f32 * 0.5).floor() as i32)
            } else {
                Ok(cursor.charge)
            }
        }
    }
}

fn use_sculk_charge(
    world: &mut impl SculkWorld,
    origin: BlockPos,
    cursor: &ChargeCursor,
    random: &mut impl GenerationRandom,
) -> Result<i32, SculkError> {
    let charge = cursor.charge;
    if charge == 0 || bounded(random, 5) != 0 {
        return Ok(charge);
    }
    let close = euclidean_distance(cursor.position, origin) < 1.0;
    if close || !can_place_growth(world, cursor.position)? {
        if bounded(random, 10) != 0 {
            return Ok(charge);
        }
        if close {
            return Ok(charge - 1);
        }
        let outer = (euclidean_distance(cursor.position, origin) as f32 - 1.0).powi(2);
        let factor = 1.0_f32.min(outer / 23_f32.powi(2));
        let penalty = 1.max((charge as f32 * factor * 0.5) as i32);
        return Ok(charge - penalty);
    }
    if bounded(random, 50) < charge {
        let growth_position = offset(cursor.position, Direction::Up)?;
        let shrieker = bounded(random, 11) == 0;
        let waterlogged = world.has_nonempty_fluid(growth_position);
        let growth = if shrieker {
            world.shrieker_state(true, waterlogged)
        } else {
            world.sensor_state(waterlogged)
        };
        let _ = world.offer_sculk(growth_position, growth, 3);
        world.play_placement_sound(cursor.position, growth);
    }
    Ok((charge - 50).max(0))
}

fn can_place_growth(world: &mut impl SculkWorld, position: BlockPos) -> Result<bool, SculkError> {
    let above = offset(position, Direction::Up)?;
    let state = world.block_state(above);
    if !(world.is_air(state) || world.is_water_block(state) && world.has_nonempty_fluid(above)) {
        return Ok(false);
    }
    let mut found = 0;
    for z in -4..=4 {
        for y in 0..=2 {
            for x in -4..=4 {
                let candidate = offset_xyz(above, x, y, z)?;
                let candidate_state = world.block_state(candidate);
                if world.is_sensor_or_shrieker(candidate_state) {
                    found += 1;
                    if found > 2 {
                        return Ok(false);
                    }
                }
            }
        }
    }
    Ok(true)
}

fn convert_vein(
    world: &mut impl SculkWorld,
    position: BlockPos,
    vein_state: BlockStateId,
    random: &mut impl GenerationRandom,
) -> Result<bool, SculkError> {
    let mut directions = Direction::ALL.to_vec();
    shuffle(&mut directions, random);
    let faces = world.available_vein_faces(vein_state);
    for direction in directions {
        if faces & face_bit(direction) == 0 {
            continue;
        }
        let support = offset(position, direction)?;
        let old_state = world.block_state(support);
        if !world.is_worldgen_replaceable(old_state) {
            continue;
        }
        let sculk = world.sculk_state();
        let _ = world.offer_sculk(support, sculk, 3);
        world.push_entities_up(support, old_state, sculk);
        world.play_spread_sound(support);
        let _ = world.spread_vein_all(support, sculk, true);
        let skip = direction.opposite();
        for neighbor_direction in Direction::ALL {
            if neighbor_direction == skip {
                continue;
            }
            let neighbor = offset(support, neighbor_direction)?;
            let state = world.block_state(neighbor);
            if world.behavior(state) == SculkBehavior::Vein {
                world.discharge_vein(neighbor, state, random);
            }
        }
        return Ok(true);
    }
    Ok(false)
}

fn movement_destination(
    world: &mut impl SculkWorld,
    position: BlockPos,
    random: &mut impl GenerationRandom,
) -> Result<Option<BlockPos>, SculkError> {
    let mut offsets = noncorner_offsets();
    shuffle(&mut offsets, random);
    for [x, y, z] in offsets {
        let candidate = offset_xyz(position, x, y, z)?;
        let state = world.block_state(candidate);
        if world.behavior(state) != SculkBehavior::Vein
            || !movement_unobstructed(world, position, [x, y, z])?
        {
            continue;
        }
        let faces = world.available_vein_faces(state);
        let mut access = false;
        for direction in Direction::ALL {
            if faces & face_bit(direction) == 0 {
                continue;
            }
            let support = offset(candidate, direction)?;
            let support_state = world.block_state(support);
            if world.is_ordinary_sculk_replaceable(support_state) {
                access = true;
                break;
            }
        }
        if access {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

fn movement_unobstructed(
    world: &mut impl SculkWorld,
    from: BlockPos,
    delta: [i32; 3],
) -> Result<bool, SculkError> {
    if delta[0].abs() + delta[1].abs() + delta[2].abs() == 1 {
        return Ok(true);
    }
    let x = axis_direction(Direction::West, Direction::East, delta[0]);
    let y = axis_direction(Direction::Down, Direction::Up, delta[1]);
    let z = axis_direction(Direction::North, Direction::South, delta[2]);
    let pair = if delta[0] == 0 {
        [y, z]
    } else if delta[1] == 0 {
        [x, z]
    } else {
        [x, y]
    };
    Ok(is_unobstructed(world, from, pair[0])? || is_unobstructed(world, from, pair[1])?)
}

fn is_unobstructed(
    world: &mut impl SculkWorld,
    from: BlockPos,
    direction: Direction,
) -> Result<bool, SculkError> {
    let position = offset(from, direction)?;
    let state = world.block_state(position);
    Ok(!world.is_face_sturdy(state, position, direction.opposite()))
}

fn discharge(
    world: &mut impl SculkWorld,
    position: BlockPos,
    state: BlockStateId,
    behavior: SculkBehavior,
    random: &mut impl GenerationRandom,
) {
    if behavior == SculkBehavior::Vein {
        world.discharge_vein(position, state, random);
    }
}

fn emit_charge_events(world: &mut impl SculkWorld, cursors: &[ChargeCursor]) {
    let mut groups: Vec<(BlockPos, i32, u8, i32)> = Vec::new();
    for cursor in cursors {
        if let Some(group) = groups
            .iter_mut()
            .find(|(position, _, _, _)| *position == cursor.position)
        {
            group.1 += cursor.charge;
            if cursor.charge < group.3 {
                group.2 = cursor.faces.unwrap_or(0);
                group.3 = cursor.charge;
            }
        } else {
            groups.push((
                cursor.position,
                cursor.charge,
                cursor.faces.unwrap_or(0),
                cursor.charge,
            ));
        }
    }
    let group_count = groups.len();
    groups.sort_by_key(|(position, _, _, _)| {
        std::cmp::Reverse(fastutil_slot(*position, group_count))
    });
    for (position, charge, faces, _) in groups {
        if faces == 0 {
            continue;
        }
        let intensity =
            ((f64::from(charge).ln_1p() / 2.299_999_952_316_284).floor() as i32 + 1) << 6;
        world.level_event_3006(position, intensity + i32::from(faces));
    }
}

fn fastutil_slot(position: BlockPos, entries: usize) -> usize {
    let capacity = if entries > 24 { 64 } else { 32 };
    let hash = position
        .z
        .wrapping_mul(31)
        .wrapping_add(position.y)
        .wrapping_mul(31)
        .wrapping_add(position.x) as u32;
    let mixed = hash.wrapping_mul(0x9e37_79b9) ^ hash.wrapping_mul(0x9e37_79b9).wrapping_shr(16);
    mixed as usize & (capacity - 1)
}

fn add_cursors(cursors: &mut Vec<ChargeCursor>, position: BlockPos, count: u32, amount: i32) {
    for _ in 0..count {
        if cursors.len() >= 32 {
            break;
        }
        let mut remaining = amount;
        while remaining > 0 && cursors.len() < 32 {
            let charge = remaining.min(1_000);
            cursors.push(ChargeCursor::new(position, charge));
            remaining -= charge;
        }
    }
}

fn can_spread_from(world: &mut impl SculkWorld, origin: BlockPos) -> Result<bool, SculkError> {
    let state = world.block_state(origin);
    if world.is_sculk_behavior(state) {
        return Ok(true);
    }
    if !(world.is_air(state) || world.is_water_block(state) && world.fluid_is_source(origin)) {
        return Ok(false);
    }
    for direction in Direction::ALL {
        let neighbor = offset(origin, direction)?;
        let state = world.block_state(neighbor);
        if world.has_full_collision(state, neighbor) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn validate_config(config: &SculkPatchConfig) -> Result<(), SculkError> {
    if !(1..=32).contains(&config.charge_count)
        || !(1..=500).contains(&config.amount_per_charge)
        || !(1..=64).contains(&config.spread_attempts)
        || config.growth_rounds > 8
        || config.spread_rounds > 8
        || !(0.0..=1.0).contains(&config.catalyst_chance)
    {
        Err(SculkError::InvalidConfiguration)
    } else {
        Ok(())
    }
}

fn noncorner_offsets() -> Vec<[i32; 3]> {
    let mut offsets = Vec::with_capacity(18);
    for z in -1..=1 {
        for y in -1..=1 {
            for x in -1..=1 {
                if (x == 0 || y == 0 || z == 0) && (x != 0 || y != 0 || z != 0) {
                    offsets.push([x, y, z]);
                }
            }
        }
    }
    offsets
}

fn shuffle<T>(values: &mut [T], random: &mut impl GenerationRandom) {
    for length in (2..=values.len()).rev() {
        let index = random
            .next_u32(NonZeroU32::new(length as u32).expect("shuffle bound is nonzero"))
            as usize;
        values.swap(length - 1, index);
    }
}

fn face_bit(direction: Direction) -> u8 {
    1 << direction_index(direction)
}

fn direction_index(direction: Direction) -> u8 {
    match direction {
        Direction::Down => 0,
        Direction::Up => 1,
        Direction::North => 2,
        Direction::South => 3,
        Direction::West => 4,
        Direction::East => 5,
    }
}

fn axis_direction(negative: Direction, positive: Direction, delta: i32) -> Direction {
    if delta < 0 { negative } else { positive }
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> i32 {
    random.next_u32(NonZeroU32::new(bound).expect("sculk bound is nonzero")) as i32
}

fn chebyshev(left: BlockPos, right: BlockPos) -> i32 {
    left.x
        .abs_diff(right.x)
        .max(left.y.abs_diff(right.y))
        .max(left.z.abs_diff(right.z)) as i32
}

fn euclidean_distance(left: BlockPos, right: BlockPos) -> f64 {
    let x = f64::from(left.x) - f64::from(right.x);
    let y = f64::from(left.y) - f64::from(right.y);
    let z = f64::from(left.z) - f64::from(right.z);
    (x * x + y * y + z * z).sqrt()
}

fn horizontal_distance(left: BlockPos, right: BlockPos) -> f64 {
    let x = f64::from(left.x) - f64::from(right.x);
    let z = f64::from(left.z) - f64::from(right.z);
    (x * x + z * z).sqrt()
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, SculkError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x, y, z)
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, SculkError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(SculkError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(SculkError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(SculkError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SculkError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("sculk-patch configuration violates codec bounds")]
    InvalidConfiguration,
    #[error("sculk-patch position overflow")]
    PositionOverflow,
}
