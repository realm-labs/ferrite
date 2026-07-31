//! Structure-piece and jigsaw-junction terrain density.

use crate::generation::density::DensityContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAdjustment {
    None,
    Bury,
    BeardThin,
    BeardBox,
    Encapsulate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeardPiece {
    pub minimum: [i32; 3],
    pub maximum: [i32; 3],
    pub ground_level_delta: i32,
    pub adjustment: TerrainAdjustment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeardJunction {
    pub source_x: i32,
    pub source_ground_y: i32,
    pub source_z: i32,
}

#[derive(Debug, Clone)]
pub struct Beardifier {
    pieces: Vec<BeardPiece>,
    junctions: Vec<BeardJunction>,
    affected: Option<([i32; 3], [i32; 3])>,
}

impl Beardifier {
    pub fn new(pieces: Vec<BeardPiece>, junctions: Vec<BeardJunction>) -> Self {
        let mut minimum = [i32::MAX; 3];
        let mut maximum = [i32::MIN; 3];
        for piece in &pieces {
            for axis in 0..3 {
                minimum[axis] = minimum[axis].min(piece.minimum[axis]);
                maximum[axis] = maximum[axis].max(piece.maximum[axis]);
            }
        }
        for junction in &junctions {
            let point = [
                junction.source_x,
                junction.source_ground_y,
                junction.source_z,
            ];
            for axis in 0..3 {
                minimum[axis] = minimum[axis].min(point[axis]);
                maximum[axis] = maximum[axis].max(point[axis]);
            }
        }
        let affected = if pieces.is_empty() && junctions.is_empty() {
            None
        } else {
            Some((
                minimum.map(|value| value.wrapping_sub(24)),
                maximum.map(|value| value.wrapping_add(24)),
            ))
        };
        Self {
            pieces,
            junctions,
            affected,
        }
    }

    pub fn sample(&self, context: DensityContext) -> f64 {
        let Some((minimum, maximum)) = self.affected else {
            return 0.0;
        };
        let position = [context.x, context.y, context.z];
        if (0..3).any(|axis| position[axis] < minimum[axis] || position[axis] > maximum[axis]) {
            return 0.0;
        }
        let mut result = 0.0;
        for piece in &self.pieces {
            let x = outside_distance(context.x, piece.minimum[0], piece.maximum[0]);
            let z = outside_distance(context.z, piece.minimum[2], piece.maximum[2]);
            let ground = piece.minimum[1].wrapping_add(piece.ground_level_delta);
            let raw_y = context.y.wrapping_sub(ground);
            let vertical = match piece.adjustment {
                TerrainAdjustment::None => 0,
                TerrainAdjustment::Bury | TerrainAdjustment::BeardThin => raw_y,
                TerrainAdjustment::BeardBox => {
                    outside_distance(context.y, ground, piece.maximum[1])
                }
                TerrainAdjustment::Encapsulate => {
                    outside_distance(context.y, piece.minimum[1], piece.maximum[1])
                }
            };
            result += match piece.adjustment {
                TerrainAdjustment::None => 0.0,
                TerrainAdjustment::Bury => {
                    bury(f64::from(x), f64::from(vertical) / 2.0, f64::from(z))
                }
                TerrainAdjustment::BeardThin | TerrainAdjustment::BeardBox => {
                    0.8 * beard(x, vertical, z, raw_y)
                }
                TerrainAdjustment::Encapsulate => {
                    0.8 * bury(
                        f64::from(x) / 2.0,
                        f64::from(vertical) / 2.0,
                        f64::from(z) / 2.0,
                    )
                }
            };
        }
        for junction in &self.junctions {
            let x = context.x.wrapping_sub(junction.source_x);
            let y = context.y.wrapping_sub(junction.source_ground_y);
            let z = context.z.wrapping_sub(junction.source_z);
            result += 0.4 * beard(x, y, z, y);
        }
        result
    }

    pub fn bounds(&self) -> (f64, f64) {
        if self.affected.is_none() {
            (0.0, 0.0)
        } else {
            (f64::NEG_INFINITY, f64::INFINITY)
        }
    }

    pub fn affected_box(&self) -> Option<([i32; 3], [i32; 3])> {
        self.affected
    }
}

fn outside_distance(value: i32, minimum: i32, maximum: i32) -> i32 {
    0.max(minimum.wrapping_sub(value))
        .max(value.wrapping_sub(maximum))
}

fn bury(x: f64, y: f64, z: f64) -> f64 {
    let distance = (x * x + y * y + z * z).sqrt();
    if distance <= 0.0 {
        1.0
    } else if distance >= 6.0 {
        0.0
    } else {
        1.0 - distance / 6.0
    }
}

fn beard(x: i32, y: i32, z: i32, raw_y: i32) -> f64 {
    let kernel_x = x.wrapping_add(12);
    let kernel_y = y.wrapping_add(12);
    let kernel_z = z.wrapping_add(12);
    if !(0..24).contains(&kernel_x) || !(0..24).contains(&kernel_y) || !(0..24).contains(&kernel_z)
    {
        return 0.0;
    }
    let shifted_y = f64::from(y) + 0.5;
    let exponent =
        -(f64::from(x.wrapping_mul(x)) + shifted_y * shifted_y + f64::from(z.wrapping_mul(z)))
            / 16.0;
    let kernel = std::f64::consts::E.powf(exponent) as f32;
    let sign_y = f64::from(raw_y) + 0.5;
    let squared = f64::from(x.wrapping_mul(x)) + sign_y * sign_y + f64::from(z.wrapping_mul(z));
    (-sign_y * fast_inverse_sqrt(squared / 2.0) / 2.0) * f64::from(kernel)
}

fn fast_inverse_sqrt(value: f64) -> f64 {
    let bits = value.to_bits();
    let estimate_bits = 6_910_469_410_427_058_090_u64.wrapping_sub(bits >> 1);
    let estimate = f64::from_bits(estimate_bits);
    estimate * (1.5 - 0.5 * value * estimate * estimate)
}
