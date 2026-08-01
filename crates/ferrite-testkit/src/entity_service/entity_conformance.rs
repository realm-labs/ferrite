//! Executable entity-service entity and mob Region conformance.

use std::num::NonZeroU64;

use ferrite_foundation::identity::{ActivationGeneration, StableEntityId};
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{SnapshotRecord, SnapshotRecordKind};
use ferrite_replay::envelope::{
    CommandEnvelope, CommandSource, EnvelopePayload, SequenceNumber, TickNumber,
};
use ferrite_replay::hash::{RegionHashRecord, StateHash};
use ferrite_replay::log::{ReplayFrame, ReplayHeader, ReplayLog};
use ferrite_replay::verify::{ObservedFrame, ReplayTarget, VerificationReport, verify_replay};
use ferrite_server_runtime::entity_service::continuity::{encode_entity, entity_domain};
use ferrite_server_runtime::entity_service::model::{
    EntityCommandHeader, EntityLifecycleState, EntityMutation, EntityProjection,
    EntityTransferRequest,
};
use ferrite_server_runtime::entity_service::runtime::{
    EntityServiceRegionRuntime, EntityServiceRuntimeError, EntityServiceRuntimeLimits,
};
use ferrite_server_runtime::entity_service::transfer::TransferAcceptance;
use ferrite_simulation::random::{DeterministicRng, RandomAlgorithm};
use ferrite_simulation::tick::GameTick;

use crate::entity_service::fixtures::{chunk, entity, limits, payload, region, runtime, state};

const PROPERTY_CASES: usize = 128;
const FUZZ_CASES: usize = 256;
const FAULT_CASES: usize = 10;
const TRANSFER_CASES: usize = 64;
const REPLAY_FRAMES: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityConformanceReport {
    pub golden_digest: String,
    pub property_cases: usize,
    pub fuzz_cases: usize,
    pub fault_cases: usize,
    pub transfer_cases: usize,
    pub replay_frames: usize,
    pub client_trace_events: usize,
}

#[must_use]
pub fn run_entity_conformance() -> EntityConformanceReport {
    let trace = golden_client_trace();
    run_ordering_properties();
    run_operation_fuzz();
    run_fault_vectors();
    run_transfer_equivalence();
    run_replay_vectors();
    EntityConformanceReport {
        golden_digest: digest_debug(&trace),
        property_cases: PROPERTY_CASES,
        fuzz_cases: FUZZ_CASES,
        fault_cases: FAULT_CASES,
        transfer_cases: TRANSFER_CASES,
        replay_frames: REPLAY_FRAMES,
        client_trace_events: trace.len(),
    }
}

fn golden_client_trace() -> Vec<EntityProjection> {
    let stable_id = entity(1);
    let source_observer = entity(100);
    let target_observer = entity(101);
    let mut source = runtime(0, 32);
    let mut target = runtime(1, 32);
    source.add_observer(source_observer).unwrap();
    target.add_observer(target_observer).unwrap();

    source.insert(stable_id, state(0, 1)).unwrap();
    source
        .apply_mutation(
            &next_header(&source, stable_id),
            EntityMutation {
                chunk: chunk(0, 1),
                payload: payload(vec![2]),
            },
        )
        .unwrap();
    source.deactivate(&next_header(&source, stable_id)).unwrap();
    source.activate(&next_header(&source, stable_id)).unwrap();

    let current = source.state(stable_id).unwrap();
    let transfer = source
        .prepare_transfer(EntityTransferRequest {
            tick: GameTick::new(7),
            source: region(0),
            source_generation: ActivationGeneration::INITIAL,
            target: region(1),
            target_generation: ActivationGeneration::INITIAL,
            entity: stable_id,
            expected_revision: current.revision,
            sequence: current.last_command_sequence + 1,
            candidate: EntityMutation {
                chunk: chunk(1, 0),
                payload: payload(vec![3]),
            },
        })
        .unwrap();
    let TransferAcceptance::Accepted(receipt) = target.accept_transfer(&transfer).unwrap() else {
        panic!("first transfer delivery must be accepted");
    };
    assert!(matches!(
        target.accept_transfer(&transfer).unwrap(),
        TransferAcceptance::AlreadyApplied(_)
    ));
    source.commit_transfer(&receipt).unwrap();

    target
        .apply_mutation(
            &next_header(&target, stable_id),
            EntityMutation {
                chunk: chunk(1, 1),
                payload: payload(vec![4]),
            },
        )
        .unwrap();
    target.deactivate(&next_header(&target, stable_id)).unwrap();
    target.activate(&next_header(&target, stable_id)).unwrap();
    target.despawn(&next_header(&target, stable_id)).unwrap();

    let mut trace = source
        .drain_projections(source_observer, usize::MAX)
        .unwrap();
    trace.extend(
        target
            .drain_projections(target_observer, usize::MAX)
            .unwrap(),
    );
    trace
}

fn run_ordering_properties() {
    for case in 0..PROPERTY_CASES {
        let base = case as u128 * 4 + 1;
        let ids = [entity(base), entity(base + 1), entity(base + 2)];
        let mut ascending = runtime(0, 16);
        let mut descending = runtime(0, 16);
        for (offset, id) in ids.into_iter().enumerate() {
            ascending.insert(id, state(0, offset as u8)).unwrap();
        }
        for (offset, id) in ids.into_iter().enumerate().rev() {
            descending.insert(id, state(0, offset as u8)).unwrap();
        }
        assert_eq!(
            ascending.snapshot_records().unwrap(),
            descending.snapshot_records().unwrap()
        );

        let observer = entity(10_000 + case as u128);
        ascending.add_observer(observer).unwrap();
        descending.add_observer(observer).unwrap();
        assert_eq!(
            ascending.drain_projections(observer, usize::MAX).unwrap(),
            descending.drain_projections(observer, usize::MAX).unwrap()
        );
    }
}

fn run_operation_fuzz() {
    let mut random = DeterministicRng::from_seed(0x656e_7469_7479);
    for case in 0..FUZZ_CASES {
        let stable_id = entity(case as u128 + 1);
        let observer = entity(100_000 + case as u128);
        let mut first = runtime(0, 32);
        let mut second = runtime(0, 32);
        first.add_observer(observer).unwrap();
        second.add_observer(observer).unwrap();
        first.insert(stable_id, state(0, case as u8)).unwrap();
        second.insert(stable_id, state(0, case as u8)).unwrap();

        for step in 0..8 {
            let lifecycle = first.state(stable_id).unwrap().lifecycle.clone();
            match lifecycle {
                EntityLifecycleState::Active
                    if random.uniform_u64(NonZeroU64::new(3).unwrap()) == 0 =>
                {
                    let first_result = first.deactivate(&next_header(&first, stable_id));
                    let second_result = second.deactivate(&next_header(&second, stable_id));
                    assert_eq!(first_result, second_result);
                }
                EntityLifecycleState::Active => {
                    let marker = random.uniform_u64(NonZeroU64::new(256).unwrap()) as u8;
                    let local_x = random.uniform_u64(NonZeroU64::new(8).unwrap()) as i32;
                    let mutation = EntityMutation {
                        chunk: chunk(0, local_x),
                        payload: payload(vec![case as u8, step, marker]),
                    };
                    let first_result =
                        first.apply_mutation(&next_header(&first, stable_id), mutation.clone());
                    let second_result =
                        second.apply_mutation(&next_header(&second, stable_id), mutation);
                    assert_eq!(first_result, second_result);
                }
                EntityLifecycleState::Inactive => {
                    let first_result = first.activate(&next_header(&first, stable_id));
                    let second_result = second.activate(&next_header(&second, stable_id));
                    assert_eq!(first_result, second_result);
                }
                EntityLifecycleState::OutboundPending(_) => {
                    panic!("fuzz operations do not prepare transfers")
                }
            }
        }
        assert_eq!(
            first.snapshot_records().unwrap(),
            second.snapshot_records().unwrap()
        );
        assert_eq!(
            first.drain_projections(observer, usize::MAX).unwrap(),
            second.drain_projections(observer, usize::MAX).unwrap()
        );
    }
}

fn run_fault_vectors() {
    assert!(
        EntityServiceRegionRuntime::new(
            region(0),
            ActivationGeneration::INITIAL,
            RegionMapping::V1,
            EntityServiceRuntimeLimits::new(0, 1, 1, 1),
        )
        .is_err()
    );

    let stable_id = entity(1);
    let mut duplicate = runtime(0, 8);
    duplicate.insert(stable_id, state(0, 1)).unwrap();
    assert!(duplicate.insert(stable_id, state(0, 2)).is_err());

    let mut fenced = runtime(0, 8);
    fenced.insert(stable_id, state(0, 1)).unwrap();
    let mut header = next_header(&fenced, stable_id);
    header.region = region(1);
    assert!(matches!(
        fenced.apply_mutation(&header, mutation(0, 2)),
        Err(EntityServiceRuntimeError::WrongRegion)
    ));
    header = next_header(&fenced, stable_id);
    header.generation = ActivationGeneration::new(2).unwrap();
    assert!(matches!(
        fenced.apply_mutation(&header, mutation(0, 2)),
        Err(EntityServiceRuntimeError::StaleGeneration { .. })
    ));
    header = next_header(&fenced, stable_id);
    header.sequence += 1;
    assert!(matches!(
        fenced.apply_mutation(&header, mutation(0, 2)),
        Err(EntityServiceRuntimeError::CommandSequenceGap { .. })
    ));
    header = next_header(&fenced, stable_id);
    header.expected_revision += 1;
    assert!(matches!(
        fenced.apply_mutation(&header, mutation(0, 2)),
        Err(EntityServiceRuntimeError::RevisionMismatch { .. })
    ));
    assert!(matches!(
        fenced.apply_mutation(&next_header(&fenced, stable_id), mutation(1, 2)),
        Err(EntityServiceRuntimeError::WrongChunkOwner { .. })
    ));

    let mut blocked = EntityServiceRegionRuntime::new(
        region(0),
        ActivationGeneration::INITIAL,
        RegionMapping::V1,
        EntityServiceRuntimeLimits::new(8, 8, 1, 8),
    )
    .unwrap();
    blocked.add_observer(entity(2)).unwrap();
    blocked.insert(stable_id, state(0, 1)).unwrap();
    assert!(matches!(
        blocked.apply_mutation(&next_header(&blocked, stable_id), mutation(0, 2)),
        Err(EntityServiceRuntimeError::ProjectionCapacity { .. })
    ));
    assert_eq!(blocked.state(stable_id).unwrap().revision, 0);

    let mut source = runtime(0, 8);
    let mut target = EntityServiceRegionRuntime::new(
        region(1),
        ActivationGeneration::new(2).unwrap(),
        RegionMapping::V1,
        limits(8),
    )
    .unwrap();
    source.insert(stable_id, state(0, 1)).unwrap();
    let transfer = source
        .prepare_transfer(transfer_request(&source, stable_id, 1))
        .unwrap();
    assert!(matches!(
        target.accept_transfer(&transfer),
        Err(EntityServiceRuntimeError::StaleGeneration { .. })
    ));

    let valid = encode_entity(entity(9), &state(0, 1)).unwrap();
    let mut corrupt = valid.value().to_vec();
    corrupt.push(0xff);
    let corrupt = SnapshotRecord::new(
        SnapshotRecordKind::Entity,
        entity_domain(),
        entity(9).to_be_bytes().to_vec(),
        corrupt,
    )
    .unwrap();
    assert!(
        EntityServiceRegionRuntime::restore(
            region(0),
            ActivationGeneration::INITIAL,
            RegionMapping::V1,
            limits(8),
            &[corrupt],
        )
        .is_err()
    );
}

fn run_transfer_equivalence() {
    for case in 0..TRANSFER_CASES {
        let stable_id = entity(case as u128 + 1);
        let local_x = case as i32 % 8;
        let mut source = runtime(0, 8);
        let mut transferred = runtime(1, 8);
        let mut direct = runtime(1, 8);
        source.insert(stable_id, state(0, case as u8)).unwrap();
        let request = EntityTransferRequest {
            tick: GameTick::new(case as u64 + 1),
            source: region(0),
            source_generation: ActivationGeneration::INITIAL,
            target: region(1),
            target_generation: ActivationGeneration::INITIAL,
            entity: stable_id,
            expected_revision: 0,
            sequence: 1,
            candidate: EntityMutation {
                chunk: chunk(1, local_x),
                payload: payload(vec![0xa5, case as u8]),
            },
        };
        let transfer = source.prepare_transfer(request).unwrap();
        let TransferAcceptance::Accepted(receipt) = transferred.accept_transfer(&transfer).unwrap()
        else {
            panic!("first transfer delivery must be accepted");
        };
        source.commit_transfer(&receipt).unwrap();

        let mut expected = state(1, 0);
        expected.chunk = chunk(1, local_x);
        expected.payload = payload(vec![0xa5, case as u8]);
        expected.revision = 1;
        expected.last_command_sequence = 1;
        direct.insert(stable_id, expected).unwrap();
        assert_eq!(transferred.state(stable_id), direct.state(stable_id));
        assert_eq!(source.entity_count(), 0);
    }
}

fn run_replay_vectors() {
    let key = region(0);
    let frames = (1..=REPLAY_FRAMES)
        .map(|tick| {
            let seed = 0x7000 + tick as u64;
            let hash = entity_hash(seed, false);
            let command = CommandEnvelope::new(
                TickNumber::new(tick as u64),
                SequenceNumber::new(1),
                CommandSource::System,
                key.clone(),
                ResourceId::new("ferrite", "entity-service/entity-seed").unwrap(),
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
        .collect();
    let log = ReplayLog::new(
        ReplayHeader::new(
            ResourceId::new("ferrite", "entity-service-conformance").unwrap(),
            key.world(),
            StateHash::from_bytes([0x71; 32]),
            key.mapping_version(),
            RandomAlgorithm::Xoshiro256StarStarV1,
            TickNumber::new(0),
        ),
        frames,
    )
    .unwrap();
    assert!(
        verify_replay(
            &log,
            &mut EntityReplayTarget {
                region: key.clone(),
                perturb: false,
            },
        )
        .is_converged()
    );
    assert!(matches!(
        verify_replay(
            &log,
            &mut EntityReplayTarget {
                region: key,
                perturb: true,
            },
        ),
        VerificationReport::Diverged(_)
    ));
}

struct EntityReplayTarget {
    region: SimulationRegionKey,
    perturb: bool,
}

impl ReplayTarget for EntityReplayTarget {
    type Error = String;

    fn begin(&mut self, _header: &ReplayHeader) -> Result<(), Self::Error> {
        Ok(())
    }

    fn execute(
        &mut self,
        tick: TickNumber,
        commands: &[CommandEnvelope],
    ) -> Result<ObservedFrame, Self::Error> {
        let seed = u64::from_be_bytes(
            commands
                .first()
                .ok_or_else(|| "entity replay command is missing".to_owned())?
                .payload()
                .as_slice()
                .try_into()
                .map_err(|_| "entity seed must contain eight bytes")?,
        );
        let hash = entity_hash(seed, self.perturb);
        ObservedFrame::new(
            tick,
            Vec::new(),
            vec![RegionHashRecord::new(self.region.clone(), hash)],
            hash,
        )
        .map_err(|error| error.to_string())
    }
}

fn entity_hash(seed: u64, perturb: bool) -> StateHash {
    let stable_id = entity(u128::from(seed) + 1);
    let mut random = DeterministicRng::from_seed(seed);
    let mut runtime = runtime(0, 16);
    runtime.insert(stable_id, state(0, seed as u8)).unwrap();
    for step in 0..4 {
        let local_x = random.uniform_u64(NonZeroU64::new(8).unwrap()) as i32;
        let marker = random.uniform_u64(NonZeroU64::new(256).unwrap()) as u8;
        runtime
            .apply_mutation(
                &next_header(&runtime, stable_id),
                EntityMutation {
                    chunk: chunk(0, local_x),
                    payload: payload(vec![step, marker]),
                },
            )
            .unwrap();
    }
    let mut bytes = Vec::new();
    for record in runtime.snapshot_records().unwrap() {
        bytes.push(record.kind() as u8);
        bytes.extend_from_slice(record.domain().to_string().as_bytes());
        bytes.extend_from_slice(record.key());
        bytes.extend_from_slice(record.value());
    }
    if perturb {
        bytes.push(1);
    }
    StateHash::from_bytes(*blake3::hash(&bytes).as_bytes())
}

fn next_header(
    runtime: &EntityServiceRegionRuntime,
    stable_id: StableEntityId,
) -> EntityCommandHeader {
    let state = runtime
        .state(stable_id)
        .expect("fixture entity remains live");
    EntityCommandHeader {
        region: runtime.key().clone(),
        generation: runtime.generation(),
        entity: stable_id,
        expected_revision: state.revision,
        sequence: state.last_command_sequence + 1,
    }
}

fn mutation(coordinate_x: i32, marker: u8) -> EntityMutation {
    EntityMutation {
        chunk: chunk(coordinate_x, 0),
        payload: payload(vec![marker]),
    }
}

fn transfer_request(
    source: &EntityServiceRegionRuntime,
    stable_id: StableEntityId,
    target_generation: u64,
) -> EntityTransferRequest {
    let state = source
        .state(stable_id)
        .expect("fixture entity remains live");
    EntityTransferRequest {
        tick: GameTick::new(1),
        source: source.key().clone(),
        source_generation: source.generation(),
        target: region(1),
        target_generation: ActivationGeneration::new(target_generation).unwrap(),
        entity: stable_id,
        expected_revision: state.revision,
        sequence: state.last_command_sequence + 1,
        candidate: mutation(1, 2),
    }
}

fn digest_debug(values: &[impl std::fmt::Debug]) -> String {
    let bytes = values
        .iter()
        .flat_map(|value| format!("{value:?}\n").into_bytes())
        .collect::<Vec<_>>();
    blake3::hash(&bytes)
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
