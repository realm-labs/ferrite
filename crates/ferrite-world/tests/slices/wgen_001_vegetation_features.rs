use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::vegetation::{BambooStates, BambooWorld, place_bamboo};
use ferrite_world::id::BlockStateId;

#[test]
fn bamboo_draws_height_probability_radius_then_overwrites_the_obstruction_with_a_crown() {
    let origin = BlockPos::new(0, 10, 0);
    let states = BambooStates {
        trunk: BlockStateId::new(2),
        final_large: BlockStateId::new(3),
        top_large: BlockStateId::new(4),
        top_small: BlockStateId::new(5),
        podzol: BlockStateId::new(6),
    };
    let mut world = BambooFixture {
        origin,
        empty_reads: Vec::new(),
        state_reads: Vec::new(),
        surface_queries: Vec::new(),
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom {
        integers: [0, 0].into_iter().collect(),
        floats: [0.1].into_iter().collect(),
        bounds: Vec::new(),
        float_draws: 0,
    };
    assert!(place_bamboo(&mut world, origin, 0.2, states, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [12, 4]);
    assert_eq!(random.float_draws, 1);
    assert_eq!(
        world.surface_queries,
        [(-1, 0), (0, -1), (0, 0), (0, 1), (1, 0)]
    );
    assert_eq!(
        &world.offers[..5],
        [
            (BlockPos::new(-1, 19, 0), states.podzol, 2),
            (BlockPos::new(0, 19, -1), states.podzol, 2),
            (BlockPos::new(0, 19, 0), states.podzol, 2),
            (BlockPos::new(0, 19, 1), states.podzol, 2),
            (BlockPos::new(1, 19, 0), states.podzol, 2),
        ]
    );
    assert_eq!(
        &world.offers[5..],
        [
            (origin, states.trunk, 2),
            (BlockPos::new(0, 11, 0), states.trunk, 2),
            (BlockPos::new(0, 12, 0), states.trunk, 2),
            (BlockPos::new(0, 13, 0), states.final_large, 2),
            (BlockPos::new(0, 12, 0), states.top_large, 2),
            (BlockPos::new(0, 11, 0), states.top_small, 2),
        ]
    );
    assert_eq!(
        world.empty_reads,
        [
            origin,
            origin,
            BlockPos::new(0, 11, 0),
            BlockPos::new(0, 12, 0),
            BlockPos::new(0, 13, 0),
        ]
    );
}

#[derive(Debug)]
struct BambooFixture {
    origin: BlockPos,
    empty_reads: Vec<BlockPos>,
    state_reads: Vec<BlockPos>,
    surface_queries: Vec<(i32, i32)>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl BambooWorld for BambooFixture {
    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        self.empty_reads.push(position);
        position.y <= self.origin.y + 2
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.state_reads.push(position);
        BlockStateId::new(1)
    }

    fn supports_bamboo(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn world_surface_height(&mut self, x: i32, z: i32) -> i32 {
        self.surface_queries.push((x, z));
        20
    }

    fn beneath_bamboo_podzol_replaceable(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn offer_bamboo_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
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
        panic!("bamboo does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("bamboo does not draw Gaussian values")
    }
}
