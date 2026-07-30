use ferrite_gameplay::block::aquatic::{BubbleFlow, bubble_boat_launch};
use ferrite_gameplay::entity::runtime::ent_002::boat::{
    BoatContact, BoatContactInput, BoatInput, BoatStatus, BubbleDirection, DismountPose,
    FloatInput, Vector3, animal_seat_yaw, boat_contact, bubble_expiry, clamp_passenger_yaw,
    control_boat, first_boat_dismount, float_boat, may_mount_boat, underwater_step,
};
use ferrite_gameplay::entity::runtime::ent_002::damage::{
    VehicleDamageInput, damage_vehicle, decay_vehicle_damage,
};
use ferrite_gameplay::entity::runtime::ent_002::minecart::{
    MinecartEngine, RailShape, RideableCollision, cart_push_admitted, first_minecart_dismount,
    natural_slowdown, new_maximum_speed, new_substep_count, off_rail_velocity, old_rail_movement,
    old_rotation, ordinary_entity_impulse, player_start_impulse, powered_rail, project_to_rail,
    push_carts, rideable_collision, selected_engine, slope_acceleration, stops_in_opposing_v,
    unpowered_rail,
};
use ferrite_gameplay::entity::runtime::ent_002::subtypes::{
    FURNACE_FUEL_CAP, SubtypeHook, activate_rideable, command_cart_activates, fuel_furnace,
    furnace_maximum_speed, furnace_step, hopper_enabled, prime_tnt, shortened_tnt_fuse,
    subtype_hooks, tnt_collision_explodes, tnt_explosion_power,
};
use ferrite_gameplay::item::runtime::ply_005::vehicles::{
    fuel_furnace_cart, interact_ordinary_cart,
};

#[test]
fn common_vehicle_damage_keeps_strict_threshold_and_creative_branches() {
    let base = VehicleDamageInput {
        removed: false,
        invulnerable: false,
        mob_explosion: false,
        mob_griefing: true,
        amount: 4.0,
        hurt_direction: 1,
        accumulated_damage: 0.0,
        creative_attacker: false,
        source_forces_destruction: false,
        entity_drops: true,
    };
    let exact = damage_vehicle(base);
    assert!(exact.admitted);
    assert_eq!(exact.accumulated_damage, 40.0);
    assert!(!exact.destroyed);
    assert_eq!((exact.hurt_direction, exact.hurt_time), (-1, 10));

    let above = damage_vehicle(VehicleDamageInput {
        amount: 4.0001,
        ..base
    });
    assert!(above.destroyed);
    assert!(above.itemized);
    assert!(above.copies_custom_name);
    let creative = damage_vehicle(VehicleDamageInput {
        creative_attacker: true,
        ..base
    });
    assert!(creative.discarded);
    assert!(!creative.itemized);
    assert!(
        !damage_vehicle(VehicleDamageInput {
            mob_explosion: true,
            mob_griefing: false,
            ..base
        })
        .admitted
    );
    assert_eq!(
        decay_vehicle_damage(1, 0.5),
        ferrite_gameplay::entity::runtime::ent_002::damage::VehicleDamageDecay {
            hurt_time: 0,
            accumulated_damage: -0.5,
        }
    );
}

#[test]
fn boat_float_statuses_preserve_friction_buoyancy_and_air_snap() {
    let base = FloatInput {
        previous_status: BoatStatus::InWater,
        status: BoatStatus::InWater,
        velocity: Vector3::new(1.0, 0.0, -1.0),
        delta_rotation: 2.0,
        water_level: 65.0,
        y: 64.5,
        height: 1.0,
        water_above: 65.0,
        player_controller: false,
        air_to_water_collision_free: true,
    };
    let water = float_boat(base);
    assert_eq!((water.velocity.x, water.velocity.z), (0.9, -0.9));
    assert!((water.velocity.y - -0.006923076923076924).abs() < 1.0e-12);
    assert_eq!(water.delta_rotation, 1.8);

    let flowing = float_boat(FloatInput {
        status: BoatStatus::UnderFlowingWater,
        ..base
    });
    assert_eq!(flowing.velocity.y, -0.0007);
    let source = float_boat(FloatInput {
        status: BoatStatus::UnderWater,
        ..base
    });
    assert_eq!((source.velocity.x, source.velocity.z), (0.45, -0.45));
    let land = float_boat(FloatInput {
        status: BoatStatus::OnLand { friction: 0.8 },
        player_controller: true,
        ..base
    });
    assert_eq!((land.velocity.x, land.velocity.z), (0.4, -0.4));
    let snap = float_boat(FloatInput {
        previous_status: BoatStatus::InAir,
        status: BoatStatus::InWater,
        ..base
    });
    assert_eq!(snap.snapped_y, Some(64.101));
    assert_eq!(snap.velocity.y, 0.0);
}

#[test]
fn boat_input_paddles_underwater_and_bubbles_are_tick_exact() {
    let turn = control_boat(
        BoatInput {
            left: true,
            right: false,
            forward: false,
            backward: false,
        },
        0.0,
        Vector3::ZERO,
        0.0,
    );
    assert_eq!(turn.delta_rotation, -1.0);
    assert_eq!(turn.velocity.z, 0.005);
    assert!(!turn.left_paddle);
    assert!(turn.right_paddle);
    let forward = control_boat(
        BoatInput {
            left: false,
            right: false,
            forward: true,
            backward: false,
        },
        90.0,
        Vector3::ZERO,
        0.0,
    );
    assert!((forward.velocity.x + 0.04).abs() < 1.0e-12);
    assert!(forward.left_paddle && forward.right_paddle);

    assert!(!underwater_step(BoatStatus::UnderWater, 58).eject_passengers);
    assert!(underwater_step(BoatStatus::UnderWater, 59).eject_passengers);
    assert!((bubble_expiry(BubbleDirection::Down, true, 0.2).velocity_y + 0.5).abs() < 1.0e-12);
    assert_eq!(
        bubble_expiry(BubbleDirection::Up, true, 0.0).velocity_y,
        2.7
    );
    assert_eq!(bubble_boat_launch(BubbleFlow::Up, false), 0.6);
}

#[test]
fn boat_mount_contact_attachment_and_dismount_orders_are_closed() {
    assert!(may_mount_boat(false, 1, 2));
    assert!(!may_mount_boat(true, 0, 2));
    assert_eq!(
        boat_contact(BoatContactInput {
            server_side: true,
            has_player_controller: false,
            passenger_count: 1,
            entity_is_living: true,
            entity_is_passenger: false,
            entity_width_fits: true,
            entity_tag_allows_mount: true,
            vertical_boxes_overlap: true,
        }),
        BoatContact::AutoMount
    );
    assert_eq!(
        boat_contact(BoatContactInput {
            server_side: true,
            has_player_controller: true,
            passenger_count: 0,
            entity_is_living: true,
            entity_is_passenger: false,
            entity_width_fits: true,
            entity_tag_allows_mount: true,
            vertical_boxes_overlap: true,
        }),
        BoatContact::Push
    );
    assert_eq!(clamp_passenger_yaw(170.0), 105.0);
    assert_eq!(clamp_passenger_yaw(-170.0), -105.0);
    assert_eq!((animal_seat_yaw(2), animal_seat_yaw(3)), (90.0, 270.0));
    assert_eq!(
        first_boat_dismount(
            &[DismountPose::Standing, DismountPose::Crouching],
            &[vec![false, false], vec![false, true]],
        )
        .unwrap()
        .target_index,
        1
    );
}

#[test]
fn engine_selection_offrail_and_slowdown_are_feature_exact() {
    assert_eq!(selected_engine(false), MinecartEngine::Old);
    assert_eq!(selected_engine(true), MinecartEngine::Improved);
    assert_eq!(
        off_rail_velocity(Vector3::new(1.0, 0.2, -1.0), 0.4, true),
        Vector3::new(0.2, 0.1, -0.2)
    );
    assert_eq!(
        off_rail_velocity(Vector3::new(1.0, 0.2, -1.0), 0.4, false),
        Vector3::new(0.38, 0.19, -0.38)
    );
    assert_eq!(
        natural_slowdown(
            MinecartEngine::Old,
            false,
            false,
            Vector3::new(1.0, 2.0, 1.0)
        ),
        Vector3::new(0.96, 0.0, 0.96)
    );
    let improved_water = natural_slowdown(
        MinecartEngine::Improved,
        false,
        true,
        Vector3::new(1.0, 2.0, 1.0),
    );
    assert!((improved_water.x - 0.92625).abs() < 1.0e-12);
    assert_eq!(improved_water.y, 0.0);
    assert!((improved_water.z - 0.92625).abs() < 1.0e-12);
}

#[test]
fn all_ten_rail_shapes_take_their_slope_and_projection_paths() {
    assert_eq!(RailShape::ALL.len(), 10);
    assert_eq!(
        RailShape::ALL
            .into_iter()
            .filter(|shape| shape.is_ascending())
            .count(),
        4
    );
    let east = slope_acceleration(
        MinecartEngine::Old,
        RailShape::AscendingEast,
        1.0,
        false,
        Vector3::new(1.0, 0.0, 0.0),
    );
    assert_eq!(east.x, 1.0 - 0.0078125);
    let improved = slope_acceleration(
        MinecartEngine::Improved,
        RailShape::AscendingWest,
        1.0,
        false,
        Vector3::ZERO,
    );
    assert_eq!(improved.x, 0.02);
    assert_eq!(
        project_to_rail(Vector3::new(-3.0, 0.2, 0.0), 1.0, 0.0),
        Vector3::new(-2.0, 0.2, -0.0)
    );
}

#[test]
fn old_and_improved_rail_speed_power_and_v_boundaries_diverge() {
    assert_eq!(
        old_rail_movement(Vector3::new(1.0, 0.0, 0.0), true, false).x,
        0.4
    );
    assert_eq!(new_maximum_speed(8.0, false), 0.4);
    assert_eq!(new_maximum_speed(8.0, true), 0.2);
    assert_eq!(new_substep_count(0.81, 0.4), 3);
    assert!(player_start_impulse(Vector3::ZERO, Vector3::new(1.0, 0.0, 1.0)).applied);
    assert_eq!(unpowered_rail(Vector3::new(0.029, 1.0, 0.0)), Vector3::ZERO);
    assert_eq!(
        unpowered_rail(Vector3::new(0.03, 1.0, 0.0)),
        Vector3::new(0.015, 0.0, 0.0)
    );
    assert_eq!(
        powered_rail(
            MinecartEngine::Old,
            Vector3::ZERO,
            Some(Vector3::new(1.0, 0.0, 0.0))
        )
        .x,
        0.02
    );
    assert_eq!(
        powered_rail(
            MinecartEngine::Improved,
            Vector3::ZERO,
            Some(Vector3::new(1.0, 0.0, 0.0))
        )
        .x,
        0.2
    );
    assert!(stops_in_opposing_v(
        RailShape::AscendingEast,
        RailShape::AscendingWest,
        0.0049
    ));
    assert!(!stops_in_opposing_v(
        RailShape::AscendingEast,
        RailShape::AscendingWest,
        0.005
    ));
}

#[test]
fn rotations_rideable_pickup_and_cart_push_keep_exact_gates() {
    let flipped = old_rotation(0.0, 170.0, false);
    assert!(flipped.flipped);
    assert_eq!(flipped.yaw, 350.0);
    assert_eq!(
        rideable_collision(0.01, false, false, false, false, false),
        RideableCollision::AutoMount
    );
    assert_eq!(
        rideable_collision(0.0099, false, false, false, false, false),
        RideableCollision::Push
    );
    assert!(!cart_push_admitted(true, true, 0.0001, 0.79, false));
    assert!(cart_push_admitted(true, true, 0.0001, 0.0, true));

    let pushed = push_carts(
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.1, 0.0, 0.0),
        true,
        false,
    );
    assert_eq!(pushed.first.x, 0.95);
    assert_eq!(pushed.second.x, 0.9);
    assert_eq!(
        ordinary_entity_impulse(Vector3::new(0.4, 0.0, -0.4)),
        Vector3::new(0.1, 0.0, -0.1)
    );
}

#[test]
fn minecart_dismount_uses_pose_direction_then_exact_height_order() {
    let choice = first_minecart_dismount(
        &[DismountPose::Standing, DismountPose::Swimming],
        &[
            vec![vec![false, false, true], vec![true, false, false]],
            vec![vec![true, false]],
        ],
    )
    .unwrap();
    assert_eq!(choice.pose, DismountPose::Standing);
    assert_eq!(choice.direction_index, 0);
    assert_eq!(choice.height_offset, -1);
}

#[test]
fn subtype_fuel_fuse_activation_and_hooks_preserve_boundaries() {
    assert_eq!(activate_rideable(true).damage, 50);
    let fuel = fuel_furnace(FURNACE_FUEL_CAP - 3_600, true, 10.0, 5.0, 7.0, 7.0);
    assert_eq!(fuel.fuel, FURNACE_FUEL_CAP);
    assert_eq!((fuel.push_x, fuel.push_z), (3.0, -2.0));
    assert_eq!(
        fuel_furnace_cart(FURNACE_FUEL_CAP - 3_600, true).new_fuel,
        fuel.fuel
    );
    assert_eq!(
        fuel_furnace(FURNACE_FUEL_CAP, true, 0.0, 0.0, 0.0, 0.0).consumed,
        0
    );
    assert!(furnace_step(1, 1.0, 0.0, Vector3::ZERO).velocity.x > 0.0);
    assert_eq!(furnace_maximum_speed(0.4, false), 0.2);

    assert_eq!(prime_tnt(-1, true), 80);
    assert!(tnt_collision_explodes(80, 0.01));
    assert_eq!(shortened_tnt_fuse(19, 19), 38);
    assert_eq!(tnt_explosion_power(true, 4.0, 1.0, 1.0, 100.0), Some(11.5));
    assert!(tnt_explosion_power(false, 4.0, 1.0, 1.0, 100.0).is_none());
    assert!(hopper_enabled(false));
    assert!(command_cart_activates(true, 14, 10));
    assert!(!command_cart_activates(true, 13, 10));
    assert_eq!(
        subtype_hooks(true, true, true, true, true),
        [
            Some(SubtypeHook::HopperPickup),
            Some(SubtypeHook::SpawnerTick),
            Some(SubtypeHook::CommandControl),
            Some(SubtypeHook::ContainerMenu),
            Some(SubtypeHook::ContainerLoot),
            Some(SubtypeHook::ContainerDrop),
        ]
    );
}

#[test]
fn rideable_interaction_keeps_the_literal_double_start_riding_quirk() {
    let server = interact_ordinary_cart(false, true, false, true);
    assert!(server.passenger_installed);
    assert!(!server.literal_success);
    assert_eq!(server.start_riding_calls, 2);
}
