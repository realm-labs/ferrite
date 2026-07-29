use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::block::breaking::{
    BlockBreakSession, BreakAction, BreakDecision, decide_break,
};
use ferrite_gameplay::block::targeting::{
    PACKET_REACH_PADDING, adjacent, valid_reconstructed_hit, within_block_reach,
};
use ferrite_region_runtime::logic::{RegionLogicError, RegionPhaseContext};
use ferrite_simulation::command::{CommandSource, RegionCommand};
use ferrite_simulation::journal::JournalDomain;
use ferrite_world::id::BlockStateId;

use crate::player::block::command::{
    BLOCK_INTERACTION_PATH, BLOCK_RESULT_PATH, BLOCK_UPDATE_PATH, BlockIntent,
    BlockInteractionCommand,
};
use crate::player::block::replication::{
    AuthoritativeBlockUpdate, BlockCommandOutcome, BlockCommandResult, encode_replication,
};

const AIR: BlockStateId = BlockStateId::new(0);

pub fn apply_block_commands(context: &mut RegionPhaseContext<'_>) -> Result<(), RegionLogicError> {
    let commands = context
        .commands()
        .iter()
        .filter(|command| is_block_command(command))
        .cloned()
        .collect::<Vec<_>>();
    for command in commands {
        let decoded =
            BlockInteractionCommand::decode(command.payload()).map_err(|_| logic_error())?;
        let CommandSource::Player(source) = command.source() else {
            return Err(logic_error());
        };
        if *source != decoded.player {
            return Err(logic_error());
        }
        apply_command(context, &command, decoded)?;
    }
    Ok(())
}

fn apply_command(
    context: &mut RegionPhaseContext<'_>,
    command: &RegionCommand,
    request: BlockInteractionCommand,
) -> Result<(), RegionLogicError> {
    let result = match request.intent {
        BlockIntent::StartDestroy { position } => {
            apply_break(context, request, position, BreakAction::Start)?
        }
        BlockIntent::AbortDestroy { position } => {
            apply_break(context, request, position, BreakAction::Abort)?
        }
        BlockIntent::StopDestroy { position } => {
            apply_break(context, request, position, BreakAction::Stop)?
        }
        BlockIntent::UseOn {
            position,
            direction,
            offset_x,
            offset_y,
            offset_z,
            inside: _,
            world_border_hit: _,
            interaction_allowed,
            placement_state,
        } => apply_use_on(
            context,
            request,
            position,
            direction,
            [offset_x, offset_y, offset_z],
            interaction_allowed,
            placement_state,
        )?,
    };
    let result = BlockCommandResult {
        player: request.player,
        command_sequence: command.sequence(),
        outcome: result.outcome,
        corrections: result.corrections,
    };
    context
        .append_journal(
            JournalDomain::Mutation,
            resource(BLOCK_RESULT_PATH)?,
            result.encode(),
        )
        .map_err(|_| logic_error())?;
    Ok(())
}

fn apply_break(
    context: &mut RegionPhaseContext<'_>,
    request: BlockInteractionCommand,
    position: BlockPos,
    action: BreakAction,
) -> Result<InteractionResult, RegionLogicError> {
    let current = match admitted_state(context, request, position) {
        Ok(state) => state,
        Err(()) => return Ok(InteractionResult::new(BlockCommandOutcome::Rejected)),
    };
    let active = context
        .state()
        .view()
        .entities()
        .component::<BlockBreakSession>(request.player)
        .copied();
    let decision = decide_break(action, active, position, current, AIR);
    match decision {
        BreakDecision::Track(session) => {
            context
                .state_mut()
                .entities_mut()
                .insert_component(request.player, session)
                .map_err(|_| logic_error())?;
            Ok(InteractionResult::new(BlockCommandOutcome::Tracking))
        }
        BreakDecision::Clear => {
            if context.state().view().entities().contains(request.player) {
                context
                    .state_mut()
                    .entities_mut()
                    .remove_component::<BlockBreakSession>(request.player)
                    .map_err(|_| logic_error())?;
            }
            Ok(InteractionResult::new(BlockCommandOutcome::Cleared))
        }
        BreakDecision::Remove(target) => {
            context
                .state_mut()
                .voxels_mut()
                .set_block(target, AIR)
                .map_err(|_| logic_error())?;
            context
                .state_mut()
                .entities_mut()
                .remove_component::<BlockBreakSession>(request.player)
                .map_err(|_| logic_error())?;
            append_replication(context, target, AIR)?;
            Ok(InteractionResult::new(BlockCommandOutcome::Applied))
        }
        BreakDecision::Correct(target) => correction(context, target),
    }
}

fn apply_use_on(
    context: &mut RegionPhaseContext<'_>,
    request: BlockInteractionCommand,
    hit: BlockPos,
    direction: ferrite_foundation::direction::Direction,
    offsets: [f32; 3],
    interaction_allowed: bool,
    placement_state: BlockStateId,
) -> Result<InteractionResult, RegionLogicError> {
    if admitted_state(context, request, hit).is_err()
        || !valid_reconstructed_hit(hit, offsets[0], offsets[1], offsets[2])
    {
        return Ok(InteractionResult::new(BlockCommandOutcome::Rejected));
    }
    let adjacent = adjacent(hit, direction).map_err(|_| logic_error())?;
    let hit_state = current_state(context, hit)?;
    let target = if hit_state == AIR { hit } else { adjacent };
    if !interaction_allowed {
        return two_position_correction(context, hit, adjacent);
    }
    let target_state = match current_state(context, target) {
        Ok(state) => state,
        Err(_) => return correction(context, hit),
    };
    let outcome = if target_state == AIR && placement_state != AIR {
        context
            .state_mut()
            .voxels_mut()
            .set_block(target, placement_state)
            .map_err(|_| logic_error())?;
        append_replication(context, target, placement_state)?;
        BlockCommandOutcome::Applied
    } else {
        BlockCommandOutcome::Rejected
    };
    let corrections = [hit, adjacent]
        .into_iter()
        .filter_map(|position| {
            context
                .state()
                .view()
                .voxels()
                .block_state(position)
                .ok()
                .map(|state| AuthoritativeBlockUpdate { position, state })
        })
        .collect();
    Ok(InteractionResult {
        outcome,
        corrections,
    })
}

fn two_position_correction(
    context: &RegionPhaseContext<'_>,
    hit: BlockPos,
    adjacent: BlockPos,
) -> Result<InteractionResult, RegionLogicError> {
    let corrections = [hit, adjacent]
        .into_iter()
        .filter_map(|position| {
            context
                .state()
                .view()
                .voxels()
                .block_state(position)
                .ok()
                .map(|state| AuthoritativeBlockUpdate { position, state })
        })
        .collect();
    Ok(InteractionResult {
        outcome: BlockCommandOutcome::Rejected,
        corrections,
    })
}

fn admitted_state(
    context: &RegionPhaseContext<'_>,
    request: BlockInteractionCommand,
    position: BlockPos,
) -> Result<BlockStateId, ()> {
    if !within_block_reach(
        request.eye,
        position,
        request.interaction_range,
        PACKET_REACH_PADDING,
    ) {
        return Err(());
    }
    current_state(context, position).map_err(|_| ())
}

fn current_state(
    context: &RegionPhaseContext<'_>,
    position: BlockPos,
) -> Result<BlockStateId, RegionLogicError> {
    context
        .state()
        .view()
        .voxels()
        .block_state(position)
        .map_err(|_| logic_error())
}

fn correction(
    context: &RegionPhaseContext<'_>,
    position: BlockPos,
) -> Result<InteractionResult, RegionLogicError> {
    let state = current_state(context, position)?;
    Ok(InteractionResult {
        outcome: BlockCommandOutcome::Rejected,
        corrections: vec![AuthoritativeBlockUpdate { position, state }],
    })
}

fn append_replication(
    context: &mut RegionPhaseContext<'_>,
    position: BlockPos,
    state: BlockStateId,
) -> Result<(), RegionLogicError> {
    context
        .append_journal(
            JournalDomain::Replication,
            resource(BLOCK_UPDATE_PATH)?,
            encode_replication(AuthoritativeBlockUpdate { position, state }),
        )
        .map_err(|_| logic_error())?;
    Ok(())
}

fn is_block_command(command: &RegionCommand) -> bool {
    command.kind().namespace() == "ferrite" && command.kind().path() == BLOCK_INTERACTION_PATH
}

fn resource(path: &str) -> Result<ResourceId, RegionLogicError> {
    ResourceId::new("ferrite", path).map_err(|_| logic_error())
}

fn logic_error() -> RegionLogicError {
    RegionLogicError::new(
        ResourceId::new("ferrite", "block/region_logic")
            .expect("locked block logic identity is valid"),
    )
}

struct InteractionResult {
    outcome: BlockCommandOutcome,
    corrections: Vec<AuthoritativeBlockUpdate>,
}

impl InteractionResult {
    const fn new(outcome: BlockCommandOutcome) -> Self {
        Self {
            outcome,
            corrections: Vec::new(),
        }
    }
}
