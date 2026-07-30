//! Boundary-cube ray initialization and affected-position collection.

use std::collections::BTreeSet;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::redstone::explosion::math::Vec3;

pub const DIRECTION_GRID_SIDE: u8 = 16;
pub const DIRECTION_RAY_COUNT: usize = 1_352;
pub const RAY_STEP: f32 = 0.3;
pub const RAY_STEP_DECAY: f32 = 0.22500001;
pub const RESISTANCE_BIAS: f32 = 0.3;
pub const RESISTANCE_SCALE: f32 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayCell {
    pub in_world_bounds: bool,
    pub resistance: Option<f32>,
}

impl RayCell {
    pub const OUT_OF_BOUNDS: Self = Self {
        in_world_bounds: false,
        resistance: None,
    };

    pub const fn air() -> Self {
        Self {
            in_world_bounds: true,
            resistance: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffectedPositions {
    pub positions: BTreeSet<BlockPos>,
    pub random_float_draws: usize,
    pub examined_cells: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RaySamplingError {
    #[error("explosion ray sampling needs {DIRECTION_RAY_COUNT} random float draws")]
    MissingRandomFloat,
}

pub fn calculate_affected_positions(
    center: Vec3,
    radius: f32,
    random_floats: impl IntoIterator<Item = f32>,
    mut inspect: impl FnMut(BlockPos) -> RayCell,
    mut should_explode: impl FnMut(BlockPos, f32) -> bool,
) -> Result<AffectedPositions, RaySamplingError> {
    let mut random_floats = random_floats.into_iter();
    let mut positions = BTreeSet::new();
    let mut random_float_draws = 0;
    let mut examined_cells = 0;
    let mut trace = RayTrace {
        positions: &mut positions,
        examined_cells: &mut examined_cells,
        inspect: &mut inspect,
        should_explode: &mut should_explode,
    };

    for x in 0..DIRECTION_GRID_SIDE {
        for y in 0..DIRECTION_GRID_SIDE {
            for z in 0..DIRECTION_GRID_SIDE {
                if !is_boundary_direction(x, y, z) {
                    continue;
                }
                let random_float = random_floats
                    .next()
                    .ok_or(RaySamplingError::MissingRandomFloat)?;
                random_float_draws += 1;
                trace_ray(center, radius, [x, y, z], random_float, &mut trace);
            }
        }
    }

    Ok(AffectedPositions {
        positions,
        random_float_draws,
        examined_cells,
    })
}

const fn is_boundary_direction(x: u8, y: u8, z: u8) -> bool {
    let last = DIRECTION_GRID_SIDE - 1;
    x == 0 || x == last || y == 0 || y == last || z == 0 || z == last
}

struct RayTrace<'a, Inspect, ShouldExplode> {
    positions: &'a mut BTreeSet<BlockPos>,
    examined_cells: &'a mut usize,
    inspect: &'a mut Inspect,
    should_explode: &'a mut ShouldExplode,
}

fn trace_ray(
    center: Vec3,
    radius: f32,
    direction_cell: [u8; 3],
    random_float: f32,
    trace: &mut RayTrace<impl FnMut(BlockPos) -> RayCell, impl FnMut(BlockPos, f32) -> bool>,
) {
    let direction = normalized_direction(direction_cell);
    let mut point = center;
    let mut power = radius * (0.7_f32 + random_float * 0.6_f32);
    let distance = f64::from(RAY_STEP);

    while power > 0.0 {
        let position = containing(point);
        *trace.examined_cells += 1;
        let cell = (trace.inspect)(position);
        if !cell.in_world_bounds {
            break;
        }
        if let Some(resistance) = cell.resistance {
            power -= (resistance + RESISTANCE_BIAS) * RESISTANCE_SCALE;
        }
        if power > 0.0 && (trace.should_explode)(position, power) {
            trace.positions.insert(position);
        }
        point.x += direction.x * distance;
        point.y += direction.y * distance;
        point.z += direction.z * distance;
        power -= RAY_STEP_DECAY;
    }
}

fn normalized_direction([x, y, z]: [u8; 3]) -> Vec3 {
    let x = f64::from(f32::from(x) / 15.0_f32 * 2.0_f32 - 1.0_f32);
    let y = f64::from(f32::from(y) / 15.0_f32 * 2.0_f32 - 1.0_f32);
    let z = f64::from(f32::from(z) / 15.0_f32 * 2.0_f32 - 1.0_f32);
    Vec3::new(x, y, z).normalize()
}

fn containing(point: Vec3) -> BlockPos {
    BlockPos::new(
        point.x.floor() as i32,
        point.y.floor() as i32,
        point.z.floor() as i32,
    )
}
