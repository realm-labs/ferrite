use std::collections::{BTreeMap, BTreeSet};

use ferrite_world::generation::dimension::clock::{Clock, ClockError, ClockManager, SavedClock};
use ferrite_world::generation::dimension::environment::{
    AttributeEntry, AttributeMap, AttributeProbe, AttributeValue, BedRule, EnvironmentCache,
    EnvironmentLayers, LayerKind, Modifier, Sanitizer, SleepRule, SpawnRule, block_center,
    declaration_by_id, gaussian_camera_cells, locked_declarations, weighted_spatial_sample,
};
use ferrite_world::generation::dimension::spawn::{
    AnchorInteraction, LightSample, SleepDenial, SpawnCandidate, SpawnColumnHeight,
    SpawnSurfaceChecks, initial_spawn_column_height, initial_spawn_plan, interact_with_bed,
    interact_with_respawn_anchor, map_sample_radius, monster_dark_enough,
    natural_spawn_requires_air_descent, retained_bed_respawn_allowed,
    sleeping_player_remains_asleep,
};
use ferrite_world::generation::dimension::timeline::{
    AttributeTrack, Easing, Keyframe, TimeMarker, Timeline, TimelineError, locked_day_timeline,
    locked_early_game_timeline, locked_moon_timeline, locked_timelines,
    locked_villager_schedule_timeline,
};
use ferrite_world::generation::dimension::{
    CardinalLight, DimensionType, DimensionTypeError, LockedDimension, MAX_COORDINATE_SCALE,
    MAX_HEIGHT, MIN_COORDINATE_SCALE, MIN_Y, Position, Skybox, SpawnLightLevel,
    dimension_storage_folder, scale_command_position, teleportation_scale,
};

fn blank_dimension(
    scale: f64,
    min_y: i32,
    height: u32,
    logical_height: u32,
    block_light_limit: u8,
    spawn_light: SpawnLightLevel,
) -> Result<DimensionType, DimensionTypeError> {
    DimensionType::validate(DimensionType {
        has_fixed_time: false,
        has_skylight: true,
        has_ceiling: false,
        has_ender_dragon_fight: false,
        coordinate_scale: scale,
        min_y,
        height,
        logical_height,
        infiniburn: "#minecraft:test".to_owned(),
        ambient_light: 0.0,
        monster_spawn_block_light_limit: block_light_limit,
        monster_spawn_light_level: spawn_light,
        skybox: Some(Skybox::Overworld),
        cardinal_light: CardinalLight::Default,
        attributes: AttributeMap::new(),
        timelines: Vec::new(),
        default_clock: None,
    })
}

#[test]
fn wgen_dimension_001_locks_all_four_dimension_records() {
    let overworld = DimensionType::locked(LockedDimension::Overworld);
    assert_eq!(overworld.min_y, -64);
    assert_eq!((overworld.height, overworld.logical_height), (384, 384));
    assert_eq!(overworld.monster_spawn_block_light_limit, 0);
    assert_eq!(
        overworld.monster_spawn_light_level,
        SpawnLightLevel::UniformInclusive {
            minimum: 0,
            maximum: 7
        }
    );
    assert_eq!(overworld.skybox, Some(Skybox::Overworld));
    assert_eq!(
        overworld.default_clock.as_deref(),
        Some("minecraft:overworld")
    );
    assert_eq!(overworld.timelines.len(), 4);

    let caves = DimensionType::locked(LockedDimension::OverworldCaves);
    assert!(caves.has_ceiling);
    let mut caves_without_ceiling = caves.clone();
    caves_without_ceiling.has_ceiling = false;
    assert_eq!(caves_without_ceiling, overworld);

    let end = DimensionType::locked(LockedDimension::TheEnd);
    assert!(end.has_fixed_time && end.has_skylight && end.has_ender_dragon_fight);
    assert!(!end.has_ceiling);
    assert_eq!(end.ambient_light, 0.25);
    assert_eq!(end.skybox, Some(Skybox::End));
    assert_eq!(end.monster_spawn_light_level, SpawnLightLevel::Constant(15));

    let nether = DimensionType::locked(LockedDimension::TheNether);
    assert!(nether.has_fixed_time && nether.has_ceiling && !nether.has_skylight);
    assert_eq!((nether.coordinate_scale, nether.logical_height), (8.0, 128));
    assert_eq!(nether.cardinal_light, CardinalLight::Nether);
    assert_eq!(nether.skybox, None);
    assert_eq!(nether.default_clock, None);
    assert_eq!(nether.timelines, ["minecraft:villager_schedule"]);
}

#[test]
fn wgen_dimension_001_validates_codec_and_constructor_boundaries() {
    assert!(
        blank_dimension(
            MIN_COORDINATE_SCALE,
            MIN_Y,
            16,
            16,
            0,
            SpawnLightLevel::Constant(0)
        )
        .is_ok()
    );
    assert!(
        blank_dimension(
            MAX_COORDINATE_SCALE,
            0,
            16,
            0,
            15,
            SpawnLightLevel::Constant(15)
        )
        .is_ok()
    );
    assert!(matches!(
        blank_dimension(0.0, 0, 16, 16, 0, SpawnLightLevel::Constant(0)),
        Err(DimensionTypeError::CoordinateScale(_))
    ));
    assert!(matches!(
        blank_dimension(f64::NAN, 0, 16, 16, 0, SpawnLightLevel::Constant(0)),
        Err(DimensionTypeError::CoordinateScale(_))
    ));
    assert!(matches!(
        blank_dimension(1.0, 1, 16, 16, 0, SpawnLightLevel::Constant(0)),
        Err(DimensionTypeError::MinimumY(1))
    ));
    assert!(matches!(
        blank_dimension(1.0, 0, 15, 15, 0, SpawnLightLevel::Constant(0)),
        Err(DimensionTypeError::Height(15))
    ));
    assert!(matches!(
        blank_dimension(
            1.0,
            0,
            MAX_HEIGHT,
            MAX_HEIGHT + 1,
            0,
            SpawnLightLevel::Constant(0)
        ),
        Err(DimensionTypeError::LogicalHeight { .. })
    ));
    assert!(matches!(
        blank_dimension(
            1.0,
            0,
            MAX_HEIGHT,
            MAX_HEIGHT,
            0,
            SpawnLightLevel::Constant(0)
        ),
        Err(DimensionTypeError::TopOverflow { .. })
    ));
    assert!(matches!(
        blank_dimension(1.0, 0, 16, 16, 16, SpawnLightLevel::Constant(0)),
        Err(DimensionTypeError::BlockLightLimit(16))
    ));
    assert!(matches!(
        blank_dimension(
            1.0,
            0,
            16,
            16,
            0,
            SpawnLightLevel::UniformInclusive {
                minimum: 8,
                maximum: 7
            }
        ),
        Err(DimensionTypeError::SpawnLight)
    ));
}

#[test]
fn wgen_dimension_001_build_height_is_inclusive_and_logical_height_is_independent() {
    let overworld = DimensionType::locked(LockedDimension::Overworld);
    assert_eq!(overworld.max_y(), 319);
    assert!(!overworld.is_inside_build_height(-65));
    assert!(overworld.is_inside_build_height(-64));
    assert!(overworld.is_inside_build_height(319));
    assert!(!overworld.is_inside_build_height(320));
    assert_eq!(overworld.section_index(-64), Some(0));
    assert_eq!(overworld.section_index(-49), Some(0));
    assert_eq!(overworld.section_index(-48), Some(1));
    assert_eq!(overworld.section_count(), 24);

    let nether = DimensionType::locked(LockedDimension::TheNether);
    assert!(nether.is_inside_build_height(255));
    assert_eq!(nether.logical_height, 128);
}

#[test]
fn wgen_dimension_001_separates_dimension_key_from_type() {
    let end = DimensionType::locked(LockedDimension::TheEnd);
    assert!(!end.can_have_weather("minecraft:the_end"));
    assert!(end.can_have_weather("example:end_type_with_custom_key"));
    assert!(end.has_ender_dragon_fight);

    let overworld = DimensionType::locked(LockedDimension::Overworld);
    assert!(!overworld.can_have_weather("minecraft:the_end"));
    assert!(overworld.can_have_weather("example:surface"));
    assert_eq!(dimension_storage_folder("minecraft:overworld"), "");
    assert_eq!(dimension_storage_folder("minecraft:the_nether"), "DIM-1");
    assert_eq!(dimension_storage_folder("minecraft:the_end"), "DIM1");
    assert_eq!(
        dimension_storage_folder("example:moon/base"),
        "dimensions/example/moon/base"
    );
}

#[test]
fn wgen_dimension_001_fixed_time_only_changes_outside_light_predicates() {
    let overworld = DimensionType::locked(LockedDimension::Overworld);
    assert!(overworld.is_bright_outside(3));
    assert!(!overworld.is_dark_outside(3));
    assert!(!overworld.is_bright_outside(4));
    assert!(overworld.is_dark_outside(4));

    let end = DimensionType::locked(LockedDimension::TheEnd);
    for darken in [0, 3, 4, 15] {
        assert!(!end.is_bright_outside(darken));
        assert!(!end.is_dark_outside(darken));
    }
    assert!((overworld.brightness_ramp(0) - 0.25).abs() < f32::EPSILON);
    assert!((end.brightness_ramp(0) - 0.4375).abs() < f32::EPSILON);
    assert!((end.brightness_ramp(15) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn wgen_dimension_001_scales_only_x_and_z_with_java_doubles() {
    let overworld = DimensionType::locked(LockedDimension::Overworld);
    let nether = DimensionType::locked(LockedDimension::TheNether);
    assert_eq!(teleportation_scale(&overworld, &nether), 0.125);
    assert_eq!(teleportation_scale(&nether, &overworld), 8.0);
    let position = Position {
        x: -1.25,
        y: 300.75,
        z: 0.5,
    };
    assert_eq!(
        scale_command_position(position, &nether, &overworld),
        Position {
            x: -10.0,
            y: 300.75,
            z: 4.0
        }
    );
}

fn signature(value: &AttributeValue) -> String {
    match value {
        AttributeValue::Float(value) => format!("f:{value}"),
        AttributeValue::Integer(value) => format!("i:{value}"),
        AttributeValue::Boolean(value) => format!("b:{value}"),
        AttributeValue::Color(value) => format!("c:{value}"),
        AttributeValue::Identifier(value) => format!("id:{value}"),
        AttributeValue::BedRule(value) => format!(
            "bed:{:?}:{:?}:{}",
            value.can_sleep, value.can_set_spawn, value.explodes
        ),
        AttributeValue::BackgroundMusic(value) => format!(
            "music:{}:{}",
            value.default.is_some(),
            value.creative.is_some()
        ),
        AttributeValue::AmbientSounds(value) => format!("ambient:{}", value.is_some()),
        AttributeValue::IdentifierList(value) => format!("list:{}", value.len()),
    }
}

#[test]
fn wgen_dimension_001_registers_all_48_typed_defaults_in_locked_order() {
    let declarations = locked_declarations();
    assert_eq!(declarations.len(), 48);
    let actual = declarations
        .iter()
        .map(|entry| {
            (
                entry.id.strip_prefix("minecraft:").unwrap(),
                signature(&entry.default),
            )
        })
        .collect::<Vec<_>>();
    let expected = [
        ("visual/fog_color", "c:0"),
        ("visual/fog_start_distance", "f:0"),
        ("visual/fog_end_distance", "f:1024"),
        ("visual/sky_fog_end_distance", "f:512"),
        ("visual/cloud_fog_end_distance", "f:2048"),
        ("visual/water_fog_color", "c:-16448205"),
        ("visual/water_fog_start_distance", "f:-8"),
        ("visual/water_fog_end_distance", "f:96"),
        ("visual/sky_color", "c:0"),
        ("visual/sunrise_sunset_color", "c:0"),
        ("visual/cloud_color", "c:0"),
        ("visual/cloud_height", "f:192.33"),
        ("visual/sun_angle", "f:0"),
        ("visual/moon_angle", "f:0"),
        ("visual/star_angle", "f:0"),
        ("visual/moon_phase", "id:full_moon"),
        ("visual/star_brightness", "f:0"),
        ("visual/block_light_tint", "c:-10100"),
        ("visual/sky_light_color", "c:-1"),
        ("visual/sky_light_factor", "f:1"),
        ("visual/night_vision_color", "c:-6710887"),
        ("visual/ambient_light_color", "c:-16777216"),
        (
            "visual/default_dripstone_particle",
            "id:minecraft:dripping_dripstone_water",
        ),
        ("visual/ambient_particles", "list:0"),
        ("audio/background_music", "music:false:false"),
        ("audio/music_volume", "f:1"),
        ("audio/ambient_sounds", "ambient:false"),
        ("audio/firefly_bush_sounds", "b:false"),
        ("gameplay/sky_light_level", "f:15"),
        ("gameplay/can_start_raid", "b:true"),
        ("gameplay/water_evaporates", "b:false"),
        ("gameplay/bed_rule", "bed:WhenDark:Always:false"),
        ("gameplay/respawn_anchor_works", "b:false"),
        ("gameplay/nether_portal_spawns_piglin", "b:false"),
        ("gameplay/fast_lava", "b:false"),
        ("gameplay/increased_fire_burnout", "b:false"),
        ("gameplay/eyeblossom_open", "id:default"),
        ("gameplay/turtle_egg_hatch_chance", "f:0.002"),
        ("gameplay/piglins_zombify", "b:true"),
        ("gameplay/snow_golem_melts", "b:false"),
        ("gameplay/creaking_active", "b:false"),
        ("gameplay/surface_slime_spawn_chance", "f:0"),
        ("gameplay/cat_waking_up_gift_chance", "f:0"),
        ("gameplay/bees_stay_in_hive", "b:false"),
        ("gameplay/monsters_burn", "b:false"),
        ("gameplay/can_pillager_patrol_spawn", "b:true"),
        ("gameplay/villager_activity", "id:minecraft:idle"),
        ("gameplay/baby_villager_activity", "id:minecraft:idle"),
    ];
    let expected = expected
        .into_iter()
        .map(|(id, value)| (id, value.to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn wgen_dimension_001_locks_attribute_metadata_and_dimension_overrides() {
    let declarations = locked_declarations();
    assert!(
        declarations[..28]
            .iter()
            .all(|value| value.positional && value.syncable)
    );
    assert!(
        declarations[..24].iter().enumerate().all(|(index, value)| {
            value.spatially_interpolated == !matches!(index, 15 | 22 | 23)
        })
    );
    assert!(
        declarations[24..28]
            .iter()
            .all(|value| !value.spatially_interpolated)
    );
    let nonpositional = declarations
        .iter()
        .filter(|value| !value.positional)
        .map(|value| value.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        nonpositional,
        BTreeSet::from([
            "minecraft:gameplay/fast_lava",
            "minecraft:gameplay/sky_light_level"
        ])
    );
    let gameplay_sync = declarations[28..]
        .iter()
        .filter(|value| value.syncable)
        .map(|value| value.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        gameplay_sync,
        BTreeSet::from([
            "minecraft:gameplay/creaking_active",
            "minecraft:gameplay/fast_lava",
            "minecraft:gameplay/piglins_zombify",
            "minecraft:gameplay/sky_light_level",
            "minecraft:gameplay/water_evaporates",
        ])
    );

    let nether = DimensionType::locked(LockedDimension::TheNether);
    assert_eq!(
        nether.attributes["minecraft:gameplay/sky_light_level"].value,
        AttributeValue::Float(4.0)
    );
    assert_eq!(
        nether.attributes["minecraft:gameplay/fast_lava"].value,
        AttributeValue::Boolean(true)
    );
    assert_eq!(
        nether.attributes["minecraft:visual/fog_start_distance"].value,
        AttributeValue::Float(10.0)
    );
    let end = DimensionType::locked(LockedDimension::TheEnd);
    assert_eq!(
        end.attributes["minecraft:visual/sky_light_factor"].value,
        AttributeValue::Float(0.0)
    );
}

fn one_value(id: &str, value: AttributeValue, modifier: Modifier) -> AttributeMap {
    BTreeMap::from([(id.to_owned(), AttributeEntry { value, modifier })])
}

#[test]
fn wgen_dimension_001_resolves_layers_in_order_and_skips_positional_dimension_queries() {
    let id = "minecraft:gameplay/sky_light_level";
    let declaration = declaration_by_id(id).unwrap();
    let layers = EnvironmentLayers::construct(
        one_value(id, AttributeValue::Float(10.0), Modifier::Override),
        Some(one_value(
            id,
            AttributeValue::Float(0.5),
            Modifier::Multiply,
        )),
        [one_value(id, AttributeValue::Float(2.0), Modifier::Add)],
        Some(one_value(id, AttributeValue::Float(99.0), Modifier::Add)),
        true,
    );
    assert_eq!(
        layers.resolve(&declaration, true),
        AttributeValue::Float(15.0)
    );
    assert_eq!(
        layers.dimension_value(&declaration),
        AttributeValue::Float(12.0)
    );
    assert_eq!(
        layers
            .layers()
            .iter()
            .map(|layer| layer.kind)
            .collect::<Vec<_>>(),
        [
            LayerKind::Dimension,
            LayerKind::Biome,
            LayerKind::Timeline,
            LayerKind::Weather
        ]
    );

    let no_weather = EnvironmentLayers::construct(
        AttributeMap::new(),
        None,
        [],
        Some(AttributeMap::new()),
        false,
    );
    assert_eq!(no_weather.layers().len(), 1);
    assert_eq!(
        no_weather.resolve(&declaration, true),
        AttributeValue::Float(15.0)
    );
    assert!(
        !Sanitizer::FloatRange {
            minimum: 0.0,
            maximum: 1.0
        }
        .accepts(&AttributeValue::Float(2.0))
    );
}

#[test]
fn wgen_dimension_001_samples_block_centers_gaussian_cells_and_render_probes() {
    assert_eq!(block_center([-1, 2, 3]), [-0.5, 2.5, 3.5]);
    let cells = gaussian_camera_cells([2.0, 2.0, 2.0]);
    assert_eq!(cells.len(), 216);
    assert!(cells.iter().all(|cell| cell.weight >= 0.0));
    assert!((cells.iter().map(|cell| cell.weight).sum::<f64>() - 4096.0).abs() < 0.0001);

    let declaration = declaration_by_id("minecraft:visual/star_brightness").unwrap();
    let blended = weighted_spatial_sample(
        &declaration,
        [
            (AttributeValue::Float(0.0), 1.0),
            (AttributeValue::Float(1.0), 3.0),
        ],
    );
    assert_eq!(blended, Some(AttributeValue::Float(0.75)));
    let mut probe = AttributeProbe::default();
    probe.update(AttributeValue::Float(0.0));
    probe.update(AttributeValue::Float(1.0));
    assert_eq!(
        probe.render(&declaration, 0.25),
        Some(AttributeValue::Float(0.25))
    );

    let moon = declaration_by_id("minecraft:visual/moon_phase").unwrap();
    let mut discrete = AttributeProbe::default();
    discrete.update(AttributeValue::Identifier("full_moon".to_owned()));
    discrete.update(AttributeValue::Identifier("new_moon".to_owned()));
    assert_eq!(
        discrete.render(&moon, 0.49),
        Some(AttributeValue::Identifier("full_moon".to_owned()))
    );
}

#[test]
fn wgen_dimension_001_invalidates_environment_cache_on_every_tick_and_clock_change() {
    let mut cache = EnvironmentCache::default();
    cache.invalidate_for_level_tick();
    cache.invalidate_for_level_tick();
    cache.invalidate_for_clock_change();
    assert_eq!(cache.generation(), 3);
}

#[test]
fn wgen_dimension_001_locks_timeline_membership_markers_and_tracks() {
    let timelines = locked_timelines();
    assert_eq!(timelines.len(), 4);
    assert!(
        timelines
            .values()
            .all(|timeline| timeline.clock == "minecraft:overworld")
    );
    let day = locked_day_timeline();
    assert_eq!(day.period_ticks, Some(24_000));
    assert_eq!(day.markers.len(), 6);
    assert_eq!(day.marker("minecraft:day"), Some(1_000));
    assert_eq!(day.marker("minecraft:noon"), Some(6_000));
    assert_eq!(day.marker("minecraft:night"), Some(13_000));
    assert_eq!(day.marker("minecraft:midnight"), Some(18_000));
    assert_eq!(day.marker("minecraft:roll_village_siege"), Some(18_000));
    assert_eq!(day.marker("minecraft:wake_up_from_sleep"), Some(0));
    assert_eq!(day.tracks.len(), 18);
    assert!(day.network_tracks().len() < day.tracks.len());
    assert_eq!(locked_moon_timeline().period_ticks, Some(192_000));
    assert_eq!(locked_moon_timeline().tracks.len(), 2);
    assert_eq!(locked_early_game_timeline().period_ticks, None);
    assert_eq!(locked_villager_schedule_timeline().tracks.len(), 2);
}

#[test]
fn wgen_dimension_001_validates_and_samples_timeline_boundaries() {
    let frames = |ticks: &[i64]| {
        ticks
            .iter()
            .map(|ticks| Keyframe {
                ticks: *ticks,
                value: AttributeValue::Float(*ticks as f32),
            })
            .collect()
    };
    assert!(matches!(
        Timeline::new("minecraft:overworld", Some(0), BTreeMap::new(), Vec::new()),
        Err(TimelineError::InvalidPeriod)
    ));
    assert!(matches!(
        Timeline::new(
            "minecraft:overworld",
            Some(10),
            BTreeMap::from([(
                "bad".to_owned(),
                TimeMarker {
                    ticks: 10,
                    show_in_commands: false
                }
            )]),
            Vec::new()
        ),
        Err(TimelineError::MarkerOutsidePeriod { .. })
    ));
    assert!(
        AttributeTrack::new(
            "minecraft:visual/star_brightness",
            Modifier::Override,
            Easing::Linear,
            frames(&[1, 1, 1]),
            Some(10)
        )
        .is_err()
    );
    assert!(
        AttributeTrack::new(
            "minecraft:visual/star_brightness",
            Modifier::Override,
            Easing::Linear,
            frames(&[0, 1, 1, 1]),
            Some(10)
        )
        .is_ok()
    );
    assert!(matches!(
        AttributeTrack::new(
            "minecraft:visual/star_brightness",
            Modifier::Override,
            Easing::Linear,
            frames(&[0, 11]),
            Some(10)
        ),
        Err(TimelineError::TrackOutsidePeriod { .. })
    ));

    let moon = locked_moon_timeline();
    assert_eq!(
        moon.tracks[1].sample(0, moon.period_ticks),
        AttributeValue::Identifier("full_moon".to_owned())
    );
    assert_eq!(
        moon.tracks[1].sample(-24_000, moon.period_ticks),
        AttributeValue::Identifier("waxing_gibbous".to_owned())
    );
    assert_eq!(
        moon.tracks[1].sample(192_000, moon.period_ticks),
        AttributeValue::Identifier("full_moon".to_owned())
    );
    let early = locked_early_game_timeline();
    assert_eq!(
        early.sample(0)["minecraft:gameplay/can_pillager_patrol_spawn"].value,
        AttributeValue::Boolean(false)
    );
    assert_eq!(
        early.sample(120_000)["minecraft:gameplay/can_pillager_patrol_spawn"].value,
        AttributeValue::Boolean(true)
    );
}

#[test]
fn wgen_dimension_001_ticks_pauses_freezes_and_serializes_named_clocks() {
    let mut negative = Clock::from_saved(SavedClock {
        total_ticks: 5,
        partial_tick: 0.25,
        rate: -0.5,
        paused: false,
    });
    negative.tick();
    assert_eq!(negative.total_ticks(), 4);
    assert_eq!(negative.partial_tick(), 0.75);
    assert_eq!(
        negative.saved(),
        SavedClock {
            total_ticks: 4,
            partial_tick: 0.75,
            rate: -0.5,
            paused: false
        }
    );

    let mut clocks = ClockManager::locked();
    clocks.tick(false);
    assert_eq!(
        clocks
            .explicit_clock("minecraft:overworld")
            .unwrap()
            .total_ticks(),
        0
    );
    clocks.tick(true);
    assert_eq!(
        clocks
            .explicit_clock("minecraft:overworld")
            .unwrap()
            .total_ticks(),
        1
    );
    clocks
        .mutate_explicit("minecraft:overworld", |clock| {
            clock.set_rate(2.5);
            clock.set_paused(true);
        })
        .unwrap();
    clocks.tick(true);
    assert_eq!(
        clocks
            .explicit_clock("minecraft:overworld")
            .unwrap()
            .total_ticks(),
        1
    );
    assert_eq!(clocks.network_state(true)["minecraft:overworld"].rate, 0.0);
    clocks
        .mutate_explicit("minecraft:overworld", |clock| clock.set_paused(false))
        .unwrap();
    assert_eq!(clocks.network_state(false)["minecraft:overworld"].rate, 0.0);
    assert_eq!(clocks.mutation_generation(), 2);
}

#[test]
fn wgen_dimension_001_handles_optional_default_time_sleep_and_siege() {
    let mut clocks = ClockManager::locked();
    let overworld = DimensionType::locked(LockedDimension::Overworld);
    let nether = DimensionType::locked(LockedDimension::TheNether);
    let day = locked_day_timeline();
    assert_eq!(clocks.default_time(&nether), 0);
    assert!(matches!(
        clocks.mutate_default(&nether, |_| {}),
        Err(ClockError::MissingDefaultClock)
    ));
    clocks
        .mutate_explicit("minecraft:overworld", |clock| clock.set_total_ticks(18_000))
        .unwrap();
    assert!(clocks.should_roll_village_siege(&overworld, Some(&day)));
    assert!(!clocks.should_roll_village_siege(&nether, Some(&day)));

    let frozen = clocks
        .complete_sleep(&overworld, Some(&day), true, false, true, true)
        .unwrap();
    assert!(!frozen.clock_advanced && frozen.players_woken && frozen.weather_reset);
    let completed = clocks
        .complete_sleep(&overworld, Some(&day), true, true, false, true)
        .unwrap();
    assert!(completed.clock_advanced && completed.players_woken && !completed.weather_reset);
    assert_eq!(clocks.default_time(&overworld), 24_000);
}

#[test]
fn wgen_dimension_001_preserves_monster_light_thresholds_and_draw_counts() {
    let overworld = DimensionType::locked(LockedDimension::Overworld);
    let bright_sky = monster_dark_enough(
        &overworld,
        LightSample {
            sky_light: 1,
            block_light: 0,
            local_raw_light: 0,
            thunder_local_raw_light: 0,
        },
        false,
        false,
        |_| 0,
    );
    assert!(!bright_sky.allowed && bright_sky.random_draws == 1);
    let block_abort = monster_dark_enough(
        &overworld,
        LightSample {
            sky_light: 0,
            block_light: 1,
            local_raw_light: 0,
            thunder_local_raw_light: 0,
        },
        false,
        false,
        |_| 31,
    );
    assert!(!block_abort.allowed && block_abort.random_draws == 1);
    let mut draws = vec![31, 7].into_iter();
    let allowed = monster_dark_enough(
        &overworld,
        LightSample {
            sky_light: 0,
            block_light: 0,
            local_raw_light: 7,
            thunder_local_raw_light: 7,
        },
        false,
        false,
        |_| draws.next().unwrap(),
    );
    assert!(allowed.allowed);
    assert_eq!(
        (allowed.random_draws, allowed.sampled_spawn_limit),
        (2, Some(7))
    );

    let nether = DimensionType::locked(LockedDimension::TheNether);
    let constant = monster_dark_enough(
        &nether,
        LightSample {
            sky_light: 0,
            block_light: 15,
            local_raw_light: 8,
            thunder_local_raw_light: 7,
        },
        true,
        true,
        |_| 31,
    );
    assert!(constant.allowed);
    assert_eq!(
        (constant.random_draws, constant.sampled_spawn_limit),
        (1, Some(7))
    );
}

#[test]
fn wgen_dimension_001_builds_bounded_initial_spawn_permutations() {
    let suggestion = SpawnCandidate { x: 10, z: -20 };
    let mut draws = 0;
    let plan = initial_spawn_plan(suggestion, 10, 3.9, false, |bound| {
        draws += 1;
        assert_eq!(bound, 49);
        4
    });
    assert_eq!(plan.radius, 3);
    assert_eq!(plan.candidates.len(), 49);
    assert_eq!(
        plan.candidates
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        49
    );
    assert_eq!(draws, 1);
    assert_eq!((plan.ticket_kind, plan.ticket_radius), ("spawn_search", 0));
    assert_eq!(plan.fallback, suggestion);

    let forced = initial_spawn_plan(suggestion, 0, 1.0, false, |_| 0);
    assert_eq!((forced.radius, forced.candidates.len()), (1, 9));
    let capped = initial_spawn_plan(suggestion, 100, 100.0, false, |_| 0);
    assert_eq!(capped.candidates.len(), 1_024);
    let adventure = initial_spawn_plan(suggestion, 10, 10.0, true, |_| panic!("no draw"));
    assert!(adventure.candidates.is_empty());
}

#[test]
fn wgen_dimension_001_applies_ceiling_owned_spawn_and_map_changes() {
    let overworld = DimensionType::locked(LockedDimension::Overworld);
    let caves = DimensionType::locked(LockedDimension::OverworldCaves);
    assert_eq!(
        initial_spawn_column_height(&overworld),
        SpawnColumnHeight::MotionBlocking
    );
    assert_eq!(
        initial_spawn_column_height(&caves),
        SpawnColumnHeight::GeneratorSpawnHeight
    );
    assert!(!natural_spawn_requires_air_descent(&overworld));
    assert!(natural_spawn_requires_air_descent(&caves));
    assert_eq!(map_sample_radius(&overworld, 9), 9);
    assert_eq!(map_sample_radius(&caves, 9), 4);
    assert!(
        SpawnSurfaceChecks {
            at_or_above_min_y: true,
            valid_surface_stack: true,
            full_support: true,
            liquid_free: true,
            collision_free: true
        }
        .accepted()
    );
    assert!(
        !SpawnSurfaceChecks {
            liquid_free: false,
            ..SpawnSurfaceChecks {
                at_or_above_min_y: true,
                valid_surface_stack: true,
                full_support: true,
                liquid_free: true,
                collision_free: true
            }
        }
        .accepted()
    );
}

#[test]
fn wgen_dimension_001_orders_bed_and_anchor_position_local_actions() {
    let exploding = interact_with_bed(&BedRule::exploding(), true);
    assert!(exploding.remove_both_halves);
    assert_eq!(exploding.explosion.unwrap().power, 5.0);
    assert!(!exploding.spawn_recorded && !exploding.starts_sleeping);

    let overworld = BedRule::overworld();
    let denied = interact_with_bed(&overworld, false);
    assert!(denied.spawn_recorded);
    assert_eq!(denied.denial, Some(SleepDenial::NotDark));
    assert!(!denied.starts_sleeping);
    assert!(retained_bed_respawn_allowed(&overworld));
    assert!(!sleeping_player_remains_asleep(&overworld, false));

    let independent = BedRule {
        can_sleep: SleepRule::Always,
        can_set_spawn: SpawnRule::Never,
        explodes: false,
        error_message: None,
    };
    let result = interact_with_bed(&independent, false);
    assert!(!result.spawn_recorded && result.starts_sleeping);
    assert_eq!(
        interact_with_respawn_anchor(true, true),
        AnchorInteraction::SetSpawn
    );
    assert!(
        matches!(interact_with_respawn_anchor(false, true), AnchorInteraction::Explode(value) if value.power == 5.0)
    );
    assert_eq!(
        interact_with_respawn_anchor(false, false),
        AnchorInteraction::Uncharged
    );
}
