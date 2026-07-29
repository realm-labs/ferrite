use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{
    PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
    SnapshotRecord, SnapshotRecordKind,
};
use ferrite_region_runtime::lattice::authority::{
    LatticeNodeIdentity, RegionAuthorityAction, RegionClaimGrant, RegionPlacementObservation,
    RegionPlacementState,
};
use ferrite_region_runtime::lattice::handoff::prepare_handoff;
use ferrite_region_runtime::lattice::spatial::{SpatialPlacementAdapter, SpatialPlacementConfig};

fn region() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(2, -3),
        RegionMappingVersion::V1,
    )
}

fn node(name: &str, port: u16, incarnation: u128) -> LatticeNodeIdentity {
    LatticeNodeIdentity::new(name, "127.0.0.1", port, incarnation).unwrap()
}

fn adapter() -> SpatialPlacementAdapter {
    SpatialPlacementAdapter::new(SpatialPlacementConfig::new("world-1", 4, 16, 7).unwrap()).unwrap()
}

fn running_observation(revision: u64) -> RegionPlacementObservation {
    RegionPlacementObservation {
        generation: ActivationGeneration::INITIAL,
        coordinator_term: 1,
        revision,
        state: RegionPlacementState::Running,
        target: None,
        move_id: None,
    }
}

fn recovery_point() -> RegionRecoveryPoint {
    RegionRecoveryPoint::new(
        RegionCommitSnapshot::new(
            RegionSnapshotHeader {
                key: region(),
                generation: ActivationGeneration::INITIAL,
                committed_tick: 12,
                persistence_revision: PersistenceRevision::INITIAL,
                region_side_chunks: 8,
                content_manifest: [1; 32],
                state_hash: [2; 32],
            },
            vec![
                SnapshotRecord::new(
                    SnapshotRecordKind::Entity,
                    ResourceId::new("ferrite", "entity/v1").unwrap(),
                    vec![1],
                    vec![2],
                )
                .unwrap(),
            ],
        )
        .unwrap(),
        vec![],
    )
    .unwrap()
}

#[test]
fn lattice_claim_deadline_and_generation_fence_region_admission() {
    let spatial = adapter();
    let route = spatial.route(&region()).unwrap();
    let mut authority = spatial
        .authority_adapter(&route, node("node-a", 7001, 1), 1_000)
        .unwrap();
    authority.reconcile(running_observation(1)).unwrap();
    let outcome = authority
        .install_claim(
            RegionClaimGrant {
                generation: ActivationGeneration::INITIAL,
                coordinator_term: 1,
                grant_sequence: 1,
                ttl_millis: 10_000,
            },
            100,
        )
        .unwrap();
    assert!(outcome.contains(RegionAuthorityAction::StartRegion));
    assert!(outcome.contains(RegionAuthorityAction::OpenAdmission));
    assert!(authority.admission_open(ActivationGeneration::INITIAL, 9_099));
    assert!(!authority.admission_open(ActivationGeneration::INITIAL, 9_100));
    assert!(!authority.admission_open(ActivationGeneration::new(2).unwrap(), 200));

    let outcome = authority.claim_lost().unwrap();
    assert!(outcome.contains(RegionAuthorityAction::FenceAdmission));
    assert!(outcome.contains(RegionAuthorityAction::StopRegion));
}

#[test]
fn graceful_handoff_moves_only_durable_fenced_ferrite_state() {
    let spatial = adapter();
    let route = spatial.route(&region()).unwrap();
    let mut authority = spatial
        .authority_adapter(&route, node("node-a", 7001, 1), 1_000)
        .unwrap();
    authority.reconcile(running_observation(1)).unwrap();
    authority
        .install_claim(
            RegionClaimGrant {
                generation: ActivationGeneration::INITIAL,
                coordinator_term: 1,
                grant_sequence: 1,
                ttl_millis: 10_000,
            },
            100,
        )
        .unwrap();
    authority
        .reconcile(RegionPlacementObservation {
            generation: ActivationGeneration::INITIAL,
            coordinator_term: 1,
            revision: 2,
            state: RegionPlacementState::BeginHandoff,
            target: Some(node("node-b", 7002, 2)),
            move_id: Some(9),
        })
        .unwrap();

    let target_generation = ActivationGeneration::new(2).unwrap();
    let envelope = prepare_handoff(&mut authority, &recovery_point(), target_generation).unwrap();
    assert!(!authority.admission_open(ActivationGeneration::INITIAL, 200));
    assert_eq!(envelope.source_generation(), ActivationGeneration::INITIAL);
    assert_eq!(envelope.target_generation(), target_generation);
    let recovered = envelope.recover(&region()).unwrap();
    assert_eq!(recovered.generation(), target_generation);
    assert_eq!(recovered.committed_tick(), 12);
}
