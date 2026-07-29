//! Comparator input sampling, cache, scheduling, refresh, use, and signal behavior.

use ferrite_foundation::direction::Direction;

pub const NEIGHBOR_DELAY: u8 = 2;
pub const PLACEMENT_DELAY: u8 = 1;
pub const STATE_WRITE_FLAGS: u16 = 2;
pub const EXPERIMENTAL_ORIENTATION_BOUND: u8 = 48;
pub const COMPARATOR_SHAPE_HEIGHT: f32 = 2.0 / 16.0;
pub const CLICK_VOLUME: f32 = 0.3;
pub const COMPARE_PITCH: f32 = 0.5;
pub const SUBTRACT_PITCH: f32 = 0.55;
pub const OUTPUT_SIGNAL_KEY: &str = "OutputSignal";
pub const BLOCK_ENTITY_SETTER_MARKS_CHANGED: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparatorMode {
    Compare,
    Subtract,
}

impl ComparatorMode {
    pub const fn cycled(self) -> Self {
        match self {
            Self::Compare => Self::Subtract,
            Self::Subtract => Self::Compare,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparatorState {
    pub facing: Direction,
    pub mode: ComparatorMode,
    pub powered: bool,
}

impl ComparatorState {
    pub const fn default_state() -> Self {
        Self {
            facing: Direction::North,
            mode: ComparatorMode::Compare,
            powered: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemFrameSample {
    pub attachment: Direction,
    pub has_item: bool,
    pub rotation: u8,
}

impl ItemFrameSample {
    pub const fn analog_output(self) -> i32 {
        if self.has_item {
            (self.rotation % 8 + 1) as i32
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RearInputProbe<'a> {
    pub facing: Direction,
    pub immediate_signal: u8,
    pub immediate_wire_power: Option<u8>,
    pub immediate_analog: Option<i32>,
    pub immediate_conductor: bool,
    pub second_analog: Option<i32>,
    pub frames: &'a [ItemFrameSample],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RearInputResult {
    pub input: i32,
    pub wire_queried: bool,
    pub immediate_analog_replaced: bool,
    pub second_position_queried: bool,
    pub matching_frames: u8,
    pub frame_candidate: Option<i32>,
    pub second_analog_candidate: Option<i32>,
}

pub fn rear_input(probe: RearInputProbe<'_>) -> RearInputResult {
    let mut input = i32::from(probe.immediate_signal);
    let wire_queried = input < 15;
    if wire_queried {
        input = input.max(i32::from(probe.immediate_wire_power.unwrap_or(0)));
    }
    if let Some(analog) = probe.immediate_analog {
        return RearInputResult {
            input: analog,
            wire_queried,
            immediate_analog_replaced: true,
            second_position_queried: false,
            matching_frames: 0,
            frame_candidate: None,
            second_analog_candidate: None,
        };
    }
    if input >= 15 || !probe.immediate_conductor {
        return RearInputResult {
            input,
            wire_queried,
            immediate_analog_replaced: false,
            second_position_queried: false,
            matching_frames: 0,
            frame_candidate: None,
            second_analog_candidate: None,
        };
    }

    let mut matching = 0_u8;
    let mut sole_frame = None;
    for frame in probe.frames {
        if frame.attachment == probe.facing {
            matching = matching.saturating_add(1);
            sole_frame = Some(frame.analog_output());
        }
    }
    let frame_candidate = if matching == 1 { sole_frame } else { None };
    if frame_candidate.is_some() || probe.second_analog.is_some() {
        input = frame_candidate
            .unwrap_or(0)
            .max(probe.second_analog.unwrap_or(0));
    }
    RearInputResult {
        input,
        wire_queried,
        immediate_analog_replaced: false,
        second_position_queried: true,
        matching_frames: matching,
        frame_candidate,
        second_analog_candidate: probe.second_analog,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparatorCalculation {
    pub output: i32,
    pub powered: bool,
    pub side_sampled: bool,
}

pub const fn comparator_calculation(
    mode: ComparatorMode,
    rear: i32,
    side: i32,
) -> ComparatorCalculation {
    if rear == 0 {
        return ComparatorCalculation {
            output: 0,
            powered: false,
            side_sampled: false,
        };
    }
    let output = if side > rear {
        0
    } else {
        match mode {
            ComparatorMode::Compare => rear,
            ComparatorMode::Subtract => rear - side,
        }
    };
    ComparatorCalculation {
        output,
        powered: rear > side || rear == side && matches!(mode, ComparatorMode::Compare),
        side_sampled: true,
    }
}

pub const fn side_input(clockwise: u8, counter_clockwise: u8) -> u8 {
    if clockwise > counter_clockwise {
        clockwise
    } else {
        counter_clockwise
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickPriority {
    Normal,
    High,
}

pub fn neighbor_priority(
    comparator_facing: Direction,
    output_is_diode: bool,
    output_diode_facing: Direction,
) -> TickPriority {
    if output_is_diode && output_diode_facing != comparator_facing.opposite() {
        TickPriority::High
    } else {
        TickPriority::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparatorSchedule {
    pub delay: u8,
    pub priority: TickPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeighborCheck {
    pub calculation_performed: bool,
    pub powered_resampled: bool,
    pub schedule: Option<ComparatorSchedule>,
}

pub const fn neighbor_check(
    already_running_this_tick: bool,
    calculated_output: i32,
    cached_output: Option<i32>,
    calculated_powered: bool,
    state_powered: bool,
    priority: TickPriority,
) -> NeighborCheck {
    if already_running_this_tick {
        return NeighborCheck {
            calculation_performed: false,
            powered_resampled: false,
            schedule: None,
        };
    }
    let old = match cached_output {
        Some(value) => value,
        None => 0,
    };
    let output_changed = calculated_output != old;
    let powered_resampled = !output_changed;
    NeighborCheck {
        calculation_performed: true,
        powered_resampled,
        schedule: if output_changed || calculated_powered != state_powered {
            Some(ComparatorSchedule {
                delay: NEIGHBOR_DELAY,
                priority,
            })
        } else {
            None
        },
    }
}

pub const fn placement_schedule(initially_powered: bool) -> Option<ComparatorSchedule> {
    if initially_powered {
        Some(ComparatorSchedule {
            delay: PLACEMENT_DELAY,
            priority: TickPriority::Normal,
        })
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputNotification {
    NeighborChanged,
    NeighborsExceptFacing,
}

pub const OUTPUT_NOTIFICATION_ORDER: [OutputNotification; 2] = [
    OutputNotification::NeighborChanged,
    OutputNotification::NeighborsExceptFacing,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshStage {
    CalculateOutput,
    ReadOldCache,
    WriteCompatibleCache,
    ResamplePowered,
    OfferPoweredState,
    NeighborChanged,
    NeighborsExceptFacing,
}

pub const REFRESH_ORDER: [RefreshStage; 7] = [
    RefreshStage::CalculateOutput,
    RefreshStage::ReadOldCache,
    RefreshStage::WriteCompatibleCache,
    RefreshStage::ResamplePowered,
    RefreshStage::OfferPoweredState,
    RefreshStage::NeighborChanged,
    RefreshStage::NeighborsExceptFacing,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrientationBias {
    Left,
    Up,
    FrontOppositeFacing,
}

pub const EXPERIMENTAL_ORIENTATION_BIAS: [OrientationBias; 3] = [
    OrientationBias::Left,
    OrientationBias::Up,
    OrientationBias::FrontOppositeFacing,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefreshPlan {
    pub old_cached_output: i32,
    pub cache_write: Option<i32>,
    pub cache_marked_changed: bool,
    pub powered_state_offer: Option<bool>,
    pub state_write_flags: Option<u16>,
    pub notify_output: bool,
    pub experimental_orientation_draw_consumed: bool,
    pub orientation_bound: Option<u8>,
}

pub const fn refresh_plan(
    mode: ComparatorMode,
    state_powered: bool,
    calculated_output: i32,
    calculated_powered: bool,
    compatible_block_entity: bool,
    cached_output: i32,
    redstone_experiments: bool,
) -> RefreshPlan {
    let old = if compatible_block_entity {
        cached_output
    } else {
        0
    };
    let notify = old != calculated_output || matches!(mode, ComparatorMode::Compare);
    RefreshPlan {
        old_cached_output: old,
        cache_write: if compatible_block_entity {
            Some(calculated_output)
        } else {
            None
        },
        cache_marked_changed: false,
        powered_state_offer: if notify && state_powered != calculated_powered {
            Some(calculated_powered)
        } else {
            None
        },
        state_write_flags: if notify && state_powered != calculated_powered {
            Some(STATE_WRITE_FLAGS)
        } else {
            None
        },
        notify_output: notify,
        experimental_orientation_draw_consumed: notify && redstone_experiments,
        orientation_bound: if notify && redstone_experiments {
            Some(EXPERIMENTAL_ORIENTATION_BOUND)
        } else {
            None
        },
    }
}

pub fn comparator_signal(
    state: ComparatorState,
    cached_output: i32,
    query_direction: Direction,
) -> i32 {
    if state.powered && query_direction == state.facing {
        cached_output
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionResult {
    Pass,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundRecipients {
    LocalExceptPlayer,
    ServerExceptPlayer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComparatorUsePlan {
    pub result: InteractionResult,
    pub intended_mode: Option<ComparatorMode>,
    pub sound_seed_long_consumed: bool,
    pub sound_recipients: Option<SoundRecipients>,
    pub pitch: Option<f32>,
    pub state_write_offered: bool,
    pub state_write_flags: Option<u16>,
    pub refresh_intended_state: bool,
}

pub const fn comparator_use(
    captured_mode: ComparatorMode,
    may_build: bool,
    client: bool,
    live_identity_after_write: bool,
) -> ComparatorUsePlan {
    if !may_build {
        return ComparatorUsePlan {
            result: InteractionResult::Pass,
            intended_mode: None,
            sound_seed_long_consumed: false,
            sound_recipients: None,
            pitch: None,
            state_write_offered: false,
            state_write_flags: None,
            refresh_intended_state: false,
        };
    }
    let intended = captured_mode.cycled();
    ComparatorUsePlan {
        result: InteractionResult::Success,
        intended_mode: Some(intended),
        sound_seed_long_consumed: true,
        sound_recipients: Some(if client {
            SoundRecipients::LocalExceptPlayer
        } else {
            SoundRecipients::ServerExceptPlayer
        }),
        pitch: Some(match intended {
            ComparatorMode::Compare => COMPARE_PITCH,
            ComparatorMode::Subtract => SUBTRACT_PITCH,
        }),
        state_write_offered: !client,
        state_write_flags: if client {
            None
        } else {
            Some(STATE_WRITE_FLAGS)
        },
        refresh_intended_state: !client && live_identity_after_write,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportLossPlan {
    pub capture_block_entity_for_drops: bool,
    pub remove_moving_false: bool,
    pub update_all_six_neighbors: bool,
}

pub const fn support_loss_plan(has_rigid_support: bool) -> Option<SupportLossPlan> {
    if has_rigid_support {
        None
    } else {
        Some(SupportLossPlan {
            capture_block_entity_for_drops: true,
            remove_moving_false: true,
            update_all_six_neighbors: true,
        })
    }
}

pub const fn removal_notifies_output(piston_moved: bool) -> bool {
    !piston_moved
}

pub const fn loaded_output_signal(value: Option<i32>) -> i32 {
    match value {
        Some(value) => value,
        None => 0,
    }
}

pub const fn forward_block_event(super_result: bool, block_entity_result: Option<bool>) -> bool {
    let _ = super_result;
    match block_entity_result {
        Some(result) => result,
        None => false,
    }
}
