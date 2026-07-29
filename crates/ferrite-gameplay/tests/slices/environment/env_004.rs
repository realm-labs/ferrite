use ferrite_foundation::coordinate::BlockPos;
use ferrite_gameplay::environment::weather::{
    CHUNK_WEATHER_ORDER, CauldronKind, CauldronResult, ChunkWeatherStage, ClientWeather,
    DEFAULT_MAX_SNOW_ACCUMULATION, DEFAULT_RANDOM_TICK_SPEED, FREEZE_LIGHT_LIMIT, FreezeProbe,
    LEVEL_WEATHER_ORDER, LIGHTNING_CHANCE_BOUND, LIGHTNING_ENTITY_INFLATION, LIGHTNING_ROD_RADIUS,
    LevelWeatherStage, LightningSearchVolume, LightningTargetKind, LocalRainProbe, MAX_SNOW_LAYERS,
    PERSISTED_WEATHER_ORDER, PRECIPITATION_CHANCE_BOUND, PRECIPITATION_ORDER, Precipitation,
    PrecipitationStage, RAIN_CAULDRON_CHANCE, RAIN_DELAY_MAX, RAIN_DELAY_MIN, RAIN_DURATION_MAX,
    RAIN_DURATION_MIN, RAINING_THRESHOLD, SNOW_CAULDRON_CHANCE, SnowProbe, SnowWrite,
    THUNDER_DELAY_MAX, THUNDER_DELAY_MIN, THUNDER_DURATION_MAX, THUNDER_DURATION_MIN,
    THUNDERING_THRESHOLD, TrapDecision, WeatherCommand, WeatherData, WeatherDimension,
    WeatherField, WeatherPacket, WeatherPacketKind, WeatherPacketScope, WeatherRandom,
    WeatherSpawnAnchor, WeatherStrengths, adjusted_temperature, advance_random_position,
    advance_weather_targets, cauldron_precipitation, chunk_precipitation_samples,
    clear_weather_after_sleep, command_weather, is_raining_at, join_weather_packets,
    lightning_column, lightning_search_volume, precipitation_at, precipitation_sample,
    precipitation_transaction, rod_matches_surface, run_weather_phase, select_lightning_target,
    should_freeze, skeleton_trap_decision, snow_write, thunder_attempted, weather_entity_commit,
    weather_spawn_plan,
};

#[derive(Debug, Default)]
struct RandomScript {
    ints: Vec<u32>,
    index: usize,
    bounds: Vec<u32>,
}

impl RandomScript {
    fn new(ints: Vec<u32>) -> Self {
        Self {
            ints,
            ..Self::default()
        }
    }
}

impl WeatherRandom for RandomScript {
    fn next_int(&mut self, bound: u32) -> u32 {
        self.bounds.push(bound);
        let value = self.ints[self.index];
        self.index += 1;
        assert!(value < bound);
        value
    }
}

fn overworld() -> WeatherDimension {
    WeatherDimension {
        has_sky_light: true,
        has_ceiling: false,
        is_end: false,
    }
}

#[test]
fn phase_order_and_constants_match_the_audited_pipeline() {
    assert_eq!(
        LEVEL_WEATHER_ORDER,
        [
            LevelWeatherStage::WorldBorder,
            LevelWeatherStage::Weather,
            LevelWeatherStage::Sleep,
            LevelWeatherStage::Clock,
            LevelWeatherStage::ScheduledBlockTicks,
            LevelWeatherStage::ScheduledFluidTicks,
            LevelWeatherStage::ChunkWork,
        ]
    );
    assert_eq!(
        CHUNK_WEATHER_ORDER,
        [
            ChunkWeatherStage::ShuffleSpawningChunks,
            ChunkWeatherStage::Thunder,
            ChunkWeatherStage::NaturalSpawning,
            ChunkWeatherStage::Precipitation,
            ChunkWeatherStage::RandomBlockAndFluidTicks,
            ChunkWeatherStage::CustomSpawners,
        ]
    );
    assert_eq!(
        PERSISTED_WEATHER_ORDER,
        [
            WeatherField::ClearTime,
            WeatherField::RainTime,
            WeatherField::ThunderTime,
            WeatherField::Raining,
            WeatherField::Thundering,
        ]
    );
    assert_eq!(
        (
            DEFAULT_RANDOM_TICK_SPEED,
            DEFAULT_MAX_SNOW_ACCUMULATION,
            MAX_SNOW_LAYERS,
            FREEZE_LIGHT_LIMIT,
            PRECIPITATION_CHANCE_BOUND,
            LIGHTNING_CHANCE_BOUND,
            LIGHTNING_ROD_RADIUS,
            LIGHTNING_ENTITY_INFLATION
        ),
        (3, 1, 8, 10, 48, 100_000, 128, 3)
    );
}

#[test]
fn capability_requires_skylight_no_ceiling_and_non_end_key() {
    assert!(overworld().can_have_weather());
    assert!(
        !WeatherDimension {
            has_sky_light: false,
            ..overworld()
        }
        .can_have_weather()
    );
    assert!(
        !WeatherDimension {
            has_ceiling: true,
            ..overworld()
        }
        .can_have_weather()
    );
    assert!(
        !WeatherDimension {
            is_end: true,
            ..overworld()
        }
        .can_have_weather()
    );
}

#[test]
fn clear_override_decrements_and_rewrites_timers_from_old_targets() {
    let mut data = WeatherData {
        clear_weather_time: 2,
        rain_time: 99,
        thunder_time: 98,
        raining: true,
        thundering: false,
    };
    let mut random = RandomScript::default();
    advance_weather_targets(&mut data, overworld(), true, true, &mut random);
    assert_eq!(
        data,
        WeatherData {
            clear_weather_time: 1,
            rain_time: 0,
            thunder_time: 1,
            raining: false,
            thundering: false,
        }
    );
    assert!(random.bounds.is_empty());

    advance_weather_targets(&mut data, overworld(), true, true, &mut random);
    assert_eq!(data.clear_weather_time, 0);
    assert_eq!((data.rain_time, data.thunder_time), (1, 1));
}

#[test]
fn thunder_then_rain_timers_toggle_at_one_and_sample_at_zero() {
    let mut data = WeatherData {
        rain_time: 1,
        thunder_time: 1,
        ..WeatherData::default()
    };
    let mut random = RandomScript::default();
    advance_weather_targets(&mut data, overworld(), true, true, &mut random);
    assert_eq!((data.thundering, data.raining), (true, true));
    assert_eq!((data.thunder_time, data.rain_time), (0, 0));
    assert!(random.bounds.is_empty());

    let mut random = RandomScript::new(vec![0, RAIN_DURATION_MAX - RAIN_DURATION_MIN]);
    advance_weather_targets(&mut data, overworld(), true, true, &mut random);
    assert_eq!(data.thunder_time, THUNDER_DURATION_MIN as i32);
    assert_eq!(data.rain_time, RAIN_DURATION_MAX as i32);
    assert_eq!(
        random.bounds,
        vec![
            THUNDER_DURATION_MAX - THUNDER_DURATION_MIN + 1,
            RAIN_DURATION_MAX - RAIN_DURATION_MIN + 1
        ]
    );

    data = WeatherData::default();
    let mut random = RandomScript::new(vec![
        THUNDER_DELAY_MAX - THUNDER_DELAY_MIN,
        RAIN_DELAY_MAX - RAIN_DELAY_MIN,
    ]);
    advance_weather_targets(&mut data, overworld(), true, true, &mut random);
    assert_eq!(data.thunder_time, THUNDER_DELAY_MAX as i32);
    assert_eq!(data.rain_time, RAIN_DELAY_MAX as i32);

    data = WeatherData {
        rain_time: 1,
        thunder_time: 1,
        raining: true,
        thundering: true,
        ..WeatherData::default()
    };
    let mut random = RandomScript::default();
    advance_weather_targets(&mut data, overworld(), true, true, &mut random);
    assert_eq!((data.thundering, data.raining), (false, false));
}

#[test]
fn frozen_incapable_and_disabled_ticks_preserve_targets_and_draws() {
    for (dimension, normal, advance) in [
        (overworld(), false, true),
        (
            WeatherDimension {
                has_sky_light: false,
                ..overworld()
            },
            true,
            true,
        ),
        (overworld(), true, false),
    ] {
        let original = WeatherData {
            rain_time: 1,
            thunder_time: 1,
            raining: true,
            thundering: true,
            ..WeatherData::default()
        };
        let mut data = original;
        let mut random = RandomScript::default();
        advance_weather_targets(&mut data, dimension, normal, advance, &mut random);
        assert_eq!(data, original);
        assert!(random.bounds.is_empty());
    }
}

#[test]
fn capable_levels_advance_one_shared_record_in_level_order_but_keep_local_ramps() {
    let mut shared = WeatherData {
        rain_time: 3,
        thunder_time: 3,
        raining: true,
        thundering: false,
        ..WeatherData::default()
    };
    let mut first = WeatherStrengths::default();
    let mut second = WeatherStrengths {
        rain: 0.5,
        thunder: 0.5,
        ..WeatherStrengths::default()
    };
    let mut random = RandomScript::default();
    run_weather_phase(
        &mut shared,
        &mut first,
        overworld(),
        true,
        true,
        &mut random,
    );
    run_weather_phase(
        &mut shared,
        &mut second,
        overworld(),
        true,
        true,
        &mut random,
    );
    assert_eq!((shared.rain_time, shared.thunder_time), (1, 1));
    assert_eq!((first.rain, second.rain), (0.01, 0.51));
    assert_eq!((first.thunder, second.thunder), (0.0, 0.49));
    assert!(random.bounds.is_empty());
}

#[test]
fn strengths_ramp_and_publish_when_timer_advancement_is_disabled() {
    let mut data = WeatherData {
        raining: true,
        thundering: true,
        ..WeatherData::default()
    };
    let mut strengths = WeatherStrengths {
        rain: 0.2,
        thunder: 0.9,
        ..WeatherStrengths::default()
    };
    let mut random = RandomScript::default();
    let phase = run_weather_phase(
        &mut data,
        &mut strengths,
        overworld(),
        true,
        false,
        &mut random,
    );
    assert_eq!(
        (strengths.previous_thunder, strengths.thunder),
        (0.9, 0.90999997)
    );
    assert_eq!((strengths.previous_rain, strengths.rain), (0.2, 0.21000001));
    assert_eq!(
        phase.packets,
        vec![
            WeatherPacket {
                scope: WeatherPacketScope::Dimension,
                kind: WeatherPacketKind::ThunderStrength(0.90999997),
            },
            WeatherPacket {
                scope: WeatherPacketScope::Dimension,
                kind: WeatherPacketKind::RainStrength(0.21000001),
            },
            WeatherPacket {
                scope: WeatherPacketScope::Global,
                kind: WeatherPacketKind::StartRaining,
            },
            WeatherPacket {
                scope: WeatherPacketScope::Global,
                kind: WeatherPacketKind::RainStrength(0.21000001),
            },
            WeatherPacket {
                scope: WeatherPacketScope::Global,
                kind: WeatherPacketKind::ThunderStrength(0.90999997),
            },
        ]
    );
    assert!(strengths.is_raining(true));
    assert!(!strengths.is_thundering(true));
    strengths.rain = 1.0;
    strengths.thunder = 0.90000004;
    assert!(strengths.is_thundering(true));
    assert_eq!((RAINING_THRESHOLD, THUNDERING_THRESHOLD), (0.2, 0.9));

    data.raining = false;
    data.thundering = false;
    strengths.rain = 0.21;
    strengths.thunder = 0.0;
    let stop = run_weather_phase(
        &mut data,
        &mut strengths,
        overworld(),
        true,
        false,
        &mut random,
    );
    assert_eq!(stop.packets.len(), 4);
    assert!(matches!(
        stop.packets[0],
        WeatherPacket {
            scope: WeatherPacketScope::Dimension,
            kind: WeatherPacketKind::RainStrength(_)
        }
    ));
    assert_eq!(stop.packets[1].kind, WeatherPacketKind::StopRaining);
    assert_eq!(
        stop.packets[2].kind,
        WeatherPacketKind::RainStrength(strengths.rain)
    );
    assert_eq!(
        stop.packets[3].kind,
        WeatherPacketKind::ThunderStrength(strengths.thunder)
    );
}

#[test]
fn saved_initialization_commands_and_sleep_keep_strengths_separate() {
    let rainy = WeatherData {
        raining: true,
        thundering: true,
        ..WeatherData::default()
    };
    assert_eq!(
        WeatherStrengths::from_saved(rainy, true),
        WeatherStrengths {
            previous_rain: 1.0,
            rain: 1.0,
            previous_thunder: 1.0,
            thunder: 1.0,
        }
    );
    assert_eq!(
        WeatherStrengths::from_saved(
            WeatherData {
                raining: false,
                thundering: true,
                ..WeatherData::default()
            },
            true
        )
        .thunder,
        0.0
    );

    let mut random = RandomScript::default();
    assert_eq!(
        command_weather(WeatherCommand::Clear, Some(40), &mut random),
        WeatherData {
            clear_weather_time: 40,
            ..WeatherData::default()
        }
    );
    assert_eq!(
        command_weather(WeatherCommand::Rain, Some(41), &mut random),
        WeatherData {
            rain_time: 41,
            thunder_time: 41,
            raining: true,
            thundering: false,
            ..WeatherData::default()
        }
    );
    assert_eq!(
        command_weather(WeatherCommand::Thunder, Some(42), &mut random),
        WeatherData {
            rain_time: 42,
            thunder_time: 42,
            raining: true,
            thundering: true,
            ..WeatherData::default()
        }
    );

    let strengths = WeatherStrengths {
        rain: 0.21,
        thunder: 0.5,
        ..WeatherStrengths::default()
    };
    let mut data = rainy;
    assert!(clear_weather_after_sleep(&mut data, strengths, true, true));
    assert_eq!(data, WeatherData::default());
    assert_eq!(strengths.rain, 0.21);
}

#[test]
fn omitted_command_durations_use_the_command_source_level_stream() {
    let mut clear_random = RandomScript::new(vec![RAIN_DELAY_MAX - RAIN_DELAY_MIN]);
    let clear = command_weather(WeatherCommand::Clear, None, &mut clear_random);
    assert_eq!(clear.clear_weather_time, RAIN_DELAY_MAX as i32);

    let mut rain_random = RandomScript::new(vec![0]);
    let rain = command_weather(WeatherCommand::Rain, None, &mut rain_random);
    assert_eq!(rain.rain_time, RAIN_DURATION_MIN as i32);

    let mut thunder_random = RandomScript::new(vec![THUNDER_DURATION_MAX - THUNDER_DURATION_MIN]);
    let thunder = command_weather(WeatherCommand::Thunder, None, &mut thunder_random);
    assert_eq!(thunder.thunder_time, THUNDER_DURATION_MAX as i32);
}

#[test]
fn clients_snap_to_packets_and_join_only_projects_active_rain() {
    let mut client = ClientWeather::default();
    client.apply(WeatherPacketKind::StopRaining);
    assert_eq!((client.previous_rain, client.rain), (1.0, 1.0));
    client.apply(WeatherPacketKind::RainStrength(-2.0));
    client.apply(WeatherPacketKind::ThunderStrength(3.0));
    assert_eq!((client.previous_rain, client.rain), (0.0, 0.0));
    assert_eq!((client.previous_thunder, client.thunder), (1.0, 1.0));

    let strengths = WeatherStrengths {
        rain: 0.8,
        thunder: 0.4,
        ..WeatherStrengths::default()
    };
    assert_eq!(
        join_weather_packets(strengths, true),
        vec![
            WeatherPacketKind::StartRaining,
            WeatherPacketKind::RainStrength(0.8),
            WeatherPacketKind::ThunderStrength(0.4),
        ]
    );
    assert!(join_weather_packets(strengths, false).is_empty());
}

#[test]
fn separate_position_stream_advances_only_on_precipitation_hits() {
    let mut stream = 7_i32;
    let before = stream;
    assert_eq!(precipitation_sample(1, &mut stream, 32, -16), None);
    assert_eq!(stream, before);

    let expected_stream = before.wrapping_mul(3).wrapping_add(1_013_904_223);
    let bits = (expected_stream as u32) >> 2;
    let hit = precipitation_sample(0, &mut stream, 32, -16).unwrap();
    assert_eq!(stream, expected_stream);
    assert_eq!(
        hit,
        BlockPos::new(
            32 + (bits & 15) as i32,
            ((bits >> 16) & 15) as i32,
            -16 + ((bits >> 8) & 15) as i32,
        )
    );

    let direct = advance_random_position(&mut stream, 0, 64, 0, 15);
    assert!((64..80).contains(&direct.y));

    let mut disabled_stream = 19;
    let mut disabled_random = RandomScript::default();
    assert!(
        chunk_precipitation_samples(0, &mut disabled_stream, 0, 0, &mut disabled_random).is_empty()
    );
    assert_eq!(disabled_stream, 19);
    assert!(disabled_random.bounds.is_empty());

    let mut scripted_stream = 19;
    let mut scripted_random = RandomScript::new(vec![1, 0, 0]);
    assert_eq!(
        chunk_precipitation_samples(3, &mut scripted_stream, 0, 0, &mut scripted_random).len(),
        2
    );
    assert_eq!(scripted_random.bounds, vec![48, 48, 48]);

    let mut one_stream = 19;
    let mut one_random = RandomScript::new(vec![0]);
    assert_eq!(
        chunk_precipitation_samples(1, &mut one_stream, 0, 0, &mut one_random).len(),
        1
    );
    assert_eq!(one_random.bounds, vec![48]);
}

#[test]
fn temperature_freeze_and_snow_boundaries_are_exact() {
    assert_eq!(
        PRECIPITATION_ORDER,
        [
            PrecipitationStage::Freeze,
            PrecipitationStage::Snow,
            PrecipitationStage::Receiver,
        ]
    );
    assert_eq!(adjusted_temperature(0.2, 1.0, 81, 64), 0.2);
    assert!(adjusted_temperature(0.2, 1.0, 82, 64) < 0.2);
    assert_eq!(precipitation_at(false, -1.0), Precipitation::None);
    assert_eq!(precipitation_at(true, 0.15), Precipitation::Rain);
    assert_eq!(precipitation_at(true, 0.149), Precipitation::Snow);

    let freeze = FreezeProbe {
        precipitation: Precipitation::Snow,
        inside_build_height: true,
        block_light: 9,
        source_water: true,
        liquid_block: true,
        horizontal_non_water: true,
    };
    assert!(should_freeze(freeze));
    assert!(!should_freeze(FreezeProbe {
        block_light: 10,
        ..freeze
    }));
    assert!(!should_freeze(FreezeProbe {
        horizontal_non_water: false,
        ..freeze
    }));

    let snow = SnowProbe {
        active_rain: true,
        precipitation: Precipitation::Snow,
        max_accumulation: 8,
        inside_build_height: true,
        block_light: 9,
        air_or_snow: true,
        default_snow_survives: true,
        existing_layers: Some(7),
    };
    assert_eq!(
        snow_write(snow),
        SnowWrite::Increase {
            from: 7,
            to: 8,
            push_entities_up: true,
        }
    );
    assert_eq!(
        snow_write(SnowProbe {
            existing_layers: Some(8),
            ..snow
        }),
        SnowWrite::None
    );
    assert_eq!(
        snow_write(SnowProbe {
            existing_layers: None,
            ..snow
        }),
        SnowWrite::DefaultLayer
    );
    assert_eq!(
        snow_write(SnowProbe {
            max_accumulation: 0,
            ..snow
        }),
        SnowWrite::None
    );

    let dry = precipitation_transaction(
        freeze,
        SnowProbe {
            active_rain: false,
            ..snow
        },
        Precipitation::Snow,
    );
    assert!(dry.freeze);
    assert_eq!(dry.snow, SnowWrite::None);
    assert_eq!(dry.receiver, None);

    let wet = precipitation_transaction(freeze, snow, Precipitation::Snow);
    assert!(wet.freeze);
    assert!(matches!(wet.snow, SnowWrite::Increase { .. }));
    assert_eq!(wet.receiver, Some(Precipitation::Snow));
}

#[test]
fn cauldrons_draw_before_type_or_fullness_rejection() {
    assert_eq!((RAIN_CAULDRON_CHANCE, SNOW_CAULDRON_CHANCE), (0.05, 0.1));
    assert_eq!(
        cauldron_precipitation(CauldronKind::Empty, Precipitation::Rain, 0.049),
        CauldronResult {
            draw_consumed: true,
            replacement: Some(CauldronKind::Water { level: 1 }),
            emit_block_change: true,
        }
    );
    assert_eq!(
        cauldron_precipitation(CauldronKind::Empty, Precipitation::Rain, 0.05).replacement,
        None
    );
    assert_eq!(
        cauldron_precipitation(CauldronKind::Empty, Precipitation::Snow, 0.099).replacement,
        Some(CauldronKind::PowderSnow { level: 1 })
    );
    assert_eq!(
        cauldron_precipitation(CauldronKind::Empty, Precipitation::Snow, 0.1).replacement,
        None
    );
    assert_eq!(
        cauldron_precipitation(CauldronKind::Water { level: 2 }, Precipitation::Rain, 0.0)
            .replacement,
        Some(CauldronKind::Water { level: 3 })
    );
    assert_eq!(
        cauldron_precipitation(
            CauldronKind::PowderSnow { level: 2 },
            Precipitation::Snow,
            0.0
        )
        .replacement,
        Some(CauldronKind::PowderSnow { level: 3 })
    );
    let wrong = cauldron_precipitation(CauldronKind::Water { level: 2 }, Precipitation::Snow, 0.0);
    assert!(wrong.draw_consumed);
    assert_eq!(wrong.replacement, None);
    let full = cauldron_precipitation(CauldronKind::Water { level: 3 }, Precipitation::Rain, 0.0);
    assert!(full.draw_consumed);
    assert_eq!(full.replacement, None);
    assert!(!cauldron_precipitation(CauldronKind::Lava, Precipitation::Rain, 0.0).draw_consumed);
    assert!(!cauldron_precipitation(CauldronKind::Other, Precipitation::Rain, 0.0).draw_consumed);
    assert!(!cauldron_precipitation(CauldronKind::Empty, Precipitation::None, 0.0).draw_consumed);
}

#[test]
fn lightning_admission_targets_and_local_rain_filter_are_separate() {
    assert!(thunder_attempted(true, true));
    assert!(!thunder_attempted(true, false));
    let mut stream = 11;
    let unchanged = stream;
    assert_eq!(lightning_column(true, true, 1, &mut stream, 0, 0), None);
    assert_eq!(stream, unchanged);
    let column = lightning_column(true, true, 0, &mut stream, 0, 0).unwrap();
    assert_eq!(
        lightning_search_volume(column, 320),
        LightningSearchVolume {
            surface: column,
            top_exclusive_y: 321,
            inflation: 3,
        }
    );

    let rod = BlockPos::new(column.x, 70, column.z);
    assert!(rod_matches_surface(rod, 71));
    assert!(!rod_matches_surface(rod, 72));
    let rod_target = select_lightning_target(column, -64, Some(rod), &[], 0);
    assert_eq!(rod_target.kind, LightningTargetKind::Rod);
    assert_eq!(rod_target.position.y, 71);
    assert!(!rod_target.selection_draw_consumed);

    let entities = [BlockPos::new(1, 80, 2), BlockPos::new(3, 90, 4)];
    let entity_target = select_lightning_target(column, -64, None, &entities, 1);
    assert_eq!(entity_target.position, entities[1]);
    assert!(entity_target.selection_draw_consumed);

    let fallback = select_lightning_target(BlockPos::new(4, -65, 5), -64, None, &[], 0);
    assert_eq!(fallback.position, BlockPos::new(4, -63, 5));
    assert!(is_raining_at(LocalRainProbe {
        level_raining: true,
        sky_visible: true,
        motion_blocking_y: 80,
        position: BlockPos::new(0, 80, 0),
        precipitation: Precipitation::Rain,
    }));
    assert!(!is_raining_at(LocalRainProbe {
        precipitation: Precipitation::Snow,
        level_raining: true,
        sky_visible: true,
        motion_blocking_y: 80,
        position: BlockPos::new(0, 80, 0),
    }));
    assert!(!is_raining_at(LocalRainProbe {
        level_raining: true,
        sky_visible: false,
        motion_blocking_y: 80,
        position: BlockPos::new(0, 80, 0),
        precipitation: Precipitation::Rain,
    }));
    assert!(!is_raining_at(LocalRainProbe {
        level_raining: true,
        sky_visible: true,
        motion_blocking_y: 81,
        position: BlockPos::new(0, 80, 0),
        precipitation: Precipitation::Rain,
    }));
}

#[test]
fn trap_draw_and_entity_failures_never_roll_back_the_bolt_plan() {
    assert_eq!(
        skeleton_trap_decision(false, 4.0, false, 0.0),
        TrapDecision {
            selected: false,
            draw_consumed: false,
        }
    );
    assert_eq!(
        skeleton_trap_decision(true, 4.0, false, 0.04),
        TrapDecision {
            selected: false,
            draw_consumed: true,
        }
    );
    assert!(skeleton_trap_decision(true, 4.0, false, 0.039).selected);
    assert!(!skeleton_trap_decision(true, 4.0, true, 0.0).selected);

    let target = BlockPos::new(4, 80, -2);
    let spawn = weather_spawn_plan(target, true);
    let horse = spawn.horse.unwrap();
    assert!(horse.event_spawn);
    assert!(horse.trap);
    assert_eq!(horse.age, 0);
    assert_eq!(horse.anchor, WeatherSpawnAnchor::IntegerCorner(target));
    assert!(spawn.bolt.visual_only);
    assert_eq!(spawn.bolt.anchor, WeatherSpawnAnchor::BottomCenter(target));
    let ordinary = weather_spawn_plan(target, false);
    assert!(ordinary.horse.is_none());
    assert!(!ordinary.bolt.visual_only);

    let failed_horse = weather_entity_commit(true, true, false, true, true);
    assert!(failed_horse.horse_admission_attempted);
    assert!(!failed_horse.horse_admitted);
    assert!(failed_horse.bolt_admitted);
    assert!(failed_horse.bolt_visual_only);

    let absent_factories = weather_entity_commit(true, false, false, false, false);
    assert!(absent_factories.horse_factory_attempted);
    assert!(!absent_factories.horse_admission_attempted);
    assert!(absent_factories.bolt_factory_attempted);
    assert!(!absent_factories.bolt_admission_attempted);
    assert!(absent_factories.bolt_visual_only);
}
