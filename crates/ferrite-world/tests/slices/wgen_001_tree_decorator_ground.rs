use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use ferrite_world::generation::feature::tree_decorator_ground::{
    GroundDecoratorWorld, PlaceOnGroundConfig, decorate_alter_ground, decorate_place_on_ground,
};
use ferrite_world::id::BlockStateId;

#[test]
fn alter_ground_completes_four_fixed_circles_before_five_random_draws() {
    let trunk = BlockPos::new(0, 10, 0);
    let mut world = Fixture {
        optional_ground: Some(PODZOL),
        ..Fixture::default()
    };
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_trunk(trunk, LOG);
    context.world().clear();
    let mut random = ScriptedRandom::new([9; 5]);

    decorate_alter_ground(&mut context, &mut random).unwrap();

    assert_eq!(context.world().optional_samples.len(), 84);
    assert_eq!(context.world().offers.len(), 84);
    assert_eq!(random.bounds, [64; 5]);
    assert!(context.world().reads.is_empty());
}

#[test]
fn place_on_ground_draws_xyz_then_applies_air_solid_and_heightmap_gates() {
    let base = BlockPos::new(4, 6, 8);
    let above = BlockPos::new(4, 7, 8);
    let mut world = Fixture::default();
    world.states.insert(base, SOLID);
    world.height = above.y;
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_trunk(base, LOG);
    context.world().clear();
    let mut random = ScriptedRandom::new([0, 0, 0]);

    decorate_place_on_ground(
        &mut context,
        PlaceOnGroundConfig {
            tries: 1,
            radius: 0,
            height: 0,
        },
        &mut random,
    )
    .unwrap();

    assert_eq!(random.bounds, [1, 1, 1]);
    assert_eq!(context.world().reads, [above, base]);
    assert_eq!(context.world().height_queries, [(base.x, base.z)]);
    assert_eq!(context.world().ground_samples, [above]);
    assert_eq!(context.world().offers, [(above, LITTER, 19)]);
}

#[test]
fn lower_roots_replace_logs_in_the_shared_ground_selection() {
    let log = BlockPos::new(0, 5, 0);
    let root = BlockPos::new(0, 4, 0);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_trunk(log, LOG);
    context.offer_root(root, ROOT);

    assert_eq!(context.lowest_trunk_or_root(), [root]);
}

const AIR: BlockStateId = BlockStateId::new(0);
const LOG: BlockStateId = BlockStateId::new(1);
const ROOT: BlockStateId = BlockStateId::new(2);
const SOLID: BlockStateId = BlockStateId::new(3);
const PODZOL: BlockStateId = BlockStateId::new(4);
const LITTER: BlockStateId = BlockStateId::new(5);

#[derive(Debug, Default)]
struct Fixture {
    states: BTreeMap<BlockPos, BlockStateId>,
    optional_ground: Option<BlockStateId>,
    height: i32,
    optional_samples: Vec<BlockPos>,
    ground_samples: Vec<BlockPos>,
    reads: Vec<BlockPos>,
    height_queries: Vec<(i32, i32)>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl Fixture {
    fn clear(&mut self) {
        self.optional_samples.clear();
        self.ground_samples.clear();
        self.reads.clear();
        self.height_queries.clear();
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
        self.states.get(&position).copied().unwrap_or(AIR)
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == AIR
    }

    fn is_replaceable_by_trees(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_log(&self, state: BlockStateId) -> bool {
        state == LOG
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

impl GroundDecoratorWorld for Fixture {
    fn sample_optional_altered_ground(
        &mut self,
        position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) -> Option<BlockStateId> {
        self.optional_samples.push(position);
        self.optional_ground
    }

    fn is_solid_render(&self, state: BlockStateId) -> bool {
        state == SOLID
    }

    fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32 {
        self.height_queries.push((x, z));
        self.height
    }

    fn sample_ground_decoration(
        &mut self,
        position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) -> BlockStateId {
        self.ground_samples.push(position);
        LITTER
    }
}

#[derive(Debug, Default)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl ScriptedRandom {
    fn new(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            integers: values.into_iter().collect(),
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
