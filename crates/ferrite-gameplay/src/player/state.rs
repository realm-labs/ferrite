use crate::player::movement::MovementOutcome;

const CLIENT_LOAD_GRACE_TICKS: u8 = 60;
const MINIMUM_GRAVITY_FOR_FLOATING_LIMIT: f64 = 1.0e-5;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn subtract(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    #[must_use]
    pub const fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl Rotation {
    #[must_use]
    pub fn wrapped(self) -> Self {
        Self {
            yaw: wrap_degrees(self.yaw),
            pitch: wrap_degrees(self.pitch),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PlayerPose {
    pub position: Vec3,
    pub rotation: Rotation,
}

impl PlayerPose {
    #[must_use]
    pub const fn new(position: Vec3, rotation: Rotation) -> Self {
        Self { position, rotation }
    }
}

#[derive(Debug, Clone, PartialEq, bevy_ecs::prelude::Component)]
pub struct PlayerSessionState {
    pub(super) pose: PlayerPose,
    pub(super) first_good_position: Vec3,
    pub(super) last_good_position: Vec3,
    pub(super) velocity: Vec3,
    pub(super) known_movement: Vec3,
    pub(super) on_ground: bool,
    pub(super) horizontal_collision: bool,
    pub(super) movement_packets_this_tick: u32,
    pub(super) movement_seen_this_client_interval: bool,
    pub(super) client_load_ticks_remaining: u8,
    pub(super) floating: bool,
    pub(super) floating_ticks: u32,
}

impl PlayerSessionState {
    #[must_use]
    pub const fn new(pose: PlayerPose) -> Self {
        Self {
            pose,
            first_good_position: pose.position,
            last_good_position: pose.position,
            velocity: Vec3::new(0.0, 0.0, 0.0),
            known_movement: Vec3::new(0.0, 0.0, 0.0),
            on_ground: false,
            horizontal_collision: false,
            movement_packets_this_tick: 0,
            movement_seen_this_client_interval: false,
            client_load_ticks_remaining: CLIENT_LOAD_GRACE_TICKS,
            floating: false,
            floating_ticks: 0,
        }
    }

    #[must_use]
    pub const fn pose(&self) -> PlayerPose {
        self.pose
    }

    #[must_use]
    pub const fn first_good_position(&self) -> Vec3 {
        self.first_good_position
    }

    #[must_use]
    pub const fn last_good_position(&self) -> Vec3 {
        self.last_good_position
    }

    #[must_use]
    pub const fn velocity(&self) -> Vec3 {
        self.velocity
    }

    #[must_use]
    pub const fn known_movement(&self) -> Vec3 {
        self.known_movement
    }

    #[must_use]
    pub const fn on_ground(&self) -> bool {
        self.on_ground
    }

    #[must_use]
    pub const fn horizontal_collision(&self) -> bool {
        self.horizontal_collision
    }

    #[must_use]
    pub const fn client_loaded(&self) -> bool {
        self.client_load_ticks_remaining == 0
    }

    #[must_use]
    pub const fn client_load_ticks_remaining(&self) -> u8 {
        self.client_load_ticks_remaining
    }

    #[must_use]
    pub const fn floating(&self) -> bool {
        self.floating
    }

    pub const fn accept_player_loaded(&mut self) {
        self.client_load_ticks_remaining = 0;
    }

    pub const fn restart_client_load_gate(&mut self) {
        self.client_load_ticks_remaining = CLIENT_LOAD_GRACE_TICKS;
    }

    pub const fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }

    pub fn encode_transfer(&self) -> Vec<u8> {
        crate::player::transfer::encode(self)
    }

    pub fn decode_transfer(
        bytes: &[u8],
    ) -> Result<Self, crate::player::transfer::PlayerStateCodecError> {
        crate::player::transfer::decode(bytes)
    }

    pub const fn begin_server_tick(&mut self) {
        self.first_good_position = self.pose.position;
        self.movement_packets_this_tick = 0;
        self.client_load_ticks_remaining = self.client_load_ticks_remaining.saturating_sub(1);
    }

    pub fn finish_client_tick(&mut self) {
        if !self.movement_seen_this_client_interval {
            self.known_movement = Vec3::default();
        }
        self.movement_seen_this_client_interval = false;
    }

    pub fn finish_server_tick(
        &mut self,
        gravity: f64,
        floating_exempt: bool,
    ) -> Option<MovementOutcome> {
        if floating_exempt || !self.floating {
            self.floating_ticks = 0;
            return None;
        }
        if gravity < MINIMUM_GRAVITY_FOR_FLOATING_LIMIT {
            self.floating_ticks = 0;
            return None;
        }
        self.floating_ticks = self.floating_ticks.saturating_add(1);
        let limit = (80.0 * (0.08 / gravity).max(1.0)).ceil() as u32;
        (self.floating_ticks > limit).then_some(MovementOutcome::DisconnectFlying)
    }

    pub(crate) const fn increment_movement_packets(&mut self) -> u32 {
        self.movement_packets_this_tick = self.movement_packets_this_tick.saturating_add(1);
        if self.movement_packets_this_tick > 5 {
            1
        } else {
            self.movement_packets_this_tick
        }
    }

    pub(crate) const fn install_rotation(&mut self, rotation: Rotation) {
        self.pose.rotation = rotation;
    }

    pub(crate) const fn install_authoritative_pose(&mut self, pose: PlayerPose) {
        self.pose = pose;
        self.last_good_position = pose.position;
    }

    pub(crate) const fn accept_movement(
        &mut self,
        pose: PlayerPose,
        known_movement: Vec3,
        on_ground: bool,
        horizontal_collision: bool,
        floating: bool,
    ) {
        self.pose = pose;
        self.last_good_position = pose.position;
        self.known_movement = known_movement;
        self.on_ground = on_ground;
        self.horizontal_collision = horizontal_collision;
        self.movement_seen_this_client_interval = true;
        self.floating = floating;
        if !floating {
            self.floating_ticks = 0;
        }
    }
}

fn wrap_degrees(value: f32) -> f32 {
    let wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped - 360.0
    } else if wrapped < -180.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_gate_expires_and_player_loaded_is_idempotent() {
        let mut state = PlayerSessionState::new(PlayerPose::default());
        for _ in 0..59 {
            state.begin_server_tick();
        }
        assert!(!state.client_loaded());
        state.begin_server_tick();
        assert!(state.client_loaded());
        state.accept_player_loaded();
        state.accept_player_loaded();
        assert!(state.client_loaded());
    }

    #[test]
    fn rotations_wrap_like_the_java_adapter() {
        assert_eq!(
            Rotation {
                yaw: 540.0,
                pitch: -181.0,
            }
            .wrapped(),
            Rotation {
                yaw: -180.0,
                pitch: 179.0,
            }
        );
    }

    #[test]
    fn client_tick_end_zeros_only_missing_known_movement() {
        let mut state = PlayerSessionState::new(PlayerPose::default());
        state.known_movement = Vec3::new(1.0, 0.0, 0.0);
        state.finish_client_tick();
        assert_eq!(state.known_movement(), Vec3::default());
        state.known_movement = Vec3::new(2.0, 0.0, 0.0);
        state.movement_seen_this_client_interval = true;
        state.finish_client_tick();
        assert_eq!(state.known_movement(), Vec3::new(2.0, 0.0, 0.0));
    }
}
