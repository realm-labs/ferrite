use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::monster_room::{MonsterRoomWorld, place_monster_room};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn invalid_first_floor_cell_stops_after_the_two_radius_draws() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = RoomFixture::new(origin);
    world.invalid_first_floor = true;
    let mut random = ScriptedRandom::new([0, 0]);

    assert!(!place_monster_room(&mut world, origin, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [2, 2]);
    assert!(world.offers.is_empty());
    assert_eq!(world.spawner_initializations, 0);
}

#[test]
fn admitted_room_draws_every_floor_before_chest_attempts_and_spawner_handoff() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = RoomFixture::new(origin);
    let mut integers = vec![0, 0];
    integers.extend([0; 49]);
    integers.extend([0; 12]);
    let mut random = ScriptedRandom::new(integers);

    assert!(place_monster_room(&mut world, origin, &mut random, |_| true).unwrap());

    let mut expected_bounds = vec![2, 2];
    expected_bounds.extend([4; 49]);
    expected_bounds.extend([5; 12]);
    assert_eq!(random.bounds, expected_bounds);
    assert_eq!(
        world.offers.last(),
        Some(&(origin, world.default_spawner(), 2))
    );
    assert_eq!(world.loot_initializations, 0);
    assert_eq!(world.spawner_initializations, 1);
}

#[test]
fn admitted_chest_initializes_loot_even_when_the_safe_write_is_protected() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = RoomFixture::new(origin);
    world.admit_origin_chest = true;
    let mut integers = vec![0, 0];
    integers.extend([0; 49]);
    integers.extend([2, 2, 2, 2]);
    let mut random = ScriptedRandom::new(integers);

    assert!(place_monster_room(&mut world, origin, &mut random, |_| true).unwrap());

    let mut expected_bounds = vec![2, 2];
    expected_bounds.extend([4; 49]);
    expected_bounds.extend([5; 4]);
    assert_eq!(random.bounds, expected_bounds);
    assert_eq!(world.loot_initializations, 2);
    assert_eq!(world.spawner_initializations, 1);
    assert!(
        !world
            .offers
            .iter()
            .any(|(position, state, _)| *position == origin && *state == world.default_chest())
    );
}

#[derive(Debug)]
struct RoomFixture {
    origin: BlockPos,
    invalid_first_floor: bool,
    admit_origin_chest: bool,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    loot_initializations: usize,
    spawner_initializations: usize,
}

impl RoomFixture {
    fn new(origin: BlockPos) -> Self {
        Self {
            origin,
            invalid_first_floor: false,
            admit_origin_chest: false,
            offers: Vec::new(),
            loot_initializations: 0,
            spawner_initializations: 0,
        }
    }

    fn opening(&self) -> BlockPos {
        BlockPos::new(self.origin.x - 3, self.origin.y, self.origin.z)
    }
}

impl MonsterRoomWorld for RoomFixture {
    fn minimum_y(&self) -> i32 {
        0
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        let first_floor = BlockPos::new(self.origin.x - 3, self.origin.y - 1, self.origin.z - 3);
        let chest_neighbor = BlockPos::new(self.origin.x + 1, self.origin.y, self.origin.z);
        let chest_empty = position == self.origin
            || [
                BlockPos::new(self.origin.x, self.origin.y, self.origin.z - 1),
                BlockPos::new(self.origin.x, self.origin.y, self.origin.z + 1),
                BlockPos::new(self.origin.x - 1, self.origin.y, self.origin.z),
            ]
            .contains(&position);
        if self.admit_origin_chest && chest_empty {
            BlockStateId::new(0)
        } else if self.admit_origin_chest && position == chest_neighbor {
            BlockStateId::new(9)
        } else if (self.invalid_first_floor && position == first_floor)
            || position == self.opening()
            || position == BlockPos::new(self.opening().x, self.opening().y + 1, self.opening().z)
        {
            BlockStateId::new(0)
        } else {
            BlockStateId::new(9)
        }
    }

    fn is_solid(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(9)
    }

    fn is_empty(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_chest(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_spawner(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_protected_from_features(&self, state: BlockStateId) -> bool {
        self.admit_origin_chest && state == BlockStateId::new(0)
    }

    fn cave_air(&self) -> BlockStateId {
        BlockStateId::new(1)
    }

    fn cobblestone(&self) -> BlockStateId {
        BlockStateId::new(2)
    }

    fn mossy_cobblestone(&self) -> BlockStateId {
        BlockStateId::new(3)
    }

    fn default_chest(&self) -> BlockStateId {
        BlockStateId::new(4)
    }

    fn default_spawner(&self) -> BlockStateId {
        BlockStateId::new(5)
    }

    fn reorient_chest(&mut self, _position: BlockPos, default_state: BlockStateId) -> BlockStateId {
        default_state
    }

    fn offer_monster_room(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn initialize_dungeon_loot<R: GenerationRandom>(
        &mut self,
        _position: BlockPos,
        _random: &mut R,
    ) {
        self.loot_initializations += 1;
    }

    fn initialize_spawner<R: GenerationRandom>(&mut self, _position: BlockPos, _random: &mut R) {
        self.spawner_initializations += 1;
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
        panic!("monster room does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("monster room does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("monster room does not draw Gaussian values")
    }
}
