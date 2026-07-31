use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::portal::gateway::{
    EndGateway, GATEWAY_ATTENTION_INTERVAL, GATEWAY_COOLDOWN, GatewayEndStoneCandidate,
    GatewayFullBlock, GatewaySurfaceCandidate, SavedGateway, configured_exit, gateway_transition,
    generation_plan, may_generate_unconfigured, radial_chunk_walk, reciprocal_gateway_position,
    select_gateway_anchor,
};
use ferrite_world::generation::portal::{Rotation, Vec3};

#[test]
fn gateway_persists_age_exit_exact_but_not_transient_cooldown() {
    let saved = SavedGateway {
        age: 123,
        exit_position: Some(BlockPos::new(1, 2, 3)),
        exact_teleport: true,
    };
    let mut gateway = EndGateway::from_saved(saved);
    assert_eq!(gateway.cooldown, 0);
    gateway.cooldown = 17;
    assert_eq!(gateway.saved(), saved);
    assert!(gateway.spawning());
    gateway.age = 200;
    assert!(!gateway.spawning());
}

#[test]
fn gateway_tick_decrements_cooldown_before_attention_without_same_tick_retrigger() {
    let mut gateway = EndGateway {
        age: GATEWAY_ATTENTION_INTERVAL - 1,
        cooldown: 1,
        ..EndGateway::default()
    };
    let cooling = gateway.tick();
    assert_eq!(gateway.age, GATEWAY_ATTENTION_INTERVAL);
    assert_eq!(gateway.cooldown, 0);
    assert!(!cooling.attention_trigger && !cooling.broadcast_cooldown);

    gateway.age = GATEWAY_ATTENTION_INTERVAL * 2 - 1;
    let attention = gateway.tick();
    assert!(attention.attention_trigger && attention.broadcast_cooldown);
    assert_eq!(attention.cooldown, GATEWAY_COOLDOWN);
}

#[test]
fn gateway_contact_broadcasts_cooldown_before_failed_transition() {
    let mut gateway = EndGateway::default();
    let failed: ferrite_world::generation::portal::gateway::GatewayContact<()> =
        gateway.contact(|| None);
    assert!(failed.admitted && failed.marked_entity && failed.broadcast_cooldown);
    assert_eq!(failed.transition, None);
    assert_eq!(gateway.cooldown, 40);
    let skipped = gateway.contact(|| Some(()));
    assert!(!skipped.admitted && skipped.transition.is_none());
}

#[test]
fn configured_exit_handles_exact_surface_exclusions_fallback_and_first_height_tie() {
    let exit = BlockPos::new(10, 50, 20);
    assert_eq!(configured_exit(false, Some(exit), true, []), None);
    let result = configured_exit(
        true,
        Some(exit),
        false,
        [
            GatewaySurfaceCandidate {
                position: BlockPos::new(10, 52, 20),
                full_collision: true,
                bedrock: false,
                encounter_order: 0,
            },
            GatewaySurfaceCandidate {
                position: BlockPos::new(11, 60, 20),
                full_collision: true,
                bedrock: true,
                encounter_order: 1,
            },
            GatewaySurfaceCandidate {
                position: BlockPos::new(12, 59, 20),
                full_collision: true,
                bedrock: false,
                encounter_order: 2,
            },
            GatewaySurfaceCandidate {
                position: BlockPos::new(9, 59, 20),
                full_collision: true,
                bedrock: false,
                encounter_order: 1,
            },
        ],
    );
    assert_eq!(result, Some(BlockPos::new(9, 60, 20)));
    assert_eq!(
        configured_exit(true, Some(exit), false, []),
        Some(BlockPos::new(10, 53, 20))
    );
    assert_eq!(configured_exit(true, None, false, []), None);
    assert!(may_generate_unconfigured(true, None));
    assert!(!may_generate_unconfigured(false, None));
    assert!(!may_generate_unconfigured(true, Some(exit)));
}

#[test]
fn radial_walk_obeys_both_sixteen_step_limits_and_zero_direction() {
    let backward = radial_chunk_walk(BlockPos::new(100, 70, 0), |_| false);
    assert_eq!(backward, [768.0, 0.0]);
    let forward = radial_chunk_walk(BlockPos::new(100, 70, 0), |_| true);
    assert_eq!(forward, [1280.0, 0.0]);
    assert_eq!(
        radial_chunk_walk(BlockPos::new(0, 70, 0), |_| true),
        [0.0, 0.0]
    );
}

#[test]
fn gateway_anchor_uses_end_stone_clearance_origin_distance_and_fallback_rounding() {
    let (selected, island) = select_gateway_anchor(
        [100.2, -100.8],
        [
            GatewayEndStoneCandidate {
                position: BlockPos::new(4, 31, 0),
                end_stone: true,
                two_clear_above: true,
                encounter_order: 5,
            },
            GatewayEndStoneCandidate {
                position: BlockPos::new(-4, 31, 0),
                end_stone: true,
                two_clear_above: true,
                encounter_order: 1,
            },
            GatewayEndStoneCandidate {
                position: BlockPos::new(0, 20, 0),
                end_stone: true,
                two_clear_above: true,
                encounter_order: 0,
            },
            GatewayEndStoneCandidate {
                position: BlockPos::new(0, 30, 0),
                end_stone: false,
                two_clear_above: true,
                encounter_order: 0,
            },
        ],
    );
    assert_eq!(selected, BlockPos::new(-4, 31, 0));
    assert!(!island);
    let (fallback, island) = select_gateway_anchor([100.2, -100.8], []);
    assert_eq!(fallback, BlockPos::new(100, 75, -101));
    assert!(island);
}

#[test]
fn reciprocal_surface_retains_first_equal_height_and_generation_links_both_ways() {
    let anchor = BlockPos::new(100, 75, 100);
    let reciprocal = reciprocal_gateway_position(
        anchor,
        [
            GatewayFullBlock {
                position: BlockPos::new(101, 80, 100),
                full_collision: true,
                scan_order: 1,
            },
            GatewayFullBlock {
                position: BlockPos::new(99, 80, 100),
                full_collision: true,
                scan_order: 2,
            },
            GatewayFullBlock {
                position: BlockPos::new(100, 100, 117),
                full_collision: true,
                scan_order: 0,
            },
        ],
    );
    assert_eq!(reciprocal, BlockPos::new(101, 90, 100));
    let source = BlockPos::new(1, 2, 3);
    let plan = generation_plan(source, true, anchor, true, reciprocal);
    assert_eq!(plan.stored_exit, reciprocal);
    assert_eq!(plan.reciprocal_exit, source);
    assert!(!plan.reciprocal_exact);
    assert!(
        plan.retained_source_exact && plan.place_end_island && plan.fresh_feature_random_source
    );
}

#[test]
fn gateway_transition_stays_same_level_tickets_final_position_and_zeros_pearl_motion() {
    let exit = BlockPos::new(-2, 80, 5);
    let velocity = Vec3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let rotation = Rotation {
        yaw: 30.0,
        pitch: -10.0,
    };
    let ordinary = gateway_transition(exit, false, velocity, rotation);
    assert_eq!(
        ordinary.position,
        Vec3 {
            x: -1.5,
            y: 80.0,
            z: 5.5
        }
    );
    assert_eq!(ordinary.velocity, velocity);
    assert_eq!(ordinary.rotation, rotation);
    assert!(ordinary.same_level && ordinary.relative_motion_and_rotation);
    assert!(!ordinary.portal_sound);
    assert_eq!(ordinary.ticket.position, exit);

    let pearl = gateway_transition(exit, true, velocity, rotation);
    assert_eq!(pearl.velocity, Vec3::ZERO);
    assert_eq!(
        pearl.rotation,
        Rotation {
            yaw: 0.0,
            pitch: 0.0
        }
    );
    assert!(!pearl.relative_motion_and_rotation);
}
