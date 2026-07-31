use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::desert_pyramid::{
    DesertPyramidPiece, DesertPyramidWorld, desert_pyramid_start_allowed,
};
use ferrite_world::generation::structure::desert_pyramid_archaeology::{
    DesertArchaeologyWorld, place_desert_archaeology,
};
use ferrite_world::generation::structure::piece::{FluidState, HorizontalDirection, PieceWorld};
use ferrite_world::generation::structure::processor::StructureState;

#[test]
fn start_gate_probes_plus_twenty_one_before_piece_randomness() {
    let mut probes = Vec::new();
    assert!(!desert_pyramid_start_allowed(
        &mut |x, z| {
            probes.push((x, z));
            if (x, z) == (37, 53) { 62 } else { 63 }
        },
        pos(16, 0, 32),
        63,
    ));
    assert_eq!(probes, [(16, 32), (16, 53), (37, 32), (37, 53)]);
}

#[test]
fn full_piece_matches_fixed_offer_candidate_and_container_census() {
    let mut piece = DesertPyramidPiece::new(pos(0, 0, 0), HorizontalDirection::South);
    let clip = BlockBox::new(pos(-100, -100, -100), pos(200, 200, 200)).unwrap();
    let mut world = World {
        terrain_height: 70,
        ..World::default()
    };
    let mut seeds = [1_i64, 2, 3, 4].into_iter();
    let mut next_seed = || seeds.next().expect("four pyramid chests");
    assert!(piece.place(
        &mut world,
        &clip,
        &mut ZeroRandom,
        &mut ZeroRandom,
        &mut next_seed,
    ));
    assert_eq!(piece.height_position, 70);
    assert_eq!(world.height_probes.len(), 21 * 21);
    assert_eq!(world.writes.len(), 6_639);
    assert_eq!(piece.archaeology_candidates.len(), 83);
    assert_eq!(piece.placed_chests, [true; 4]);
    assert_eq!(
        world
            .loot
            .iter()
            .map(|(_, table, seed)| (table.as_str(), *seed))
            .collect::<Vec<_>>(),
        [
            ("minecraft:chests/desert_pyramid", 1),
            ("minecraft:chests/desert_pyramid", 2),
            ("minecraft:chests/desert_pyramid", 3),
            ("minecraft:chests/desert_pyramid", 4),
        ]
    );
    assert!(seeds.next().is_none());
}

#[test]
fn after_place_deduplicates_global_population_and_adds_one_roof_selection() {
    let mut piece = DesertPyramidPiece::new(pos(0, 0, 0), HorizontalDirection::South);
    let clip = BlockBox::new(pos(-100, -100, -100), pos(200, 200, 200)).unwrap();
    let mut world = World {
        terrain_height: 70,
        ..World::default()
    };
    let mut no_seed = || 0;
    piece.place(
        &mut world,
        &clip,
        &mut ZeroRandom,
        &mut ZeroRandom,
        &mut no_seed,
    );
    piece
        .archaeology_candidates
        .extend(piece.archaeology_candidates.clone());
    world.writes.clear();
    place_desert_archaeology(&mut world, &[piece], &clip);
    assert_eq!(world.writes.len(), 84);
    let suspicious = world
        .writes
        .iter()
        .filter(|(_, state, _)| state.block == "minecraft:suspicious_sand")
        .count();
    assert!((6..=8).contains(&suspicious));
    assert_eq!(world.archaeology.len(), suspicious);
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
    archaeology: Vec<(BlockPos, String, i64)>,
}

impl DesertPyramidWorld for World {
    fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32 {
        self.height_probes.push((x, z));
        self.terrain_height
    }
    fn minimum_y(&self) -> i32 {
        -64
    }
    fn positional_seed(&self, position: BlockPos) -> i64 {
        i64::from(position.x) * 31 + i64::from(position.y) * 17 + i64::from(position.z)
    }
}

impl DesertArchaeologyWorld for World {
    fn is_brushable_block_entity(&mut self, position: BlockPos) -> bool {
        self.states
            .get(&position)
            .is_some_and(|state| state.block == "minecraft:suspicious_sand")
    }
    fn install_archaeology_loot(&mut self, position: BlockPos, table: &str, seed: i64) {
        self.archaeology.push((position, table.into(), seed));
    }
}

impl PieceWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:stone"))
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
        self.states
            .get(&position)
            .is_some_and(|state| state.block == "minecraft:chest")
    }
    fn install_loot(&mut self, position: BlockPos, table: &str, seed: i64) {
        self.loot.push((position, table.into(), seed));
    }
}

struct ZeroRandom;

impl GenerationRandom for ZeroRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        0
    }
    fn next_f32(&mut self) -> f32 {
        0.0
    }
    fn next_f64(&mut self) -> f64 {
        0.0
    }
    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
