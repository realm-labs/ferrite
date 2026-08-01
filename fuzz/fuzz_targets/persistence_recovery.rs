#![no_main]

use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_persistence::snapshot::{
    JournalTailFrame, PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint,
    RegionSnapshotHeader, SnapshotRecord, SnapshotRecordKind,
};
use libfuzzer_sys::fuzz_target;

fn recovery_point(data: &[u8]) -> RegionRecoveryPoint {
    let key = SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    );
    let records = data
        .chunks(8)
        .take(16)
        .enumerate()
        .map(|(index, bytes)| {
            SnapshotRecord::new(
                SnapshotRecordKind::Extension,
                ResourceId::new("ferrite", "fuzz/persistence").unwrap(),
                (index as u64).to_le_bytes().to_vec(),
                bytes.to_vec(),
            )
            .unwrap()
        })
        .collect();
    let snapshot = RegionCommitSnapshot::new(
        RegionSnapshotHeader {
            key,
            generation: ActivationGeneration::INITIAL,
            committed_tick: 4,
            persistence_revision: PersistenceRevision::INITIAL,
            region_side_chunks: 8,
            content_manifest: [0x11; 32],
            state_hash: [0x22; 32],
        },
        records,
    )
    .unwrap();
    let tail = JournalTailFrame::new(5, Vec::new()).unwrap();
    RegionRecoveryPoint::new(snapshot, vec![tail]).unwrap()
}

fuzz_target!(|data: &[u8]| {
    if let Ok(decoded) = RegionRecoveryPoint::decode(data) {
        let encoded = decoded.encode().unwrap();
        assert_eq!(RegionRecoveryPoint::decode(&encoded).unwrap(), decoded);
        assert_eq!(
            decoded.digest().unwrap(),
            *blake3::hash(&encoded).as_bytes()
        );
    }

    let point = recovery_point(data);
    let encoded = point.encode().unwrap();
    let decoded = RegionRecoveryPoint::decode(&encoded).unwrap();
    assert_eq!(decoded, point);
    assert_eq!(decoded.digest().unwrap(), point.digest().unwrap());
    if !encoded.is_empty() {
        let mut corrupted = encoded;
        let index = data.first().copied().map_or(0, usize::from) % corrupted.len();
        corrupted[index] ^= data.get(1).copied().unwrap_or(0xff);
        let _ = RegionRecoveryPoint::decode(&corrupted);
    }
});
