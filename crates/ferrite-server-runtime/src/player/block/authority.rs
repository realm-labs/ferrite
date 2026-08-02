//! Formal block interaction against the composite world-service authority.

use std::collections::BTreeMap;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::identity::StableEntityId;
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::block::breaking::{
    BlockBreakSession, BreakAction, BreakDecision, decide_break,
};
use ferrite_gameplay::block::targeting::{
    PACKET_REACH_PADDING, adjacent, valid_reconstructed_hit, within_block_reach,
};
use ferrite_region_runtime::logic::RegionPhaseContext;
use ferrite_simulation::command::{CommandSource, RegionCommand};
use ferrite_simulation::journal::JournalDomain;
use ferrite_simulation::pipeline::PipelineError;
use ferrite_world::id::BlockStateId;
use thiserror::Error;

use crate::player::block::command::{
    BLOCK_INTERACTION_PATH, BLOCK_RESULT_PATH, BlockCommandError, BlockIntent,
    BlockInteractionCommand,
};
use crate::player::block::replication::{
    AuthoritativeBlockUpdate, BlockCommandOutcome, BlockCommandResult,
};

const AIR: BlockStateId = BlockStateId::new(0);

/// Read and staged-write access to the one formal world authority.
pub(crate) trait BlockAuthority {
    fn block_state(
        &self,
        position: BlockPos,
    ) -> Result<Option<BlockStateId>, AuthoritativeBlockError>;

    /// Returns `false` when the target has no mutable authoritative chunk.
    fn stage_block(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
    ) -> Result<bool, AuthoritativeBlockError>;
}

/// Transient interaction state that is independent of any target Region's ECS shadow.
#[derive(Debug, Default)]
pub(crate) struct AuthoritativeBlockInteractions {
    break_sessions: BTreeMap<StableEntityId, BlockBreakSession>,
}

impl AuthoritativeBlockInteractions {
    pub(crate) fn apply_commands(
        &mut self,
        context: &mut RegionPhaseContext<'_>,
        authority: &mut impl BlockAuthority,
    ) -> Result<(), AuthoritativeBlockError> {
        let commands = context
            .commands()
            .iter()
            .filter(|command| is_block_command(command))
            .cloned()
            .collect::<Vec<_>>();
        for command in commands {
            let request = BlockInteractionCommand::decode(command.payload())?;
            let CommandSource::Player(source) = command.source() else {
                return Err(AuthoritativeBlockError::InvalidSource {
                    sequence: command.sequence(),
                });
            };
            if *source != request.player {
                return Err(AuthoritativeBlockError::SourceMismatch {
                    sequence: command.sequence(),
                    source_player: *source,
                    payload: request.player,
                });
            }
            let result = self.apply_command(authority, request)?;
            append_result(context, &command, request.player, result)?;
        }
        Ok(())
    }

    fn apply_command(
        &mut self,
        authority: &mut impl BlockAuthority,
        request: BlockInteractionCommand,
    ) -> Result<InteractionResult, AuthoritativeBlockError> {
        match request.intent {
            BlockIntent::StartDestroy { position } => {
                self.apply_break(authority, request, position, BreakAction::Start)
            }
            BlockIntent::AbortDestroy { position } => {
                self.apply_break(authority, request, position, BreakAction::Abort)
            }
            BlockIntent::StopDestroy { position } => {
                self.apply_break(authority, request, position, BreakAction::Stop)
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
            } => self.apply_use_on(
                authority,
                request,
                UseOnInteraction {
                    hit: position,
                    direction,
                    offsets: [offset_x, offset_y, offset_z],
                    interaction_allowed,
                    placement_state,
                },
            ),
        }
    }

    fn apply_break(
        &mut self,
        authority: &mut impl BlockAuthority,
        request: BlockInteractionCommand,
        position: BlockPos,
        action: BreakAction,
    ) -> Result<InteractionResult, AuthoritativeBlockError> {
        let Some(current) = admitted_state(authority, request, position)? else {
            return Ok(InteractionResult::new(BlockCommandOutcome::Rejected));
        };
        let active = self.break_sessions.get(&request.player).copied();
        match decide_break(action, active, position, current, AIR) {
            BreakDecision::Track(session) => {
                self.break_sessions.insert(request.player, session);
                Ok(InteractionResult::new(BlockCommandOutcome::Tracking))
            }
            BreakDecision::Clear => {
                self.break_sessions.remove(&request.player);
                Ok(InteractionResult::new(BlockCommandOutcome::Cleared))
            }
            BreakDecision::Remove(target) => {
                if !authority.stage_block(target, AIR)? {
                    return correction(authority, target);
                }
                self.break_sessions.remove(&request.player);
                Ok(InteractionResult::new(BlockCommandOutcome::Applied))
            }
            BreakDecision::Correct(target) => correction(authority, target),
        }
    }

    fn apply_use_on(
        &mut self,
        authority: &mut impl BlockAuthority,
        request: BlockInteractionCommand,
        interaction: UseOnInteraction,
    ) -> Result<InteractionResult, AuthoritativeBlockError> {
        let UseOnInteraction {
            hit,
            direction,
            offsets,
            interaction_allowed,
            placement_state,
        } = interaction;
        let Some(hit_state) = admitted_state(authority, request, hit)? else {
            return Ok(InteractionResult::new(BlockCommandOutcome::Rejected));
        };
        if !valid_reconstructed_hit(hit, offsets[0], offsets[1], offsets[2]) {
            return Ok(InteractionResult::new(BlockCommandOutcome::Rejected));
        }
        let Ok(adjacent) = adjacent(hit, direction) else {
            return correction(authority, hit);
        };
        let target = if hit_state == AIR { hit } else { adjacent };
        if !interaction_allowed {
            return two_position_correction(authority, hit, adjacent);
        }
        let Some(target_state) = authority.block_state(target)? else {
            return correction(authority, hit);
        };
        let outcome = if target_state == AIR
            && placement_state != AIR
            && authority.stage_block(target, placement_state)?
        {
            BlockCommandOutcome::Applied
        } else {
            BlockCommandOutcome::Rejected
        };
        let corrections = authoritative_updates(authority, [hit, adjacent])?;
        Ok(InteractionResult {
            outcome,
            corrections,
        })
    }
}

fn append_result(
    context: &mut RegionPhaseContext<'_>,
    command: &RegionCommand,
    player: StableEntityId,
    result: InteractionResult,
) -> Result<(), AuthoritativeBlockError> {
    let result = BlockCommandResult {
        player,
        command_sequence: command.sequence(),
        outcome: result.outcome,
        corrections: result.corrections,
    };
    context
        .append_journal(
            JournalDomain::Mutation,
            ResourceId::new("ferrite", BLOCK_RESULT_PATH)
                .expect("locked block-result identity is valid"),
            result.encode(),
        )
        .map_err(AuthoritativeBlockError::JournalAppend)?;
    Ok(())
}

fn admitted_state(
    authority: &impl BlockAuthority,
    request: BlockInteractionCommand,
    position: BlockPos,
) -> Result<Option<BlockStateId>, AuthoritativeBlockError> {
    if !within_block_reach(
        request.eye,
        position,
        request.interaction_range,
        PACKET_REACH_PADDING,
    ) {
        return Ok(None);
    }
    authority.block_state(position)
}

fn correction(
    authority: &impl BlockAuthority,
    position: BlockPos,
) -> Result<InteractionResult, AuthoritativeBlockError> {
    let corrections = authoritative_updates(authority, [position])?;
    Ok(InteractionResult {
        outcome: BlockCommandOutcome::Rejected,
        corrections,
    })
}

fn two_position_correction(
    authority: &impl BlockAuthority,
    hit: BlockPos,
    adjacent: BlockPos,
) -> Result<InteractionResult, AuthoritativeBlockError> {
    Ok(InteractionResult {
        outcome: BlockCommandOutcome::Rejected,
        corrections: authoritative_updates(authority, [hit, adjacent])?,
    })
}

fn authoritative_updates<const N: usize>(
    authority: &impl BlockAuthority,
    positions: [BlockPos; N],
) -> Result<Vec<AuthoritativeBlockUpdate>, AuthoritativeBlockError> {
    positions
        .into_iter()
        .filter_map(|position| match authority.block_state(position) {
            Ok(Some(state)) => Some(Ok(AuthoritativeBlockUpdate { position, state })),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn is_block_command(command: &RegionCommand) -> bool {
    command.kind().namespace() == "ferrite" && command.kind().path() == BLOCK_INTERACTION_PATH
}

struct InteractionResult {
    outcome: BlockCommandOutcome,
    corrections: Vec<AuthoritativeBlockUpdate>,
}

struct UseOnInteraction {
    hit: BlockPos,
    direction: ferrite_foundation::direction::Direction,
    offsets: [f32; 3],
    interaction_allowed: bool,
    placement_state: BlockStateId,
}

impl InteractionResult {
    const fn new(outcome: BlockCommandOutcome) -> Self {
        Self {
            outcome,
            corrections: Vec::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AuthoritativeBlockError {
    #[error(transparent)]
    Command(#[from] BlockCommandError),
    #[error("block interaction command {sequence} does not have a player source")]
    InvalidSource { sequence: u64 },
    #[error(
        "block interaction command {sequence} source {source_player:?} differs from payload player {payload:?}"
    )]
    SourceMismatch {
        sequence: u64,
        source_player: StableEntityId,
        payload: StableEntityId,
    },
    #[error("authoritative block interaction found duplicate chunk ownership at {position:?}")]
    DuplicateChunkOwnership { position: BlockPos },
    #[error("failed to append the authoritative block interaction result")]
    JournalAppend(#[source] PipelineError),
}
