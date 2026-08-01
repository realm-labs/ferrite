use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::portal::processor::{
    ConfusionState, ContactResult, CrossKeyAdmission, PortalContactState, PortalEntityEligibility,
    PortalTickResult, PortalWaitInput, admits_destination, can_use_portal, entity_portal_cooldown,
    nether_portal_wait,
};

#[test]
fn contact_replaces_objects_updates_entry_once_and_refreshes_cooldown() {
    let mut state = PortalContactState::default();
    assert_eq!(
        state.contact(1, BlockPos::new(1, 2, 3), 300),
        ContactResult::ProcessorCreated
    );
    assert_eq!(
        state.contact(1, BlockPos::new(4, 5, 6), 300),
        ContactResult::ProcessorMarked
    );
    assert_eq!(
        state.processor.as_ref().unwrap().entry_block,
        BlockPos::new(1, 2, 3)
    );
    let _ = state.tick(true, 80);
    assert_eq!(
        state.contact(1, BlockPos::new(4, 5, 6), 300),
        ContactResult::ProcessorMarked
    );
    assert_eq!(
        state.processor.as_ref().unwrap().entry_block,
        BlockPos::new(4, 5, 6)
    );
    assert_eq!(
        state.contact(2, BlockPos::new(7, 8, 9), 300),
        ContactResult::ProcessorReplaced
    );
    assert_eq!(state.processor.as_ref().unwrap().accumulated_time, 0);

    state.cooldown = 1;
    assert_eq!(
        state.contact(3, BlockPos::new(0, 0, 0), 10),
        ContactResult::CooldownRefreshed
    );
    assert_eq!(state.cooldown, 10);
    assert_eq!(state.processor.as_ref().unwrap().portal_object, 2);
}

#[test]
fn wait_uses_preincrement_value_and_unmarked_time_decays_by_four() {
    let mut state = PortalContactState::default();
    let entry = BlockPos::new(0, 64, 0);
    let _ = state.contact(1, entry, 300);
    assert!(matches!(
        state.tick(true, 0),
        PortalTickResult::Ready { .. }
    ));

    state = PortalContactState::default();
    for tick in 0..=80 {
        let _ = state.contact(1, entry, 300);
        let result = state.tick(true, 80);
        if tick < 80 {
            assert!(
                matches!(result, PortalTickResult::Accumulating { old_time, .. } if old_time == tick)
            );
        } else {
            assert!(matches!(result, PortalTickResult::Ready { .. }));
        }
    }
    assert_eq!(state.processor.as_ref().unwrap().accumulated_time, 81);
    assert_eq!(state.tick(true, 80), PortalTickResult::Decayed(77));
    state.processor.as_mut().unwrap().accumulated_time = 3;
    assert_eq!(state.tick(true, 80), PortalTickResult::Expired);
    assert!(state.processor.is_none());
}

#[test]
fn ineligible_marked_contact_does_not_advance_and_failed_attempt_keeps_cooldown() {
    let mut state = PortalContactState::default();
    let _ = state.contact(1, BlockPos::new(0, 0, 0), 300);
    assert_eq!(state.tick(false, 0), PortalTickResult::Ineligible);
    assert_eq!(state.processor.as_ref().unwrap().accumulated_time, 0);
    let result: Option<()> = state.attempt_ready(300, || None);
    assert_eq!(result, None);
    assert_eq!(state.cooldown, 300);
    let _ = state.tick(true, 0);
    assert_eq!(state.cooldown, 299);
}

#[test]
fn player_wait_and_root_cooldown_follow_owned_gamerules() {
    assert_eq!(
        nether_portal_wait(PortalWaitInput {
            is_player: false,
            default_delay: 80,
            ..PortalWaitInput::default()
        }),
        0
    );
    assert_eq!(
        nether_portal_wait(PortalWaitInput {
            is_player: true,
            invulnerable_ability: false,
            default_delay: 80,
            creative_delay: 0,
        }),
        80
    );
    assert_eq!(
        nether_portal_wait(PortalWaitInput {
            is_player: true,
            invulnerable_ability: true,
            default_delay: 80,
            creative_delay: -5,
        }),
        0
    );
    assert_eq!(entity_portal_cooldown(true, false), 10);
    assert_eq!(entity_portal_cooldown(false, true), 10);
    assert_eq!(entity_portal_cooldown(false, false), 300);
}

fn ordinary() -> PortalEntityEligibility {
    PortalEntityEligibility {
        alive: true,
        ..PortalEntityEligibility::default()
    }
}

#[test]
fn eligibility_covers_base_living_boss_creaking_hook_and_throwable_overrides() {
    assert!(can_use_portal(ordinary()));
    for rejected in [
        PortalEntityEligibility {
            alive: false,
            ..ordinary()
        },
        PortalEntityEligibility {
            passenger: true,
            ..ordinary()
        },
        PortalEntityEligibility {
            sleeping_living: true,
            ..ordinary()
        },
        PortalEntityEligibility {
            fishing_hook: true,
            ..ordinary()
        },
        PortalEntityEligibility {
            wither: true,
            ..ordinary()
        },
        PortalEntityEligibility {
            ender_dragon: true,
            ..ordinary()
        },
        PortalEntityEligibility {
            heart_bound_creaking: true,
            ..ordinary()
        },
    ] {
        assert!(!can_use_portal(rejected));
    }
    assert!(can_use_portal(PortalEntityEligibility {
        alive: false,
        passenger: true,
        sleeping_living: true,
        wither: true,
        throwable_projectile: true,
        ..PortalEntityEligibility::default()
    }));
    assert!(can_use_portal(PortalEntityEligibility {
        passenger: true,
        passenger_permitted: true,
        ..ordinary()
    }));
}

#[test]
fn cross_key_gates_nether_gamerule_teleport_credits_and_pearl_owner() {
    assert!(!admits_destination(CrossKeyAdmission {
        destination_is_nether: true,
        allow_entering_nether_using_portals: false,
        same_key: true,
        entity_can_teleport: true,
        ..CrossKeyAdmission::default()
    }));
    assert!(admits_destination(CrossKeyAdmission {
        same_key: true,
        allow_entering_nether_using_portals: true,
        ..CrossKeyAdmission::default()
    }));
    assert!(!admits_destination(CrossKeyAdmission {
        entity_can_teleport: false,
        allow_entering_nether_using_portals: true,
        ..CrossKeyAdmission::default()
    }));
    assert!(!admits_destination(CrossKeyAdmission {
        entity_can_teleport: true,
        literal_end_to_overworld: true,
        direct_unseen_credits_player: true,
        allow_entering_nether_using_portals: true,
        ..CrossKeyAdmission::default()
    }));
    assert!(!admits_destination(CrossKeyAdmission {
        entity_can_teleport: true,
        literal_end_to_overworld: true,
        is_ender_pearl: true,
        pearl_owner_is_server_player: true,
        pearl_owner_seen_credits: false,
        allow_entering_nether_using_portals: true,
        ..CrossKeyAdmission::default()
    }));
    assert!(admits_destination(CrossKeyAdmission {
        entity_can_teleport: true,
        literal_end_to_overworld: true,
        is_ender_pearl: true,
        pearl_owner_is_server_player: true,
        pearl_owner_seen_credits: true,
        allow_entering_nether_using_portals: true,
        ..CrossKeyAdmission::default()
    }));
}

#[test]
fn confusion_closes_ui_draws_once_and_clamps_both_directions() {
    let mut state = ConfusionState::default();
    let first = state.tick(true, true, true, || 0.5);
    assert!(first.close_disallowed_screen && first.close_open_container);
    assert!(first.play_trigger_sound);
    assert_eq!(
        (first.pitch, first.volume, first.random_draws),
        (Some(1.0), Some(0.25), 1)
    );
    assert_eq!(first.intensity, 0.0125);
    let second = state.tick(true, false, false, || panic!("one trigger draw only"));
    assert!(!second.play_trigger_sound);
    assert_eq!(second.intensity, 0.025);
    let allowed_container = state.tick(true, false, true, || panic!("intensity is nonzero"));
    assert!(!allowed_container.close_disallowed_screen);
    assert!(!allowed_container.close_open_container);
    state.intensity = 1.0;
    assert_eq!(state.tick(true, false, false, || 0.0).intensity, 1.0);
    state.intensity = 0.01;
    assert_eq!(state.tick(false, false, false, || 0.0).intensity, 0.0);
    let reentry = state.tick(true, false, false, || 0.25);
    assert!(reentry.play_trigger_sound);
    assert!((reentry.pitch.unwrap() - 0.9).abs() < f32::EPSILON);
}
