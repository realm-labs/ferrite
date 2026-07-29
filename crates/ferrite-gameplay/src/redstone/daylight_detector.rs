//! Daylight-detector periodic and manual signal recomputation.

pub const TICK_PERIOD: u64 = 20;
pub const INVERT_WRITE_FLAGS: u16 = 2;
pub const POWER_WRITE_FLAGS: u16 = 3;
pub const MIN_POWER: i32 = 0;
pub const MAX_POWER: i32 = 15;
pub const SHAPE_HEIGHT: f32 = 6.0 / 16.0;
pub const SUN_SMOOTHING: f32 = 0.2;
pub const DEGREE_TO_RADIAN: f32 = std::f32::consts::PI / 180.0;
pub const BLOCK_ENTITY_HAS_DATA: bool = false;
pub const BLOCK_ENTITY_HAS_UPDATE_PACKET: bool = false;
pub const BLOCK_ENTITY_HAS_RENDERER: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaylightState {
    pub inverted: bool,
    pub power: u8,
}

impl DaylightState {
    pub const fn default_state() -> Self {
        Self {
            inverted: false,
            power: 0,
        }
    }
}

pub const fn ticker_installed(server: bool, has_sky_light: bool) -> bool {
    server && has_sky_light
}

pub const fn periodic_tick_admitted(game_time: u64) -> bool {
    game_time.is_multiple_of(TICK_PERIOD)
}

fn mth_cos(angle: f32) -> f32 {
    let index = ((angle * 10_430.378 + 16_384.0) as i32 & 65_535) as u32;
    let sample_angle = f64::from(index) * std::f64::consts::TAU / 65_536.0;
    sample_angle.sin() as f32
}

fn java_round(value: f32) -> i32 {
    f64::from(value + 0.5).floor() as i32
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DaylightFormula {
    pub effective_brightness: i32,
    pub initial_angle: Option<f32>,
    pub smoothed_angle: Option<f32>,
    pub unclamped_target: i32,
    pub target: u8,
}

pub fn daylight_formula(
    inverted: bool,
    sky_brightness: i32,
    sky_darken: i32,
    sun_angle_degrees: f64,
) -> DaylightFormula {
    let brightness = sky_brightness - sky_darken;
    if inverted {
        let target = 15 - brightness;
        return DaylightFormula {
            effective_brightness: brightness,
            initial_angle: None,
            smoothed_angle: None,
            unclamped_target: target,
            target: target.clamp(MIN_POWER, MAX_POWER) as u8,
        };
    }
    if brightness <= 0 {
        return DaylightFormula {
            effective_brightness: brightness,
            initial_angle: None,
            smoothed_angle: None,
            unclamped_target: brightness,
            target: brightness.clamp(MIN_POWER, MAX_POWER) as u8,
        };
    }
    let angle = sun_angle_degrees as f32 * DEGREE_TO_RADIAN;
    let endpoint = if angle < std::f32::consts::PI {
        0.0
    } else {
        std::f32::consts::TAU
    };
    let smoothed = angle + (endpoint - angle) * SUN_SMOOTHING;
    let target = java_round(brightness as f32 * mth_cos(smoothed));
    DaylightFormula {
        effective_brightness: brightness,
        initial_angle: Some(angle),
        smoothed_angle: Some(smoothed),
        unclamped_target: target,
        target: target.clamp(MIN_POWER, MAX_POWER) as u8,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaylightUpdatePlan {
    pub offered_power: Option<u8>,
    pub write_flags: Option<u16>,
    pub write_result_ignored: bool,
}

pub const fn daylight_update(current_power: u8, target: u8) -> DaylightUpdatePlan {
    if current_power == target {
        DaylightUpdatePlan {
            offered_power: None,
            write_flags: None,
            write_result_ignored: false,
        }
    } else {
        DaylightUpdatePlan {
            offered_power: Some(target),
            write_flags: Some(POWER_WRITE_FLAGS),
            write_result_ignored: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaylightInteractionResult {
    Pass,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaylightUsePlan {
    pub result: DaylightInteractionResult,
    pub intended_inverted: Option<bool>,
    pub first_write_flags: Option<u16>,
    pub emit_block_change: bool,
    pub recompute_intended_state: bool,
    pub second_power_offer: Option<u8>,
    pub second_write_flags: Option<u16>,
}

pub const fn daylight_use(
    captured: DaylightState,
    may_build: bool,
    client: bool,
    intended_target: u8,
) -> DaylightUsePlan {
    if !may_build {
        return DaylightUsePlan {
            result: DaylightInteractionResult::Pass,
            intended_inverted: None,
            first_write_flags: None,
            emit_block_change: false,
            recompute_intended_state: false,
            second_power_offer: None,
            second_write_flags: None,
        };
    }
    if client {
        return DaylightUsePlan {
            result: DaylightInteractionResult::Success,
            intended_inverted: Some(!captured.inverted),
            first_write_flags: None,
            emit_block_change: false,
            recompute_intended_state: false,
            second_power_offer: None,
            second_write_flags: None,
        };
    }
    DaylightUsePlan {
        result: DaylightInteractionResult::Success,
        intended_inverted: Some(!captured.inverted),
        first_write_flags: Some(INVERT_WRITE_FLAGS),
        emit_block_change: true,
        recompute_intended_state: true,
        second_power_offer: if captured.power != intended_target {
            Some(intended_target)
        } else {
            None
        },
        second_write_flags: if captured.power != intended_target {
            Some(POWER_WRITE_FLAGS)
        } else {
            None
        },
    }
}

pub const fn ordinary_signal(state: DaylightState) -> u8 {
    state.power
}

pub const fn direct_signal() -> u8 {
    0
}
