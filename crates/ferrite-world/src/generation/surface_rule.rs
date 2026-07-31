//! Surface material conditions, lazy context, and first-match rules.

use std::collections::HashMap;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::provider::{HeightAnchor, HeightContext};
use crate::id::BlockStateId;

pub trait SurfaceEnvironment {
    type Biome: Copy + Eq;

    fn surface_noise(&mut self, x: i32, z: i32) -> f64;

    fn surface_depth_random(&mut self, x: i32, z: i32) -> f64;

    fn preliminary_surface(&mut self, x: i32, z: i32) -> i32;

    fn secondary_noise(&mut self, x: i32, z: i32) -> f64;

    fn condition_noise(&mut self, key: &str, position: BlockPos, is_3d: bool) -> f64;

    fn vertical_gradient_random(&mut self, name: &str, position: BlockPos) -> f32;

    fn biome(&mut self, position: BlockPos) -> Self::Biome;

    fn cold_enough_to_snow(
        &mut self,
        biome: Self::Biome,
        position: BlockPos,
        sea_level: i32,
    ) -> bool;

    fn surface_height(&mut self, local_x: u8, local_z: u8) -> i32;

    fn bandlands_state(&mut self, position: BlockPos) -> BlockStateId;
}

pub struct SurfaceContext<'a, E: SurfaceEnvironment> {
    environment: &'a mut E,
    height: HeightContext,
    sea_level: i32,
    position: BlockPos,
    surface_depth: i32,
    stone_depth_above: i32,
    stone_depth_below: i32,
    water_height: Option<i32>,
    preliminary_floor: i32,
    xz_epoch: u64,
    y_epoch: u64,
    secondary_cache: Option<(u64, f64)>,
    biome_cache: Option<(u64, E::Biome)>,
    noise_2d_cache: HashMap<String, (u64, f64)>,
    noise_3d_cache: HashMap<String, (u64, f64)>,
    preliminary_cell: Option<([i32; 2], [i32; 4])>,
}

impl<'a, E> SurfaceContext<'a, E>
where
    E: SurfaceEnvironment,
{
    pub fn new(environment: &'a mut E, height: HeightContext, sea_level: i32) -> Self {
        Self {
            environment,
            height,
            sea_level,
            position: BlockPos::new(0, 0, 0),
            surface_depth: 0,
            stone_depth_above: 0,
            stone_depth_below: 0,
            water_height: None,
            preliminary_floor: 0,
            xz_epoch: 0,
            y_epoch: 0,
            secondary_cache: None,
            biome_cache: None,
            noise_2d_cache: HashMap::new(),
            noise_3d_cache: HashMap::new(),
            preliminary_cell: None,
        }
    }

    pub fn update_xz(&mut self, x: i32, z: i32) {
        self.xz_epoch = self.xz_epoch.wrapping_add(1);
        self.position = BlockPos::new(x, self.position.y, z);
        let depth = self.environment.surface_noise(x, z) * 2.75
            + 3.0
            + self.environment.surface_depth_random(x, z) * 0.25;
        self.surface_depth = depth as i32;
        let preliminary = self.interpolated_preliminary_surface(x, z);
        self.preliminary_floor = preliminary.wrapping_add(self.surface_depth).wrapping_sub(8);
    }

    pub fn update_y(
        &mut self,
        y: i32,
        stone_depth_above: i32,
        stone_depth_below: i32,
        water_height: Option<i32>,
    ) {
        self.y_epoch = self.y_epoch.wrapping_add(1);
        self.position.y = y;
        self.stone_depth_above = stone_depth_above;
        self.stone_depth_below = stone_depth_below;
        self.water_height = water_height;
    }

    pub fn environment_mut(&mut self) -> &mut E {
        self.environment
    }

    pub fn position(&self) -> BlockPos {
        self.position
    }

    pub fn surface_depth(&self) -> i32 {
        self.surface_depth
    }

    pub fn preliminary_floor(&self) -> i32 {
        self.preliminary_floor
    }

    fn interpolated_preliminary_surface(&mut self, x: i32, z: i32) -> i32 {
        let cell = [x >> 4, z >> 4];
        let corners = if let Some((cached_cell, corners)) = self.preliminary_cell {
            if cached_cell == cell {
                corners
            } else {
                self.load_preliminary_cell(cell)
            }
        } else {
            self.load_preliminary_cell(cell)
        };
        let x_fraction = f64::from((x & 15) as f32 / 16.0);
        let z_fraction = f64::from((z & 15) as f32 / 16.0);
        let north = lerp(x_fraction, f64::from(corners[0]), f64::from(corners[1]));
        let south = lerp(x_fraction, f64::from(corners[2]), f64::from(corners[3]));
        lerp(z_fraction, north, south).floor() as i32
    }

    fn load_preliminary_cell(&mut self, cell: [i32; 2]) -> [i32; 4] {
        let minimum_x = cell[0].wrapping_mul(16);
        let minimum_z = cell[1].wrapping_mul(16);
        let corners = [
            self.environment.preliminary_surface(minimum_x, minimum_z),
            self.environment
                .preliminary_surface(minimum_x.wrapping_add(16), minimum_z),
            self.environment
                .preliminary_surface(minimum_x, minimum_z.wrapping_add(16)),
            self.environment
                .preliminary_surface(minimum_x.wrapping_add(16), minimum_z.wrapping_add(16)),
        ];
        self.preliminary_cell = Some((cell, corners));
        corners
    }

    fn secondary(&mut self) -> f64 {
        if let Some((epoch, value)) = self.secondary_cache
            && epoch == self.xz_epoch
        {
            return value;
        }
        let value = self
            .environment
            .secondary_noise(self.position.x, self.position.z);
        self.secondary_cache = Some((self.xz_epoch, value));
        value
    }

    fn noise(&mut self, key: &str, is_3d: bool) -> f64 {
        let epoch = if is_3d { self.y_epoch } else { self.xz_epoch };
        let cache = if is_3d {
            &mut self.noise_3d_cache
        } else {
            &mut self.noise_2d_cache
        };
        if let Some((cached_epoch, value)) = cache.get(key)
            && *cached_epoch == epoch
        {
            return *value;
        }
        let position = if is_3d {
            self.position
        } else {
            BlockPos::new(self.position.x, 0, self.position.z)
        };
        let value = self.environment.condition_noise(key, position, is_3d);
        cache.insert(key.to_owned(), (epoch, value));
        value
    }

    fn biome(&mut self) -> E::Biome {
        if let Some((epoch, biome)) = self.biome_cache
            && epoch == self.y_epoch
        {
            return biome;
        }
        let biome = self.environment.biome(self.position);
        self.biome_cache = Some((self.y_epoch, biome));
        biome
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceCondition<B> {
    Biome(Vec<B>),
    BiomeAlways,
    BiomeNever,
    NoiseThreshold {
        key: String,
        minimum: f64,
        maximum: f64,
        is_3d: bool,
    },
    VerticalGradient {
        random_name: String,
        true_at_and_below: HeightAnchor,
        false_at_and_above: HeightAnchor,
    },
    Not(Box<Self>),
    YAbove {
        anchor: HeightAnchor,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    Water {
        offset: i32,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    StoneDepth {
        surface: StoneSurface,
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
    },
    Hole,
    Steep,
    Temperature,
    AbovePreliminarySurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoneSurface {
    Floor,
    Ceiling,
}

impl<B> SurfaceCondition<B>
where
    B: Copy + Eq,
{
    pub fn test<E>(&self, context: &mut SurfaceContext<'_, E>) -> bool
    where
        E: SurfaceEnvironment<Biome = B>,
    {
        match self {
            Self::Biome(biomes) => biomes.contains(&context.biome()),
            Self::BiomeAlways => true,
            Self::BiomeNever => false,
            Self::NoiseThreshold {
                key,
                minimum,
                maximum,
                is_3d,
            } => {
                let value = context.noise(key, *is_3d);
                value >= *minimum && value <= *maximum
            }
            Self::VerticalGradient {
                random_name,
                true_at_and_below,
                false_at_and_above,
            } => {
                let lower = true_at_and_below
                    .resolve(context.height)
                    .expect("validated surface height anchor");
                let upper = false_at_and_above
                    .resolve(context.height)
                    .expect("validated surface height anchor");
                if context.position.y <= lower {
                    true
                } else if context.position.y >= upper {
                    false
                } else {
                    let probability =
                        1.0 - (context.position.y - lower) as f32 / (upper - lower) as f32;
                    context
                        .environment
                        .vertical_gradient_random(random_name, context.position)
                        < probability
                }
            }
            Self::Not(condition) => !condition.test(context),
            Self::YAbove {
                anchor,
                surface_depth_multiplier,
                add_stone_depth,
            } => {
                let left = context.position.y.wrapping_add(if *add_stone_depth {
                    context.stone_depth_above
                } else {
                    0
                });
                let right = anchor
                    .resolve(context.height)
                    .expect("validated surface height anchor")
                    .wrapping_add(
                        context
                            .surface_depth
                            .wrapping_mul(*surface_depth_multiplier),
                    );
                left >= right
            }
            Self::Water {
                offset,
                surface_depth_multiplier,
                add_stone_depth,
            } => {
                let Some(water_height) = context.water_height else {
                    return true;
                };
                let left = context.position.y.wrapping_add(if *add_stone_depth {
                    context.stone_depth_above
                } else {
                    0
                });
                let right = water_height.wrapping_add(*offset).wrapping_add(
                    context
                        .surface_depth
                        .wrapping_mul(*surface_depth_multiplier),
                );
                left >= right
            }
            Self::StoneDepth {
                surface,
                offset,
                add_surface_depth,
                secondary_depth_range,
            } => {
                let depth = match surface {
                    StoneSurface::Floor => context.stone_depth_above,
                    StoneSurface::Ceiling => context.stone_depth_below,
                };
                let secondary = if *secondary_depth_range == 0 {
                    0
                } else {
                    (map(
                        context.secondary(),
                        -1.0,
                        1.0,
                        0.0,
                        f64::from(*secondary_depth_range),
                    )) as i32
                };
                let threshold = 1_i32
                    .wrapping_add(*offset)
                    .wrapping_add(if *add_surface_depth {
                        context.surface_depth
                    } else {
                        0
                    })
                    .wrapping_add(secondary);
                depth <= threshold
            }
            Self::Hole => context.surface_depth <= 0,
            Self::Steep => {
                let local_x = (context.position.x & 15) as u8;
                let local_z = (context.position.z & 15) as u8;
                let south = context
                    .environment
                    .surface_height(local_x, local_z.saturating_add(1).min(15));
                let north = context
                    .environment
                    .surface_height(local_x, local_z.saturating_sub(1));
                if south >= north.wrapping_add(4) {
                    true
                } else {
                    let west = context
                        .environment
                        .surface_height(local_x.saturating_sub(1), local_z);
                    let east = context
                        .environment
                        .surface_height(local_x.saturating_add(1).min(15), local_z);
                    west >= east.wrapping_add(4)
                }
            }
            Self::Temperature => {
                let biome = context.biome();
                context
                    .environment
                    .cold_enough_to_snow(biome, context.position, context.sea_level)
            }
            Self::AbovePreliminarySurface => context.position.y >= context.preliminary_floor,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceRule<B> {
    Block(BlockStateId),
    Condition {
        condition: SurfaceCondition<B>,
        follow_up: Box<Self>,
    },
    Sequence(Vec<Self>),
    Bandlands,
}

impl<B> SurfaceRule<B>
where
    B: Copy + Eq,
{
    pub fn evaluate<E>(&self, context: &mut SurfaceContext<'_, E>) -> Option<BlockStateId>
    where
        E: SurfaceEnvironment<Biome = B>,
    {
        match self {
            Self::Block(state) => Some(*state),
            Self::Condition {
                condition,
                follow_up,
            } => condition
                .test(context)
                .then(|| follow_up.evaluate(context))
                .flatten(),
            Self::Sequence(rules) => rules.iter().find_map(|rule| rule.evaluate(context)),
            Self::Bandlands => Some(context.environment.bandlands_state(context.position)),
        }
    }
}

fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

fn map(value: f64, from_minimum: f64, from_maximum: f64, to_minimum: f64, to_maximum: f64) -> f64 {
    to_minimum + (value - from_minimum) / (from_maximum - from_minimum) * (to_maximum - to_minimum)
}
