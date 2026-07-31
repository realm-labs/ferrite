use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::basalt_columns::{
    BasaltColumnsConfig, BasaltColumnsWorld, place_basalt_columns,
};
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn basalt_columns_sparse_branch_keeps_xyz_draws_and_eager_candidate_calls() {
    let origin = BlockPos::new(0, 10, 0);
    let basalt = BlockStateId::new(3);
    let mut world = BasaltFixture { offers: Vec::new() };
    let mut random = ScriptedRandom {
        integers: vec![0; 45].into_iter().collect(),
        floats: [0.9].into_iter().collect(),
        bounds: Vec::new(),
    };
    assert!(
        place_basalt_columns(
            &mut world,
            origin,
            &BasaltColumnsConfig {
                reach: IntProvider::Constant(1),
                height: IntProvider::Constant(0),
                basalt,
            },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(random.bounds, vec![1; 45]);
    assert_eq!(world.offers.len(), 60);
    assert_eq!(
        &world.offers[..4],
        [
            (BlockPos::new(0, 9, -1), basalt, 3),
            (BlockPos::new(-1, 9, 0), basalt, 3),
            (BlockPos::new(1, 9, 0), basalt, 3),
            (BlockPos::new(0, 9, 1), basalt, 3),
        ]
    );
}

#[derive(Debug)]
struct BasaltFixture {
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl BasaltColumnsWorld for BasaltFixture {
    fn sea_level(&self) -> i32 {
        32
    }

    fn minimum_y(&self) -> i32 {
        -64
    }

    fn maximum_y(&self) -> i32 {
        319
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position.y <= 8 || position == BlockPos::new(0, 9, 0) {
            BlockStateId::new(1)
        } else {
            BlockStateId::new(0)
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_exact_lava(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(2)
    }

    fn is_exact_basalt(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(3)
    }

    fn is_banned_basalt_support(&self, _state: BlockStateId) -> bool {
        false
    }

    fn offer_basalt_column(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    bounds: Vec<u32>,
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.floats.pop_front().expect("scripted float")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("basalt columns does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("basalt columns does not draw Gaussian values")
    }
}
