use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::end_spike::{
    EndSpike, EndSpikeConfig, EndSpikeWorld, IronBarConnections, derive_end_spikes,
    place_end_spikes,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn derived_end_spikes_keep_angular_order_and_assign_each_dimension_value_once() {
    let spikes = derive_end_spikes(123_456_789);
    assert_eq!(spikes.len(), 10);
    let mut values = spikes
        .iter()
        .map(|spike| (spike.height - 76) / 3)
        .collect::<Vec<_>>();
    values.sort_unstable();
    assert_eq!(values, (0..10).collect::<Vec<_>>());
    assert_eq!(spikes[0].center_x, -84);
    assert_eq!(spikes.iter().filter(|spike| spike.guarded).count(), 2);
}

#[test]
fn end_spike_add_failure_still_places_bedrock_and_position_derived_fire() {
    let spike = EndSpike {
        center_x: 0,
        center_z: 0,
        radius: 0,
        height: 66,
        guarded: false,
    };
    let config = EndSpikeConfig {
        spikes: vec![spike],
        crystal_invulnerable: true,
        beam_target: Some(BlockPos::new(0, 128, 0)),
        obsidian: BlockStateId::new(1),
        air: BlockStateId::new(2),
        iron_bars: BlockStateId::new(3),
        bedrock: BlockStateId::new(4),
    };
    let mut world = SpikeFixture {
        offers: Vec::new(),
        crystal_calls: Vec::new(),
    };
    let mut random = ScriptedRandom {
        floats: [0.5].into_iter().collect(),
    };
    assert!(
        place_end_spikes(
            &mut world,
            BlockPos::new(0, 45, 0),
            &config,
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(world.offers.len(), 14);
    assert_eq!(
        world.offers[0],
        (BlockPos::new(0, 65, 0), config.obsidian, 3)
    );
    assert_eq!(
        &world.offers[12..],
        [
            (BlockPos::new(0, 66, 0), config.bedrock, 3),
            (BlockPos::new(0, 67, 0), BlockStateId::new(5), 3),
        ]
    );
    assert_eq!(
        world.crystal_calls,
        [([0.5, 67.0, 0.5], 180.0, config.beam_target, true)]
    );
}

#[derive(Debug)]
struct SpikeFixture {
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    crystal_calls: Vec<([f64; 3], f32, Option<BlockPos>, bool)>,
}

impl EndSpikeWorld for SpikeFixture {
    fn world_seed(&self) -> i64 {
        0
    }

    fn minimum_y(&self) -> i32 {
        65
    }

    fn configure_iron_bars(
        &mut self,
        default_state: BlockStateId,
        _connections: IronBarConnections,
    ) -> BlockStateId {
        default_state
    }

    fn can_create_end_crystal(&mut self) -> bool {
        true
    }

    fn add_end_crystal(
        &mut self,
        position: [f64; 3],
        yaw_degrees: f32,
        beam_target: Option<BlockPos>,
        invulnerable: bool,
    ) -> bool {
        self.crystal_calls
            .push((position, yaw_degrees, beam_target, invulnerable));
        false
    }

    fn fire_state(&mut self, _position: BlockPos) -> BlockStateId {
        BlockStateId::new(5)
    }

    fn offer_end_spike_block(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        flags: u32,
    ) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    floats: VecDeque<f32>,
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        panic!("custom End spike does not draw integers")
    }

    fn next_f32(&mut self) -> f32 {
        self.floats.pop_front().expect("scripted float")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("End spike does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("End spike does not draw Gaussian values")
    }
}
