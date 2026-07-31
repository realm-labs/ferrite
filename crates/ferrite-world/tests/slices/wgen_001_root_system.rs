use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::root_system::{
    RootSystemConfig, RootSystemWorld, place_root_system,
};
use ferrite_world::id::BlockStateId;

#[test]
fn nonair_origin_rejects_before_height_or_random_work() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = RootFixture::new(origin);
    world.origin_air = false;
    let mut random = ScriptedRandom::new([]);

    assert!(!place_root_system(&mut world, origin, config(), &mut random, |_| true).unwrap());
    assert!(world.height_queries.is_empty());
    assert!(random.bounds.is_empty());
}

#[test]
fn successful_second_candidate_roots_one_layer_then_uses_origin_centered_hanging_offsets() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = RootFixture::new(origin);
    let mut random = ScriptedRandom::new([0; 10]);

    assert!(place_root_system(&mut world, origin, config(), &mut random, |_| true).unwrap());

    assert_eq!(
        world.allowed_candidates,
        [BlockPos::new(0, 11, 0), BlockPos::new(0, 12, 0)]
    );
    assert_eq!(world.children, [BlockPos::new(0, 12, 0)]);
    assert_eq!(random.bounds, [3, 3, 3, 3, 2, 2, 2, 2, 2, 2]);
    assert_eq!(
        world.offers,
        [
            (origin, BlockStateId::new(2), 2),
            (origin, BlockStateId::new(3), 2),
        ]
    );
    assert_eq!(world.sturdy_checks, [(BlockPos::new(0, 11, 0), origin)]);
}

fn config() -> RootSystemConfig {
    RootSystemConfig {
        required_vertical_space_for_tree: 1,
        root_radius: 3,
        root_placement_attempts: 1,
        root_column_max_height: 2,
        hanging_root_radius: 2,
        hanging_roots_vertical_span: 2,
        hanging_root_placement_attempts: 1,
        allowed_vertical_water_for_tree: 1,
        level_test_distance: 0,
        maximum_level_deviation: 0,
    }
}

#[derive(Debug)]
struct RootFixture {
    origin: BlockPos,
    origin_air: bool,
    height_queries: Vec<(i32, i32)>,
    allowed_candidates: Vec<BlockPos>,
    children: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    sturdy_checks: Vec<(BlockPos, BlockPos)>,
}

impl RootFixture {
    fn new(origin: BlockPos) -> Self {
        Self {
            origin,
            origin_air: true,
            height_queries: Vec::new(),
            allowed_candidates: Vec::new(),
            children: Vec::new(),
            offers: Vec::new(),
            sturdy_checks: Vec::new(),
        }
    }
}

impl RootSystemWorld for RootFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position == self.origin {
            if self.origin_air {
                BlockStateId::new(0)
            } else {
                BlockStateId::new(9)
            }
        } else if position.y == self.origin.y + 1 {
            BlockStateId::new(9)
        } else {
            BlockStateId::new(0)
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn has_water_tagged_fluid(&self, _state: BlockStateId) -> bool {
        false
    }

    fn has_lava_fluid(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_solid(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(9)
    }

    fn world_surface(&mut self, x: i32, z: i32) -> i32 {
        self.height_queries.push((x, z));
        12
    }

    fn allowed_tree_position(&mut self, position: BlockPos) -> bool {
        self.allowed_candidates.push(position);
        position.y == self.origin.y + 2
    }

    fn place_root_child<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        _random: &mut R,
    ) -> bool {
        self.children.push(position);
        true
    }

    fn is_root_replaceable(&self, _state: BlockStateId) -> bool {
        true
    }

    fn sample_root_state<R: GenerationRandom>(
        &mut self,
        _position: BlockPos,
        _random: &mut R,
    ) -> BlockStateId {
        BlockStateId::new(2)
    }

    fn sample_hanging_root_state<R: GenerationRandom>(
        &mut self,
        _position: BlockPos,
        _random: &mut R,
    ) -> BlockStateId {
        BlockStateId::new(3)
    }

    fn can_hanging_root_survive(&mut self, _state: BlockStateId, _position: BlockPos) -> bool {
        true
    }

    fn has_sturdy_downward_face_at(
        &mut self,
        above_position: BlockPos,
        _above_state: BlockStateId,
        queried_position: BlockPos,
    ) -> bool {
        self.sturdy_checks.push((above_position, queried_position));
        true
    }

    fn offer_root_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
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
        panic!("root system does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("root system does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("root system does not draw Gaussian values")
    }
}
