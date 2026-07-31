use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::buried_treasure::{
    BuriedTreasureWorld, place_buried_treasure,
};
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::processor::StructureState;

#[test]
fn admitted_support_encloses_in_enum_order_and_conditionally_seeds_chest() {
    let candidate = pos(9, 5, 9);
    let mut world = World {
        ocean_floor: 5,
        minimum_y: -64,
        loot_targets: BTreeSet::from([candidate]),
        ..World::default()
    };
    world
        .states
        .insert(candidate, StructureState::new("minecraft:water"));
    world
        .states
        .insert(pos(9, 4, 9), StructureState::new("minecraft:stone"));
    let seed_calls = Cell::new(0);
    let result = place_buried_treasure(&mut world, 9, 9, &BlockBox::point(candidate), || {
        seed_calls.set(seed_calls.get() + 1);
        73
    });
    assert_eq!(result.final_box, Some(BlockBox::point(candidate)));
    assert!(result.chest_attempted);
    assert_eq!(seed_calls.get(), 1);
    assert_eq!(world.loot, [(candidate, 73)]);
    assert_eq!(
        world
            .writes
            .iter()
            .take(5)
            .map(|write| (write.0, write.1.block.as_str(), write.2))
            .collect::<Vec<_>>(),
        [
            (pos(9, 6, 9), "minecraft:sand", 3),
            (pos(9, 5, 8), "minecraft:stone", 3),
            (pos(9, 5, 10), "minecraft:stone", 3),
            (pos(8, 5, 9), "minecraft:stone", 3),
            (pos(10, 5, 9), "minecraft:stone", 3),
        ]
    );
    assert_eq!(world.writes.last().unwrap().2, 2);
}

#[test]
fn exhausted_search_and_processing_box_miss_consume_no_loot_seed() {
    let calls = Cell::new(0);
    let mut no_support = World {
        ocean_floor: 2,
        minimum_y: 0,
        ..World::default()
    };
    let result = place_buried_treasure(
        &mut no_support,
        0,
        0,
        &BlockBox::point(pos(0, 0, 0)),
        || {
            calls.set(calls.get() + 1);
            1
        },
    );
    assert_eq!(result.final_box, None);
    assert_eq!(calls.get(), 0);

    let candidate = pos(0, 2, 0);
    no_support
        .states
        .insert(pos(0, 1, 0), StructureState::new("minecraft:sandstone"));
    no_support.loot_targets.insert(candidate);
    let result = place_buried_treasure(
        &mut no_support,
        0,
        0,
        &BlockBox::point(pos(20, 2, 20)),
        || {
            calls.set(calls.get() + 1);
            2
        },
    );
    assert!(!result.chest_attempted);
    assert_eq!(calls.get(), 0);
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    states: BTreeMap<BlockPos, StructureState>,
    loot_targets: BTreeSet<BlockPos>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    loot: Vec<(BlockPos, i64)>,
    ocean_floor: i32,
    minimum_y: i32,
}

impl PieceWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:air"))
    }

    fn fluid_at(&mut self, _position: BlockPos) -> FluidState {
        FluidState::Empty
    }

    fn set_state(&mut self, position: BlockPos, state: StructureState, flags: u32) -> bool {
        self.writes.push((position, state.clone(), flags));
        self.states.insert(position, state);
        true
    }

    fn schedule_fluid_tick(&mut self, _position: BlockPos, _fluid: FluidState, _delay: u32) {}

    fn mark_shape_postprocessing(&mut self, _position: BlockPos) {}

    fn solid_render(&mut self, _position: BlockPos) -> bool {
        false
    }

    fn is_loot_container(&mut self, position: BlockPos) -> bool {
        self.loot_targets.contains(&position)
    }

    fn install_loot(&mut self, position: BlockPos, _table: &str, seed: i64) {
        self.loot.push((position, seed));
    }
}

impl BuriedTreasureWorld for World {
    fn ocean_floor_height(&mut self, _x: i32, _z: i32) -> i32 {
        self.ocean_floor
    }

    fn minimum_y(&self) -> i32 {
        self.minimum_y
    }
}
