use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::geode::{
    GeodeConfig, GeodeCrackSettings, GeodeLayerSettings, GeodeMaterial, GeodeWorld, place_geode,
};
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn invalid_distribution_point_aborts_before_point_offset_and_cube_work() {
    let origin = BlockPos::new(0, 20, 0);
    let mut config = config();
    config.distribution_points = IntProvider::Constant(2);
    config.invalid_blocks_threshold = 0;
    let mut world = GeodeFixture::new();
    world.invalid_sample = true;
    let mut random = ScriptedRandom::new([], [1.0], [1.0]);

    assert!(!place_geode(&mut world, origin, &config, &mut random, |_| true).unwrap());

    assert_eq!(random.double_draws, 1);
    assert_eq!(random.float_draws, 1);
    assert_eq!(world.reads, [BlockPos::new(1, 21, 1)]);
    assert!(world.offers.is_empty());
}

#[test]
fn inner_alternate_offer_precedes_singleton_growth_and_down_is_first() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = GeodeFixture::new();
    let mut random = ScriptedRandom::new([0], [1.0], [1.0, 0.0, 0.0]);

    assert!(place_geode(&mut world, origin, &config(), &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [1]);
    assert_eq!(random.double_draws, 1);
    assert_eq!(random.float_draws, 3);
    assert_eq!(world.materials, [(GeodeMaterial::AlternateInner, origin)]);
    assert_eq!(
        world.offers,
        [
            (origin, BlockStateId::new(12), 2),
            (BlockPos::new(0, 19, 0), BlockStateId::new(110), 2),
        ]
    );
}

#[test]
fn protected_crack_still_queries_and_schedules_all_six_neighbor_fluids() {
    let origin = BlockPos::new(0, 20, 0);
    let mut config = config();
    config.crack.generate_chance = 1.0;
    let mut world = GeodeFixture::new();
    world.protected = true;
    world.nonempty_fluids = true;
    let mut random = ScriptedRandom::new([3], [0.0], [0.0]);

    assert!(place_geode(&mut world, origin, &config, &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [4]);
    assert!(world.offers.is_empty());
    assert_eq!(
        world.fluid_queries,
        [
            BlockPos::new(0, 19, 0),
            BlockPos::new(0, 21, 0),
            BlockPos::new(0, 20, -1),
            BlockPos::new(0, 20, 1),
            BlockPos::new(-1, 20, 0),
            BlockPos::new(1, 20, 0),
        ]
    );
    assert_eq!(world.fluid_ticks, world.fluid_queries);
}

fn config() -> GeodeConfig {
    GeodeConfig {
        distribution_points: IntProvider::Constant(1),
        outer_wall_distance: IntProvider::Constant(1),
        point_offset: IntProvider::Constant(0),
        minimum_generation_offset: 0,
        maximum_generation_offset: 0,
        invalid_blocks_threshold: 1,
        layers: GeodeLayerSettings {
            filling: 1.7,
            inner_layer: 2.2,
            middle_layer: 3.2,
            outer_layer: 4.2,
        },
        crack: GeodeCrackSettings {
            generate_chance: 0.0,
            base_crack_size: 2.0,
            crack_point_offset: 2,
        },
        noise_multiplier: 0.0,
        use_alternate_layer_chance: 1.0,
        use_potential_placements_chance: 1.0,
        placements_require_alternate: true,
        inner_placements: vec![BlockStateId::new(10)],
    }
}

#[derive(Debug)]
struct GeodeFixture {
    invalid_sample: bool,
    reads: Vec<BlockPos>,
    materials: Vec<(GeodeMaterial, BlockPos)>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    protected: bool,
    nonempty_fluids: bool,
    fluid_queries: Vec<BlockPos>,
    fluid_ticks: Vec<BlockPos>,
}

impl GeodeFixture {
    fn new() -> Self {
        Self {
            invalid_sample: false,
            reads: Vec::new(),
            materials: Vec::new(),
            offers: Vec::new(),
            protected: false,
            nonempty_fluids: false,
            fluid_queries: Vec::new(),
            fluid_ticks: Vec::new(),
        }
    }
}

impl GeodeWorld for GeodeFixture {
    fn world_seed(&self) -> u64 {
        42
    }

    fn initialize_geode_noise(&mut self, seed: u64) {
        assert_eq!(seed, 42);
    }

    fn geode_noise(&mut self, _position: BlockPos) -> f64 {
        0.0
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        if self.invalid_sample || position == BlockPos::new(0, 19, 0) {
            BlockStateId::new(0)
        } else {
            BlockStateId::new(9)
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_invalid_geode_block(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_protected_from_geode(&self, _state: BlockStateId) -> bool {
        self.protected
    }

    fn sample_geode_material<R: GenerationRandom>(
        &mut self,
        material: GeodeMaterial,
        position: BlockPos,
        _random: &mut R,
    ) -> BlockStateId {
        self.materials.push((material, position));
        BlockStateId::new(12)
    }

    fn canonical_air(&self) -> BlockStateId {
        BlockStateId::new(0)
    }

    fn fluid_is_nonempty_at(&mut self, position: BlockPos) -> bool {
        self.fluid_queries.push(position);
        self.nonempty_fluids
    }

    fn fluid_is_full(&mut self, _position: BlockPos, _state: BlockStateId) -> bool {
        false
    }

    fn is_water_block_identity(&self, _state: BlockStateId) -> bool {
        false
    }

    fn with_facing(&self, state: BlockStateId, direction: Direction) -> BlockStateId {
        BlockStateId::new(state.get() + direction as u32)
    }

    fn with_waterlogged_from_neighbor(
        &mut self,
        state: BlockStateId,
        _neighbor_position: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        BlockStateId::new(state.get() + 100)
    }

    fn offer_geode_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn schedule_zero_delay_fluid_tick(&mut self, position: BlockPos) {
        self.fluid_ticks.push(position);
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    doubles: VecDeque<f64>,
    floats: VecDeque<f32>,
    bounds: Vec<u32>,
    double_draws: usize,
    float_draws: usize,
}

impl ScriptedRandom {
    fn new(
        integers: impl IntoIterator<Item = u32>,
        doubles: impl IntoIterator<Item = f64>,
        floats: impl IntoIterator<Item = f32>,
    ) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            doubles: doubles.into_iter().collect(),
            floats: floats.into_iter().collect(),
            bounds: Vec::new(),
            double_draws: 0,
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
        self.double_draws += 1;
        self.doubles.pop_front().expect("scripted double")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("geode feature does not draw Gaussian values")
    }
}
