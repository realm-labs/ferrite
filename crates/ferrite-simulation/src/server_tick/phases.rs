//! Source-ordered server and per-level tick phase plans.

pub const TIME_SYNC_PERIOD_TICKS: i32 = 20;
pub const STATUS_EXPIRE_NANOS: i64 = 5_000_000_000;
pub const LEVEL_SCHEDULED_TICK_LIMIT: usize = 65_536;
pub const LEVEL_EMPTY_ACTIVITY_CUTOFF: i32 = 300;
pub const TICK_TIME_RING_SIZE: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerChildStage {
    SuspendPlayerFlushing,
    TickCommandFunctions,
    TickClockManager,
    SynchronizeTime,
    RefreshEffectiveRespawn,
    TickLevel(usize),
    TickConnections,
    TickPlayerList,
    TickDebugSubscribers,
    TickGameTests,
    TickGui,
    SendChunksAndResumeFlushing,
    TickActivityMonitor,
}

pub fn server_child_order(
    run_game_elements: bool,
    admitted_tick_count: i32,
    level_count: usize,
) -> Vec<ServerChildStage> {
    let mut order = vec![
        ServerChildStage::SuspendPlayerFlushing,
        ServerChildStage::TickCommandFunctions,
    ];
    if run_game_elements {
        order.push(ServerChildStage::TickClockManager);
    }
    if admitted_tick_count % TIME_SYNC_PERIOD_TICKS == 0 {
        order.push(ServerChildStage::SynchronizeTime);
    }
    order.push(ServerChildStage::RefreshEffectiveRespawn);
    order.extend((0..level_count).map(ServerChildStage::TickLevel));
    order.extend([
        ServerChildStage::TickConnections,
        ServerChildStage::TickPlayerList,
        ServerChildStage::TickDebugSubscribers,
    ]);
    if run_game_elements {
        order.push(ServerChildStage::TickGameTests);
    }
    order.extend([
        ServerChildStage::TickGui,
        ServerChildStage::SendChunksAndResumeFlushing,
        ServerChildStage::TickActivityMonitor,
    ]);
    order
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelStage {
    InvalidateEnvironmentAttributeCache,
    TickWorldBorder,
    TickWeather,
    EvaluateSleepAndWake,
    UpdateSkyBrightness,
    TickTime { owns_game_time_increment: bool },
    DrainScheduledBlocks { maximum: usize },
    DrainScheduledFluids { maximum: usize },
    TickRaids,
    TickChunkSource,
    RunBlockEvents,
    ClearHandlingTick,
    TickDragonFight,
    TraverseEligibleEntities,
    ProcessBlockEntityTickers,
    TickPersistentEntitySections,
    SynchronizeDebugState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelPhaseInput {
    pub run_game_elements: bool,
    pub debug_level: bool,
    pub owns_game_time_increment: bool,
    pub empty_time: i32,
    pub has_active_chunk_tickets: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelPhasePlan {
    pub next_empty_time: i32,
    pub stages: Vec<LevelStage>,
}

pub fn level_phase_plan(input: LevelPhaseInput) -> LevelPhasePlan {
    let mut stages = vec![LevelStage::InvalidateEnvironmentAttributeCache];
    if input.run_game_elements {
        stages.extend([LevelStage::TickWorldBorder, LevelStage::TickWeather]);
    }
    stages.extend([
        LevelStage::EvaluateSleepAndWake,
        LevelStage::UpdateSkyBrightness,
    ]);
    if input.run_game_elements {
        stages.push(LevelStage::TickTime {
            owns_game_time_increment: input.owns_game_time_increment,
        });
    }
    if input.run_game_elements && !input.debug_level {
        stages.extend([
            LevelStage::DrainScheduledBlocks {
                maximum: LEVEL_SCHEDULED_TICK_LIMIT,
            },
            LevelStage::DrainScheduledFluids {
                maximum: LEVEL_SCHEDULED_TICK_LIMIT,
            },
        ]);
    }
    if input.run_game_elements {
        stages.push(LevelStage::TickRaids);
    }
    stages.push(LevelStage::TickChunkSource);
    if input.run_game_elements {
        stages.push(LevelStage::RunBlockEvents);
    }
    stages.push(LevelStage::ClearHandlingTick);

    let next_empty_time = next_empty_time(
        input.empty_time,
        input.has_active_chunk_tickets,
        input.run_game_elements,
    );
    if next_empty_time < LEVEL_EMPTY_ACTIVITY_CUTOFF {
        if input.run_game_elements {
            stages.push(LevelStage::TickDragonFight);
        }
        stages.extend([
            LevelStage::TraverseEligibleEntities,
            LevelStage::ProcessBlockEntityTickers,
        ]);
    }
    stages.extend([
        LevelStage::TickPersistentEntitySections,
        LevelStage::SynchronizeDebugState,
    ]);
    LevelPhasePlan {
        next_empty_time,
        stages,
    }
}

pub const fn next_empty_time(
    current: i32,
    has_active_chunk_tickets: bool,
    run_game_elements: bool,
) -> i32 {
    let current = if has_active_chunk_tickets { 0 } else { current };
    if run_game_elements {
        current.wrapping_add(1)
    } else {
        current
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityTickInput {
    pub removed: bool,
    pub is_player: bool,
    pub player_passengers: u32,
    pub in_entity_ticking_range: bool,
    pub has_vehicle: bool,
    pub vehicle_link_valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityTickOutcome {
    SkipRemoved,
    LeaveForVehicleTraversal,
    DetachVehicleAndTick,
    TickRoot,
    SkipFrozen,
    SkipOutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityTickPlan {
    pub outcome: EntityTickOutcome,
    pub check_despawn: bool,
}

pub const fn entity_tick_plan(run_game_elements: bool, input: EntityTickInput) -> EntityTickPlan {
    if input.removed {
        return EntityTickPlan {
            outcome: EntityTickOutcome::SkipRemoved,
            check_despawn: false,
        };
    }
    let frozen = !run_game_elements && !input.is_player && input.player_passengers == 0;
    if frozen {
        return EntityTickPlan {
            outcome: EntityTickOutcome::SkipFrozen,
            check_despawn: false,
        };
    }
    if !input.is_player && !input.in_entity_ticking_range {
        return EntityTickPlan {
            outcome: EntityTickOutcome::SkipOutOfRange,
            check_despawn: true,
        };
    }
    let outcome = match (input.has_vehicle, input.vehicle_link_valid) {
        (true, true) => EntityTickOutcome::LeaveForVehicleTraversal,
        (true, false) => EntityTickOutcome::DetachVehicleAndTick,
        (false, _) => EntityTickOutcome::TickRoot,
    };
    EntityTickPlan {
        outcome,
        check_despawn: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockEntityTickOutcome {
    RemoveInvalidTicker,
    KeepWithoutCallback,
    InvokeCallback,
}

pub const fn block_entity_tick_outcome(
    ticker_removed_or_invalid: bool,
    run_game_elements: bool,
    position_should_tick: bool,
) -> BlockEntityTickOutcome {
    if ticker_removed_or_invalid {
        BlockEntityTickOutcome::RemoveInvalidTicker
    } else if run_game_elements && position_should_tick {
        BlockEntityTickOutcome::InvokeCallback
    } else {
        BlockEntityTickOutcome::KeepWithoutCallback
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SleepTransitionInput {
    pub enough_sleeping: bool,
    pub enough_deep_sleeping: bool,
    pub advance_time: bool,
    pub default_clock_present: bool,
    pub advance_weather: bool,
    pub raining: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SleepTransitionPlan {
    pub move_default_clock_to_wake_marker: bool,
    pub wake_all_players: bool,
    pub reset_weather_cycle: bool,
}

pub const fn sleep_transition(input: SleepTransitionInput) -> SleepTransitionPlan {
    let admitted = input.enough_sleeping && input.enough_deep_sleeping;
    SleepTransitionPlan {
        move_default_clock_to_wake_marker: admitted
            && input.advance_time
            && input.default_clock_present,
        wake_all_players: admitted,
        reset_weather_cycle: admitted && input.advance_weather && input.raining,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockManagerPlan {
    pub invoke_manager: bool,
    pub advance_registered_clocks: bool,
}

pub const fn clock_manager_plan(
    run_game_elements: bool,
    overworld_advance_time: bool,
) -> ClockManagerPlan {
    ClockManagerPlan {
        invoke_manager: run_game_elements,
        advance_registered_clocks: run_game_elements && overworld_advance_time,
    }
}

pub const fn increment_shared_game_time(current: i64, owns_increment: bool) -> i64 {
    if owns_increment {
        current.wrapping_add(1)
    } else {
        current
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaseTickBookkeeping {
    pub tick_count: i32,
    pub ticks_until_autosave: i32,
    pub status_refresh: bool,
    pub auto_save: bool,
    pub ring_index: i32,
}

pub fn advance_base_tick_bookkeeping(
    tick_count: i32,
    ticks_until_autosave: i32,
    tick_start_nanos: i64,
    last_status_nanos: i64,
) -> BaseTickBookkeeping {
    let tick_count = tick_count.wrapping_add(1);
    let ticks_until_autosave = ticks_until_autosave.wrapping_sub(1);
    BaseTickBookkeeping {
        tick_count,
        ticks_until_autosave,
        status_refresh: tick_start_nanos.wrapping_sub(last_status_nanos) >= STATUS_EXPIRE_NANOS,
        auto_save: ticks_until_autosave <= 0,
        ring_index: tick_count % TICK_TIME_RING_SIZE,
    }
}
