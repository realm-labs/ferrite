//! Source-ordered client input shaping and server intent retention.

use crate::player::state::Vec3;

const FORWARD_IMPULSE_THRESHOLD: f32 = 1.0e-5;
const DEG_TO_RADIANS: f32 = 0.017_453_292;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ButtonInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shift: bool,
    pub sprint: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SampledInput {
    pub buttons: ButtonInput,
    pub movement: Vec2,
}

impl SampledInput {
    #[must_use]
    pub fn from_buttons(buttons: ButtonInput) -> Self {
        let strafe = opposing_axis(buttons.left, buttons.right);
        let forward = opposing_axis(buttons.forward, buttons.backward);
        let length = (strafe * strafe + forward * forward).sqrt();
        let movement = if length < 1.0e-4 {
            Vec2::ZERO
        } else {
            Vec2::new(strafe / length, forward / length)
        };
        Self { buttons, movement }
    }

    #[must_use]
    pub const fn has_forward_impulse(self) -> bool {
        self.movement.y > FORWARD_IMPULSE_THRESHOLD
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputModifiers {
    pub using_item: bool,
    pub passenger: bool,
    pub item_speed_multiplier: f32,
    pub crouching: bool,
    pub visually_crawling: bool,
    pub sneaking_speed: f64,
}

impl Default for InputModifiers {
    fn default() -> Self {
        Self {
            using_item: false,
            passenger: false,
            item_speed_multiplier: 0.2,
            crouching: false,
            visually_crawling: false,
            sneaking_speed: 0.3,
        }
    }
}

#[must_use]
pub fn shape_movement(sampled: Vec2, modifiers: InputModifiers) -> Vec2 {
    if sampled == Vec2::ZERO {
        return sampled;
    }
    let mut shaped = scale(sampled, 0.98);
    if modifiers.using_item && !modifiers.passenger {
        shaped = scale(shaped, modifiers.item_speed_multiplier);
    }
    if modifiers.crouching || modifiers.visually_crawling {
        shaped = scale(shaped, modifiers.sneaking_speed as f32);
    }
    square_remap(shaped)
}

#[must_use]
pub fn square_remap(vector: Vec2) -> Vec2 {
    let length = vector.length_squared().sqrt();
    if length == 0.0 {
        return vector;
    }
    let direction = Vec2::new(vector.x / length, vector.y / length);
    let minimum = direction.x.abs().min(direction.y.abs());
    let maximum = direction.x.abs().max(direction.y.abs());
    let ratio = minimum / maximum;
    let square_distance = (1.0_f32 + ratio * ratio).sqrt();
    scale(direction, (length * square_distance).min(1.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStance {
    Standing,
    Crouching,
    Swimming,
    Sleeping,
    FallFlying,
    SpinAttack,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalInputContext {
    pub loaded: bool,
    pub ability_flying: bool,
    pub may_fly: bool,
    pub spectator: bool,
    pub swimming: bool,
    pub riding: bool,
    pub sleeping: bool,
    pub crouching_fits: bool,
    pub standing_fits: bool,
    pub swimming_fits: bool,
    pub fall_flying: bool,
    pub spin_attacking: bool,
    pub blindness: bool,
    pub food_level: u8,
    pub in_water: bool,
    pub underwater: bool,
    pub using_item: bool,
    pub item_can_sprint: bool,
    pub moving_slowly: bool,
    pub horizontal_collision: bool,
    pub minor_horizontal_collision: bool,
    pub on_ground: bool,
    pub climbable: bool,
    pub can_start_fall_flying: bool,
    pub vehicle_permits_sprint: bool,
    pub vehicle_locally_authoritative: bool,
    pub vehicle_jumpable: bool,
    pub sprint_window: u8,
    pub flying_speed: f32,
}

impl Default for LocalInputContext {
    fn default() -> Self {
        Self {
            loaded: true,
            ability_flying: false,
            may_fly: false,
            spectator: false,
            swimming: false,
            riding: false,
            sleeping: false,
            crouching_fits: true,
            standing_fits: true,
            swimming_fits: true,
            fall_flying: false,
            spin_attacking: false,
            blindness: false,
            food_level: 20,
            in_water: false,
            underwater: false,
            using_item: false,
            item_can_sprint: false,
            moving_slowly: false,
            horizontal_collision: false,
            minor_horizontal_collision: false,
            on_ground: false,
            climbable: false,
            can_start_fall_flying: false,
            vehicle_permits_sprint: false,
            vehicle_locally_authoritative: false,
            vehicle_jumpable: false,
            sprint_window: 7,
            flying_speed: 0.05,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    AbilitiesChanged,
    StartFallFlying,
    GroundJumpForFlight,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalInputState {
    previous: ButtonInput,
    current: ButtonInput,
    movement: Vec2,
    pub crouching: bool,
    pub stance: PlayerStance,
    pub sprinting: bool,
    pub flying: bool,
    pub sprint_trigger_time: u8,
    pub jump_trigger_time: u8,
    pub auto_jump_time: u8,
}

impl Default for LocalInputState {
    fn default() -> Self {
        Self {
            previous: ButtonInput::default(),
            current: ButtonInput::default(),
            movement: Vec2::ZERO,
            crouching: false,
            stance: PlayerStance::Standing,
            sprinting: false,
            flying: false,
            sprint_trigger_time: 0,
            jump_trigger_time: 0,
            auto_jump_time: 0,
        }
    }
}

impl LocalInputState {
    #[must_use]
    pub const fn current(&self) -> ButtonInput {
        self.current
    }

    #[must_use]
    pub const fn movement(&self) -> Vec2 {
        self.movement
    }

    pub fn schedule_auto_jump(&mut self) {
        self.auto_jump_time = 1;
    }

    pub fn tick(
        &mut self,
        sampled_buttons: ButtonInput,
        context: LocalInputContext,
    ) -> Vec<InputAction> {
        if !context.loaded {
            return Vec::new();
        }
        self.sprint_trigger_time = self.sprint_trigger_time.saturating_sub(1);
        let previous = self.current;
        let previous_forward = SampledInput::from_buttons(previous).has_forward_impulse();
        self.crouching = !context.ability_flying
            && !context.swimming
            && !context.riding
            && context.crouching_fits
            && (previous.shift || (!context.sleeping && !context.standing_fits));

        let mut sample = SampledInput::from_buttons(sampled_buttons);
        let auto_jump = self.auto_jump_time > 0;
        if auto_jump {
            self.auto_jump_time -= 1;
            sample.buttons.jump = true;
        }
        self.previous = previous;
        self.current = sample.buttons;
        self.movement = sample.movement;

        self.update_sprint(previous, previous_forward, context);
        let mut actions = Vec::new();
        self.update_flight(previous, auto_jump, context, &mut actions);
        self.stance = select_stance(self.stance, self.current, context);
        actions
    }

    #[must_use]
    pub fn vertical_flight_acceleration(&self, context: LocalInputContext) -> f64 {
        if !self.flying {
            return 0.0;
        }
        let direction = i8::from(self.current.jump) - i8::from(self.current.shift);
        f64::from(direction) * f64::from(context.flying_speed * 3.0)
    }

    fn update_sprint(
        &mut self,
        previous: ButtonInput,
        previous_forward: bool,
        context: LocalInputContext,
    ) {
        let item_disallows = context.using_item && !context.item_can_sprint;
        if previous.shift || (context.riding && item_disallows) || self.current.backward {
            self.sprint_trigger_time = 0;
        }
        let forward = self.movement.y > FORWARD_IMPULSE_THRESHOLD;
        let sprint_possible = sprint_possible(context);
        let can_start = forward
            && sprint_possible
            && !item_disallows
            && (!context.fall_flying || context.underwater)
            && (!context.moving_slowly || context.underwater);
        if !self.sprinting && can_start {
            if !previous_forward {
                if self.sprint_trigger_time > 0 {
                    self.sprinting = true;
                } else {
                    self.sprint_trigger_time = context.sprint_window.min(10);
                }
            }
            if self.current.sprint {
                self.sprinting = true;
            }
        }
        if self.sprinting {
            let stop = if context.swimming {
                !sprint_possible
                    || !context.in_water
                    || !(forward || context.on_ground || self.current.shift)
            } else {
                !sprint_possible
                    || !forward
                    || (context.horizontal_collision && !context.minor_horizontal_collision)
            };
            if stop {
                self.sprinting = false;
            }
        }
    }

    fn update_flight(
        &mut self,
        previous: ButtonInput,
        auto_jump: bool,
        context: LocalInputContext,
        actions: &mut Vec<InputAction>,
    ) {
        let rising_jump = self.current.jump && !previous.jump;
        if context.spectator && context.may_fly && !self.flying {
            self.flying = true;
            actions.push(InputAction::AbilitiesChanged);
        } else if context.may_fly && rising_jump && !auto_jump {
            if self.jump_trigger_time == 0 {
                self.jump_trigger_time = 7;
            } else if !context.swimming && (!context.riding || context.vehicle_jumpable) {
                self.flying = !self.flying;
                if self.flying && context.on_ground {
                    actions.push(InputAction::GroundJumpForFlight);
                }
                actions.push(InputAction::AbilitiesChanged);
                self.jump_trigger_time = 0;
            }
        }
        if self.current.jump
            && actions.is_empty()
            && !previous.jump
            && !context.climbable
            && context.can_start_fall_flying
        {
            actions.push(InputAction::StartFallFlying);
        }
        self.jump_trigger_time = self.jump_trigger_time.saturating_sub(1);
        if context.on_ground
            && self.flying
            && !context.spectator
            && !actions.contains(&InputAction::GroundJumpForFlight)
        {
            self.flying = false;
            actions.push(InputAction::AbilitiesChanged);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSyncMessage {
    Input(ButtonInput),
    StartSprinting,
    StopSprinting,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InputSyncState {
    last_input: ButtonInput,
    last_sprinting: bool,
}

impl InputSyncState {
    pub fn select(
        &mut self,
        current: ButtonInput,
        sprinting: bool,
        passenger: bool,
    ) -> Vec<InputSyncMessage> {
        let mut messages = Vec::new();
        if current != self.last_input {
            messages.push(InputSyncMessage::Input(current));
            self.last_input = current;
        }
        if !passenger && sprinting != self.last_sprinting {
            messages.push(if sprinting {
                InputSyncMessage::StartSprinting
            } else {
                InputSyncMessage::StopSprinting
            });
            self.last_sprinting = sprinting;
        }
        messages
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ServerInputState {
    retained: ButtonInput,
    pub shared_shift: bool,
    pub sprinting: bool,
    pub flying: bool,
    pub action_time_resets: u64,
}

impl ServerInputState {
    pub fn handle_input(&mut self, input: ButtonInput, client_loaded: bool) {
        self.retained = input;
        if client_loaded {
            self.shared_shift = input.shift;
            self.action_time_resets = self.action_time_resets.saturating_add(1);
        }
    }

    pub const fn handle_sprint(&mut self, sprinting: bool, client_loaded: bool) {
        if client_loaded {
            self.sprinting = sprinting;
        }
    }

    pub const fn handle_abilities(&mut self, requested_flying: bool, may_fly: bool) {
        self.flying = requested_flying && may_fly;
    }

    #[must_use]
    pub fn move_intent(&self, yaw: f32) -> Vec3 {
        let sampled = SampledInput::from_buttons(self.retained);
        let angle = yaw * DEG_TO_RADIANS;
        let sine = f64::from(angle.sin());
        let cosine = f64::from(angle.cos());
        Vec3::new(
            f64::from(sampled.movement.x) * cosine - f64::from(sampled.movement.y) * sine,
            0.0,
            f64::from(sampled.movement.y) * cosine + f64::from(sampled.movement.x) * sine,
        )
    }
}

fn sprint_possible(context: LocalInputContext) -> bool {
    if context.blindness {
        return false;
    }
    let resource_gate = if context.riding {
        context.vehicle_permits_sprint && context.vehicle_locally_authoritative
    } else {
        context.food_level > 6 || context.may_fly
    };
    resource_gate && (context.ability_flying || !context.in_water || context.underwater)
}

fn select_stance(
    previous: PlayerStance,
    input: ButtonInput,
    context: LocalInputContext,
) -> PlayerStance {
    let desired = if context.sleeping {
        PlayerStance::Sleeping
    } else if context.swimming {
        PlayerStance::Swimming
    } else if context.fall_flying {
        PlayerStance::FallFlying
    } else if context.spin_attacking {
        PlayerStance::SpinAttack
    } else if input.shift && !context.ability_flying {
        PlayerStance::Crouching
    } else {
        PlayerStance::Standing
    };
    if !context.swimming_fits {
        return previous;
    }
    if context.spectator || context.riding || stance_fits(desired, context) {
        desired
    } else if context.crouching_fits {
        PlayerStance::Crouching
    } else {
        PlayerStance::Swimming
    }
}

const fn stance_fits(stance: PlayerStance, context: LocalInputContext) -> bool {
    match stance {
        PlayerStance::Standing => context.standing_fits,
        PlayerStance::Crouching => context.crouching_fits,
        PlayerStance::Swimming => context.swimming_fits,
        PlayerStance::Sleeping | PlayerStance::FallFlying | PlayerStance::SpinAttack => true,
    }
}

const fn opposing_axis(positive: bool, negative: bool) -> f32 {
    if positive == negative {
        0.0
    } else if positive {
        1.0
    } else {
        -1.0
    }
}

const fn scale(vector: Vec2, factor: f32) -> Vec2 {
    Vec2::new(vector.x * factor, vector.y * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposing_keys_cancel_and_diagonal_uses_java_float_normalization() {
        let canceled = SampledInput::from_buttons(ButtonInput {
            forward: true,
            backward: true,
            left: true,
            right: true,
            ..ButtonInput::default()
        });
        assert_eq!(canceled.movement, Vec2::ZERO);
        let diagonal = SampledInput::from_buttons(ButtonInput {
            forward: true,
            left: true,
            ..ButtonInput::default()
        });
        assert_eq!(diagonal.movement.x, 1.0_f32 / 2.0_f32.sqrt());
        assert_eq!(diagonal.movement.y, diagonal.movement.x);
    }

    #[test]
    fn square_remap_preserves_cardinal_slowdown_and_caps_diagonal() {
        assert_eq!(
            shape_movement(Vec2::new(1.0, 0.0), InputModifiers::default()),
            Vec2::new(0.98, 0.0)
        );
        let diagonal = SampledInput::from_buttons(ButtonInput {
            forward: true,
            left: true,
            ..ButtonInput::default()
        });
        let shaped = shape_movement(
            diagonal.movement,
            InputModifiers {
                using_item: false,
                ..InputModifiers::default()
            },
        );
        assert_eq!(shaped, diagonal.movement);
    }

    #[test]
    fn auto_jump_is_consumed_after_previous_shift_slowdown_is_selected() {
        let mut state = LocalInputState::default();
        state.tick(
            ButtonInput {
                shift: true,
                ..ButtonInput::default()
            },
            LocalInputContext::default(),
        );
        state.schedule_auto_jump();
        state.tick(ButtonInput::default(), LocalInputContext::default());
        assert!(state.crouching);
        assert!(state.current().jump);
        assert_eq!(state.auto_jump_time, 0);
    }

    #[test]
    fn server_retains_input_before_load_but_delays_shared_state() {
        let mut state = ServerInputState::default();
        let input = ButtonInput {
            forward: true,
            shift: true,
            ..ButtonInput::default()
        };
        state.handle_input(input, false);
        assert!(!state.shared_shift);
        assert_eq!(state.move_intent(0.0), Vec3::new(0.0, 0.0, 1.0));
        state.handle_input(input, true);
        assert!(state.shared_shift);
        assert_eq!(state.action_time_resets, 1);
    }
}
