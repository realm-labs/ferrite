use ferrite_world::generation::border::BorderPoint3;
use ferrite_world::generation::border::effects::{
    BorderDamageDecision, BorderDamageInput, OUTSIDE_BORDER_DAMAGE,
};
use ferrite_world::generation::border::geometry::BorderAabb;
use ferrite_world::generation::border::state::WorldBorder;

fn player_at(x: f64, width: f64) -> BorderDamageInput {
    BorderDamageInput {
        alive_living: true,
        in_wall_hit: false,
        is_player: true,
        bounds: BorderAabb {
            minimum_x: x - width * 0.5,
            minimum_y: 0.0,
            minimum_z: -0.5,
            maximum_x: x + width * 0.5,
            maximum_y: 2.0,
            maximum_z: 0.5,
        },
        center_x: x,
        center_z: 0.0,
    }
}

#[test]
fn outside_border_damage_type_locks_registry_metadata_and_tags() {
    assert_eq!(OUTSIDE_BORDER_DAMAGE.identifier, "minecraft:outside_border");
    assert_eq!(OUTSIDE_BORDER_DAMAGE.exhaustion, 0.0);
    assert_eq!(OUTSIDE_BORDER_DAMAGE.message_id, "outsideBorder");
    assert_eq!(
        OUTSIDE_BORDER_DAMAGE.scaling,
        "when_caused_by_living_non_player"
    );
    assert_eq!(
        (
            OUTSIDE_BORDER_DAMAGE.bypasses_armor,
            OUTSIDE_BORDER_DAMAGE.bypasses_wolf_armor,
            OUTSIDE_BORDER_DAMAGE.no_knockback,
        ),
        (true, true, true)
    );
}

#[test]
fn in_wall_dead_nonplayer_and_contained_branches_precede_distance_damage() {
    let mut border = WorldBorder::default();
    border.set_size(10.0);
    let mut input = player_at(20.0, 1.0);
    input.alive_living = false;
    assert_eq!(
        border.damage_decision(input),
        BorderDamageDecision::SkippedDead
    );
    input.alive_living = true;
    input.in_wall_hit = true;
    assert_eq!(
        border.damage_decision(input),
        BorderDamageDecision::InWallPrecedence
    );
    input.in_wall_hit = false;
    input.is_player = false;
    assert_eq!(
        border.damage_decision(input),
        BorderDamageDecision::NotPlayer
    );
    assert_eq!(
        border.damage_decision(player_at(4.0, 1.0)),
        BorderDamageDecision::BoundsContained
    );
}

#[test]
fn safe_zone_rate_and_floor_boundaries_produce_the_exact_float_amount() {
    let mut border = WorldBorder::default();
    border.set_size(10.0);
    border.set_safe_zone(5.0);
    border.set_damage_per_block(0.2);
    assert_eq!(
        border.damage_decision(player_at(10.0, 1.0)),
        BorderDamageDecision::SafeOrDisabled {
            outside_distance: 0.0
        }
    );
    assert_eq!(
        border.damage_decision(player_at(15.0, 1.0)),
        BorderDamageDecision::Submit {
            amount: 1.0,
            outside_distance: -5.0,
        }
    );
    assert_eq!(
        border.damage_decision(player_at(20.0, 1.0)),
        BorderDamageDecision::Submit {
            amount: 2.0,
            outside_distance: -10.0,
        }
    );
    border.set_damage_per_block(0.0);
    assert!(matches!(
        border.damage_decision(player_at(20.0, 1.0)),
        BorderDamageDecision::SafeOrDisabled { .. }
    ));
}

#[test]
fn moving_damage_uses_previous_geometry_until_static_completion_jump() {
    let mut border = WorldBorder::default();
    border.lerp_size_between(10.0, 2.0, 2, 0);
    border.set_safe_zone(0.0);
    border.tick_if_running(true);
    assert_eq!(border.get_size(), 6.0);
    assert_eq!(
        border.damage_decision(player_at(4.0, 1.0)),
        BorderDamageDecision::BoundsContained
    );
    border.tick_if_running(true);
    assert!(matches!(
        border.damage_decision(player_at(4.0, 1.0)),
        BorderDamageDecision::Submit { .. }
    ));
}

#[test]
fn hud_mixes_previous_distance_with_current_size_projection_and_clamps_outside() {
    let mut border = WorldBorder::default();
    border.lerp_size_between(20.0, 10.0, 10, 0);
    border.tick_if_running(true);
    let warning = border.hud_warning(9.0, 0.0);
    assert_eq!(warning.distance, 1.0);
    assert_eq!(warning.projected, 9.0);
    assert_eq!(warning.threshold, 9.0);
    assert!((warning.intensity - 8.0 / 9.0).abs() < f32::EPSILON);

    border.set_size(10.0);
    border.set_warning_blocks(0);
    border.set_warning_time(0);
    assert_eq!(border.hud_warning(8.0, 0.0).intensity, 1.0);
}

#[test]
fn force_field_uses_partial_geometry_but_previous_distance_for_alpha() {
    let mut border = WorldBorder::default();
    border.lerp_size_between(20.0, 10.0, 10, 0);
    border.tick_if_running(true);
    let frame = border.force_field_frame(
        BorderPoint3 {
            x: 9.0,
            y: 64.0,
            z: 0.0,
        },
        0.5,
        4.0,
    );
    assert_eq!((frame.minimum_x, frame.maximum_x), (-9.75, 9.75));
    assert_eq!(frame.previous_distance, 1.0);
    assert!((frame.alpha - 0.75_f64.powi(4)).abs() < f64::EPSILON);
    assert_eq!(
        border
            .force_field_frame(BorderPoint3::default(), 0.5, 4.0)
            .alpha,
        0.0
    );
}
