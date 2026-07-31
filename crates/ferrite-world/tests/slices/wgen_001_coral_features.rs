use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::coral::{CoralShape, CoralWorld, place_coral_feature};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn coral_tree_shared_cell_uses_two_top_draws_then_four_wall_draws() {
    let origin = BlockPos::new(0, 20, 0);
    let coral = BlockStateId::new(1);
    let mut world = CoralFixture::new(origin, coral);
    let mut random = ScriptedRandom::new([0, 0, 0, 3, 2, 1, 0, 0], [0.3, 0.1, 1.0, 1.0, 1.0, 1.0]);
    assert!(
        place_coral_feature(&mut world, origin, CoralShape::Tree, &mut random, |_| true).unwrap()
    );
    assert_eq!(random.bounds, [1, 3, 3, 4, 3, 2, 5, 5]);
    assert_eq!(random.float_draws, 6);
    assert_eq!(world.offers, [(origin, coral, 3)]);
}

#[test]
fn coral_mushroom_draws_only_for_face_interiors_and_skips_below_point_one() {
    let origin = BlockPos::new(0, 20, 0);
    let coral = BlockStateId::new(1);
    let mut world = CoralFixture::new(origin, coral);
    let mut random = ScriptedRandom::new([0, 0, 0, 0, 0], [0.0; 24]);
    assert!(
        place_coral_feature(
            &mut world,
            origin,
            CoralShape::Mushroom,
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(random.bounds, [1, 3, 3, 3, 3]);
    assert_eq!(random.float_draws, 24);
    assert!(world.offers.is_empty());
}

#[test]
fn coral_claw_uses_forward_then_side_arm_specific_draw_sequences() {
    let origin = BlockPos::new(0, 20, 0);
    let coral = BlockStateId::new(1);
    let mut world = CoralFixture::new(origin, coral);
    let mut random = ScriptedRandom::new(
        [0, 0, 0, 2, 1, 0, 0, 0, 0, 0],
        [0.3, 0.1, 1.0, 1.0, 1.0, 1.0],
    );
    assert!(
        place_coral_feature(&mut world, origin, CoralShape::Claw, &mut random, |_| true).unwrap()
    );
    assert_eq!(random.bounds, [1, 4, 2, 3, 2, 2, 3, 2, 2, 3]);
    assert_eq!(random.float_draws, 6);
    assert_eq!(world.offers, [(origin, coral, 3)]);
}

#[derive(Debug)]
struct CoralFixture {
    origin: BlockPos,
    coral_blocks: Vec<BlockStateId>,
    corals: Vec<BlockStateId>,
    wall_corals: Vec<BlockStateId>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl CoralFixture {
    fn new(origin: BlockPos, coral: BlockStateId) -> Self {
        Self {
            origin,
            coral_blocks: vec![coral],
            corals: vec![BlockStateId::new(2)],
            wall_corals: vec![BlockStateId::new(3)],
            offers: Vec::new(),
        }
    }
}

impl CoralWorld for CoralFixture {
    fn coral_blocks(&self) -> &[BlockStateId] {
        &self.coral_blocks
    }

    fn corals(&self) -> &[BlockStateId] {
        &self.corals
    }

    fn wall_corals(&self) -> &[BlockStateId] {
        &self.wall_corals
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position == self.origin || position == BlockPos::new(0, 21, 0) {
            BlockStateId::new(0)
        } else {
            BlockStateId::new(9)
        }
    }

    fn is_exact_water(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_coral(&self, state: BlockStateId) -> bool {
        self.coral_blocks.contains(&state)
    }

    fn sea_pickle_state(&self, count: u8) -> BlockStateId {
        BlockStateId::new(20 + u32::from(count))
    }

    fn wall_coral_facing(&self, state: BlockStateId, _direction: Direction) -> BlockStateId {
        state
    }

    fn offer_coral_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
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
        panic!("coral features do not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("coral features do not draw Gaussian values")
    }
}
