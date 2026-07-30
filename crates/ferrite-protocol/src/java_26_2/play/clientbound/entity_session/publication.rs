#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationPublicationAudience {
    Trackers,
    TrackersAndSelf,
}

#[must_use]
pub const fn animation_audience(
    action: u8,
    swing_include_self: bool,
) -> AnimationPublicationAudience {
    match action {
        0 | 3 if swing_include_self => AnimationPublicationAudience::TrackersAndSelf,
        2 | 4 | 5 => AnimationPublicationAudience::TrackersAndSelf,
        _ => AnimationPublicationAudience::Trackers,
    }
}

#[must_use]
pub const fn publish_damage_event(took_full_damage: bool, blocked: bool) -> bool {
    took_full_damage && !blocked
}

#[must_use]
pub const fn publish_hurt_animation_to_damaged_player(blocked_indication: bool) -> bool {
    !blocked_indication
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPublicationStep {
    ChangeAuthoritativeCamera,
    RelocatePlayer,
    UpdateChunkTracking,
    SendCamera,
    ResetKnownPosition,
}

pub const CAMERA_PUBLICATION_ORDER: [CameraPublicationStep; 5] = [
    CameraPublicationStep::ChangeAuthoritativeCamera,
    CameraPublicationStep::RelocatePlayer,
    CameraPublicationStep::UpdateChunkTracking,
    CameraPublicationStep::SendCamera,
    CameraPublicationStep::ResetKnownPosition,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickupAudience {
    pub tracking_source: bool,
    pub include_source_when_player: bool,
}

pub const PICKUP_AUDIENCE: PickupAudience = PickupAudience {
    tracking_source: true,
    include_source_when_player: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespawnPublicationStep {
    Respawn,
    PositionChallenge,
    DefaultSpawn,
    Difficulty,
    Experience,
    ActiveEffects,
    LevelInfo,
    Permission,
}

pub const DEATH_RESPAWN_ORDER: [RespawnPublicationStep; 8] = [
    RespawnPublicationStep::Respawn,
    RespawnPublicationStep::PositionChallenge,
    RespawnPublicationStep::DefaultSpawn,
    RespawnPublicationStep::Difficulty,
    RespawnPublicationStep::Experience,
    RespawnPublicationStep::ActiveEffects,
    RespawnPublicationStep::LevelInfo,
    RespawnPublicationStep::Permission,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossDimensionPublicationStep {
    Respawn,
    Difficulty,
    Permission,
    TransferLevel,
    PositionChallenge,
    Abilities,
    NewLevelProjection,
}

pub const CROSS_DIMENSION_ORDER: [CrossDimensionPublicationStep; 7] = [
    CrossDimensionPublicationStep::Respawn,
    CrossDimensionPublicationStep::Difficulty,
    CrossDimensionPublicationStep::Permission,
    CrossDimensionPublicationStep::TransferLevel,
    CrossDimensionPublicationStep::PositionChallenge,
    CrossDimensionPublicationStep::Abilities,
    CrossDimensionPublicationStep::NewLevelProjection,
];
