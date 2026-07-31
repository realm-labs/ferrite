use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use ferrite_world::generation::feature::tree_decorator_attachments::{
    AttachedToLeavesConfig, AttachedToLogsConfig, AttachmentDecoratorWorld,
    decorate_attached_to_leaves, decorate_attached_to_logs,
};
use ferrite_world::id::BlockStateId;

#[test]
fn leaves_choose_direction_before_exclusion_and_skip_the_second_float() {
    let left = BlockPos::new(0, 8, 0);
    let right = BlockPos::new(2, 8, 0);
    let center = BlockPos::new(1, 8, 0);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_foliage(left, LEAF);
    context.offer_foliage(right, LEAF);
    let ordered = context.ordered_foliage();
    let choices = ordered
        .iter()
        .map(|position| u32::from(position.x == right.x))
        .collect::<Vec<_>>();
    context.world().clear();
    let mut random = ScriptedRandom::new([1, choices[0], choices[1]], [0.0]);
    let config = AttachedToLeavesConfig {
        probability: 1.0,
        exclusion_radius_xz: 1,
        exclusion_radius_y: 0,
        required_empty_blocks: 2,
        directions: vec![Direction::East, Direction::West],
    };

    decorate_attached_to_leaves(&mut context, &config, &mut random).unwrap();

    assert_eq!(random.bounds, [2, 2, 2]);
    assert!(random.floats.is_empty());
    assert_eq!(context.world().reads.len(), 2);
    assert_eq!(context.world().samples, [center]);
    assert_eq!(context.world().offers, [(center, ATTACHED_LEAF, 19)]);
}

#[test]
fn logs_use_inclusive_probability_and_draw_singleton_direction() {
    let log = BlockPos::new(4, 5, 6);
    let target = BlockPos::new(4, 6, 6);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_trunk(log, LOG);
    context.world().clear();
    let mut random = ScriptedRandom::new([0], [0.5]);
    let config = AttachedToLogsConfig {
        probability: 0.5,
        directions: vec![Direction::Up],
    };

    decorate_attached_to_logs(&mut context, &config, &mut random).unwrap();

    assert_eq!(random.bounds, [1]);
    assert_eq!(context.world().reads, [target]);
    assert_eq!(context.world().samples, [target]);
    assert_eq!(context.world().offers, [(target, ATTACHED_LOG, 19)]);
}

const AIR: BlockStateId = BlockStateId::new(0);
const LOG: BlockStateId = BlockStateId::new(1);
const LEAF: BlockStateId = BlockStateId::new(2);
const ATTACHED_LEAF: BlockStateId = BlockStateId::new(3);
const ATTACHED_LOG: BlockStateId = BlockStateId::new(4);

#[derive(Debug, Default)]
struct Fixture {
    reads: Vec<BlockPos>,
    samples: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl Fixture {
    fn clear(&mut self) {
        self.reads.clear();
        self.samples.clear();
        self.offers.clear();
    }
}

impl TreeWorld for Fixture {
    fn minimum_y(&self) -> i32 {
        0
    }

    fn maximum_y(&self) -> i32 {
        255
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        AIR
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == AIR
    }

    fn is_replaceable_by_trees(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_log(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_vine(&self, _state: BlockStateId) -> bool {
        false
    }

    fn optional_leaf_distance(&self, _state: BlockStateId) -> Option<u8> {
        None
    }

    fn with_leaf_distance(&self, state: BlockStateId, _distance: u8) -> BlockStateId {
        state
    }

    fn offer_tree_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn update_tree_shape_at_edge(
        &mut self,
        _radius: u32,
        _minimum: BlockPos,
        _maximum: BlockPos,
        _filled: &[BlockPos],
    ) {
    }
}

impl AttachmentDecoratorWorld for Fixture {
    fn sample_attached_to_leaves(
        &mut self,
        position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) -> BlockStateId {
        self.samples.push(position);
        ATTACHED_LEAF
    }

    fn sample_attached_to_logs(
        &mut self,
        position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) -> BlockStateId {
        self.samples.push(position);
        ATTACHED_LOG
    }
}

#[derive(Debug, Default)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    bounds: Vec<u32>,
}

impl ScriptedRandom {
    fn new(integers: impl IntoIterator<Item = u32>, floats: impl IntoIterator<Item = f32>) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            floats: floats.into_iter().collect(),
            bounds: Vec::new(),
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
        self.floats.pop_front().expect("scripted float")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("fixture does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("fixture does not draw Gaussian values")
    }
}
