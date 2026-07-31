use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use ferrite_world::generation::feature::tree_decorator_logs::{
    LogDecoratorWorld, decorate_beehive, decorate_cocoa, decorate_creaking_heart,
};
use ferrite_world::id::BlockStateId;

#[test]
fn cocoa_uses_inclusive_face_chance_and_draws_age_only_after_air() {
    let log = BlockPos::new(0, 5, 0);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_trunk(log, LOG);
    context.world().clear();
    let mut random = ScriptedRandom::new([2, 1], [0.0, 0.25, 0.26, 0.25, 0.26]);

    decorate_cocoa(&mut context, 1.0, &mut random).unwrap();

    assert_eq!(random.bounds, [3, 3]);
    assert_eq!(
        context.world().offers,
        [
            (BlockPos::new(0, 5, 1), COCOA_NORTH_AGE_2, 19),
            (BlockPos::new(0, 5, -1), COCOA_SOUTH_AGE_1, 19),
        ]
    );
}

#[test]
fn creaking_heart_does_not_recheck_the_candidate_state() {
    let candidate = BlockPos::new(2, 8, 4);
    let mut world = Fixture {
        default_state: LOG,
        ..Fixture::default()
    };
    world.states.insert(candidate, AIR);
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_trunk(candidate, LOG);
    context.world().clear();
    let mut random = ScriptedRandom::new([], [0.0]);

    decorate_creaking_heart(&mut context, 1.0, &mut random).unwrap();

    assert!(!context.world().reads.contains(&candidate));
    assert_eq!(context.world().reads.len(), 6);
    assert_eq!(context.world().offers, [(candidate, HEART, 19)]);
}

#[test]
fn missing_beehive_entity_stops_before_occupant_draws() {
    let log = BlockPos::new(0, 5, 0);
    let leaf = BlockPos::new(0, 7, 0);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    context.offer_trunk(log, LOG);
    context.offer_trunk(BlockPos::new(0, 6, 0), LOG);
    context.offer_foliage(leaf, LEAF);
    context.world().clear();
    let mut random = ScriptedRandom::new([2, 1], [0.0]);

    decorate_beehive(&mut context, 1.0, &mut random).unwrap();

    assert_eq!(random.bounds, [3, 2]);
    assert_eq!(context.world().offers.len(), 1);
    assert_eq!(context.world().entity_queries.len(), 1);
    assert!(context.world().occupants.is_empty());
}

const AIR: BlockStateId = BlockStateId::new(0);
const LOG: BlockStateId = BlockStateId::new(1);
const LEAF: BlockStateId = BlockStateId::new(2);
const HEART: BlockStateId = BlockStateId::new(3);
const NEST: BlockStateId = BlockStateId::new(4);
const COCOA_NORTH_AGE_2: BlockStateId = BlockStateId::new(12);
const COCOA_SOUTH_AGE_1: BlockStateId = BlockStateId::new(31);

#[derive(Debug)]
struct Fixture {
    states: BTreeMap<BlockPos, BlockStateId>,
    default_state: BlockStateId,
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    entity_queries: Vec<BlockPos>,
    occupants: Vec<(BlockPos, u32, u32)>,
}

impl Default for Fixture {
    fn default() -> Self {
        Self {
            states: BTreeMap::new(),
            default_state: AIR,
            reads: Vec::new(),
            offers: Vec::new(),
            entity_queries: Vec::new(),
            occupants: Vec::new(),
        }
    }
}

impl Fixture {
    fn clear(&mut self) {
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
        self.states
            .get(&position)
            .copied()
            .unwrap_or(self.default_state)
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

impl LogDecoratorWorld for Fixture {
    fn cocoa_state(&self, facing: Direction, age: u8) -> BlockStateId {
        let facing = match facing {
            Direction::North => 1,
            Direction::East => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::Down | Direction::Up => panic!("cocoa facing must be horizontal"),
        };
        BlockStateId::new(facing * 10 + u32::from(age))
    }

    fn creaking_heart_state(&self) -> BlockStateId {
        HEART
    }

    fn belongs_to_logs_tag(&self, state: BlockStateId) -> bool {
        state == LOG
    }

    fn bee_nest_facing_south(&self) -> BlockStateId {
        NEST
    }

    fn has_beehive_block_entity(&mut self, position: BlockPos) -> bool {
        self.entity_queries.push(position);
        false
    }

    fn store_bee_occupant(&mut self, position: BlockPos, ticks_in_hive: u32, minimum_ticks: u32) {
        self.occupants
            .push((position, ticks_in_hive, minimum_ticks));
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
