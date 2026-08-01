use ferrite_foundation::coordinate::BlockPos;

use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::clientbound::world_effect::packet::LevelEvent;
use crate::java_26_2::play::clientbound::world_effect::projection::{
    GlobalLevelEffect, local_event_id,
};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelEventViewer {
    pub player_id: u128,
    pub dimension: Identifier,
    pub position: Vector3,
    pub block_position: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelEventDelivery {
    pub recipient: u128,
    pub packet: LevelEvent,
}

#[derive(Debug, Clone, Copy)]
pub struct LocalLevelEventRequest<'a> {
    pub excluded_source_player: Option<u128>,
    pub dimension: &'a Identifier,
    pub effect: &'a Identifier,
    pub position: BlockPos,
    pub data: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownLocalLevelEffect(pub Identifier);

pub fn publish_local_level_event(
    viewers: &[LevelEventViewer],
    request: LocalLevelEventRequest<'_>,
) -> Result<Vec<LevelEventDelivery>, UnknownLocalLevelEffect> {
    let event_type = local_event_id(request.effect)
        .ok_or_else(|| UnknownLocalLevelEffect(request.effect.clone()))?;
    Ok(publish_local_raw(
        viewers,
        request.excluded_source_player,
        request.dimension,
        event_type,
        request.position,
        request.data,
    ))
}

fn publish_local_raw(
    viewers: &[LevelEventViewer],
    excluded_source_player: Option<u128>,
    dimension: &Identifier,
    event_type: i32,
    position: BlockPos,
    data: i32,
) -> Vec<LevelEventDelivery> {
    viewers
        .iter()
        .filter(|viewer| {
            Some(viewer.player_id) != excluded_source_player
                && &viewer.dimension == dimension
                && squared_distance_to_block(viewer.position, position) < 64.0 * 64.0
        })
        .map(|viewer| LevelEventDelivery {
            recipient: viewer.player_id,
            packet: LevelEvent {
                event_type,
                position,
                data,
                global: false,
            },
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalLevelEventRequest<'a> {
    pub dimension: &'a Identifier,
    pub effect: GlobalLevelEffect,
    pub position: BlockPos,
    pub data: i32,
    pub global_sound_events: bool,
}

#[must_use]
pub fn publish_global_level_event(
    viewers: &[LevelEventViewer],
    request: GlobalLevelEventRequest<'_>,
) -> Vec<LevelEventDelivery> {
    if !request.global_sound_events {
        return publish_local_raw(
            viewers,
            None,
            request.dimension,
            request.effect.wire_id(),
            request.position,
            request.data,
        );
    }

    let event_center = block_center(request.position);
    viewers
        .iter()
        .map(|viewer| {
            let position = if &viewer.dimension != request.dimension {
                viewer.block_position
            } else if squared_distance(viewer.position, event_center) < 32.0 * 32.0 {
                request.position
            } else {
                projected_global_position(viewer.position, event_center)
            };
            LevelEventDelivery {
                recipient: viewer.player_id,
                packet: LevelEvent {
                    event_type: request.effect.wire_id(),
                    position,
                    data: request.data,
                    global: true,
                },
            }
        })
        .collect()
}

fn block_center(position: BlockPos) -> Vector3 {
    Vector3 {
        x: f64::from(position.x) + 0.5,
        y: f64::from(position.y) + 0.5,
        z: f64::from(position.z) + 0.5,
    }
}

fn squared_distance_to_block(position: Vector3, block: BlockPos) -> f64 {
    squared_distance(
        position,
        Vector3 {
            x: f64::from(block.x),
            y: f64::from(block.y),
            z: f64::from(block.z),
        },
    )
}

fn squared_distance(left: Vector3, right: Vector3) -> f64 {
    (left.x - right.x).powi(2) + (left.y - right.y).powi(2) + (left.z - right.z).powi(2)
}

fn projected_global_position(player: Vector3, event_center: Vector3) -> BlockPos {
    let direction = Vector3 {
        x: event_center.x - player.x,
        y: event_center.y - player.y,
        z: event_center.z - player.z,
    };
    let length = squared_distance(player, event_center).sqrt();
    if length == 0.0 {
        return floor_position(player);
    }
    floor_position(Vector3 {
        x: player.x + direction.x / length * 32.0,
        y: player.y + direction.y / length * 32.0,
        z: player.z + direction.z / length * 32.0,
    })
}

fn floor_position(position: Vector3) -> BlockPos {
    BlockPos::new(
        position.x.floor() as i32,
        position.y.floor() as i32,
        position.z.floor() as i32,
    )
}
