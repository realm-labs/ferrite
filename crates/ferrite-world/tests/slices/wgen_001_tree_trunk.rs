use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Axis;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use ferrite_world::generation::feature::tree_trunk::{
    TrunkWorld, place_mega_jungle_trunk, place_straight_trunk,
};
use ferrite_world::id::BlockStateId;

#[test]
fn straight_calls_below_provider_then_visits_every_log_and_returns_top_attachment() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = TrunkFixture::new();
    let mut random = ScriptedRandom::new([]);
    let mut context = TreePlacementContext::new(&mut world);

    let attachments = place_straight_trunk(&mut context, &mut random, 2, origin).unwrap();

    assert_eq!(
        context.world().events,
        [
            (BlockPos::new(0, 19, 0), BlockStateId::new(3)),
            (origin, BlockStateId::new(2)),
            (BlockPos::new(0, 21, 0), BlockStateId::new(2)),
        ]
    );
    assert_eq!(attachments[0].position, BlockPos::new(0, 22, 0));
}

#[test]
fn mega_jungle_consumes_initial_branch_height_even_when_no_branch_qualifies() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = TrunkFixture::new();
    let mut random = ScriptedRandom::new([0]);
    let mut context = TreePlacementContext::new(&mut world);

    let attachments = place_mega_jungle_trunk(&mut context, &mut random, 2, origin).unwrap();

    assert_eq!(random.bounds, [4]);
    assert_eq!(attachments.len(), 1);
    assert!(attachments[0].double_trunk);
}

#[derive(Debug)]
struct TrunkFixture {
    events: Vec<(BlockPos, BlockStateId)>,
}

impl TrunkFixture {
    fn new() -> Self {
        Self { events: Vec::new() }
    }
}

impl TreeWorld for TrunkFixture {
    fn minimum_y(&self) -> i32 {
        0
    }
    fn maximum_y(&self) -> i32 {
        255
    }
    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        BlockStateId::new(0)
    }
    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
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
    fn offer_tree_block(&mut self, position: BlockPos, state: BlockStateId, _flags: u32) -> bool {
        self.events.push((position, state));
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

impl TrunkWorld for TrunkFixture {
    fn sample_below_trunk<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        _random: &mut R,
    ) -> Option<BlockStateId> {
        self.events.push((position, BlockStateId::new(3)));
        None
    }

    fn sample_trunk<R: GenerationRandom>(
        &mut self,
        _position: BlockPos,
        _random: &mut R,
    ) -> BlockStateId {
        BlockStateId::new(2)
    }

    fn with_trunk_axis(&self, state: BlockStateId, _axis: Axis) -> BlockStateId {
        state
    }

    fn can_upward_branch_grow_through(&self, _state: BlockStateId) -> bool {
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl ScriptedRandom {
    fn new(integers: impl IntoIterator<Item = u32>) -> Self {
        Self {
            integers: integers.into_iter().collect(),
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
        panic!("fixture does not draw floats")
    }
    fn next_f64(&mut self) -> f64 {
        panic!("fixture does not draw doubles")
    }
    fn next_gaussian(&mut self) -> f64 {
        panic!("fixture does not draw Gaussian values")
    }
}
