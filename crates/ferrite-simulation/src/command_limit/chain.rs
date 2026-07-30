//! Independent command-block chain traversal counter and warning semantics.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainCell {
    pub is_chain_command_block: bool,
    pub has_command_block_entity: bool,
    pub sequence_mode: bool,
    pub powered: bool,
    pub automatic: bool,
    pub condition_met: bool,
    pub conditional: bool,
    pub command_succeeded: bool,
    pub next_facing: Direction,
}

impl ChainCell {
    pub const fn terminating(next_facing: Direction) -> Self {
        Self {
            is_chain_command_block: false,
            has_command_block_entity: false,
            sequence_mode: false,
            powered: false,
            automatic: false,
            condition_met: false,
            conditional: false,
            command_succeeded: false,
            next_facing,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainVisitOutcome {
    TerminatingPosition,
    SkippedUnpowered,
    ConditionFailed { success_count_reset: bool },
    Executed { comparator_updated: bool },
    CommandReturnedFalse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainVisit {
    pub position: BlockPos,
    pub incoming_facing: Direction,
    pub outcome: ChainVisitOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainPlan {
    pub initial_limit: i32,
    pub remaining_counter: i32,
    pub initiator_counted: bool,
    pub visits: Vec<ChainVisit>,
    pub warning_limit: Option<i32>,
}

pub fn execute_chain(
    start: BlockPos,
    initial_facing: Direction,
    initial_live_limit: i32,
    mut inspect: impl FnMut(BlockPos, Direction) -> ChainCell,
    warning_live_limit: i32,
) -> ChainPlan {
    let mut position = start;
    let mut facing = initial_facing;
    let mut remaining = initial_live_limit;
    let mut visits = Vec::new();

    loop {
        let before_decrement = remaining;
        remaining = remaining.wrapping_sub(1);
        if before_decrement <= 0 {
            break;
        }
        position = wrapping_offset(position, facing);
        let cell = inspect(position, facing);
        if !cell.is_chain_command_block || !cell.has_command_block_entity || !cell.sequence_mode {
            visits.push(ChainVisit {
                position,
                incoming_facing: facing,
                outcome: ChainVisitOutcome::TerminatingPosition,
            });
            break;
        }
        let outcome = if !cell.powered && !cell.automatic {
            ChainVisitOutcome::SkippedUnpowered
        } else if !cell.condition_met {
            ChainVisitOutcome::ConditionFailed {
                success_count_reset: cell.conditional,
            }
        } else if !cell.command_succeeded {
            visits.push(ChainVisit {
                position,
                incoming_facing: facing,
                outcome: ChainVisitOutcome::CommandReturnedFalse,
            });
            break;
        } else {
            ChainVisitOutcome::Executed {
                comparator_updated: true,
            }
        };
        visits.push(ChainVisit {
            position,
            incoming_facing: facing,
            outcome,
        });
        facing = cell.next_facing;
    }

    ChainPlan {
        initial_limit: initial_live_limit,
        remaining_counter: remaining,
        initiator_counted: false,
        visits,
        warning_limit: (remaining <= 0).then_some(warning_live_limit.max(0)),
    }
}

fn wrapping_offset(position: BlockPos, direction: Direction) -> BlockPos {
    let [x, y, z] = direction.step();
    BlockPos::new(
        position.x.wrapping_add(x),
        position.y.wrapping_add(y),
        position.z.wrapping_add(z),
    )
}
