use ferrite_foundation::coordinate::BlockPos;
use ferrite_gameplay::block::falling::{
    AnvilStage, DRAGON_EGG_FALL_DELAY, DRAGON_EGG_TELEPORT_FLAGS, FALLING_UNLOAD_EFFECTS,
    FallStartEffect, FallingEntity, FallingEntitySnapshot, FallingHurtEffect, FallingKind,
    FallingTickEffect, FallingUnloadEffect, GENERIC_FALL_DELAY, LandingInputs,
    SCAFFOLDING_FALL_DELAY, ScaffoldingDistance, ScaffoldingTick, SubtypeFallEffect, ambient_dust,
    anvil_impact, anvil_target_is_hurt, concrete_powder_solidifies, concrete_powder_uses_water_hit,
    degrade_anvil, fall_delay, falling_entity_hurt, falling_entity_position, falling_kind,
    landing_velocity, loaded_hurts_entities, on_broken_subtype_effects, on_land_subtype_effects,
    plan_dragon_egg_teleport, plan_fall_start, plan_landing, restore_snapshot,
    scaffolding_distance, scaffolding_survives, scaffolding_tick, should_finish_removed_tick,
    should_start_fall, start_subtype_effect,
};
use ferrite_gameplay::block::falling::{DragonEggCandidate, DragonEggEffect};
use ferrite_world::id::BlockStateId;

const STATE: BlockStateId = BlockStateId::new(41);

fn landing_inputs() -> LandingInputs {
    LandingInputs {
        moving_piston_target: false,
        cancel_drop: false,
        target_replaceable: true,
        carried_state_survives: true,
        block_below_still_free: false,
        water_contact_bypasses_below_free: false,
        placement_succeeded: true,
        drop_item: true,
        do_entity_drops: true,
        serialized_block_entity_data: false,
    }
}

fn invalid_egg_candidate() -> DragonEggCandidate {
    DragonEggCandidate {
        candidate_is_air: false,
        below_is_nonair: true,
        inside_build_height: true,
        inside_world_border: true,
    }
}

#[test]
fn falling_catalog_is_exactly_the_twenty_six_audited_blocks() {
    let generic = ["sand", "red_sand", "gravel"];
    let powders = [
        "white_concrete_powder",
        "orange_concrete_powder",
        "magenta_concrete_powder",
        "light_blue_concrete_powder",
        "yellow_concrete_powder",
        "lime_concrete_powder",
        "pink_concrete_powder",
        "gray_concrete_powder",
        "light_gray_concrete_powder",
        "cyan_concrete_powder",
        "purple_concrete_powder",
        "blue_concrete_powder",
        "brown_concrete_powder",
        "green_concrete_powder",
        "red_concrete_powder",
        "black_concrete_powder",
    ];
    let anvils = ["anvil", "chipped_anvil", "damaged_anvil"];
    let remaining = [
        "dragon_egg",
        "suspicious_sand",
        "suspicious_gravel",
        "scaffolding",
    ];
    assert_eq!(
        generic.len() + powders.len() + anvils.len() + remaining.len(),
        26
    );
    assert!(
        generic
            .into_iter()
            .all(|path| falling_kind(path) == Some(FallingKind::Generic))
    );
    assert!(
        powders
            .into_iter()
            .all(|path| falling_kind(path) == Some(FallingKind::ConcretePowder))
    );
    assert_eq!(
        anvils.map(falling_kind),
        [
            Some(FallingKind::Anvil(AnvilStage::Intact)),
            Some(FallingKind::Anvil(AnvilStage::Chipped)),
            Some(FallingKind::Anvil(AnvilStage::Damaged)),
        ]
    );
    assert_eq!(falling_kind("dragon_egg"), Some(FallingKind::DragonEgg));
    assert_eq!(
        falling_kind("suspicious_sand"),
        Some(FallingKind::Brushable)
    );
    assert_eq!(
        falling_kind("suspicious_gravel"),
        Some(FallingKind::Brushable)
    );
    assert_eq!(falling_kind("scaffolding"), Some(FallingKind::Scaffolding));
    assert_eq!(falling_kind("invented_concrete_powder"), None);
    assert_eq!(fall_delay(FallingKind::Generic), GENERIC_FALL_DELAY);
    assert_eq!(fall_delay(FallingKind::ConcretePowder), GENERIC_FALL_DELAY);
    assert_eq!(fall_delay(FallingKind::DragonEgg), DRAGON_EGG_FALL_DELAY);
    assert_eq!(fall_delay(FallingKind::Scaffolding), SCAFFOLDING_FALL_DELAY);
}

#[test]
fn fall_start_preserves_coordinates_gates_and_origin_before_admission_order() {
    let origin = BlockPos::new(-3, -64, 9);
    assert_eq!(falling_entity_position(origin), [-2.5, -64.0, 9.5]);
    assert!(should_start_fall(-64, -64, true));
    assert!(!should_start_fall(-65, -64, true));
    assert!(!should_start_fall(-64, -64, false));

    let generic = plan_fall_start(FallingKind::Generic);
    assert_eq!(
        generic,
        [
            FallStartEffect::CreateEntityAtBlockCenter,
            FallStartEffect::ClearCarriedWaterlogged,
            FallStartEffect::RecordStartPosition,
            FallStartEffect::ReplaceOriginWithFluid { flags: 3 },
            FallStartEffect::OfferEntityAdmission,
        ]
    );
    assert_eq!(
        plan_fall_start(FallingKind::Brushable),
        [
            FallStartEffect::ResetBrushableBlockEntity,
            FallStartEffect::CreateEntityAtBlockCenter,
            FallStartEffect::ClearCarriedWaterlogged,
            FallStartEffect::RecordStartPosition,
            FallStartEffect::ReplaceOriginWithFluid { flags: 3 },
            FallStartEffect::OfferEntityAdmission,
            FallStartEffect::SetCancelDrop,
        ]
    );
    assert_eq!(
        start_subtype_effect(FallingKind::Anvil(AnvilStage::Intact)),
        Some(SubtypeFallEffect::ConfigureAnvilDamage {
            amount: 2,
            maximum: 40,
        })
    );
    assert_eq!(start_subtype_effect(FallingKind::Generic), None);
}

#[test]
fn entity_motion_timeout_and_unload_are_tick_exact() {
    let mut entity = FallingEntity::new(STATE, false, 12.0);
    assert!(!entity.hurts_entities);
    assert_eq!(entity.fall_damage_amount, 0.0);
    assert_eq!(entity.fall_damage_maximum, 40);
    entity.configure_anvil_damage();
    assert!(entity.hurts_entities);
    assert_eq!(entity.fall_damage_amount, 2.0);
    assert_eq!(entity.fall_damage_maximum, 40);
    entity.velocity = [1.0, 2.0, -1.0];
    assert_eq!(
        entity.begin_tick(),
        [
            FallingTickEffect::IncrementTime,
            FallingTickEffect::ApplyGravity,
            FallingTickEffect::MoveSelf,
            FallingTickEffect::ApplyBlockEffectsAndPortals,
        ]
    );
    assert_eq!(entity.time, 1);
    assert_eq!(entity.velocity, [1.0, 1.96, -1.0]);
    assert_eq!(entity.finish_tick(), FallingTickEffect::ApplyDrag);
    assert_eq!(entity.velocity, [0.98, 1.96 * 0.98, -0.98]);
    assert_eq!(
        landing_velocity([2.0, -4.0, 6.0]),
        [2.0 * 0.7, -4.0 * -0.5, 6.0 * 0.7]
    );

    let mut air = FallingEntity::new(STATE, true, 0.0);
    assert_eq!(air.begin_tick(), [FallingTickEffect::DiscardAirState]);
    assert_eq!(air.time, 0);

    let mut boundary = FallingEntity::new(STATE, false, -64.0);
    boundary.time = 100;
    assert!(!boundary.timed_out(-64, 320));
    boundary.time = 101;
    assert!(boundary.timed_out(-64, 320));
    boundary.position_y = 320.0;
    assert!(!boundary.timed_out(-64, 320));
    boundary.position_y = 320.000_001;
    assert!(boundary.timed_out(-64, 320));
    boundary.position_y = 0.0;
    boundary.time = 600;
    assert!(!boundary.timed_out(-64, 320));
    boundary.time = 601;
    assert!(boundary.timed_out(-64, 320));
    boundary.cancel_drop = true;
    assert_eq!(
        boundary.timeout_effects(true),
        [
            FallingTickEffect::SpawnCarriedItem,
            FallingTickEffect::DiscardTimeout,
        ]
    );

    let snapshot = FallingEntitySnapshot {
        persistent_id: 7,
        entity: boundary,
    };
    assert_eq!(restore_snapshot(snapshot, true), Some(boundary));
    assert_eq!(restore_snapshot(snapshot, false), None);
    assert_eq!(
        FALLING_UNLOAD_EFFECTS,
        [
            FallingUnloadEffect::StoreEntity,
            FallingUnloadEffect::RemoveTickCallback,
        ]
    );
    assert!(should_finish_removed_tick(true, true));
    assert!(!should_finish_removed_tick(true, false));
    assert!(should_finish_removed_tick(false, false));
    assert_eq!(
        falling_entity_hurt(),
        (false, FallingHurtEffect::RecordLastHurt)
    );
}

#[test]
fn landing_matrix_locks_pause_cancel_retry_drop_and_success_order() {
    let piston = plan_landing(LandingInputs {
        moving_piston_target: true,
        ..landing_inputs()
    });
    assert!(piston.remains_active);
    assert_eq!(
        piston.effects,
        [
            FallingTickEffect::ApplyLandingVelocity,
            FallingTickEffect::DeferAboveMovingPiston,
        ]
    );

    let cancelled = plan_landing(LandingInputs {
        cancel_drop: true,
        ..landing_inputs()
    });
    assert!(!cancelled.remains_active);
    assert_eq!(
        cancelled.effects,
        [
            FallingTickEffect::ApplyLandingVelocity,
            FallingTickEffect::DiscardLanding,
            FallingTickEffect::BrokenHookWithoutItem,
        ]
    );

    let ineligible = plan_landing(LandingInputs {
        target_replaceable: false,
        ..landing_inputs()
    });
    assert_eq!(
        ineligible.effects,
        [
            FallingTickEffect::ApplyLandingVelocity,
            FallingTickEffect::DiscardLanding,
            FallingTickEffect::BrokenHookWithItem,
            FallingTickEffect::SpawnCarriedItem,
        ]
    );

    let retry = plan_landing(LandingInputs {
        placement_succeeded: false,
        do_entity_drops: false,
        ..landing_inputs()
    });
    assert!(retry.remains_active);
    assert_eq!(
        retry.effects,
        [
            FallingTickEffect::ApplyLandingVelocity,
            FallingTickEffect::CopyDestinationWaterlogged,
            FallingTickEffect::AttemptPlacement { flags: 3 },
        ]
    );

    let water_success = plan_landing(LandingInputs {
        block_below_still_free: true,
        water_contact_bypasses_below_free: true,
        serialized_block_entity_data: true,
        ..landing_inputs()
    });
    assert!(!water_success.remains_active);
    assert_eq!(
        water_success.effects,
        [
            FallingTickEffect::ApplyLandingVelocity,
            FallingTickEffect::CopyDestinationWaterlogged,
            FallingTickEffect::AttemptPlacement { flags: 3 },
            FallingTickEffect::SendTrackingBlockUpdate,
            FallingTickEffect::DiscardLanding,
            FallingTickEffect::OnLand,
            FallingTickEffect::OverlaySerializedBlockEntityData,
        ]
    );
}

#[test]
fn anvil_formula_target_gate_and_degradation_use_strict_source_boundaries() {
    assert!(!loaded_hurts_entities(false, None));
    assert!(loaded_hurts_entities(true, None));
    assert!(!loaded_hurts_entities(true, Some(false)));
    assert!(anvil_target_is_hurt(true, false, false, true));
    assert!(!anvil_target_is_hurt(false, false, false, true));
    assert!(!anvil_target_is_hurt(true, true, false, true));
    assert!(!anvil_target_is_hurt(true, false, true, true));
    assert!(!anvil_target_is_hurt(true, false, false, false));

    let below_zero = anvil_impact(0.0, 2.0, 40);
    assert_eq!(below_zero.distance_index, -1);
    assert_eq!(below_zero.damage, 0);
    assert_eq!(below_zero.degradation_threshold, None);
    let capped = anvil_impact(100.0, 2.0, 40);
    assert_eq!(capped.distance_index, 99);
    assert_eq!(capped.damage, 40);
    assert_eq!(
        capped.degradation_threshold,
        Some(0.05_f32 + 0.05_f32 * 99.0_f32)
    );
    let zero_damage = anvil_impact(1.0, 2.0, 40);
    assert_eq!(zero_damage.distance_index, 0);
    assert_eq!(zero_damage.degradation_threshold, None);

    assert_eq!(
        degrade_anvil(AnvilStage::Intact, 0.099, 0.1),
        (AnvilStage::Chipped, false)
    );
    assert_eq!(
        degrade_anvil(AnvilStage::Intact, 0.1, 0.1),
        (AnvilStage::Intact, false)
    );
    assert_eq!(
        degrade_anvil(AnvilStage::Damaged, 0.0, 0.1),
        (AnvilStage::Damaged, true)
    );
    assert_eq!(
        on_land_subtype_effects(FallingKind::Anvil(AnvilStage::Intact), false, false),
        [SubtypeFallEffect::AnvilLandEvent(1031)]
    );
    assert_eq!(
        on_broken_subtype_effects(FallingKind::Anvil(AnvilStage::Intact), false),
        [SubtypeFallEffect::AnvilBrokenEvent(1029)]
    );
    assert!(on_broken_subtype_effects(FallingKind::Anvil(AnvilStage::Intact), true).is_empty());
}

#[test]
fn concrete_and_brushable_subtypes_preserve_water_and_drop_quirks() {
    assert!(!concrete_powder_uses_water_hit(1.0, true));
    assert!(concrete_powder_uses_water_hit(1.000_001, true));
    assert!(!concrete_powder_uses_water_hit(2.0, false));
    assert!(concrete_powder_solidifies(true, false));
    assert!(concrete_powder_solidifies(false, true));
    assert!(!concrete_powder_solidifies(false, false));
    assert_eq!(
        on_land_subtype_effects(FallingKind::ConcretePowder, true, false),
        [SubtypeFallEffect::SolidifyConcrete]
    );
    assert!(on_land_subtype_effects(FallingKind::ConcretePowder, false, false).is_empty());
    assert_eq!(
        on_broken_subtype_effects(FallingKind::Brushable, false),
        [
            SubtypeFallEffect::BrushableDestroyEvent(2001),
            SubtypeFallEffect::BlockDestroyGameEvent,
        ]
    );
}

#[test]
fn scaffolding_distance_and_distance_seven_transitions_are_exact() {
    assert_eq!(
        scaffolding_distance(true, Some(6), &[0]),
        ScaffoldingDistance {
            distance: 0,
            bottom: false,
        }
    );
    assert_eq!(
        scaffolding_distance(false, Some(4), &[0]),
        ScaffoldingDistance {
            distance: 4,
            bottom: false,
        }
    );
    assert_eq!(
        scaffolding_distance(false, None, &[6, 2, 5]),
        ScaffoldingDistance {
            distance: 3,
            bottom: true,
        }
    );
    let unsupported = scaffolding_distance(false, None, &[]);
    assert_eq!(
        unsupported,
        ScaffoldingDistance {
            distance: 7,
            bottom: true,
        }
    );
    assert_eq!(
        scaffolding_tick(6, true, unsupported),
        ScaffoldingTick::DestroyWithDrops
    );
    assert_eq!(
        scaffolding_tick(7, true, unsupported),
        ScaffoldingTick::SpawnFallingEntity
    );
    assert!(!scaffolding_survives(7));
    assert!(scaffolding_survives(6));
    assert_eq!(
        scaffolding_tick(
            4,
            false,
            ScaffoldingDistance {
                distance: 3,
                bottom: true,
            },
        ),
        ScaffoldingTick::WriteSupportedState { flags: 3 }
    );
    assert_eq!(
        scaffolding_tick(
            3,
            true,
            ScaffoldingDistance {
                distance: 3,
                bottom: true,
            },
        ),
        ScaffoldingTick::NoChange
    );
}

#[test]
fn dragon_egg_attempts_rng_cardinality_and_side_effects_are_bounded() {
    let valid = DragonEggCandidate {
        candidate_is_air: true,
        below_is_nonair: true,
        inside_build_height: true,
        inside_world_border: true,
    };
    let server = plan_dragon_egg_teleport(
        false,
        &[invalid_egg_candidate(), invalid_egg_candidate(), valid],
    );
    assert_eq!(server.attempts, 3);
    assert_eq!(server.random_draws, 18);
    assert_eq!(server.accepted_candidate, Some(2));
    assert_eq!(
        server.effects,
        [
            DragonEggEffect::WriteCandidate {
                flags: DRAGON_EGG_TELEPORT_FLAGS,
            },
            DragonEggEffect::RemoveOriginWithoutDrops,
        ]
    );
    let client = plan_dragon_egg_teleport(true, &[valid]);
    assert_eq!(client.random_draws, 6);
    assert_eq!(client.effects, [DragonEggEffect::EmitPortalParticles(128)]);

    let invalid = vec![invalid_egg_candidate(); 1_001];
    let bounded = plan_dragon_egg_teleport(false, &invalid);
    assert_eq!(bounded.attempts, 1_000);
    assert_eq!(bounded.random_draws, 6_000);
    assert_eq!(bounded.accepted_candidate, None);
    assert!(bounded.effects.is_empty());
    assert_eq!(ambient_dust(0).random_draws, 1);
    assert!(ambient_dust(0).emitted);
    assert!(!ambient_dust(1).emitted);
    assert!(!ambient_dust(15).emitted);
}
