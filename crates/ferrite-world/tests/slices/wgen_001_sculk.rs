use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::sculk::{
    SculkBehavior, SculkPatchConfig, SculkWorld, place_sculk_patch,
};
use ferrite_world::id::BlockStateId;

#[test]
fn rejected_origin_stops_before_configuration_and_random_access() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = Fixture::default();
    world.states.insert(origin, STONE);
    let mut random = ScriptedRandom::default();
    let mut invalid = config();
    invalid.charge_count = 0;

    assert!(!place_sculk_patch(&mut world, origin, &invalid, &mut random, |_| true).unwrap());

    assert_eq!(world.reads, [origin]);
    assert!(random.bounds.is_empty());
    assert_eq!(random.float_draws, 0);
}

#[test]
fn water_source_admission_checks_neighbors_in_direction_order() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = Fixture::default();
    world.states.insert(origin, WATER);
    world.source_fluids.insert(origin);
    world.states.insert(BlockPos::new(0, 20, -1), STONE);
    let mut config = config();
    config.catalyst_chance = 0.0;
    let mut random = ScriptedRandom::with_floats([1.0]);

    assert!(place_sculk_patch(&mut world, origin, &config, &mut random, |_| true).unwrap());

    assert_eq!(
        world.reads,
        [
            origin,
            BlockPos::new(0, 19, 0),
            BlockPos::new(0, 21, 0),
            BlockPos::new(0, 20, -1),
        ]
    );
    assert_eq!(random.float_draws, 1);
}

#[test]
fn catalyst_equality_and_rare_shrieker_use_flags_three() {
    let origin = BlockPos::new(0, 20, 0);
    let rare = BlockPos::new(0, 20, 1);
    let mut world = Fixture::default();
    world.states.insert(origin, SCULK);
    world.states.insert(BlockPos::new(0, 19, 0), STONE);
    world.states.insert(BlockPos::new(0, 19, 1), STONE);
    let mut config = config();
    config.catalyst_chance = 0.5;
    config.extra_rare_growths = IntProvider::Constant(1);
    let mut random = ScriptedRandom::new([2, 3], [0.5]);

    assert!(place_sculk_patch(&mut world, origin, &config, &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [5, 5]);
    assert_eq!(
        world.offers,
        [(origin, CATALYST, 3), (rare, SHRIEKER_DRY, 3)]
    );
}

#[test]
fn close_sculk_charge_uses_both_decay_draws_then_emits_zero_event() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = Fixture::default();
    world.states.insert(origin, SCULK);
    let mut config = config();
    config.amount_per_charge = 1;
    config.growth_rounds = 1;
    config.spread_attempts = 1;
    config.catalyst_chance = 0.0;
    let mut random = ScriptedRandom::new([0, 0], [1.0]);

    assert!(place_sculk_patch(&mut world, origin, &config, &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [5, 10]);
    assert_eq!(world.events, [(origin, 0)]);
    assert!(world.spread_all.is_empty());
}

fn config() -> SculkPatchConfig {
    SculkPatchConfig {
        charge_count: 1,
        amount_per_charge: 1,
        spread_attempts: 1,
        growth_rounds: 0,
        spread_rounds: 0,
        extra_rare_growths: IntProvider::Constant(0),
        catalyst_chance: 0.0,
    }
}

const AIR: BlockStateId = BlockStateId::new(0);
const WATER: BlockStateId = BlockStateId::new(1);
const STONE: BlockStateId = BlockStateId::new(2);
const SCULK: BlockStateId = BlockStateId::new(3);
const VEIN: BlockStateId = BlockStateId::new(4);
const SENSOR_DRY: BlockStateId = BlockStateId::new(5);
const SENSOR_WET: BlockStateId = BlockStateId::new(6);
const SHRIEKER_DRY: BlockStateId = BlockStateId::new(7);
const SHRIEKER_WET: BlockStateId = BlockStateId::new(8);
const CATALYST: BlockStateId = BlockStateId::new(9);

#[derive(Debug, Default)]
struct Fixture {
    states: BTreeMap<BlockPos, BlockStateId>,
    source_fluids: BTreeSet<BlockPos>,
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    events: Vec<(BlockPos, i32)>,
    spread_all: Vec<BlockPos>,
}

impl SculkWorld for Fixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        self.states.get(&position).copied().unwrap_or(AIR)
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == AIR
    }

    fn is_water_block(&self, state: BlockStateId) -> bool {
        state == WATER
    }

    fn fluid_is_source(&mut self, position: BlockPos) -> bool {
        self.source_fluids.contains(&position)
    }

    fn has_nonempty_fluid(&mut self, position: BlockPos) -> bool {
        self.states.get(&position) == Some(&WATER)
    }

    fn has_full_collision(&self, state: BlockStateId, _position: BlockPos) -> bool {
        state == STONE
    }

    fn is_face_sturdy(&self, state: BlockStateId, _position: BlockPos, _face: Direction) -> bool {
        state == STONE
    }

    fn behavior(&self, state: BlockStateId) -> SculkBehavior {
        match state {
            SCULK => SculkBehavior::Sculk,
            VEIN => SculkBehavior::Vein,
            _ => SculkBehavior::Default,
        }
    }

    fn available_vein_faces(&self, state: BlockStateId) -> u8 {
        u8::from(state == VEIN)
    }

    fn is_worldgen_replaceable(&self, state: BlockStateId) -> bool {
        state == STONE
    }

    fn is_ordinary_sculk_replaceable(&self, state: BlockStateId) -> bool {
        state == STONE
    }

    fn is_sensor_or_shrieker(&self, state: BlockStateId) -> bool {
        matches!(state, SENSOR_DRY | SENSOR_WET | SHRIEKER_DRY | SHRIEKER_WET)
    }

    fn spread_vein_same_space(
        &mut self,
        _position: BlockPos,
        _state: BlockStateId,
        _postprocess: bool,
    ) -> u64 {
        0
    }

    fn spread_vein_all(
        &mut self,
        position: BlockPos,
        _state: BlockStateId,
        _postprocess: bool,
    ) -> u64 {
        self.spread_all.push(position);
        0
    }

    fn regrow_vein(&mut self, _position: BlockPos, _replaced: BlockStateId, _faces: u8) -> bool {
        false
    }

    fn discharge_vein(
        &mut self,
        _position: BlockPos,
        _state: BlockStateId,
        _random: &mut impl GenerationRandom,
    ) {
    }

    fn sculk_state(&self) -> BlockStateId {
        SCULK
    }

    fn sensor_state(&self, waterlogged: bool) -> BlockStateId {
        if waterlogged { SENSOR_WET } else { SENSOR_DRY }
    }

    fn shrieker_state(&self, _can_summon: bool, waterlogged: bool) -> BlockStateId {
        if waterlogged {
            SHRIEKER_WET
        } else {
            SHRIEKER_DRY
        }
    }

    fn catalyst_state(&self) -> BlockStateId {
        CATALYST
    }

    fn offer_sculk(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        true
    }

    fn push_entities_up(
        &mut self,
        _position: BlockPos,
        _old_state: BlockStateId,
        _new_state: BlockStateId,
    ) {
    }

    fn play_spread_sound(&mut self, _position: BlockPos) {}

    fn play_placement_sound(&mut self, _position: BlockPos, _state: BlockStateId) {}

    fn level_event_3006(&mut self, position: BlockPos, data: i32) {
        self.events.push((position, data));
    }
}

#[derive(Debug, Default)]
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
            ..Self::default()
        }
    }

    fn with_floats(floats: impl IntoIterator<Item = f32>) -> Self {
        Self::new([], floats)
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("unexpected integer draw");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        self.floats.pop_front().expect("unexpected float draw")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("unexpected double draw")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("unexpected Gaussian draw")
    }
}
