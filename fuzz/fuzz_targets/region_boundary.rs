#![no_main]

use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::boundary::{
    BoundaryBatch, BoundaryBatchHeader, BoundaryEvent, BoundaryInbox,
};
use ferrite_simulation::tick::{GameTick, TickPhase};
use libfuzzer_sys::fuzz_target;

fn region(x: i32) -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(x, 0),
        RegionMappingVersion::V1,
    )
}

fuzz_target!(|data: &[u8]| {
    let target = region(0);
    let expected_generation = ActivationGeneration::INITIAL;
    let mut inbox = BoundaryInbox::new(target.clone(), 32).unwrap();

    for operation in data.chunks(7).take(128) {
        let source = region(i32::from(operation.first().copied().unwrap_or(1) % 4) + 1);
        let tick = GameTick::new(u64::from(operation.get(1).copied().unwrap_or(1) % 8) + 1);
        let phase = TickPhase::ALL[usize::from(operation.get(2).copied().unwrap_or_default()) % 20];
        let sequence = u64::from(operation.get(3).copied().unwrap_or_default());
        let generation =
            ActivationGeneration::new(u64::from(operation.get(4).copied().unwrap_or(1) % 3) + 1)
                .unwrap();
        let events = operation
            .get(5..)
            .unwrap_or_default()
            .iter()
            .enumerate()
            .map(|(order, byte)| {
                BoundaryEvent::new(
                    order as u64,
                    ResourceId::new("ferrite", "fuzz/boundary").unwrap(),
                    vec![*byte],
                )
                .unwrap()
            })
            .collect();
        if let Ok(batch) = BoundaryBatch::new(
            BoundaryBatchHeader {
                tick,
                phase,
                source,
                target: target.clone(),
                source_generation: generation,
                source_sequence: sequence,
            },
            events,
            2,
        ) {
            let _ = inbox.admit(batch, expected_generation, GameTick::ZERO);
        }
        assert!(inbox.len() <= 32);
    }

    for tick in 1..=8 {
        for phase in TickPhase::ALL {
            let drained = inbox.drain(GameTick::new(tick), phase);
            assert!(drained.windows(2).all(|pair| {
                (pair[0].source(), pair[0].source_sequence())
                    <= (pair[1].source(), pair[1].source_sequence())
            }));
        }
        inbox.prune_committed(GameTick::new(tick));
        assert!(inbox.len() <= 32);
    }
});
