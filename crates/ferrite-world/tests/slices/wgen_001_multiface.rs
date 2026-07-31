use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::multiface::{
    MultifaceGrowthConfig, MultifaceSpreadType, MultifaceWorld, place_multiface_growth,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn primary_offer_marks_and_spreads_after_failure_with_distinct_flags() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = MultifaceFixture::new(origin);
    world.primary_support = true;
    world.spread_same_plane = true;
    let config = config(1, 1.0);
    let mut random = ScriptedRandom::new([5, 4, 3, 2, 1], [0.0]);

    assert!(place_multiface_growth(&mut world, origin, config, &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [6, 5, 4, 3, 2]);
    assert_eq!(random.float_draws, 1);
    let same_plane = BlockPos::new(0, 20, -1);
    assert_eq!(
        world.offers,
        [
            (origin, BlockStateId::new(3), 3),
            (same_plane, BlockStateId::new(3), 2),
        ]
    );
    assert_eq!(world.marks, [origin, same_plane]);
    assert_eq!(
        world.spread_checks,
        [
            (
                origin,
                origin,
                Direction::Up,
                MultifaceSpreadType::SamePosition,
            ),
            (
                origin,
                same_plane,
                Direction::Up,
                MultifaceSpreadType::SamePlane,
            ),
        ]
    );
}

#[test]
fn search_range_retries_the_same_distance_one_candidate() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = MultifaceFixture::new(origin);
    world.support_after_query = Some(3);
    let config = config(3, 0.0);
    let mut random = ScriptedRandom::new([], [0.0]);

    assert!(place_multiface_growth(&mut world, origin, config, &mut random, |_| true).unwrap());

    let candidate = BlockPos::new(0, 21, 0);
    assert_eq!(world.placement_candidates, [candidate]);
    assert_eq!(
        world
            .reads
            .iter()
            .filter(|position| **position == candidate)
            .count(),
        4
    );
    assert_eq!(world.offers, [(candidate, BlockStateId::new(3), 3)]);
    assert_eq!(random.float_draws, 1);
}

#[test]
fn null_state_on_the_first_supported_face_rejects_the_whole_candidate() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = MultifaceFixture::new(origin);
    world.primary_support = true;
    world.null_placement_at = Some(origin);
    let mut config = config(1, 1.0);
    config.place_on_floor = true;
    let mut random = ScriptedRandom::new([1], []);

    assert!(!place_multiface_growth(&mut world, origin, config, &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [2]);
    assert_eq!(random.float_draws, 0);
    assert_eq!(world.placement_candidates, [origin]);
    assert!(world.offers.is_empty());
}

fn config(search_range: u32, chance_of_spreading: f32) -> MultifaceGrowthConfig {
    MultifaceGrowthConfig {
        block: BlockStateId::new(2),
        place_on_ceiling: true,
        place_on_floor: false,
        place_on_walls: false,
        search_range,
        chance_of_spreading,
    }
}

#[derive(Debug)]
struct MultifaceFixture {
    origin: BlockPos,
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    marks: Vec<BlockPos>,
    spread_checks: Vec<(BlockPos, BlockPos, Direction, MultifaceSpreadType)>,
    placement_candidates: Vec<BlockPos>,
    primary_support: bool,
    support_after_query: Option<usize>,
    support_queries: usize,
    spread_same_plane: bool,
    null_placement_at: Option<BlockPos>,
}

impl MultifaceFixture {
    fn new(origin: BlockPos) -> Self {
        Self {
            origin,
            reads: Vec::new(),
            offers: Vec::new(),
            marks: Vec::new(),
            spread_checks: Vec::new(),
            placement_candidates: Vec::new(),
            primary_support: false,
            support_after_query: None,
            support_queries: 0,
            spread_same_plane: false,
            null_placement_at: None,
        }
    }
}

impl MultifaceWorld for MultifaceFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        if position == self.origin
            || position == BlockPos::new(self.origin.x, self.origin.y, self.origin.z - 1)
        {
            BlockStateId::new(0)
        } else if position == BlockPos::new(self.origin.x, self.origin.y + 1, self.origin.z) {
            if self.primary_support {
                BlockStateId::new(9)
            } else {
                BlockStateId::new(0)
            }
        } else if position == BlockPos::new(self.origin.x, self.origin.y + 2, self.origin.z) {
            self.support_queries += 1;
            if self
                .support_after_query
                .is_some_and(|minimum| self.support_queries >= minimum)
            {
                BlockStateId::new(9)
            } else {
                BlockStateId::new(0)
            }
        } else {
            BlockStateId::new(9)
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_water_block_identity(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_configured_multiface(&self, state: BlockStateId, configured: BlockStateId) -> bool {
        state == configured
    }

    fn is_multiface_spreadable(&self, _configured: BlockStateId) -> bool {
        true
    }

    fn can_be_placed_on(&self, support: BlockStateId) -> bool {
        support == BlockStateId::new(9)
    }

    fn placement_state(
        &mut self,
        _configured: BlockStateId,
        _current: BlockStateId,
        position: BlockPos,
        _face: Direction,
    ) -> Option<BlockStateId> {
        self.placement_candidates.push(position);
        (self.null_placement_at != Some(position)).then_some(BlockStateId::new(3))
    }

    fn has_face(&self, _state: BlockStateId, face: Direction) -> bool {
        face == Direction::Up
    }

    fn can_spread_into(
        &mut self,
        source: BlockPos,
        target: BlockPos,
        target_face: Direction,
        spread_type: MultifaceSpreadType,
    ) -> bool {
        self.spread_checks
            .push((source, target, target_face, spread_type));
        self.spread_same_plane && spread_type == MultifaceSpreadType::SamePlane
    }

    fn offer_multiface(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        flags == 2
    }

    fn mark_for_postprocessing(&mut self, position: BlockPos) {
        self.marks.push(position);
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
        panic!("multiface growth does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("multiface growth does not draw Gaussian values")
    }
}
