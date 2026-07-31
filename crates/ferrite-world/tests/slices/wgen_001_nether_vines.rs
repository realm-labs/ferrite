use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::nether_vines::{
    TwistingVinesConfig, TwistingVinesPlacement, TwistingVinesWorld, WeepingVinesPlacement,
    WeepingVinesWorld, place_twisting_vines, place_weeping_vines,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn twisting_vines_draws_xyz_then_modifiers_and_only_draws_age_for_an_offered_head() {
    let origin = BlockPos::new(0, 10, 0);
    let support = BlockStateId::new(1);
    let mut world = TwistingFixture {
        origin,
        support,
        empty_reads: Vec::new(),
        state_reads: Vec::new(),
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom {
        integers: [1, 1, 1, 1, 1, 0].into_iter().collect(),
        bounds: Vec::new(),
    };
    assert!(
        place_twisting_vines(
            &mut world,
            origin,
            TwistingVinesConfig {
                spread_width: 1,
                spread_height: 1,
                maximum_height: 1,
            },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(random.bounds, [3, 3, 3, 6, 5, 9]);
    assert_eq!(
        world.offers,
        [(origin, TwistingVinesPlacement::Head { age: 17 }, 2)]
    );
    assert_eq!(
        world.state_reads,
        [BlockPos::new(0, 9, 0), BlockPos::new(0, 9, 0)]
    );
}

#[test]
fn weeping_vines_runs_all_roof_then_column_attempts_with_fixed_draw_prefixes() {
    let origin = BlockPos::new(0, 10, 0);
    let support = BlockStateId::new(1);
    let mut integers = Vec::with_capacity(2_200);
    for _ in 0..200 {
        integers.extend([0; 6]);
    }
    for _ in 0..100 {
        integers.extend([0, 0, 0, 0, 0, 0, 0, 1, 1, 0]);
    }
    let mut random = ScriptedRandom {
        integers: integers.into_iter().collect(),
        bounds: Vec::new(),
    };
    let mut world = WeepingFixture {
        origin,
        support,
        wart_offers: Vec::new(),
        vine_offers: Vec::new(),
    };
    assert!(place_weeping_vines(&mut world, origin, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds.len(), 2_200);
    assert_eq!(&random.bounds[..6], [6, 6, 2, 5, 6, 6]);
    assert_eq!(&random.bounds[1_200..1_210], [8, 8, 2, 7, 8, 8, 8, 6, 5, 9]);
    assert_eq!(world.wart_offers.len(), 201);
    assert_eq!(world.vine_offers.len(), 100);
    assert!(
        world
            .vine_offers
            .iter()
            .all(|offer| { *offer == (origin, WeepingVinesPlacement::Head { age: 17 }, 2,) })
    );
}

#[derive(Debug)]
struct TwistingFixture {
    origin: BlockPos,
    support: BlockStateId,
    empty_reads: Vec<BlockPos>,
    state_reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, TwistingVinesPlacement, u32)>,
}

impl TwistingVinesWorld for TwistingFixture {
    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        self.empty_reads.push(position);
        position == self.origin
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.state_reads.push(position);
        self.support
    }

    fn is_twisting_vines_support(&self, state: BlockStateId) -> bool {
        state == self.support
    }

    fn is_outside_build_height(&self, _position: BlockPos) -> bool {
        false
    }

    fn offer_twisting_vines(
        &mut self,
        position: BlockPos,
        placement: TwistingVinesPlacement,
        flags: u32,
    ) -> bool {
        self.offers.push((position, placement, flags));
        false
    }
}

#[derive(Debug)]
struct WeepingFixture {
    origin: BlockPos,
    support: BlockStateId,
    wart_offers: Vec<(BlockPos, u32)>,
    vine_offers: Vec<(BlockPos, WeepingVinesPlacement, u32)>,
}

impl WeepingVinesWorld for WeepingFixture {
    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        position == self.origin
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position == BlockPos::new(0, 11, 0) {
            self.support
        } else {
            BlockStateId::new(2)
        }
    }

    fn is_weeping_vines_support(&self, state: BlockStateId) -> bool {
        state == self.support
    }

    fn offer_nether_wart(&mut self, position: BlockPos, flags: u32) -> bool {
        self.wart_offers.push((position, flags));
        false
    }

    fn offer_weeping_vines(
        &mut self,
        position: BlockPos,
        placement: WeepingVinesPlacement,
        flags: u32,
    ) -> bool {
        self.vine_offers.push((position, placement, flags));
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        panic!("twisting vines does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("twisting vines does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("twisting vines does not draw Gaussian values")
    }
}
