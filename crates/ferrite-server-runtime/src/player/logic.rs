use ferrite_foundation::identity::StableEntityId;
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::state::PlayerSessionState;
use ferrite_region_runtime::logic::{
    ImmediateEffectContext, RegionLogic, RegionLogicError, RegionPhaseContext, RegionPhaseOutput,
};
use ferrite_region_runtime::transfer::{TransferRole, TransferredEntityState};
use ferrite_simulation::command::CommandSource;
use ferrite_simulation::journal::JournalDomain;
use ferrite_simulation::tick::TickPhase;

use crate::player::block::logic::apply_block_commands;
use crate::player::command::{PLAYER_STATE_PATH, decode_state};
use crate::session::command::{SessionJoinPayload, SessionLeavePayload};

const JOIN_PATH: &str = "session/join";
const LEAVE_PATH: &str = "session/leave";

#[derive(Debug, Default)]
pub struct PlayerRegionLogic;

impl RegionLogic for PlayerRegionLogic {
    fn execute_phase(
        &mut self,
        mut context: RegionPhaseContext<'_>,
        _output: &mut RegionPhaseOutput,
    ) -> Result<(), RegionLogicError> {
        match context.phase() {
            TickPhase::Ingress => {
                apply_player_commands(&mut context)?;
                apply_block_commands(&mut context)
            }
            TickPhase::ReconcileBoundary => materialize_transferred_players(&mut context),
            _ => Ok(()),
        }
    }

    fn apply_immediate_effect(
        &mut self,
        _context: ImmediateEffectContext<'_>,
    ) -> Result<(), RegionLogicError> {
        Ok(())
    }
}

fn apply_player_commands(context: &mut RegionPhaseContext<'_>) -> Result<(), RegionLogicError> {
    let commands = context.commands().to_vec();
    for command in commands {
        if is_kind(command.kind(), "ferrite", JOIN_PATH) {
            let join = SessionJoinPayload::decode(command.payload()).map_err(|_| logic_error())?;
            validate_player_source(command.source(), join.player)?;
            spawn_player(context, join)?;
        } else if is_kind(command.kind(), "ferrite", LEAVE_PATH) {
            let leave =
                SessionLeavePayload::decode(command.payload()).map_err(|_| logic_error())?;
            validate_player_source(command.source(), leave.player)?;
            remove_player(context, leave)?;
        } else if is_kind(command.kind(), "ferrite", PLAYER_STATE_PATH) {
            let CommandSource::Player(player) = command.source() else {
                return Err(logic_error());
            };
            let state = decode_state(command.payload()).map_err(|_| logic_error())?;
            context
                .state_mut()
                .entities_mut()
                .insert_component(*player, state)
                .map_err(|_| logic_error())?;
            context
                .append_journal(
                    JournalDomain::Mutation,
                    command.kind().clone(),
                    command.payload().to_vec(),
                )
                .map_err(|_| logic_error())?;
        }
    }
    Ok(())
}

fn validate_player_source(
    source: &CommandSource,
    player: StableEntityId,
) -> Result<(), RegionLogicError> {
    match source {
        CommandSource::Player(source_player) if *source_player == player => Ok(()),
        _ => Err(logic_error()),
    }
}

fn remove_player(
    context: &mut RegionPhaseContext<'_>,
    leave: SessionLeavePayload,
) -> Result<(), RegionLogicError> {
    context
        .state_mut()
        .entities_mut()
        .despawn(leave.player)
        .map_err(|_| logic_error())?;
    context
        .append_journal(
            JournalDomain::Mutation,
            ResourceId::new("ferrite", LEAVE_PATH).map_err(|_| logic_error())?,
            leave.encode(),
        )
        .map_err(|_| logic_error())?;
    Ok(())
}

fn spawn_player(
    context: &mut RegionPhaseContext<'_>,
    join: SessionJoinPayload,
) -> Result<(), RegionLogicError> {
    let entities = context.state_mut().entities_mut();
    entities.spawn(join.player).map_err(|_| logic_error())?;
    entities
        .insert_component(join.player, PlayerSessionState::new(join.spawn_pose))
        .map_err(|_| logic_error())?;
    context
        .append_journal(
            JournalDomain::Mutation,
            ResourceId::new("ferrite", JOIN_PATH).map_err(|_| logic_error())?,
            join.encode().map_err(|_| logic_error())?,
        )
        .map_err(|_| logic_error())?;
    Ok(())
}

fn materialize_transferred_players(
    context: &mut RegionPhaseContext<'_>,
) -> Result<(), RegionLogicError> {
    let players = context
        .state()
        .view()
        .entities()
        .stable_ids()
        .filter(|stable_id| {
            context
                .state()
                .view()
                .entities()
                .component::<TransferredEntityState>(*stable_id)
                .is_some_and(|transfer| {
                    transfer.role() == TransferRole::Player
                        && is_kind(transfer.kind(), "minecraft", "player")
                })
        })
        .collect::<Vec<StableEntityId>>();
    for player in players {
        let state = {
            let transfer = context
                .state()
                .view()
                .entities()
                .component::<TransferredEntityState>(player)
                .ok_or_else(logic_error)?;
            PlayerSessionState::decode_transfer(transfer.state()).map_err(|_| logic_error())?
        };
        let entities = context.state_mut().entities_mut();
        entities
            .insert_component(player, state)
            .map_err(|_| logic_error())?;
        entities
            .remove_component::<TransferredEntityState>(player)
            .map_err(|_| logic_error())?;
    }
    Ok(())
}

fn is_kind(kind: &ResourceId, namespace: &str, path: &str) -> bool {
    kind.namespace() == namespace && kind.path() == path
}

fn logic_error() -> RegionLogicError {
    RegionLogicError::new(
        ResourceId::new("ferrite", "player/region_logic")
            .expect("locked player logic identity is valid"),
    )
}
