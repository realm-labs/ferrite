use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use ferrite_world::generation::feature::tree_decorator_vines::{
    VineDecoratorWorld, decorate_leaf_vines, decorate_pale_moss, decorate_trunk_vines,
};
use ferrite_world::id::BlockStateId;

#[test]
fn trunk_vines_draw_before_reads_in_west_east_north_south_order() {
    let log = BlockPos::new(3, 7, 9);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_trunk(log, LOG);
    context.world().clear_observations();
    let mut random = ScriptedRandom::new([0, 1, 0, 2], []);

    decorate_trunk_vines(&mut context, &mut random).unwrap();

    assert_eq!(random.bounds, [3, 3, 3, 3]);
    assert_eq!(
        context.world().reads,
        [BlockPos::new(4, 7, 9), BlockPos::new(3, 7, 10)]
    );
    assert_eq!(
        context.world().offers,
        [
            (BlockPos::new(4, 7, 9), VINE_WEST, 19),
            (BlockPos::new(3, 7, 10), VINE_NORTH, 19),
        ]
    );
}

#[test]
fn leaf_vine_reads_the_sixth_cell_after_five_offers() {
    let leaf = BlockPos::new(0, 10, 0);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_foliage(leaf, LEAF);
    context.world().clear_observations();
    let mut random = ScriptedRandom::new([], [0.0, 1.0, 1.0, 1.0]);

    decorate_leaf_vines(&mut context, 0.5, &mut random).unwrap();

    assert_eq!(context.world().offers.len(), 5);
    assert_eq!(
        context.world().offers[0],
        (BlockPos::new(-1, 10, 0), VINE_EAST, 19)
    );
    assert_eq!(context.world().reads.last(), Some(&BlockPos::new(-1, 5, 0)));
    assert_eq!(context.world().reads.len(), 6);
}

#[test]
fn pale_moss_with_no_logs_consumes_no_probability_draw() {
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::default();

    decorate_pale_moss(&mut context, 0.15, 0.4, 0.8, &mut random).unwrap();

    assert!(random.floats.is_empty());
    assert!(context.world().patches.is_empty());
}

const AIR: BlockStateId = BlockStateId::new(0);
const LOG: BlockStateId = BlockStateId::new(1);
const LEAF: BlockStateId = BlockStateId::new(2);
const VINE_EAST: BlockStateId = BlockStateId::new(3);
const VINE_WEST: BlockStateId = BlockStateId::new(4);
const VINE_SOUTH: BlockStateId = BlockStateId::new(5);
const VINE_NORTH: BlockStateId = BlockStateId::new(6);

#[derive(Debug, Default)]
struct Fixture {
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    patches: Vec<BlockPos>,
}

impl Fixture {
    fn clear_observations(&mut self) {
        self.reads.clear();
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

impl VineDecoratorWorld for Fixture {
    fn vine_with_face(&self, face: Direction) -> BlockStateId {
        match face {
            Direction::East => VINE_EAST,
            Direction::West => VINE_WEST,
            Direction::South => VINE_SOUTH,
            Direction::North => VINE_NORTH,
            Direction::Down | Direction::Up => panic!("vine face must be horizontal"),
        }
    }

    fn pale_hanging_moss(&self, tip: bool) -> BlockStateId {
        BlockStateId::new(10 + u32::from(tip))
    }

    fn try_place_registered_pale_moss_patch(
        &mut self,
        position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) {
        self.patches.push(position);
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
