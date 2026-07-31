use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use ferrite_world::generation::feature::tree_roots::{
    MangroveRootConfig, MangroveRootWorld, place_mangrove_roots,
};
use ferrite_world::id::BlockStateId;

#[test]
fn blocked_origin_column_aborts_before_simulation_or_writes() {
    let origin = BlockPos::new(0, 4, 0);
    let trunk = BlockPos::new(0, 6, 0);
    let mut world = Fixture::default();
    world.states.insert(origin, STONE);
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::default();

    assert!(
        !place_mangrove_roots(&mut context, origin, trunk, &config(8, 15), &mut random).unwrap()
    );
    assert_eq!(context.world().reads, [origin]);
    assert!(context.world().offers.is_empty());
    assert!(random.floats.is_empty());
}

#[test]
fn admitted_candidate_reaching_length_guard_aborts_with_no_staged_writes() {
    let origin = BlockPos::new(0, 5, 0);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::with_floats([0.0]);

    assert!(
        !place_mangrove_roots(&mut context, origin, origin, &config(8, 1), &mut random).unwrap()
    );
    assert!(context.world().offers.is_empty());
}

#[test]
fn muddy_material_skips_common_recheck_while_ordinary_roots_repeat_it() {
    let trunk = BlockPos::new(0, 1, 0);
    let base = BlockPos::new(0, 0, 0);
    let mut world = Fixture::default();
    world.states.insert(base, MUD);
    for candidate in [
        BlockPos::new(0, 0, -1),
        BlockPos::new(1, 0, 0),
        BlockPos::new(0, 0, 1),
        BlockPos::new(-1, 0, 0),
    ] {
        world.states.insert(candidate, STONE);
    }
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::with_floats([0.0; 4]);

    assert!(place_mangrove_roots(&mut context, trunk, trunk, &config(8, 15), &mut random).unwrap());

    assert_eq!(
        context.world().offers,
        [
            (base, MUDDY_ROOT, 19),
            (BlockPos::new(0, 1, -1), ROOT, 19),
            (BlockPos::new(1, 1, 0), ROOT, 19),
            (BlockPos::new(0, 1, 1), ROOT, 19),
            (BlockPos::new(-1, 1, 0), ROOT, 19),
        ]
    );
    assert_eq!(
        context
            .world()
            .reads
            .iter()
            .filter(|&&position| position == base)
            .count(),
        1
    );
    for start in [
        BlockPos::new(0, 1, -1),
        BlockPos::new(1, 1, 0),
        BlockPos::new(0, 1, 1),
        BlockPos::new(-1, 1, 0),
    ] {
        assert_eq!(
            context
                .world()
                .reads
                .iter()
                .filter(|&&position| position == start)
                .count(),
            2
        );
    }
}

fn config(width: i32, length: usize) -> MangroveRootConfig {
    MangroveRootConfig {
        trunk_offset_y: IntProvider::Constant(0),
        above_root_placement_chance: None,
        max_root_width: width,
        max_root_length: length,
        random_skew_chance: 1.0,
    }
}

const AIR: BlockStateId = BlockStateId::new(0);
const STONE: BlockStateId = BlockStateId::new(1);
const MUD: BlockStateId = BlockStateId::new(2);
const ROOT: BlockStateId = BlockStateId::new(3);
const MUDDY_ROOT: BlockStateId = BlockStateId::new(4);

#[derive(Debug, Default)]
struct Fixture {
    states: BTreeMap<BlockPos, BlockStateId>,
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl TreeWorld for Fixture {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn maximum_y(&self) -> i32 {
        319
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

impl MangroveRootWorld for Fixture {
    fn can_grow_through_mangrove_roots(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_muddy_roots_material(&self, state: BlockStateId) -> bool {
        state == MUD
    }

    fn sample_root(
        &mut self,
        _position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) -> BlockStateId {
        ROOT
    }

    fn sample_muddy_root(
        &mut self,
        _position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) -> BlockStateId {
        MUDDY_ROOT
    }

    fn sample_above_root(
        &mut self,
        _position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) -> BlockStateId {
        panic!("above-root provider is absent")
    }

    fn supports_waterlogged(&self, _state: BlockStateId) -> bool {
        false
    }

    fn has_water_fluid(&mut self, _position: BlockPos) -> bool {
        false
    }

    fn with_waterlogged(&self, state: BlockStateId, _waterlogged: bool) -> BlockStateId {
        state
    }
}

#[derive(Debug, Default)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
}

impl ScriptedRandom {
    fn with_floats(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            integers: VecDeque::new(),
            floats: values.into_iter().collect(),
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
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
