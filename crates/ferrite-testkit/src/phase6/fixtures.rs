//! Shared deterministic Phase 6 conformance fixtures.

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{
    RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::state::{PlayerPose, Rotation, Vec3};
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, MainHand, ParticleStatus, SessionId, SessionIdentity,
};
use ferrite_server_runtime::session::command::{SessionJoinPayload, SessionLeavePayload};
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::GameTick;
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;

pub fn region() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).expect("fixture world identity is nonzero"),
        DimensionId::new(
            ResourceId::minecraft("overworld").expect("fixture dimension identity is valid"),
        ),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

pub fn simulation_state() -> RegionSimulationState {
    RegionSimulationState::new(
        RegionVoxelState::new(
            region(),
            RegionMapping::V1,
            ChunkLayout::new(
                VerticalSectionRange::new(-4, 24).expect("fixture section range is valid"),
                BlockStateId::new(0),
                BiomeId::new(0),
            ),
        )
        .expect("fixture Region mapping is valid"),
    )
}

pub fn player(value: u128) -> StableEntityId {
    StableEntityId::new(value).expect("fixture player identity is nonzero")
}

pub fn session(value: u64) -> SessionId {
    SessionId::new(value).expect("fixture session identity is nonzero")
}

pub fn identity(value: u128) -> SessionIdentity {
    SessionIdentity {
        profile_id: value,
        name: format!("player-{value}"),
    }
}

pub fn settings() -> ClientSettings {
    ClientSettings {
        language: "en_us".to_owned(),
        view_distance: 10,
        chat_visibility: ChatVisibility::Full,
        chat_colors: true,
        model_customization: 0x7f,
        main_hand: MainHand::Right,
        text_filtering: false,
        allows_listing: true,
        particle_status: ParticleStatus::All,
    }
}

pub fn join_payload(value: u128, transferred: bool) -> SessionJoinPayload {
    SessionJoinPayload {
        session: session(value as u64),
        player: player(value),
        identity: identity(value),
        settings: settings(),
        transferred,
        spawn_pose: PlayerPose::new(Vec3::new(8.5, 65.0, 8.5), Rotation::default()),
    }
}

pub fn leave_payload(value: u128) -> SessionLeavePayload {
    SessionLeavePayload {
        session: session(value as u64),
        player: player(value),
    }
}

pub fn join_command(
    value: u128,
    tick: u64,
    sequence: u64,
) -> ferrite_simulation::command::RegionCommand {
    join_payload(value, false)
        .into_region_command(region(), GameTick::new(tick), sequence)
        .expect("fixture join command is valid")
}

pub fn leave_command(
    value: u128,
    tick: u64,
    sequence: u64,
) -> ferrite_simulation::command::RegionCommand {
    leave_payload(value)
        .into_region_command(region(), GameTick::new(tick), sequence)
        .expect("fixture leave command is valid")
}

#[must_use]
pub const fn initial_generation() -> ActivationGeneration {
    ActivationGeneration::INITIAL
}

#[must_use]
pub const fn spawn_chunk() -> ChunkPos {
    ChunkPos::new(0, 0)
}
