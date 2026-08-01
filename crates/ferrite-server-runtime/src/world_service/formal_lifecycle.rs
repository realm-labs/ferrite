//! Bounded ticket and generation orchestration for the formal local world.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::thread::{self, JoinHandle};

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{DimensionId, WorldId};
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_simulation::tick::GameTick;
use ferrite_world::generation::status::ChunkStatus;
use thiserror::Error;

use crate::chunk::ticket::{ChunkTicket, ChunkTicketBook, ChunkTicketError};
use crate::composite::gateway::{CompositeGatewayError, CompositeRegionRouter};
use crate::world_service::dimension::{DimensionRuntimeError, FormalDimensionGenerator};
use crate::world_service::model::{
    ChunkActivity, ChunkLifecycle, GenerationRequest, GenerationResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FormalChunkLifecycleConfig {
    pub(crate) maximum_tickets: usize,
    pub(crate) maximum_generation_in_flight: usize,
    pub(crate) maximum_generation_results_per_tick: usize,
    pub(crate) maximum_lifecycle_actions_per_tick: usize,
    pub(crate) maximum_events_per_region_per_tick: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FormalChunkLifecycleReport {
    pub(crate) tick: GameTick,
    pub(crate) tickets: usize,
    pub(crate) generation_results: usize,
    pub(crate) generation_requests: usize,
    pub(crate) lifecycle_actions: usize,
    pub(crate) events: usize,
    pub(crate) generation_in_flight: usize,
}

pub(crate) struct FormalChunkLifecycle {
    world: WorldId,
    dimension: DimensionId,
    mapping: RegionMapping,
    config: FormalChunkLifecycleConfig,
    tickets: ChunkTicketBook,
    generation_worker: FormalGenerationWorker,
    generation_results: VecDeque<GenerationResult>,
}

struct FormalGenerationWorker {
    requests: Option<SyncSender<GenerationRequest>>,
    results: Receiver<Result<GenerationResult, DimensionRuntimeError>>,
    thread: Option<JoinHandle<()>>,
}

impl FormalChunkLifecycle {
    pub(crate) fn new(
        world: WorldId,
        dimension: DimensionId,
        mapping: RegionMapping,
        seed: i64,
        config: FormalChunkLifecycleConfig,
    ) -> Result<Self, FormalChunkLifecycleError> {
        if config.maximum_generation_in_flight == 0
            || config.maximum_generation_results_per_tick == 0
            || config.maximum_lifecycle_actions_per_tick == 0
            || config.maximum_events_per_region_per_tick == 0
        {
            return Err(FormalChunkLifecycleError::ZeroCapacity);
        }
        let generation_worker =
            FormalGenerationWorker::new(config.maximum_generation_in_flight, &dimension, seed)?;
        Ok(Self {
            world,
            dimension,
            mapping,
            tickets: ChunkTicketBook::new(config.maximum_tickets)?,
            generation_worker,
            generation_results: VecDeque::new(),
            config,
        })
    }

    pub(crate) fn drive(
        &mut self,
        tick: GameTick,
        tickets: impl IntoIterator<Item = ChunkTicket>,
        router: &mut CompositeRegionRouter,
    ) -> Result<FormalChunkLifecycleReport, FormalChunkLifecycleError> {
        let mut events = router
            .take_world_events(self.config.maximum_events_per_region_per_tick)
            .len();
        self.replace_tickets(tickets)?;
        let mut lifecycle_actions = 0;
        let mut generation_requests = 0;
        let demanded = self
            .tickets
            .tickets()
            .map(|ticket| ticket.position)
            .collect::<BTreeSet<_>>();
        let mut loaded = loaded_chunks(router, self.world, &self.dimension)?;

        for position in &demanded {
            if lifecycle_actions == self.config.maximum_lifecycle_actions_per_tick {
                break;
            }
            let region = self.region_for(*position);
            match loaded.get(position) {
                Some((actual, lifecycle)) if actual != &region => {
                    return Err(FormalChunkLifecycleError::WrongChunkOwner(*position));
                }
                Some((_, lifecycle)) if lifecycle.pending_unload.is_some() => {
                    router.demand_world_chunk(&region, *position)?;
                    lifecycle_actions += 1;
                }
                Some(_) => {}
                None => {
                    router.demand_world_chunk(&region, *position)?;
                    lifecycle_actions += 1;
                }
            }
        }
        loaded = loaded_chunks(router, self.world, &self.dimension)?;

        for position in &demanded {
            if lifecycle_actions == self.config.maximum_lifecycle_actions_per_tick {
                break;
            }
            let region = self.region_for(*position);
            let Some((actual, lifecycle)) = loaded.get(position).cloned() else {
                continue;
            };
            if actual != region {
                return Err(FormalChunkLifecycleError::WrongChunkOwner(*position));
            }
            if lifecycle.status == ChunkStatus::Full {
                let target = activation_activity(self.tickets.activation(*position));
                if lifecycle.activity != target {
                    router.reconcile_world_activity(&region, *position, target)?;
                    lifecycle_actions += 1;
                }
            } else if self
                .generation_results
                .len()
                .saturating_add(generation_requests)
                < self.config.maximum_generation_in_flight
                && generation_requests < self.config.maximum_generation_results_per_tick
                && lifecycle.pending_generation.is_some()
            {
                let request = router.resume_world_generation(&region, *position)?;
                self.generation_worker.submit(request)?;
                generation_requests += 1;
                lifecycle_actions += 1;
            } else if lifecycle.pending_generation.is_none()
                && self
                    .generation_results
                    .len()
                    .saturating_add(generation_requests)
                    < self.config.maximum_generation_in_flight
                && generation_requests < self.config.maximum_generation_results_per_tick
            {
                let target = next_status(lifecycle.status)
                    .ok_or(FormalChunkLifecycleError::InvalidGenerationStatus)?;
                let request = router.begin_world_generation(&region, *position, target)?;
                self.generation_worker.submit(request)?;
                generation_requests += 1;
                lifecycle_actions += 1;
            }
        }

        loaded = loaded_chunks(router, self.world, &self.dimension)?;
        for (position, (region, lifecycle)) in loaded {
            if demanded.contains(&position)
                || lifecycle_actions == self.config.maximum_lifecycle_actions_per_tick
            {
                continue;
            }
            if lifecycle.activity != ChunkActivity::Dormant {
                router.reconcile_world_activity(&region, position, ChunkActivity::Dormant)?;
                lifecycle_actions += 1;
            } else if lifecycle.pending_generation.is_none() && lifecycle.pending_unload.is_none() {
                router.schedule_world_unload(&region, position)?;
                lifecycle_actions += 1;
            }
        }
        self.generation_worker
            .collect(generation_requests, &mut self.generation_results)?;
        let generation_results = self.apply_generation_results(router)?;
        events = events.saturating_add(
            router
                .take_world_events(self.config.maximum_events_per_region_per_tick)
                .len(),
        );
        Ok(FormalChunkLifecycleReport {
            tick,
            tickets: self.tickets.len(),
            generation_results,
            generation_requests,
            lifecycle_actions,
            events,
            generation_in_flight: self.generation_results.len(),
        })
    }

    fn apply_generation_results(
        &mut self,
        router: &mut CompositeRegionRouter,
    ) -> Result<usize, FormalChunkLifecycleError> {
        let count = self
            .config
            .maximum_generation_results_per_tick
            .min(self.generation_results.len());
        for _ in 0..count {
            let result = self
                .generation_results
                .pop_front()
                .expect("bounded count came from the queue");
            router.apply_world_generation(result)?;
        }
        Ok(count)
    }

    fn replace_tickets(
        &mut self,
        tickets: impl IntoIterator<Item = ChunkTicket>,
    ) -> Result<(), FormalChunkLifecycleError> {
        let mut candidate = ChunkTicketBook::new(self.config.maximum_tickets)?;
        for ticket in tickets {
            candidate.upsert(ticket)?;
        }
        self.tickets = candidate;
        Ok(())
    }

    fn region_for(&self, position: ChunkPos) -> SimulationRegionKey {
        self.mapping
            .region_for_chunk(self.world, self.dimension.clone(), position)
    }
}

impl FormalGenerationWorker {
    fn new(
        capacity: usize,
        dimension: &DimensionId,
        seed: i64,
    ) -> Result<Self, FormalChunkLifecycleError> {
        let (requests, worker_requests) = sync_channel::<GenerationRequest>(capacity);
        let (worker_results, results) = sync_channel(capacity);
        let generator = FormalDimensionGenerator::new(dimension, seed)?;
        let thread_name = format!("ferrite-generation-{}", dimension.resource().path());
        let thread = thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                while let Ok(request) = worker_requests.recv() {
                    let mut generated = request.source.clone();
                    let result = generator
                        .apply_stage(&mut generated, request.target_status)
                        .map(|()| request.complete(generated));
                    if worker_results.send(result).is_err() {
                        break;
                    }
                }
            })
            .map_err(FormalChunkLifecycleError::WorkerSpawn)?;
        Ok(Self {
            requests: Some(requests),
            results,
            thread: Some(thread),
        })
    }

    fn submit(&self, request: GenerationRequest) -> Result<(), FormalChunkLifecycleError> {
        let sender = self
            .requests
            .as_ref()
            .ok_or(FormalChunkLifecycleError::WorkerDisconnected)?;
        match sender.try_send(request) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(FormalChunkLifecycleError::WorkerFull),
            Err(TrySendError::Disconnected(_)) => {
                Err(FormalChunkLifecycleError::WorkerDisconnected)
            }
        }
    }

    fn collect(
        &self,
        count: usize,
        destination: &mut VecDeque<GenerationResult>,
    ) -> Result<(), FormalChunkLifecycleError> {
        for _ in 0..count {
            let result = self
                .results
                .recv()
                .map_err(|_| FormalChunkLifecycleError::WorkerDisconnected)??;
            destination.push_back(result);
        }
        Ok(())
    }
}

impl Drop for FormalGenerationWorker {
    fn drop(&mut self) {
        self.requests.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn loaded_chunks(
    router: &CompositeRegionRouter,
    world: WorldId,
    dimension: &DimensionId,
) -> Result<BTreeMap<ChunkPos, (SimulationRegionKey, ChunkLifecycle)>, FormalChunkLifecycleError> {
    let mut loaded = BTreeMap::new();
    for (region, position, lifecycle) in router.world_chunks() {
        if region.world() != world || region.dimension() != dimension {
            continue;
        }
        if loaded.insert(position, (region, lifecycle)).is_some() {
            return Err(FormalChunkLifecycleError::DuplicateChunk(position));
        }
    }
    Ok(loaded)
}

fn activation_activity(activation: crate::chunk::ticket::ChunkActivation) -> ChunkActivity {
    if activation.ticking_entities {
        ChunkActivity::EntityTicking
    } else if activation.ticking_blocks {
        ChunkActivity::BlockTicking
    } else if activation.visible_to_clients {
        ChunkActivity::Accessible
    } else {
        ChunkActivity::Dormant
    }
}

fn next_status(status: ChunkStatus) -> Option<ChunkStatus> {
    ChunkStatus::ALL.get(status as usize + 1).copied()
}

#[derive(Debug, Error)]
pub(crate) enum FormalChunkLifecycleError {
    #[error("formal chunk lifecycle capacities must be nonzero")]
    ZeroCapacity,
    #[error("formal chunk lifecycle observed duplicate loaded chunk {0:?}")]
    DuplicateChunk(ChunkPos),
    #[error("formal chunk lifecycle observed the wrong owner for chunk {0:?}")]
    WrongChunkOwner(ChunkPos),
    #[error("formal chunk lifecycle cannot advance the generation status")]
    InvalidGenerationStatus,
    #[error("formal generation worker queue is full")]
    WorkerFull,
    #[error("formal generation worker disconnected")]
    WorkerDisconnected,
    #[error("formal generation worker could not start")]
    WorkerSpawn(#[source] std::io::Error),
    #[error(transparent)]
    Dimension(#[from] DimensionRuntimeError),
    #[error(transparent)]
    Ticket(#[from] ChunkTicketError),
    #[error(transparent)]
    Gateway(#[from] CompositeGatewayError),
}

#[cfg(test)]
mod tests {
    use ferrite_foundation::identity::{ActivationGeneration, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
    use ferrite_foundation::resource::ResourceId;
    use ferrite_persistence::snapshot::{
        PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
    };
    use ferrite_persistence::store::RegionFileStore;
    use ferrite_protocol::semantic::SessionId;
    use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
    use ferrite_simulation::region::RegionSimulationState;
    use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
    use ferrite_world::generation::overworld::OverworldGeneratorV1;
    use ferrite_world::id::{BiomeId, BlockStateId};
    use ferrite_world::region::RegionVoxelState;

    use super::*;
    use crate::chunk::ticket::{ACCESSIBLE_LEVEL, TicketLevel, TicketSource};
    use crate::composite::runtime::CompositeRuntimeConfig;
    use crate::composite::services::{
        CompositeProductionRegionRuntime, CompositeProductionRuntimeConfig,
    };
    use crate::entity_service::runtime::EntityServiceRuntimeLimits;
    use crate::simulation::budget::{SimulationQueueBudget, SimulationQueueKind};
    use crate::simulation::runtime::SimulationRuntimeConfig;
    use crate::world_service::model::WorldServiceRuntimeConfig;

    fn key() -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        )
    }

    fn layout() -> ChunkLayout {
        ChunkLayout::new(
            VerticalSectionRange::new(-4, 24).unwrap(),
            BlockStateId::new(0),
            BiomeId::new(0),
        )
    }

    fn composite_config() -> CompositeProductionRuntimeConfig {
        let budget = SimulationQueueBudget::new([
            (SimulationQueueKind::ScheduledBlocks, 32),
            (SimulationQueueKind::ScheduledFluids, 32),
            (SimulationQueueKind::BoundaryTransactions, 32),
            (SimulationQueueKind::ImmediateNeighbors, 32),
            (SimulationQueueKind::Fluids, 32),
            (SimulationQueueKind::Redstone, 32),
            (SimulationQueueKind::Lighting, 32),
            (SimulationQueueKind::ProjectionPositions, 32),
        ])
        .unwrap();
        CompositeProductionRuntimeConfig {
            coordinator: CompositeRuntimeConfig {
                command_capacity: 32,
                event_capacity: 32,
                projection_capacity: 32,
                continuity_record_capacity: 64,
                maximum_future_ticks: 4,
                maximum_payload_bytes: 1024 * 1024,
            },
            simulation: SimulationRuntimeConfig {
                mapping: RegionMapping::V1,
                budget,
                projection_capacity: 32,
                receipt_capacity: 32,
                gameplay_random_seed: 4,
            },
            entities: EntityServiceRuntimeLimits::new(4, 4, 4, 4),
            world: WorldServiceRuntimeConfig {
                mapping: RegionMapping::V1,
                layout: layout(),
                region_side_chunks: 8,
                chunk_capacity: 8,
                event_capacity: 32,
                content_manifest: [3; 32],
            },
            player_capacity: 4,
            projection_capacity_per_player: 4,
        }
    }

    fn test_router() -> CompositeRegionRouter {
        let key = key();
        let voxels = RegionVoxelState::new(key.clone(), RegionMapping::V1, layout()).unwrap();
        let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
        runner
            .insert_region(
                RegionSimulationState::new(voxels),
                ActivationGeneration::INITIAL,
                GameTick::ZERO,
            )
            .unwrap();
        let runtime = CompositeProductionRegionRuntime::new(
            key,
            ActivationGeneration::INITIAL,
            GameTick::ZERO,
            0,
            [],
            composite_config(),
        )
        .unwrap();
        CompositeRegionRouter::new(runner, [runtime]).unwrap()
    }

    fn lifecycle() -> FormalChunkLifecycle {
        FormalChunkLifecycle::new(
            key().world(),
            key().dimension().clone(),
            RegionMapping::V1,
            0,
            FormalChunkLifecycleConfig {
                maximum_tickets: 2,
                maximum_generation_in_flight: 1,
                maximum_generation_results_per_tick: 1,
                maximum_lifecycle_actions_per_tick: 4,
                maximum_events_per_region_per_tick: 8,
            },
        )
        .unwrap()
    }

    fn view_ticket() -> ChunkTicket {
        ChunkTicket {
            source: TicketSource::PlayerView(SessionId::new(1).unwrap()),
            position: ChunkPos::new(0, 0),
            level: TicketLevel::new(ACCESSIBLE_LEVEL),
            expires_at: None,
        }
    }

    #[test]
    fn generation_request_and_fenced_result_finish_before_continuity_commit() {
        let mut router = test_router();
        let mut lifecycle = lifecycle();
        let first = lifecycle
            .drive(GameTick::new(1), [view_ticket()], &mut router)
            .unwrap();
        assert_eq!(first.generation_requests, 1);
        assert_eq!(first.generation_results, 1);
        assert_eq!(first.generation_in_flight, 0);
        assert_eq!(
            router.world_chunks()[0].2.status,
            ChunkStatus::StructureStarts
        );
        router.run_tick(GameTick::new(1)).unwrap();

        let mut stale_router = test_router();
        stale_router
            .demand_world_chunk(&key(), ChunkPos::new(0, 0))
            .unwrap();
        let request = stale_router
            .begin_world_generation(&key(), ChunkPos::new(0, 0), ChunkStatus::StructureStarts)
            .unwrap();
        let generated = request.source.clone();
        let mut stale = request.complete(generated);
        stale.generation = ActivationGeneration::new(2).unwrap();
        lifecycle.generation_results.push_back(stale);
        assert!(matches!(
            lifecycle.apply_generation_results(&mut stale_router),
            Err(FormalChunkLifecycleError::Gateway(_))
        ));
        assert_eq!(stale_router.world_chunks()[0].2.status, ChunkStatus::Empty);
    }

    #[test]
    fn ticket_loss_waits_for_generation_then_unloads_only_after_matching_save_receipt() {
        let mut router = test_router();
        let mut lifecycle = lifecycle();
        lifecycle
            .drive(GameTick::new(1), [view_ticket()], &mut router)
            .unwrap();
        router.run_tick(GameTick::new(1)).unwrap();
        let second = lifecycle.drive(GameTick::new(2), [], &mut router).unwrap();
        assert_eq!(second.generation_results, 0);
        let chunk = router.world_chunks()[0].2;
        assert!(chunk.pending_generation.is_none());
        assert!(chunk.pending_unload.is_some());

        let report = router.run_tick(GameTick::new(2)).unwrap();
        let continuity = &report.region(&key()).unwrap().continuity;
        let point = RegionRecoveryPoint::new(
            RegionCommitSnapshot::new(
                RegionSnapshotHeader {
                    key: key(),
                    generation: ActivationGeneration::INITIAL,
                    committed_tick: 2,
                    persistence_revision: PersistenceRevision::INITIAL,
                    region_side_chunks: 8,
                    content_manifest: [3; 32],
                    state_hash: continuity.hash,
                },
                continuity.records.clone(),
            )
            .unwrap(),
            Vec::new(),
        )
        .unwrap();
        let temporary = tempfile::tempdir().unwrap();
        let receipt = RegionFileStore::open(temporary.path())
            .unwrap()
            .commit(&point)
            .unwrap();
        assert_eq!(
            router
                .apply_world_save_receipt(&key(), &point, receipt)
                .unwrap(),
            1
        );
        assert!(router.world_chunks().is_empty());
    }

    #[test]
    fn repeated_bounded_work_reaches_full_then_derives_accessible_activity() {
        let mut router = test_router();
        let mut lifecycle = lifecycle();
        for raw_tick in 1..=11 {
            let tick = GameTick::new(raw_tick);
            let report = lifecycle.drive(tick, [view_ticket()], &mut router).unwrap();
            assert_eq!(report.generation_requests, 1);
            assert_eq!(report.generation_results, 1);
            assert_eq!(report.generation_in_flight, 0);
            router.run_tick(tick).unwrap();
        }
        let full = router.world_chunks()[0].2;
        assert_eq!(full.status, ChunkStatus::Full);
        assert_eq!(full.activity, ChunkActivity::Dormant);
        let generated = router.world_chunk(&key(), ChunkPos::new(0, 0)).unwrap();
        let generator = OverworldGeneratorV1::new(
            0,
            BlockStateId::new(1),
            BlockStateId::new(2),
            [BiomeId::new(0), BiomeId::new(1), BiomeId::new(2)],
        );
        let height = generator.surface_height(0, 0);
        assert_eq!(
            generated.block_state(ferrite_foundation::coordinate::BlockPos::new(0, height, 0)),
            Ok(BlockStateId::new(2))
        );
        assert_eq!(
            generated.block_state(ferrite_foundation::coordinate::BlockPos::new(
                0,
                height + 1,
                0,
            )),
            Ok(BlockStateId::new(0))
        );

        let tick = GameTick::new(12);
        let report = lifecycle.drive(tick, [view_ticket()], &mut router).unwrap();
        assert_eq!(report.generation_requests, 0);
        assert_eq!(report.lifecycle_actions, 1);
        router.run_tick(tick).unwrap();
        assert_eq!(
            router.world_chunks()[0].2.activity,
            ChunkActivity::Accessible
        );
        let snapshots = router
            .projectable_world_snapshots(key().dimension(), [ChunkPos::new(0, 0)])
            .unwrap();
        let snapshot = snapshots.get(&ChunkPos::new(0, 0)).unwrap();
        assert_eq!(snapshot.revision(), generated.revision());
        assert_eq!(
            snapshot.heightmaps().values().next().unwrap()[0],
            height + 1
        );
    }

    #[test]
    fn oversized_ticket_replacement_preserves_the_previous_demand_set() {
        let mut router = test_router();
        let mut lifecycle = lifecycle();
        lifecycle
            .drive(GameTick::new(1), [view_ticket()], &mut router)
            .unwrap();
        let source = TicketSource::PlayerView(SessionId::new(2).unwrap());
        let replacements = (1..=3).map(|x| ChunkTicket {
            source: source.clone(),
            position: ChunkPos::new(x, 0),
            level: TicketLevel::new(ACCESSIBLE_LEVEL),
            expires_at: None,
        });
        assert!(matches!(
            lifecycle.drive(GameTick::new(2), replacements, &mut router),
            Err(FormalChunkLifecycleError::Ticket(
                ChunkTicketError::Full { .. }
            ))
        ));
        assert_eq!(lifecycle.tickets.len(), 1);
        assert_eq!(lifecycle.tickets.tickets().next().unwrap(), &view_ticket());
    }
}
