use ferrite_world::generation::border::command::{
    BorderCommandError, MAX_COMMAND_SIZE, MIN_COMMAND_SIZE, TimeSuffix, add_size_command,
    command_ticks, set_size_command,
};
use ferrite_world::generation::border::state::{BorderEvent, BorderExtent, WorldBorder};

#[test]
fn time_suffixes_use_float_multiplication_then_java_round() {
    assert_eq!(command_ticks(0.49, TimeSuffix::None), 0);
    assert_eq!(command_ticks(0.5, TimeSuffix::Tick), 1);
    assert_eq!(command_ticks(1.25, TimeSuffix::Second), 25);
    assert_eq!(command_ticks(0.5, TimeSuffix::Day), 12_000);
    assert_eq!(command_ticks(f32::NAN, TimeSuffix::Tick), 0);
    assert_eq!(
        command_ticks(f32::INFINITY, TimeSuffix::Tick),
        i64::from(i32::MAX)
    );
}

#[test]
fn command_target_bounds_are_closed_and_reject_nan() {
    let mut border = WorldBorder::default();
    assert!(set_size_command(&mut border, MIN_COMMAND_SIZE, 0, 0).is_ok());
    assert!(set_size_command(&mut border, MAX_COMMAND_SIZE, 0, 0).is_ok());
    for target in [0.999, MAX_COMMAND_SIZE + 1.0, f64::NAN] {
        assert!(matches!(
            set_size_command(&mut border, target, 0, 0),
            Err(BorderCommandError::TargetOutOfRange(value)) if value.to_bits() == target.to_bits()
        ));
    }
}

#[test]
fn set_zero_is_immediate_while_positive_duration_starts_at_current_size() {
    let mut border = WorldBorder::default();
    border.add_listener(1);
    let immediate = set_size_command(&mut border, 100.0, 0, 5).unwrap();
    assert_eq!(border.extent, BorderExtent::Static { size: 100.0 });
    assert_eq!(
        immediate.deliveries[0].event,
        BorderEvent::SetSize { size: 100.0 }
    );

    let moving = set_size_command(&mut border, 200.0, 20, 7).unwrap();
    assert_eq!(border.get_size(), 100.0);
    assert_eq!(border.target_size(), 200.0);
    assert_eq!(border.remaining_ticks(), 20);
    assert_eq!(
        moving.deliveries[0].event,
        BorderEvent::LerpSize {
            from: 100.0,
            to: 200.0,
            duration_ticks: 20,
        }
    );
}

#[test]
fn add_uses_current_size_and_adds_existing_remaining_time() {
    let mut border = WorldBorder::default();
    border.lerp_size_between(100.0, 200.0, 20, 0);
    border.tick_if_running(true);
    assert_eq!(border.get_size(), 105.0);
    let mutation = add_size_command(&mut border, 10.0, 5, 1).unwrap();
    assert_eq!(border.get_size(), 105.0);
    assert_eq!(border.target_size(), 115.0);
    assert_eq!(border.remaining_ticks(), 24);
    assert_eq!(
        mutation.deliveries.len(),
        0,
        "the command does not invent listeners"
    );
}
