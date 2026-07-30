//! Effective-AI phase selection and classic goal arbitration.

pub const MOVE: u8 = 1;
pub const LOOK: u8 = 2;
pub const JUMP: u8 = 4;
pub const TARGET: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorPhase {
    Full,
    Reduced,
}

#[must_use]
pub const fn selector_phase(tick_count: u32, entity_id: i32) -> SelectorPhase {
    if tick_count > 1 && (tick_count as i32).wrapping_add(entity_id) & 1 != 0 {
        SelectorPhase::Reduced
    } else {
        SelectorPhase::Full
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiStep {
    pub increment_no_action_time: bool,
    pub clear_sensing_cache: bool,
    pub target_selector: SelectorPhase,
    pub goal_selector: SelectorPhase,
    pub tick_navigation_before_custom_ai: bool,
    pub controls_order_move_look_jump: bool,
    pub refresh_passenger_flags: bool,
}

#[must_use]
pub const fn ai_step(tick_count: u32, entity_id: i32, effective_ai: bool) -> Option<AiStep> {
    if !effective_ai {
        return None;
    }
    let phase = selector_phase(tick_count, entity_id);
    Some(AiStep {
        increment_no_action_time: true,
        clear_sensing_cache: true,
        target_selector: phase,
        goal_selector: phase,
        tick_navigation_before_custom_ai: true,
        controls_order_move_look_jump: true,
        refresh_passenger_flags: tick_count.is_multiple_of(5),
    })
}

#[must_use]
pub const fn passenger_disabled_flags(controlling_passenger: bool, in_boat: bool) -> u8 {
    let passenger_flags = if controlling_passenger {
        MOVE | LOOK | JUMP
    } else {
        0
    };
    passenger_flags | if in_boat { JUMP } else { 0 }
}

#[must_use]
pub const fn adjusted_tick_delay(delay: u32, requires_every_tick: bool) -> u32 {
    if requires_every_tick {
        delay
    } else {
        let adjusted = delay.div_ceil(2);
        if adjusted == 0 { 1 } else { adjusted }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GoalState {
    pub priority: i32,
    pub flags: u8,
    pub running: bool,
    pub interruptible: bool,
    pub requires_every_tick: bool,
    pub can_continue: bool,
    pub can_use: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorOutcome {
    pub stopped_in_order: Vec<usize>,
    pub started_in_order: Vec<usize>,
    pub ticked_in_order: Vec<usize>,
}

#[must_use]
pub fn tick_selector(
    goals_in_insertion_order: &mut [GoalState],
    disabled_flags: u8,
    phase: SelectorPhase,
) -> SelectorOutcome {
    if matches!(phase, SelectorPhase::Reduced) {
        return SelectorOutcome {
            stopped_in_order: Vec::new(),
            started_in_order: Vec::new(),
            ticked_in_order: goals_in_insertion_order
                .iter()
                .enumerate()
                .filter_map(|(index, goal)| {
                    (goal.running && goal.requires_every_tick).then_some(index)
                })
                .collect(),
        };
    }

    let mut stopped = Vec::new();
    for (index, goal) in goals_in_insertion_order.iter_mut().enumerate() {
        if goal.running && (goal.flags & disabled_flags != 0 || !goal.can_continue) {
            goal.running = false;
            stopped.push(index);
        }
    }

    let mut holders = [None; 4];
    for (index, goal) in goals_in_insertion_order.iter().enumerate() {
        if goal.running {
            assign_holders(&mut holders, goal.flags, index);
        }
    }

    let mut started = Vec::new();
    for candidate_index in 0..goals_in_insertion_order.len() {
        let candidate = goals_in_insertion_order[candidate_index];
        if candidate.running
            || candidate.flags & disabled_flags != 0
            || !candidate.can_use
            || !holders_replaceable(&holders, candidate, goals_in_insertion_order)
        {
            continue;
        }
        for holder in conflicting_holders(&holders, candidate.flags) {
            if goals_in_insertion_order[holder].running {
                goals_in_insertion_order[holder].running = false;
                stopped.push(holder);
                clear_holder(&mut holders, holder);
            }
        }
        assign_holders(&mut holders, candidate.flags, candidate_index);
        goals_in_insertion_order[candidate_index].running = true;
        started.push(candidate_index);
    }

    SelectorOutcome {
        stopped_in_order: stopped,
        started_in_order: started,
        ticked_in_order: goals_in_insertion_order
            .iter()
            .enumerate()
            .filter_map(|(index, goal)| goal.running.then_some(index))
            .collect(),
    }
}

fn assign_holders(holders: &mut [Option<usize>; 4], flags: u8, index: usize) {
    for (bit, holder) in holders.iter_mut().enumerate() {
        if flags & (1 << bit) != 0 {
            *holder = Some(index);
        }
    }
}

fn clear_holder(holders: &mut [Option<usize>; 4], index: usize) {
    for holder in holders {
        if *holder == Some(index) {
            *holder = None;
        }
    }
}

fn conflicting_holders(holders: &[Option<usize>; 4], flags: u8) -> Vec<usize> {
    let mut result = Vec::new();
    for (bit, holder) in holders.iter().enumerate() {
        if flags & (1 << bit) != 0
            && let Some(index) = holder
            && !result.contains(index)
        {
            result.push(*index);
        }
    }
    result
}

fn holders_replaceable(
    holders: &[Option<usize>; 4],
    candidate: GoalState,
    goals: &[GoalState],
) -> bool {
    conflicting_holders(holders, candidate.flags)
        .into_iter()
        .all(|holder| goals[holder].interruptible && candidate.priority < goals[holder].priority)
}
