use ferrite_gameplay::player::collision::{Aabb, CollisionScene, EntityMotion};
use ferrite_gameplay::player::special_travel::{
    AbilityFlightContext, AbilitySuperclassTravel, FallFlyingContext, FallFlyingEffect, FluidKind,
    FluidTravelContext, GliderSlot, TravelMode, ability_flight_tick, apply_swimming_steering,
    descend_in_water, fall_flying_tick, fluid_travel_tick, liquid_jump, select_travel_mode,
};
use ferrite_gameplay::player::state::Vec3;
use ferrite_gameplay::player::travel::{
    TravelAttributes, TravelContext, TravelInput, TravelTimers,
};

fn player_motion() -> EntityMotion {
    let position = Vec3::new(0.0, 65.0, 0.0);
    EntityMotion::new(
        position,
        Aabb::new(Vec3::new(-0.3, 65.0, -0.3), Vec3::new(0.3, 66.8, 0.3)),
    )
}

#[test]
fn water_slowdown_efficiency_dolphins_grace_and_sprint_gravity_are_distinct() {
    assert_eq!(
        select_travel_mode(true, true, true, false, true),
        TravelMode::Water
    );
    assert_eq!(
        select_travel_mode(true, false, true, true, true),
        TravelMode::FallFlying
    );
    let mut default_water = player_motion();
    let default_result = fluid_travel_tick(
        &mut default_water,
        FluidTravelContext::default(),
        &CollisionScene::default(),
    );
    assert_eq!(default_result.horizontal_slowdown, 0.8);
    assert_eq!(default_result.acceleration, 0.02);
    assert_eq!(default_water.velocity.y, -0.005);

    let mut efficient_airborne = player_motion();
    let efficient_result = fluid_travel_tick(
        &mut efficient_airborne,
        FluidTravelContext {
            water_movement_efficiency: 1.0,
            ..FluidTravelContext::default()
        },
        &CollisionScene::default(),
    );
    assert_eq!(
        efficient_result.horizontal_slowdown,
        0.8_f32 + (0.546_000_06_f32 - 0.8_f32) * 0.5_f32
    );
    assert_eq!(
        efficient_result.acceleration,
        0.02_f32 + (0.1_f32 - 0.02_f32) * 0.5_f32
    );

    let mut sprinting_grace = player_motion();
    let grace_result = fluid_travel_tick(
        &mut sprinting_grace,
        FluidTravelContext {
            sprinting: true,
            dolphins_grace: true,
            ..FluidTravelContext::default()
        },
        &CollisionScene::default(),
    );
    assert_eq!(grace_result.horizontal_slowdown, 0.96);
    assert_eq!(sprinting_grace.velocity.y, 0.0);
}

#[test]
fn shallow_lava_applies_two_gravity_steps_while_deep_lava_applies_one() {
    let mut shallow = player_motion();
    fluid_travel_tick(
        &mut shallow,
        FluidTravelContext {
            kind: FluidKind::Lava,
            fluid_height: 0.4,
            fluid_jump_threshold: 0.4,
            ..FluidTravelContext::default()
        },
        &CollisionScene::default(),
    );
    assert_eq!(shallow.velocity.y, -0.025);

    let mut deep = player_motion();
    fluid_travel_tick(
        &mut deep,
        FluidTravelContext {
            kind: FluidKind::Lava,
            fluid_height: 0.400_000_000_1,
            fluid_jump_threshold: 0.4,
            ..FluidTravelContext::default()
        },
        &CollisionScene::default(),
    );
    assert_eq!(deep.velocity.y, -0.02);

    let mut sprinting_shallow = player_motion();
    fluid_travel_tick(
        &mut sprinting_shallow,
        FluidTravelContext {
            kind: FluidKind::Lava,
            fluid_height: 0.4,
            fluid_jump_threshold: 0.4,
            sprinting: true,
            ..FluidTravelContext::default()
        },
        &CollisionScene::default(),
    );
    assert_eq!(sprinting_shallow.velocity.y, -0.02);
    assert_eq!(liquid_jump(Vec3::ZERO), Vec3::new(0.0, 0.04, 0.0));
    assert_eq!(descend_in_water(Vec3::ZERO), Vec3::new(0.0, -0.04, 0.0));
}

#[test]
fn swimming_steering_uses_strict_downward_threshold_and_surface_gate() {
    let mut exact = Vec3::ZERO;
    apply_swimming_steering(
        &mut exact,
        true,
        false,
        false,
        Vec3::new(0.0, -0.2, 1.0),
        false,
    );
    assert_eq!(exact.y, -0.012);

    let mut below = Vec3::ZERO;
    apply_swimming_steering(
        &mut below,
        true,
        false,
        false,
        Vec3::new(0.0, -0.200_001, 1.0),
        false,
    );
    assert_eq!(below.y, -0.200_001 * 0.085);

    let mut upward = Vec3::ZERO;
    apply_swimming_steering(
        &mut upward,
        true,
        false,
        false,
        Vec3::new(0.0, 1.0, 0.0),
        false,
    );
    assert_eq!(upward, Vec3::ZERO);
    apply_swimming_steering(
        &mut upward,
        true,
        false,
        false,
        Vec3::new(0.0, 1.0, 0.0),
        true,
    );
    assert_eq!(upward.y, 0.06);
}

fn valid_gliders() -> [GliderSlot; 2] {
    [
        GliderSlot {
            slot: 2,
            damage: 0,
            maximum_damage: 100,
            has_glider: true,
            equippable_slot_matches: true,
        },
        GliderSlot {
            slot: 5,
            damage: 98,
            maximum_damage: 100,
            has_glider: true,
            equippable_slot_matches: true,
        },
    ]
}

#[test]
fn fall_flying_collision_damage_and_twenty_tick_glider_choice_are_locked() {
    let mut motion = player_motion();
    motion.velocity = Vec3::new(1.0, 0.0, 0.0);
    let scene = CollisionScene {
        block_shapes: vec![Aabb::new(
            Vec3::new(0.3, 64.0, -2.0),
            Vec3::new(2.0, 68.0, 2.0),
        )],
        ..CollisionScene::default()
    };
    let mut timers = TravelTimers::default();
    let gliders = valid_gliders();
    let result = fall_flying_tick(
        &mut motion,
        &mut timers,
        TravelAttributes::default(),
        FallFlyingContext {
            look: Vec3::new(0.0, 0.0, 1.0),
            pitch_radians: 0.0,
            gravity: 0.08,
            on_ground: false,
            passenger: false,
            levitating: false,
            ability_flying: false,
            climbable: false,
            flight_ticks: 20,
            glider_slots: &gliders,
            glider_choice: Some(1),
            ordinary_input: TravelInput::ZERO,
            yaw: 0.0,
            sprinting: false,
        },
        &scene,
    );
    assert!(result.movement.horizontal_collision);
    assert!(result.wall_damage > 0.0);
    assert!(result.remains_fall_flying);
    assert!(result.glide_event);
    assert_eq!(result.damaged_slot, Some(5));
    assert_eq!(
        result.effects,
        vec![
            FallFlyingEffect::DamageGlider(5),
            FallFlyingEffect::GlideEvent
        ]
    );
}

#[test]
fn glider_next_break_and_climbable_each_clear_fall_flying() {
    let invalid = [GliderSlot {
        slot: 2,
        damage: 99,
        maximum_damage: 100,
        has_glider: true,
        equippable_slot_matches: true,
    }];
    let mut motion = player_motion();
    let mut timers = TravelTimers::default();
    let invalid_result = fall_flying_tick(
        &mut motion,
        &mut timers,
        TravelAttributes::default(),
        FallFlyingContext {
            look: Vec3::new(0.0, 0.0, 1.0),
            pitch_radians: 0.0,
            gravity: 0.08,
            on_ground: false,
            passenger: false,
            levitating: false,
            ability_flying: false,
            climbable: false,
            flight_ticks: 10,
            glider_slots: &invalid,
            glider_choice: None,
            ordinary_input: TravelInput::ZERO,
            yaw: 0.0,
            sprinting: false,
        },
        &CollisionScene::default(),
    );
    assert!(!invalid_result.remains_fall_flying);
    assert!(!invalid_result.glide_event);

    let gliders = valid_gliders();
    let climb_result = fall_flying_tick(
        &mut motion,
        &mut timers,
        TravelAttributes::default(),
        FallFlyingContext {
            climbable: true,
            glider_slots: &gliders,
            ..FallFlyingContext {
                look: Vec3::new(0.0, 0.0, 1.0),
                pitch_radians: 0.0,
                gravity: 0.08,
                on_ground: false,
                passenger: false,
                levitating: false,
                ability_flying: false,
                climbable: false,
                flight_ticks: 11,
                glider_slots: &gliders,
                glider_choice: None,
                ordinary_input: TravelInput::ZERO,
                yaw: 0.0,
                sprinting: false,
            }
        },
        &CollisionScene::default(),
    );
    assert!(!climb_result.remains_fall_flying);
}

#[test]
fn ability_flight_wraps_ordinary_and_fluid_travel_but_restores_entry_y() {
    let mut ordinary = player_motion();
    let mut timers = TravelTimers::default();
    let ordinary_result = ability_flight_tick(
        &mut ordinary,
        &mut timers,
        TravelAttributes::default(),
        AbilityFlightContext {
            jump: true,
            shift: false,
            sprinting: true,
            flying_speed: 0.05,
            superclass: AbilitySuperclassTravel::Ordinary(TravelContext {
                input: TravelInput {
                    strafe: 0.0,
                    vertical: 0.0,
                    forward: 1.0,
                },
                sprinting: true,
                ..TravelContext::default()
            }),
        },
        &CollisionScene::default(),
    );
    assert!(ordinary_result.ordinary.is_some());
    assert_eq!(
        ordinary_result.restored_vertical_velocity,
        f64::from(0.15_f32) * 0.6
    );
    assert!(ordinary.position.z > 0.09);

    let mut fluid = player_motion();
    let fluid_result = ability_flight_tick(
        &mut fluid,
        &mut timers,
        TravelAttributes::default(),
        AbilityFlightContext {
            jump: false,
            shift: true,
            sprinting: false,
            flying_speed: 0.05,
            superclass: AbilitySuperclassTravel::Fluid(FluidTravelContext::default()),
        },
        &CollisionScene::default(),
    );
    assert!(fluid_result.fluid.is_some());
    assert_eq!(
        fluid_result.restored_vertical_velocity,
        f64::from(-0.15_f32) * 0.6
    );
}
