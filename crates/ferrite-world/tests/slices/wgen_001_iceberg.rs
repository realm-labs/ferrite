use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::iceberg::{IcebergWorld, place_iceberg};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn ellipse_normalizes_to_sea_level_and_submerged_radius_draws_for_every_candidate() {
    let origin = BlockPos::new(7, 999, 9);
    let mut world = IcebergFixture::new();
    let mut random = CountingRandom::new([4, 0, 0, 0, 0], [0.7, 0.0, 0.8], 0, 0.0, 0.0);

    assert!(
        place_iceberg(
            &mut world,
            origin,
            BlockStateId::new(2),
            &mut random,
            |_| true,
        )
        .unwrap()
    );

    assert_eq!(&random.bounds[..6], [5, 3, 6, 11, 7, 5]);
    assert_eq!(random.float_draws, 14 * 14 * 5);
    assert!(world.reads.iter().all(|position| position.y != origin.y));
    assert!(world.offers.is_empty());
}

#[derive(Debug)]
struct IcebergFixture {
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl IcebergFixture {
    fn new() -> Self {
        Self {
            reads: Vec::new(),
            offers: Vec::new(),
        }
    }
}

impl IcebergWorld for IcebergFixture {
    fn sea_level(&self) -> i32 {
        63
    }

    fn canonical_air(&self) -> BlockStateId {
        BlockStateId::new(0)
    }

    fn source_water(&self) -> BlockStateId {
        BlockStateId::new(1)
    }

    fn snow_block(&self) -> BlockStateId {
        BlockStateId::new(3)
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        BlockStateId::new(9)
    }

    fn is_air(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_snow_block(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_ordinary_ice(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_water_block(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_fixed_iceberg_state(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_snow_layer(&self, _state: BlockStateId) -> bool {
        false
    }

    fn offer_iceberg_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct CountingRandom {
    integers: VecDeque<u32>,
    doubles: VecDeque<f64>,
    default_integer: u32,
    default_float: f32,
    default_double: f64,
    bounds: Vec<u32>,
    float_draws: usize,
}

impl CountingRandom {
    fn new(
        integers: impl IntoIterator<Item = u32>,
        doubles: impl IntoIterator<Item = f64>,
        default_integer: u32,
        default_float: f32,
        default_double: f64,
    ) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            doubles: doubles.into_iter().collect(),
            default_integer,
            default_float,
            default_double,
            bounds: Vec::new(),
            float_draws: 0,
        }
    }
}

impl GenerationRandom for CountingRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().unwrap_or(self.default_integer);
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        self.default_float
    }

    fn next_f64(&mut self) -> f64 {
        self.doubles.pop_front().unwrap_or(self.default_double)
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("iceberg feature does not draw Gaussian values")
    }
}
