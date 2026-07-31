use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::vegetation_patch::{
    BonemealablePatchType, BonemealablePatchWorld, CaveSurface, VegetationPatchConfig,
    VegetationPatchWorld, is_patch_bonemeal_success, is_valid_patch_bonemeal_target,
    patch_bonemeal_type, perform_patch_bonemeal, place_vegetation_patch,
    place_waterlogged_vegetation_patch,
};
use ferrite_world::id::BlockStateId;

#[test]
fn ordinary_patch_keeps_a_failed_ground_offer_and_ignores_child_failure() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = PatchFixture::new(origin);
    let mut random = ScriptedRandom::new([0.5]);

    assert!(place_vegetation_patch(&mut world, origin, &config(), &mut random, |_| true).unwrap());

    let ground = BlockPos::new(0, 9, 0);
    assert_eq!(random.float_draws, 1);
    assert_eq!(world.offers, [(ground, BlockStateId::new(2), 2)]);
    assert_eq!(world.children, [origin]);
}

#[test]
fn waterlogged_patch_checks_five_faces_then_rewaterlogs_after_true_child() {
    let origin = BlockPos::new(0, 10, 0);
    let ground = BlockPos::new(0, 9, 0);
    let mut world = PatchFixture::new(origin);
    world.child_result = true;
    let mut random = ScriptedRandom::new([0.5]);

    assert!(
        place_waterlogged_vegetation_patch(&mut world, origin, &config(), &mut random, |_| true)
            .unwrap()
    );

    assert_eq!(
        world.sturdy_checks,
        [
            (BlockPos::new(0, 9, -1), Direction::South),
            (BlockPos::new(1, 9, 0), Direction::West),
            (BlockPos::new(0, 9, 1), Direction::North),
            (BlockPos::new(-1, 9, 0), Direction::East),
            (BlockPos::new(0, 8, 0), Direction::Up),
        ]
    );
    assert_eq!(world.children, [ground]);
    assert_eq!(
        world.offers,
        [
            (ground, BlockStateId::new(2), 2),
            (ground, BlockStateId::new(5), 2),
            (ground, BlockStateId::new(7), 2),
        ]
    );
}

#[test]
fn patch_bonemeal_targets_air_above_and_ignores_the_feature_result() {
    let position = BlockPos::new(4, 30, 5);
    let mut world = BonemealFixture::new();
    let mut random = ScriptedRandom::new([]);

    assert!(is_valid_patch_bonemeal_target(&mut world, position).unwrap());
    assert!(is_patch_bonemeal_success());
    assert_eq!(
        patch_bonemeal_type(),
        BonemealablePatchType::NeighborSpreader
    );
    perform_patch_bonemeal(&mut world, position, &mut random).unwrap();
    assert_eq!(world.placements, [BlockPos::new(4, 31, 5)]);
}

fn config() -> VegetationPatchConfig {
    VegetationPatchConfig {
        surface: CaveSurface::Floor,
        depth: IntProvider::Constant(1),
        extra_bottom_block_chance: 0.0,
        vertical_range: 5,
        vegetation_chance: 1.0,
        xz_radius: IntProvider::Constant(0),
        extra_edge_column_chance: 0.0,
    }
}

#[derive(Debug)]
struct PatchFixture {
    origin: BlockPos,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    children: Vec<BlockPos>,
    sturdy_checks: Vec<(BlockPos, Direction)>,
    child_result: bool,
}

impl PatchFixture {
    fn new(origin: BlockPos) -> Self {
        Self {
            origin,
            offers: Vec::new(),
            children: Vec::new(),
            sturdy_checks: Vec::new(),
            child_result: false,
        }
    }
}

impl VegetationPatchWorld for PatchFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position.y >= self.origin.y {
            BlockStateId::new(0)
        } else {
            BlockStateId::new(9)
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_empty(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_face_sturdy(
        &mut self,
        position: BlockPos,
        _state: BlockStateId,
        face: Direction,
    ) -> bool {
        if position != BlockPos::new(0, 9, 0) {
            self.sturdy_checks.push((position, face));
        }
        true
    }

    fn sample_ground<R: GenerationRandom>(
        &mut self,
        _position: BlockPos,
        _random: &mut R,
    ) -> BlockStateId {
        BlockStateId::new(2)
    }

    fn same_block_type(&self, left: BlockStateId, right: BlockStateId) -> bool {
        left == right
    }

    fn is_ground_replaceable(&self, _state: BlockStateId) -> bool {
        true
    }

    fn source_water(&self) -> BlockStateId {
        BlockStateId::new(5)
    }

    fn with_waterlogged_true(&self, _state: BlockStateId) -> Option<BlockStateId> {
        Some(BlockStateId::new(7))
    }

    fn offer_patch_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn place_nested_vegetation<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        _random: &mut R,
    ) -> bool {
        self.children.push(position);
        self.child_result
    }
}

#[derive(Debug)]
struct BonemealFixture {
    placements: Vec<BlockPos>,
}

impl BonemealFixture {
    fn new() -> Self {
        Self {
            placements: Vec::new(),
        }
    }
}

impl BonemealablePatchWorld for BonemealFixture {
    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        BlockStateId::new(0)
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn resolve_bonemeal_patch(&mut self) -> bool {
        true
    }

    fn place_bonemeal_patch<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        _random: &mut R,
    ) -> bool {
        self.placements.push(position);
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    floats: VecDeque<f32>,
    float_draws: usize,
}

impl ScriptedRandom {
    fn new(floats: impl IntoIterator<Item = f32>) -> Self {
        Self {
            floats: floats.into_iter().collect(),
            float_draws: 0,
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        panic!("constant providers and center-only patch do not draw integers")
    }

    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        self.floats.pop_front().expect("scripted float")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("vegetation patch does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("vegetation patch does not draw Gaussian values")
    }
}
