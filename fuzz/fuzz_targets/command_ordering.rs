#![no_main]

use ferrite_foundation::identity::{DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::command::{CommandInbox, CommandSource, RegionCommand};
use ferrite_simulation::tick::GameTick;
use libfuzzer_sys::fuzz_target;

fn region() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).expect("fixed world is valid"),
        DimensionId::new(ResourceId::minecraft("overworld").expect("fixed dimension is valid")),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

fn source(tag: u8, id: u8) -> CommandSource {
    match tag % 3 {
        0 => CommandSource::System(ResourceId::new("ferrite", "fuzz/system").unwrap()),
        1 => CommandSource::Player(StableEntityId::new(u128::from(id) + 1).unwrap()),
        _ => CommandSource::Region(region()),
    }
}

fuzz_target!(|data: &[u8]| {
    let target = region();
    let mut inbox = CommandInbox::new(target.clone(), 32, 8).unwrap();
    for operation in data.chunks(5).take(128) {
        let tag = operation.first().copied().unwrap_or_default();
        let id = operation.get(1).copied().unwrap_or_default();
        let tick = GameTick::new(u64::from(
            operation.get(2).copied().unwrap_or_default() % 10,
        ));
        let sequence = u64::from(operation.get(3).copied().unwrap_or_default());
        let payload = operation.get(4..).unwrap_or_default().to_vec();
        let command = RegionCommand::new(
            target.clone(),
            tick,
            source(tag, id),
            sequence,
            ResourceId::new("ferrite", "fuzz/command").unwrap(),
            payload,
        )
        .unwrap();
        let _ = inbox.admit(command, GameTick::ZERO);
        assert!(inbox.len() <= 32);
    }

    for tick in 1..=8 {
        let drained = inbox.drain_tick(GameTick::new(tick));
        assert!(drained.windows(2).all(|pair| {
            (pair[0].source(), pair[0].sequence()) <= (pair[1].source(), pair[1].sequence())
        }));
        inbox.prune_committed(GameTick::new(tick));
        assert!(inbox.len() <= 32);
    }
});
