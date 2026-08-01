use crate::java_26_2::play::clientbound::entity_effects::packet::SoundEventHolder;
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::clientbound::sound::packet::{
    SoundAtEntity, SoundAtPosition, SoundSource, StopSound,
};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct SoundViewer {
    pub player_id: u128,
    pub dimension: Identifier,
    pub position: Vector3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthoredSound {
    pub holder: SoundEventHolder,
    pub fixed_range: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundDelivery<T> {
    pub recipient: u128,
    pub packet: T,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntitySoundTarget {
    pub entity_id: i32,
    pub position: Vector3,
}

#[derive(Debug, Clone, Copy)]
pub struct PositionSoundRequest<'a> {
    pub excluded_source_player: Option<u128>,
    pub dimension: &'a Identifier,
    pub position: Vector3,
    pub event: &'a AuthoredSound,
    pub source: SoundSource,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

pub fn publish_position_sound(
    viewers: &[SoundViewer],
    request: PositionSoundRequest<'_>,
) -> Vec<SoundDelivery<SoundAtPosition>> {
    let packet = SoundAtPosition::new(
        request.event.holder.clone(),
        request.source,
        request.position,
        request.volume,
        request.pitch,
        request.seed,
    );
    publish_to_audience(
        viewers,
        request.excluded_source_player,
        request.dimension,
        request.position,
        sound_range(request.event, request.volume),
    )
    .into_iter()
    .map(|recipient| SoundDelivery {
        recipient,
        packet: packet.clone(),
    })
    .collect()
}

#[derive(Debug, Clone, Copy)]
pub struct EntitySoundRequest<'a> {
    pub excluded_source_player: Option<u128>,
    pub dimension: &'a Identifier,
    pub target: EntitySoundTarget,
    pub event: &'a AuthoredSound,
    pub source: SoundSource,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

pub fn publish_entity_sound(
    viewers: &[SoundViewer],
    request: EntitySoundRequest<'_>,
) -> Vec<SoundDelivery<SoundAtEntity>> {
    let packet = SoundAtEntity {
        sound: request.event.holder.clone(),
        source: request.source,
        entity_id: request.target.entity_id,
        volume: request.volume,
        pitch: request.pitch,
        seed: request.seed,
    };
    publish_to_audience(
        viewers,
        request.excluded_source_player,
        request.dimension,
        request.target.position,
        sound_range(request.event, request.volume),
    )
    .into_iter()
    .map(|recipient| SoundDelivery {
        recipient,
        packet: packet.clone(),
    })
    .collect()
}

#[must_use]
pub fn publish_stop_sound(
    selected_players: &[u128],
    packet: StopSound,
) -> Vec<SoundDelivery<StopSound>> {
    selected_players
        .iter()
        .map(|recipient| SoundDelivery {
            recipient: *recipient,
            packet: packet.clone(),
        })
        .collect()
}

#[must_use]
pub fn sound_range(event: &AuthoredSound, volume: f32) -> f32 {
    event
        .fixed_range
        .unwrap_or(if volume > 1.0 { 16.0 * volume } else { 16.0 })
}

fn publish_to_audience(
    viewers: &[SoundViewer],
    excluded_source_player: Option<u128>,
    dimension: &Identifier,
    position: Vector3,
    range: f32,
) -> Vec<u128> {
    let squared_range = f64::from(range) * f64::from(range);
    viewers
        .iter()
        .filter(|viewer| {
            Some(viewer.player_id) != excluded_source_player
                && &viewer.dimension == dimension
                && squared_distance(viewer.position, position) < squared_range
        })
        .map(|viewer| viewer.player_id)
        .collect()
}

fn squared_distance(left: Vector3, right: Vector3) -> f64 {
    let dx = left.x - right.x;
    let dy = left.y - right.y;
    let dz = left.z - right.z;
    dx * dx + dy * dy + dz * dz
}
