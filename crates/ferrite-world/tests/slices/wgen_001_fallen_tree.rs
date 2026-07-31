use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::{Axis, Direction};
use ferrite_world::generation::feature::fallen_tree::{
    FallenTreeConfig, FallenTreeDecorator, FallenTreeWorld, WeightedBlockState, place_fallen_tree,
};
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn stump_vines_draw_before_direction_length_and_start_offset() {
    let origin = BlockPos::new(0, 20, 0);
    let config = FallenTreeConfig {
        log_length: IntProvider::Constant(2),
        stump_decorators: vec![FallenTreeDecorator::TrunkVine],
        log_decorators: Vec::new(),
    };
    let mut world = FallenFixture::new(origin);
    let mut random = ScriptedRandom::new([0, 1, 0, 1, 0, 0], []);

    assert!(place_fallen_tree(&mut world, origin, &config, &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [3, 3, 3, 3, 4, 2]);
    assert_eq!(
        world.offers,
        [
            (origin, BlockStateId::new(10), 3),
            (BlockPos::new(1, 20, 0), BlockStateId::new(24), 19),
            (BlockPos::new(0, 20, 1), BlockStateId::new(22), 19),
        ]
    );
}

#[test]
fn horizontal_log_axis_and_attached_provider_follow_validation() {
    let origin = BlockPos::new(0, 20, 0);
    let config = FallenTreeConfig {
        log_length: IntProvider::Constant(3),
        stump_decorators: Vec::new(),
        log_decorators: vec![FallenTreeDecorator::AttachedToLogs {
            probability: 0.1,
            directions: vec![Direction::Up],
            provider: vec![
                WeightedBlockState {
                    state: BlockStateId::new(30),
                    weight: NonZeroU32::new(2).unwrap(),
                },
                WeightedBlockState {
                    state: BlockStateId::new(31),
                    weight: NonZeroU32::new(1).unwrap(),
                },
            ],
        }],
    };
    let mut world = FallenFixture::new(origin);
    let mut random = ScriptedRandom::new([1, 0, 0, 2], [0.1]);

    assert!(place_fallen_tree(&mut world, origin, &config, &mut random, |_| true).unwrap());

    let log = BlockPos::new(2, 20, 0);
    assert_eq!(random.bounds, [4, 2, 1, 3]);
    assert_eq!(random.float_draws, 1);
    assert_eq!(
        world.offers,
        [
            (origin, BlockStateId::new(10), 3),
            (log, BlockStateId::new(11), 3),
            (BlockPos::new(2, 21, 0), BlockStateId::new(31), 19),
        ]
    );
}

#[derive(Debug)]
struct FallenFixture {
    origin: BlockPos,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl FallenFixture {
    fn new(origin: BlockPos) -> Self {
        Self {
            origin,
            offers: Vec::new(),
        }
    }
}

impl FallenTreeWorld for FallenFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position.y >= self.origin.y {
            BlockStateId::new(0)
        } else {
            BlockStateId::new(9)
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_replaceable_by_trees(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_upward_face_sturdy_at(
        &mut self,
        _below_position: BlockPos,
        below_state: BlockStateId,
        _queried_position: BlockPos,
    ) -> bool {
        below_state == BlockStateId::new(9)
    }

    fn sample_trunk<R: GenerationRandom>(
        &mut self,
        _position: BlockPos,
        _random: &mut R,
    ) -> BlockStateId {
        BlockStateId::new(10)
    }

    fn with_log_axis(&self, state: BlockStateId, axis: Axis) -> BlockStateId {
        if axis == Axis::X {
            BlockStateId::new(state.get() + 1)
        } else {
            state
        }
    }

    fn vine_with_face(&self, face: Direction) -> BlockStateId {
        BlockStateId::new(20 + face as u32)
    }

    fn offer_fallen_tree(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn mark_for_postprocessing(&mut self, _position: BlockPos) {}
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    bounds: Vec<u32>,
    float_draws: usize,
}

impl ScriptedRandom {
    fn new(integers: impl IntoIterator<Item = u32>, floats: impl IntoIterator<Item = f32>) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            floats: floats.into_iter().collect(),
            bounds: Vec::new(),
            float_draws: 0,
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
        self.float_draws += 1;
        self.floats.pop_front().expect("scripted float")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("fallen tree does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("fallen tree does not draw Gaussian values")
    }
}
