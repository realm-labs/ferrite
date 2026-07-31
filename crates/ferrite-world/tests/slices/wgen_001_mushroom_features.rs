use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::mushroom::{
    HugeMushroomConfig, HugeMushroomKind, HugeMushroomWorld, MushroomCapProperties,
    place_huge_mushroom,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn brown_mushroom_clears_full_upper_square_then_places_cap_before_trunk() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = MushroomFixture::default();
    let mut random = ScriptedRandom::new([0, 1]);
    assert!(
        place_huge_mushroom(
            &mut world,
            origin,
            HugeMushroomKind::Brown,
            HugeMushroomConfig { foliage_radius: 1 },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(random.bounds, [3, 12]);
    assert_eq!(world.clearance_reads, 13);
    assert_eq!(world.provider_calls.len(), 9);
    assert!(
        world.provider_calls[..5]
            .iter()
            .all(|call| *call == ("cap", origin))
    );
    assert!(
        world.provider_calls[5..]
            .iter()
            .all(|call| *call == ("stem", origin))
    );
    assert_eq!(
        &world.offers[..5],
        [
            (BlockPos::new(-1, 14, 0), BlockStateId::new(1), 3),
            (BlockPos::new(0, 14, -1), BlockStateId::new(1), 3),
            (BlockPos::new(0, 14, 0), BlockStateId::new(1), 3),
            (BlockPos::new(0, 14, 1), BlockStateId::new(1), 3),
            (BlockPos::new(1, 14, 0), BlockStateId::new(1), 3),
        ]
    );
}

#[test]
fn red_mushroom_clearance_uses_only_the_sentinel_trunk_column() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = MushroomFixture::default();
    let mut random = ScriptedRandom::new([0, 1]);
    assert!(
        place_huge_mushroom(
            &mut world,
            origin,
            HugeMushroomKind::Red,
            HugeMushroomConfig { foliage_radius: 2 },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(world.clearance_reads, 5);
    assert_eq!(world.provider_calls.len(), 49);
    assert!(
        world.provider_calls[..45]
            .iter()
            .all(|call| *call == ("cap", origin))
    );
    assert!(
        world.provider_calls[45..]
            .iter()
            .all(|call| *call == ("stem", origin))
    );
    assert_eq!(world.offers.len(), 49);
}

#[derive(Debug, Default)]
struct MushroomFixture {
    clearance_phase: bool,
    clearance_reads: usize,
    provider_calls: Vec<(&'static str, BlockPos)>,
    configured_caps: Vec<MushroomCapProperties>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl HugeMushroomWorld<ScriptedRandom> for MushroomFixture {
    fn minimum_y(&self) -> i32 {
        0
    }

    fn maximum_y_exclusive(&self) -> i32 {
        100
    }

    fn can_place_mushroom_on(&mut self, _position: BlockPos, _random: &mut ScriptedRandom) -> bool {
        self.clearance_phase = true;
        true
    }

    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        if self.clearance_phase && self.provider_calls.is_empty() {
            self.clearance_reads += 1;
        }
        BlockStateId::new(0)
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_leaves(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_replaceable_by_mushrooms(&self, _state: BlockStateId) -> bool {
        false
    }

    fn provide_cap_state(
        &mut self,
        provider_position: BlockPos,
        _random: &mut ScriptedRandom,
    ) -> BlockStateId {
        self.provider_calls.push(("cap", provider_position));
        BlockStateId::new(1)
    }

    fn provide_stem_state(
        &mut self,
        provider_position: BlockPos,
        _random: &mut ScriptedRandom,
    ) -> BlockStateId {
        self.provider_calls.push(("stem", provider_position));
        BlockStateId::new(2)
    }

    fn configure_brown_cap(
        &mut self,
        state: BlockStateId,
        properties: MushroomCapProperties,
    ) -> BlockStateId {
        self.configured_caps.push(properties);
        state
    }

    fn configure_red_cap(
        &mut self,
        state: BlockStateId,
        properties: MushroomCapProperties,
    ) -> BlockStateId {
        self.configured_caps.push(properties);
        state
    }

    fn offer_mushroom_block(
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
        panic!("huge mushroom does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("huge mushroom does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("huge mushroom does not draw Gaussian values")
    }
}
