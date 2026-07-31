use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::basic::{
    BASIC_FEATURE_WRITE_FLAGS, BlockBlobWorld, BlueIceWorld, EndIslandWorld, FreezeTopLayerStates,
    FreezeTopLayerWorld, GlowstoneWorld, KelpWorld, NetherVegetationWorld,
    STRUCTURAL_FEATURE_WRITE_FLAGS, SeaPickleState, SeaPickleWorld, SeagrassPart, SeagrassWorld,
    SpringConfiguration, SpringWorld, VineFace, VinesWorld, freeze_top_layer, place_block_blob,
    place_blue_ice, place_end_island, place_glowstone_blob, place_kelp,
    place_nether_forest_vegetation, place_sea_pickles, place_seagrass, place_spring, place_vines,
};
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::{BiomeId, BlockStateId};

#[test]
fn vines_uses_fixed_support_priority_and_ignores_write_failure() {
    let origin = BlockPos::new(4, 70, -2);
    let mut world = VineFixture {
        empty: true,
        accepted: [VineFace::South, VineFace::East].into_iter().collect(),
        ..VineFixture::default()
    };
    assert!(place_vines(&mut world, origin, |_| true).unwrap());
    assert_eq!(
        world.neighbor_faces,
        [VineFace::Up, VineFace::North, VineFace::South]
    );
    assert_eq!(
        world.offers,
        [(origin, VineFace::South, BASIC_FEATURE_WRITE_FLAGS)]
    );

    world.empty = false;
    world.neighbor_faces.clear();
    world.offers.clear();
    assert!(!place_vines(&mut world, origin, |_| true).unwrap());
    assert!(world.neighbor_faces.is_empty());
    assert!(world.offers.is_empty());
}

#[test]
fn sea_pickle_draws_all_attempt_fields_before_candidate_admission() {
    let origin = BlockPos::new(-8, 100, 12);
    let mut world = SeaPickleFixture::default();
    let mut random = ScriptedRandom::new([7, 0, 2, 5, 3, 7, 0, 2, 5, 0]);
    assert!(
        place_sea_pickles(
            &mut world,
            origin,
            &IntProvider::Constant(2),
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(random.bounds, [8, 8, 8, 8, 4, 8, 8, 8, 8, 4]);
    assert_eq!(
        world.trace,
        [
            "height:-1:9",
            "read:-1:63:9",
            "survive:-1:63:9:4",
            "offer:-1:63:9:4",
            "height:-1:9",
            "read:-1:63:9",
            "survive:-1:63:9:1",
            "offer:-1:63:9:1",
        ]
    );
    assert_eq!(world.offers, 2, "rejected writes still count as offers");
}

#[test]
fn blue_ice_runs_all_two_hundred_six_draw_attempts_after_admission() {
    let origin = BlockPos::new(0, 62, 0);
    let blue_ice = BlockStateId::new(4);
    let mut world = BlueIceFixture { origin, offers: 0 };
    let values = (0..200).flat_map(|_| [0, 0, 0, 0, 0, 0]);
    let mut random = ScriptedRandom::new(values);
    assert!(place_blue_ice(&mut world, origin, blue_ice, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds.len(), 1_200);
    for bounds in random.bounds.chunks_exact(6) {
        assert_eq!(bounds, [5, 6, 3, 3, 3, 3]);
    }
    assert_eq!(world.offers, 201);

    let mut no_draw = ScriptedRandom::new([]);
    assert!(
        !place_blue_ice(
            &mut world,
            BlockPos::new(0, 63, 0),
            blue_ice,
            &mut no_draw,
            |_| true,
        )
        .unwrap()
    );
    assert!(no_draw.bounds.is_empty());
}

#[test]
fn kelp_ignores_origin_y_and_places_body_before_terminal_head() {
    let origin = BlockPos::new(7, 999, -3);
    let mut world = KelpFixture::default();
    let mut random = ScriptedRandom::new([0, 3]);
    assert!(place_kelp(&mut world, origin, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [10, 4]);
    assert_eq!(
        world.offers,
        [
            ("body", BlockPos::new(7, 50, -3), 0),
            ("head", BlockPos::new(7, 51, -3), 23),
        ]
    );
    assert_eq!(
        world.reads,
        [
            BlockPos::new(7, 50, -3),
            BlockPos::new(7, 50, -3),
            BlockPos::new(7, 51, -3),
            BlockPos::new(7, 51, -3),
            BlockPos::new(7, 52, -3),
        ]
    );
}

#[test]
fn end_island_uses_strict_radius_layers_and_x_z_traversal() {
    let origin = BlockPos::new(10, 80, -10);
    let end_stone = BlockStateId::new(8);
    let mut world = EndIslandFixture::default();
    let mut random = ScriptedRandom::new([0; 8]);
    assert!(place_end_island(&mut world, origin, end_stone, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [3, 2, 2, 2, 2, 2, 2, 2]);
    assert_eq!(world.offers.len(), 283);
    assert_eq!(world.offers[0].0, BlockPos::new(6, 80, -13));
    assert!(
        world.offers.iter().all(
            |(_, state, flags)| *state == end_stone && *flags == STRUCTURAL_FEATURE_WRITE_FLAGS
        )
    );
    assert_eq!(
        world.offers.iter().map(|(position, _, _)| position.y).min(),
        Some(74)
    );
}

#[test]
fn glowstone_blob_consumes_five_draws_for_every_growth_attempt() {
    let origin = BlockPos::new(3, 40, 5);
    let glowstone = BlockStateId::new(3);
    let mut world = GlowstoneFixture { origin, offers: 0 };
    let values = (0..1_500).flat_map(|_| [0, 0, 0, 0, 0]);
    let mut random = ScriptedRandom::new(values);
    assert!(place_glowstone_blob(&mut world, origin, glowstone, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds.len(), 7_500);
    for bounds in random.bounds.chunks_exact(5) {
        assert_eq!(bounds, [8, 8, 12, 8, 8]);
    }
    assert_eq!(world.offers, 1_501);
}

#[test]
fn block_blob_uses_three_extent_and_three_shift_draws_per_pass() {
    let origin = BlockPos::new(10, 20, 30);
    let state = BlockStateId::new(9);
    let mut world = BlockBlobFixture::default();
    let mut random = ScriptedRandom::new([0; 18]);
    assert!(place_block_blob(&mut world, origin, state, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [2; 18]);
    assert_eq!(
        world.offers,
        [
            (origin, state, STRUCTURAL_FEATURE_WRITE_FLAGS),
            (
                BlockPos::new(9, 20, 29),
                state,
                STRUCTURAL_FEATURE_WRITE_FLAGS,
            ),
            (
                BlockPos::new(8, 20, 28),
                state,
                STRUCTURAL_FEATURE_WRITE_FLAGS,
            ),
        ]
    );
    assert_eq!(world.support_tests, [BlockPos::new(10, 19, 30)]);
}

#[test]
fn seagrass_uses_strict_probability_and_tall_upper_water_gate() {
    let origin = BlockPos::new(-8, 100, 12);
    let mut short_world = SeagrassFixture {
        above_water: true,
        ..SeagrassFixture::default()
    };
    let mut equality = ScriptedRandom::with_doubles([7, 0, 2, 5], [0.6]);
    assert!(place_seagrass(&mut short_world, origin, 0.6, &mut equality, |_| true).unwrap());
    assert_eq!(equality.bounds, [8, 8, 8, 8]);
    assert_eq!(
        short_world.offers,
        [(BlockPos::new(-1, 63, 9), SeagrassPart::Short)]
    );

    let mut blocked_upper = SeagrassFixture::default();
    let mut tall = ScriptedRandom::with_doubles([7, 0, 2, 5], [0.599]);
    assert!(place_seagrass(&mut blocked_upper, origin, 0.6, &mut tall, |_| true).unwrap());
    assert!(blocked_upper.offers.is_empty());
    assert_eq!(
        blocked_upper.reads,
        [BlockPos::new(-1, 63, 9), BlockPos::new(-1, 64, 9)]
    );
}

#[test]
fn nether_vegetation_calls_provider_before_candidate_gates() {
    let origin = BlockPos::new(2, 70, 4);
    let mut world = NetherVegetationFixture::default();
    let mut random = ScriptedRandom::new([0; 6]);
    assert!(
        place_nether_forest_vegetation(
            &mut world,
            origin,
            NonZeroU32::new(1).unwrap(),
            NonZeroU32::new(1).unwrap(),
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(random.bounds, [1; 6]);
    assert_eq!(
        world.trace,
        ["below", "provider", "empty", "survive", "offer"]
    );
}

#[test]
fn spring_rechecks_all_five_neighbors_and_schedules_after_failed_write() {
    let origin = BlockPos::new(0, 30, 0);
    let fluid = BlockStateId::new(7);
    let mut world = SpringFixture::default();
    assert!(
        place_spring(
            &mut world,
            origin,
            SpringConfiguration {
                fluid_legacy_block: fluid,
                requires_block_below: true,
                rock_count: 5,
                hole_count: 5,
            },
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(world.reads.len(), 8);
    assert_eq!(
        world
            .reads
            .iter()
            .filter(|position| position.y == 29)
            .count(),
        2,
        "below is read by admission and the complete rock scan"
    );
    assert_eq!(world.empty_checks.len(), 5);
    assert_eq!(world.offers, [(origin, fluid, BASIC_FEATURE_WRITE_FLAGS)]);
    assert_eq!(world.scheduled, [(origin, 0)]);
}

#[test]
fn freeze_top_layer_uses_x_z_order_and_ice_snow_snowy_offers() {
    let origin = BlockPos::new(-2, 999, 4);
    let states = FreezeTopLayerStates {
        ice: BlockStateId::new(10),
        snow: BlockStateId::new(11),
    };
    let mut world = FreezeFixture::default();
    assert!(freeze_top_layer(&mut world, origin, states, |_| true).unwrap());
    assert_eq!(world.heights.len(), 256);
    assert_eq!(world.biomes.len(), 256);
    assert_eq!(world.below_reads.len(), 256);
    assert_eq!(world.offers.len(), 768);
    assert_eq!(
        &world.offers[..3],
        [
            (
                BlockPos::new(-2, 69, 4),
                states.ice,
                BASIC_FEATURE_WRITE_FLAGS,
            ),
            (
                BlockPos::new(-2, 70, 4),
                states.snow,
                BASIC_FEATURE_WRITE_FLAGS,
            ),
            (
                BlockPos::new(-2, 69, 4),
                BlockStateId::new(6),
                BASIC_FEATURE_WRITE_FLAGS,
            ),
        ]
    );
    assert!(world.freeze_edge_flags.iter().all(|required| !required));
    assert_eq!(world.heights[1], (-2, 5));
    assert_eq!(world.heights[16], (-1, 4));
}

#[derive(Debug, Default)]
struct VineFixture {
    empty: bool,
    accepted: Vec<VineFace>,
    neighbor_faces: Vec<VineFace>,
    offers: Vec<(BlockPos, VineFace, u32)>,
}

impl VinesWorld for VineFixture {
    fn is_empty_block(&mut self, _position: BlockPos) -> bool {
        self.empty
    }

    fn can_attach_vine_to(&mut self, _neighbor: BlockPos, face: VineFace) -> bool {
        self.neighbor_faces.push(face);
        self.accepted.contains(&face)
    }

    fn offer_vine(&mut self, position: BlockPos, attached: VineFace, flags: u32) -> bool {
        self.offers.push((position, attached, flags));
        false
    }
}

#[derive(Debug, Default)]
struct SeaPickleFixture {
    trace: Vec<String>,
    offers: usize,
}

impl SeaPickleWorld for SeaPickleFixture {
    fn ocean_floor_height(&mut self, x: i32, z: i32) -> i32 {
        self.trace.push(format!("height:{x}:{z}"));
        63
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.trace
            .push(format!("read:{}:{}:{}", position.x, position.y, position.z));
        BlockStateId::new(1)
    }

    fn is_exact_water(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn sea_pickle_survives(&mut self, position: BlockPos, state: SeaPickleState) -> bool {
        self.trace.push(format!(
            "survive:{}:{}:{}:{}",
            position.x, position.y, position.z, state.count
        ));
        true
    }

    fn offer_sea_pickle(&mut self, position: BlockPos, state: SeaPickleState, flags: u32) -> bool {
        assert_eq!(flags, BASIC_FEATURE_WRITE_FLAGS);
        self.trace.push(format!(
            "offer:{}:{}:{}:{}",
            position.x, position.y, position.z, state.count
        ));
        self.offers += 1;
        false
    }
}

#[derive(Debug)]
struct BlueIceFixture {
    origin: BlockPos,
    offers: usize,
}

impl BlueIceWorld for BlueIceFixture {
    fn sea_level(&self) -> i32 {
        63
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position == self.origin {
            BlockStateId::new(1)
        } else if position == BlockPos::new(self.origin.x, self.origin.y + 1, self.origin.z) {
            BlockStateId::new(2)
        } else if position == BlockPos::new(self.origin.x, self.origin.y - 1, self.origin.z) {
            BlockStateId::new(4)
        } else {
            BlockStateId::new(0)
        }
    }

    fn is_exact_water(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn is_packed_ice(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(2)
    }

    fn is_blue_ice(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(4)
    }

    fn is_blue_ice_candidate(&self, state: BlockStateId) -> bool {
        matches!(state.get(), 0..=3)
    }

    fn offer_blue_ice(&mut self, _position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        assert_eq!(state, BlockStateId::new(4));
        assert_eq!(flags, BASIC_FEATURE_WRITE_FLAGS);
        self.offers += 1;
        false
    }
}

#[derive(Debug, Default)]
struct KelpFixture {
    reads: Vec<BlockPos>,
    offers: Vec<(&'static str, BlockPos, u8)>,
}

impl KelpWorld for KelpFixture {
    fn ocean_floor_height(&mut self, _x: i32, _z: i32) -> i32 {
        50
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        BlockStateId::new(1)
    }

    fn is_exact_water(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn kelp_body_survives(&mut self, _position: BlockPos) -> bool {
        true
    }

    fn kelp_head_survives(&mut self, _position: BlockPos) -> bool {
        true
    }

    fn is_kelp_head(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(2)
    }

    fn offer_kelp_body(&mut self, position: BlockPos, flags: u32) -> bool {
        assert_eq!(flags, BASIC_FEATURE_WRITE_FLAGS);
        self.offers.push(("body", position, 0));
        false
    }

    fn offer_kelp_head(&mut self, position: BlockPos, age: u8, flags: u32) -> bool {
        assert_eq!(flags, BASIC_FEATURE_WRITE_FLAGS);
        self.offers.push(("head", position, age));
        false
    }
}

#[derive(Debug, Default)]
struct EndIslandFixture {
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl EndIslandWorld for EndIslandFixture {
    fn offer_end_stone(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct GlowstoneFixture {
    origin: BlockPos,
    offers: usize,
}

impl GlowstoneWorld for GlowstoneFixture {
    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        position == self.origin
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position == BlockPos::new(self.origin.x, self.origin.y + 1, self.origin.z) {
            BlockStateId::new(2)
        } else if position == BlockPos::new(self.origin.x, self.origin.y - 1, self.origin.z) {
            BlockStateId::new(3)
        } else {
            BlockStateId::new(0)
        }
    }

    fn is_glowstone_support(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(2)
    }

    fn is_glowstone(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(3)
    }

    fn offer_glowstone(&mut self, _position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        assert_eq!(state, BlockStateId::new(3));
        assert_eq!(flags, BASIC_FEATURE_WRITE_FLAGS);
        self.offers += 1;
        false
    }
}

#[derive(Debug, Default)]
struct BlockBlobFixture {
    support_tests: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl BlockBlobWorld for BlockBlobFixture {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn can_place_blob_on(&mut self, position: BlockPos) -> bool {
        self.support_tests.push(position);
        true
    }

    fn offer_blob_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug, Default)]
struct SeagrassFixture {
    above_water: bool,
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, SeagrassPart)>,
}

impl SeagrassWorld for SeagrassFixture {
    fn ocean_floor_height(&mut self, _x: i32, _z: i32) -> i32 {
        63
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        if position.y == 63 || self.above_water {
            BlockStateId::new(1)
        } else {
            BlockStateId::new(0)
        }
    }

    fn is_exact_water(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn seagrass_survives(&mut self, _position: BlockPos, _tall: bool) -> bool {
        true
    }

    fn offer_seagrass(&mut self, position: BlockPos, part: SeagrassPart, flags: u32) -> bool {
        assert_eq!(flags, BASIC_FEATURE_WRITE_FLAGS);
        self.offers.push((position, part));
        false
    }
}

#[derive(Debug, Default)]
struct NetherVegetationFixture {
    trace: Vec<&'static str>,
}

impl NetherVegetationWorld<ScriptedRandom> for NetherVegetationFixture {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn maximum_y(&self) -> i32 {
        319
    }

    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        self.trace.push("below");
        BlockStateId::new(1)
    }

    fn is_nylium(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn provide_vegetation_state(
        &mut self,
        _position: BlockPos,
        _random: &mut ScriptedRandom,
    ) -> BlockStateId {
        self.trace.push("provider");
        BlockStateId::new(9)
    }

    fn is_empty_block(&mut self, _position: BlockPos) -> bool {
        self.trace.push("empty");
        true
    }

    fn vegetation_survives(&mut self, _state: BlockStateId, _position: BlockPos) -> bool {
        self.trace.push("survive");
        true
    }

    fn offer_vegetation(&mut self, _position: BlockPos, _state: BlockStateId, flags: u32) -> bool {
        assert_eq!(flags, BASIC_FEATURE_WRITE_FLAGS);
        self.trace.push("offer");
        false
    }
}

#[derive(Debug, Default)]
struct SpringFixture {
    reads: Vec<BlockPos>,
    empty_checks: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    scheduled: Vec<(BlockPos, u32)>,
}

impl SpringWorld for SpringFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        if position == BlockPos::new(0, 30, 0) {
            BlockStateId::new(0)
        } else {
            BlockStateId::new(1)
        }
    }

    fn is_valid_spring_block(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        self.empty_checks.push(position);
        true
    }

    fn offer_spring_fluid(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn schedule_spring_fluid(&mut self, position: BlockPos, delay: u32) {
        self.scheduled.push((position, delay));
    }
}

#[derive(Debug, Default)]
struct FreezeFixture {
    heights: Vec<(i32, i32)>,
    biomes: Vec<BlockPos>,
    freeze_edge_flags: Vec<bool>,
    below_reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl FreezeTopLayerWorld for FreezeFixture {
    fn motion_blocking_height(&mut self, x: i32, z: i32) -> i32 {
        self.heights.push((x, z));
        70
    }

    fn biome(&mut self, position: BlockPos) -> BiomeId {
        self.biomes.push(position);
        BiomeId::new(1)
    }

    fn should_freeze(
        &mut self,
        _biome: BiomeId,
        _position: BlockPos,
        require_horizontal_edge: bool,
    ) -> bool {
        self.freeze_edge_flags.push(require_horizontal_edge);
        true
    }

    fn should_snow(&mut self, _biome: BiomeId, _position: BlockPos) -> bool {
        true
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.below_reads.push(position);
        BlockStateId::new(5)
    }

    fn with_snowy_true(&self, state: BlockStateId) -> Option<BlockStateId> {
        (state == BlockStateId::new(5)).then(|| BlockStateId::new(6))
    }

    fn offer_frozen_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    values: VecDeque<u32>,
    bounds: Vec<u32>,
    doubles: VecDeque<f64>,
}

impl ScriptedRandom {
    fn new(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            values: values.into_iter().collect(),
            bounds: Vec::new(),
            doubles: VecDeque::new(),
        }
    }

    fn with_doubles(
        values: impl IntoIterator<Item = u32>,
        doubles: impl IntoIterator<Item = f64>,
    ) -> Self {
        Self {
            values: values.into_iter().collect(),
            bounds: Vec::new(),
            doubles: doubles.into_iter().collect(),
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.values.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        panic!("sea-pickle and vines do not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        self.doubles.pop_front().expect("scripted double")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("sea-pickle and vines do not draw Gaussian values")
    }
}
