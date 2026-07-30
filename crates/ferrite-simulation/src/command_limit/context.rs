//! Outer/nested execution-context snapshots and source-ordered action draining.

use std::collections::VecDeque;

pub const DEFAULT_COMMAND_LIMIT: i32 = 65_536;
pub const DEFAULT_FORK_LIMIT: i32 = 65_536;
pub const MIN_CONTEXT_COMMAND_LIMIT: i32 = 1;
pub const MAX_QUEUE_DEPTH: usize = 10_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextLimits {
    pub command_limit: i32,
    pub fork_limit: i32,
}

impl ContextLimits {
    pub const fn snapshot(max_command_sequence_length: i32, max_command_forks: i32) -> Self {
        Self {
            command_limit: if max_command_sequence_length < MIN_CONTEXT_COMMAND_LIMIT {
                MIN_CONTEXT_COMMAND_LIMIT
            } else {
                max_command_sequence_length
            },
            fork_limit: max_command_forks,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextAdmission {
    CreateOuter(ContextLimits),
    ReuseExisting,
}

pub const fn context_admission(
    outer_context_exists: bool,
    live_sequence_limit: i32,
    live_fork_limit: i32,
) -> ContextAdmission {
    if outer_context_exists {
        ContextAdmission::ReuseExisting
    } else {
        ContextAdmission::CreateOuter(ContextLimits::snapshot(
            live_sequence_limit,
            live_fork_limit,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomaticCost {
    RedirectModifier,
    CallFunction,
    ExecuteCommand,
    None,
}

impl AutomaticCost {
    const fn is_charged(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedAction<A> {
    pub frame_depth: i32,
    pub automatic_cost: AutomaticCost,
    pub payload: A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueAdmission {
    Admit,
    OverflowAndClear,
    AlreadyOverflowed,
}

pub const fn queue_admission(
    new_top_size: usize,
    deque_size: usize,
    already_overflowed: bool,
) -> QueueAdmission {
    if already_overflowed {
        QueueAdmission::AlreadyOverflowed
    } else if new_top_size.saturating_add(deque_size) > MAX_QUEUE_DEPTH {
        QueueAdmission::OverflowAndClear
    } else {
        QueueAdmission::Admit
    }
}

#[derive(Debug)]
pub struct ExecutionContext<A> {
    limits: ContextLimits,
    command_quota: i32,
    costs_debited: usize,
    queue_overflow: bool,
    command_queue: VecDeque<QueuedAction<A>>,
    new_top_commands: Vec<QueuedAction<A>>,
    current_frame_depth: i32,
}

impl<A> ExecutionContext<A> {
    pub fn new(limits: ContextLimits) -> Self {
        Self {
            command_quota: limits.command_limit,
            costs_debited: 0,
            limits,
            queue_overflow: false,
            command_queue: VecDeque::new(),
            new_top_commands: Vec::new(),
            current_frame_depth: 0,
        }
    }

    pub const fn limits(&self) -> ContextLimits {
        self.limits
    }

    pub const fn remaining_quota(&self) -> i32 {
        self.command_quota
    }

    pub fn increment_cost(&mut self) {
        self.command_quota = self.command_quota.wrapping_sub(1);
        self.costs_debited = self.costs_debited.wrapping_add(1);
    }

    pub const fn queue_overflowed(&self) -> bool {
        self.queue_overflow
    }

    pub const fn current_frame_depth(&self) -> i32 {
        self.current_frame_depth
    }

    pub fn queued_len(&self) -> usize {
        self.command_queue.len() + self.new_top_commands.len()
    }

    pub fn queue_next(&mut self, action: QueuedAction<A>) -> QueueAdmission {
        let admission = queue_admission(
            self.new_top_commands.len(),
            self.command_queue.len(),
            self.queue_overflow,
        );
        match admission {
            QueueAdmission::Admit => self.new_top_commands.push(action),
            QueueAdmission::OverflowAndClear => {
                self.queue_overflow = true;
                self.new_top_commands.clear();
                self.command_queue.clear();
            }
            QueueAdmission::AlreadyOverflowed => {}
        }
        admission
    }

    pub fn discard_at_depth_or_higher(&mut self, depth: i32) -> usize {
        let mut discarded = 0;
        while self
            .command_queue
            .front()
            .is_some_and(|entry| entry.frame_depth >= depth)
        {
            self.command_queue.pop_front();
            discarded += 1;
        }
        discarded
    }

    pub fn run(&mut self, mut execute: impl FnMut(A, &mut ExecutionContext<A>)) -> DrainReport {
        self.push_new_commands();
        let mut executed_actions = 0;
        let starting_costs = self.costs_debited;
        loop {
            if self.command_quota <= 0 {
                self.current_frame_depth = 0;
                return DrainReport {
                    stop: DrainStop::CommandLimit,
                    executed_actions,
                    charged_actions: self.costs_debited.wrapping_sub(starting_costs),
                    abandoned_actions: self.queued_len(),
                    overflow_was_logged: false,
                };
            }
            let Some(action) = self.command_queue.pop_front() else {
                return DrainReport {
                    stop: DrainStop::QueueEmpty,
                    executed_actions,
                    charged_actions: self.costs_debited.wrapping_sub(starting_costs),
                    abandoned_actions: 0,
                    overflow_was_logged: false,
                };
            };
            self.current_frame_depth = action.frame_depth;
            if action.automatic_cost.is_charged() {
                self.increment_cost();
            }
            executed_actions += 1;
            execute(action.payload, self);
            if self.queue_overflow {
                self.current_frame_depth = 0;
                return DrainReport {
                    stop: DrainStop::QueueOverflow,
                    executed_actions,
                    charged_actions: self.costs_debited.wrapping_sub(starting_costs),
                    abandoned_actions: 0,
                    overflow_was_logged: true,
                };
            }
            self.push_new_commands();
        }
    }

    fn push_new_commands(&mut self) {
        for action in self.new_top_commands.drain(..).rev() {
            self.command_queue.push_front(action);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainStop {
    QueueEmpty,
    CommandLimit,
    QueueOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrainReport {
    pub stop: DrainStop,
    pub executed_actions: usize,
    pub charged_actions: usize,
    pub abandoned_actions: usize,
    pub overflow_was_logged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextLifecycleStage {
    SnapshotRulesAndInstallThreadLocal,
    InvokeInitialConsumer,
    DrainSynchronously,
    CloseTracer,
    ClearThreadLocal,
}

pub const OUTER_CONTEXT_LIFECYCLE: [ContextLifecycleStage; 5] = [
    ContextLifecycleStage::SnapshotRulesAndInstallThreadLocal,
    ContextLifecycleStage::InvokeInitialConsumer,
    ContextLifecycleStage::DrainSynchronously,
    ContextLifecycleStage::CloseTracer,
    ContextLifecycleStage::ClearThreadLocal,
];
