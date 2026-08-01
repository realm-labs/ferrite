//! Receive-order boss-bar and waypoint client collections.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::play::clientbound::boss_waypoint::packet::{
    BossColor, BossEvent, BossOperation, BossOverlay, TrackedWaypoint, WaypointIdentifier,
    WaypointLocation, WaypointOperation, WaypointPacket,
};
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossProperties {
    pub darken_screen: bool,
    pub play_music: bool,
    pub create_fog: bool,
}

impl BossProperties {
    #[must_use]
    pub const fn from_byte(value: u8) -> Self {
        Self {
            darken_screen: value & 0x01 != 0,
            play_music: value & 0x02 != 0,
            create_fog: value & 0x04 != 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedBossBar {
    pub name: TextComponentNbt,
    pub start_progress: f32,
    pub target_progress: f32,
    pub progress_set_ms: u64,
    pub color: BossColor,
    pub overlay: BossOverlay,
    pub properties: BossProperties,
}

impl ProjectedBossBar {
    #[must_use]
    pub fn visible_progress(&self, now_ms: u64) -> f32 {
        let elapsed = now_ms.saturating_sub(self.progress_set_ms) as f32;
        let fraction = (elapsed / 100.0).clamp(0.0, 1.0);
        self.start_progress + (self.target_progress - self.start_progress) * fraction
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossAggregate {
    pub darken_screen: bool,
    pub play_music: bool,
    pub create_fog: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossCollectionAction {
    Added { replaced: bool },
    Removed { existed: bool },
    Updated,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum BossProjectionError {
    #[error("boss update references missing UUID {id:032x}")]
    MissingBoss { id: u128 },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BossClientProjection {
    order: Vec<u128>,
    bars: BTreeMap<u128, ProjectedBossBar>,
}

impl BossClientProjection {
    pub fn apply(
        &mut self,
        packet: &BossEvent,
        now_ms: u64,
    ) -> Result<BossCollectionAction, BossProjectionError> {
        match &packet.operation {
            BossOperation::Add {
                name,
                progress,
                color,
                overlay,
                properties,
            } => {
                let replaced = self.bars.contains_key(&packet.id);
                if !replaced {
                    self.order.push(packet.id);
                }
                self.bars.insert(
                    packet.id,
                    ProjectedBossBar {
                        name: name.clone(),
                        start_progress: *progress,
                        target_progress: *progress,
                        progress_set_ms: now_ms,
                        color: *color,
                        overlay: *overlay,
                        properties: BossProperties::from_byte(*properties),
                    },
                );
                Ok(BossCollectionAction::Added { replaced })
            }
            BossOperation::Remove => {
                let existed = self.bars.remove(&packet.id).is_some();
                if existed {
                    self.order.retain(|id| *id != packet.id);
                }
                Ok(BossCollectionAction::Removed { existed })
            }
            operation => {
                let bar = self
                    .bars
                    .get_mut(&packet.id)
                    .ok_or(BossProjectionError::MissingBoss { id: packet.id })?;
                match operation {
                    BossOperation::UpdateProgress(progress) => {
                        bar.start_progress = bar.visible_progress(now_ms);
                        bar.target_progress = *progress;
                        bar.progress_set_ms = now_ms;
                    }
                    BossOperation::UpdateName(name) => bar.name = name.clone(),
                    BossOperation::UpdateStyle { color, overlay } => {
                        bar.color = *color;
                        bar.overlay = *overlay;
                    }
                    BossOperation::UpdateProperties(properties) => {
                        bar.properties = BossProperties::from_byte(*properties);
                    }
                    BossOperation::Add { .. } | BossOperation::Remove => unreachable!(),
                }
                Ok(BossCollectionAction::Updated)
            }
        }
    }

    #[must_use]
    pub fn bar(&self, id: u128) -> Option<&ProjectedBossBar> {
        self.bars.get(&id)
    }

    #[must_use]
    pub fn ordered_ids(&self) -> &[u128] {
        &self.order
    }

    #[must_use]
    pub fn rendered_ids(&self, gui_height: i32) -> Vec<u128> {
        let mut next_y = 12;
        let mut rendered = Vec::new();
        for id in &self.order {
            rendered.push(*id);
            next_y += 19;
            if next_y >= gui_height / 3 {
                break;
            }
        }
        rendered
    }

    #[must_use]
    pub fn aggregate(&self) -> BossAggregate {
        BossAggregate {
            darken_screen: self.bars.values().any(|bar| bar.properties.darken_screen),
            play_music: self.bars.values().any(|bar| bar.properties.play_music),
            create_fog: self.bars.values().any(|bar| bar.properties.create_fog),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointCollectionAction {
    Tracked { replaced: bool },
    Untracked { existed: bool },
    Updated,
    TypeMismatchWarned,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WaypointProjectionError {
    #[error("waypoint update references a missing identifier")]
    MissingWaypoint,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackedEntityEye {
    pub block_position: [i32; 3],
    pub eye_position: [f64; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaypointViewer {
    pub camera_position: [f64; 3],
    pub block_position: [i32; 3],
    pub yaw_degrees: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkerProjection {
    pub point: Option<[f64; 3]>,
    pub yaw_difference: f64,
    pub pitch_degrees: f64,
    pub distance_squared: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WaypointClientProjection {
    waypoints: BTreeMap<WaypointIdentifier, TrackedWaypoint>,
    entities: BTreeMap<u128, TrackedEntityEye>,
}

impl WaypointClientProjection {
    pub fn track_entity(&mut self, uuid: u128, entity: TrackedEntityEye) {
        self.entities.insert(uuid, entity);
    }

    pub fn apply(
        &mut self,
        packet: &WaypointPacket,
    ) -> Result<WaypointCollectionAction, WaypointProjectionError> {
        let identifier = packet.waypoint.identifier.clone();
        match packet.operation {
            WaypointOperation::Track => {
                let replaced = self
                    .waypoints
                    .insert(identifier, packet.waypoint.clone())
                    .is_some();
                Ok(WaypointCollectionAction::Tracked { replaced })
            }
            WaypointOperation::Untrack => Ok(WaypointCollectionAction::Untracked {
                existed: self.waypoints.remove(&identifier).is_some(),
            }),
            WaypointOperation::Update => {
                let existing = self
                    .waypoints
                    .get_mut(&identifier)
                    .ok_or(WaypointProjectionError::MissingWaypoint)?;
                match (&mut existing.location, &packet.waypoint.location) {
                    (WaypointLocation::Empty, WaypointLocation::Empty) => {
                        Ok(WaypointCollectionAction::Updated)
                    }
                    (
                        WaypointLocation::Position { x, y, z },
                        WaypointLocation::Position {
                            x: next_x,
                            y: next_y,
                            z: next_z,
                        },
                    ) => {
                        (*x, *y, *z) = (*next_x, *next_y, *next_z);
                        Ok(WaypointCollectionAction::Updated)
                    }
                    (
                        WaypointLocation::Chunk { x, z },
                        WaypointLocation::Chunk {
                            x: next_x,
                            z: next_z,
                        },
                    ) => {
                        (*x, *z) = (*next_x, *next_z);
                        Ok(WaypointCollectionAction::Updated)
                    }
                    (
                        WaypointLocation::Azimuth { angle },
                        WaypointLocation::Azimuth { angle: next },
                    ) => {
                        *angle = *next;
                        Ok(WaypointCollectionAction::Updated)
                    }
                    _ => Ok(WaypointCollectionAction::TypeMismatchWarned),
                }
            }
        }
    }

    #[must_use]
    pub fn waypoint(&self, identifier: &WaypointIdentifier) -> Option<&TrackedWaypoint> {
        self.waypoints.get(identifier)
    }

    #[must_use]
    pub fn project_marker(
        &self,
        identifier: &WaypointIdentifier,
        viewer: WaypointViewer,
    ) -> Option<MarkerProjection> {
        let waypoint = self.waypoints.get(identifier)?;
        Some(match waypoint.location {
            WaypointLocation::Empty => MarkerProjection {
                point: None,
                yaw_difference: f64::NAN,
                pitch_degrees: 0.0,
                distance_squared: f64::INFINITY,
            },
            WaypointLocation::Azimuth { angle } => MarkerProjection {
                point: None,
                yaw_difference: wrap_degrees(f64::from(angle).to_degrees() - viewer.yaw_degrees),
                pitch_degrees: 0.0,
                distance_squared: f64::INFINITY,
            },
            WaypointLocation::Position { x, y, z } => {
                let carried = [x, y, z];
                let point = match identifier {
                    WaypointIdentifier::Uuid(uuid) => self
                        .entities
                        .get(uuid)
                        .filter(|entity| manhattan(entity.block_position, carried) <= 3)
                        .map_or_else(|| block_center(carried), |entity| entity.eye_position),
                    WaypointIdentifier::String(_) => block_center(carried),
                };
                point_projection(viewer, point, point)
            }
            WaypointLocation::Chunk { x, z } => {
                let center_x = f64::from(x) * 16.0 + 8.5;
                let center_z = f64::from(z) * 16.0 + 8.5;
                point_projection(
                    viewer,
                    [center_x, viewer.camera_position[1], center_z],
                    [
                        center_x,
                        f64::from(viewer.block_position[1]) + 0.5,
                        center_z,
                    ],
                )
            }
        })
    }

    #[must_use]
    pub fn markers_by_descending_distance(
        &self,
        viewer: WaypointViewer,
    ) -> Vec<(&WaypointIdentifier, MarkerProjection)> {
        let mut markers: Vec<_> = self
            .waypoints
            .keys()
            .filter_map(|identifier| {
                self.project_marker(identifier, viewer)
                    .map(|projection| (identifier, projection))
            })
            .collect();
        markers.sort_by(|left, right| right.1.distance_squared.total_cmp(&left.1.distance_squared));
        markers
    }
}

fn point_projection(
    viewer: WaypointViewer,
    yaw_point: [f64; 3],
    distance_point: [f64; 3],
) -> MarkerProjection {
    let delta = difference(yaw_point, viewer.camera_position);
    let horizontal = (delta[0] * delta[0] + delta[2] * delta[2]).sqrt();
    let yaw = delta[2].atan2(delta[0]).to_degrees() - 90.0;
    MarkerProjection {
        point: Some(yaw_point),
        yaw_difference: wrap_degrees(yaw - viewer.yaw_degrees),
        pitch_degrees: -delta[1].atan2(horizontal).to_degrees(),
        distance_squared: difference(distance_point, viewer.camera_position)
            .into_iter()
            .map(|value| value * value)
            .sum(),
    }
}

fn block_center(position: [i32; 3]) -> [f64; 3] {
    position.map(|coordinate| f64::from(coordinate) + 0.5)
}

fn manhattan(left: [i32; 3], right: [i32; 3]) -> i64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (i64::from(left) - i64::from(right)).abs())
        .sum()
}

fn difference(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn wrap_degrees(value: f64) -> f64 {
    let wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped - 360.0
    } else if wrapped < -180.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}
