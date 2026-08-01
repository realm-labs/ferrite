use ferrite_world::generation::border::state::{
    BorderEvent, BorderExtent, BorderStatus, SavedBorder, WorldBorder,
};
use ferrite_world::generation::border::{
    DEFAULT_ABSOLUTE_MAX, DEFAULT_DAMAGE_PER_BLOCK, DEFAULT_SAFE_ZONE, DEFAULT_SIZE,
    DEFAULT_WARNING_BLOCKS, DEFAULT_WARNING_TIME,
};

#[test]
fn defaults_lock_all_authoritative_fields() {
    let border = WorldBorder::default();
    assert_eq!(
        (
            border.center_x,
            border.center_z,
            border.get_size(),
            border.absolute_max,
            border.damage_per_block,
            border.safe_zone,
            border.warning_blocks,
            border.warning_time,
            border.status(),
        ),
        (
            0.0,
            0.0,
            DEFAULT_SIZE,
            DEFAULT_ABSOLUTE_MAX,
            DEFAULT_DAMAGE_PER_BLOCK,
            DEFAULT_SAFE_ZONE,
            DEFAULT_WARNING_BLOCKS,
            DEFAULT_WARNING_TIME,
            BorderStatus::Stationary,
        )
    );
}

#[test]
fn setters_dirty_before_source_ordered_listener_delivery_without_equality_suppression() {
    let mut border = WorldBorder::default();
    border.add_listener(8);
    border.add_listener(3);
    let size = border.set_size(DEFAULT_SIZE);
    assert_eq!(size.dirty_revision, 1);
    assert_eq!(
        size.deliveries
            .iter()
            .map(|delivery| delivery.listener)
            .collect::<Vec<_>>(),
        [8, 3]
    );
    assert!(size.deliveries.iter().all(|delivery| {
        delivery.broadcast_to_dimension
            && delivery.event == BorderEvent::SetSize { size: DEFAULT_SIZE }
    }));

    let damage = border.set_damage_per_block(0.5);
    assert_eq!(damage.dirty_revision, 2);
    assert!(damage.deliveries.iter().all(|delivery| {
        !delivery.broadcast_to_dimension
            && delivery.event == BorderEvent::SetDamagePerBlock { rate: 0.5 }
    }));
    assert_eq!(border.set_safe_zone(7.0).dirty_revision, 3);
    assert_eq!(border.set_center(4.0, -2.0).dirty_revision, 4);
    assert_eq!(border.set_warning_blocks(9).dirty_revision, 5);
    assert_eq!(border.set_warning_time(40).dirty_revision, 6);
    assert_eq!(border.set_absolute_max(100).dirty_revision, 7);
}

#[test]
fn positive_lerp_decrements_first_lags_geometry_and_installs_target_on_completion() {
    let mut border = WorldBorder::default();
    border.lerp_size_between(10.0, 2.0, 2, 100);
    assert_eq!(border.get_size(), 10.0);
    assert_eq!(
        (border.edges().minimum_x, border.edges().maximum_x),
        (-5.0, 5.0)
    );
    assert!(border.tick_if_running(true));
    assert_eq!(border.get_size(), 6.0);
    assert_eq!(
        (border.edges().minimum_x, border.edges().maximum_x),
        (-5.0, 5.0)
    );
    assert_eq!(
        (
            border.edges_at(1.0).minimum_x,
            border.edges_at(1.0).maximum_x,
        ),
        (-3.0, 3.0)
    );
    assert!(border.tick_if_running(true));
    assert_eq!(border.get_size(), 2.0);
    assert_eq!(
        (border.edges().minimum_x, border.edges().maximum_x),
        (-1.0, 1.0)
    );
    assert_eq!(border.status(), BorderStatus::Stationary);
}

#[test]
fn freeze_and_overload_gates_consume_no_remaining_steps() {
    let mut border = WorldBorder::default();
    border.lerp_size_between(20.0, 40.0, 20, 0);
    let revision = border.dirty_revision;
    assert!(!border.tick_if_running(false));
    assert_eq!(border.remaining_ticks(), 20);
    assert_eq!(border.dirty_revision, revision);
    assert!(border.tick_if_running(true));
    assert_eq!(border.remaining_ticks(), 19);
    assert_eq!(border.get_size(), 21.0);
}

#[test]
fn zero_negative_and_equal_direct_lerps_preserve_locked_quirks() {
    let mut zero = WorldBorder::default();
    zero.lerp_size_between(10.0, 30.0, 0, 4);
    assert_eq!(zero.get_size(), 30.0);
    assert!(matches!(zero.extent, BorderExtent::Moving(_)));
    zero.tick_if_running(true);
    assert_eq!(zero.extent, BorderExtent::Static { size: 30.0 });

    let mut negative = WorldBorder::default();
    negative.lerp_size_between(10.0, 30.0, -4, 7);
    assert_eq!(negative.get_size(), 10.0);
    assert_eq!(negative.status(), BorderStatus::Growing);
    negative.tick_if_running(true);
    assert_eq!(negative.extent, BorderExtent::Static { size: 30.0 });

    let mut equal = WorldBorder::default();
    equal.add_listener(4);
    let mutation = equal.lerp_size_between(12.0, 12.0, 99, 0);
    assert_eq!(equal.extent, BorderExtent::Static { size: 12.0 });
    assert_eq!(
        mutation.deliveries[0].event,
        BorderEvent::LerpSize {
            from: 12.0,
            to: 12.0,
            duration_ticks: 99,
        }
    );
}

#[test]
fn save_reload_and_reconnect_snapshot_restart_from_current_and_reset_lag_history() {
    let mut border = WorldBorder::default();
    border.lerp_size_between(20.0, 10.0, 10, 100);
    border.tick_if_running(true);
    border.tick_if_running(true);
    assert_eq!(border.get_size(), 18.0);
    assert_eq!(border.edges().maximum_x, 9.5);
    let saved = border.saved();
    let snapshot = border.snapshot();
    assert_eq!(
        (
            saved.size,
            saved.target_size,
            saved.remaining_ticks,
            snapshot.old_size,
            snapshot.new_size,
            snapshot.remaining_ticks,
        ),
        (18.0, 10.0, 8, 18.0, 10.0, 8)
    );
    let mut resumed = WorldBorder::from_saved(saved, 1_000);
    assert_eq!(resumed.edges().maximum_x, 9.0);
    let BorderExtent::Moving(moving) = resumed.extent else {
        panic!("unequal saved endpoints must resume moving");
    };
    assert_eq!(
        (moving.begin_game_time, moving.end_game_time),
        (1_000, 1_008)
    );
    resumed.tick_if_running(false);
    assert_eq!(resumed.remaining_ticks(), 8);

    let mut client = WorldBorder::from_snapshot(snapshot, 2_000);
    assert_eq!(client.edges().maximum_x, 9.0);
    assert_eq!(client.remaining_ticks(), 8);
    assert!(!client.tick_if_running(false));
    assert_eq!(client.remaining_ticks(), 8);
    assert!(client.tick_if_running(true));
    assert_eq!(client.remaining_ticks(), 7);
}

#[test]
fn static_saved_state_ignores_stale_remaining_ticks() {
    let border = WorldBorder::from_saved(
        SavedBorder {
            center_x: 0.0,
            center_z: 0.0,
            size: 7.0,
            target_size: 7.0,
            remaining_ticks: 50,
            damage_per_block: 0.2,
            safe_zone: 5.0,
            warning_blocks: 5,
            warning_time: 300,
        },
        9,
    );
    assert_eq!(border.remaining_ticks(), 0);
    assert_eq!(border.extent, BorderExtent::Static { size: 7.0 });
    assert_eq!(border.absolute_max, DEFAULT_ABSOLUTE_MAX);

    let expired = WorldBorder::from_saved(
        SavedBorder {
            size: 7.0,
            target_size: 12.0,
            remaining_ticks: -1,
            ..border.saved()
        },
        9,
    );
    assert_eq!(expired.extent, BorderExtent::Static { size: 7.0 });
}
