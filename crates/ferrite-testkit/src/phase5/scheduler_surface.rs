//! Executable TickScheduler root-surface conformance.

use crate::phase5::fixtures::{
    block_for_region, phase5_runtime, region, registry_map, voxel_state,
};
use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::RegionMapping;
use ferrite_foundation::resource::ResourceId;
use ferrite_replay::envelope::{
    CommandEnvelope, CommandSource, EnvelopePayload, SequenceNumber, TickNumber,
};
use ferrite_replay::hash::{RegionHashRecord, StateHash};
use ferrite_replay::log::{ReplayFrame, ReplayHeader, ReplayLog};
use ferrite_replay::verify::{ObservedFrame, ReplayTarget, VerificationReport, verify_replay};
use ferrite_server_runtime::phase5::boundary::{
    BoundaryMechanic, BoundaryMutation, BoundarySchedule, BoundaryTransactionHeader,
    BoundaryTransactionLimits, MechanicBoundaryTransaction,
};
use ferrite_server_runtime::phase5::budget::Phase5QueueKind;
use ferrite_server_runtime::phase5::continuity::ScheduledQueueKind;
use ferrite_server_runtime::player::block::replication::{
    AuthoritativeBlockUpdate, project_authoritative_updates,
};
use ferrite_simulation::random::RandomAlgorithm;
use ferrite_simulation::random_tick::activity::{HolderAccess, random_tick_chunk_order};
use ferrite_simulation::random_tick::position::RandomPositionStream;
use ferrite_simulation::random_tick::tracker::SimulationChunkTracker;
use ferrite_simulation::scheduled_tick::container::ChunkTickContainer;
use ferrite_simulation::scheduled_tick::level::{
    LevelScheduledTicks, LevelTickAdmission, ScheduleOutcome, ScheduledTickQueue,
};
use ferrite_simulation::scheduled_tick::record::{ScheduledTick, TickPriority};
use ferrite_simulation::tick::{GameTick, TickPhase};
use ferrite_world::id::BlockStateId;
use std::num::NonZeroU64;

const PROPERTY_CASES: usize = 128;
const REPLAY_FRAMES: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickSchedulerSurfaceReport {
    pub golden_digest: String,
    pub property_cases: usize,
    pub fault_cases: usize,
    pub boundary_cases: usize,
    pub replay_frames: usize,
}

pub fn run_tick_scheduler_surface() -> TickSchedulerSurfaceReport {
    let golden_digest = hex_digest(&golden_trace());
    run_property_sweep();
    run_fault_vectors();
    let boundary_cases = run_boundary_equivalence();
    run_replay_vectors();
    TickSchedulerSurfaceReport {
        golden_digest,
        property_cases: PROPERTY_CASES,
        fault_cases: 4,
        boundary_cases,
        replay_frames: REPLAY_FRAMES,
    }
}

fn golden_trace() -> Vec<u8> {
    let mut trace = TickPhase::ALL
        .into_iter()
        .map(TickPhase::stable_tag)
        .collect::<Vec<_>>();

    let mut queue = queue_with_chunks([ChunkPos::new(0, 0), ChunkPos::new(1, 0)]);
    for tick in [
        scheduled(
            1,
            BlockPos::new(0, 0, 0),
            -100,
            TickPriority::ExtremelyLow,
            0,
        ),
        scheduled(2, BlockPos::new(1, 0, 0), 0, TickPriority::ExtremelyHigh, 1),
        scheduled(3, BlockPos::new(16, 0, 0), -10, TickPriority::Normal, 2),
    ] {
        assert_eq!(queue.schedule(tick), ScheduleOutcome::Queued);
    }
    queue.tick(
        0,
        16,
        |_| true,
        |_, tick| {
            trace.extend_from_slice(&tick.type_identity.to_be_bytes());
        },
    );

    let mut tracker = SimulationChunkTracker::new();
    for (chunk, level) in [
        (ChunkPos::new(0, 0), 31),
        (ChunkPos::new(1, 0), 32),
        (ChunkPos::new(-1, 0), 30),
        (ChunkPos::new(2, 0), 29),
    ] {
        tracker.set_level(chunk, level);
    }
    for chunk in random_tick_chunk_order(&tracker, |chunk| {
        if chunk == ChunkPos::new(2, 0) {
            HolderAccess::MissingTickingChunk
        } else {
            HolderAccess::TickingChunk
        }
    }) {
        trace.extend_from_slice(&chunk.x.to_be_bytes());
        trace.extend_from_slice(&chunk.z.to_be_bytes());
    }

    let mut positions = RandomPositionStream::new(0x1234_5678);
    for _ in 0..5 {
        let position = positions.next(BlockPos::new(100, -64, -200), 15);
        trace.extend_from_slice(&position.x.to_be_bytes());
        trace.extend_from_slice(&position.y.to_be_bytes());
        trace.extend_from_slice(&position.z.to_be_bytes());
    }
    trace
}

fn run_property_sweep() {
    for case in 0..PROPERTY_CASES {
        let seed = (case as u64)
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(0x5eed);
        let first = scheduler_trace_for_seed(seed);
        let second = scheduler_trace_for_seed(seed);
        assert_eq!(first, second, "scheduler trace diverged for seed {seed}");
        assert_eq!(first.len(), 32);

        let mut left = RandomPositionStream::new(seed as i32);
        let mut right = RandomPositionStream::new(seed as i32);
        for sample in 0..64 {
            assert_eq!(
                left.next(BlockPos::new(case as i32, -64, sample), 15),
                right.next(BlockPos::new(case as i32, -64, sample), 15),
                "position stream diverged for seed {seed}"
            );
        }
    }
}

fn scheduler_trace_for_seed(seed: u64) -> Vec<u16> {
    let mut queue = queue_with_chunks((0..4).map(|x| ChunkPos::new(x, 0)));
    let mut random = ferrite_simulation::random::DeterministicRng::from_seed(seed);
    for identity in 0..32_u16 {
        let chunk = random.uniform_u64(NonZeroU64::new(4).unwrap()) as i32;
        let trigger = random.uniform_u64(NonZeroU64::new(8).unwrap()) as i64;
        let priority =
            TickPriority::from_value(random.uniform_u64(NonZeroU64::new(7).unwrap()) as i32 - 3);
        let position = BlockPos::new(
            chunk * 16 + i32::from(identity % 16),
            i32::from(identity),
            0,
        );
        assert_eq!(
            queue.schedule(scheduled(
                identity,
                position,
                trigger,
                priority,
                i64::from(identity),
            )),
            ScheduleOutcome::Queued
        );
    }
    let mut output = Vec::new();
    queue.tick(
        i64::MAX,
        64,
        |_| true,
        |_, tick| output.push(tick.type_identity),
    );
    assert_eq!(queue.count(), 0);
    output
}

fn run_fault_vectors() {
    let position = BlockPos::new(0, 64, 0);
    let mut unregistered = ScheduledTickQueue::new();
    assert_eq!(
        unregistered.schedule(scheduled(1, position, 0, TickPriority::Normal, 0,)),
        ScheduleOutcome::UnregisteredChunk
    );

    let mut inactive = queue_with_chunks([position.chunk()]);
    inactive.schedule(scheduled(2, position, 0, TickPriority::Normal, 0));
    assert_eq!(inactive.tick(100, 16, |_| false, |_, _| {}), 0);
    assert_eq!(inactive.count(), 1);

    let mut level = LevelScheduledTicks::<u16, u16>::default();
    level
        .blocks
        .register_container(position.chunk(), ChunkTickContainer::new());
    level
        .blocks
        .schedule(scheduled(3, position, 0, TickPriority::Normal, 0));
    let frozen = level.tick(
        LevelTickAdmission {
            runs_normally: false,
            debug_level: false,
        },
        100,
        &mut |_| true,
        &mut |_, _| panic!("frozen block callback"),
        &mut |_, _| panic!("frozen fluid callback"),
    );
    assert_eq!(frozen.blocks, 0);
    assert_eq!(level.blocks.count(), 1);

    let mut capped = queue_with_chunks([position.chunk()]);
    for identity in 0..3 {
        capped.schedule(scheduled(
            identity,
            BlockPos::new(identity.into(), 64, 0),
            0,
            TickPriority::Normal,
            i64::from(identity),
        ));
    }
    assert_eq!(capped.tick(0, 2, |_| true, |_, _| {}), 2);
    assert_eq!(capped.count(), 1);
}

fn run_boundary_equivalence() -> usize {
    let registries = registry_map();
    for (case, coordinate) in (-2..=2).enumerate() {
        let first = block_for_region(coordinate, 0);
        let second = block_for_region(coordinate, 1);
        let mut interior_runtime = phase5_runtime(coordinate);
        let mut interior_voxels = voxel_state(coordinate);
        interior_voxels
            .set_block(first, BlockStateId::new(1))
            .unwrap();
        interior_voxels
            .set_block(second, BlockStateId::new(2))
            .unwrap();
        interior_runtime
            .schedule_local(
                ScheduledQueueKind::Block,
                ResourceId::minecraft("stone").unwrap(),
                first,
                3,
                TickPriority::High,
            )
            .unwrap();
        let interior_packets = project_authoritative_updates(
            [
                AuthoritativeBlockUpdate {
                    position: first,
                    state: BlockStateId::new(1),
                },
                AuthoritativeBlockUpdate {
                    position: second,
                    state: BlockStateId::new(2),
                },
            ],
            &registries,
        )
        .unwrap();

        let mut boundary_runtime = phase5_runtime(coordinate);
        let mut boundary_voxels = voxel_state(coordinate);
        let transaction = MechanicBoundaryTransaction::new(
            BoundaryTransactionHeader {
                tick: GameTick::new(7),
                source: region(coordinate - 1),
                source_generation: ActivationGeneration::INITIAL,
                target: region(coordinate),
                target_generation: ActivationGeneration::INITIAL,
                source_sequence: case as u64 + 1,
            },
            BoundaryMechanic::Neighbor,
            vec![
                BoundaryMutation {
                    order: 0,
                    position: first,
                    expected: BlockStateId::new(0),
                    replacement: BlockStateId::new(1),
                },
                BoundaryMutation {
                    order: 1,
                    position: second,
                    expected: BlockStateId::new(0),
                    replacement: BlockStateId::new(2),
                },
            ],
            vec![BoundarySchedule {
                order: 0,
                kind: ScheduledQueueKind::Block,
                type_identity: ResourceId::minecraft("stone").unwrap(),
                position: first,
                delay: 3,
                priority: TickPriority::High,
            }],
            RegionMapping::V1,
            BoundaryTransactionLimits::new(8, 8),
        )
        .unwrap();
        boundary_runtime
            .apply_transaction(&mut boundary_voxels, &transaction)
            .unwrap();
        boundary_runtime
            .drain_effects(Phase5QueueKind::ImmediateNeighbors, usize::MAX)
            .unwrap();
        let boundary_packets = boundary_runtime.project_and_clear(&registries).unwrap();

        assert_eq!(
            interior_voxels.view().block_state(first).unwrap(),
            boundary_voxels.view().block_state(first).unwrap()
        );
        assert_eq!(
            interior_voxels.view().block_state(second).unwrap(),
            boundary_voxels.view().block_state(second).unwrap()
        );
        assert_eq!(interior_packets, boundary_packets);

        interior_runtime
            .advance_commit(GameTick::new(8), 103)
            .unwrap();
        boundary_runtime
            .advance_commit(GameTick::new(8), 103)
            .unwrap();
        let mut interior_due = Vec::new();
        let mut boundary_due = Vec::new();
        interior_runtime.tick_scheduled(
            ScheduledQueueKind::Block,
            8,
            |_| true,
            |tick| interior_due.push(tick.type_identity),
        );
        boundary_runtime.tick_scheduled(
            ScheduledQueueKind::Block,
            8,
            |_| true,
            |tick| boundary_due.push(tick.type_identity),
        );
        assert_eq!(interior_due, boundary_due);
    }
    5
}

fn run_replay_vectors() {
    let key = region(0);
    let frames = (1..=REPLAY_FRAMES)
        .map(|tick| {
            let seed = 1000 + tick as u64;
            let hash = scheduler_hash(seed, false);
            let command = CommandEnvelope::new(
                TickNumber::new(tick as u64),
                SequenceNumber::new(1),
                CommandSource::System,
                key.clone(),
                ResourceId::new("ferrite", "phase5/scheduler-seed").unwrap(),
                EnvelopePayload::new(seed.to_be_bytes().to_vec()).unwrap(),
            );
            ReplayFrame::new(
                TickNumber::new(tick as u64),
                vec![command],
                Vec::new(),
                vec![RegionHashRecord::new(key.clone(), hash)],
                hash,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let log = ReplayLog::new(
        ReplayHeader::new(
            ResourceId::new("ferrite", "phase5-conformance").unwrap(),
            key.world(),
            StateHash::from_bytes([0x51; 32]),
            key.mapping_version(),
            RandomAlgorithm::Xoshiro256StarStarV1,
            TickNumber::new(0),
        ),
        frames,
    )
    .unwrap();
    let mut converging = SchedulerReplayTarget {
        region: key.clone(),
        perturb: false,
    };
    assert!(verify_replay(&log, &mut converging).is_converged());
    let mut diverging = SchedulerReplayTarget {
        region: key,
        perturb: true,
    };
    assert!(matches!(
        verify_replay(&log, &mut diverging),
        VerificationReport::Diverged(_)
    ));
}

struct SchedulerReplayTarget {
    region: ferrite_foundation::region::SimulationRegionKey,
    perturb: bool,
}

impl ReplayTarget for SchedulerReplayTarget {
    type Error = String;

    fn begin(&mut self, _header: &ReplayHeader) -> Result<(), Self::Error> {
        Ok(())
    }

    fn execute(
        &mut self,
        tick: TickNumber,
        commands: &[CommandEnvelope],
    ) -> Result<ObservedFrame, Self::Error> {
        let command = commands
            .first()
            .ok_or_else(|| "scheduler replay command is missing".to_owned())?;
        let seed = u64::from_be_bytes(
            command
                .payload()
                .as_slice()
                .try_into()
                .map_err(|_| "scheduler seed must contain eight bytes")?,
        );
        let hash = scheduler_hash(seed, self.perturb);
        ObservedFrame::new(
            tick,
            Vec::new(),
            vec![RegionHashRecord::new(self.region.clone(), hash)],
            hash,
        )
        .map_err(|error| error.to_string())
    }
}

fn scheduler_hash(seed: u64, perturb: bool) -> StateHash {
    let mut bytes = scheduler_trace_for_seed(seed)
        .into_iter()
        .flat_map(u16::to_be_bytes)
        .collect::<Vec<_>>();
    if perturb {
        bytes.push(1);
    }
    StateHash::from_bytes(*blake3::hash(&bytes).as_bytes())
}

fn queue_with_chunks(chunks: impl IntoIterator<Item = ChunkPos>) -> ScheduledTickQueue<u16> {
    let mut queue = ScheduledTickQueue::new();
    for chunk in chunks {
        queue.register_container(chunk, ChunkTickContainer::new());
    }
    queue
}

const fn scheduled(
    type_identity: u16,
    position: BlockPos,
    trigger_tick: i64,
    priority: TickPriority,
    sub_tick_order: i64,
) -> ScheduledTick<u16> {
    ScheduledTick::new(
        type_identity,
        position,
        trigger_tick,
        priority,
        sub_tick_order,
    )
}

fn hex_digest(bytes: &[u8]) -> String {
    blake3::hash(bytes)
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
