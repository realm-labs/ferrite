//! Concrete placed-feature modifiers and their ordered world-query boundary.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::resource::ResourceId;

use crate::generation::feature::placement::{PlacementError, PlacementModifier};
use crate::generation::feature::predicate::{BlockPredicate, PredicateWorld};
use crate::generation::feature::provider::{
    HeightAnchor, HeightContext, IntProvider, uniform_height,
};
use crate::generation::feature::random::GenerationRandom;
use crate::generation::status::GenerationHeightmap;

pub const MAX_MODIFIER_OUTPUTS: u32 = 4_096;

pub trait PlacementWorld: PredicateWorld {
    fn minimum_y(&self) -> i32;

    fn generation_depth(&self) -> i32;

    fn height(&self, heightmap: GenerationHeightmap, x: i32, z: i32) -> i32;

    fn biome_contains_feature(&self, position: BlockPos, feature: &ResourceId) -> bool;

    fn biome_info_noise(&self, x: f64, z: f64) -> f64;

    fn is_outside_build_height(&self, y: i32) -> bool {
        let Some(maximum_y) = self.minimum_y().checked_add(self.generation_depth()) else {
            return true;
        };
        y < self.minimum_y() || y >= maximum_y
    }
}

#[derive(Debug)]
pub struct PlacementContext<'a, W> {
    world: &'a W,
    top_feature: Option<&'a ResourceId>,
}

impl<W> Clone for PlacementContext<'_, W> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<W> Copy for PlacementContext<'_, W> {}

impl<'a, W> PlacementContext<'a, W> {
    #[must_use]
    pub const fn plain(world: &'a W) -> Self {
        Self {
            world,
            top_feature: None,
        }
    }

    #[must_use]
    pub const fn with_biome_check(world: &'a W, top_feature: &'a ResourceId) -> Self {
        Self {
            world,
            top_feature: Some(top_feature),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalDirection {
    Up,
    Down,
}

impl VerticalDirection {
    const fn step(self) -> i32 {
        match self {
            Self::Up => 1,
            Self::Down => -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlacementModifierSpec {
    BlockPredicateFilter(BlockPredicate),
    RarityFilter {
        chance: NonZeroU32,
    },
    InSquare,
    Heightmap {
        heightmap: GenerationHeightmap,
    },
    Biome,
    Count {
        count: IntProvider,
    },
    RandomOffset {
        horizontal: IntProvider,
        vertical: IntProvider,
    },
    SurfaceWaterDepth {
        maximum_water_depth: i32,
    },
    NoiseThresholdCount {
        noise_level: f64,
        below_noise: u32,
        above_noise: u32,
    },
    HeightRange {
        minimum: HeightAnchor,
        maximum: HeightAnchor,
    },
    EnvironmentScan {
        direction: VerticalDirection,
        maximum_steps: u8,
        target: BlockPredicate,
        allowed_search: BlockPredicate,
    },
}

impl PlacementModifierSpec {
    pub fn apply<R, W>(
        &self,
        context: PlacementContext<'_, W>,
        input: BlockPos,
        random: &mut R,
        output: &mut Vec<BlockPos>,
    ) -> Result<(), PlacementError>
    where
        R: GenerationRandom,
        W: PlacementWorld,
    {
        match self {
            Self::BlockPredicateFilter(predicate) => {
                if predicate.test(context.world, input)? {
                    output.push(input);
                }
            }
            Self::RarityFilter { chance } => {
                if random.next_f32() < 1.0_f32 / chance.get() as f32 {
                    output.push(input);
                }
            }
            Self::InSquare => {
                let x = random.next_u32(NonZeroU32::new(16).expect("sixteen is nonzero"));
                let z = random.next_u32(NonZeroU32::new(16).expect("sixteen is nonzero"));
                output.push(offset(input, x as i32, 0, z as i32)?);
            }
            Self::Heightmap { heightmap } => {
                let height = context.world.height(*heightmap, input.x, input.z);
                if height > context.world.minimum_y() {
                    output.push(BlockPos::new(input.x, height, input.z));
                }
            }
            Self::Biome => {
                let top_feature = context
                    .top_feature
                    .ok_or(PlacementError::MissingTopFeature)?;
                if context.world.biome_contains_feature(input, top_feature) {
                    output.push(input);
                }
            }
            Self::Count { count } => {
                let count = bounded_count(count.sample(random)?)?;
                for _ in 0..count {
                    output.push(input);
                }
            }
            Self::RandomOffset {
                horizontal,
                vertical,
            } => {
                let x = horizontal.sample(random)?;
                let y = vertical.sample(random)?;
                let z = horizontal.sample(random)?;
                output.push(offset(input, x, y, z)?);
            }
            Self::SurfaceWaterDepth {
                maximum_water_depth,
            } => {
                let floor = context
                    .world
                    .height(GenerationHeightmap::OceanFloor, input.x, input.z);
                let surface =
                    context
                        .world
                        .height(GenerationHeightmap::WorldSurface, input.x, input.z);
                if surface.saturating_sub(floor) <= *maximum_water_depth {
                    output.push(input);
                }
            }
            Self::NoiseThresholdCount {
                noise_level,
                below_noise,
                above_noise,
            } => {
                let noise = context
                    .world
                    .biome_info_noise(f64::from(input.x) / 200.0, f64::from(input.z) / 200.0);
                let count = if noise < *noise_level {
                    *below_noise
                } else {
                    *above_noise
                };
                ensure_count_bound(count)?;
                for _ in 0..count {
                    output.push(input);
                }
            }
            Self::HeightRange { minimum, maximum } => {
                let height = uniform_height(
                    *minimum,
                    *maximum,
                    HeightContext {
                        minimum_y: context.world.minimum_y(),
                        depth: context.world.generation_depth(),
                    },
                    random,
                )?;
                output.push(BlockPos::new(input.x, height, input.z));
            }
            Self::EnvironmentScan {
                direction,
                maximum_steps,
                target,
                allowed_search,
            } => {
                environment_scan(
                    context.world,
                    input,
                    *direction,
                    *maximum_steps,
                    target,
                    allowed_search,
                    output,
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundPlacementModifier<'a, W> {
    spec: &'a PlacementModifierSpec,
    context: PlacementContext<'a, W>,
}

impl<'a, W> BoundPlacementModifier<'a, W> {
    #[must_use]
    pub const fn new(spec: &'a PlacementModifierSpec, context: PlacementContext<'a, W>) -> Self {
        Self { spec, context }
    }
}

impl<R, W> PlacementModifier<R> for BoundPlacementModifier<'_, W>
where
    R: GenerationRandom,
    W: PlacementWorld,
{
    fn apply(
        &self,
        input: BlockPos,
        random: &mut R,
        output: &mut Vec<BlockPos>,
    ) -> Result<(), PlacementError> {
        self.spec.apply(self.context, input, random, output)
    }
}

fn environment_scan<W: PlacementWorld>(
    world: &W,
    input: BlockPos,
    direction: VerticalDirection,
    maximum_steps: u8,
    target: &BlockPredicate,
    allowed_search: &BlockPredicate,
    output: &mut Vec<BlockPos>,
) -> Result<(), PlacementError> {
    if !(1..=32).contains(&maximum_steps) {
        return Err(PlacementError::InvalidEnvironmentScanSteps);
    }
    if !allowed_search.test(world, input)? {
        return Ok(());
    }
    let mut cursor = input;
    for _ in 0..maximum_steps {
        if target.test(world, cursor)? {
            output.push(cursor);
            return Ok(());
        }
        cursor = offset(cursor, 0, direction.step(), 0)?;
        if world.is_outside_build_height(cursor.y) {
            return Ok(());
        }
        if !allowed_search.test(world, cursor)? {
            break;
        }
    }
    if target.test(world, cursor)? {
        output.push(cursor);
    }
    Ok(())
}

fn bounded_count(count: i32) -> Result<u32, PlacementError> {
    let count = u32::try_from(count).map_err(|_| PlacementError::CountOutOfRange {
        count,
        maximum: MAX_MODIFIER_OUTPUTS,
    })?;
    ensure_count_bound(count)?;
    Ok(count)
}

fn ensure_count_bound(count: u32) -> Result<(), PlacementError> {
    if count <= MAX_MODIFIER_OUTPUTS {
        Ok(())
    } else {
        Err(PlacementError::CountOutOfRange {
            count: i32::try_from(count).unwrap_or(i32::MAX),
            maximum: MAX_MODIFIER_OUTPUTS,
        })
    }
}

fn offset(input: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, PlacementError> {
    Ok(BlockPos::new(
        input
            .x
            .checked_add(x)
            .ok_or(PlacementError::PositionOverflow)?,
        input
            .y
            .checked_add(y)
            .ok_or(PlacementError::PositionOverflow)?,
        input
            .z
            .checked_add(z)
            .ok_or(PlacementError::PositionOverflow)?,
    ))
}
