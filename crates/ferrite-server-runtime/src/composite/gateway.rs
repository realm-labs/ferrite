//! Formal-gateway adapter for the composite production Region runtime.

use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::state::PlayerSessionState;
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerError, LocalTickReport};
use ferrite_region_runtime::logic::{
    ImmediateEffectContext, RegionLogic, RegionLogicError, RegionPhaseContext, RegionPhaseOutput,
};
use ferrite_region_runtime::transfer::EntityTransfer;
use ferrite_simulation::command::RegionCommand;
use ferrite_simulation::tick::{GameTick, TickPhase};
use thiserror::Error;

use crate::composite::model::CompositeCommitReceipt;
use crate::composite::services::{
    CompositeProductionRegionRuntime, CompositeServiceAction, CompositeServiceCommand,
    CompositeServiceRuntimeError, CompositeServiceTickReport,
};
use crate::player::block::logic::apply_block_commands;
use crate::player::logic::{apply_player_commands, materialize_transferred_players};
use crate::player::router::{PlayerRegionRouteError, PlayerRegionRouter};
use crate::player_service::model::PlayerPersistentState;
use crate::session::router::{RegionCommandRouter, RegionRouteError};

/// The single Region route used by the formal Minecraft gateway.
///
/// `LocalRegionRunner` remains the deterministic executor for Region phases and
/// transfer delivery. Gameplay-service authority and continuity are committed by
/// the composite runtime owned by the logic adapter.
pub struct CompositeRegionRouter {
    runner: LocalRegionRunner,
    logic: CompositeGatewayLogic,
}

impl CompositeRegionRouter {
    pub fn new(
        runner: LocalRegionRunner,
        runtimes: impl IntoIterator<Item = CompositeProductionRegionRuntime>,
    ) -> Result<Self, CompositeGatewayError> {
        let logic = CompositeGatewayLogic::new(runtimes)?;
        if runner.len() != logic.regions.len()
            || logic.regions.keys().any(|key| runner.region(key).is_none())
        {
            return Err(CompositeGatewayError::RegionSetMismatch);
        }
        Ok(Self { runner, logic })
    }

    pub fn run_tick(
        &mut self,
        tick: GameTick,
    ) -> Result<CompositeGatewayTickReport, CompositeGatewayError> {
        self.logic.begin_tick();
        let local = match self.runner.run_tick(tick, &mut self.logic) {
            Ok(report) => report,
            Err(error) => {
                if let Some(failure) = self.logic.failure.take() {
                    return Err(CompositeGatewayError::Composite {
                        region: failure.region,
                        source: Box::new(failure.source),
                    });
                }
                return Err(error.into());
            }
        };
        let regions = self.logic.take_reports();
        if regions.len() != self.runner.len()
            || regions.values().any(|report| report.commit.tick != tick)
        {
            return Err(CompositeGatewayError::IncompleteCompositeTick(tick));
        }
        Ok(CompositeGatewayTickReport { local, regions })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.runner.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.runner.is_empty()
    }

    #[must_use]
    pub fn last_commit(&self, region: &SimulationRegionKey) -> Option<CompositeCommitReceipt> {
        self.logic.last_commits.get(region).copied()
    }

    #[must_use]
    pub fn player_is_owned(&self, region: &SimulationRegionKey, player: StableEntityId) -> bool {
        self.logic
            .regions
            .get(region)
            .is_some_and(|owned| owned.runtime.players().state(player).is_some())
    }
}

impl RegionCommandRouter for CompositeRegionRouter {
    fn route(&mut self, command: RegionCommand) -> Result<(), RegionRouteError> {
        self.runner.admit_command(command)?;
        Ok(())
    }
}

impl PlayerRegionRouter for CompositeRegionRouter {
    fn route_player_command(
        &mut self,
        command: RegionCommand,
    ) -> Result<(), PlayerRegionRouteError> {
        self.runner.admit_command(command)?;
        Ok(())
    }

    fn route_player_transfer(
        &mut self,
        transfer: EntityTransfer,
    ) -> Result<(), PlayerRegionRouteError> {
        self.runner.admit_transfer(transfer)?;
        Ok(())
    }

    fn activation_generation(
        &self,
        region: &SimulationRegionKey,
    ) -> Result<ActivationGeneration, PlayerRegionRouteError> {
        Ok(self.runner.activation_generation(region)?)
    }
}

#[derive(Debug)]
pub struct CompositeGatewayTickReport {
    local: LocalTickReport,
    regions: BTreeMap<SimulationRegionKey, CompositeServiceTickReport>,
}

impl CompositeGatewayTickReport {
    pub const fn local(&self) -> &LocalTickReport {
        &self.local
    }

    pub fn region(&self, key: &SimulationRegionKey) -> Option<&CompositeServiceTickReport> {
        self.regions.get(key)
    }

    pub fn regions(
        &self,
    ) -> impl Iterator<Item = (&SimulationRegionKey, &CompositeServiceTickReport)> {
        self.regions.iter()
    }
}

struct CompositeGatewayLogic {
    regions: BTreeMap<SimulationRegionKey, OwnedCompositeRegion>,
    reports: BTreeMap<SimulationRegionKey, CompositeServiceTickReport>,
    last_commits: BTreeMap<SimulationRegionKey, CompositeCommitReceipt>,
    failure: Option<CompositeFailure>,
}

struct OwnedCompositeRegion {
    runtime: CompositeProductionRegionRuntime,
    next_sequence: u64,
}

struct CompositeFailure {
    region: SimulationRegionKey,
    source: CompositeServiceRuntimeError,
}

impl CompositeGatewayLogic {
    fn new(
        runtimes: impl IntoIterator<Item = CompositeProductionRegionRuntime>,
    ) -> Result<Self, CompositeGatewayError> {
        let mut regions = BTreeMap::new();
        for runtime in runtimes {
            let key = runtime.coordinator().key().clone();
            if regions
                .insert(
                    key.clone(),
                    OwnedCompositeRegion {
                        runtime,
                        next_sequence: 1,
                    },
                )
                .is_some()
            {
                return Err(CompositeGatewayError::DuplicateRegion(key));
            }
        }
        if regions.is_empty() {
            return Err(CompositeGatewayError::NoRegions);
        }
        Ok(Self {
            regions,
            reports: BTreeMap::new(),
            last_commits: BTreeMap::new(),
            failure: None,
        })
    }

    fn begin_tick(&mut self) {
        self.reports.clear();
        self.failure = None;
    }

    fn take_reports(&mut self) -> BTreeMap<SimulationRegionKey, CompositeServiceTickReport> {
        std::mem::take(&mut self.reports)
    }

    fn synchronize_players(
        &mut self,
        context: &RegionPhaseContext<'_>,
    ) -> Result<(), CompositeServiceRuntimeError> {
        let key = context.key();
        let owned = self
            .regions
            .get_mut(key)
            .expect("gateway Region sets were validated at construction");
        let session_players = context
            .state()
            .view()
            .entities()
            .stable_ids()
            .filter(|player| {
                context
                    .state()
                    .view()
                    .entities()
                    .component::<PlayerSessionState>(*player)
                    .is_some()
            })
            .collect::<BTreeSet<_>>();
        let service_players = owned.runtime.players().players().collect::<BTreeSet<_>>();
        for player in service_players.difference(&session_players).copied() {
            owned.admit(
                context.tick(),
                CompositeServiceAction::LeavePlayer { player },
            )?;
        }
        for player in session_players.difference(&service_players).copied() {
            owned.admit(
                context.tick(),
                CompositeServiceAction::JoinPlayer {
                    player,
                    state: PlayerPersistentState::default(),
                },
            )?;
        }
        Ok(())
    }

    fn run_composite(&mut self, context: &RegionPhaseContext<'_>) -> Result<(), RegionLogicError> {
        let key = context.key().clone();
        let result = self
            .regions
            .get_mut(&key)
            .expect("gateway Region sets were validated at construction")
            .runtime
            .run_tick(context.tick(), game_time(context.tick()), usize::MAX);
        match result {
            Ok(report) => {
                self.last_commits.insert(key.clone(), report.commit);
                self.reports.insert(key, report);
                Ok(())
            }
            Err(source) => {
                self.failure = Some(CompositeFailure {
                    region: key,
                    source,
                });
                Err(logic_error())
            }
        }
    }
}

impl OwnedCompositeRegion {
    fn admit(
        &mut self,
        tick: GameTick,
        action: CompositeServiceAction,
    ) -> Result<(), CompositeServiceRuntimeError> {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.checked_add(1).ok_or({
            CompositeServiceRuntimeError::Coordinator(
                crate::composite::runtime::CompositeRuntimeError::SequenceExhausted,
            )
        })?;
        self.runtime
            .admit_command(CompositeServiceCommand::new(tick, sequence, action))
    }
}

impl RegionLogic for CompositeGatewayLogic {
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
            TickPhase::ReconcileBoundary => {
                materialize_transferred_players(&mut context)?;
                self.synchronize_players(&context).map_err(|source| {
                    self.failure = Some(CompositeFailure {
                        region: context.key().clone(),
                        source,
                    });
                    logic_error()
                })
            }
            TickPhase::Commit => self.run_composite(&context),
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

fn game_time(tick: GameTick) -> i64 {
    i64::try_from(tick.get()).unwrap_or(i64::MAX)
}

fn logic_error() -> RegionLogicError {
    RegionLogicError::new(
        ResourceId::new("ferrite", "composite/gateway_logic")
            .expect("static composite gateway identity is valid"),
    )
}

#[derive(Debug, Error)]
pub enum CompositeGatewayError {
    #[error("composite gateway requires at least one Region")]
    NoRegions,
    #[error("composite gateway contains duplicate Region {0:?}")]
    DuplicateRegion(SimulationRegionKey),
    #[error("local executor and composite authority Region sets differ")]
    RegionSetMismatch,
    #[error("composite gateway did not commit every Region at tick {0:?}")]
    IncompleteCompositeTick(GameTick),
    #[error("composite Region {region:?} failed")]
    Composite {
        region: SimulationRegionKey,
        #[source]
        source: Box<CompositeServiceRuntimeError>,
    },
    #[error(transparent)]
    Local(#[from] LocalRunnerError),
}
