use behavior_runner::client::effects::audience::{
    EffectPlayer, ExceptSource, level_event_recipients, ordinary_level_event_recipients,
    particle_recipients, position_sound_recipients, sound_range, tracking_and_self_recipients,
};
use behavior_runner::client::effects::events::{
    DamageEventAction, EntityEventAction, EventDispatcher, LevelEventAction, PresentationAction,
};
use behavior_runner::client::effects::particle::{
    ParticleOptionDraws, ParticlePacket, ParticleSetting, present_particles,
};
use behavior_runner::client::effects::player_rules::{
    ClientRuleProjection, CombatKillAction, ConnectionUpdate, PlayerRule, PlayerRuleValues,
    RuleCategory, RulePlayer, RuleProjectionStep, WaypointManager, join_projection,
    project_rule_change,
};
use behavior_runner::client::effects::sound::{
    EntityBoundSound, SoundAvailability, SoundRejection, SoundRequest, entity_bound_sound,
    schedule_local_sound, start_sound,
};
use ferrite_testkit::phase9::effects::run_combat_rule_projection;

fn player(id: u64, level: u64, position: [f64; 3], block_position: [i32; 3]) -> EffectPlayer {
    EffectPlayer {
        id,
        level,
        position,
        block_position,
    }
}

fn sound_request() -> SoundRequest {
    SoundRequest {
        original_volume: 2.0,
        pitch: 0.25,
        final_category_volume: 0.5,
        category_gain: 2.0,
        resource_attenuation_distance: 16.0,
        seed: 99,
        music: false,
        permits_silent_start: false,
    }
}

fn particle_packet() -> ParticlePacket {
    ParticlePacket {
        position: [1.0, 2.0, 3.0],
        spread: [2.0, 3.0, 4.0],
        speed: 0.5,
        count: 1,
        override_limiter: false,
        always_show: false,
        type_overrides_limiter: false,
    }
}

#[test]
fn server_effect_audiences_lock_exclusion_dimension_and_strict_ranges() {
    let players = [
        player(1, 1, [0.0, 0.0, 0.0], [0, 0, 0]),
        player(2, 1, [15.999, 0.0, 0.0], [15, 0, 0]),
        player(3, 1, [16.0, 0.0, 0.0], [16, 0, 0]),
        player(4, 2, [0.0, 0.0, 0.0], [0, 0, 0]),
        player(5, 1, [63.999, 0.0, 0.0], [63, 0, 0]),
        player(6, 1, [64.0, 0.0, 0.0], [64, 0, 0]),
    ];
    assert_eq!(sound_range(0.5, None), 16.0);
    assert_eq!(sound_range(2.0, None), 32.0);
    assert_eq!(sound_range(9.0, Some(7.0)), 7.0);
    assert_eq!(
        position_sound_recipients(&players, 1, [0.0; 3], 16.0, Some(ExceptSource::Player(1)),),
        vec![2]
    );
    assert_eq!(
        position_sound_recipients(&players, 1, [0.0; 3], 16.0, Some(ExceptSource::Other),),
        vec![1, 2]
    );
    assert_eq!(
        ordinary_level_event_recipients(&players, 1, [0, 0, 0], Some(ExceptSource::Player(1)),),
        vec![2, 3, 5]
    );

    let particle_players = [
        player(1, 1, [0.0; 3], [31, 0, 0]),
        player(2, 1, [0.0; 3], [32, 0, 0]),
        player(3, 1, [0.0; 3], [511, 0, 0]),
        player(4, 1, [0.0; 3], [512, 0, 0]),
        player(5, 2, [0.0; 3], [0, 0, 0]),
    ];
    assert_eq!(
        particle_recipients(&particle_players, 1, [0.0; 3], false),
        vec![1]
    );
    assert_eq!(
        particle_recipients(&particle_players, 1, [0.0; 3], true),
        vec![1, 2, 3]
    );
    assert_eq!(
        tracking_and_self_recipients(&[2, 3], Some(1)),
        vec![2, 3, 1]
    );
    assert_eq!(tracking_and_self_recipients(&[1, 2], Some(1)), vec![1, 2]);
}

#[test]
fn global_level_events_project_near_far_and_cross_dimension_positions() {
    let players = [
        player(1, 1, [0.5, 0.5, 0.5], [0, 0, 0]),
        player(2, 1, [100.5, 0.5, 0.5], [100, 0, 0]),
        player(3, 2, [7.5, 8.5, 9.5], [7, 8, 9]),
    ];
    let projected = level_event_recipients(&players, 1, [0, 0, 0], None, true);
    assert_eq!(projected[0].position, [0, 0, 0]);
    assert_eq!(projected[1].position, [68, 0, 0]);
    assert_eq!(projected[2].position, [7, 8, 9]);
    assert!(projected.iter().all(|event| event.global));

    let fallback = level_event_recipients(&players, 1, [0, 0, 0], None, false);
    assert_eq!(fallback.len(), 1);
    assert!(!fallback[0].global);
}

#[test]
fn sound_engine_clamps_presentation_but_retains_original_volume_for_distance() {
    let started = start_sound(sound_request(), SoundAvailability::default(), 7).unwrap();
    assert_eq!(started.seed, 99);
    assert_eq!(started.pitch, 0.5);
    assert_eq!(started.gain, 1.0);
    assert_eq!(started.attenuation_distance, 32.0);
    assert_eq!(started.retained_until_tick, 27);

    let silent = SoundRequest {
        original_volume: 0.0,
        ..sound_request()
    };
    assert_eq!(
        start_sound(silent, SoundAvailability::default(), 0),
        Err(SoundRejection::Silent)
    );
    assert!(
        start_sound(
            SoundRequest {
                music: true,
                ..silent
            },
            SoundAvailability::default(),
            0,
        )
        .is_ok()
    );

    for (availability, rejection) in [
        (
            SoundAvailability {
                resources_loaded: false,
                ..SoundAvailability::default()
            },
            SoundRejection::ResourcesUnloaded,
        ),
        (
            SoundAvailability {
                allowed: false,
                ..SoundAvailability::default()
            },
            SoundRejection::Disallowed,
        ),
        (
            SoundAvailability {
                known_event: false,
                ..SoundAvailability::default()
            },
            SoundRejection::UnknownEvent,
        ),
        (
            SoundAvailability {
                intentionally_empty: true,
                ..SoundAvailability::default()
            },
            SoundRejection::IntentionallyEmpty,
        ),
        (
            SoundAvailability {
                event_has_variants: false,
                ..SoundAvailability::default()
            },
            SoundRejection::EmptyEvent,
        ),
        (
            SoundAvailability {
                channel_available: false,
                ..SoundAvailability::default()
            },
            SoundRejection::ChannelUnavailable,
        ),
    ] {
        assert_eq!(
            start_sound(sound_request(), availability, 0),
            Err(rejection)
        );
    }
}

#[test]
fn local_sound_seed_and_distance_delay_use_the_locked_threshold_and_speed() {
    assert_eq!(
        entity_bound_sound(7, true, 11),
        Some(EntityBoundSound {
            entity_id: 7,
            seed: 11,
        })
    );
    assert_eq!(entity_bound_sound(7, false, 11), None);
    assert_eq!(
        schedule_local_sound(Some(5), 9, 100.0, true),
        behavior_runner::client::effects::sound::LocalSoundSchedule {
            seed: 5,
            consumed_client_next_long: false,
            delay_ticks: 0,
        }
    );
    let delayed = schedule_local_sound(None, 9, 400.0, true);
    assert_eq!((delayed.seed, delayed.consumed_client_next_long), (9, true));
    assert_eq!(delayed.delay_ticks, 10);
    assert_eq!(schedule_local_sound(None, 9, 400.0, false).delay_ticks, 0);
}

#[test]
fn particle_count_zero_and_positive_distribution_consume_distinct_rng_shapes() {
    let zero = present_particles(
        ParticlePacket {
            count: 0,
            ..particle_packet()
        },
        ParticleSetting::All,
        0.0,
        &[],
        &[],
        None,
    );
    assert_eq!(zero.gaussian_draws, 0);
    assert_eq!(zero.attempts[0].position, [1.0, 2.0, 3.0]);
    assert_eq!(zero.attempts[0].velocity, [1.0, 1.5, 2.0]);

    let draws = [
        [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        [-1.0, -2.0, -3.0, -4.0, -5.0, -6.0],
    ];
    let positive = present_particles(
        ParticlePacket {
            count: 2,
            ..particle_packet()
        },
        ParticleSetting::All,
        0.0,
        &draws,
        &[],
        None,
    );
    assert_eq!(positive.gaussian_draws, 12);
    assert_eq!(positive.attempts[0].position, [3.0, 8.0, 15.0]);
    assert_eq!(positive.attempts[0].velocity, [2.0, 2.5, 3.0]);

    let failed = present_particles(
        ParticlePacket {
            count: 3,
            ..particle_packet()
        },
        ParticleSetting::All,
        0.0,
        &draws,
        &[],
        Some(1),
    );
    assert_eq!(failed.attempts.len(), 1);
    assert_eq!(failed.gaussian_draws, 12);
    assert_eq!(failed.logged_failures, 1);
    assert!(failed.stopped_after_failure);
}

#[test]
fn particle_options_draw_before_override_and_always_show_is_not_unconditional() {
    let minimal = ParticleOptionDraws {
        one_in_ten: 1,
        one_in_three: 1,
    };
    let suppressed = present_particles(
        ParticlePacket {
            always_show: true,
            ..particle_packet()
        },
        ParticleSetting::Minimal,
        0.0,
        &[[0.0; 6]],
        &[minimal],
        None,
    );
    assert!(!suppressed.attempts[0].created);
    assert_eq!(suppressed.option_draws, 1);

    let promoted = present_particles(
        ParticlePacket {
            always_show: true,
            ..particle_packet()
        },
        ParticleSetting::Minimal,
        1_024.0,
        &[[0.0; 6]],
        &[ParticleOptionDraws {
            one_in_ten: 0,
            one_in_three: 1,
        }],
        None,
    );
    assert!(promoted.attempts[0].created);
    assert_eq!(promoted.option_draws, 2);

    let override_result = present_particles(
        ParticlePacket {
            override_limiter: true,
            always_show: true,
            ..particle_packet()
        },
        ParticleSetting::Minimal,
        1_025.0,
        &[[0.0; 6]],
        &[minimal],
        None,
    );
    assert!(override_result.attempts[0].created);
    assert_eq!(override_result.option_draws, 1);

    let distant = present_particles(
        particle_packet(),
        ParticleSetting::All,
        1_025.0,
        &[[0.0; 6]],
        &[],
        None,
    );
    assert!(!distant.attempts[0].created);
}

#[test]
fn entity_damage_level_and_game_events_keep_their_distinct_dispatch_paths() {
    let mut dispatch = EventDispatcher::default();
    assert_eq!(
        dispatch.entity_event(7, 21, false),
        EntityEventAction::IgnoredMissingEntity
    );
    dispatch.track_entity(7);
    assert_eq!(
        dispatch.entity_event(7, 21, false),
        EntityEventAction::GuardianAttackSound
    );
    assert_eq!(
        dispatch.entity_event(7, 35, true),
        EntityEventAction::TotemActivation {
            emitter_ticks: 30,
            sound: true,
            local_activation_display: true,
        }
    );
    assert_eq!(
        dispatch.entity_event(7, 63, false),
        EntityEventAction::SnifferSound
    );
    assert_eq!(
        dispatch.entity_event(7, 9, false),
        EntityEventAction::EntityHandler(9)
    );
    assert_eq!(
        dispatch.damage_event(7),
        DamageEventAction::EntityDamageHandler(7)
    );
    dispatch.remove_entity(7);
    assert_eq!(
        dispatch.damage_event(7),
        DamageEventAction::IgnoredMissingEntity
    );
    assert_eq!(
        dispatch.level_event(2001, 4, false),
        LevelEventAction::OrdinaryHandler {
            event: 2001,
            data: 4,
        }
    );
    assert_eq!(
        dispatch.level_event(1023, 0, true),
        LevelEventAction::GlobalHandler {
            event: 1023,
            data: 0,
        }
    );
    dispatch.game_event();
    dispatch.local_call_site_effect(1);
    dispatch.local_call_site_effect(1);
    assert_eq!(
        dispatch
            .actions
            .iter()
            .rev()
            .take(3)
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            PresentationAction::LocalCallSiteEffect { call_site: 1 },
            PresentationAction::LocalCallSiteEffect { call_site: 1 },
            PresentationAction::GameEventHasNoClientPresentation,
        ]
    );
}

#[test]
fn player_rule_join_live_respawn_and_repeated_combat_projection_are_exact() {
    let defaults = PlayerRuleValues::default();
    assert_eq!(
        (
            defaults.immediate_respawn,
            defaults.locator_bar,
            defaults.reduced_debug_info
        ),
        (false, true, false)
    );
    assert_eq!(
        PlayerRule::ImmediateRespawn.category(),
        RuleCategory::Player
    );
    assert_eq!(PlayerRule::LocatorBar.category(), RuleCategory::Player);
    assert_eq!(PlayerRule::ReducedDebugInfo.category(), RuleCategory::Misc);
    let join = join_projection(PlayerRuleValues {
        immediate_respawn: true,
        locator_bar: false,
        reduced_debug_info: true,
    });
    assert!(join.reduced_debug_info);
    assert!(!join.show_death_screen);
    let mut client = ClientRuleProjection::from_join(7, true, join);
    let replacement = client.respawn_replacement();
    assert_eq!(
        (
            replacement.reduced_debug_info,
            replacement.show_death_screen
        ),
        (true, false)
    );
    client.immediate_respawn_event(0.0);
    assert!(client.show_death_screen);
    client.immediate_respawn_event(f32::NAN);
    assert!(!client.show_death_screen);
    client.reduced_debug_entity_event(8, 23);
    assert!(client.reduced_debug_info);
    client.reduced_debug_entity_event(7, 23);
    assert!(!client.reduced_debug_info);
    assert_eq!(client.combat_kill(8), CombatKillAction::Ignored);
    assert_eq!(
        client.combat_kill(7),
        CombatKillAction::PerformRespawnAndResetToggleKeys
    );
    assert_eq!(
        client.combat_kill(7),
        CombatKillAction::PerformRespawnAndResetToggleKeys
    );
    assert_eq!((client.respawn_requests, client.toggle_key_resets), (2, 2));

    let protocol = run_combat_rule_projection();
    assert_eq!(protocol.repeated_death_screens, 2);
    assert_eq!(protocol.repeated_respawn_requests, 2);
    assert_eq!(protocol.repeated_toggle_resets, 2);
    assert!(protocol.missing_local_ignored);
}

#[test]
fn live_rule_callbacks_notify_first_and_waypoint_disable_clears_then_rebuilds() {
    let players = [
        RulePlayer { id: 1, level: 10 },
        RulePlayer { id: 2, level: 20 },
    ];
    let mut managers = [
        WaypointManager::new(10, true),
        WaypointManager::new(20, true),
    ];
    let immediate =
        project_rule_change(PlayerRule::ImmediateRespawn, true, &players, &mut managers);
    assert_eq!(
        immediate[0],
        RuleProjectionStep::Notify(PlayerRule::ImmediateRespawn)
    );
    assert_eq!(
        &immediate[1..],
        &[
            RuleProjectionStep::ImmediateRespawn {
                player: 1,
                value: 1.0,
            },
            RuleProjectionStep::ImmediateRespawn {
                player: 2,
                value: 1.0,
            },
        ]
    );
    let debug = project_rule_change(PlayerRule::ReducedDebugInfo, false, &players, &mut managers);
    assert_eq!(
        debug[0],
        RuleProjectionStep::Notify(PlayerRule::ReducedDebugInfo)
    );
    assert!(
        debug[1..]
            .iter()
            .all(|step| matches!(step, RuleProjectionStep::ReducedDebugInfo { event: 23, .. }))
    );

    managers[0].track_transmitter(9, Some(90));
    let added = managers[0].add_player(1);
    assert!(added.contains(&ConnectionUpdate::Connected { representation: 90 }));
    let disabled = project_rule_change(PlayerRule::LocatorBar, false, &players, &mut managers);
    assert_eq!(
        disabled[0],
        RuleProjectionStep::Notify(PlayerRule::LocatorBar)
    );
    assert_eq!(managers[0].connection_count(), 0);
    let enabled = project_rule_change(PlayerRule::LocatorBar, true, &players, &mut managers);
    assert!(enabled.contains(&RuleProjectionStep::LocatorPlayerAdded {
        level: 10,
        player: 1,
    }));
    assert_eq!(managers[0].connection_count(), 1);
}

#[test]
fn waypoint_connections_reject_self_remove_absent_and_recheck_broken_entries() {
    let mut manager = WaypointManager::new(1, true);
    manager.track_transmitter(1, Some(10));
    manager.track_transmitter(2, None);
    let added = manager.add_player(1);
    assert!(
        added
            .iter()
            .all(|update| *update == ConnectionUpdate::Ineligible)
    );
    manager.add_player(3);
    assert_eq!(manager.connection_count(), 1);
    assert_eq!(
        manager.update_connection(3, 1, false),
        ConnectionUpdate::Retained
    );
    manager.set_representation(2, Some(20));
    assert_eq!(
        manager.update_connection(3, 2, true),
        ConnectionUpdate::Connected { representation: 20 }
    );
    manager.set_representation(1, None);
    assert_eq!(
        manager.update_connection(3, 1, true),
        ConnectionUpdate::Disconnected
    );
    assert_eq!(manager.remove_player(2).len(), 1);
    assert_eq!(manager.connection_count(), 0);

    let player_transmitter_updates = manager.add_player_transmitter(4, Some(40));
    assert!(
        player_transmitter_updates
            .iter()
            .all(|update| *update == ConnectionUpdate::Ineligible),
        "the newly added player must reject a connection to itself"
    );
    manager.add_player(5);
    assert_eq!(manager.connection_count(), 1);
}
