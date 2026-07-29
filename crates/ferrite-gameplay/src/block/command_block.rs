//! Command-block carrier scheduling, execution, and chain semantics.

pub const SCHEDULE_DELAY: u64 = 1;
pub const MINECART_THROTTLE: u32 = 4;
pub const LAST_EXECUTION_SENTINEL: i64 = -1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandMode {
    Redstone,
    Auto,
    Sequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandState {
    pub command: String,
    pub success_count: u32,
    pub last_output: Option<String>,
    pub track_output: bool,
    pub update_last_execution: bool,
    pub last_execution: i64,
}

impl Default for CommandState {
    fn default() -> Self {
        Self {
            command: String::new(),
            success_count: 0,
            last_output: None,
            track_output: true,
            update_last_execution: true,
            last_execution: LAST_EXECUTION_SENTINEL,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionResult {
    SameTickSuppressed,
    Searge,
    Completed { dispatched: bool },
}

impl CommandState {
    pub fn set_command(&mut self, command: impl Into<String>) {
        self.command = command.into();
        self.success_count = 0;
    }

    pub fn perform(
        &mut self,
        game_time: i64,
        command_blocks_work: bool,
        successful_callbacks: u32,
    ) -> ExecutionResult {
        if self.last_execution == game_time {
            return ExecutionResult::SameTickSuppressed;
        }
        if self.command.eq_ignore_ascii_case("searge") {
            self.last_output = Some("#itzlipofutzli".to_owned());
            self.success_count = 1;
            return ExecutionResult::Searge;
        }

        self.success_count = 0;
        let dispatched = command_blocks_work && !self.command.is_empty();
        if dispatched {
            self.last_output = None;
            self.success_count = successful_callbacks;
        }
        self.last_execution = if self.update_last_execution {
            game_time
        } else {
            LAST_EXECUTION_SENTINEL
        };
        ExecutionResult::Completed { dispatched }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriggerDecision {
    pub powered: bool,
    pub snapshot_condition: bool,
    pub schedule_delay: Option<u64>,
}

pub const fn neighbor_edge(
    was_powered: bool,
    is_powered: bool,
    automatic: bool,
    mode: CommandMode,
    predecessor_succeeded: bool,
) -> TriggerDecision {
    let rising = !was_powered && is_powered;
    let schedule = rising && !automatic && !matches!(mode, CommandMode::Sequence);
    TriggerDecision {
        powered: is_powered,
        snapshot_condition: predecessor_succeeded,
        schedule_delay: if schedule { Some(SCHEDULE_DELAY) } else { None },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DueResult {
    pub execute: bool,
    pub clear_success: bool,
    pub next_condition: bool,
    pub reschedule: bool,
    pub update_comparator: bool,
}

pub const fn due_tick(
    mode: CommandMode,
    captured_condition: bool,
    next_predecessor_succeeded: bool,
    powered_or_automatic: bool,
) -> DueResult {
    DueResult {
        execute: !matches!(mode, CommandMode::Sequence) && captured_condition,
        clear_success: !matches!(mode, CommandMode::Sequence) && !captured_condition,
        next_condition: if matches!(mode, CommandMode::Auto) {
            next_predecessor_succeeded
        } else {
            captured_condition
        },
        reschedule: matches!(mode, CommandMode::Auto) && powered_or_automatic,
        update_comparator: true,
    }
}

pub const fn minecart_admitted(powered: bool, tick_count: u32, last_activated: u32) -> bool {
    powered && tick_count.saturating_sub(last_activated) >= MINECART_THROTTLE
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainStep {
    Terminate,
    SkipAndContinue,
    ExecuteAndContinue,
    ClearAndContinue,
}

pub const fn chain_step(
    is_chain_state: bool,
    entity_mode: CommandMode,
    powered_or_automatic: bool,
    condition_met: bool,
) -> ChainStep {
    if !is_chain_state || !matches!(entity_mode, CommandMode::Sequence) {
        ChainStep::Terminate
    } else if !powered_or_automatic {
        ChainStep::SkipAndContinue
    } else if condition_met {
        ChainStep::ExecuteAndContinue
    } else {
        ChainStep::ClearAndContinue
    }
}
