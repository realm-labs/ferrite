use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::structure::{
    BonusChestRandom, BonusChestStates, BonusChestWorld, place_bonus_chest,
};
use ferrite_world::id::BlockStateId;

#[test]
fn bonus_chest_shuffles_x_then_z_before_scanning_and_conditionally_draws_loot_seed() {
    let origin = BlockPos::new(-1, 40, -1);
    let states = BonusChestStates {
        chest: BlockStateId::new(1),
        torch: BlockStateId::new(2),
    };
    let identity_shuffle = (1..=15).rev();
    let mut random = ScriptedRandom {
        integers: identity_shuffle.clone().chain(identity_shuffle).collect(),
        bounds: Vec::new(),
        long_draws: 0,
    };
    let mut world = BonusChestFixture::default();
    assert!(place_bonus_chest(&mut world, origin, states, &mut random, |_| true).unwrap());
    assert_eq!(
        random.bounds,
        (2..=16).rev().chain((2..=16).rev()).collect::<Vec<_>>()
    );
    assert_eq!(random.long_draws, 1);
    let chest_position = BlockPos::new(-16, 70, -16);
    assert_eq!(world.surface_queries, [(-16, -16)]);
    assert_eq!(world.empty_reads, [chest_position]);
    assert!(world.state_reads.is_empty());
    assert_eq!(
        world.offers,
        [
            (chest_position, states.chest, 2),
            (BlockPos::new(-16, 70, -17), states.torch, 2),
            (BlockPos::new(-15, 70, -16), states.torch, 2),
            (BlockPos::new(-16, 70, -15), states.torch, 2),
            (BlockPos::new(-17, 70, -16), states.torch, 2),
        ]
    );
    assert_eq!(world.loot, [(chest_position, -7)]);
}

#[derive(Debug, Default)]
struct BonusChestFixture {
    surface_queries: Vec<(i32, i32)>,
    empty_reads: Vec<BlockPos>,
    state_reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    loot: Vec<(BlockPos, i64)>,
}

impl BonusChestWorld for BonusChestFixture {
    fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32 {
        self.surface_queries.push((x, z));
        70
    }

    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        self.empty_reads.push(position);
        true
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.state_reads.push(position);
        BlockStateId::new(0)
    }

    fn has_empty_collision_shape(&self, _state: BlockStateId, _position: BlockPos) -> bool {
        true
    }

    fn offer_bonus_chest(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn has_randomizable_container(&mut self, _position: BlockPos) -> bool {
        true
    }

    fn assign_bonus_chest_loot(&mut self, position: BlockPos, seed: i64) {
        self.loot.push((position, seed));
    }

    fn torch_can_survive(&mut self, _position: BlockPos, _torch: BlockStateId) -> bool {
        true
    }

    fn offer_bonus_torch(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    bounds: Vec<u32>,
    long_draws: usize,
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        panic!("bonus chest does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("bonus chest does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("bonus chest does not draw Gaussian values")
    }
}

impl BonusChestRandom for ScriptedRandom {
    fn next_i64(&mut self) -> i64 {
        self.long_draws += 1;
        -7
    }
}
