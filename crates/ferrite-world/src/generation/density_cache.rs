//! Noise-chunk cache and interpolation marker lifecycles.

use crate::generation::density::DensityContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DensityRuntime {
    pub owner: bool,
    pub interpolation_running: bool,
    pub filling_cell: bool,
    pub interpolation_counter: u64,
    pub array_counter: u64,
    pub array_index: usize,
}

#[derive(Debug, Clone)]
pub struct FlatCache {
    first_quart_x: i32,
    first_quart_z: i32,
    width: usize,
    values: Vec<f64>,
}

impl FlatCache {
    pub fn new(
        first_noise_x: i32,
        first_noise_z: i32,
        noise_size_xz: usize,
        mut child: impl FnMut(DensityContext) -> f64,
    ) -> Self {
        let width = noise_size_xz + 1;
        let mut values = Vec::with_capacity(width * width);
        for x in 0..width {
            for z in 0..width {
                values.push(child(DensityContext {
                    x: first_noise_x.wrapping_add(x as i32).wrapping_mul(4),
                    y: 0,
                    z: first_noise_z.wrapping_add(z as i32).wrapping_mul(4),
                }));
            }
        }
        Self {
            first_quart_x: first_noise_x,
            first_quart_z: first_noise_z,
            width,
            values,
        }
    }

    pub fn sample(
        &self,
        context: DensityContext,
        mut child: impl FnMut(DensityContext) -> f64,
    ) -> f64 {
        let quart_x = context.x >> 2;
        let quart_z = context.z >> 2;
        let local_x = quart_x.wrapping_sub(self.first_quart_x);
        let local_z = quart_z.wrapping_sub(self.first_quart_z);
        let (Ok(local_x), Ok(local_z)) = (usize::try_from(local_x), usize::try_from(local_z))
        else {
            return child(context);
        };
        if local_x >= self.width || local_z >= self.width {
            child(context)
        } else {
            self.values[local_x * self.width + local_z]
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cache2D {
    last_x: i32,
    last_z: i32,
    value: f64,
}

impl Default for Cache2D {
    fn default() -> Self {
        Self {
            last_x: 1_875_066,
            last_z: 1_875_066,
            value: 0.0,
        }
    }
}

impl Cache2D {
    pub fn sample(
        &mut self,
        context: DensityContext,
        mut child: impl FnMut(DensityContext) -> f64,
    ) -> f64 {
        if context.x != self.last_x || context.z != self.last_z {
            self.last_x = context.x;
            self.last_z = context.z;
            self.value = child(context);
        }
        self.value
    }

    pub fn fill(
        &mut self,
        contexts: &[DensityContext],
        output: &mut [f64],
        mut child: impl FnMut(DensityContext) -> f64,
    ) {
        for (output, context) in output.iter_mut().zip(contexts.iter().copied()) {
            *output = child(context);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheOnce {
    scalar_counter: u64,
    scalar_value: f64,
    array_counter: u64,
    array: Vec<f64>,
}

impl Default for CacheOnce {
    fn default() -> Self {
        Self {
            scalar_counter: 0,
            scalar_value: 0.0,
            array_counter: 0,
            array: Vec::new(),
        }
    }
}

impl CacheOnce {
    pub fn sample(
        &mut self,
        context: DensityContext,
        runtime: DensityRuntime,
        mut child: impl FnMut(DensityContext) -> f64,
    ) -> f64 {
        if !runtime.owner {
            return child(context);
        }
        if self.array_counter == runtime.array_counter && !self.array.is_empty() {
            return self.array[runtime.array_index];
        }
        if self.scalar_counter == runtime.interpolation_counter {
            return self.scalar_value;
        }
        self.scalar_counter = runtime.interpolation_counter;
        self.scalar_value = child(context);
        self.scalar_value
    }

    pub fn fill(
        &mut self,
        contexts: &[DensityContext],
        output: &mut [f64],
        runtime: DensityRuntime,
        mut child: impl FnMut(DensityContext) -> f64,
    ) {
        if runtime.owner
            && self.array_counter == runtime.array_counter
            && self.array.len() >= output.len()
        {
            output.copy_from_slice(&self.array[..output.len()]);
            return;
        }
        for (output, context) in output.iter_mut().zip(contexts.iter().copied()) {
            *output = child(context);
        }
        if runtime.owner {
            self.array.clear();
            self.array.extend_from_slice(output);
            self.array_counter = runtime.array_counter;
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheAllInCell {
    cell_width: usize,
    cell_height: usize,
    cell_origin: DensityContext,
    values: Vec<f64>,
}

impl CacheAllInCell {
    pub fn new(cell_width: usize, cell_height: usize) -> Self {
        Self {
            cell_width,
            cell_height,
            cell_origin: DensityContext { x: 0, y: 0, z: 0 },
            values: vec![0.0; cell_width * cell_width * cell_height],
        }
    }

    pub fn fill(
        &mut self,
        cell_origin: DensityContext,
        mut child: impl FnMut(DensityContext) -> f64,
    ) {
        self.cell_origin = cell_origin;
        let mut index = 0;
        for local_y in (0..self.cell_height).rev() {
            for local_x in 0..self.cell_width {
                for local_z in 0..self.cell_width {
                    self.values[index] = child(DensityContext {
                        x: cell_origin.x.wrapping_add(local_x as i32),
                        y: cell_origin.y.wrapping_add(local_y as i32),
                        z: cell_origin.z.wrapping_add(local_z as i32),
                    });
                    index += 1;
                }
            }
        }
    }

    pub fn sample(
        &self,
        context: DensityContext,
        runtime: DensityRuntime,
        mut child: impl FnMut(DensityContext) -> f64,
    ) -> Result<f64, DensityCacheError> {
        if !runtime.owner {
            return Ok(child(context));
        }
        if !runtime.interpolation_running {
            return Err(DensityCacheError::OutsideInterpolation);
        }
        let local_x = context.x.wrapping_sub(self.cell_origin.x);
        let local_y = context.y.wrapping_sub(self.cell_origin.y);
        let local_z = context.z.wrapping_sub(self.cell_origin.z);
        let (Ok(local_x), Ok(local_y), Ok(local_z)) = (
            usize::try_from(local_x),
            usize::try_from(local_y),
            usize::try_from(local_z),
        ) else {
            return Ok(child(context));
        };
        if local_x >= self.cell_width || local_y >= self.cell_height || local_z >= self.cell_width {
            return Ok(child(context));
        }
        let index = (self.cell_height - 1 - local_y) * self.cell_width * self.cell_width
            + local_x * self.cell_width
            + local_z;
        Ok(self.values[index])
    }
}

pub struct Interpolated {
    first_noise_x: i32,
    first_noise_z: i32,
    minimum_y: i32,
    cell_width: i32,
    cell_height: i32,
    count_xz: usize,
    count_y: usize,
    first_slice: Vec<f64>,
    second_slice: Vec<f64>,
    corners: [f64; 8],
    vertical: [f64; 4],
    horizontal: [f64; 2],
    value: f64,
    running: bool,
}

impl Interpolated {
    pub fn new(
        first_noise_x: i32,
        first_noise_z: i32,
        minimum_y: i32,
        cell_width: i32,
        cell_height: i32,
        count_xz: usize,
        count_y: usize,
    ) -> Self {
        let slice_size = (count_xz + 1) * (count_y + 1);
        Self {
            first_noise_x,
            first_noise_z,
            minimum_y,
            cell_width,
            cell_height,
            count_xz,
            count_y,
            first_slice: vec![0.0; slice_size],
            second_slice: vec![0.0; slice_size],
            corners: [0.0; 8],
            vertical: [0.0; 4],
            horizontal: [0.0; 2],
            value: 0.0,
            running: false,
        }
    }

    pub fn start(
        &mut self,
        child: &mut impl FnMut(DensityContext) -> f64,
    ) -> Result<(), DensityCacheError> {
        if self.running {
            return Err(DensityCacheError::InterpolationAlreadyRunning);
        }
        self.running = true;
        let layout = SliceLayout::from(&*self);
        fill_slice_values(&mut self.first_slice, self.first_noise_x, layout, child);
        Ok(())
    }

    pub fn advance_x(
        &mut self,
        cell_x: usize,
        child: &mut impl FnMut(DensityContext) -> f64,
    ) -> Result<(), DensityCacheError> {
        self.require_running()?;
        let layout = SliceLayout::from(&*self);
        fill_slice_values(
            &mut self.second_slice,
            self.first_noise_x.wrapping_add(cell_x as i32 + 1),
            layout,
            child,
        );
        Ok(())
    }

    pub fn select_cell(&mut self, cell_y: usize, cell_z: usize) -> Result<(), DensityCacheError> {
        self.require_running()?;
        let width_y = self.count_y + 1;
        let index = |z: usize, y: usize| z * width_y + y;
        self.corners = [
            self.first_slice[index(cell_z, cell_y)],
            self.first_slice[index(cell_z, cell_y + 1)],
            self.first_slice[index(cell_z + 1, cell_y)],
            self.first_slice[index(cell_z + 1, cell_y + 1)],
            self.second_slice[index(cell_z, cell_y)],
            self.second_slice[index(cell_z, cell_y + 1)],
            self.second_slice[index(cell_z + 1, cell_y)],
            self.second_slice[index(cell_z + 1, cell_y + 1)],
        ];
        Ok(())
    }

    pub fn update_y(&mut self, fraction: f64) {
        self.vertical = [
            lerp(fraction, self.corners[0], self.corners[1]),
            lerp(fraction, self.corners[2], self.corners[3]),
            lerp(fraction, self.corners[4], self.corners[5]),
            lerp(fraction, self.corners[6], self.corners[7]),
        ];
    }

    pub fn update_x(&mut self, fraction: f64) {
        self.horizontal = [
            lerp(fraction, self.vertical[0], self.vertical[2]),
            lerp(fraction, self.vertical[1], self.vertical[3]),
        ];
    }

    pub fn update_z(&mut self, fraction: f64) {
        self.value = lerp(fraction, self.horizontal[0], self.horizontal[1]);
    }

    pub fn sample(
        &self,
        context: DensityContext,
        runtime: DensityRuntime,
        mut child: impl FnMut(DensityContext) -> f64,
        cell_fraction: [f64; 3],
    ) -> Result<f64, DensityCacheError> {
        if !runtime.owner {
            return Ok(child(context));
        }
        self.require_running()?;
        if runtime.filling_cell {
            return Ok(lerp3(
                cell_fraction,
                [
                    self.corners[0],
                    self.corners[4],
                    self.corners[1],
                    self.corners[5],
                    self.corners[2],
                    self.corners[6],
                    self.corners[3],
                    self.corners[7],
                ],
            ));
        }
        Ok(self.value)
    }

    pub fn swap_slices(&mut self) -> Result<(), DensityCacheError> {
        self.require_running()?;
        std::mem::swap(&mut self.first_slice, &mut self.second_slice);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), DensityCacheError> {
        if !self.running {
            return Err(DensityCacheError::InterpolationNotRunning);
        }
        self.running = false;
        Ok(())
    }

    fn require_running(&self) -> Result<(), DensityCacheError> {
        if self.running {
            Ok(())
        } else {
            Err(DensityCacheError::OutsideInterpolation)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SliceLayout {
    first_noise_z: i32,
    minimum_y: i32,
    cell_width: i32,
    cell_height: i32,
    count_xz: usize,
    count_y: usize,
}

impl From<&Interpolated> for SliceLayout {
    fn from(interpolated: &Interpolated) -> Self {
        Self {
            first_noise_z: interpolated.first_noise_z,
            minimum_y: interpolated.minimum_y,
            cell_width: interpolated.cell_width,
            cell_height: interpolated.cell_height,
            count_xz: interpolated.count_xz,
            count_y: interpolated.count_y,
        }
    }
}

fn fill_slice_values(
    output: &mut [f64],
    noise_x: i32,
    layout: SliceLayout,
    child: &mut impl FnMut(DensityContext) -> f64,
) {
    let mut index = 0;
    for z in 0..=layout.count_xz {
        for y in 0..=layout.count_y {
            output[index] = child(DensityContext {
                x: noise_x.wrapping_mul(layout.cell_width),
                y: layout
                    .minimum_y
                    .wrapping_add((y as i32).wrapping_mul(layout.cell_height)),
                z: layout
                    .first_noise_z
                    .wrapping_add(z as i32)
                    .wrapping_mul(layout.cell_width),
            });
            index += 1;
        }
    }
}

fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

fn lerp3(fraction: [f64; 3], values: [f64; 8]) -> f64 {
    let [x, y, z] = fraction;
    lerp(
        z,
        lerp(
            x,
            lerp(y, values[0], values[2]),
            lerp(y, values[1], values[3]),
        ),
        lerp(
            x,
            lerp(y, values[4], values[6]),
            lerp(y, values[5], values[7]),
        ),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityCacheError {
    InterpolationAlreadyRunning,
    InterpolationNotRunning,
    OutsideInterpolation,
}
