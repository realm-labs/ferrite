use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::huge_fungus::{
    FungusBonemealWorld, FungusVinePlacement, HugeFungusConfig, HugeFungusWorld,
    is_fungus_bonemeal_success, is_valid_fungus_bonemeal_target, perform_fungus_bonemeal,
    place_huge_fungus,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn invalid_base_rejects_before_height_draws() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = FungusFixture::new(origin);
    world.valid_base = false;
    let mut random = ScriptedRandom::new([], []);

    assert!(!place_huge_fungus(&mut world, origin, config(false), &mut random, |_| true).unwrap());
    assert!(random.bounds.is_empty());
    assert!(world.offers.is_empty());
}

#[test]
fn planted_stem_destroys_each_supported_target_and_still_draws_every_hat_radius() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = FungusFixture::new(origin);
    let mut random = ScriptedRandom::new([0, 1, 0, 0, 0, 0, 0, 0], []);

    assert!(place_huge_fungus(&mut world, origin, config(true), &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [10, 12, 2, 3, 3, 3, 3, 3]);
    assert_eq!(
        world.destroyed,
        [
            BlockPos::new(0, 20, 0),
            BlockPos::new(0, 21, 0),
            BlockPos::new(0, 22, 0),
            BlockPos::new(0, 23, 0),
        ]
    );
    assert_eq!(world.offers[0], (origin, BlockStateId::new(0), 260));
    assert_eq!(
        world
            .offers
            .iter()
            .filter(|(_, state, flags)| *state == BlockStateId::new(2) && *flags == 3)
            .count(),
        4
    );
}

#[test]
fn ordinary_wide_stem_draws_every_corner_but_writes_five_columns() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = FungusFixture::new(origin);
    let integers = [0, 1, 0, 0, 0, 0, 0, 0];
    let mut floats = vec![0.05];
    floats.extend([0.2; 16]);
    let mut random = ScriptedRandom::new(integers, floats);

    assert!(place_huge_fungus(&mut world, origin, config(false), &mut random, |_| true).unwrap());

    assert_eq!(random.float_draws, 17);
    assert_eq!(
        world
            .offers
            .iter()
            .filter(|(_, state, flags)| *state == BlockStateId::new(2) && *flags == 3)
            .count(),
        20
    );
}

#[test]
fn fungus_bonemeal_uses_exact_base_height_gate_and_strict_success_float() {
    let position = BlockPos::new(3, 40, 5);
    let mut world = BonemealFixture::new();
    let mut random = ScriptedRandom::new([], [0.4]);

    assert!(is_valid_fungus_bonemeal_target(&mut world, position).unwrap());
    assert!(!is_fungus_bonemeal_success(&mut random));
    perform_fungus_bonemeal(&mut world, position, &mut random);
    assert_eq!(world.placements, [position]);
}

fn config(planted: bool) -> HugeFungusConfig {
    HugeFungusConfig {
        valid_base: BlockStateId::new(1),
        stem: BlockStateId::new(2),
        hat: BlockStateId::new(3),
        decor: BlockStateId::new(4),
        planted,
        crimson_vines: true,
    }
}

#[derive(Debug)]
struct FungusFixture {
    origin: BlockPos,
    valid_base: bool,
    states: BTreeMap<BlockPos, BlockStateId>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    destroyed: Vec<BlockPos>,
}

impl FungusFixture {
    fn new(origin: BlockPos) -> Self {
        Self {
            origin,
            valid_base: true,
            states: BTreeMap::new(),
            offers: Vec::new(),
            destroyed: Vec::new(),
        }
    }
}

impl HugeFungusWorld for FungusFixture {
    fn generation_depth(&self) -> i32 {
        100
    }

    fn canonical_air(&self) -> BlockStateId {
        BlockStateId::new(0)
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if let Some(state) = self.states.get(&position) {
            *state
        } else if position == BlockPos::new(self.origin.x, self.origin.y - 1, self.origin.z) {
            if self.valid_base {
                BlockStateId::new(1)
            } else {
                BlockStateId::new(9)
            }
        } else {
            BlockStateId::new(8)
        }
    }

    fn same_block_type(&self, left: BlockStateId, right: BlockStateId) -> bool {
        left == right
    }

    fn can_be_replaced(&self, _state: BlockStateId) -> bool {
        false
    }

    fn matches_stem_replacement(&mut self, _state: BlockStateId, _position: BlockPos) -> bool {
        true
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn offer_fungus_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        self.states.insert(position, state);
        false
    }

    fn destroy_fungus_target(&mut self, position: BlockPos, drop_items: bool) -> bool {
        assert!(drop_items);
        self.destroyed.push(position);
        self.states.insert(position, BlockStateId::new(0));
        false
    }

    fn offer_fungus_vine(
        &mut self,
        _position: BlockPos,
        _placement: FungusVinePlacement,
        _flags: u32,
    ) -> bool {
        panic!("hat admission is disabled in this fixture")
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

impl FungusBonemealWorld for BonemealFixture {
    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        BlockStateId::new(1)
    }

    fn is_required_fungus_base(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn is_inside_build_height(&self, _position: BlockPos) -> bool {
        true
    }

    fn resolve_planted_fungus(&mut self) -> bool {
        true
    }

    fn place_planted_fungus<R: GenerationRandom>(
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
            bounds: Vec::new(),
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
        panic!("huge fungus does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("huge fungus does not draw Gaussian values")
    }
}
