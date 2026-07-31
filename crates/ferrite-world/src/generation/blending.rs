//! Upgrade blending between old-noise and current-noise terrain.

use std::collections::BTreeSet;

use crate::generation::density::DensityContext;
use crate::id::BiomeId;

const QUARTS_PER_CHUNK: i32 = 4;
const INSIDE_MAX: i32 = 3;
const OUTSIDE_MAX: i32 = 4;
const INSIDE_COUNT: usize = 7;
const COLUMN_COUNT: usize = 16;
const HEIGHT_RANGE: f64 = 27.0;
const DENSITY_RANGE: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction8 {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OldBlock {
    Air,
    LeavesOrLogs,
    MushroomBlock,
    NoCollision,
    Solid,
    Surface,
}

impl OldBlock {
    fn is_ground(self) -> bool {
        matches!(self, Self::Solid | Self::Surface)
    }

    fn is_surface(self) -> bool {
        self == Self::Surface
    }
}

pub trait OldChunkColumnSource {
    /// Returns the primed `WORLD_SURFACE_WG` height, when that heightmap exists.
    fn primed_surface_height(&self, block_x: i32, block_z: i32) -> Option<i32>;

    fn block(&self, block_x: i32, block_y: i32, block_z: i32) -> OldBlock;

    fn biome(&self, _quart_x: i32, _quart_y: i32, _quart_z: i32) -> Option<BiomeId> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlendingData {
    min_section: i32,
    max_section: i32,
    calculated: bool,
    heights: [Option<f64>; COLUMN_COUNT],
    densities: [Option<Vec<f64>>; COLUMN_COUNT],
    biomes: [Option<Vec<Option<BiomeId>>>; COLUMN_COUNT],
}

impl BlendingData {
    pub fn new(min_section: i32, max_section: i32) -> Self {
        assert!(
            max_section > min_section,
            "old-generation area must be nonempty"
        );
        Self {
            min_section,
            max_section,
            calculated: false,
            heights: [None; COLUMN_COUNT],
            densities: std::array::from_fn(|_| None),
            biomes: std::array::from_fn(|_| None),
        }
    }

    pub fn from_packed_heights(
        min_section: i32,
        max_section: i32,
        heights: Option<[f64; COLUMN_COUNT]>,
    ) -> Self {
        let mut data = Self::new(min_section, max_section);
        if let Some(heights) = heights {
            data.heights = heights.map(|height| (height != f64::MAX).then_some(height));
        }
        data
    }

    pub fn min_section(&self) -> i32 {
        self.min_section
    }

    pub fn max_section(&self) -> i32 {
        self.max_section
    }

    pub fn packed_heights(&self) -> Option<[f64; COLUMN_COUNT]> {
        self.heights
            .iter()
            .any(Option::is_some)
            .then(|| self.heights.map(|height| height.unwrap_or(f64::MAX)))
    }

    pub fn calculate_boundary_columns<S: OldChunkColumnSource>(
        &mut self,
        source: &S,
        new_sides: &BTreeSet<Direction8>,
    ) {
        if self.calculated {
            return;
        }
        if new_sides.contains(&Direction8::North)
            || new_sides.contains(&Direction8::West)
            || new_sides.contains(&Direction8::NorthWest)
        {
            self.add_column(inside_index(0, 0), source, 0, 0);
        }
        if new_sides.contains(&Direction8::North) {
            for cell in 1..QUARTS_PER_CHUNK {
                self.add_column(inside_index(cell, 0), source, 4 * cell, 0);
            }
        }
        if new_sides.contains(&Direction8::West) {
            for cell in 1..QUARTS_PER_CHUNK {
                self.add_column(inside_index(0, cell), source, 0, 4 * cell);
            }
        }
        if new_sides.contains(&Direction8::East) {
            for cell in 1..QUARTS_PER_CHUNK {
                self.add_column(outside_index(OUTSIDE_MAX, cell), source, 15, 4 * cell);
            }
        }
        if new_sides.contains(&Direction8::South) {
            for cell in 0..QUARTS_PER_CHUNK {
                self.add_column(outside_index(cell, OUTSIDE_MAX), source, 4 * cell, 15);
            }
        }
        if new_sides.contains(&Direction8::East) && new_sides.contains(&Direction8::NorthEast) {
            self.add_column(outside_index(OUTSIDE_MAX, 0), source, 15, 0);
        }
        if new_sides.contains(&Direction8::East)
            && new_sides.contains(&Direction8::South)
            && new_sides.contains(&Direction8::SouthEast)
        {
            self.add_column(outside_index(OUTSIDE_MAX, OUTSIDE_MAX), source, 15, 15);
        }
        self.calculated = true;
    }

    pub fn set_boundary_column(
        &mut self,
        cell_x: i32,
        cell_z: i32,
        height: f64,
        densities: Vec<f64>,
    ) -> bool {
        let Some(index) = boundary_index(cell_x, cell_z) else {
            return false;
        };
        self.heights[index] = Some(height);
        self.densities[index] = Some(densities);
        true
    }

    pub fn set_boundary_biomes(
        &mut self,
        cell_x: i32,
        cell_z: i32,
        biomes: Vec<Option<BiomeId>>,
    ) -> bool {
        let Some(index) = boundary_index(cell_x, cell_z) else {
            return false;
        };
        self.biomes[index] = Some(biomes);
        true
    }

    pub fn get_height(&self, cell_x: i32, cell_z: i32) -> Option<f64> {
        boundary_index(cell_x, cell_z).and_then(|index| self.heights[index])
    }

    pub fn get_density(&self, cell_x: i32, cell_y: i32, cell_z: i32) -> Option<f64> {
        if cell_y == self.minimum_cell_y() {
            return Some(0.1);
        }
        let index = boundary_index(cell_x, cell_z)?;
        let density = self.densities[index].as_ref()?;
        let y_index = cell_y.checked_sub(self.column_min_y())?;
        let y_index = usize::try_from(y_index).ok()?;
        density.get(y_index).map(|value| value * 0.1)
    }

    fn add_column<S: OldChunkColumnSource>(
        &mut self,
        index: usize,
        source: &S,
        block_x: i32,
        block_z: i32,
    ) {
        let height = match self.heights[index] {
            Some(height) => height,
            None => {
                let height = f64::from(self.find_surface_height(source, block_x, block_z));
                self.heights[index] = Some(height);
                height
            }
        };
        self.densities[index] =
            Some(self.calculate_density_column(source, block_x, block_z, height.floor() as i32));
        let quart_x = block_x >> 2;
        let quart_z = block_z >> 2;
        self.biomes[index] = Some(
            (0..self.quart_count())
                .map(|offset| {
                    source.biome(
                        quart_x,
                        self.minimum_quart_y()
                            .wrapping_add(i32::try_from(offset).expect("quart index fits i32")),
                        quart_z,
                    )
                })
                .collect(),
        );
    }

    fn find_surface_height<S: OldChunkColumnSource>(
        &self,
        source: &S,
        block_x: i32,
        block_z: i32,
    ) -> i32 {
        let old_maximum = self.maximum_block_y();
        let mut y = source
            .primed_surface_height(block_x, block_z)
            .map_or(old_maximum, |height| height.min(old_maximum));
        let minimum = self.minimum_block_y();
        while y > minimum {
            if source.block(block_x, y, block_z).is_surface() {
                return y;
            }
            y -= 1;
        }
        minimum
    }

    fn calculate_density_column<S: OldChunkColumnSource>(
        &self,
        source: &S,
        block_x: i32,
        block_z: i32,
        height: i32,
    ) -> Vec<f64> {
        let mut densities = vec![-1.0; self.cell_count()];
        let mut y = self.maximum_block_y() + 1;
        let mut last_seven = read_seven(source, block_x, block_z, &mut y);
        for cell_index in (0..densities.len() - 1).rev() {
            let middle = read_one(source, block_x, block_z, &mut y);
            let current_seven = read_seven(source, block_x, block_z, &mut y);
            densities[cell_index] = (last_seven + middle + current_seven) / 15.0;
            last_seven = current_seven;
        }
        let surface_index = height.div_euclid(8) - self.column_min_y();
        if let Ok(index) = usize::try_from(surface_index)
            && index < densities.len() - 1
        {
            let fraction = ((f64::from(height) + 0.5) % 8.0) / 8.0;
            let ratio = (1.0 - fraction) / fraction;
            let scale = 0.25 * ratio.max(1.0);
            densities[index + 1] = -ratio / scale;
            densities[index] = 1.0 / scale;
        }
        densities
    }

    fn cell_count(&self) -> usize {
        usize::try_from((self.max_section - self.min_section) * 2)
            .expect("validated section range fits usize")
    }

    fn quart_count(&self) -> usize {
        usize::try_from((self.max_section - self.min_section) * 4)
            .expect("validated section range fits usize")
    }

    fn minimum_block_y(&self) -> i32 {
        self.min_section * 16
    }

    fn maximum_block_y(&self) -> i32 {
        self.max_section * 16 - 1
    }

    fn minimum_cell_y(&self) -> i32 {
        self.min_section * 2
    }

    fn column_min_y(&self) -> i32 {
        self.minimum_cell_y() + 1
    }

    fn minimum_quart_y(&self) -> i32 {
        self.min_section * 4
    }

    fn visit_heights(&self, chunk_x: i32, chunk_z: i32, mut visit: impl FnMut(i32, i32, f64)) {
        let minimum_x = chunk_x * QUARTS_PER_CHUNK;
        let minimum_z = chunk_z * QUARTS_PER_CHUNK;
        for (index, height) in self.heights.iter().enumerate() {
            if let Some(height) = height {
                visit(
                    minimum_x + column_x(index),
                    minimum_z + column_z(index),
                    *height,
                );
            }
        }
    }

    fn visit_biomes(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        quart_y: i32,
        mut visit: impl FnMut(i32, i32, BiomeId),
    ) {
        let quart_index = quart_y - self.minimum_quart_y();
        let Ok(quart_index) = usize::try_from(quart_index) else {
            return;
        };
        if quart_index >= self.quart_count() {
            return;
        }
        let minimum_x = chunk_x * QUARTS_PER_CHUNK;
        let minimum_z = chunk_z * QUARTS_PER_CHUNK;
        for (index, column) in self.biomes.iter().enumerate() {
            if let Some(biome) = column
                .as_ref()
                .and_then(|column| column.get(quart_index))
                .copied()
                .flatten()
            {
                visit(
                    minimum_x + column_x(index),
                    minimum_z + column_z(index),
                    biome,
                );
            }
        }
    }

    fn visit_densities(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        from_cell_y: i32,
        to_cell_y: i32,
        mut visit: impl FnMut(i32, i32, i32, f64),
    ) {
        let minimum_x = chunk_x * QUARTS_PER_CHUNK;
        let minimum_z = chunk_z * QUARTS_PER_CHUNK;
        let minimum_y = self.column_min_y();
        let first =
            usize::try_from((from_cell_y - minimum_y).max(0)).expect("nonnegative density start");
        let end = usize::try_from((to_cell_y - minimum_y).max(0))
            .expect("nonnegative density end")
            .min(self.cell_count());
        for (index, column) in self.densities.iter().enumerate() {
            let Some(column) = column else {
                continue;
            };
            for (y_index, density) in column.iter().enumerate().take(end).skip(first) {
                visit(
                    minimum_x + column_x(index),
                    minimum_y + i32::try_from(y_index).expect("density index fits i32"),
                    minimum_z + column_z(index),
                    density * 0.1,
                );
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendingOutput {
    pub alpha: f64,
    pub offset: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlendFlatCache {
    first_quart_x: i32,
    first_quart_z: i32,
    width: usize,
    values: Vec<BlendingOutput>,
}

impl BlendFlatCache {
    pub fn new(
        blender: &Blender,
        first_noise_x: i32,
        first_noise_z: i32,
        noise_size_xz: usize,
    ) -> Self {
        let width = noise_size_xz + 1;
        let mut values = Vec::with_capacity(width * width);
        for x in 0..width {
            for z in 0..width {
                values.push(blender.blend_offset_and_factor(
                    first_noise_x.wrapping_add(x as i32).wrapping_mul(4),
                    first_noise_z.wrapping_add(z as i32).wrapping_mul(4),
                ));
            }
        }
        Self {
            first_quart_x: first_noise_x,
            first_quart_z: first_noise_z,
            width,
            values,
        }
    }

    pub fn sample(&self, blender: &Blender, block_x: i32, block_z: i32) -> BlendingOutput {
        let local_x = (block_x >> 2).wrapping_sub(self.first_quart_x);
        let local_z = (block_z >> 2).wrapping_sub(self.first_quart_z);
        let (Ok(local_x), Ok(local_z)) = (usize::try_from(local_x), usize::try_from(local_z))
        else {
            return blender.blend_offset_and_factor(block_x, block_z);
        };
        if local_x >= self.width || local_z >= self.width {
            blender.blend_offset_and_factor(block_x, block_z)
        } else {
            self.values[local_x * self.width + local_z]
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Blender {
    height_data: Vec<((i32, i32), BlendingData)>,
    density_data: Vec<((i32, i32), BlendingData)>,
}

impl Blender {
    pub fn empty() -> Self {
        Self {
            height_data: Vec::new(),
            density_data: Vec::new(),
        }
    }

    pub fn new(
        height_data: Vec<((i32, i32), BlendingData)>,
        density_data: Vec<((i32, i32), BlendingData)>,
    ) -> Self {
        Self {
            height_data,
            density_data,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.height_data.is_empty() && self.density_data.is_empty()
    }

    pub fn blend_offset_and_factor(&self, block_x: i32, block_z: i32) -> BlendingOutput {
        let cell_x = block_x >> 2;
        let cell_z = block_z >> 2;
        if let Some(height) = direct_value(&self.height_data, cell_x, cell_z, |data, x, z| {
            data.get_height(x, z)
        }) {
            return BlendingOutput {
                alpha: 0.0,
                offset: height_to_offset(height),
            };
        }

        let mut total_weight = 0.0;
        let mut weighted_heights = 0.0;
        let mut closest_distance = f64::INFINITY;
        for ((chunk_x, chunk_z), data) in &self.height_data {
            data.visit_heights(*chunk_x, *chunk_z, |sample_x, sample_z, height| {
                let dx = f64::from(cell_x - sample_x);
                let dz = f64::from(cell_z - sample_z);
                let distance = dx.hypot(dz);
                if distance <= HEIGHT_RANGE {
                    closest_distance = closest_distance.min(distance);
                    let squared = distance * distance;
                    let weight = 1.0 / (squared * squared);
                    weighted_heights += height * weight;
                    total_weight += weight;
                }
            });
        }
        if closest_distance == f64::INFINITY {
            return BlendingOutput {
                alpha: 1.0,
                offset: 0.0,
            };
        }
        let height = weighted_heights / total_weight;
        let alpha = (closest_distance / 28.0).clamp(0.0, 1.0);
        BlendingOutput {
            alpha: 3.0 * alpha * alpha - 2.0 * alpha * alpha * alpha,
            offset: height_to_offset(height),
        }
    }

    pub fn blend_density(&self, context: DensityContext, new_density: f64) -> f64 {
        let cell_x = context.x >> 2;
        let cell_y = context.y / 8;
        let cell_z = context.z >> 2;
        if let Some(density) = direct_value(&self.density_data, cell_x, cell_z, |data, x, z| {
            data.get_density(x, cell_y, z)
        }) {
            return density;
        }

        let mut total_weight = 0.0;
        let mut weighted_density = 0.0;
        let mut closest_distance = f64::INFINITY;
        for ((chunk_x, chunk_z), data) in &self.density_data {
            data.visit_densities(
                *chunk_x,
                *chunk_z,
                cell_y - 1,
                cell_y + 1,
                |sample_x, sample_y, sample_z, density| {
                    let dx = f64::from(cell_x - sample_x);
                    let dy = f64::from((cell_y - sample_y) * 2);
                    let dz = f64::from(cell_z - sample_z);
                    let distance = (dx * dx + dy * dy + dz * dz).sqrt();
                    if distance <= DENSITY_RANGE {
                        closest_distance = closest_distance.min(distance);
                        let squared = distance * distance;
                        let weight = 1.0 / (squared * squared);
                        weighted_density += density * weight;
                        total_weight += weight;
                    }
                },
            );
        }
        if closest_distance == f64::INFINITY {
            return new_density;
        }
        let old_density = weighted_density / total_weight;
        let alpha = (closest_distance / 3.0).clamp(0.0, 1.0);
        old_density + alpha * (new_density - old_density)
    }

    pub fn blend_density_array(&self, contexts: &[DensityContext], values: &mut [f64]) {
        assert_eq!(contexts.len(), values.len());
        for (context, value) in contexts.iter().copied().zip(values) {
            *value = self.blend_density(context, *value);
        }
    }

    pub fn blend_biome(
        &self,
        quart_x: i32,
        quart_y: i32,
        quart_z: i32,
        shift_noise: impl FnOnce(i32, i32) -> f64,
    ) -> Option<BiomeId> {
        let mut closest_distance = f64::INFINITY;
        let mut closest_biome = None;
        for ((chunk_x, chunk_z), data) in &self.height_data {
            data.visit_biomes(*chunk_x, *chunk_z, quart_y, |sample_x, sample_z, biome| {
                let dx = f64::from(quart_x - sample_x);
                let dz = f64::from(quart_z - sample_z);
                let distance = dx.hypot(dz);
                if distance <= HEIGHT_RANGE && distance < closest_distance {
                    closest_distance = distance;
                    closest_biome = Some(biome);
                }
            });
        }
        if closest_distance == f64::INFINITY {
            return None;
        }
        let shifted_distance = closest_distance + shift_noise(quart_x, quart_z) * 12.0;
        let alpha = (shifted_distance / 28.0).clamp(0.0, 1.0);
        (alpha <= 0.5).then_some(closest_biome).flatten()
    }

    pub fn resolve_biome(
        &self,
        quart_x: i32,
        quart_y: i32,
        quart_z: i32,
        shift_noise: impl FnOnce(i32, i32) -> f64,
        fallback: impl FnOnce() -> BiomeId,
    ) -> BiomeId {
        self.blend_biome(quart_x, quart_y, quart_z, shift_noise)
            .unwrap_or_else(fallback)
    }
}

pub fn height_to_offset(height: f64) -> f64 {
    let target = height + 0.5;
    let remainder = ((target % 8.0) + 8.0) % 8.0;
    (32.0 * (target - 128.0) - 3.0 * (target - 120.0) * remainder + 3.0 * remainder * remainder)
        / (128.0 * (32.0 - 3.0 * remainder))
}

fn direct_value(
    data: &[((i32, i32), BlendingData)],
    cell_x: i32,
    cell_z: i32,
    get: impl Fn(&BlendingData, i32, i32) -> Option<f64>,
) -> Option<f64> {
    let chunk_x = cell_x >> 2;
    let chunk_z = cell_z >> 2;
    let minimum_x = cell_x & 3 == 0;
    let minimum_z = cell_z & 3 == 0;
    let candidates = [
        Some((chunk_x, chunk_z)),
        (minimum_x && minimum_z).then_some((chunk_x - 1, chunk_z - 1)),
        minimum_x.then_some((chunk_x - 1, chunk_z)),
        minimum_z.then_some((chunk_x, chunk_z - 1)),
    ];
    for candidate in candidates.into_iter().flatten() {
        let Some((_, blending)) = data.iter().find(|(position, _)| *position == candidate) else {
            continue;
        };
        let local_x = cell_x - candidate.0 * QUARTS_PER_CHUNK;
        let local_z = cell_z - candidate.1 * QUARTS_PER_CHUNK;
        if let Some(value) = get(blending, local_x, local_z) {
            return Some(value);
        }
    }
    None
}

fn read_one<S: OldChunkColumnSource>(source: &S, block_x: i32, block_z: i32, y: &mut i32) -> f64 {
    *y -= 1;
    if source.block(block_x, *y, block_z).is_ground() {
        1.0
    } else {
        -1.0
    }
}

fn read_seven<S: OldChunkColumnSource>(source: &S, block_x: i32, block_z: i32, y: &mut i32) -> f64 {
    (0..7).map(|_| read_one(source, block_x, block_z, y)).sum()
}

fn boundary_index(x: i32, z: i32) -> Option<usize> {
    if x == OUTSIDE_MAX || z == OUTSIDE_MAX {
        ((0..=OUTSIDE_MAX).contains(&x) && (0..=OUTSIDE_MAX).contains(&z))
            .then(|| outside_index(x, z))
    } else if x == 0 || z == 0 {
        ((0..=INSIDE_MAX).contains(&x) && (0..=INSIDE_MAX).contains(&z)).then(|| inside_index(x, z))
    } else {
        None
    }
}

fn inside_index(x: i32, z: i32) -> usize {
    usize::try_from(INSIDE_MAX - x + z).expect("inside boundary index is nonnegative")
}

fn outside_index(x: i32, z: i32) -> usize {
    INSIDE_COUNT
        + usize::try_from(x + OUTSIDE_MAX - z).expect("outside boundary index is nonnegative")
}

fn column_x(index: usize) -> i32 {
    if index < INSIDE_COUNT {
        (INSIDE_MAX - i32::try_from(index).expect("column index fits i32")).max(0)
    } else {
        let offset = i32::try_from(index - INSIDE_COUNT).expect("column index fits i32");
        OUTSIDE_MAX - (OUTSIDE_MAX - offset).max(0)
    }
}

fn column_z(index: usize) -> i32 {
    if index < INSIDE_COUNT {
        (i32::try_from(index).expect("column index fits i32") - INSIDE_MAX).max(0)
    } else {
        let offset = i32::try_from(index - INSIDE_COUNT).expect("column index fits i32");
        OUTSIDE_MAX - (offset - OUTSIDE_MAX).max(0)
    }
}
