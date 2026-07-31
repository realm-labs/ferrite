use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::chorus::{ChorusPlantWorld, place_chorus_plant};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn chorus_recurses_depth_first_and_an_admitted_child_suppresses_the_parent_flower() {
    let origin = BlockPos::new(0, 10, 0);
    let plant = BlockStateId::new(2);
    let flower = BlockStateId::new(3);
    let mut world = ChorusFixture {
        origin,
        plant,
        flower,
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom {
        integers: [0, 0, 0, 0, 0].into_iter().collect(),
        bounds: Vec::new(),
    };
    assert!(place_chorus_plant(&mut world, origin, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [4, 4, 4, 4, 4]);
    assert_eq!(world.offers.len(), 10);
    assert_eq!(world.offers[0], (origin, plant, 2));
    assert_eq!(
        world.offers.last(),
        Some(&(BlockPos::new(0, 13, -1), flower, 2))
    );
    assert!(
        !world
            .offers
            .iter()
            .any(|offer| offer.0 == BlockPos::new(0, 12, 0) && offer.1 == flower)
    );
}

#[derive(Debug)]
struct ChorusFixture {
    origin: BlockPos,
    plant: BlockStateId,
    flower: BlockStateId,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl ChorusPlantWorld for ChorusFixture {
    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        position != BlockPos::new(self.origin.x, self.origin.y - 1, self.origin.z)
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position == BlockPos::new(self.origin.x, self.origin.y - 1, self.origin.z) {
            BlockStateId::new(1)
        } else {
            BlockStateId::new(0)
        }
    }

    fn supports_chorus_plant(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn connected_chorus_plant_state(
        &mut self,
        _position: BlockPos,
        _neighbors: [BlockStateId; 6],
    ) -> BlockStateId {
        self.plant
    }

    fn chorus_flower_state(&self, age: u8) -> BlockStateId {
        assert_eq!(age, 5);
        self.flower
    }

    fn offer_chorus_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
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
        panic!("chorus does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("chorus does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("chorus does not draw Gaussian values")
    }
}
