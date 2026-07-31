use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::large_dripstone::{
    FloatRange, LargeDripstoneConfig, LargeDripstoneWorld, place_large_dripstone,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn large_dripstone_admission_samples_both_shapes_even_when_radius_one_cannot_retreat() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = DripstoneFixture {
        ceiling_y: 13,
        floor_y: 6,
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom {
        integers: [0].into_iter().collect(),
        floats: [0.5; 4].into_iter().collect(),
        bounds: Vec::new(),
        default_float: 1.0,
    };
    assert!(place_large_dripstone(&mut world, origin, config(1), &mut random, |_| true,).unwrap());
    assert_eq!(random.bounds, [1]);
    assert!(random.floats.is_empty());
    assert!(world.offers.is_empty());
}

#[test]
fn large_dripstone_successful_retreat_places_open_cells_with_flags_two() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = DripstoneFixture {
        ceiling_y: 15,
        floor_y: 5,
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom {
        integers: [0].into_iter().collect(),
        floats: [0.5; 4].into_iter().collect(),
        bounds: Vec::new(),
        default_float: 1.0,
    };
    assert!(place_large_dripstone(&mut world, origin, config(2), &mut random, |_| true,).unwrap());
    assert!(!world.offers.is_empty());
    assert!(world.offers.iter().all(|offer| offer.2 == 2));
    assert_eq!(world.offers[0].1, BlockStateId::new(3));
}

fn config(radius: i32) -> LargeDripstoneConfig {
    LargeDripstoneConfig {
        floor_to_ceiling_search_range: 12,
        radius_minimum: radius,
        radius_maximum: radius,
        maximum_radius_to_cave_height_ratio: 1.0,
        height_scale: FloatRange {
            minimum: 1.0,
            maximum: 1.0,
        },
        stalactite_bluntness: FloatRange {
            minimum: 0.5,
            maximum: 0.5,
        },
        stalagmite_bluntness: FloatRange {
            minimum: 0.5,
            maximum: 0.5,
        },
        wind_speed: FloatRange {
            minimum: 0.0,
            maximum: 0.0,
        },
        minimum_radius_for_wind: 4,
        minimum_bluntness_for_wind: 0.6,
        dripstone_block: BlockStateId::new(3),
    }
}

#[derive(Debug)]
struct DripstoneFixture {
    ceiling_y: i32,
    floor_y: i32,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl LargeDripstoneWorld for DripstoneFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position.y >= self.ceiling_y || position.y <= self.floor_y {
            BlockStateId::new(2)
        } else {
            BlockStateId::new(0)
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_water_block(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_lava_block(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_dripstone_block(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_replaceable_dripstone_block(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(2)
    }

    fn is_base_stone_overworld(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(2)
    }

    fn world_surface_worldgen_height(&mut self, _x: i32, _z: i32) -> i32 {
        100
    }

    fn offer_large_dripstone(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        flags: u32,
    ) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    bounds: Vec<u32>,
    default_float: f32,
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.floats.pop_front().unwrap_or(self.default_float)
    }

    fn next_f64(&mut self) -> f64 {
        panic!("large dripstone does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("large dripstone does not draw Gaussian values")
    }
}
