use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::entity_motion::packet::{
    EntityPositionSync, MinecartStep, MoveMinecartAlongTrack, ProjectilePower, RelativePosition,
    RelativePositionRotation, RelativeRotation, RotateHead, SetEntityMotion, TeleportEntity,
    decode_rotation,
};
use crate::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};

const RELATIVE_X: u32 = 1 << 0;
const RELATIVE_Y: u32 = 1 << 1;
const RELATIVE_Z: u32 = 1 << 2;
const RELATIVE_YAW: u32 = 1 << 3;
const RELATIVE_PITCH: u32 = 1 << 4;
const RELATIVE_MOTION_X: u32 = 1 << 5;
const RELATIVE_MOTION_Y: u32 = 1 << 6;
const RELATIVE_MOTION_Z: u32 = 1 << 7;
const ROTATE_MOTION: u32 = 1 << 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InterpolationMode {
    Immediate,
    #[default]
    DefaultThreeTicks,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterpolationTarget {
    pub position: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub remaining_steps: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MinecartProjectionKind {
    #[default]
    NotMinecart,
    OldBehavior,
    NewBehaviorEnabled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackedMotionEntity {
    pub position: Vector3,
    pub old_position: Vector3,
    pub motion: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub old_yaw: f32,
    pub old_pitch: f32,
    pub head_yaw: f32,
    pub on_ground: bool,
    pub packet_position_base: Vector3,
    pub locally_authoritative: bool,
    pub ticking: bool,
    pub interpolation_mode: InterpolationMode,
    pub interpolation_target: Option<InterpolationTarget>,
    pub living: bool,
    pub head_target: Option<(f32, u8)>,
    pub carries_local_player: bool,
    pub noninterpolating_vehicle: bool,
    pub hurting_projectile: bool,
    pub acceleration_power: f64,
    pub minecart_kind: MinecartProjectionKind,
    pub pending_minecart_steps: Vec<MinecartStep>,
    pub current_minecart_steps: Vec<MinecartStep>,
    pub minecart_window_steps: u8,
    pub old_minecart_motion_target: Option<Vector3>,
}

impl Default for TrackedMotionEntity {
    fn default() -> Self {
        Self {
            position: Vector3::default(),
            old_position: Vector3::default(),
            motion: Vector3::default(),
            yaw: 0.0,
            pitch: 0.0,
            old_yaw: 0.0,
            old_pitch: 0.0,
            head_yaw: 0.0,
            on_ground: false,
            packet_position_base: Vector3::default(),
            locally_authoritative: false,
            ticking: true,
            interpolation_mode: InterpolationMode::DefaultThreeTicks,
            interpolation_target: None,
            living: false,
            head_target: None,
            carries_local_player: false,
            noninterpolating_vehicle: false,
            hurting_projectile: false,
            acceleration_power: 0.0,
            minecart_kind: MinecartProjectionKind::NotMinecart,
            pending_minecart_steps: Vec::new(),
            current_minecart_steps: Vec::new(),
            minecart_window_steps: 0,
            old_minecart_motion_target: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalPlayerMotionState {
    pub position: Vector3,
    pub motion: Vector3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for LocalPlayerMotionState {
    fn default() -> Self {
        Self {
            position: Vector3::default(),
            motion: Vector3::default(),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityMotionAction {
    Ignored,
    PacketBaseOnly,
    Applied,
    Interpolated,
    RiderRepositioned,
    EchoVehicle {
        position: Vector3,
        yaw: f32,
        pitch: f32,
    },
    EchoPlayerPositionRotation {
        position: Vector3,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
        horizontal_collision: bool,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntityMotionClientProjection {
    entities: BTreeMap<i32, TrackedMotionEntity>,
    removed_player_vehicle_id: Option<i32>,
    local_player: LocalPlayerMotionState,
}

impl EntityMotionClientProjection {
    pub fn track_entity(&mut self, entity_id: i32, entity: TrackedMotionEntity) {
        self.entities.insert(entity_id, entity);
    }

    pub fn retain_removed_player_vehicle(&mut self, entity_id: i32) {
        self.removed_player_vehicle_id = Some(entity_id);
    }

    pub fn clear_retained_vehicle_on_add(&mut self, entity_id: i32) {
        if self.removed_player_vehicle_id == Some(entity_id) {
            self.removed_player_vehicle_id = None;
        }
    }

    pub const fn set_local_player(&mut self, state: LocalPlayerMotionState) {
        self.local_player = state;
    }

    #[must_use]
    pub const fn local_player(&self) -> LocalPlayerMotionState {
        self.local_player
    }

    #[must_use]
    pub fn entity(&self, entity_id: i32) -> Option<&TrackedMotionEntity> {
        self.entities.get(&entity_id)
    }

    #[must_use]
    pub const fn retained_vehicle_id(&self) -> Option<i32> {
        self.removed_player_vehicle_id
    }

    pub fn apply(&mut self, packet: &PlayClientboundPacket) -> EntityMotionAction {
        match packet {
            PlayClientboundPacket::EntityPositionSync(packet) => self.apply_position_sync(*packet),
            PlayClientboundPacket::MoveEntityPosition(packet) => self.apply_position(*packet),
            PlayClientboundPacket::MoveEntityPositionRotation(packet) => {
                self.apply_position_rotation(*packet)
            }
            PlayClientboundPacket::MoveEntityRotation(packet) => self.apply_rotation(*packet),
            PlayClientboundPacket::MoveMinecartAlongTrack(packet) => self.apply_minecart(packet),
            PlayClientboundPacket::RotateHead(packet) => self.apply_head(*packet),
            PlayClientboundPacket::SetEntityMotion(packet) => self.apply_motion(*packet),
            PlayClientboundPacket::TeleportEntity(packet) => self.apply_teleport(*packet),
            PlayClientboundPacket::ProjectilePower(packet) => self.apply_projectile(*packet),
            _ => EntityMotionAction::Ignored,
        }
    }

    pub fn tick_interpolation(&mut self, entity_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        let Some(target) = entity.interpolation_target else {
            return false;
        };
        let fraction = 1.0 / f64::from(target.remaining_steps);
        entity.position = lerp_vector(entity.position, target.position, fraction);
        entity.yaw += wrap_degrees(target.yaw - entity.yaw) / f32::from(target.remaining_steps);
        entity.pitch += (target.pitch - entity.pitch) / f32::from(target.remaining_steps);
        if target.remaining_steps == 1 {
            entity.interpolation_target = None;
        } else {
            entity.interpolation_target = Some(InterpolationTarget {
                remaining_steps: target.remaining_steps - 1,
                ..target
            });
        }
        true
    }

    pub fn tick_head_interpolation(&mut self, entity_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        let Some((target, remaining)) = entity.head_target else {
            return false;
        };
        entity.head_yaw += wrap_degrees(target - entity.head_yaw) / f32::from(remaining);
        entity.head_target = (remaining > 1).then_some((target, remaining - 1));
        true
    }

    pub fn activate_minecart_window(&mut self, entity_id: i32) -> Option<f64> {
        let entity = self.entities.get_mut(&entity_id)?;
        if entity.minecart_kind != MinecartProjectionKind::NewBehaviorEnabled {
            return None;
        }
        entity.current_minecart_steps = std::mem::take(&mut entity.pending_minecart_steps);
        let total = entity
            .current_minecart_steps
            .iter()
            .map(|step| f64::from(step.weight))
            .sum::<f64>();
        entity.minecart_window_steps = if total != 0.0 { 3 } else { 0 };
        Some(total)
    }

    #[must_use]
    pub fn select_minecart_step(&self, entity_id: i32, weighted_progress: f64) -> Option<usize> {
        let steps = &self.entities.get(&entity_id)?.current_minecart_steps;
        let mut remaining = weighted_progress;
        for (index, step) in steps.iter().enumerate() {
            if step.weight > 0.0 {
                if remaining < f64::from(step.weight) {
                    return Some(index);
                }
                remaining -= f64::from(step.weight);
            }
        }
        (!steps.is_empty()).then_some(steps.len() - 1)
    }

    fn apply_position(&mut self, packet: RelativePosition) -> EntityMotionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntityMotionAction::Ignored;
        };
        let position = decode_relative_position(
            entity.packet_position_base,
            packet.delta_x,
            packet.delta_y,
            packet.delta_z,
        );
        entity.packet_position_base = position;
        if entity.locally_authoritative {
            return EntityMotionAction::PacketBaseOnly;
        }
        submit_pose(entity, Some(position), None, None);
        entity.on_ground = packet.on_ground;
        interpolation_action(entity)
    }

    fn apply_position_rotation(&mut self, packet: RelativePositionRotation) -> EntityMotionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntityMotionAction::Ignored;
        };
        let position = decode_relative_position(
            entity.packet_position_base,
            packet.delta_x,
            packet.delta_y,
            packet.delta_z,
        );
        entity.packet_position_base = position;
        if entity.locally_authoritative {
            return EntityMotionAction::PacketBaseOnly;
        }
        submit_pose(
            entity,
            Some(position),
            Some(decode_rotation(packet.yaw)),
            Some(decode_rotation(packet.pitch)),
        );
        entity.on_ground = packet.on_ground;
        interpolation_action(entity)
    }

    fn apply_rotation(&mut self, packet: RelativeRotation) -> EntityMotionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntityMotionAction::Ignored;
        };
        if entity.locally_authoritative {
            return EntityMotionAction::PacketBaseOnly;
        }
        submit_pose(
            entity,
            None,
            Some(decode_rotation(packet.yaw)),
            Some(decode_rotation(packet.pitch)),
        );
        entity.on_ground = packet.on_ground;
        interpolation_action(entity)
    }

    fn apply_position_sync(&mut self, packet: EntityPositionSync) -> EntityMotionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntityMotionAction::Ignored;
        };
        entity.packet_position_base = packet.change.position;
        if entity.locally_authoritative {
            return EntityMotionAction::PacketBaseOnly;
        }
        let snap =
            distance_squared(entity.position, packet.change.position) > 4_096.0 || !entity.ticking;
        if snap {
            install_pose(
                entity,
                packet.change.position,
                packet.change.yaw,
                packet.change.pitch,
            );
        } else {
            submit_pose(
                entity,
                Some(packet.change.position),
                Some(packet.change.yaw),
                Some(packet.change.pitch),
            );
        }
        entity.on_ground = packet.on_ground;
        if entity.noninterpolating_vehicle && entity.carries_local_player {
            EntityMotionAction::RiderRepositioned
        } else if snap {
            EntityMotionAction::Applied
        } else {
            interpolation_action(entity)
        }
    }

    fn apply_motion(&mut self, packet: SetEntityMotion) -> EntityMotionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntityMotionAction::Ignored;
        };
        entity.motion = packet.motion;
        if entity.minecart_kind == MinecartProjectionKind::OldBehavior {
            entity.old_minecart_motion_target = Some(packet.motion);
        }
        EntityMotionAction::Applied
    }

    fn apply_head(&mut self, packet: RotateHead) -> EntityMotionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntityMotionAction::Ignored;
        };
        let target = decode_rotation(packet.head_yaw);
        if entity.living {
            entity.head_target = Some((target, 3));
            EntityMotionAction::Interpolated
        } else {
            entity.head_yaw = target;
            EntityMotionAction::Applied
        }
    }

    fn apply_projectile(&mut self, packet: ProjectilePower) -> EntityMotionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntityMotionAction::Ignored;
        };
        if !entity.hurting_projectile {
            return EntityMotionAction::Ignored;
        }
        entity.acceleration_power = packet.acceleration_power;
        EntityMotionAction::Applied
    }

    fn apply_minecart(&mut self, packet: &MoveMinecartAlongTrack) -> EntityMotionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            return EntityMotionAction::Ignored;
        };
        if entity.minecart_kind != MinecartProjectionKind::NewBehaviorEnabled {
            return EntityMotionAction::Ignored;
        }
        entity
            .pending_minecart_steps
            .extend_from_slice(&packet.steps);
        EntityMotionAction::Applied
    }

    fn apply_teleport(&mut self, packet: TeleportEntity) -> EntityMotionAction {
        let Some(entity) = self.entities.get_mut(&packet.entity_id) else {
            if self.removed_player_vehicle_id == Some(packet.entity_id) {
                self.local_player = calculate_absolute(self.local_player, packet);
                return EntityMotionAction::EchoPlayerPositionRotation {
                    position: self.local_player.position,
                    yaw: self.local_player.yaw,
                    pitch: self.local_player.pitch,
                    on_ground: false,
                    horizontal_collision: false,
                };
            }
            return EntityMotionAction::Ignored;
        };
        let source = teleport_source(entity);
        let target = calculate_absolute(source, packet);
        let request_interpolation = entity.ticking
            || !entity.locally_authoritative
            || packet.relative_flags & (RELATIVE_X | RELATIVE_Y | RELATIVE_Z) != 0;
        let interpolate =
            request_interpolation && distance_squared(entity.position, target.position) <= 4_096.0;
        if interpolate {
            submit_pose(
                entity,
                Some(target.position),
                Some(target.yaw),
                Some(target.pitch),
            );
            entity.motion = target.motion;
            entity.on_ground = packet.on_ground;
            return EntityMotionAction::Interpolated;
        }

        let old_source = LocalPlayerMotionState {
            position: entity.old_position,
            motion: Vector3::default(),
            yaw: entity.old_yaw,
            pitch: entity.old_pitch,
        };
        let old_target = calculate_absolute(old_source, packet);
        entity.old_position = old_target.position;
        entity.old_yaw = old_target.yaw;
        entity.old_pitch = old_target.pitch;
        install_pose(entity, target.position, target.yaw, target.pitch);
        entity.motion = target.motion;
        entity.on_ground = packet.on_ground;
        if entity.carries_local_player && entity.locally_authoritative {
            EntityMotionAction::EchoVehicle {
                position: entity.position,
                yaw: entity.yaw,
                pitch: entity.pitch,
            }
        } else if entity.carries_local_player {
            EntityMotionAction::RiderRepositioned
        } else {
            EntityMotionAction::Applied
        }
    }
}

fn decode_relative_position(base: Vector3, delta_x: i16, delta_y: i16, delta_z: i16) -> Vector3 {
    Vector3 {
        x: decode_relative_component(base.x, delta_x),
        y: decode_relative_component(base.y, delta_y),
        z: decode_relative_component(base.z, delta_z),
    }
}

fn decode_relative_component(base: f64, delta: i16) -> f64 {
    if delta == 0 {
        base
    } else {
        java_round(base * 4_096.0).wrapping_add(i64::from(delta)) as f64 / 4_096.0
    }
}

fn java_round(value: f64) -> i64 {
    (value + 0.5).floor() as i64
}

fn submit_pose(
    entity: &mut TrackedMotionEntity,
    position: Option<Vector3>,
    yaw: Option<f32>,
    pitch: Option<f32>,
) {
    let target = InterpolationTarget {
        position: position.unwrap_or(entity.position),
        yaw: yaw.unwrap_or(entity.yaw),
        pitch: pitch.unwrap_or(entity.pitch),
        remaining_steps: 3,
    };
    match entity.interpolation_mode {
        InterpolationMode::Immediate => {
            entity.position = target.position;
            entity.yaw = target.yaw % 360.0;
            entity.pitch = target.pitch % 360.0;
            entity.interpolation_target = None;
        }
        InterpolationMode::DefaultThreeTicks => {
            if !entity.interpolation_target.is_some_and(|active| {
                active.position == target.position
                    && active.yaw == target.yaw
                    && active.pitch == target.pitch
            }) {
                entity.interpolation_target = Some(target);
            }
        }
    }
}

fn interpolation_action(entity: &TrackedMotionEntity) -> EntityMotionAction {
    if entity.interpolation_mode == InterpolationMode::Immediate {
        EntityMotionAction::Applied
    } else {
        EntityMotionAction::Interpolated
    }
}

fn install_pose(entity: &mut TrackedMotionEntity, position: Vector3, yaw: f32, pitch: f32) {
    entity.position = position;
    entity.yaw = yaw;
    entity.pitch = pitch;
    entity.interpolation_target = None;
}

fn teleport_source(entity: &TrackedMotionEntity) -> LocalPlayerMotionState {
    let (position, yaw, pitch) = entity
        .interpolation_target
        .map_or((entity.position, entity.yaw, entity.pitch), |target| {
            (target.position, target.yaw, target.pitch)
        });
    LocalPlayerMotionState {
        position,
        motion: entity.motion,
        yaw,
        pitch,
    }
}

fn calculate_absolute(
    source: LocalPlayerMotionState,
    packet: TeleportEntity,
) -> LocalPlayerMotionState {
    let yaw = component_f32(
        source.yaw,
        packet.change.yaw,
        packet.relative_flags & RELATIVE_YAW != 0,
    );
    let pitch = component_f32(
        source.pitch,
        packet.change.pitch,
        packet.relative_flags & RELATIVE_PITCH != 0,
    )
    .clamp(-90.0, 90.0);
    let mut prior_motion = source.motion;
    if packet.relative_flags & ROTATE_MOTION != 0 {
        prior_motion = rotate_x(prior_motion, (source.pitch - pitch).to_radians());
        prior_motion = rotate_y(prior_motion, (source.yaw - yaw).to_radians());
    }
    LocalPlayerMotionState {
        position: Vector3 {
            x: component(
                source.position.x,
                packet.change.position.x,
                packet.relative_flags & RELATIVE_X != 0,
            ),
            y: component(
                source.position.y,
                packet.change.position.y,
                packet.relative_flags & RELATIVE_Y != 0,
            ),
            z: component(
                source.position.z,
                packet.change.position.z,
                packet.relative_flags & RELATIVE_Z != 0,
            ),
        },
        motion: Vector3 {
            x: component(
                prior_motion.x,
                packet.change.motion.x,
                packet.relative_flags & RELATIVE_MOTION_X != 0,
            ),
            y: component(
                prior_motion.y,
                packet.change.motion.y,
                packet.relative_flags & RELATIVE_MOTION_Y != 0,
            ),
            z: component(
                prior_motion.z,
                packet.change.motion.z,
                packet.relative_flags & RELATIVE_MOTION_Z != 0,
            ),
        },
        yaw,
        pitch,
    }
}

fn component(current: f64, change: f64, relative: bool) -> f64 {
    if relative { current + change } else { change }
}

fn component_f32(current: f32, change: f32, relative: bool) -> f32 {
    if relative { current + change } else { change }
}

fn rotate_x(vector: Vector3, radians: f32) -> Vector3 {
    let sin = f64::from(radians.sin());
    let cos = f64::from(radians.cos());
    Vector3 {
        x: vector.x,
        y: vector.y * cos + vector.z * sin,
        z: vector.z * cos - vector.y * sin,
    }
}

fn rotate_y(vector: Vector3, radians: f32) -> Vector3 {
    let sin = f64::from(radians.sin());
    let cos = f64::from(radians.cos());
    Vector3 {
        x: vector.x * cos + vector.z * sin,
        y: vector.y,
        z: vector.z * cos - vector.x * sin,
    }
}

fn distance_squared(left: Vector3, right: Vector3) -> f64 {
    let x = left.x - right.x;
    let y = left.y - right.y;
    let z = left.z - right.z;
    x * x + y * y + z * z
}

fn lerp_vector(left: Vector3, right: Vector3, fraction: f64) -> Vector3 {
    Vector3 {
        x: left.x + (right.x - left.x) * fraction,
        y: left.y + (right.y - left.y) * fraction,
        z: left.z + (right.z - left.z) * fraction,
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
