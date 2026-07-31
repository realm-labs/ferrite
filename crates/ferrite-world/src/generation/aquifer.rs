//! Disabled and noise-enabled aquifer material resolution.

use std::collections::HashMap;

use ferrite_foundation::coordinate::BlockPos;

use crate::id::BlockStateId;

const FLOW_UPDATE_SIMILARITY: f64 = -0.76;
const SURFACE_OFFSETS: [[i32; 2]; 13] = [
    [0, 0],
    [-2, -1],
    [-1, -1],
    [0, -1],
    [1, -1],
    [-3, 0],
    [-2, 0],
    [-1, 0],
    [1, 0],
    [-2, 1],
    [-1, 1],
    [0, 1],
    [1, 1],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidStatus {
    pub level: i32,
    pub state: BlockStateId,
}

impl FluidStatus {
    pub fn at(self, y: i32, air: BlockStateId) -> BlockStateId {
        if y < self.level { self.state } else { air }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AquiferStates {
    pub air: BlockStateId,
    pub water: BlockStateId,
    pub lava: BlockStateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalFluidPicker {
    pub sea_level: i32,
    pub default_fluid: BlockStateId,
    pub states: AquiferStates,
}

impl GlobalFluidPicker {
    pub fn status(self, position: BlockPos) -> FluidStatus {
        let lava_level = self.sea_level.min(-54);
        if position.y < lava_level {
            FluidStatus {
                level: -54,
                state: self.states.lava,
            }
        } else {
            FluidStatus {
                level: self.sea_level,
                state: self.default_fluid,
            }
        }
    }
}

pub trait AquiferEnvironment {
    fn global_fluid(&mut self, position: BlockPos) -> FluidStatus;

    fn preliminary_surface(&mut self, x: i32, z: i32) -> i32;

    fn center_offsets(&mut self, cell: [i32; 3]) -> [i32; 3];

    fn erosion(&mut self, position: BlockPos) -> f64;

    fn depth(&mut self, position: BlockPos) -> f64;

    fn floodedness(&mut self, position: BlockPos) -> f64;

    fn spread(&mut self, cell: [i32; 3]) -> f64;

    fn lava(&mut self, cell: [i32; 3]) -> f64;

    fn barrier(&mut self, position: BlockPos) -> f64;
}

pub trait AquiferResolver {
    fn compute_substance(&mut self, position: BlockPos, density: f64) -> Option<BlockStateId>;

    fn should_schedule_fluid_update(&self) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct DisabledAquifer {
    global: GlobalFluidPicker,
    schedule_update: bool,
}

impl DisabledAquifer {
    pub fn new(global: GlobalFluidPicker) -> Self {
        Self {
            global,
            schedule_update: false,
        }
    }
}

impl AquiferResolver for DisabledAquifer {
    fn compute_substance(&mut self, position: BlockPos, density: f64) -> Option<BlockStateId> {
        self.schedule_update = false;
        if density > 0.0 {
            None
        } else {
            Some(
                self.global
                    .status(position)
                    .at(position.y, self.global.states.air),
            )
        }
    }

    fn should_schedule_fluid_update(&self) -> bool {
        self.schedule_update
    }
}

pub struct EnabledAquifer<E> {
    environment: E,
    states: AquiferStates,
    way_below_minimum_y: i32,
    disable_fluid_generation: bool,
    skip_sampling_above_y: i32,
    center_cache: HashMap<[i32; 3], BlockPos>,
    status_cache: HashMap<[i32; 3], FluidStatus>,
    schedule_update: bool,
}

impl<E> EnabledAquifer<E>
where
    E: AquiferEnvironment,
{
    pub fn new(
        environment: E,
        states: AquiferStates,
        maximum_preliminary_surface: i32,
        way_below_minimum_y: i32,
        disable_fluid_generation: bool,
    ) -> Self {
        let adjusted = maximum_preliminary_surface.wrapping_add(8);
        let skip_grid_y = adjusted.wrapping_add(12).div_euclid(12).wrapping_add(1);
        let skip_sampling_above_y = skip_grid_y
            .wrapping_mul(12)
            .wrapping_add(11)
            .wrapping_sub(1);
        Self {
            environment,
            states,
            way_below_minimum_y,
            disable_fluid_generation,
            skip_sampling_above_y,
            center_cache: HashMap::new(),
            status_cache: HashMap::new(),
            schedule_update: false,
        }
    }

    pub fn environment(&self) -> &E {
        &self.environment
    }

    pub fn environment_mut(&mut self) -> &mut E {
        &mut self.environment
    }

    pub fn skip_sampling_above_y(&self) -> i32 {
        self.skip_sampling_above_y
    }

    fn nearest_cells(&mut self, position: BlockPos) -> [([i32; 3], i32); 4] {
        let anchor = [
            position.x.wrapping_sub(5) >> 4,
            position.y.wrapping_add(1).div_euclid(12),
            position.z.wrapping_sub(5) >> 4,
        ];
        let mut nearest = [([0; 3], i32::MAX); 4];
        for x_offset in 0..=1 {
            for y_offset in -1..=1 {
                for z_offset in 0..=1 {
                    let cell = [
                        anchor[0].wrapping_add(x_offset),
                        anchor[1].wrapping_add(y_offset),
                        anchor[2].wrapping_add(z_offset),
                    ];
                    let center = self.center(cell);
                    let x = center.x.wrapping_sub(position.x);
                    let y = center.y.wrapping_sub(position.y);
                    let z = center.z.wrapping_sub(position.z);
                    let distance = x
                        .wrapping_mul(x)
                        .wrapping_add(y.wrapping_mul(y))
                        .wrapping_add(z.wrapping_mul(z));
                    insert_nonstrict(&mut nearest, (cell, distance));
                }
            }
        }
        nearest
    }

    fn center(&mut self, cell: [i32; 3]) -> BlockPos {
        if let Some(center) = self.center_cache.get(&cell) {
            return *center;
        }
        let offsets = self.environment.center_offsets(cell);
        let center = BlockPos::new(
            cell[0].wrapping_mul(16).wrapping_add(offsets[0]),
            cell[1].wrapping_mul(12).wrapping_add(offsets[1]),
            cell[2].wrapping_mul(16).wrapping_add(offsets[2]),
        );
        self.center_cache.insert(cell, center);
        center
    }

    fn status(&mut self, cell: [i32; 3]) -> FluidStatus {
        if let Some(status) = self.status_cache.get(&cell) {
            return *status;
        }
        let center = self.center(cell);
        let status = self.compute_status(center);
        self.status_cache.insert(cell, status);
        status
    }

    fn compute_status(&mut self, center: BlockPos) -> FluidStatus {
        let global = self.environment.global_fluid(center);
        let mut lowest_surface = i32::MAX;
        let top = center.y.wrapping_add(12);
        let bottom = center.y.wrapping_sub(12);
        let mut center_surface_has_fluid = false;
        for [chunk_x, chunk_z] in SURFACE_OFFSETS {
            let sample = BlockPos::new(
                center.x.wrapping_add(chunk_x.wrapping_mul(16)),
                center.y,
                center.z.wrapping_add(chunk_z.wrapping_mul(16)),
            );
            let surface = self.environment.preliminary_surface(sample.x, sample.z);
            let adjusted = surface.wrapping_add(8);
            let is_center = chunk_x == 0 && chunk_z == 0;
            if is_center && bottom > adjusted {
                return global;
            }
            let pokes_above = top > adjusted;
            if pokes_above || is_center {
                let surface_position = BlockPos::new(sample.x, adjusted, sample.z);
                let surface_fluid = self.environment.global_fluid(surface_position);
                let at_surface = surface_fluid.at(adjusted, self.states.air);
                if at_surface != self.states.air {
                    if is_center {
                        center_surface_has_fluid = true;
                    }
                    if pokes_above {
                        return surface_fluid;
                    }
                }
            }
            lowest_surface = lowest_surface.min(surface);
        }
        let level = self.local_level(center, global, lowest_surface, center_surface_has_fluid);
        let state = self.local_fluid_type(center, global, level);
        FluidStatus { level, state }
    }

    fn local_level(
        &mut self,
        center: BlockPos,
        global: FluidStatus,
        lowest_surface: i32,
        center_surface_has_fluid: bool,
    ) -> i32 {
        if self.environment.erosion(center) < f64::from(-0.225_f32)
            && self.environment.depth(center) > f64::from(0.9_f32)
        {
            return self.way_below_minimum_y;
        }
        let distance = lowest_surface.wrapping_add(8).wrapping_sub(center.y);
        let factor = if center_surface_has_fluid {
            clamped_map(f64::from(distance), 0.0, 64.0, 1.0, 0.0)
        } else {
            0.0
        };
        let floodedness = self.environment.floodedness(center).clamp(-1.0, 1.0);
        let fully_flooded = map(factor, 1.0, 0.0, -0.3, 0.8);
        let partially_flooded = map(factor, 1.0, 0.0, -0.8, 0.4);
        if floodedness > fully_flooded {
            global.level
        } else if floodedness > partially_flooded {
            let cell = [
                center.x.div_euclid(16),
                center.y.div_euclid(40),
                center.z.div_euclid(16),
            ];
            let spread = self.environment.spread(cell) * 10.0;
            let quantized = (spread / 3.0).floor() as i32 * 3;
            lowest_surface.min(
                cell[1]
                    .wrapping_mul(40)
                    .wrapping_add(20)
                    .wrapping_add(quantized),
            )
        } else {
            self.way_below_minimum_y
        }
    }

    fn local_fluid_type(
        &mut self,
        center: BlockPos,
        global: FluidStatus,
        level: i32,
    ) -> BlockStateId {
        if level <= -10 && level != self.way_below_minimum_y && global.state != self.states.lava {
            let cell = [
                center.x.div_euclid(64),
                center.y.div_euclid(40),
                center.z.div_euclid(64),
            ];
            if self.environment.lava(cell).abs() > 0.3 {
                return self.states.lava;
            }
        }
        global.state
    }

    fn output_state(&self, state: BlockStateId) -> BlockStateId {
        if self.disable_fluid_generation {
            self.states.air
        } else {
            state
        }
    }
}

impl<E> AquiferResolver for EnabledAquifer<E>
where
    E: AquiferEnvironment,
{
    fn compute_substance(&mut self, position: BlockPos, density: f64) -> Option<BlockStateId> {
        if density > 0.0 {
            self.schedule_update = false;
            return None;
        }
        let global = self.environment.global_fluid(position);
        if position.y > self.skip_sampling_above_y {
            self.schedule_update = false;
            return Some(global.at(position.y, self.states.air));
        }
        if global.at(position.y, self.states.air) == self.states.lava {
            self.schedule_update = false;
            return Some(self.output_state(self.states.lava));
        }

        let nearest = self.nearest_cells(position);
        let first = self.status(nearest[0].0);
        let similarity12 = similarity(nearest[0].1, nearest[1].1);
        let fluid = first.at(position.y, self.states.air);
        let output = self.output_state(fluid);
        if similarity12 <= 0.0 {
            self.schedule_update = if similarity12 >= FLOW_UPDATE_SIMILARITY {
                first != self.status(nearest[1].0)
            } else {
                false
            };
            return Some(output);
        }
        let below = BlockPos::new(position.x, position.y.wrapping_sub(1), position.z);
        if fluid == self.states.water
            && self
                .environment
                .global_fluid(below)
                .at(below.y, self.states.air)
                == self.states.lava
        {
            self.schedule_update = true;
            return Some(output);
        }

        let second = self.status(nearest[1].0);
        let mut barrier = None;
        let pressure12 = similarity12
            * pressure(
                &mut self.environment,
                self.states,
                position,
                first,
                second,
                &mut barrier,
            );
        if density + pressure12 > 0.0 {
            self.schedule_update = false;
            return None;
        }
        let third = self.status(nearest[2].0);
        let similarity13 = similarity(nearest[0].1, nearest[2].1);
        if similarity13 > 0.0
            && density
                + similarity12
                    * similarity13
                    * pressure(
                        &mut self.environment,
                        self.states,
                        position,
                        first,
                        third,
                        &mut barrier,
                    )
                > 0.0
        {
            self.schedule_update = false;
            return None;
        }
        let similarity23 = similarity(nearest[1].1, nearest[2].1);
        if similarity23 > 0.0
            && density
                + similarity12
                    * similarity23
                    * pressure(
                        &mut self.environment,
                        self.states,
                        position,
                        second,
                        third,
                        &mut barrier,
                    )
                > 0.0
        {
            self.schedule_update = false;
            return None;
        }
        self.schedule_update = first != second
            || similarity23 >= FLOW_UPDATE_SIMILARITY && second != third
            || similarity13 >= FLOW_UPDATE_SIMILARITY && first != third
            || similarity13 >= FLOW_UPDATE_SIMILARITY
                && similarity(nearest[0].1, nearest[3].1) >= FLOW_UPDATE_SIMILARITY
                && first != self.status(nearest[3].0);
        Some(output)
    }

    fn should_schedule_fluid_update(&self) -> bool {
        self.schedule_update
    }
}

pub fn pressure_value(
    position_y: i32,
    first: FluidStatus,
    second: FluidStatus,
    states: AquiferStates,
    barrier_noise: f64,
) -> f64 {
    pressure_from_value(position_y, first, second, states, barrier_noise)
}

fn pressure(
    environment: &mut impl AquiferEnvironment,
    states: AquiferStates,
    position: BlockPos,
    first: FluidStatus,
    second: FluidStatus,
    barrier: &mut Option<f64>,
) -> f64 {
    let first_state = first.at(position.y, states.air);
    let second_state = second.at(position.y, states.air);
    if first_state == states.lava && second_state == states.water
        || first_state == states.water && second_state == states.lava
    {
        return 2.0;
    }
    if first.level == second.level {
        return 0.0;
    }
    let base = pressure_base(position.y, first.level, second.level);
    let noise = if (-2.0..=2.0).contains(&base) {
        *barrier.get_or_insert_with(|| environment.barrier(position))
    } else {
        0.0
    };
    2.0 * (noise + base)
}

fn pressure_from_value(
    y: i32,
    first: FluidStatus,
    second: FluidStatus,
    states: AquiferStates,
    barrier_noise: f64,
) -> f64 {
    let first_state = first.at(y, states.air);
    let second_state = second.at(y, states.air);
    if first_state == states.lava && second_state == states.water
        || first_state == states.water && second_state == states.lava
    {
        2.0
    } else if first.level == second.level {
        0.0
    } else {
        let base = pressure_base(y, first.level, second.level);
        let noise = if (-2.0..=2.0).contains(&base) {
            barrier_noise
        } else {
            0.0
        };
        2.0 * (base + noise)
    }
}

fn pressure_base(y: i32, first_level: i32, second_level: i32) -> f64 {
    let difference = first_level.wrapping_sub(second_level).wrapping_abs();
    let average = 0.5 * f64::from(first_level.wrapping_add(second_level));
    let delta = f64::from(y) + 0.5 - average;
    let half_difference = f64::from(difference) / 2.0;
    let edge = half_difference - delta.abs();
    if delta > 0.0 {
        if edge > 0.0 { edge / 1.5 } else { edge / 2.5 }
    } else {
        let shifted = 3.0 + edge;
        if shifted > 0.0 {
            shifted / 3.0
        } else {
            shifted / 10.0
        }
    }
}

fn insert_nonstrict(nearest: &mut [([i32; 3], i32); 4], candidate: ([i32; 3], i32)) {
    if nearest[0].1 >= candidate.1 {
        nearest[3] = nearest[2];
        nearest[2] = nearest[1];
        nearest[1] = nearest[0];
        nearest[0] = candidate;
    } else if nearest[1].1 >= candidate.1 {
        nearest[3] = nearest[2];
        nearest[2] = nearest[1];
        nearest[1] = candidate;
    } else if nearest[2].1 >= candidate.1 {
        nearest[3] = nearest[2];
        nearest[2] = candidate;
    } else if nearest[3].1 >= candidate.1 {
        nearest[3] = candidate;
    }
}

fn similarity(first: i32, second: i32) -> f64 {
    1.0 - f64::from(second.wrapping_sub(first)) / 25.0
}

fn clamped_map(
    value: f64,
    from_minimum: f64,
    from_maximum: f64,
    to_minimum: f64,
    to_maximum: f64,
) -> f64 {
    if value <= from_minimum {
        to_minimum
    } else if value >= from_maximum {
        to_maximum
    } else {
        map(value, from_minimum, from_maximum, to_minimum, to_maximum)
    }
}

fn map(value: f64, from_minimum: f64, from_maximum: f64, to_minimum: f64, to_maximum: f64) -> f64 {
    to_minimum + (value - from_minimum) / (from_maximum - from_minimum) * (to_maximum - to_minimum)
}
