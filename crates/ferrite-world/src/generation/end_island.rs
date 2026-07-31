//! End-island density and its two-dimensional simplex noise.

use std::num::NonZeroU32;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};

const GRADIENTS: [[i32; 3]; 12] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
];

#[derive(Debug, Clone)]
pub struct SimplexNoise {
    permutation: [u8; 256],
    pub x_offset: f64,
    pub y_offset: f64,
    pub z_offset: f64,
}

impl SimplexNoise {
    pub fn new(random: &mut impl GenerationRandom) -> Self {
        let x_offset = random.next_f64() * 256.0;
        let y_offset = random.next_f64() * 256.0;
        let z_offset = random.next_f64() * 256.0;
        let mut permutation = [0_u8; 256];
        for (index, value) in permutation.iter_mut().enumerate() {
            *value = index as u8;
        }
        for index in 0..256 {
            let bound =
                NonZeroU32::new((256 - index) as u32).expect("simplex shuffle bound is nonzero");
            let offset = random.next_u32(bound) as usize;
            permutation.swap(index, index + offset);
        }
        Self {
            permutation,
            x_offset,
            y_offset,
            z_offset,
        }
    }

    pub fn sample_2d(&self, x: f64, z: f64) -> f64 {
        let skew_factor = (3.0_f64.sqrt() - 1.0) / 2.0;
        let unskew_factor = (3.0 - 3.0_f64.sqrt()) / 6.0;
        let skew = (x + z) * skew_factor;
        let cell_x = minecraft_floor(x + skew);
        let cell_z = minecraft_floor(z + skew);
        let unskew = f64::from(cell_x.wrapping_add(cell_z)) * unskew_factor;
        let local_x = x - (f64::from(cell_x) - unskew);
        let local_z = z - (f64::from(cell_z) - unskew);
        let [middle_x, middle_z] = if local_x > local_z { [1, 0] } else { [0, 1] };
        let middle_local_x = local_x - f64::from(middle_x) + unskew_factor;
        let middle_local_z = local_z - f64::from(middle_z) + unskew_factor;
        let last_local_x = local_x - 1.0 + 2.0 * unskew_factor;
        let last_local_z = local_z - 1.0 + 2.0 * unskew_factor;
        let first_gradient = self.p(cell_x.wrapping_add(self.p(cell_z))) % 12;
        let middle_gradient = self.p(cell_x
            .wrapping_add(middle_x)
            .wrapping_add(self.p(cell_z.wrapping_add(middle_z))))
            % 12;
        let last_gradient = self.p(cell_x
            .wrapping_add(1)
            .wrapping_add(self.p(cell_z.wrapping_add(1))))
            % 12;
        70.0 * (corner(first_gradient, local_x, local_z)
            + corner(middle_gradient, middle_local_x, middle_local_z)
            + corner(last_gradient, last_local_x, last_local_z))
    }

    pub fn permutation(&self) -> &[u8; 256] {
        &self.permutation
    }

    fn p(&self, index: i32) -> i32 {
        i32::from(self.permutation[(index & 255) as usize])
    }
}

#[derive(Debug, Clone)]
pub struct EndIslandDensity {
    simplex: SimplexNoise,
}

impl EndIslandDensity {
    pub fn new(world_seed: i64) -> Self {
        let mut random = LegacyRandom::new(world_seed);
        for _ in 0..17_292 {
            let _ = random.next_i32();
        }
        Self {
            simplex: SimplexNoise::new(&mut random),
        }
    }

    pub fn sample(&self, block_x: i32, block_z: i32) -> f64 {
        let section_x = block_x / 8;
        let section_z = block_z / 8;
        let squared = section_x
            .wrapping_mul(section_x)
            .wrapping_add(section_z.wrapping_mul(section_z));
        let mut height = (100.0_f32 - (squared as f32).sqrt() * 8.0).clamp(-100.0, 80.0);
        let half_x = section_x / 2;
        let half_z = section_z / 2;
        let remainder_x = section_x % 2;
        let remainder_z = section_z % 2;
        for x_offset in -12_i32..=12 {
            for z_offset in -12_i32..=12 {
                let island_x = i64::from(half_x.wrapping_add(x_offset));
                let island_z = i64::from(half_z.wrapping_add(z_offset));
                if island_x
                    .wrapping_mul(island_x)
                    .wrapping_add(island_z.wrapping_mul(island_z))
                    <= 4_096
                    || self.simplex.sample_2d(island_x as f64, island_z as f64)
                        >= f64::from(-0.9_f32)
                {
                    continue;
                }
                let scale = ((island_x as f32).abs() * 3_439.0 + (island_z as f32).abs() * 147.0)
                    % 13.0
                    + 9.0;
                let relative_x = remainder_x.wrapping_sub(x_offset.wrapping_mul(2)) as f32;
                let relative_z = remainder_z.wrapping_sub(z_offset.wrapping_mul(2)) as f32;
                let candidate = (100.0
                    - (relative_x * relative_x + relative_z * relative_z).sqrt() * scale)
                    .clamp(-100.0, 80.0);
                height = java_max_f32(height, candidate);
            }
        }
        (f64::from(height) - 8.0) / 128.0
    }

    pub fn simplex(&self) -> &SimplexNoise {
        &self.simplex
    }

    pub fn bounds(&self) -> (f64, f64) {
        (-0.84375, 0.5625)
    }
}

fn corner(gradient_index: i32, x: f64, z: f64) -> f64 {
    let radius = 0.5 - x * x - z * z;
    if radius < 0.0 {
        return 0.0;
    }
    let gradient = GRADIENTS[gradient_index as usize];
    let squared = radius * radius;
    squared * squared * (f64::from(gradient[0]) * x + f64::from(gradient[1]) * z)
}

fn minecraft_floor(value: f64) -> i32 {
    let truncated = value as i32;
    if value < f64::from(truncated) {
        truncated.wrapping_sub(1)
    } else {
        truncated
    }
}

fn java_max_f32(first: f32, second: f32) -> f32 {
    if first.is_nan() || second.is_nan() {
        f32::NAN
    } else {
        first.max(second)
    }
}
