//! Noise-setting validation, material precedence, and chunk fill traversal.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::aquifer::AquiferResolver;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseSettings {
    pub minimum_y: i32,
    pub height: u32,
    pub horizontal_size: u8,
    pub vertical_size: u8,
}

impl NoiseSettings {
    pub fn validate(
        self,
        dimension_minimum_y: i32,
        dimension_maximum_y: i32,
    ) -> Result<Self, NoiseSettingsError> {
        let height = i32::try_from(self.height).map_err(|_| NoiseSettingsError::HeightTooLarge)?;
        let maximum = self
            .minimum_y
            .checked_add(height)
            .ok_or(NoiseSettingsError::HeightTooLarge)?;
        if !(1..=4).contains(&self.horizontal_size) || !(1..=4).contains(&self.vertical_size) {
            return Err(NoiseSettingsError::InvalidCellSize);
        }
        if self.minimum_y % 16 != 0 || height % 16 != 0 {
            return Err(NoiseSettingsError::UnalignedHeight);
        }
        if self.minimum_y < dimension_minimum_y || maximum > dimension_maximum_y.saturating_add(1) {
            return Err(NoiseSettingsError::OutsideDimension);
        }
        Ok(self)
    }

    pub fn clamp_to(self, accessor_minimum_y: i32, accessor_maximum_y: i32) -> ClampedNoise {
        let setting_maximum = self.minimum_y.wrapping_add(self.height as i32);
        let minimum_y = self.minimum_y.max(accessor_minimum_y);
        let height = setting_maximum
            .min(accessor_maximum_y.saturating_add(1))
            .wrapping_sub(minimum_y);
        ClampedNoise {
            minimum_y,
            height,
            cell_width: i32::from(self.horizontal_size) * 4,
            cell_height: i32::from(self.vertical_size) * 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClampedNoise {
    pub minimum_y: i32,
    pub height: i32,
    pub cell_width: i32,
    pub cell_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseMaterial {
    pub state: BlockStateId,
    pub schedule_fluid_update: bool,
}

pub fn resolve_material(
    aquifer: &mut impl AquiferResolver,
    position: BlockPos,
    density: f64,
    ore: impl FnOnce() -> Option<BlockStateId>,
    default_block: BlockStateId,
) -> NoiseMaterial {
    if let Some(state) = aquifer.compute_substance(position, density) {
        return NoiseMaterial {
            state,
            schedule_fluid_update: aquifer.should_schedule_fluid_update(),
        };
    }
    NoiseMaterial {
        state: ore().unwrap_or(default_block),
        schedule_fluid_update: false,
    }
}

pub trait NoiseFillWorld {
    type Error;

    fn accessor_minimum_y(&self) -> i32;

    fn accessor_maximum_y(&self) -> i32;

    fn chunk_minimum_x(&self) -> i32;

    fn chunk_minimum_z(&self) -> i32;

    fn acquire_section(&mut self, section_y: i32) -> Result<(), Self::Error>;

    fn release_section(&mut self, section_y: i32);

    fn start_interpolation(&mut self) -> Result<(), Self::Error>;

    fn advance_cell_x(&mut self, cell_x: i32) -> Result<(), Self::Error>;

    fn select_cell(&mut self, cell_y: i32, cell_z: i32) -> Result<(), Self::Error>;

    fn update_for_y(&mut self, y_fraction: f64) -> Result<(), Self::Error>;

    fn update_for_x(&mut self, x_fraction: f64) -> Result<(), Self::Error>;

    fn update_for_z(&mut self, z_fraction: f64) -> Result<(), Self::Error>;

    fn material(&mut self, position: BlockPos) -> Result<NoiseMaterial, Self::Error>;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn has_nonempty_fluid(&self, state: BlockStateId) -> bool;

    fn write_block(&mut self, position: BlockPos, state: BlockStateId) -> Result<(), Self::Error>;

    fn update_ocean_floor_heightmap(&mut self, position: BlockPos, state: BlockStateId);

    fn update_world_surface_heightmap(&mut self, position: BlockPos, state: BlockStateId);

    fn mark_for_postprocessing(&mut self, position: BlockPos);

    fn swap_slices(&mut self) -> Result<(), Self::Error>;

    fn stop_interpolation(&mut self) -> Result<(), Self::Error>;
}

pub fn fill_noise_chunk<W>(world: &mut W, settings: NoiseSettings) -> Result<bool, W::Error>
where
    W: NoiseFillWorld,
{
    let clamped = settings.clamp_to(world.accessor_minimum_y(), world.accessor_maximum_y());
    let cell_count_y = clamped.height / clamped.cell_height;
    if cell_count_y <= 0 {
        return Ok(false);
    }
    let top_y = clamped
        .minimum_y
        .wrapping_add(cell_count_y.wrapping_mul(clamped.cell_height))
        .wrapping_sub(1);
    let bottom_section = clamped.minimum_y.div_euclid(16);
    let top_section = top_y.div_euclid(16);
    let mut acquired = Vec::new();
    for section in (bottom_section..=top_section).rev() {
        if let Err(error) = world.acquire_section(section) {
            release_sections(world, &acquired);
            return Err(error);
        }
        acquired.push(section);
    }

    let result = fill_locked(world, clamped, cell_count_y);
    release_sections(world, &acquired);
    result.map(|()| true)
}

fn fill_locked<W>(world: &mut W, clamped: ClampedNoise, cell_count_y: i32) -> Result<(), W::Error>
where
    W: NoiseFillWorld,
{
    world.start_interpolation()?;
    let fill_result = fill_cells(world, clamped, cell_count_y);
    let stop_result = world.stop_interpolation();
    match fill_result {
        Err(error) => Err(error),
        Ok(()) => stop_result,
    }
}

fn fill_cells<W>(world: &mut W, clamped: ClampedNoise, cell_count_y: i32) -> Result<(), W::Error>
where
    W: NoiseFillWorld,
{
    let cell_count_xz = 16 / clamped.cell_width;
    for cell_x in 0..cell_count_xz {
        world.advance_cell_x(cell_x)?;
        for cell_z in 0..cell_count_xz {
            for cell_y in (0..cell_count_y).rev() {
                world.select_cell(cell_y, cell_z)?;
                for local_y in (0..clamped.cell_height).rev() {
                    world.update_for_y(f64::from(local_y) / f64::from(clamped.cell_height))?;
                    let y = clamped
                        .minimum_y
                        .wrapping_add(cell_y.wrapping_mul(clamped.cell_height))
                        .wrapping_add(local_y);
                    for local_x in 0..clamped.cell_width {
                        world.update_for_x(f64::from(local_x) / f64::from(clamped.cell_width))?;
                        let x = world
                            .chunk_minimum_x()
                            .wrapping_add(cell_x.wrapping_mul(clamped.cell_width))
                            .wrapping_add(local_x);
                        for local_z in 0..clamped.cell_width {
                            world
                                .update_for_z(f64::from(local_z) / f64::from(clamped.cell_width))?;
                            let z = world
                                .chunk_minimum_z()
                                .wrapping_add(cell_z.wrapping_mul(clamped.cell_width))
                                .wrapping_add(local_z);
                            let position = BlockPos::new(x, y, z);
                            let material = world.material(position)?;
                            if world.is_air(material.state) {
                                continue;
                            }
                            world.write_block(position, material.state)?;
                            world.update_ocean_floor_heightmap(position, material.state);
                            world.update_world_surface_heightmap(position, material.state);
                            if material.schedule_fluid_update
                                && world.has_nonempty_fluid(material.state)
                            {
                                world.mark_for_postprocessing(position);
                            }
                        }
                    }
                }
            }
        }
        world.swap_slices()?;
    }
    Ok(())
}

fn release_sections(world: &mut impl NoiseFillWorld, acquired: &[i32]) {
    for section in acquired {
        world.release_section(*section);
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NoiseSettingsError {
    #[error("noise horizontal and vertical sizes must be one through four")]
    InvalidCellSize,
    #[error("noise minimum Y and height must be multiples of sixteen")]
    UnalignedHeight,
    #[error("noise settings exceed the dimension height range")]
    OutsideDimension,
    #[error("noise height does not fit a Java integer")]
    HeightTooLarge,
}
