//! Typed environment attributes and deterministic layer resolution.

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SleepRule {
    Always,
    WhenDark,
    Never,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpawnRule {
    Always,
    Never,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BedRule {
    pub can_sleep: SleepRule,
    pub can_set_spawn: SpawnRule,
    pub explodes: bool,
    pub error_message: Option<String>,
}

impl BedRule {
    pub fn overworld() -> Self {
        Self {
            can_sleep: SleepRule::WhenDark,
            can_set_spawn: SpawnRule::Always,
            explodes: false,
            error_message: Some("block.minecraft.bed.no_sleep".to_owned()),
        }
    }

    pub fn exploding() -> Self {
        Self {
            can_sleep: SleepRule::Never,
            can_set_spawn: SpawnRule::Never,
            explodes: true,
            error_message: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Music {
    pub sound: String,
    pub minimum_delay: u32,
    pub maximum_delay: u32,
    pub replace_current: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundMusic {
    pub default: Option<Music>,
    pub creative: Option<Music>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MoodSound {
    pub sound: String,
    pub tick_delay: u32,
    pub block_search_extent: u32,
    pub offset: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AttributeValue {
    Float(f32),
    Integer(i32),
    Boolean(bool),
    Color(i32),
    Identifier(String),
    BedRule(BedRule),
    BackgroundMusic(BackgroundMusic),
    AmbientSounds(Option<MoodSound>),
    IdentifierList(Vec<String>),
}

impl AttributeValue {
    fn same_type(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Modifier {
    Override,
    Add,
    Multiply,
    Maximum,
    Minimum,
    Or,
    And,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttributeEntry {
    pub value: AttributeValue,
    pub modifier: Modifier,
}

impl AttributeEntry {
    pub fn override_with(value: AttributeValue) -> Self {
        Self {
            value,
            modifier: Modifier::Override,
        }
    }

    pub fn apply(&self, preceding: &AttributeValue) -> Option<AttributeValue> {
        if !self.value.same_type(preceding) {
            return None;
        }
        match (&self.value, preceding, self.modifier) {
            (value, _, Modifier::Override) => Some(value.clone()),
            (AttributeValue::Float(value), AttributeValue::Float(base), Modifier::Add) => {
                Some(AttributeValue::Float(base + value))
            }
            (AttributeValue::Float(value), AttributeValue::Float(base), Modifier::Multiply) => {
                Some(AttributeValue::Float(base * value))
            }
            (AttributeValue::Float(value), AttributeValue::Float(base), Modifier::Maximum) => {
                Some(AttributeValue::Float(base.max(*value)))
            }
            (AttributeValue::Float(value), AttributeValue::Float(base), Modifier::Minimum) => {
                Some(AttributeValue::Float(base.min(*value)))
            }
            (AttributeValue::Integer(value), AttributeValue::Integer(base), Modifier::Add) => {
                Some(AttributeValue::Integer(base.saturating_add(*value)))
            }
            (AttributeValue::Integer(value), AttributeValue::Integer(base), Modifier::Multiply) => {
                Some(AttributeValue::Integer(base.saturating_mul(*value)))
            }
            (AttributeValue::Integer(value), AttributeValue::Integer(base), Modifier::Maximum) => {
                Some(AttributeValue::Integer((*base).max(*value)))
            }
            (AttributeValue::Integer(value), AttributeValue::Integer(base), Modifier::Minimum) => {
                Some(AttributeValue::Integer((*base).min(*value)))
            }
            (AttributeValue::Color(value), AttributeValue::Color(base), Modifier::Multiply) => {
                Some(AttributeValue::Color(multiply_color(*base, *value)))
            }
            (AttributeValue::Boolean(value), AttributeValue::Boolean(base), Modifier::Or) => {
                Some(AttributeValue::Boolean(*base || *value))
            }
            (AttributeValue::Boolean(value), AttributeValue::Boolean(base), Modifier::And) => {
                Some(AttributeValue::Boolean(*base && *value))
            }
            _ => None,
        }
    }
}

pub type AttributeMap = BTreeMap<String, AttributeEntry>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sanitizer {
    None,
    FloatRange { minimum: f32, maximum: f32 },
}

impl Sanitizer {
    pub fn sanitize(self, value: AttributeValue) -> AttributeValue {
        match (self, value) {
            (Self::FloatRange { minimum, maximum }, AttributeValue::Float(value)) => {
                AttributeValue::Float(value.clamp(minimum, maximum))
            }
            (_, value) => value,
        }
    }

    pub fn accepts(self, value: &AttributeValue) -> bool {
        match (self, value) {
            (Self::FloatRange { minimum, maximum }, AttributeValue::Float(value)) => {
                value.is_finite() && (minimum..=maximum).contains(value)
            }
            _ => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttributeDeclaration {
    pub id: String,
    pub default: AttributeValue,
    pub positional: bool,
    pub syncable: bool,
    pub spatially_interpolated: bool,
    pub sanitizer: Sanitizer,
}

impl AttributeDeclaration {
    pub fn sanitize(&self, value: AttributeValue) -> AttributeValue {
        self.sanitizer.sanitize(value)
    }

    pub fn interpolate(
        &self,
        start: &AttributeValue,
        end: &AttributeValue,
        amount: f32,
    ) -> Option<AttributeValue> {
        if !start.same_type(end) {
            return None;
        }
        let amount = amount.clamp(0.0, 1.0);
        let value = match (start, end) {
            (AttributeValue::Float(start), AttributeValue::Float(end)) => {
                AttributeValue::Float(start + (end - start) * amount)
            }
            (AttributeValue::Integer(start), AttributeValue::Integer(end)) => {
                AttributeValue::Integer(
                    (*start as f32 + (*end - *start) as f32 * amount).round() as i32
                )
            }
            (AttributeValue::Color(start), AttributeValue::Color(end)) => {
                AttributeValue::Color(lerp_color(*start, *end, amount))
            }
            _ if amount < 0.5 => start.clone(),
            _ => end.clone(),
        };
        Some(self.sanitize(value))
    }
}

fn declaration(
    id: &str,
    default: AttributeValue,
    positional: bool,
    syncable: bool,
    spatially_interpolated: bool,
    sanitizer: Sanitizer,
) -> AttributeDeclaration {
    AttributeDeclaration {
        id: format!("minecraft:{id}"),
        default,
        positional,
        syncable,
        spatially_interpolated,
        sanitizer,
    }
}

fn float(value: f32) -> AttributeValue {
    AttributeValue::Float(value)
}

fn color(value: i32) -> AttributeValue {
    AttributeValue::Color(value)
}

fn rgb(value: u32) -> AttributeValue {
    AttributeValue::Color((0xff00_0000 | value) as i32)
}

/// Returns all 48 declarations in the locked registry order.
pub fn locked_declarations() -> Vec<AttributeDeclaration> {
    let nonnegative = Sanitizer::FloatRange {
        minimum: 0.0,
        maximum: f32::MAX,
    };
    let unit = Sanitizer::FloatRange {
        minimum: 0.0,
        maximum: 1.0,
    };
    let visual = [
        ("visual/fog_color", color(0), Sanitizer::None, true),
        ("visual/fog_start_distance", float(0.0), nonnegative, true),
        ("visual/fog_end_distance", float(1024.0), nonnegative, true),
        (
            "visual/sky_fog_end_distance",
            float(512.0),
            nonnegative,
            true,
        ),
        (
            "visual/cloud_fog_end_distance",
            float(2048.0),
            nonnegative,
            true,
        ),
        (
            "visual/water_fog_color",
            color(-16_448_205),
            Sanitizer::None,
            true,
        ),
        (
            "visual/water_fog_start_distance",
            float(-8.0),
            Sanitizer::None,
            true,
        ),
        (
            "visual/water_fog_end_distance",
            float(96.0),
            nonnegative,
            true,
        ),
        ("visual/sky_color", color(0), Sanitizer::None, true),
        (
            "visual/sunrise_sunset_color",
            color(0),
            Sanitizer::None,
            true,
        ),
        ("visual/cloud_color", color(0), Sanitizer::None, true),
        ("visual/cloud_height", float(192.33), Sanitizer::None, true),
        ("visual/sun_angle", float(0.0), Sanitizer::None, true),
        ("visual/moon_angle", float(0.0), Sanitizer::None, true),
        ("visual/star_angle", float(0.0), Sanitizer::None, true),
        (
            "visual/moon_phase",
            AttributeValue::Identifier("full_moon".to_owned()),
            Sanitizer::None,
            false,
        ),
        ("visual/star_brightness", float(0.0), unit, true),
        (
            "visual/block_light_tint",
            color(-10_100),
            Sanitizer::None,
            true,
        ),
        ("visual/sky_light_color", color(-1), Sanitizer::None, true),
        ("visual/sky_light_factor", float(1.0), unit, true),
        (
            "visual/night_vision_color",
            color(-6_710_887),
            Sanitizer::None,
            true,
        ),
        (
            "visual/ambient_light_color",
            color(-16_777_216),
            Sanitizer::None,
            true,
        ),
        (
            "visual/default_dripstone_particle",
            AttributeValue::Identifier("minecraft:dripping_dripstone_water".to_owned()),
            Sanitizer::None,
            false,
        ),
        (
            "visual/ambient_particles",
            AttributeValue::IdentifierList(Vec::new()),
            Sanitizer::None,
            false,
        ),
    ];
    let mut declarations = visual
        .into_iter()
        .map(|(id, default, sanitizer, interpolated)| {
            declaration(id, default, true, true, interpolated, sanitizer)
        })
        .collect::<Vec<_>>();

    declarations.extend([
        declaration(
            "audio/background_music",
            AttributeValue::BackgroundMusic(BackgroundMusic {
                default: None,
                creative: None,
            }),
            true,
            true,
            false,
            Sanitizer::None,
        ),
        declaration("audio/music_volume", float(1.0), true, true, false, unit),
        declaration(
            "audio/ambient_sounds",
            AttributeValue::AmbientSounds(None),
            true,
            true,
            false,
            Sanitizer::None,
        ),
        declaration(
            "audio/firefly_bush_sounds",
            AttributeValue::Boolean(false),
            true,
            true,
            false,
            Sanitizer::None,
        ),
    ]);

    let sky_light_range = Sanitizer::FloatRange {
        minimum: 0.0,
        maximum: 15.0,
    };
    let gameplay = [
        (
            "gameplay/sky_light_level",
            float(15.0),
            false,
            true,
            sky_light_range,
        ),
        (
            "gameplay/can_start_raid",
            AttributeValue::Boolean(true),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/water_evaporates",
            AttributeValue::Boolean(false),
            true,
            true,
            Sanitizer::None,
        ),
        (
            "gameplay/bed_rule",
            AttributeValue::BedRule(BedRule::overworld()),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/respawn_anchor_works",
            AttributeValue::Boolean(false),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/nether_portal_spawns_piglin",
            AttributeValue::Boolean(false),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/fast_lava",
            AttributeValue::Boolean(false),
            false,
            true,
            Sanitizer::None,
        ),
        (
            "gameplay/increased_fire_burnout",
            AttributeValue::Boolean(false),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/eyeblossom_open",
            AttributeValue::Identifier("default".to_owned()),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/turtle_egg_hatch_chance",
            float(0.002),
            true,
            false,
            unit,
        ),
        (
            "gameplay/piglins_zombify",
            AttributeValue::Boolean(true),
            true,
            true,
            Sanitizer::None,
        ),
        (
            "gameplay/snow_golem_melts",
            AttributeValue::Boolean(false),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/creaking_active",
            AttributeValue::Boolean(false),
            true,
            true,
            Sanitizer::None,
        ),
        (
            "gameplay/surface_slime_spawn_chance",
            float(0.0),
            true,
            false,
            unit,
        ),
        (
            "gameplay/cat_waking_up_gift_chance",
            float(0.0),
            true,
            false,
            unit,
        ),
        (
            "gameplay/bees_stay_in_hive",
            AttributeValue::Boolean(false),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/monsters_burn",
            AttributeValue::Boolean(false),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/can_pillager_patrol_spawn",
            AttributeValue::Boolean(true),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/villager_activity",
            AttributeValue::Identifier("minecraft:idle".to_owned()),
            true,
            false,
            Sanitizer::None,
        ),
        (
            "gameplay/baby_villager_activity",
            AttributeValue::Identifier("minecraft:idle".to_owned()),
            true,
            false,
            Sanitizer::None,
        ),
    ];
    declarations.extend(gameplay.into_iter().map(
        |(id, default, positional, syncable, sanitizer)| {
            declaration(id, default, positional, syncable, false, sanitizer)
        },
    ));
    declarations
}

pub fn declaration_by_id(id: &str) -> Option<AttributeDeclaration> {
    locked_declarations()
        .into_iter()
        .find(|entry| entry.id == id)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayerKind {
    Dimension,
    Biome,
    Timeline,
    Weather,
}

impl LayerKind {
    pub const fn positional(self) -> bool {
        matches!(self, Self::Biome | Self::Weather)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampledLayer {
    pub kind: LayerKind,
    pub values: AttributeMap,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnvironmentLayers {
    layers: Vec<SampledLayer>,
}

impl EnvironmentLayers {
    pub fn construct(
        dimension: AttributeMap,
        biome: Option<AttributeMap>,
        timelines: impl IntoIterator<Item = AttributeMap>,
        weather: Option<AttributeMap>,
        can_have_weather: bool,
    ) -> Self {
        let mut layers = vec![SampledLayer {
            kind: LayerKind::Dimension,
            values: dimension,
        }];
        if let Some(values) = biome {
            layers.push(SampledLayer {
                kind: LayerKind::Biome,
                values,
            });
        }
        layers.extend(timelines.into_iter().map(|values| SampledLayer {
            kind: LayerKind::Timeline,
            values,
        }));
        if can_have_weather && let Some(values) = weather {
            layers.push(SampledLayer {
                kind: LayerKind::Weather,
                values,
            });
        }
        Self { layers }
    }

    pub fn layers(&self) -> &[SampledLayer] {
        &self.layers
    }

    pub fn resolve(&self, declaration: &AttributeDeclaration, at_position: bool) -> AttributeValue {
        let mut value = declaration.default.clone();
        for layer in &self.layers {
            if layer.kind.positional() && !at_position {
                continue;
            }
            if let Some(entry) = layer.values.get(&declaration.id)
                && let Some(updated) = entry.apply(&value)
            {
                value = updated;
            }
        }
        declaration.sanitize(value)
    }

    pub fn dimension_value(&self, declaration: &AttributeDeclaration) -> AttributeValue {
        self.resolve(declaration, false)
    }

    pub fn network_map(&self) -> AttributeMap {
        locked_declarations()
            .into_iter()
            .filter(|declaration| declaration.syncable)
            .map(|declaration| {
                let value = self.resolve(&declaration, declaration.positional);
                (declaration.id, AttributeEntry::override_with(value))
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnvironmentCache {
    generation: u64,
}

impl EnvironmentCache {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Must be called at the start of every level tick, including frozen ticks.
    pub fn invalidate_for_level_tick(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Called for all levels after a direct clock mutation is broadcast.
    pub fn invalidate_for_clock_change(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaussianCell {
    pub quart: [i32; 3],
    pub weight: f64,
}

/// Produces the client camera probe's 6³ quart-biome samples.
pub fn gaussian_camera_cells(position: [f64; 3]) -> Vec<GaussianCell> {
    let scaled = position.map(|axis| axis * 0.25 - 0.5);
    let base = scaled.map(|axis| axis.floor() as i32);
    let fraction = [
        scaled[0] - f64::from(base[0]),
        scaled[1] - f64::from(base[1]),
        scaled[2] - f64::from(base[2]),
    ];
    let mut cells = Vec::with_capacity(216);
    for dx in -2..=3 {
        let wx = kernel_weight(dx, fraction[0]);
        for dy in -2..=3 {
            let wy = kernel_weight(dy, fraction[1]);
            for dz in -2..=3 {
                let wz = kernel_weight(dz, fraction[2]);
                cells.push(GaussianCell {
                    quart: [base[0] + dx, base[1] + dy, base[2] + dz],
                    weight: wx * wy * wz,
                });
            }
        }
    }
    cells
}

pub fn block_center(position: [i32; 3]) -> [f64; 3] {
    position.map(|axis| f64::from(axis) + 0.5)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttributeProbe {
    previous: Option<AttributeValue>,
    current: Option<AttributeValue>,
}

impl AttributeProbe {
    pub fn update(&mut self, value: AttributeValue) {
        self.previous = self.current.replace(value);
    }

    pub fn render(
        &self,
        declaration: &AttributeDeclaration,
        partial_tick: f32,
    ) -> Option<AttributeValue> {
        let current = self.current.as_ref()?;
        let previous = self.previous.as_ref().unwrap_or(current);
        declaration.interpolate(previous, current, partial_tick)
    }
}

fn kernel_weight(offset: i32, fraction: f64) -> f64 {
    const KERNEL: [f64; 7] = [0.0, 1.0, 4.0, 6.0, 4.0, 1.0, 0.0];
    let index = (offset + 2) as usize;
    KERNEL[index] + (KERNEL[index + 1] - KERNEL[index]) * fraction
}

pub fn weighted_spatial_sample(
    declaration: &AttributeDeclaration,
    samples: impl IntoIterator<Item = (AttributeValue, f64)>,
) -> Option<AttributeValue> {
    let samples = samples
        .into_iter()
        .filter(|(_, weight)| *weight > 0.0)
        .collect::<Vec<_>>();
    let total = samples.iter().map(|(_, weight)| weight).sum::<f64>();
    if samples.is_empty() || total == 0.0 {
        return None;
    }
    let mut accumulated = samples[0].0.clone();
    let mut accumulated_weight = samples[0].1;
    for (value, weight) in samples.into_iter().skip(1) {
        let amount = (weight / (accumulated_weight + weight)) as f32;
        accumulated = declaration.interpolate(&accumulated, &value, amount)?;
        accumulated_weight += weight;
    }
    Some(declaration.sanitize(accumulated))
}

fn put(map: &mut AttributeMap, id: &str, value: AttributeValue) {
    map.insert(
        format!("minecraft:{id}"),
        AttributeEntry::override_with(value),
    );
}

pub fn locked_dimension_attributes(dimension: super::LockedDimension) -> AttributeMap {
    use super::LockedDimension;

    let mut map = AttributeMap::new();
    match dimension {
        LockedDimension::Overworld | LockedDimension::OverworldCaves => {
            put(&mut map, "audio/ambient_sounds", cave_mood());
            put(&mut map, "audio/background_music", overworld_music());
            put(
                &mut map,
                "gameplay/bed_rule",
                AttributeValue::BedRule(BedRule::overworld()),
            );
            put(
                &mut map,
                "gameplay/nether_portal_spawns_piglin",
                AttributeValue::Boolean(true),
            );
            put(
                &mut map,
                "gameplay/respawn_anchor_works",
                AttributeValue::Boolean(false),
            );
            put(&mut map, "visual/ambient_light_color", rgb(0x000a0a));
            put(
                &mut map,
                "visual/cloud_color",
                color(0xccff_ffff_u32 as i32),
            );
            put(&mut map, "visual/cloud_height", float(192.33));
            put(&mut map, "visual/fog_color", rgb(0xc0_d8ff));
            put(&mut map, "visual/sky_color", rgb(0x78_a7ff));
        }
        LockedDimension::TheEnd => {
            put(&mut map, "audio/ambient_sounds", cave_mood());
            put(&mut map, "audio/background_music", end_music());
            put(
                &mut map,
                "gameplay/bed_rule",
                AttributeValue::BedRule(BedRule::exploding()),
            );
            put(
                &mut map,
                "gameplay/respawn_anchor_works",
                AttributeValue::Boolean(false),
            );
            put(&mut map, "visual/ambient_light_color", rgb(0x3f_473f));
            put(&mut map, "visual/fog_color", rgb(0x18_1318));
            put(&mut map, "visual/sky_color", rgb(0));
            put(&mut map, "visual/sky_light_color", rgb(0xac_60cd));
            put(&mut map, "visual/sky_light_factor", float(0.0));
        }
        LockedDimension::TheNether => {
            put(
                &mut map,
                "gameplay/bed_rule",
                AttributeValue::BedRule(BedRule::exploding()),
            );
            put(
                &mut map,
                "gameplay/can_start_raid",
                AttributeValue::Boolean(false),
            );
            put(
                &mut map,
                "gameplay/fast_lava",
                AttributeValue::Boolean(true),
            );
            put(
                &mut map,
                "gameplay/piglins_zombify",
                AttributeValue::Boolean(false),
            );
            put(
                &mut map,
                "gameplay/respawn_anchor_works",
                AttributeValue::Boolean(true),
            );
            put(&mut map, "gameplay/sky_light_level", float(4.0));
            put(
                &mut map,
                "gameplay/snow_golem_melts",
                AttributeValue::Boolean(true),
            );
            put(
                &mut map,
                "gameplay/water_evaporates",
                AttributeValue::Boolean(true),
            );
            put(&mut map, "visual/ambient_light_color", rgb(0x30_2821));
            put(
                &mut map,
                "visual/default_dripstone_particle",
                AttributeValue::Identifier("minecraft:dripping_dripstone_lava".to_owned()),
            );
            put(&mut map, "visual/fog_start_distance", float(10.0));
            put(&mut map, "visual/fog_end_distance", float(96.0));
            put(&mut map, "visual/sky_light_color", rgb(0x7a_7aff));
            put(&mut map, "visual/sky_light_factor", float(0.0));
        }
    }
    map
}

fn cave_mood() -> AttributeValue {
    AttributeValue::AmbientSounds(Some(MoodSound {
        sound: "minecraft:ambient.cave".to_owned(),
        tick_delay: 6_000,
        block_search_extent: 8,
        offset: 2.0,
    }))
}

fn music(sound: &str, minimum_delay: u32, maximum_delay: u32, replace_current: bool) -> Music {
    Music {
        sound: sound.to_owned(),
        minimum_delay,
        maximum_delay,
        replace_current,
    }
}

fn overworld_music() -> AttributeValue {
    AttributeValue::BackgroundMusic(BackgroundMusic {
        default: Some(music("minecraft:music.game", 12_000, 24_000, false)),
        creative: Some(music("minecraft:music.creative", 12_000, 24_000, false)),
    })
}

fn end_music() -> AttributeValue {
    AttributeValue::BackgroundMusic(BackgroundMusic {
        default: Some(music("minecraft:music.end", 6_000, 24_000, true)),
        creative: None,
    })
}

fn color_channels(value: i32) -> [u8; 4] {
    value.to_be_bytes()
}

fn channels_color(channels: [u8; 4]) -> i32 {
    i32::from_be_bytes(channels)
}

fn lerp_color(start: i32, end: i32, amount: f32) -> i32 {
    let start = color_channels(start);
    let end = color_channels(end);
    channels_color(std::array::from_fn(|index| {
        (f32::from(start[index]) + (f32::from(end[index]) - f32::from(start[index])) * amount)
            .round() as u8
    }))
}

fn multiply_color(base: i32, modifier: i32) -> i32 {
    let base = color_channels(base);
    let modifier = color_channels(modifier);
    channels_color(std::array::from_fn(|index| {
        ((u16::from(base[index]) * u16::from(modifier[index])) / 255) as u8
    }))
}
