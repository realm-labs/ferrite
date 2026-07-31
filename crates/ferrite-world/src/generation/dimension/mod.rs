//! Dimension records and the gates derived from dimension type and level identity.

pub mod clock;
pub mod environment;
pub mod spawn;
pub mod timeline;

use environment::{AttributeMap, locked_dimension_attributes};

/// Lowest dimension minimum accepted by the 26.2 dimension codec.
pub const MIN_Y: i32 = -2_032;
/// Highest block coordinate admitted by a valid dimension record.
pub const MAX_Y: i32 = 2_031;
/// Largest storage height admitted by the 26.2 dimension codec.
pub const MAX_HEIGHT: u32 = 4_064;
/// Smallest positive coordinate scale admitted by the codec.
pub const MIN_COORDINATE_SCALE: f64 = 0.000_009_999_999_747_378_752;
/// Largest coordinate scale admitted by the codec.
pub const MAX_COORDINATE_SCALE: f64 = 30_000_000.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Skybox {
    Overworld,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CardinalLight {
    Default,
    Nether,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpawnLightLevel {
    Constant(u8),
    UniformInclusive { minimum: u8, maximum: u8 },
}

impl SpawnLightLevel {
    /// Samples the provider. The callback is invoked only by a uniform provider.
    pub fn sample(self, mut next_bounded: impl FnMut(u32) -> u32) -> u8 {
        match self {
            Self::Constant(value) => value,
            Self::UniformInclusive { minimum, maximum } => {
                let width = u32::from(maximum - minimum) + 1;
                minimum + next_bounded(width) as u8
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DimensionType {
    pub has_fixed_time: bool,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub has_ender_dragon_fight: bool,
    pub coordinate_scale: f64,
    pub min_y: i32,
    pub height: u32,
    pub logical_height: u32,
    pub infiniburn: String,
    pub ambient_light: f32,
    pub monster_spawn_block_light_limit: u8,
    pub monster_spawn_light_level: SpawnLightLevel,
    pub skybox: Option<Skybox>,
    pub cardinal_light: CardinalLight,
    pub attributes: AttributeMap,
    pub timelines: Vec<String>,
    pub default_clock: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DimensionTypeError {
    #[error("coordinate scale {0} is outside the codec range")]
    CoordinateScale(String),
    #[error("minimum Y {0} is outside the codec range or is not a multiple of 16")]
    MinimumY(i32),
    #[error("height {0} is outside the codec range or is not a multiple of 16")]
    Height(u32),
    #[error("logical height {logical} exceeds storage height {height}")]
    LogicalHeight { logical: u32, height: u32 },
    #[error("dimension top {top} exceeds MAX_Y + 1")]
    TopOverflow { top: i64 },
    #[error("monster block-light limit {0} exceeds 15")]
    BlockLightLimit(u8),
    #[error("spawn-light provider is outside 0..=15")]
    SpawnLight,
}

impl DimensionType {
    /// Validates a post-codec record before it becomes active runtime state.
    pub fn validate(record: Self) -> Result<Self, DimensionTypeError> {
        if !record.coordinate_scale.is_finite()
            || !(MIN_COORDINATE_SCALE..=MAX_COORDINATE_SCALE).contains(&record.coordinate_scale)
        {
            return Err(DimensionTypeError::CoordinateScale(
                record.coordinate_scale.to_string(),
            ));
        }
        if !(MIN_Y..=MAX_Y).contains(&record.min_y) || record.min_y.rem_euclid(16) != 0 {
            return Err(DimensionTypeError::MinimumY(record.min_y));
        }
        if !(16..=MAX_HEIGHT).contains(&record.height) || !record.height.is_multiple_of(16) {
            return Err(DimensionTypeError::Height(record.height));
        }
        if record.logical_height > record.height {
            return Err(DimensionTypeError::LogicalHeight {
                logical: record.logical_height,
                height: record.height,
            });
        }
        let top = i64::from(record.min_y) + i64::from(record.height);
        if top > i64::from(MAX_Y) + 1 {
            return Err(DimensionTypeError::TopOverflow { top });
        }
        if record.monster_spawn_block_light_limit > 15 {
            return Err(DimensionTypeError::BlockLightLimit(
                record.monster_spawn_block_light_limit,
            ));
        }
        match record.monster_spawn_light_level {
            SpawnLightLevel::Constant(value) if value > 15 => {
                return Err(DimensionTypeError::SpawnLight);
            }
            SpawnLightLevel::UniformInclusive { minimum, maximum }
                if minimum > maximum || maximum > 15 =>
            {
                return Err(DimensionTypeError::SpawnLight);
            }
            _ => {}
        }
        Ok(record)
    }

    pub fn locked(kind: LockedDimension) -> Self {
        let (
            fixed,
            skylight,
            ceiling,
            dragon,
            scale,
            min_y,
            height,
            logical,
            ambient,
            limit,
            provider,
            skybox,
            cardinal,
            infiniburn,
            timelines,
            clock,
        ) = match kind {
            LockedDimension::Overworld => (
                false,
                true,
                false,
                false,
                1.0,
                -64,
                384,
                384,
                0.0,
                0,
                SpawnLightLevel::UniformInclusive {
                    minimum: 0,
                    maximum: 7,
                },
                Some(Skybox::Overworld),
                CardinalLight::Default,
                "#minecraft:infiniburn_overworld",
                vec![
                    "minecraft:villager_schedule",
                    "minecraft:day",
                    "minecraft:moon",
                    "minecraft:early_game",
                ],
                Some("minecraft:overworld"),
            ),
            LockedDimension::OverworldCaves => (
                false,
                true,
                true,
                false,
                1.0,
                -64,
                384,
                384,
                0.0,
                0,
                SpawnLightLevel::UniformInclusive {
                    minimum: 0,
                    maximum: 7,
                },
                Some(Skybox::Overworld),
                CardinalLight::Default,
                "#minecraft:infiniburn_overworld",
                vec![
                    "minecraft:villager_schedule",
                    "minecraft:day",
                    "minecraft:moon",
                    "minecraft:early_game",
                ],
                Some("minecraft:overworld"),
            ),
            LockedDimension::TheEnd => (
                true,
                true,
                false,
                true,
                1.0,
                0,
                256,
                256,
                0.25,
                0,
                SpawnLightLevel::Constant(15),
                Some(Skybox::End),
                CardinalLight::Default,
                "#minecraft:infiniburn_end",
                vec!["minecraft:villager_schedule"],
                Some("minecraft:the_end"),
            ),
            LockedDimension::TheNether => (
                true,
                false,
                true,
                false,
                8.0,
                0,
                256,
                128,
                0.1,
                15,
                SpawnLightLevel::Constant(7),
                None,
                CardinalLight::Nether,
                "#minecraft:infiniburn_nether",
                vec!["minecraft:villager_schedule"],
                None,
            ),
        };
        Self::validate(Self {
            has_fixed_time: fixed,
            has_skylight: skylight,
            has_ceiling: ceiling,
            has_ender_dragon_fight: dragon,
            coordinate_scale: scale,
            min_y,
            height,
            logical_height: logical,
            infiniburn: infiniburn.to_owned(),
            ambient_light: ambient,
            monster_spawn_block_light_limit: limit,
            monster_spawn_light_level: provider,
            skybox,
            cardinal_light: cardinal,
            attributes: locked_dimension_attributes(kind),
            timelines: timelines.into_iter().map(str::to_owned).collect(),
            default_clock: clock.map(str::to_owned),
        })
        .expect("locked dimension records are valid")
    }

    pub fn max_y(&self) -> i32 {
        self.min_y + self.height as i32 - 1
    }

    pub fn is_inside_build_height(&self, y: i32) -> bool {
        (self.min_y..=self.max_y()).contains(&y)
    }

    pub fn section_index(&self, y: i32) -> Option<u32> {
        self.is_inside_build_height(y)
            .then(|| y.div_euclid(16) - self.min_y.div_euclid(16))
            .map(|index| index as u32)
    }

    pub fn section_count(&self) -> u32 {
        self.height / 16
    }

    pub fn can_have_weather(&self, dimension_key: &str) -> bool {
        self.has_skylight && !self.has_ceiling && dimension_key != "minecraft:the_end"
    }

    pub fn is_bright_outside(&self, sky_darken: i32) -> bool {
        !self.has_fixed_time && sky_darken < 4
    }

    pub fn is_dark_outside(&self, sky_darken: i32) -> bool {
        !self.has_fixed_time && sky_darken >= 4
    }

    pub fn brightness_ramp(&self, raw_light: u8) -> f32 {
        let raw = f32::from(raw_light.min(15)) / 15.0;
        let vanilla = 1.0 / (4.0 - 3.0 * raw);
        vanilla + self.ambient_light * (1.0 - vanilla)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockedDimension {
    Overworld,
    OverworldCaves,
    TheEnd,
    TheNether,
}

pub fn teleportation_scale(source: &DimensionType, destination: &DimensionType) -> f64 {
    source.coordinate_scale / destination.coordinate_scale
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub fn scale_command_position(
    position: Position,
    source: &DimensionType,
    destination: &DimensionType,
) -> Position {
    let scale = teleportation_scale(source, destination);
    Position {
        x: position.x * scale,
        y: position.y,
        z: position.z * scale,
    }
}

pub fn dimension_storage_folder(dimension_key: &str) -> String {
    match dimension_key {
        "minecraft:overworld" => String::new(),
        "minecraft:the_nether" => "DIM-1".to_owned(),
        "minecraft:the_end" => "DIM1".to_owned(),
        key => {
            let (namespace, path) = key.split_once(':').unwrap_or(("minecraft", key));
            format!("dimensions/{namespace}/{path}")
        }
    }
}
