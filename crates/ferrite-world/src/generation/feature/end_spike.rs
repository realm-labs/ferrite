//! End-spike layout derivation and source-ordered spike placement.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndSpike {
    pub center_x: i32,
    pub center_z: i32,
    pub radius: i32,
    pub height: i32,
    pub guarded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndSpikeConfig {
    pub spikes: Vec<EndSpike>,
    pub crystal_invulnerable: bool,
    pub beam_target: Option<BlockPos>,
    pub obsidian: BlockStateId,
    pub air: BlockStateId,
    pub iron_bars: BlockStateId,
    pub bedrock: BlockStateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IronBarConnections {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

pub trait EndSpikeWorld {
    fn world_seed(&self) -> i64;

    fn minimum_y(&self) -> i32;

    fn configure_iron_bars(
        &mut self,
        default_state: BlockStateId,
        connections: IronBarConnections,
    ) -> BlockStateId;

    fn can_create_end_crystal(&mut self) -> bool;

    fn add_end_crystal(
        &mut self,
        position: [f64; 3],
        yaw_degrees: f32,
        beam_target: Option<BlockPos>,
        invulnerable: bool,
    ) -> bool;

    fn fire_state(&mut self, position: BlockPos) -> BlockStateId;

    fn offer_end_spike_block(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        flags: u32,
    ) -> bool;
}

pub fn place_end_spikes<R, W>(
    world: &mut W,
    origin: BlockPos,
    config: &EndSpikeConfig,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, EndSpikeError>
where
    R: GenerationRandom,
    W: EndSpikeWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let derived;
    let spikes = if config.spikes.is_empty() {
        derived = derive_end_spikes(world.world_seed());
        derived.as_slice()
    } else {
        config.spikes.as_slice()
    };
    for spike in spikes {
        if section_coordinate(spike.center_x) != section_coordinate(origin.x)
            || section_coordinate(spike.center_z) != section_coordinate(origin.z)
        {
            continue;
        }
        place_spike(world, *spike, config, random)?;
    }
    Ok(true)
}

#[must_use]
pub fn derive_end_spikes(world_seed: i64) -> Vec<EndSpike> {
    let mut seed_source = LegacyRandom::new(world_seed);
    let cache_key = seed_source.next_i64() & 65_535;
    let mut layout_source = LegacyRandom::new(cache_key);
    let mut values = [0_i32, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    for remaining in (2..=values.len()).rev() {
        let index = layout_source.next_bounded(remaining as i32) as usize;
        values.swap(remaining - 1, index);
    }
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let angle = -std::f64::consts::PI + index as f64 * std::f64::consts::PI / 10.0;
            EndSpike {
                center_x: (84.0 * angle.cos()).floor() as i32,
                center_z: (84.0 * angle.sin()).floor() as i32,
                radius: 2 + value / 3,
                height: 76 + 3 * value,
                guarded: value == 1 || value == 2,
            }
        })
        .collect()
}

fn place_spike<R, W>(
    world: &mut W,
    spike: EndSpike,
    config: &EndSpikeConfig,
    random: &mut R,
) -> Result<(), EndSpikeError>
where
    R: GenerationRandom,
    W: EndSpikeWorld,
{
    let minimum_x = spike
        .center_x
        .checked_sub(spike.radius)
        .ok_or(EndSpikeError::PositionOverflow)?;
    let maximum_x = spike
        .center_x
        .checked_add(spike.radius)
        .ok_or(EndSpikeError::PositionOverflow)?;
    let minimum_z = spike
        .center_z
        .checked_sub(spike.radius)
        .ok_or(EndSpikeError::PositionOverflow)?;
    let maximum_z = spike
        .center_z
        .checked_add(spike.radius)
        .ok_or(EndSpikeError::PositionOverflow)?;
    let maximum_y = spike
        .height
        .checked_add(10)
        .ok_or(EndSpikeError::PositionOverflow)?;
    let radius_squared = i64::from(spike.radius)
        .checked_mul(i64::from(spike.radius))
        .and_then(|value| value.checked_add(1))
        .ok_or(EndSpikeError::PositionOverflow)?;
    for z in minimum_z..=maximum_z {
        for y in world.minimum_y()..=maximum_y {
            for x in minimum_x..=maximum_x {
                let dx = i64::from(x) - i64::from(spike.center_x);
                let dz = i64::from(z) - i64::from(spike.center_z);
                let inside = dx * dx + dz * dz <= radius_squared;
                let state = if inside && y < spike.height {
                    Some(config.obsidian)
                } else if y > 65 {
                    Some(config.air)
                } else {
                    None
                };
                if let Some(state) = state {
                    let _ = world.offer_end_spike_block(BlockPos::new(x, y, z), state, 3);
                }
            }
        }
    }
    if spike.guarded {
        place_guard(world, spike, config.iron_bars)?;
    }
    if !world.can_create_end_crystal() {
        return Ok(());
    }
    let yaw = random.next_f32() * 360.0;
    let crystal_block = BlockPos::new(
        spike.center_x,
        spike
            .height
            .checked_add(1)
            .ok_or(EndSpikeError::PositionOverflow)?,
        spike.center_z,
    );
    let _ = world.add_end_crystal(
        [
            f64::from(spike.center_x) + 0.5,
            f64::from(crystal_block.y),
            f64::from(spike.center_z) + 0.5,
        ],
        yaw,
        config.beam_target,
        config.crystal_invulnerable,
    );
    let bedrock = BlockPos::new(spike.center_x, spike.height, spike.center_z);
    let _ = world.offer_end_spike_block(bedrock, config.bedrock, 3);
    let fire = world.fire_state(crystal_block);
    let _ = world.offer_end_spike_block(crystal_block, fire, 3);
    Ok(())
}

fn place_guard<W: EndSpikeWorld>(
    world: &mut W,
    spike: EndSpike,
    iron_bars: BlockStateId,
) -> Result<(), EndSpikeError> {
    for y_offset in 0..=3 {
        for x_offset in -2_i32..=2 {
            for z_offset in -2_i32..=2 {
                let x_wall = x_offset.abs() == 2;
                let z_wall = z_offset.abs() == 2;
                let roof = y_offset == 3;
                if !x_wall && !z_wall && !roof {
                    continue;
                }
                let state = world.configure_iron_bars(
                    iron_bars,
                    IronBarConnections {
                        north: (x_wall || roof) && z_offset != -2,
                        east: (z_wall || roof) && x_offset != 2,
                        south: (x_wall || roof) && z_offset != 2,
                        west: (z_wall || roof) && x_offset != -2,
                    },
                );
                let position = BlockPos::new(
                    spike
                        .center_x
                        .checked_add(x_offset)
                        .ok_or(EndSpikeError::PositionOverflow)?,
                    spike
                        .height
                        .checked_add(y_offset)
                        .ok_or(EndSpikeError::PositionOverflow)?,
                    spike
                        .center_z
                        .checked_add(z_offset)
                        .ok_or(EndSpikeError::PositionOverflow)?,
                );
                let _ = world.offer_end_spike_block(position, state, 3);
            }
        }
    }
    Ok(())
}

const fn section_coordinate(block: i32) -> i32 {
    block >> 4
}

#[derive(Debug, Clone, Copy)]
struct LegacyRandom {
    seed: u64,
}

impl LegacyRandom {
    const MULTIPLIER: u64 = 0x5deece66d;
    const ADDEND: u64 = 0xb;
    const MASK: u64 = (1_u64 << 48) - 1;

    fn new(seed: i64) -> Self {
        Self {
            seed: (seed as u64 ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next_bits(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as u32 as i32
    }

    fn next_i64(&mut self) -> i64 {
        (i64::from(self.next_bits(32)) << 32).wrapping_add(i64::from(self.next_bits(32)))
    }

    fn next_bounded(&mut self, bound: i32) -> i32 {
        debug_assert!(bound > 0);
        if bound & -bound == bound {
            return ((i64::from(bound) * i64::from(self.next_bits(31))) >> 31) as i32;
        }
        loop {
            let bits = self.next_bits(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EndSpikeError {
    #[error("End-spike position arithmetic overflowed")]
    PositionOverflow,
}
