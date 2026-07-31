use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::jungle_temple::{
    JungleTemplePiece, JungleTempleWorld, jungle_temple_start_allowed,
};
use ferrite_world::generation::structure::piece::{FluidState, HorizontalDirection, PieceWorld};
use ferrite_world::generation::structure::processor::StructureState;

#[test]
fn start_gate_uses_four_one_cell_beyond_footprint_probes() {
    let mut probes = Vec::new();
    let allowed = jungle_temple_start_allowed(
        &mut |x, z| {
            probes.push((x, z));
            if (x, z) == (28, 47) { 62 } else { 63 }
        },
        pos(16, 0, 32),
        63,
    );
    assert!(!allowed);
    assert_eq!(probes, [(16, 32), (16, 47), (28, 32), (28, 47)]);
}

#[test]
fn full_layout_consumes_1522_selector_floats_and_places_four_seeded_containers() {
    let mut piece = JungleTemplePiece::new(pos(0, 0, 0), HorizontalDirection::South);
    let clip = BlockBox::new(pos(-20, -100, -20), pos(40, 200, 40)).unwrap();
    let mut world = World {
        terrain_height: 70,
        ..World::default()
    };
    let mut random = CountingRandom::default();
    let mut seeds = [10_i64, 20, 30, 40].into_iter();
    let mut next_seed = || seeds.next().expect("two dispensers and two chests");

    assert!(piece.place(&mut world, &clip, &mut random, &mut next_seed));
    assert_eq!(piece.average_ground_height, 70);
    assert_eq!(piece.piece.bounds.minimum.y, 70);
    assert_eq!(world.height_probes.len(), 12 * 15);
    assert_eq!(random.float_draws, 1_522);
    assert_eq!(world.writes.len(), 2_302);
    assert!(piece.placed_trap_one);
    assert!(piece.placed_trap_two);
    assert!(piece.placed_main_chest);
    assert!(piece.placed_hidden_chest);
    assert_eq!(
        world
            .loot
            .iter()
            .map(|(_, table, seed)| (table.as_str(), *seed))
            .collect::<Vec<_>>(),
        [
            ("minecraft:chests/jungle_temple_dispenser", 10),
            ("minecraft:chests/jungle_temple_dispenser", 20),
            ("minecraft:chests/jungle_temple", 30),
            ("minecraft:chests/jungle_temple", 40),
        ]
    );
    assert!(seeds.next().is_none());
}

#[test]
fn empty_probe_clip_aborts_before_layout_randomness_and_latches() {
    let mut piece = JungleTemplePiece::new(pos(0, 0, 0), HorizontalDirection::East);
    let clip = BlockBox::point(pos(100, 64, 100));
    let mut world = World::default();
    let mut random = CountingRandom::default();
    let mut no_seed = || panic!("aborted temple cannot seed loot");

    assert!(!piece.place(&mut world, &clip, &mut random, &mut no_seed));
    assert_eq!(piece.average_ground_height, -1);
    assert_eq!(random.float_draws, 0);
    assert!(world.writes.is_empty());
    assert!(!piece.placed_trap_one);
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    terrain_height: i32,
    height_probes: Vec<(i32, i32)>,
    states: BTreeMap<BlockPos, StructureState>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    loot: Vec<(BlockPos, String, i64)>,
}

impl JungleTempleWorld for World {
    fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32 {
        self.height_probes.push((x, z));
        self.terrain_height
    }
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
        self.states.get(&position).is_some_and(|state| {
            matches!(
                state.block.as_str(),
                "minecraft:chest" | "minecraft:dispenser"
            )
        })
    }
    fn install_loot(&mut self, position: BlockPos, table: &str, seed: i64) {
        self.loot.push((position, table.into(), seed));
    }
}

#[derive(Default)]
struct CountingRandom {
    float_draws: usize,
}

impl GenerationRandom for CountingRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        0
    }
    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        0.39
    }
    fn next_f64(&mut self) -> f64 {
        0.0
    }
    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
