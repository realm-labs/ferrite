//! Base one-shot move, look, and jump control transitions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveOperation {
    Wait,
    Strafe,
    MoveTo,
    Jumping,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrafeControl {
    pub speed: f32,
    pub forward: f32,
    pub sideways: f32,
    pub next_operation: MoveOperation,
}

#[must_use]
pub fn strafe_control(
    requested_forward: f32,
    requested_sideways: f32,
    movement_speed: f32,
    walkable_step: bool,
) -> StrafeControl {
    let norm = (requested_forward * requested_forward + requested_sideways * requested_sideways)
        .sqrt()
        .max(1.0);
    let speed = 0.25 * movement_speed;
    if walkable_step {
        StrafeControl {
            speed,
            forward: requested_forward / norm,
            sideways: requested_sideways / norm,
            next_operation: MoveOperation::Wait,
        }
    } else {
        StrafeControl {
            speed,
            forward: 1.0,
            sideways: 0.0,
            next_operation: MoveOperation::Wait,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveToControl {
    pub yaw_change_limit: f32,
    pub speed: f32,
    pub stop_forward_input: bool,
    pub request_jump: bool,
    pub next_operation: MoveOperation,
}

#[must_use]
pub fn move_to_control(input: MoveToInput) -> MoveToControl {
    let stop = input.distance_squared < 2.500_000_3e-7;
    let jump = !stop
        && ((input.high_close_target) || (input.obstructing_shape && !input.door && !input.fence));
    MoveToControl {
        yaw_change_limit: 90.0,
        speed: input.speed_modifier * input.movement_speed,
        stop_forward_input: stop,
        request_jump: jump,
        next_operation: if jump {
            MoveOperation::Jumping
        } else {
            MoveOperation::Wait
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveToInput {
    pub distance_squared: f64,
    pub speed_modifier: f32,
    pub movement_speed: f32,
    pub high_close_target: bool,
    pub obstructing_shape: bool,
    pub door: bool,
    pub fence: bool,
}

#[must_use]
pub const fn jumping_continues(on_ground: bool, affected_liquid: bool) -> bool {
    !on_ground && !affected_liquid
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookControlTick {
    pub request_remaining_ticks: u8,
    pub rotate_to_request: bool,
    pub turn_head_to_body_degrees: u8,
    pub clamp_while_navigating: bool,
}

#[must_use]
pub const fn look_control_tick(request_remaining_ticks: u8, navigating: bool) -> LookControlTick {
    LookControlTick {
        request_remaining_ticks: request_remaining_ticks.saturating_sub(1),
        rotate_to_request: request_remaining_ticks > 0,
        turn_head_to_body_degrees: if request_remaining_ticks > 0 { 0 } else { 10 },
        clamp_while_navigating: navigating,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpControlTick {
    pub entity_jumping: bool,
    pub stored_request_after_tick: bool,
}

#[must_use]
pub const fn jump_control_tick(stored_request: bool) -> JumpControlTick {
    JumpControlTick {
        entity_jumping: stored_request,
        stored_request_after_tick: false,
    }
}
