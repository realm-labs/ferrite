use ferrite_foundation::coordinate::BlockPos;
use ferrite_gameplay::redstone::explosion::block::{
    BLOCK_EFFECT_ORDER, BlockEffectStage, BlockInteraction, DROP_COLLECTOR_MERGE_CAP, DropStack,
    ExplosionSourceKind, LARGE_EXPLOSION_RADIUS, ShuffleError, StackCollector, add_or_append_stack,
    can_trigger_blocks, is_small_explosion, should_affect_blocklike_entities, shuffle_in_place,
};
use ferrite_gameplay::redstone::explosion::entity::{
    ENTITY_EFFECT_ORDER, ENTITY_PHASE_MIN_RADIUS, EntityEffectInput, EntityEffectStage,
    PlayerState, PostPushRouting, calculate_exposure, default_damage, entity_query_bounds,
    plan_entity_effect,
};
use ferrite_gameplay::redstone::explosion::fire::{
    FIRE_RANDOM_BOUND, FireCandidate, FirePlanError, plan_fire,
};
use ferrite_gameplay::redstone::explosion::math::{Aabb, Vec3};
use ferrite_gameplay::redstone::explosion::ray::{
    DIRECTION_RAY_COUNT, RAY_STEP, RAY_STEP_DECAY, RayCell, RaySamplingError,
    calculate_affected_positions,
};
use ferrite_gameplay::redstone::explosion::transaction::{
    ExplosionStage, explosion_order, explosion_result,
};

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos::new(x, y, z)
}

fn stack(key: u64, count: i32) -> DropStack {
    DropStack {
        item_and_components: key,
        count,
        max_stack_size: 64,
    }
}

fn entity_input() -> EntityEffectInput {
    EntityEffectInput {
        position: Vec3::ZERO,
        effect_origin: Vec3::new(1.0, 0.0, 0.0),
        ignore_explosion: false,
        should_damage: true,
        knockback_multiplier: 1.0,
        exposure: 1.0,
        living_knockback_resistance: None,
        redirectable_projectile: false,
        player: None,
    }
}

#[test]
fn ray_sampling_always_consumes_exactly_the_boundary_cube_draws() {
    let mut inspected = 0;
    let result = calculate_affected_positions(
        Vec3::new(0.5, 0.5, 0.5),
        0.0,
        vec![0.25; DIRECTION_RAY_COUNT],
        |_| {
            inspected += 1;
            RayCell::air()
        },
        |_, _| true,
    )
    .unwrap();
    assert_eq!(result.random_float_draws, 1_352);
    assert_eq!(result.examined_cells, 0);
    assert_eq!(inspected, 0);
    assert!(result.positions.is_empty());
    assert_eq!(
        calculate_affected_positions(
            Vec3::ZERO,
            0.0,
            vec![0.0; DIRECTION_RAY_COUNT - 1],
            |_| RayCell::air(),
            |_, _| true,
        ),
        Err(RaySamplingError::MissingRandomFloat)
    );
}

#[test]
fn rays_start_at_the_exact_center_deduplicate_and_use_source_float_constants() {
    assert_eq!(RAY_STEP.to_bits(), 0.3_f32.to_bits());
    assert_eq!(RAY_STEP_DECAY.to_bits(), 0.22500001_f32.to_bits());
    let center = Vec3::new(-0.25, 4.75, 8.125);
    let result = calculate_affected_positions(
        center,
        0.2,
        vec![0.0; DIRECTION_RAY_COUNT],
        |position| {
            assert_eq!(position, pos(-1, 4, 8));
            RayCell::air()
        },
        |_, _| true,
    )
    .unwrap();
    assert_eq!(result.examined_cells, DIRECTION_RAY_COUNT);
    assert_eq!(result.positions, [pos(-1, 4, 8)].into_iter().collect());
}

#[test]
fn resistance_and_world_bounds_abort_each_ray_before_admission() {
    let mut admitted_powers = Vec::new();
    let resisted = calculate_affected_positions(
        Vec3::ZERO,
        1.0,
        vec![0.0; DIRECTION_RAY_COUNT],
        |_| RayCell {
            in_world_bounds: true,
            resistance: Some(1.0),
        },
        |_, power| {
            admitted_powers.push(power);
            false
        },
    )
    .unwrap();
    assert!(resisted.positions.is_empty());
    assert_eq!(resisted.examined_cells, DIRECTION_RAY_COUNT * 2);
    assert_eq!(admitted_powers.len(), DIRECTION_RAY_COUNT);
    assert_eq!(
        admitted_powers[0].to_bits(),
        (1.0_f32 * (0.7_f32 + 0.0_f32 * 0.6_f32) - (1.0_f32 + 0.3_f32) * 0.3_f32).to_bits()
    );

    let bounded = calculate_affected_positions(
        Vec3::ZERO,
        10.0,
        vec![0.0; DIRECTION_RAY_COUNT],
        |_| RayCell::OUT_OF_BOUNDS,
        |_, _| true,
    )
    .unwrap();
    assert!(bounded.positions.is_empty());
    assert_eq!(bounded.examined_cells, DIRECTION_RAY_COUNT);
}

#[test]
fn entity_query_uses_radius_gate_and_integer_floor_expansion() {
    assert_eq!(entity_query_bounds(Vec3::ZERO, 0.0), None);
    let bounds =
        entity_query_bounds(Vec3::new(0.25, -0.25, 8.75), ENTITY_PHASE_MIN_RADIUS).unwrap();
    assert_eq!(
        (
            bounds.min_x,
            bounds.min_y,
            bounds.min_z,
            bounds.max_x,
            bounds.max_y,
            bounds.max_z,
        ),
        (-1, -2, 7, 1, 0, 9)
    );
    assert!(bounds.excludes_direct_source);
}

#[test]
fn vector_normalization_uses_component_division_and_the_locked_zero_threshold() {
    let vector = Vec3::new(1.0, 2.0, 3.0);
    let length = 14.0_f64.sqrt();
    let normalized = vector.normalize();
    assert_eq!(normalized.x.to_bits(), (1.0 / length).to_bits());
    assert_eq!(normalized.y.to_bits(), (2.0 / length).to_bits());
    assert_eq!(normalized.z.to_bits(), (3.0 / length).to_bits());
    assert_eq!(Vec3::new(0.999e-5, 0.0, 0.0).normalize(), Vec3::ZERO);
    assert_ne!(Vec3::new(1.0e-5, 0.0, 0.0).normalize(), Vec3::ZERO);
}

#[test]
fn exposure_uses_source_grid_offsets_and_collider_misses_only() {
    let bounds = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
    let mut first = None;
    let trace = calculate_exposure(Vec3::new(9.0, 9.0, 9.0), bounds, |from, center| {
        first.get_or_insert((from, center));
        from.x <= 0.5
    });
    assert_eq!(trace.samples, 64);
    assert_eq!(trace.misses, 32);
    assert_eq!(trace.seen_percent(), 0.5);
    assert_eq!(first, Some((Vec3::ZERO, Vec3::new(9.0, 9.0, 9.0))));
}

#[test]
fn invalid_exposure_extents_return_zero_without_clipping() {
    let mut clips = 0;
    let trace = calculate_exposure(
        Vec3::ZERO,
        Aabb::new(Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO),
        |_, _| {
            clips += 1;
            true
        },
    );
    assert_eq!(clips, 0);
    assert_eq!(trace.samples, 0);
    assert_eq!(trace.seen_percent(), 0.0);
}

#[test]
fn entity_effects_damage_then_push_route_and_notify() {
    let plan = plan_entity_effect(Vec3::ZERO, 2.0, entity_input()).unwrap();
    assert_eq!(plan.normalized_distance, 0.0);
    assert!(plan.exposure_was_required);
    assert_eq!(plan.damage, Some(29.0));
    assert_eq!(plan.knockback, Vec3::new(1.0, 0.0, 0.0));
    assert!(plan.push_even_if_zero);
    assert_eq!(plan.routing, PostPushRouting::None);
    assert!(plan.call_on_explosion_hit);
    assert_eq!(
        plan.order,
        [
            EntityEffectStage::Damage,
            EntityEffectStage::Push,
            EntityEffectStage::RedirectOrRecordPlayer,
            EntityEffectStage::OnExplosionHit,
        ]
    );
    assert_eq!(plan.order, ENTITY_EFFECT_ORDER);
}

#[test]
fn damage_and_knockback_preserve_edge_and_skip_branches() {
    assert_eq!(default_damage(1.0, 1.0, 4.0), 1.0);
    let mut disabled = entity_input();
    disabled.should_damage = false;
    disabled.knockback_multiplier = 0.0;
    disabled.exposure = 1.0;
    let plan = plan_entity_effect(Vec3::ZERO, 2.0, disabled).unwrap();
    assert!(!plan.exposure_was_required);
    assert_eq!(plan.exposure, 0.0);
    assert_eq!(plan.damage, None);
    assert_eq!(plan.knockback, Vec3::ZERO);
    assert!(plan.call_on_explosion_hit);

    let mut ignored = entity_input();
    ignored.ignore_explosion = true;
    assert_eq!(plan_entity_effect(Vec3::ZERO, 2.0, ignored), None);
    let mut beyond = entity_input();
    beyond.position = Vec3::new(4.01, 0.0, 0.0);
    assert_eq!(plan_entity_effect(Vec3::ZERO, 2.0, beyond), None);
}

#[test]
fn living_resistance_and_zero_length_normalization_are_source_exact() {
    let mut input = entity_input();
    input.effect_origin = Vec3::new(1.0e-6, 0.0, 0.0);
    input.living_knockback_resistance = Some(0.25);
    let plan = plan_entity_effect(Vec3::ZERO, 2.0, input).unwrap();
    assert_eq!(plan.knockback, Vec3::ZERO);

    input.effect_origin = Vec3::new(0.0, 2.0, 0.0);
    let plan = plan_entity_effect(Vec3::ZERO, 2.0, input).unwrap();
    assert_eq!(plan.knockback, Vec3::new(0.0, 0.75, 0.0));
}

#[test]
fn projectile_routing_precedes_player_velocity_recording() {
    let mut input = entity_input();
    input.player = Some(PlayerState {
        spectator: false,
        creative: false,
        flying: false,
    });
    assert_eq!(
        plan_entity_effect(Vec3::ZERO, 2.0, input).unwrap().routing,
        PostPushRouting::RecordPlayerVelocity
    );
    input.redirectable_projectile = true;
    assert_eq!(
        plan_entity_effect(Vec3::ZERO, 2.0, input).unwrap().routing,
        PostPushRouting::RedirectProjectileToDamageSource
    );
    input.redirectable_projectile = false;
    input.player = Some(PlayerState {
        spectator: false,
        creative: true,
        flying: true,
    });
    assert_eq!(
        plan_entity_effect(Vec3::ZERO, 2.0, input).unwrap().routing,
        PostPushRouting::None
    );
}

#[test]
fn interaction_gates_distinguish_trigger_wind_charge_and_smallness() {
    for interaction in [
        BlockInteraction::Keep,
        BlockInteraction::Destroy,
        BlockInteraction::DestroyWithDecay,
    ] {
        assert!(!can_trigger_blocks(
            interaction,
            ExplosionSourceKind::Other,
            true
        ));
    }
    assert!(can_trigger_blocks(
        BlockInteraction::TriggerBlock,
        ExplosionSourceKind::Other,
        false
    ));
    assert!(!can_trigger_blocks(
        BlockInteraction::TriggerBlock,
        ExplosionSourceKind::BreezeWindCharge,
        false
    ));
    assert!(can_trigger_blocks(
        BlockInteraction::TriggerBlock,
        ExplosionSourceKind::BreezeWindCharge,
        true
    ));

    assert!(!should_affect_blocklike_entities(
        BlockInteraction::Destroy,
        ExplosionSourceKind::WindCharge,
        true
    ));
    assert!(should_affect_blocklike_entities(
        BlockInteraction::Destroy,
        ExplosionSourceKind::Other,
        false
    ));
    assert!(!should_affect_blocklike_entities(
        BlockInteraction::TriggerBlock,
        ExplosionSourceKind::Other,
        false
    ));
    assert!(is_small_explosion(
        LARGE_EXPLOSION_RADIUS,
        BlockInteraction::Keep
    ));
    assert!(!is_small_explosion(
        LARGE_EXPLOSION_RADIUS,
        BlockInteraction::Destroy
    ));
}

#[test]
fn block_shuffle_is_reverse_fisher_yates_and_validates_before_mutation() {
    let mut values = vec![0, 1, 2, 3];
    assert_eq!(shuffle_in_place(&mut values, &[1, 1, 0]), Ok(3));
    assert_eq!(values, [2, 0, 3, 1]);
    assert_eq!(
        BLOCK_EFFECT_ORDER,
        [
            BlockEffectStage::ShuffleTargets,
            BlockEffectStage::ReReadCurrentStateAndCallback,
            BlockEffectStage::PopCollectedDrops,
        ]
    );

    let mut unchanged = vec![0, 1, 2];
    assert_eq!(
        shuffle_in_place(&mut unchanged, &[3, 0]),
        Err(ShuffleError::DrawOutOfRange { draw: 3, bound: 3 })
    );
    assert_eq!(unchanged, [0, 1, 2]);
    assert_eq!(
        shuffle_in_place(&mut unchanged, &[0]),
        Err(ShuffleError::MissingBoundedDraw)
    );
}

#[test]
fn drop_collectors_merge_to_sixteen_then_keep_remainder_position() {
    assert_eq!(DROP_COLLECTOR_MERGE_CAP, 16);
    let mut collectors = Vec::new();
    add_or_append_stack(&mut collectors, stack(1, 10), pos(1, 0, 0));
    add_or_append_stack(&mut collectors, stack(1, 10), pos(2, 0, 0));
    assert_eq!(
        collectors,
        [
            StackCollector {
                position: pos(1, 0, 0),
                stack: stack(1, 16),
            },
            StackCollector {
                position: pos(2, 0, 0),
                stack: stack(1, 4),
            },
        ]
    );
    add_or_append_stack(&mut collectors, stack(1, 8), pos(3, 0, 0));
    assert_eq!(collectors[0].position, pos(1, 0, 0));
    assert_eq!(collectors[1].position, pos(2, 0, 0));
    assert_eq!(collectors[1].stack.count, 12);
}

#[test]
fn drop_merge_checks_intrinsic_stack_limit_before_the_explosion_cap() {
    let mut collectors = vec![StackCollector {
        position: pos(1, 0, 0),
        stack: stack(7, 50),
    }];
    add_or_append_stack(&mut collectors, stack(7, 20), pos(2, 0, 0));
    assert_eq!(collectors.len(), 2);
    assert_eq!(collectors[0].stack.count, 50);
    assert_eq!(collectors[1].stack.count, 20);
    add_or_append_stack(&mut collectors, stack(8, 1), pos(3, 0, 0));
    assert_eq!(collectors.len(), 3);

    let mut oversized = vec![StackCollector {
        position: pos(4, 0, 0),
        stack: stack(9, 50),
    }];
    add_or_append_stack(&mut oversized, stack(9, 10), pos(5, 0, 0));
    assert_eq!(oversized[0].stack.count, 16);
    assert_eq!(oversized[1].stack.count, 44);
    assert_eq!(oversized[1].position, pos(5, 0, 0));
}

#[test]
fn fire_consumes_every_draw_before_testing_resulting_world_state() {
    let candidates = [
        FireCandidate {
            position: pos(1, 0, 0),
            current_is_air: false,
            below_is_solid_render: false,
        },
        FireCandidate {
            position: pos(2, 0, 0),
            current_is_air: true,
            below_is_solid_render: true,
        },
        FireCandidate {
            position: pos(3, 0, 0),
            current_is_air: true,
            below_is_solid_render: false,
        },
    ];
    let plan = plan_fire(&candidates, &[0, 0, 0]).unwrap();
    assert_eq!(plan.random_draws, 3);
    assert_eq!(plan.writes, [pos(2, 0, 0)]);
    assert_eq!(FIRE_RANDOM_BOUND, 3);
    assert_eq!(
        plan_fire(&candidates, &[0, 0]),
        Err(FirePlanError::MissingBoundedDraw)
    );
    assert_eq!(
        plan_fire(&candidates, &[0, 3, 0]),
        Err(FirePlanError::DrawOutOfRange { draw: 3 })
    );
}

#[test]
fn top_level_order_skips_only_the_keep_block_phase_and_returns_sampled_count() {
    assert_eq!(
        explosion_order(BlockInteraction::Keep, true),
        [
            ExplosionStage::EmitExplodeGameEvent,
            ExplosionStage::CalculateAffectedPositions,
            ExplosionStage::HurtEntities,
            ExplosionStage::CreateFire,
            ExplosionStage::ReturnSampledUniquePositionCount,
        ]
    );
    assert_eq!(
        explosion_order(BlockInteraction::DestroyWithDecay, false),
        [
            ExplosionStage::EmitExplodeGameEvent,
            ExplosionStage::CalculateAffectedPositions,
            ExplosionStage::HurtEntities,
            ExplosionStage::PushBlockProfiler,
            ExplosionStage::ShuffleAndInvokeBlockCallbacks,
            ExplosionStage::PopCollectedDrops,
            ExplosionStage::PopBlockProfiler,
            ExplosionStage::ReturnSampledUniquePositionCount,
        ]
    );
    assert_eq!(explosion_result(41), 41);
}
