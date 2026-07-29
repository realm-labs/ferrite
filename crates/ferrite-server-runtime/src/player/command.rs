use ferrite_foundation::identity::StableEntityId;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::{ResourceId, ResourceIdError};
use ferrite_gameplay::player::state::PlayerSessionState;
use ferrite_gameplay::player::transfer::PlayerStateCodecError;
use ferrite_simulation::command::{CommandError, CommandSource, RegionCommand};
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

pub const PLAYER_STATE_PATH: &str = "player/state";

pub fn state_update_command(
    target: SimulationRegionKey,
    tick: GameTick,
    sequence: u64,
    player: StableEntityId,
    state: &PlayerSessionState,
) -> Result<RegionCommand, PlayerCommandError> {
    Ok(RegionCommand::new(
        target,
        tick,
        CommandSource::Player(player),
        sequence,
        ResourceId::new("ferrite", PLAYER_STATE_PATH)?,
        state.encode_transfer(),
    )?)
}

pub fn decode_state(bytes: &[u8]) -> Result<PlayerSessionState, PlayerCommandError> {
    Ok(PlayerSessionState::decode_transfer(bytes)?)
}

#[derive(Debug, Error)]
pub enum PlayerCommandError {
    #[error(transparent)]
    Resource(#[from] ResourceIdError),
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    State(#[from] PlayerStateCodecError),
}
