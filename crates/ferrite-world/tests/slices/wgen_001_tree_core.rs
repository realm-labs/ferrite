use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::tree_core::{
    FoliageAttachment, TreeCoreConfig, TreeCoreError, TreeFeatureSize, TreePlacementContext,
    TreePlan, TreeWorld, place_tree_core,
};
use ferrite_world::id::BlockStateId;

#[test]
fn zero_height_ranges_still_draw_and_failed_offers_form_the_shape() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = TreeFixture::new();
    let mut plan = OneTrunkPlan::new();
    let mut random = ScriptedRandom::new([0, 0]);

    assert!(
        place_tree_core(&mut world, origin, config(), &mut plan, &mut random, |_| {
            true
        },)
        .unwrap()
    );

    assert_eq!(random.bounds, [1, 1]);
    assert_eq!(plan.seen_height, Some(3));
    assert_eq!(world.offers, [(origin, BlockStateId::new(2), 19)]);
    assert_eq!(world.edge_update, Some((3, origin, origin, vec![origin])));
}

#[test]
fn root_only_output_is_not_a_success_and_skips_decorators_and_repair() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = TreeFixture::new();
    let mut plan = RootOnlyPlan { decorated: false };
    let mut random = ScriptedRandom::new([0, 0]);

    assert!(
        !place_tree_core(&mut world, origin, config(), &mut plan, &mut random, |_| {
            true
        },)
        .unwrap()
    );
    assert!(!plan.decorated);
    assert!(world.edge_update.is_none());
}

fn config() -> TreeCoreConfig {
    TreeCoreConfig {
        base_height: 3,
        height_random_a: 0,
        height_random_b: 0,
        ignore_vines: false,
        size: TreeFeatureSize::TwoLayers {
            limit: 1,
            lower_size: 0,
            upper_size: 0,
            minimum_clipped_height: None,
        },
    }
}

#[derive(Debug)]
struct TreeFixture {
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    edge_update: Option<(u32, BlockPos, BlockPos, Vec<BlockPos>)>,
}

impl TreeFixture {
    fn new() -> Self {
        Self {
            offers: Vec::new(),
            edge_update: None,
        }
    }
}

impl TreeWorld for TreeFixture {
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

    fn offer_tree_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn update_tree_shape_at_edge(
        &mut self,
        radius: u32,
        minimum: BlockPos,
        maximum: BlockPos,
        filled: &[BlockPos],
    ) {
        self.edge_update = Some((radius, minimum, maximum, filled.to_vec()));
    }
}

#[derive(Debug)]
struct OneTrunkPlan {
    seen_height: Option<i32>,
}

impl OneTrunkPlan {
    fn new() -> Self {
        Self { seen_height: None }
    }
}

impl<R: GenerationRandom> TreePlan<R, TreeFixture> for OneTrunkPlan {
    fn foliage_height(&mut self, requested_height: i32, _random: &mut R) -> i32 {
        self.seen_height = Some(requested_height);
        1
    }

    fn foliage_radius(&mut self, _random: &mut R) -> i32 {
        0
    }

    fn trunk_origin(
        &mut self,
        _world: &mut TreeFixture,
        origin: BlockPos,
        _random: &mut R,
    ) -> Result<BlockPos, TreeCoreError> {
        Ok(origin)
    }

    fn place_roots(
        &mut self,
        _context: &mut TreePlacementContext<'_, TreeFixture>,
        _origin: BlockPos,
        _trunk_origin: BlockPos,
        _usable_height: i32,
        _random: &mut R,
    ) -> Result<bool, TreeCoreError> {
        Ok(true)
    }

    fn place_trunk(
        &mut self,
        context: &mut TreePlacementContext<'_, TreeFixture>,
        trunk_origin: BlockPos,
        _usable_height: i32,
        _random: &mut R,
    ) -> Result<Vec<FoliageAttachment>, TreeCoreError> {
        context.offer_trunk(trunk_origin, BlockStateId::new(2));
        Ok(Vec::new())
    }

    fn place_foliage(
        &mut self,
        _context: &mut TreePlacementContext<'_, TreeFixture>,
        _attachment: FoliageAttachment,
        _usable_height: i32,
        _foliage_height: i32,
        _foliage_radius: i32,
        _random: &mut R,
    ) -> Result<(), TreeCoreError> {
        Ok(())
    }

    fn decorate(
        &mut self,
        _context: &mut TreePlacementContext<'_, TreeFixture>,
        _random: &mut R,
    ) -> Result<(), TreeCoreError> {
        Ok(())
    }
}

#[derive(Debug)]
struct RootOnlyPlan {
    decorated: bool,
}

impl<R: GenerationRandom> TreePlan<R, TreeFixture> for RootOnlyPlan {
    fn foliage_height(&mut self, _requested_height: i32, _random: &mut R) -> i32 {
        0
    }

    fn foliage_radius(&mut self, _random: &mut R) -> i32 {
        0
    }

    fn trunk_origin(
        &mut self,
        _world: &mut TreeFixture,
        origin: BlockPos,
        _random: &mut R,
    ) -> Result<BlockPos, TreeCoreError> {
        Ok(origin)
    }

    fn place_roots(
        &mut self,
        context: &mut TreePlacementContext<'_, TreeFixture>,
        origin: BlockPos,
        _trunk_origin: BlockPos,
        _usable_height: i32,
        _random: &mut R,
    ) -> Result<bool, TreeCoreError> {
        context.offer_root(origin, BlockStateId::new(3));
        Ok(true)
    }

    fn place_trunk(
        &mut self,
        _context: &mut TreePlacementContext<'_, TreeFixture>,
        _trunk_origin: BlockPos,
        _usable_height: i32,
        _random: &mut R,
    ) -> Result<Vec<FoliageAttachment>, TreeCoreError> {
        Ok(Vec::new())
    }

    fn place_foliage(
        &mut self,
        _context: &mut TreePlacementContext<'_, TreeFixture>,
        _attachment: FoliageAttachment,
        _usable_height: i32,
        _foliage_height: i32,
        _foliage_radius: i32,
        _random: &mut R,
    ) -> Result<(), TreeCoreError> {
        Ok(())
    }

    fn decorate(
        &mut self,
        _context: &mut TreePlacementContext<'_, TreeFixture>,
        _random: &mut R,
    ) -> Result<(), TreeCoreError> {
        self.decorated = true;
        Ok(())
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
        panic!("tree core fixture does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("tree core fixture does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("tree core fixture does not draw Gaussian values")
    }
}
