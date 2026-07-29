//! Connection-local controls and chunk-flow state for the C2 movement family.

use crate::java_26_2::configuration::serverbound::packet::ClientInformation;
use crate::java_26_2::play::serverbound::packet::{
    MoveVehicle, PaddleBoat, PlayerAbilities, PlayerCommand, PlayerCommandKind, PlayerInput,
    PlayerPosition, PlayerRotation,
};

const CLIENT_LOAD_GRACE_TICKS: u8 = 60;

#[derive(Debug, Clone, PartialEq)]
pub struct MovementControlProjection {
    client_information: ClientInformation,
    input: PlayerInput,
    flying: bool,
    sprinting: bool,
    fall_flying: bool,
    boat_paddles: PaddleBoat,
    client_load_ticks_remaining: u8,
    chunk_flow: ChunkFlowProjection,
}

impl MovementControlProjection {
    #[must_use]
    pub fn new(client_information: ClientInformation) -> Self {
        Self {
            client_information,
            input: PlayerInput::default(),
            flying: false,
            sprinting: false,
            fall_flying: false,
            boat_paddles: PaddleBoat {
                left: false,
                right: false,
            },
            client_load_ticks_remaining: CLIENT_LOAD_GRACE_TICKS,
            chunk_flow: ChunkFlowProjection::new(),
        }
    }

    pub const fn begin_server_tick(&mut self) {
        self.client_load_ticks_remaining = self.client_load_ticks_remaining.saturating_sub(1);
    }

    pub const fn player_loaded(&mut self) {
        self.client_load_ticks_remaining = 0;
    }

    pub fn update_client_information(&mut self, information: ClientInformation) -> bool {
        let old_hat = self.client_information.model_customization & 0x40 != 0;
        let new_hat = information.model_customization & 0x40 != 0;
        self.client_information = information;
        old_hat != new_hat
    }

    pub const fn update_input(&mut self, input: PlayerInput) -> InputDisposition {
        self.input = input;
        if self.client_loaded() {
            InputDisposition::ApplyLoadedState
        } else {
            InputDisposition::RetainedBeforeClientLoaded
        }
    }

    pub const fn update_abilities(&mut self, abilities: PlayerAbilities, may_fly: bool) {
        if may_fly {
            self.flying = abilities.flying;
        }
    }

    pub const fn update_paddles(&mut self, paddles: PaddleBoat, controls_boat: bool) {
        if controls_boat {
            self.boat_paddles = paddles;
        }
    }

    pub const fn apply_command(
        &mut self,
        command: PlayerCommand,
        context: PlayerCommandContext,
    ) -> PlayerCommandDisposition {
        if !self.client_loaded() {
            return PlayerCommandDisposition::IgnoredBeforeClientLoaded;
        }
        match command.action {
            PlayerCommandKind::StopSleeping => PlayerCommandDisposition::StopSleepingAndCorrect,
            PlayerCommandKind::StartSprinting => {
                self.sprinting = true;
                PlayerCommandDisposition::Applied
            }
            PlayerCommandKind::StopSprinting => {
                self.sprinting = false;
                PlayerCommandDisposition::Applied
            }
            PlayerCommandKind::StartRidingJump
                if command.data > 0 && context.controlled_vehicle_can_jump =>
            {
                PlayerCommandDisposition::StartRidingJump(command.data)
            }
            PlayerCommandKind::StopRidingJump if context.controlled_vehicle_can_jump => {
                PlayerCommandDisposition::StopRidingJump
            }
            PlayerCommandKind::OpenInventory if context.controlled_vehicle_has_inventory => {
                PlayerCommandDisposition::OpenVehicleInventory
            }
            PlayerCommandKind::StartFallFlying => {
                self.fall_flying = context.can_start_fall_flying;
                if self.fall_flying {
                    PlayerCommandDisposition::StartFallFlying
                } else {
                    PlayerCommandDisposition::StopFallFlying
                }
            }
            _ => PlayerCommandDisposition::Ignored,
        }
    }

    #[must_use]
    pub const fn client_loaded(&self) -> bool {
        self.client_load_ticks_remaining == 0
    }

    #[must_use]
    pub const fn input(&self) -> PlayerInput {
        self.input
    }

    #[must_use]
    pub const fn flying(&self) -> bool {
        self.flying
    }

    #[must_use]
    pub const fn sprinting(&self) -> bool {
        self.sprinting
    }

    #[must_use]
    pub const fn fall_flying(&self) -> bool {
        self.fall_flying
    }

    #[must_use]
    pub const fn boat_paddles(&self) -> PaddleBoat {
        self.boat_paddles
    }

    #[must_use]
    pub const fn client_information(&self) -> &ClientInformation {
        &self.client_information
    }

    #[must_use]
    pub const fn chunk_flow(&self) -> &ChunkFlowProjection {
        &self.chunk_flow
    }

    pub fn acknowledge_chunk_batch(&mut self, desired_chunks_per_tick: f32) {
        self.chunk_flow.acknowledge(desired_chunks_per_tick);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDisposition {
    RetainedBeforeClientLoaded,
    ApplyLoadedState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerCommandContext {
    pub controlled_vehicle_can_jump: bool,
    pub controlled_vehicle_has_inventory: bool,
    pub can_start_fall_flying: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerCommandDisposition {
    IgnoredBeforeClientLoaded,
    Applied,
    StopSleepingAndCorrect,
    StartRidingJump(i32),
    StopRidingJump,
    OpenVehicleInventory,
    StartFallFlying,
    StopFallFlying,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkFlowProjection {
    desired_chunks_per_tick: f32,
    unacknowledged_batches: u32,
    maximum_unacknowledged_batches: u32,
    batch_quota: f32,
}

impl ChunkFlowProjection {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            desired_chunks_per_tick: 9.0,
            unacknowledged_batches: 0,
            maximum_unacknowledged_batches: 1,
            batch_quota: 0.0,
        }
    }

    pub const fn batch_sent(&mut self) {
        self.unacknowledged_batches = self.unacknowledged_batches.saturating_add(1);
    }

    pub fn acknowledge(&mut self, desired_chunks_per_tick: f32) {
        self.unacknowledged_batches = self.unacknowledged_batches.saturating_sub(1);
        self.desired_chunks_per_tick = if desired_chunks_per_tick.is_nan() {
            0.01
        } else {
            desired_chunks_per_tick.clamp(0.01, 64.0)
        };
        if self.unacknowledged_batches == 0 {
            self.batch_quota = 1.0;
        }
        self.maximum_unacknowledged_batches = 10;
    }

    #[must_use]
    pub const fn desired_chunks_per_tick(&self) -> f32 {
        self.desired_chunks_per_tick
    }

    #[must_use]
    pub const fn unacknowledged_batches(&self) -> u32 {
        self.unacknowledged_batches
    }

    #[must_use]
    pub const fn maximum_unacknowledged_batches(&self) -> u32 {
        self.maximum_unacknowledged_batches
    }

    #[must_use]
    pub const fn batch_quota(&self) -> f32 {
        self.batch_quota
    }
}

impl Default for ChunkFlowProjection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleMovementProjection {
    position: PlayerPosition,
    rotation: PlayerRotation,
    first_good_position: PlayerPosition,
    last_good_position: PlayerPosition,
    velocity: PlayerPosition,
    floating: bool,
}

impl VehicleMovementProjection {
    #[must_use]
    pub const fn new(position: PlayerPosition, rotation: PlayerRotation) -> Self {
        Self {
            position,
            rotation,
            first_good_position: position,
            last_good_position: position,
            velocity: PlayerPosition {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            floating: false,
        }
    }

    pub const fn begin_server_tick(&mut self) {
        self.first_good_position = self.position;
    }

    pub const fn set_velocity(&mut self, velocity: PlayerPosition) {
        self.velocity = velocity;
    }

    pub fn apply(
        &mut self,
        packet: MoveVehicle,
        context: VehicleMovementContext,
    ) -> VehicleMovementOutcome {
        if packet.position.x.is_nan()
            || packet.position.y.is_nan()
            || packet.position.z.is_nan()
            || !packet.rotation.yaw.is_finite()
            || !packet.rotation.pitch.is_finite()
        {
            return VehicleMovementOutcome::DisconnectInvalidVehicleMovement;
        }
        if !context.controlled_tick_vehicle {
            self.floating = false;
            return VehicleMovementOutcome::Ignored;
        }
        let target = clamp_vehicle_position(packet.position);
        let rotation = wrap_vehicle_rotation(packet.rotation);
        let displacement = subtract_position(target, self.first_good_position);
        let too_quick = length_squared(displacement) - length_squared(self.velocity) > 100.0;
        if too_quick && !context.singleplayer_owner {
            return VehicleMovementOutcome::Correct {
                position: self.position,
                rotation: self.rotation,
            };
        }
        let residual = subtract_position(target, context.collision_result_position);
        let moved_wrongly = residual.x * residual.x + residual.z * residual.z > 0.0625;
        if (moved_wrongly && context.old_box_collision_free) || context.introduced_collision {
            self.rotation = rotation;
            return VehicleMovementOutcome::Correct {
                position: self.position,
                rotation,
            };
        }
        let requested = subtract_position(target, self.last_good_position);
        self.position = target;
        self.rotation = rotation;
        self.last_good_position = target;
        self.floating = requested.y >= -0.03125
            && !context.supporting_collision_before
            && !context.nearby_block_below
            && !context.server_flight_allowed
            && !context.vehicle_flying
            && !context.vehicle_gravity_free;
        VehicleMovementOutcome::Accepted {
            position: target,
            rotation,
            floating: self.floating,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleMovementContext {
    pub controlled_tick_vehicle: bool,
    pub singleplayer_owner: bool,
    pub collision_result_position: PlayerPosition,
    pub old_box_collision_free: bool,
    pub introduced_collision: bool,
    pub supporting_collision_before: bool,
    pub nearby_block_below: bool,
    pub server_flight_allowed: bool,
    pub vehicle_flying: bool,
    pub vehicle_gravity_free: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VehicleMovementOutcome {
    Ignored,
    DisconnectInvalidVehicleMovement,
    Correct {
        position: PlayerPosition,
        rotation: PlayerRotation,
    },
    Accepted {
        position: PlayerPosition,
        rotation: PlayerRotation,
        floating: bool,
    },
}

fn clamp_vehicle_position(position: PlayerPosition) -> PlayerPosition {
    PlayerPosition {
        x: position.x.clamp(-30_000_000.0, 30_000_000.0),
        y: position.y.clamp(-20_000_000.0, 20_000_000.0),
        z: position.z.clamp(-30_000_000.0, 30_000_000.0),
    }
}

fn wrap_vehicle_rotation(rotation: PlayerRotation) -> PlayerRotation {
    PlayerRotation {
        yaw: wrap_degrees(rotation.yaw),
        pitch: wrap_degrees(rotation.pitch),
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

const fn subtract_position(left: PlayerPosition, right: PlayerPosition) -> PlayerPosition {
    PlayerPosition {
        x: left.x - right.x,
        y: left.y - right.y,
        z: left.z - right.z,
    }
}

const fn length_squared(value: PlayerPosition) -> f64 {
    value.x * value.x + value.y * value.y + value.z * value.z
}
