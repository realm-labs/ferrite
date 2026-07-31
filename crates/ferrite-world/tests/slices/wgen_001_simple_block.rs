use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::simple_block::{
    DoublePlantHalf, MossFace, MossFaces, SimpleBlockKind, SimpleBlockWorld, place_simple_block,
};
use ferrite_world::id::BlockStateId;

#[test]
fn ordinary_simple_block_schedules_the_postwrite_readable_state_after_failed_offer() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = SimpleFixture::ordinary(origin);
    let mut random = NoRandom;
    assert!(place_simple_block(&mut world, origin, true, &mut random, |_| true).unwrap());
    assert_eq!(world.offers, [(origin, BlockStateId::new(7), 2)]);
    assert_eq!(world.scheduled, [(origin, BlockStateId::new(9), 1)]);
}

#[test]
fn pale_moss_uses_level_rng_and_recomputes_base_after_topper_write() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = SimpleFixture::moss(origin);
    let mut random = NoRandom;
    assert!(place_simple_block(&mut world, origin, false, &mut random, |_| true).unwrap());
    assert_eq!(world.level_boolean_draws, 1);
    assert_eq!(
        world.offers,
        [
            (origin, BlockStateId::new(10), 2),
            (BlockPos::new(0, 11, 0), BlockStateId::new(11), 2),
            (origin, BlockStateId::new(12), 2),
        ]
    );
}

#[test]
fn double_plant_copies_each_halfs_prewrite_water_state_and_writes_lower_first() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = SimpleFixture::double_plant(origin);
    let mut random = NoRandom;
    assert!(place_simple_block(&mut world, origin, false, &mut random, |_| true).unwrap());
    assert_eq!(
        world.offers,
        [
            (origin, BlockStateId::new(20), 2),
            (BlockPos::new(0, 11, 0), BlockStateId::new(21), 2),
        ]
    );
}

#[derive(Debug)]
struct SimpleFixture {
    origin: BlockPos,
    kind: SimpleBlockKind,
    selected: BlockStateId,
    states: BTreeMap<BlockPos, BlockStateId>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    scheduled: Vec<(BlockPos, BlockStateId, u32)>,
    level_boolean_draws: usize,
    accept_writes: bool,
}

impl SimpleFixture {
    fn ordinary(origin: BlockPos) -> Self {
        Self {
            origin,
            kind: SimpleBlockKind::Ordinary,
            selected: BlockStateId::new(7),
            states: [(origin, BlockStateId::new(9))].into_iter().collect(),
            offers: Vec::new(),
            scheduled: Vec::new(),
            level_boolean_draws: 0,
            accept_writes: false,
        }
    }

    fn moss(origin: BlockPos) -> Self {
        Self {
            origin,
            kind: SimpleBlockKind::MossyCarpet,
            selected: BlockStateId::new(50),
            states: BTreeMap::new(),
            offers: Vec::new(),
            scheduled: Vec::new(),
            level_boolean_draws: 0,
            accept_writes: true,
        }
    }

    fn double_plant(origin: BlockPos) -> Self {
        Self {
            origin,
            kind: SimpleBlockKind::DoublePlant,
            selected: BlockStateId::new(19),
            states: BTreeMap::new(),
            offers: Vec::new(),
            scheduled: Vec::new(),
            level_boolean_draws: 0,
            accept_writes: true,
        }
    }
}

impl SimpleBlockWorld<NoRandom> for SimpleFixture {
    fn provide_simple_state(
        &mut self,
        _origin: BlockPos,
        _random: &mut NoRandom,
    ) -> Option<BlockStateId> {
        Some(self.selected)
    }

    fn simple_block_kind(&self, _state: BlockStateId) -> SimpleBlockKind {
        self.kind
    }

    fn state_can_survive(&mut self, _state: BlockStateId, _position: BlockPos) -> bool {
        true
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.states
            .get(&position)
            .copied()
            .unwrap_or(BlockStateId::new(0))
    }

    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        self.block_state(position) == BlockStateId::new(0)
    }

    fn state_has_waterlogged(&self, _state: BlockStateId) -> bool {
        self.kind == SimpleBlockKind::DoublePlant
    }

    fn is_water_at(&mut self, position: BlockPos) -> bool {
        self.kind == SimpleBlockKind::DoublePlant && position == self.origin
    }

    fn configure_double_plant_half(
        &mut self,
        state: BlockStateId,
        half: DoublePlantHalf,
        waterlogged: Option<bool>,
    ) -> Option<BlockStateId> {
        if self.kind != SimpleBlockKind::DoublePlant {
            return Some(state);
        }
        Some(match (half, waterlogged) {
            (DoublePlantHalf::Lower, Some(true)) => BlockStateId::new(20),
            (DoublePlantHalf::Upper, Some(false)) => BlockStateId::new(21),
            _ => panic!("unexpected double-plant half"),
        })
    }

    fn default_pale_moss_carpet(&self) -> BlockStateId {
        BlockStateId::new(8)
    }

    fn is_base_pale_moss(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(10) || state == BlockStateId::new(12)
    }

    fn is_nonbase_pale_moss(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(11)
    }

    fn is_replaceable_for_pale_moss(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn pale_moss_face(&self, state: BlockStateId, direction: Direction) -> MossFace {
        if state == BlockStateId::new(11) && direction == Direction::North {
            MossFace::Low
        } else {
            MossFace::None
        }
    }

    fn pale_moss_face_supported(&mut self, position: BlockPos, direction: Direction) -> bool {
        (position == self.origin || position == BlockPos::new(0, 11, 0))
            && direction == Direction::North
    }

    fn configure_pale_moss(
        &mut self,
        _default_state: BlockStateId,
        base: bool,
        faces: MossFaces,
    ) -> BlockStateId {
        if base && faces.north == MossFace::Tall {
            BlockStateId::new(12)
        } else if base {
            BlockStateId::new(10)
        } else {
            BlockStateId::new(11)
        }
    }

    fn next_level_bool(&mut self) -> bool {
        self.level_boolean_draws += 1;
        true
    }

    fn offer_simple_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        if self.accept_writes {
            self.states.insert(position, state);
            true
        } else {
            false
        }
    }

    fn schedule_block_tick(&mut self, position: BlockPos, block_state: BlockStateId, delay: u32) {
        self.scheduled.push((position, block_state, delay));
    }
}

#[derive(Debug)]
struct NoRandom;

impl GenerationRandom for NoRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        panic!("simple-block fixture provider does not draw integers")
    }

    fn next_f32(&mut self) -> f32 {
        panic!("simple-block feature does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("simple-block feature does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("simple-block feature does not draw Gaussian values")
    }
}
