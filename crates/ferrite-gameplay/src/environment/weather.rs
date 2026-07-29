//! World-owned weather transitions and Region-local precipitation/lightning decisions.

use ferrite_foundation::coordinate::BlockPos;

pub const WEATHER_STRENGTH_STEP: f32 = 0.01;
pub const RAINING_THRESHOLD: f32 = 0.2;
pub const THUNDERING_THRESHOLD: f32 = 0.9;
pub const RAIN_DELAY_MIN: u32 = 12_000;
pub const RAIN_DELAY_MAX: u32 = 180_000;
pub const RAIN_DURATION_MIN: u32 = 12_000;
pub const RAIN_DURATION_MAX: u32 = 24_000;
pub const THUNDER_DELAY_MIN: u32 = 12_000;
pub const THUNDER_DELAY_MAX: u32 = 180_000;
pub const THUNDER_DURATION_MIN: u32 = 3_600;
pub const THUNDER_DURATION_MAX: u32 = 15_600;
pub const PRECIPITATION_CHANCE_BOUND: u32 = 48;
pub const LIGHTNING_CHANCE_BOUND: u32 = 100_000;
pub const LIGHTNING_ROD_RADIUS: u16 = 128;
pub const LIGHTNING_ENTITY_INFLATION: i32 = 3;
pub const RAIN_CAULDRON_CHANCE: f32 = 0.05;
pub const SNOW_CAULDRON_CHANCE: f32 = 0.1;
pub const FREEZE_LIGHT_LIMIT: u8 = 10;
pub const MAX_SNOW_LAYERS: u8 = 8;
pub const DEFAULT_RANDOM_TICK_SPEED: u32 = 3;
pub const DEFAULT_MAX_SNOW_ACCUMULATION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelWeatherStage {
    WorldBorder,
    Weather,
    Sleep,
    Clock,
    ScheduledBlockTicks,
    ScheduledFluidTicks,
    ChunkWork,
}

pub const LEVEL_WEATHER_ORDER: [LevelWeatherStage; 7] = [
    LevelWeatherStage::WorldBorder,
    LevelWeatherStage::Weather,
    LevelWeatherStage::Sleep,
    LevelWeatherStage::Clock,
    LevelWeatherStage::ScheduledBlockTicks,
    LevelWeatherStage::ScheduledFluidTicks,
    LevelWeatherStage::ChunkWork,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkWeatherStage {
    ShuffleSpawningChunks,
    Thunder,
    NaturalSpawning,
    Precipitation,
    RandomBlockAndFluidTicks,
    CustomSpawners,
}

pub const CHUNK_WEATHER_ORDER: [ChunkWeatherStage; 6] = [
    ChunkWeatherStage::ShuffleSpawningChunks,
    ChunkWeatherStage::Thunder,
    ChunkWeatherStage::NaturalSpawning,
    ChunkWeatherStage::Precipitation,
    ChunkWeatherStage::RandomBlockAndFluidTicks,
    ChunkWeatherStage::CustomSpawners,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherDimension {
    pub has_sky_light: bool,
    pub has_ceiling: bool,
    pub is_end: bool,
}

impl WeatherDimension {
    pub const fn can_have_weather(self) -> bool {
        self.has_sky_light && !self.has_ceiling && !self.is_end
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WeatherData {
    pub clear_weather_time: i32,
    pub rain_time: i32,
    pub thunder_time: i32,
    pub raining: bool,
    pub thundering: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherField {
    ClearTime,
    RainTime,
    ThunderTime,
    Raining,
    Thundering,
}

pub const PERSISTED_WEATHER_ORDER: [WeatherField; 5] = [
    WeatherField::ClearTime,
    WeatherField::RainTime,
    WeatherField::ThunderTime,
    WeatherField::Raining,
    WeatherField::Thundering,
];

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct WeatherStrengths {
    pub previous_rain: f32,
    pub rain: f32,
    pub previous_thunder: f32,
    pub thunder: f32,
}

impl WeatherStrengths {
    pub const fn from_saved(data: WeatherData, capable: bool) -> Self {
        let rain = if capable && data.raining { 1.0 } else { 0.0 };
        let thunder = if capable && data.raining && data.thundering {
            1.0
        } else {
            0.0
        };
        Self {
            previous_rain: rain,
            rain,
            previous_thunder: thunder,
            thunder,
        }
    }

    pub fn is_raining(self, capable: bool) -> bool {
        capable && self.rain > RAINING_THRESHOLD
    }

    pub fn is_thundering(self, capable: bool) -> bool {
        capable && self.thunder * self.rain > THUNDERING_THRESHOLD
    }
}

pub trait WeatherRandom {
    fn next_int(&mut self, bound: u32) -> u32;
}

fn inclusive_duration(random: &mut impl WeatherRandom, min: u32, max: u32) -> i32 {
    (min + random.next_int(max - min + 1)) as i32
}

fn advance_timer(
    timer: &mut i32,
    target: &mut bool,
    true_range: (u32, u32),
    false_range: (u32, u32),
    random: &mut impl WeatherRandom,
) {
    if *timer > 0 {
        *timer -= 1;
        if *timer == 0 {
            *target = !*target;
        }
    } else {
        let (min, max) = if *target { true_range } else { false_range };
        *timer = inclusive_duration(random, min, max);
    }
}

pub fn advance_weather_targets(
    data: &mut WeatherData,
    dimension: WeatherDimension,
    normal_tick: bool,
    advance_weather: bool,
    random: &mut impl WeatherRandom,
) {
    if !normal_tick || !dimension.can_have_weather() || !advance_weather {
        return;
    }
    if data.clear_weather_time > 0 {
        data.clear_weather_time -= 1;
        data.rain_time = i32::from(!data.raining);
        data.thunder_time = i32::from(!data.thundering);
        data.raining = false;
        data.thundering = false;
        return;
    }
    advance_timer(
        &mut data.thunder_time,
        &mut data.thundering,
        (THUNDER_DURATION_MIN, THUNDER_DURATION_MAX),
        (THUNDER_DELAY_MIN, THUNDER_DELAY_MAX),
        random,
    );
    advance_timer(
        &mut data.rain_time,
        &mut data.raining,
        (RAIN_DURATION_MIN, RAIN_DURATION_MAX),
        (RAIN_DELAY_MIN, RAIN_DELAY_MAX),
        random,
    );
}

fn approach(value: f32, target: bool) -> f32 {
    (value
        + if target {
            WEATHER_STRENGTH_STEP
        } else {
            -WEATHER_STRENGTH_STEP
        })
    .clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherPacketScope {
    Dimension,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatherPacketKind {
    StartRaining,
    StopRaining,
    RainStrength(f32),
    ThunderStrength(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherPacket {
    pub scope: WeatherPacketScope,
    pub kind: WeatherPacketKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherPhase {
    pub packets: Vec<WeatherPacket>,
    pub ran: bool,
}

pub fn run_weather_phase(
    data: &mut WeatherData,
    strengths: &mut WeatherStrengths,
    dimension: WeatherDimension,
    normal_tick: bool,
    advance_weather: bool,
    random: &mut impl WeatherRandom,
) -> WeatherPhase {
    let capable = dimension.can_have_weather();
    if !normal_tick || !capable {
        return WeatherPhase {
            packets: Vec::new(),
            ran: false,
        };
    }
    let was_raining = strengths.is_raining(capable);
    advance_weather_targets(data, dimension, normal_tick, advance_weather, random);

    strengths.previous_thunder = strengths.thunder;
    strengths.thunder = approach(strengths.thunder, data.thundering);
    strengths.previous_rain = strengths.rain;
    strengths.rain = approach(strengths.rain, data.raining);

    let mut packets = Vec::with_capacity(5);
    if strengths.thunder != strengths.previous_thunder {
        packets.push(WeatherPacket {
            scope: WeatherPacketScope::Dimension,
            kind: WeatherPacketKind::ThunderStrength(strengths.thunder),
        });
    }
    if strengths.rain != strengths.previous_rain {
        packets.push(WeatherPacket {
            scope: WeatherPacketScope::Dimension,
            kind: WeatherPacketKind::RainStrength(strengths.rain),
        });
    }
    let is_raining = strengths.is_raining(capable);
    if is_raining != was_raining {
        packets.push(WeatherPacket {
            scope: WeatherPacketScope::Global,
            kind: if is_raining {
                WeatherPacketKind::StartRaining
            } else {
                WeatherPacketKind::StopRaining
            },
        });
        packets.push(WeatherPacket {
            scope: WeatherPacketScope::Global,
            kind: WeatherPacketKind::RainStrength(strengths.rain),
        });
        packets.push(WeatherPacket {
            scope: WeatherPacketScope::Global,
            kind: WeatherPacketKind::ThunderStrength(strengths.thunder),
        });
    }
    WeatherPhase { packets, ran: true }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherCommand {
    Clear,
    Rain,
    Thunder,
}

pub fn command_weather(
    command: WeatherCommand,
    duration: Option<u32>,
    random: &mut impl WeatherRandom,
) -> WeatherData {
    let duration = duration.unwrap_or_else(|| match command {
        WeatherCommand::Clear => inclusive_duration(random, RAIN_DELAY_MIN, RAIN_DELAY_MAX) as u32,
        WeatherCommand::Rain => {
            inclusive_duration(random, RAIN_DURATION_MIN, RAIN_DURATION_MAX) as u32
        }
        WeatherCommand::Thunder => {
            inclusive_duration(random, THUNDER_DURATION_MIN, THUNDER_DURATION_MAX) as u32
        }
    }) as i32;
    match command {
        WeatherCommand::Clear => WeatherData {
            clear_weather_time: duration,
            ..WeatherData::default()
        },
        WeatherCommand::Rain => WeatherData {
            rain_time: duration,
            thunder_time: duration,
            raining: true,
            thundering: false,
            ..WeatherData::default()
        },
        WeatherCommand::Thunder => WeatherData {
            rain_time: duration,
            thunder_time: duration,
            raining: true,
            thundering: true,
            ..WeatherData::default()
        },
    }
}

pub fn clear_weather_after_sleep(
    data: &mut WeatherData,
    strengths: WeatherStrengths,
    capable: bool,
    advance_weather: bool,
) -> bool {
    if !advance_weather || !strengths.is_raining(capable) {
        return false;
    }
    *data = WeatherData::default();
    true
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ClientWeather {
    pub previous_rain: f32,
    pub rain: f32,
    pub previous_thunder: f32,
    pub thunder: f32,
}

impl ClientWeather {
    pub fn apply(&mut self, packet: WeatherPacketKind) {
        match packet {
            WeatherPacketKind::StartRaining => {
                self.previous_rain = 0.0;
                self.rain = 0.0;
            }
            WeatherPacketKind::StopRaining => {
                self.previous_rain = 1.0;
                self.rain = 1.0;
            }
            WeatherPacketKind::RainStrength(value) => {
                let value = value.clamp(0.0, 1.0);
                self.previous_rain = value;
                self.rain = value;
            }
            WeatherPacketKind::ThunderStrength(value) => {
                let value = value.clamp(0.0, 1.0);
                self.previous_thunder = value;
                self.thunder = value;
            }
        }
    }
}

pub fn join_weather_packets(strengths: WeatherStrengths, capable: bool) -> Vec<WeatherPacketKind> {
    if !strengths.is_raining(capable) {
        return Vec::new();
    }
    vec![
        WeatherPacketKind::StartRaining,
        WeatherPacketKind::RainStrength(strengths.rain),
        WeatherPacketKind::ThunderStrength(strengths.thunder),
    ]
}

pub fn advance_random_position(
    rand_value: &mut i32,
    chunk_min_x: i32,
    base_y: i32,
    chunk_min_z: i32,
    mask: u32,
) -> BlockPos {
    *rand_value = rand_value.wrapping_mul(3).wrapping_add(1_013_904_223);
    let bits = (*rand_value as u32) >> 2;
    BlockPos::new(
        chunk_min_x + ((bits & mask) as i32),
        base_y + (((bits >> 16) & mask) as i32),
        chunk_min_z + (((bits >> 8) & mask) as i32),
    )
}

pub fn precipitation_sample(
    chance_draw: u32,
    rand_value: &mut i32,
    chunk_min_x: i32,
    chunk_min_z: i32,
) -> Option<BlockPos> {
    if chance_draw != 0 {
        None
    } else {
        Some(advance_random_position(
            rand_value,
            chunk_min_x,
            0,
            chunk_min_z,
            15,
        ))
    }
}

pub fn chunk_precipitation_samples(
    random_tick_speed: u32,
    rand_value: &mut i32,
    chunk_min_x: i32,
    chunk_min_z: i32,
    random: &mut impl WeatherRandom,
) -> Vec<BlockPos> {
    let mut samples = Vec::new();
    for _ in 0..random_tick_speed {
        if let Some(position) = precipitation_sample(
            random.next_int(PRECIPITATION_CHANCE_BOUND),
            rand_value,
            chunk_min_x,
            chunk_min_z,
        ) {
            samples.push(position);
        }
    }
    samples
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precipitation {
    None,
    Rain,
    Snow,
}

pub fn adjusted_temperature(
    base_or_modified: f32,
    temperature_noise: f32,
    y: i32,
    sea_level: i32,
) -> f32 {
    let threshold = sea_level + 17;
    if y <= threshold {
        base_or_modified
    } else {
        base_or_modified - (temperature_noise * 8.0 + (y - threshold) as f32) * 0.05 / 40.0
    }
}

pub fn precipitation_at(configured: bool, temperature: f32) -> Precipitation {
    if !configured {
        Precipitation::None
    } else if temperature < 0.15 {
        Precipitation::Snow
    } else {
        Precipitation::Rain
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreezeProbe {
    pub precipitation: Precipitation,
    pub inside_build_height: bool,
    pub block_light: u8,
    pub source_water: bool,
    pub liquid_block: bool,
    pub horizontal_non_water: bool,
}

pub const fn should_freeze(probe: FreezeProbe) -> bool {
    matches!(probe.precipitation, Precipitation::Snow)
        && probe.inside_build_height
        && probe.block_light < FREEZE_LIGHT_LIMIT
        && probe.source_water
        && probe.liquid_block
        && probe.horizontal_non_water
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnowWrite {
    None,
    DefaultLayer,
    Increase {
        from: u8,
        to: u8,
        push_entities_up: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnowProbe {
    pub active_rain: bool,
    pub precipitation: Precipitation,
    pub max_accumulation: u8,
    pub inside_build_height: bool,
    pub block_light: u8,
    pub air_or_snow: bool,
    pub default_snow_survives: bool,
    pub existing_layers: Option<u8>,
}

pub const fn snow_write(probe: SnowProbe) -> SnowWrite {
    let limit = if probe.max_accumulation < MAX_SNOW_LAYERS {
        probe.max_accumulation
    } else {
        MAX_SNOW_LAYERS
    };
    if !probe.active_rain
        || !matches!(probe.precipitation, Precipitation::Snow)
        || limit == 0
        || !probe.inside_build_height
        || probe.block_light >= FREEZE_LIGHT_LIMIT
        || !probe.air_or_snow
        || !probe.default_snow_survives
    {
        return SnowWrite::None;
    }
    match probe.existing_layers {
        Some(layers) if layers < limit => SnowWrite::Increase {
            from: layers,
            to: layers + 1,
            push_entities_up: true,
        },
        Some(_) => SnowWrite::None,
        None => SnowWrite::DefaultLayer,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecipitationStage {
    Freeze,
    Snow,
    Receiver,
}

pub const PRECIPITATION_ORDER: [PrecipitationStage; 3] = [
    PrecipitationStage::Freeze,
    PrecipitationStage::Snow,
    PrecipitationStage::Receiver,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrecipitationTransaction {
    pub freeze: bool,
    pub snow: SnowWrite,
    pub receiver: Option<Precipitation>,
}

pub const fn precipitation_transaction(
    freeze_probe: FreezeProbe,
    snow_probe: SnowProbe,
    below_precipitation: Precipitation,
) -> PrecipitationTransaction {
    let freeze = should_freeze(freeze_probe);
    if !snow_probe.active_rain {
        return PrecipitationTransaction {
            freeze,
            snow: SnowWrite::None,
            receiver: None,
        };
    }
    PrecipitationTransaction {
        freeze,
        snow: snow_write(snow_probe),
        receiver: if matches!(below_precipitation, Precipitation::None) {
            None
        } else {
            Some(below_precipitation)
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronKind {
    Empty,
    Water { level: u8 },
    PowderSnow { level: u8 },
    Lava,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CauldronResult {
    pub draw_consumed: bool,
    pub replacement: Option<CauldronKind>,
    pub emit_block_change: bool,
}

pub fn cauldron_precipitation(
    kind: CauldronKind,
    precipitation: Precipitation,
    chance_draw: f32,
) -> CauldronResult {
    let chance = match precipitation {
        Precipitation::None => {
            return CauldronResult {
                draw_consumed: false,
                replacement: None,
                emit_block_change: false,
            };
        }
        Precipitation::Rain => RAIN_CAULDRON_CHANCE,
        Precipitation::Snow => SNOW_CAULDRON_CHANCE,
    };
    if matches!(kind, CauldronKind::Lava | CauldronKind::Other) {
        return CauldronResult {
            draw_consumed: false,
            replacement: None,
            emit_block_change: false,
        };
    }
    if chance_draw >= chance {
        return CauldronResult {
            draw_consumed: true,
            replacement: None,
            emit_block_change: false,
        };
    }
    let replacement = match (kind, precipitation) {
        (CauldronKind::Empty, Precipitation::Rain) => Some(CauldronKind::Water { level: 1 }),
        (CauldronKind::Empty, Precipitation::Snow) => Some(CauldronKind::PowderSnow { level: 1 }),
        (CauldronKind::Water { level }, Precipitation::Rain) if level < 3 => {
            Some(CauldronKind::Water { level: level + 1 })
        }
        (CauldronKind::PowderSnow { level }, Precipitation::Snow) if level < 3 => {
            Some(CauldronKind::PowderSnow { level: level + 1 })
        }
        _ => None,
    };
    CauldronResult {
        draw_consumed: true,
        emit_block_change: matches!(kind, CauldronKind::Empty) && replacement.is_some(),
        replacement,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalRainProbe {
    pub level_raining: bool,
    pub sky_visible: bool,
    pub motion_blocking_y: i32,
    pub position: BlockPos,
    pub precipitation: Precipitation,
}

pub const fn is_raining_at(probe: LocalRainProbe) -> bool {
    probe.level_raining
        && probe.sky_visible
        && probe.motion_blocking_y <= probe.position.y
        && matches!(probe.precipitation, Precipitation::Rain)
}

pub fn lightning_column(
    raining: bool,
    thundering: bool,
    chance_draw: u32,
    rand_value: &mut i32,
    chunk_min_x: i32,
    chunk_min_z: i32,
) -> Option<BlockPos> {
    if !raining || !thundering || chance_draw != 0 {
        None
    } else {
        Some(advance_random_position(
            rand_value,
            chunk_min_x,
            0,
            chunk_min_z,
            15,
        ))
    }
}

pub const fn thunder_attempted(spawning_chunk: bool, entity_ticking_range: bool) -> bool {
    spawning_chunk && entity_ticking_range
}

pub const fn rod_matches_surface(rod: BlockPos, world_surface_y: i32) -> bool {
    rod.y == world_surface_y - 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightningSearchVolume {
    pub surface: BlockPos,
    pub top_exclusive_y: i32,
    pub inflation: i32,
}

pub const fn lightning_search_volume(surface: BlockPos, level_max_y: i32) -> LightningSearchVolume {
    LightningSearchVolume {
        surface,
        top_exclusive_y: level_max_y + 1,
        inflation: LIGHTNING_ENTITY_INFLATION,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningTargetKind {
    Rod,
    LivingEntity,
    SurfaceFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightningTarget {
    pub position: BlockPos,
    pub kind: LightningTargetKind,
    pub selection_draw_consumed: bool,
}

pub fn select_lightning_target(
    surface: BlockPos,
    min_y: i32,
    rod: Option<BlockPos>,
    living_entities: &[BlockPos],
    entity_draw: u32,
) -> LightningTarget {
    if let Some(rod) = rod {
        return LightningTarget {
            position: BlockPos::new(rod.x, rod.y + 1, rod.z),
            kind: LightningTargetKind::Rod,
            selection_draw_consumed: false,
        };
    }
    if !living_entities.is_empty() {
        return LightningTarget {
            position: living_entities[entity_draw as usize],
            kind: LightningTargetKind::LivingEntity,
            selection_draw_consumed: true,
        };
    }
    LightningTarget {
        position: if surface.y == min_y - 1 {
            BlockPos::new(surface.x, surface.y + 2, surface.z)
        } else {
            surface
        },
        kind: LightningTargetKind::SurfaceFallback,
        selection_draw_consumed: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrapDecision {
    pub selected: bool,
    pub draw_consumed: bool,
}

pub fn skeleton_trap_decision(
    spawn_mobs: bool,
    effective_difficulty: f64,
    below_is_lightning_rod: bool,
    chance_draw: f64,
) -> TrapDecision {
    if !spawn_mobs {
        return TrapDecision {
            selected: false,
            draw_consumed: false,
        };
    }
    TrapDecision {
        selected: chance_draw < effective_difficulty * 0.01 && !below_is_lightning_rod,
        draw_consumed: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherSpawnAnchor {
    IntegerCorner(BlockPos),
    BottomCenter(BlockPos),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkeletonHorseSpawn {
    pub event_spawn: bool,
    pub trap: bool,
    pub age: i32,
    pub anchor: WeatherSpawnAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightningBoltSpawn {
    pub visual_only: bool,
    pub anchor: WeatherSpawnAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherSpawnPlan {
    pub horse: Option<SkeletonHorseSpawn>,
    pub bolt: LightningBoltSpawn,
}

pub const fn weather_spawn_plan(target: BlockPos, trap: bool) -> WeatherSpawnPlan {
    WeatherSpawnPlan {
        horse: if trap {
            Some(SkeletonHorseSpawn {
                event_spawn: true,
                trap: true,
                age: 0,
                anchor: WeatherSpawnAnchor::IntegerCorner(target),
            })
        } else {
            None
        },
        bolt: LightningBoltSpawn {
            visual_only: trap,
            anchor: WeatherSpawnAnchor::BottomCenter(target),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherEntityCommit {
    pub horse_factory_attempted: bool,
    pub horse_admission_attempted: bool,
    pub horse_admitted: bool,
    pub bolt_factory_attempted: bool,
    pub bolt_admission_attempted: bool,
    pub bolt_admitted: bool,
    pub bolt_visual_only: bool,
}

pub const fn weather_entity_commit(
    trap: bool,
    horse_created: bool,
    horse_admission_succeeded: bool,
    bolt_created: bool,
    bolt_admission_succeeded: bool,
) -> WeatherEntityCommit {
    WeatherEntityCommit {
        horse_factory_attempted: trap,
        horse_admission_attempted: trap && horse_created,
        horse_admitted: trap && horse_created && horse_admission_succeeded,
        bolt_factory_attempted: true,
        bolt_admission_attempted: bolt_created,
        bolt_admitted: bolt_created && bolt_admission_succeeded,
        bolt_visual_only: trap,
    }
}
