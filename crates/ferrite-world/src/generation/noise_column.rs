//! One-column noise queries used by base-height and base-column APIs.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::noise_fill::NoiseSettings;
use crate::id::BlockStateId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoiseColumn {
    pub minimum_y: i32,
    pub states: Vec<BlockStateId>,
}

pub trait NoiseColumnSampler {
    type Error;

    fn accessor_minimum_y(&self) -> i32;

    fn accessor_maximum_y(&self) -> i32;

    fn start_interpolation(
        &mut self,
        first_cell_x: i32,
        first_cell_z: i32,
    ) -> Result<(), Self::Error>;

    fn advance_cell_x(&mut self) -> Result<(), Self::Error>;

    fn select_cell(&mut self, cell_y: i32) -> Result<(), Self::Error>;

    fn update_for_y(&mut self, fraction: f64) -> Result<(), Self::Error>;

    fn update_for_x(&mut self, fraction: f64) -> Result<(), Self::Error>;

    fn update_for_z(&mut self, fraction: f64) -> Result<(), Self::Error>;

    fn material(&mut self, position: BlockPos) -> Result<BlockStateId, Self::Error>;

    fn stop_interpolation(&mut self) -> Result<(), Self::Error>;
}

pub fn base_column<S: NoiseColumnSampler>(
    sampler: &mut S,
    settings: NoiseSettings,
    block_x: i32,
    block_z: i32,
) -> Result<Option<NoiseColumn>, S::Error> {
    let Some(layout) = ColumnLayout::new(sampler, settings, block_x, block_z) else {
        return Ok(None);
    };
    let mut states = vec![
        BlockStateId::new(0);
        usize::try_from(layout.height)
            .expect("positive clamped height fits usize")
    ];
    run_column(sampler, layout, |position, state| {
        let index = usize::try_from(position.y - layout.minimum_y)
            .expect("sampled Y lies inside the column");
        states[index] = state;
        false
    })?;
    Ok(Some(NoiseColumn {
        minimum_y: layout.minimum_y,
        states,
    }))
}

pub fn base_height<S, P>(
    sampler: &mut S,
    settings: NoiseSettings,
    block_x: i32,
    block_z: i32,
    opaque: P,
) -> Result<i32, S::Error>
where
    S: NoiseColumnSampler,
    P: Fn(BlockStateId) -> bool,
{
    let Some(layout) = ColumnLayout::new(sampler, settings, block_x, block_z) else {
        return Ok(sampler.accessor_minimum_y());
    };
    let mut result = layout.minimum_y;
    run_column(sampler, layout, |position, state| {
        if opaque(state) {
            result = position.y.wrapping_add(1);
            true
        } else {
            false
        }
    })?;
    Ok(result)
}

pub fn interpolated_noise_value(
    settings: NoiseSettings,
    context: BlockPos,
    sample: impl FnOnce(BlockPos) -> f64,
) -> f64 {
    let maximum = settings.minimum_y.wrapping_add(settings.height as i32);
    if context.y < settings.minimum_y || context.y >= maximum {
        f64::NAN
    } else {
        sample(context)
    }
}

#[derive(Debug, Clone, Copy)]
struct ColumnLayout {
    minimum_y: i32,
    height: i32,
    cell_height: i32,
    cell_count_y: i32,
    first_cell_x: i32,
    first_cell_z: i32,
    x_fraction: f64,
    z_fraction: f64,
    block_x: i32,
    block_z: i32,
}

impl ColumnLayout {
    fn new<S: NoiseColumnSampler>(
        sampler: &S,
        settings: NoiseSettings,
        block_x: i32,
        block_z: i32,
    ) -> Option<Self> {
        let clamped = settings.clamp_to(sampler.accessor_minimum_y(), sampler.accessor_maximum_y());
        let cell_count_y = clamped.height / clamped.cell_height;
        (cell_count_y > 0).then(|| Self {
            minimum_y: clamped.minimum_y,
            height: cell_count_y * clamped.cell_height,
            cell_height: clamped.cell_height,
            cell_count_y,
            first_cell_x: block_x.div_euclid(clamped.cell_width),
            first_cell_z: block_z.div_euclid(clamped.cell_width),
            x_fraction: f64::from(block_x.rem_euclid(clamped.cell_width))
                / f64::from(clamped.cell_width),
            z_fraction: f64::from(block_z.rem_euclid(clamped.cell_width))
                / f64::from(clamped.cell_width),
            block_x,
            block_z,
        })
    }
}

fn run_column<S>(
    sampler: &mut S,
    layout: ColumnLayout,
    mut visit: impl FnMut(BlockPos, BlockStateId) -> bool,
) -> Result<(), S::Error>
where
    S: NoiseColumnSampler,
{
    sampler.start_interpolation(layout.first_cell_x, layout.first_cell_z)?;
    let body = run_started(sampler, layout, &mut visit);
    let stopped = sampler.stop_interpolation();
    match body {
        Err(error) => Err(error),
        Ok(()) => stopped,
    }
}

fn run_started<S>(
    sampler: &mut S,
    layout: ColumnLayout,
    visit: &mut impl FnMut(BlockPos, BlockStateId) -> bool,
) -> Result<(), S::Error>
where
    S: NoiseColumnSampler,
{
    sampler.advance_cell_x()?;
    for cell_y in (0..layout.cell_count_y).rev() {
        sampler.select_cell(cell_y)?;
        for local_y in (0..layout.cell_height).rev() {
            sampler.update_for_y(f64::from(local_y) / f64::from(layout.cell_height))?;
            sampler.update_for_x(layout.x_fraction)?;
            sampler.update_for_z(layout.z_fraction)?;
            let y = layout
                .minimum_y
                .wrapping_add(cell_y.wrapping_mul(layout.cell_height))
                .wrapping_add(local_y);
            let position = BlockPos::new(layout.block_x, y, layout.block_z);
            let state = sampler.material(position)?;
            if visit(position, state) {
                return Ok(());
            }
        }
    }
    Ok(())
}
