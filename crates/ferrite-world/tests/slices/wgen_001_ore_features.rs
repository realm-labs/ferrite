use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::ore::{
    OreConfig, OreTargetRule, OreVolumeWorld, ScatteredOreWorld, place_ore, place_scattered_ore,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn scattered_ore_consumes_six_offsets_then_checks_neighbors_in_fixed_order() {
    let origin = BlockPos::new(2, 20, -3);
    let ore = BlockStateId::new(7);
    let mut world = OreFixture {
        reads: Vec::new(),
        offers: Vec::new(),
    };
    let mut targets = [AlwaysMatch { output: ore }];
    let mut random = ScriptedRandom {
        integers: [1].into_iter().collect(),
        floats: [0.9, 0.1, 0.8, 0.2, 0.7, 0.3].into_iter().collect(),
        doubles: VecDeque::new(),
        bounds: Vec::new(),
        float_draws: 0,
    };
    assert!(
        place_scattered_ore(
            &mut world,
            origin,
            1,
            1.0,
            &mut targets,
            &mut random,
            |_| true
        )
        .unwrap()
    );
    assert_eq!(random.bounds, [2]);
    assert_eq!(random.float_draws, 6);
    assert_eq!(
        world.reads,
        [
            origin,
            BlockPos::new(2, 19, -3),
            BlockPos::new(2, 21, -3),
            BlockPos::new(2, 20, -4),
            BlockPos::new(2, 20, -2),
            BlockPos::new(1, 20, -3),
            BlockPos::new(3, 20, -3),
        ]
    );
    assert_eq!(world.offers, [(origin, ore, 2)]);
}

#[test]
fn ore_size_zero_still_draws_endpoints_and_releases_bulk_access_after_surface_dispatch() {
    let mut world = VolumeFixture::default();
    let mut targets = [AlwaysMatch {
        output: BlockStateId::new(7),
    }];
    let mut random = ScriptedRandom {
        integers: [0, 0].into_iter().collect(),
        floats: [0.0].into_iter().collect(),
        doubles: VecDeque::new(),
        bounds: Vec::new(),
        float_draws: 0,
    };
    assert!(
        !place_ore(
            &mut world,
            BlockPos::new(0, 20, 0),
            OreConfig {
                size: 0,
                discard_chance_on_air_exposure: 0.0,
            },
            &mut targets,
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(random.bounds, [3, 3]);
    assert_eq!(random.float_draws, 1);
    assert!(world.released);
    assert!(world.direct_writes.is_empty());
}

#[test]
fn ore_volume_prunes_then_direct_writes_unique_geometric_cells() {
    let mut world = VolumeFixture::default();
    let ore = BlockStateId::new(7);
    let mut targets = [AlwaysMatch { output: ore }];
    let mut random = ScriptedRandom {
        integers: [0, 0].into_iter().collect(),
        floats: [0.0].into_iter().collect(),
        doubles: vec![0.99; 16].into_iter().collect(),
        bounds: Vec::new(),
        float_draws: 0,
    };
    assert!(
        place_ore(
            &mut world,
            BlockPos::new(0, 20, 0),
            OreConfig {
                size: 16,
                discard_chance_on_air_exposure: 0.0,
            },
            &mut targets,
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert!(!world.direct_writes.is_empty());
    let unique = world
        .direct_writes
        .iter()
        .map(|write| write.0)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), world.direct_writes.len());
    assert!(world.direct_writes.iter().all(|write| write.1 == ore));
    assert!(world.released);
}

#[derive(Debug)]
struct OreFixture {
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl ScatteredOreWorld for OreFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        BlockStateId::new(1)
    }

    fn is_air(&self, _state: BlockStateId) -> bool {
        false
    }

    fn offer_ore(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug, Default)]
struct VolumeFixture {
    direct_writes: Vec<(BlockPos, BlockStateId)>,
    released: bool,
}

impl ScatteredOreWorld for VolumeFixture {
    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        BlockStateId::new(1)
    }

    fn is_air(&self, _state: BlockStateId) -> bool {
        false
    }

    fn offer_ore(&mut self, _position: BlockPos, _state: BlockStateId, _flags: u32) -> bool {
        panic!("ordinary ore uses direct section writes")
    }
}

impl OreVolumeWorld for VolumeFixture {
    fn ocean_floor_worldgen_height(&mut self, _x: i32, _z: i32) -> i32 {
        100
    }

    fn is_outside_build_height(&self, _position: BlockPos) -> bool {
        false
    }

    fn can_write_ore(&mut self, _position: BlockPos) -> bool {
        true
    }

    fn acquire_ore_section(&mut self, _position: BlockPos) -> bool {
        true
    }

    fn set_ore_state_direct(&mut self, position: BlockPos, state: BlockStateId) {
        self.direct_writes.push((position, state));
    }

    fn release_ore_sections(&mut self) {
        self.released = true;
    }
}

#[derive(Debug)]
struct AlwaysMatch {
    output: BlockStateId,
}

impl OreTargetRule<ScriptedRandom> for AlwaysMatch {
    fn matches(&mut self, _state: BlockStateId, _random: &mut ScriptedRandom) -> bool {
        true
    }

    fn output_state(&self) -> BlockStateId {
        self.output
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    doubles: VecDeque<f64>,
    bounds: Vec<u32>,
    float_draws: usize,
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        self.floats.pop_front().expect("scripted float")
    }

    fn next_f64(&mut self) -> f64 {
        self.doubles.pop_front().expect("scripted double")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("scattered ore does not draw Gaussian values")
    }
}
