use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::speleothem::{
    PointedThickness, SpeleothemConfig, SpeleothemWorld, place_speleothem,
};
use ferrite_world::id::BlockStateId;

#[test]
fn speleothem_reads_above_then_below_and_uses_strict_taller_gate_after_patch() {
    let origin = BlockPos::new(0, 20, 0);
    let base = BlockStateId::new(1);
    let pointed = BlockStateId::new(2);
    let mut world = SpeleothemFixture {
        base,
        reads: Vec::new(),
        configured: Vec::new(),
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom {
        integers: VecDeque::new(),
        floats: [1.0, 1.0, 1.0, 1.0, 0.0].into_iter().collect(),
        bounds: Vec::new(),
    };
    assert!(
        place_speleothem(
            &mut world,
            origin,
            SpeleothemConfig {
                base_block: base,
                pointed_block: pointed,
                chance_of_directional_spread: 0.2,
                chance_of_spread_radius2: 0.7,
                chance_of_spread_radius3: 0.5,
                chance_of_taller_generation: 0.5,
            },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(
        &world.reads[..2],
        [BlockPos::new(0, 21, 0), BlockPos::new(0, 19, 0)]
    );
    assert!(random.bounds.is_empty());
    assert_eq!(
        world.configured,
        [
            (Direction::Down, PointedThickness::Frustum, false),
            (Direction::Down, PointedThickness::Tip, true),
        ]
    );
    assert_eq!(
        world.offers,
        [
            (origin, BlockStateId::new(10), 2),
            (BlockPos::new(0, 19, 0), BlockStateId::new(11), 2),
        ]
    );
}

#[derive(Debug)]
struct SpeleothemFixture {
    base: BlockStateId,
    reads: Vec<BlockPos>,
    configured: Vec<(Direction, PointedThickness, bool)>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl SpeleothemWorld for SpeleothemFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        if position == BlockPos::new(0, 21, 0) {
            self.base
        } else {
            BlockStateId::new(0)
        }
    }

    fn is_base_block_identity(&self, state: BlockStateId, base: BlockStateId) -> bool {
        state == base
    }

    fn is_replaceable_speleothem_block(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_air_or_water_block(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_water_at(&mut self, position: BlockPos) -> bool {
        position == BlockPos::new(0, 19, 0)
    }

    fn configure_pointed_state(
        &mut self,
        _default_state: BlockStateId,
        direction: Direction,
        thickness: PointedThickness,
        waterlogged: bool,
    ) -> Option<BlockStateId> {
        self.configured.push((direction, thickness, waterlogged));
        Some(match thickness {
            PointedThickness::Frustum => BlockStateId::new(10),
            PointedThickness::Tip => BlockStateId::new(11),
            PointedThickness::TipMerge | PointedThickness::Middle | PointedThickness::Base => {
                panic!("height-two fixture")
            }
        })
    }

    fn offer_speleothem_block(
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
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.floats.pop_front().expect("scripted float")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("speleothem does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("speleothem does not draw Gaussian values")
    }
}
