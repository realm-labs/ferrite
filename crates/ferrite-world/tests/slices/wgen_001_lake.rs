use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::lake::{LakeWorld, place_lake};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn lake_minimum_height_gate_precedes_every_random_draw_and_provider() {
    let origin = BlockPos::new(0, 4, 0);
    let mut world = LakeFixture::new();
    let mut random = ScriptedRandom::new([], []);

    assert!(!place_lake(&mut world, origin, &mut random, |_| true).unwrap());
    assert!(random.bounds.is_empty());
    assert_eq!(random.double_draws, 0);
    assert_eq!(world.fluid_samples, 0);
    assert_eq!(world.barrier_samples, 0);
}

#[test]
fn lake_boundary_failure_is_atomic_and_skips_barrier_sampling() {
    let origin = BlockPos::new(8, 20, 8);
    let mut world = LakeFixture::new();
    world.all_liquid = true;
    let mut random = centered_mask_random();

    assert!(!place_lake(&mut world, origin, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [4]);
    assert_eq!(random.double_draws, 24);
    assert_eq!(world.fluid_samples, 1);
    assert_eq!(world.barrier_samples, 0);
    assert!(world.offers.is_empty());
}

#[test]
fn cave_air_followups_are_offer_keyed_and_air_barrier_skips_coating_rng() {
    let origin = BlockPos::new(8, 20, 8);
    let base = BlockPos::new(0, 16, 0);
    let cavity = BlockPos::new(8, 20, 8);
    let mut world = LakeFixture::new();
    world.only_replace = Some(cavity);
    world.air_barrier = true;
    let mut random = centered_mask_random();

    assert!(place_lake(&mut world, origin, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [4]);
    assert_eq!(random.double_draws, 24);
    assert_eq!(world.provider_positions, [base, base]);
    assert_eq!(
        world.offers,
        [(cavity, world.cave_air(), 2)],
        "a rejected offer still owns its tick and postprocessing work"
    );
    assert_eq!(world.ticks, [(cavity, world.cave_air())]);
    assert_eq!(
        world.marks,
        [BlockPos::new(8, 21, 8), BlockPos::new(8, 22, 8)]
    );
}

fn centered_mask_random() -> ScriptedRandom {
    ScriptedRandom::new([0], [0.5; 24])
}

#[derive(Debug)]
struct LakeFixture {
    all_liquid: bool,
    only_replace: Option<BlockPos>,
    air_barrier: bool,
    fluid_samples: usize,
    barrier_samples: usize,
    provider_positions: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    ticks: Vec<(BlockPos, BlockStateId)>,
    marks: Vec<BlockPos>,
}

impl LakeFixture {
    fn new() -> Self {
        Self {
            all_liquid: false,
            only_replace: None,
            air_barrier: false,
            fluid_samples: 0,
            barrier_samples: 0,
            provider_positions: Vec::new(),
            offers: Vec::new(),
            ticks: Vec::new(),
            marks: Vec::new(),
        }
    }
}

impl LakeWorld for LakeFixture {
    fn minimum_y(&self) -> i32 {
        0
    }

    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        if self.all_liquid {
            BlockStateId::new(7)
        } else {
            BlockStateId::new(9)
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_liquid(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(7)
    }

    fn is_solid(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(9)
    }

    fn cave_air(&self) -> BlockStateId {
        BlockStateId::new(1)
    }

    fn default_ice(&self) -> BlockStateId {
        BlockStateId::new(4)
    }

    fn sample_fluid<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        _random: &mut R,
    ) -> BlockStateId {
        self.fluid_samples += 1;
        self.provider_positions.push(position);
        BlockStateId::new(7)
    }

    fn sample_barrier<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        _random: &mut R,
    ) -> BlockStateId {
        self.barrier_samples += 1;
        self.provider_positions.push(position);
        if self.air_barrier {
            BlockStateId::new(0)
        } else {
            BlockStateId::new(8)
        }
    }

    fn can_place_feature(&mut self, _position: BlockPos) -> bool {
        true
    }

    fn can_replace_with_air_or_fluid(&mut self, position: BlockPos) -> bool {
        self.only_replace.is_none_or(|allowed| position == allowed)
    }

    fn can_replace_with_barrier(&mut self, _position: BlockPos) -> bool {
        true
    }

    fn fluid_is_water_tagged(&self, _state: BlockStateId) -> bool {
        false
    }

    fn should_freeze_without_edge(&mut self, _position: BlockPos) -> bool {
        panic!("nonwater fixture must not run the freeze pass")
    }

    fn offer_lake_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn schedule_zero_delay_tick(&mut self, position: BlockPos, block: BlockStateId) {
        self.ticks.push((position, block));
    }

    fn mark_for_postprocessing(&mut self, position: BlockPos) {
        self.marks.push(position);
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    doubles: VecDeque<f64>,
    bounds: Vec<u32>,
    double_draws: usize,
}

impl ScriptedRandom {
    fn new(
        integers: impl IntoIterator<Item = u32>,
        doubles: impl IntoIterator<Item = f64>,
    ) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            doubles: doubles.into_iter().collect(),
            bounds: Vec::new(),
            double_draws: 0,
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        panic!("lake feature does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        self.double_draws += 1;
        self.doubles.pop_front().expect("scripted double")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("lake feature does not draw Gaussian values")
    }
}
