use ferrite_gameplay::entity::runtime::ent_004::arrow::{
    ARROW_TICK_ORDER, ArrowCandidate, ArrowTickStage, arrow_damage, arrow_flight_velocity,
    block_hit, in_ground_step, loyalty_return, piercing_targets, potion_arrow_loses_contents,
    resolve_entity_damage, spectral_glow_duration, trident_damage, trident_marks_dealt,
    trident_target_limit, trident_water_inertia,
};
use ferrite_gameplay::entity::runtime::ent_004::block::{
    BlockImpactInput, BlockMutation, ProjectileBlock, block_impact, may_break,
};
use ferrite_gameplay::entity::runtime::ent_004::geometry::{
    BLOCK_HIT_ORDER, BlockHitStage, Deflection, EntityHitStage, HitCandidate, SPAWN_ORDER,
    SpawnStage, Vector3, collision_margin, deflect, emits_shoot_event, entity_hit_order,
    first_entity_hit, launch, shoot_from_rotation, update_left_owner, world_border_bounce,
};
use ferrite_gameplay::entity::runtime::ent_004::hurting::{
    Difficulty, HURTING_TICK_ORDER, HurtingTickStage, WindChargeOwner, deflected_acceleration,
    dragon_fireball_cloud, hurting_motion, large_fireball_hit, small_fireball_entity_hit,
    small_fireball_places_fire, wind_charge_acceleration, wind_charge_deflectable,
    wind_charge_explodes_above_height, wind_charge_hit, wind_charge_ignores, wind_charge_inertia,
    wither_skull_hit,
};
use ferrite_gameplay::entity::runtime::ent_004::special::{
    FishingState, attached_firework_velocity, evoker_fang_step, eye_expires, eye_survives,
    eye_target, firework_damage, firework_lifetime, firework_target_admitted,
    fishing_ground_expires, fishing_loot_evaluated, fishing_owner_in_range, fishing_transition,
    llama_spit_damage, llama_spit_motion, shulker_bullet_hit, shulker_homing_leg,
};
use ferrite_gameplay::entity::runtime::ent_004::throwable::{
    ExperienceDirection, PearlHitInput, THROWABLE_TICK_ORDER, ThrowableKind, ThrowableTickStage,
    WATER_POTION_DOUSE_ORDER, WaterPotionTarget, egg_hatch, ender_pearl_hit, experience_bottle_hit,
    experience_bottle_value, gravity, lingering_cloud, pearl_keeps_chunk_ticket, pearl_vanishes,
    snowball_damage, snowball_hit, splash_duration, splash_query, splash_scale,
    throwable_may_use_portal, throwable_motion, water_potion_effect,
};

fn assert_vector(actual: Vector3, expected: Vector3) {
    assert!((actual.x - expected.x).abs() < 1.0e-12);
    assert!((actual.y - expected.y).abs() < 1.0e-12);
    assert!((actual.z - expected.z).abs() < 1.0e-12);
}

#[test]
fn launch_owner_and_spawn_order_preserve_source_boundaries() {
    let shot = launch(Vector3::new(0.0, 0.0, 2.0), 2.0, 0.0, [0.0; 6]);
    assert_vector(shot.velocity, Vector3::new(0.0, 0.0, 2.0));
    assert_eq!((shot.yaw, shot.pitch), (0.0, 0.0));

    let inherited = shoot_from_rotation(
        0.0,
        0.0,
        1.0,
        0.0,
        [0.0; 6],
        Vector3::new(0.25, 0.5, -0.25),
        false,
    );
    assert_vector(inherited.velocity, Vector3::new(0.25, 0.5, 0.75));
    let grounded = shoot_from_rotation(
        0.0,
        0.0,
        1.0,
        0.0,
        [0.0; 6],
        Vector3::new(0.25, 0.5, -0.25),
        true,
    );
    assert_eq!(grounded.velocity.y, 0.0);
    assert_eq!(
        SPAWN_ORDER,
        [
            SpawnStage::Shoot,
            SpawnStage::AddEntity,
            SpawnStage::ProjectileSpawnEnchantment,
        ]
    );
    assert!(emits_shoot_event(0));
    assert!(!emits_shoot_event(1));
    assert!(!update_left_owner(false, true));
    assert!(update_left_owner(false, false));
}

#[test]
fn ordinary_sweep_ties_owner_filter_and_margin_are_exact() {
    let candidates = [
        HitCandidate {
            entity_id: 1,
            squared_distance: 4.0,
            hittable: false,
            shares_owner_vehicle: false,
        },
        HitCandidate {
            entity_id: 2,
            squared_distance: 3.0,
            hittable: true,
            shares_owner_vehicle: true,
        },
        HitCandidate {
            entity_id: 3,
            squared_distance: 2.0,
            hittable: true,
            shares_owner_vehicle: false,
        },
        HitCandidate {
            entity_id: 4,
            squared_distance: 2.0,
            hittable: true,
            shares_owner_vehicle: false,
        },
    ];
    assert_eq!(first_entity_hit(&candidates, false, Some(2.5)), Some(3));
    assert_eq!(first_entity_hit(&candidates, true, Some(2.0)), None);
    assert_eq!(collision_margin(1), 0.0);
    assert_eq!(collision_margin(8), 0.3);
    assert_eq!(collision_margin(100), 0.3);
}

#[test]
fn deflection_border_and_hit_callback_order_are_explicit() {
    assert_eq!(deflect(Some(7), 7), Deflection::RejectedSameDeflector);
    assert_eq!(deflect(Some(7), 8), Deflection::Applied);
    assert_vector(
        world_border_bounce(Vector3::new(1.0, -2.0, 3.0), true).unwrap(),
        Vector3::new(-0.2, 0.4, -0.6),
    );
    assert_eq!(world_border_bounce(Vector3::ZERO, false), None);
    assert_eq!(
        entity_hit_order(true),
        [
            Some(EntityHitStage::RedirectTarget),
            Some(EntityHitStage::SubtypeCallback),
            Some(EntityHitStage::ProjectileLandEvent),
        ]
    );
    assert_eq!(
        BLOCK_HIT_ORDER,
        [
            BlockHitStage::BlockCallback,
            BlockHitStage::ProjectileLandEvent,
        ]
    );
}

fn block_input(block: ProjectileBlock) -> BlockImpactInput {
    BlockImpactInput {
        server_side: true,
        may_interact: true,
        impact_projectile_tag: true,
        projectiles_can_break_blocks: true,
        block,
        thrown_trident: false,
        speed: 0.0,
    }
}

#[test]
fn three_projectile_block_callbacks_keep_permission_and_speed_gates() {
    assert!(may_break(true, true));
    assert!(!may_break(true, false));
    assert_eq!(
        block_impact(block_input(ProjectileBlock::ChorusFlower)),
        Some(BlockMutation::DestroyWithDrops {
            projectile_is_breaker: true,
        })
    );
    assert_eq!(
        block_impact(block_input(ProjectileBlock::DecoratedPot)),
        Some(BlockMutation::CrackDecoratedPotThenDestroy { write_flags: 260 })
    );
    let mut dripstone = block_input(ProjectileBlock::PointedDripstone);
    dripstone.thrown_trident = true;
    dripstone.speed = 0.6;
    assert_eq!(block_impact(dripstone), None);
    dripstone.speed = f64::from_bits(0.6_f64.to_bits() + 1);
    assert!(block_impact(dripstone).is_some());
    dripstone.server_side = false;
    assert_eq!(block_impact(dripstone), None);
}

#[test]
fn throwable_motion_gravity_and_tick_order_precede_hit_resolution() {
    assert_eq!(gravity(ThrowableKind::Ordinary), 0.03);
    assert_eq!(gravity(ThrowableKind::Potion), 0.05);
    assert_eq!(gravity(ThrowableKind::ExperienceBottle), 0.07);
    let wet = throwable_motion(Vector3::new(1.0, 1.0, 1.0), ThrowableKind::Ordinary, true);
    assert_vector(wet.velocity, Vector3::new(0.8, 0.776, 0.8));
    assert_eq!(wet.bubble_particles, 4);
    assert_eq!(THROWABLE_TICK_ORDER[0], ThrowableTickStage::Gravity);
    assert_eq!(THROWABLE_TICK_ORDER[7], ThrowableTickStage::ResolveLiveHit);
}

#[test]
fn snowball_egg_and_experience_bottle_results_keep_rng_cardinality() {
    assert_eq!(snowball_damage(true), 3.0);
    assert_eq!(snowball_damage(false), 0.0);
    let blaze = snowball_hit(true);
    assert_eq!(blaze.damage, 3.0);
    assert!(blaze.broadcast_event && blaze.discard);
    let miss = egg_hatch(1, 0, &[]);
    assert!(miss.first_draw_consumed);
    assert!(!miss.second_draw_consumed);
    assert_eq!(miss.event_particles, 3);
    let stopped = egg_hatch(0, 0, &[true, true, false, true]);
    assert_eq!(
        (stopped.attempted_chickens, stopped.spawned_chickens),
        (4, 2)
    );
    assert!(stopped.preserves_variant && stopped.discard);
    assert_eq!(stopped.damage, 0);
    assert_eq!(experience_bottle_value(4, 4), 11);
    let bottle = experience_bottle_hit(4, 4, true);
    assert_eq!(bottle.experience, 11);
    assert!(bottle.broadcast_event && bottle.discard);
    assert_eq!(bottle.direction, ExperienceDirection::BlockNormal);
    assert_ne!(
        bottle.direction,
        experience_bottle_hit(4, 4, false).direction
    );
}

#[test]
fn ender_pearl_draw_precedes_live_spawn_gates_and_resets_owner() {
    let hit = ender_pearl_hit(PearlHitInput {
        owner_valid: true,
        owner_is_player: true,
        endermite_draw: 0.0,
        spawn_mobs: false,
        spawn_monsters: true,
        peaceful: false,
    });
    assert!(hit.endermite_draw_consumed);
    assert!(!hit.spawn_endermite);
    assert!(hit.teleport_owner && hit.reset_velocity_rotation && hit.reset_fall_and_impulse);
    assert_eq!((hit.portal_particles, hit.owner_damage), (32, 5));
    assert!(pearl_keeps_chunk_ticket(true));
    assert!(pearl_vanishes(true, true));
    assert!(!pearl_vanishes(true, false));
    assert!(throwable_may_use_portal(true));
}

#[test]
fn potion_distance_duration_and_cloud_boundaries_are_strict() {
    let inside = water_potion_effect(WaterPotionTarget {
        squared_distance: f64::from_bits(16.0_f64.to_bits() - 1),
        on_fire: true,
        water_sensitive: true,
        axolotl: true,
    });
    assert!(inside.extinguish && inside.damage_water_sensitive && inside.rehydrate_axolotl);
    let boundary = water_potion_effect(WaterPotionTarget {
        squared_distance: 16.0,
        on_fire: true,
        water_sensitive: true,
        axolotl: true,
    });
    assert_eq!(
        boundary,
        ferrite_gameplay::entity::runtime::ent_004::throwable::WaterPotionEffect {
            extinguish: false,
            damage_water_sensitive: false,
            rehydrate_axolotl: false,
        }
    );
    assert_eq!(WATER_POTION_DOUSE_ORDER.len(), 6);
    let query = splash_query(0.3);
    assert!(query.center_at_hit);
    assert_eq!(
        (
            query.inflate_x,
            query.inflate_y,
            query.inflate_z,
            query.target_margin,
        ),
        (4.0, 2.0, 4.0, 0.3)
    );
    assert_eq!(splash_scale(16.0), 0.0);
    assert_eq!(splash_duration(0.0, 20, 1.0), None);
    assert_eq!(splash_duration(0.0, 21, 1.0), Some(21));
    let cloud = lingering_cloud();
    assert_eq!(
        (
            cloud.radius,
            cloud.radius_on_use,
            cloud.duration,
            cloud.wait_time
        ),
        (3.0, -0.5, 600, 10)
    );
    assert!((cloud.radius_per_tick + 0.005).abs() < f32::EPSILON);
}

#[test]
fn arrow_ground_release_despawn_and_flight_values_are_exact() {
    let release = in_ground_step(
        500,
        true,
        true,
        Vector3::new(1.0, 2.0, 3.0),
        [0.5, 1.0, 0.25],
    );
    assert_vector(release.velocity, Vector3::new(0.1, 0.4, 0.15));
    assert!(release.released);
    assert!(in_ground_step(1_199, false, true, Vector3::ZERO, [0.0; 3]).discard);
    assert_vector(
        arrow_flight_velocity(Vector3::new(1.0, 1.0, 1.0), true, false),
        Vector3::new(0.6, 0.55, 0.6),
    );
    assert_eq!(ARROW_TICK_ORDER[0], ArrowTickStage::DetectInBlock);
    assert_eq!(ARROW_TICK_ORDER[6], ArrowTickStage::ResolveHits);
}

#[test]
fn arrow_sort_piercing_and_damage_follow_distinct_distance_rules() {
    let candidates = [
        ArrowCandidate {
            entity_id: 1,
            squared_distance_to_position: 4.0,
            admitted: true,
        },
        ArrowCandidate {
            entity_id: 2,
            squared_distance_to_position: 1.0,
            admitted: true,
        },
        ArrowCandidate {
            entity_id: 3,
            squared_distance_to_position: 1.0,
            admitted: true,
        },
    ];
    assert_eq!(piercing_targets(&candidates, 4.0, 1, &[2]), vec![3, 1]);
    assert_eq!(arrow_damage(2.1, 2.0, None), 5);
    assert_eq!(arrow_damage(2.1, 2.0, Some(3)), 8);
}

#[test]
fn arrow_entity_and_block_hits_restore_or_reset_exact_state() {
    let failed = resolve_entity_damage(false, 0, Vector3::new(0.001, 0.0, 0.0), true);
    assert!(failed.restored_fire && failed.discard && failed.drop_arrow);
    let piercing = resolve_entity_damage(true, 1, Vector3::ZERO, false);
    assert!(!piercing.discard);
    let block = block_hit(Vector3::new(2.0, 3.0, 4.0), Vector3::new(1.0, -1.0, 0.0));
    assert_vector(block.backed_up, Vector3::new(1.95, 3.05, 4.0));
    assert!(block.in_ground && block.clear_hit_sets);
    assert_eq!((block.shake_time, block.pierce_level), (7, 0));
    assert!(!block.critical);
}

#[test]
fn potion_spectral_and_trident_arrow_subtypes_keep_tick_boundaries() {
    assert!(!potion_arrow_loses_contents(599));
    assert!(potion_arrow_loses_contents(600));
    assert_eq!(spectral_glow_duration(), 200);
    assert_eq!(trident_damage(), 8.0);
    assert_eq!(trident_target_limit(), 1);
    assert!(!trident_marks_dealt(4));
    assert!(trident_marks_dealt(5));
    let loyalty = loyalty_return(Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 3.0, 4.0), 2);
    assert_vector(loyalty.velocity, Vector3::new(0.95, 0.06, 0.08));
    assert_eq!(loyalty.vertical_adjustment, 0.03);
    assert_eq!(trident_water_inertia(), 0.99);
}

#[test]
fn hurting_projectile_motion_deflection_and_owner_failure_are_ordered() {
    let motion = hurting_motion(
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 2.0, 0.0),
        false,
        true,
        false,
        true,
    );
    assert_vector(motion.velocity, Vector3::new(0.95, 0.095, 0.0));
    assert!(!motion.discard);
    assert!(hurting_motion(Vector3::ZERO, Vector3::ZERO, false, true, true, true).discard);
    assert_vector(
        deflected_acceleration(Vector3::new(2.0, 0.0, 0.0), true),
        Vector3::new(0.1, 0.0, 0.0),
    );
    assert_vector(
        deflected_acceleration(Vector3::new(2.0, 0.0, 0.0), false),
        Vector3::new(0.05, 0.0, 0.0),
    );
    assert_eq!(HURTING_TICK_ORDER[0], HurtingTickStage::Acceleration);
    assert_eq!(HURTING_TICK_ORDER[6], HurtingTickStage::Trail);
}

#[test]
fn fireball_skull_cloud_and_wind_charge_values_are_closed() {
    assert_eq!(large_fireball_hit(4).damage, 6);
    assert_eq!(large_fireball_hit(4).explosion_power, 4);
    assert!(small_fireball_entity_hit(false).restore_prior_fire_on_failed_damage);
    assert!(!small_fireball_places_fire(true, true, false));
    assert!(small_fireball_places_fire(true, false, false));
    let skull = wither_skull_hit(true, true, Difficulty::Hard);
    assert_eq!(
        (
            skull.damage,
            skull.heal_owner,
            skull.wither_ticks,
            skull.explosion_power
        ),
        (8, 5, 800, 1)
    );
    assert_eq!(wither_skull_hit(false, false, Difficulty::Normal).damage, 5);
    assert_eq!(
        wither_skull_hit(true, false, Difficulty::Easy).wither_ticks,
        0
    );
    assert_eq!(
        wither_skull_hit(true, false, Difficulty::Peaceful).wither_ticks,
        0
    );
    let cloud = dragon_fireball_cloud(3.0);
    assert_eq!(
        (cloud.duration, cloud.radius, cloud.instant_damage),
        (600, 3.0, true)
    );
    let wind = wind_charge_hit(WindChargeOwner::Player, Some(Vector3::new(0.0, 1.0, 0.0)));
    assert_eq!((wind.damage, wind.explosion_radius), (1, 1.2));
    assert_vector(wind.block_offset, Vector3::new(0.0, 0.25, 0.0));
    assert!(!wind_charge_deflectable(WindChargeOwner::Player, 4));
    assert!(wind_charge_deflectable(WindChargeOwner::Player, 5));
    assert_eq!(wind_charge_acceleration(), 0.0);
    assert_eq!(wind_charge_inertia(), 1.0);
    assert_eq!(
        wind_charge_hit(WindChargeOwner::Breeze, None).explosion_radius,
        3.0
    );
    assert!(wind_charge_ignores(true, false));
    assert!(!wind_charge_explodes_above_height(350, 320));
    assert!(wind_charge_explodes_above_height(351, 320));
}

#[test]
fn remaining_projectile_families_retain_exact_timers_ranges_and_damage() {
    assert_eq!(firework_lifetime(3, 5, 6), 51);
    assert_vector(
        attached_firework_velocity(Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)),
        Vector3::new(0.5, 0.85, 0.0),
    );
    assert_eq!(firework_damage(3), 11);
    assert!(firework_target_admitted(25.0, true));
    assert!(!firework_target_admitted(25.0, false));
    assert_vector(
        llama_spit_motion(Vector3::new(1.0, 1.0, 1.0)),
        Vector3::new(0.99, 0.9306, 0.99),
    );
    assert_eq!(llama_spit_damage(), 1);
    assert_eq!((shulker_homing_leg(0), shulker_homing_leg(4)), (10, 50));
    assert_eq!(
        (
            shulker_bullet_hit().damage,
            shulker_bullet_hit().levitation_ticks
        ),
        (4, 200)
    );
}

#[test]
fn fishing_eye_and_fang_state_machines_keep_terminal_boundaries() {
    assert_eq!(
        fishing_transition(FishingState::Flying, true, true),
        FishingState::Hooked
    );
    assert_eq!(
        fishing_transition(FishingState::Flying, false, true),
        FishingState::Bobbing
    );
    assert!(!fishing_ground_expires(1_199));
    assert!(fishing_ground_expires(1_200));
    assert!(fishing_owner_in_range(1_024.0));
    assert!(!fishing_owner_in_range(f64::from_bits(
        1_024.0_f64.to_bits() + 1
    )));
    assert!(fishing_loot_evaluated(true, false));
    assert!(!fishing_loot_evaluated(true, true));

    let target = eye_target(Vector3::ZERO, Vector3::new(24.0, 30.0, 0.0));
    assert_eq!((target.x, target.y, target.z), (12.0, 8.0, 0.0));
    assert!(!eye_expires(80));
    assert!(eye_expires(81));
    assert!(!eye_survives(0));
    assert!(eye_survives(1));

    let attack = evoker_fang_step(-7, 22);
    assert!(attack.attack);
    assert_eq!((attack.damage, attack.life), (6, 21));
    assert!(evoker_fang_step(-8, 1).discard);
}
