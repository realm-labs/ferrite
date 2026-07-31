use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::mineshaft_corridor::place_mineshaft_corridor;
use ferrite_world::generation::structure::mineshaft_graph::{
    MineshaftCorridor, MineshaftCrossing, MineshaftPiece, MineshaftRoom, MineshaftStairs,
    MineshaftType, generate_mineshaft,
};
use ferrite_world::generation::structure::mineshaft_place::{
    MineshaftChestCartSpawn, MineshaftFace, MineshaftWorld, place_non_corridor,
};
use ferrite_world::generation::structure::piece::{FluidState, HorizontalDirection, PieceWorld};
use ferrite_world::generation::structure::processor::StructureState;

#[test]
fn zero_stream_builds_depth_first_graph_and_moves_stub_with_normal_union() {
    let mut random = ZeroRandom::default();
    let graph = generate_mineshaft(
        0,
        0,
        MineshaftType::Normal,
        63,
        -64,
        &mut |_, _| panic!("normal relocation does not query a surface"),
        &mut random,
    );
    assert_eq!(random.double_draws, 1);
    assert!(graph.pieces.len() > 4);
    let MineshaftPiece::Room(room) = &graph.pieces[0] else {
        panic!("first piece is the root room");
    };
    assert_eq!(room.bounding_box.size(), [8, 5, 8]);
    assert!(!room.entrances.is_empty());
    assert_eq!(graph.stub_position.y, 50 + graph.vertical_offset);
    assert!(
        graph
            .pieces
            .iter()
            .all(|piece| piece.generation_depth() <= 9)
    );
}

#[test]
fn mesa_relocation_samples_union_center_and_targets_sea_level_endpoint() {
    let mut probes = Vec::new();
    let graph = generate_mineshaft(
        -2,
        3,
        MineshaftType::Mesa,
        63,
        -64,
        &mut |x, z| {
            probes.push((x, z));
            90
        },
        &mut ZeroRandom::default(),
    );
    assert_eq!(probes.len(), 1);
    let union = graph
        .pieces
        .iter()
        .map(MineshaftPiece::bounding_box)
        .reduce(|left, right| left.union(right))
        .unwrap();
    assert_eq!(union.center().y, 63);
}

#[test]
fn biome_and_boundary_liquid_cancel_piece_before_any_writes() {
    let piece = MineshaftPiece::Stairs(MineshaftStairs {
        bounding_box: BlockBox::new(pos(0, 45, 0), pos(2, 52, 8)).unwrap(),
        generation_depth: 1,
        orientation: HorizontalDirection::South,
    });
    let clip = BlockBox::new(pos(-20, -100, -20), pos(20, 200, 20)).unwrap();
    let mut biome_world = PlacementWorld {
        blocking_biome: true,
        ..PlacementWorld::default()
    };
    assert!(!place_non_corridor(
        &mut biome_world,
        &piece,
        MineshaftType::Normal,
        &clip,
        &mut ZeroRandom::default(),
    ));
    assert_eq!(biome_world.biome_probes, [pos(1, 48, 4)]);
    assert!(biome_world.writes.is_empty());

    let mut liquid_world = PlacementWorld::default();
    liquid_world
        .fluids
        .insert(pos(-1, 44, 3), FluidState::Water);
    assert!(!place_non_corridor(
        &mut liquid_world,
        &piece,
        MineshaftType::Normal,
        &clip,
        &mut ZeroRandom::default(),
    ));
    assert!(liquid_world.writes.is_empty());
}

#[test]
fn room_uses_vanilla_integer_centered_upper_sphere_and_palette_guard() {
    let piece = MineshaftPiece::Room(MineshaftRoom {
        bounding_box: BlockBox::new(pos(0, 50, 0), pos(7, 54, 7)).unwrap(),
        generation_depth: 0,
        entrances: Vec::new(),
    });
    let clip = BlockBox::new(pos(-20, -100, -20), pos(20, 200, 20)).unwrap();
    let mut world = PlacementWorld::default();
    world
        .states
        .insert(pos(0, 51, 0), StructureState::new("minecraft:bedrock"));
    world.generic_non_replaceable.insert(pos(0, 51, 0));
    world
        .states
        .insert(pos(1, 51, 0), StructureState::new("minecraft:oak_planks"));
    assert!(place_non_corridor(
        &mut world,
        &piece,
        MineshaftType::Normal,
        &clip,
        &mut ZeroRandom::default(),
    ));
    assert_eq!(world.states[&pos(0, 51, 0)].block, "minecraft:cave_air");
    assert_eq!(world.states[&pos(1, 51, 0)].block, "minecraft:oak_planks");
    assert_eq!(world.states[&pos(0, 54, 4)].block, "minecraft:cave_air");
    assert_eq!(world.states[&pos(4, 54, 0)].block, "minecraft:cave_air");
    assert!(!world.states.contains_key(&pos(0, 54, 3)));
}

#[test]
fn two_floor_crossing_and_stairs_preserve_exact_carve_geometry() {
    let clip = BlockBox::new(pos(-20, -100, -20), pos(20, 200, 20)).unwrap();
    let crossing = MineshaftPiece::Crossing(MineshaftCrossing {
        bounding_box: BlockBox::new(pos(0, 50, 0), pos(4, 56, 4)).unwrap(),
        generation_depth: 1,
        direction: HorizontalDirection::South,
        two_floored: true,
    });
    let mut crossing_world = PlacementWorld::default();
    assert!(place_non_corridor(
        &mut crossing_world,
        &crossing,
        MineshaftType::Mesa,
        &clip,
        &mut ZeroRandom::default(),
    ));
    assert!(!crossing_world.states.contains_key(&pos(0, 50, 0)));
    assert_eq!(
        crossing_world.states[&pos(2, 50, 0)].block,
        "minecraft:cave_air"
    );
    assert_eq!(
        crossing_world.states[&pos(0, 50, 2)].block,
        "minecraft:cave_air"
    );
    assert_eq!(
        crossing_world.states[&pos(2, 53, 2)].block,
        "minecraft:cave_air"
    );
    assert_eq!(
        crossing_world.states[&pos(1, 56, 1)].block,
        "minecraft:dark_oak_planks"
    );
    assert_eq!(
        crossing_world.states[&pos(0, 49, 0)].block,
        "minecraft:dark_oak_planks"
    );

    let stairs = MineshaftPiece::Stairs(MineshaftStairs {
        bounding_box: BlockBox::new(pos(0, 45, 0), pos(2, 52, 8)).unwrap(),
        generation_depth: 1,
        orientation: HorizontalDirection::South,
    });
    let mut stairs_world = PlacementWorld::default();
    assert!(place_non_corridor(
        &mut stairs_world,
        &stairs,
        MineshaftType::Normal,
        &clip,
        &mut ZeroRandom::default(),
    ));
    assert_eq!(stairs_world.writes.len(), 93);
    assert_eq!(
        stairs_world.states[&pos(0, 50, 0)].block,
        "minecraft:cave_air"
    );
    assert_eq!(
        stairs_world.states[&pos(2, 45, 8)].block,
        "minecraft:cave_air"
    );
    assert_eq!(
        stairs_world.states[&pos(1, 46, 6)].block,
        "minecraft:cave_air"
    );
    assert!(!stairs_world.states.contains_key(&pos(1, 45, 6)));
}

#[test]
fn two_section_corridor_preserves_exact_support_cobweb_cart_and_floor_order() {
    let mut corridor = MineshaftCorridor {
        bounding_box: BlockBox::new(pos(0, 50, 0), pos(2, 52, 9)).unwrap(),
        generation_depth: 1,
        orientation: HorizontalDirection::South,
        has_rails: false,
        spider_corridor: false,
        has_placed_spider: false,
        sections: 2,
    };
    let clip = BlockBox::new(pos(-10, -100, -10), pos(20, 200, 20)).unwrap();
    let mut world = PlacementWorld::default();
    let mut random = TraceRandom::default();
    let mut seeds = [10_i64, 20, 30, 40].into_iter();
    let mut next_seed = || seeds.next().expect("four cart seeds");

    assert!(place_mineshaft_corridor(
        &mut world,
        &mut corridor,
        MineshaftType::Normal,
        &clip,
        &mut random,
        &mut next_seed,
    ));
    assert_eq!(random.float_draws, 46);
    assert_eq!(random.integer_draws, 10);
    assert_eq!(world.writes.len(), 152);
    assert_eq!(world.carts.len(), 4);
    assert_eq!(
        world
            .carts
            .iter()
            .map(|cart| cart.loot_seed)
            .collect::<Vec<_>>(),
        [10, 20, 30, 40]
    );
    assert!(
        world
            .carts
            .iter()
            .all(|cart| cart.creation_reason_chunk_generation
                && cart.loot_table == "minecraft:chests/abandoned_mineshaft")
    );
    assert!(seeds.next().is_none());
}

#[test]
fn spider_corridor_latches_before_typed_spawner_configuration() {
    let mut corridor = MineshaftCorridor {
        bounding_box: BlockBox::new(pos(0, 50, 0), pos(2, 52, 4)).unwrap(),
        generation_depth: 1,
        orientation: HorizontalDirection::South,
        has_rails: false,
        spider_corridor: true,
        has_placed_spider: false,
        sections: 1,
    };
    let clip = BlockBox::new(pos(-10, -100, -10), pos(20, 200, 20)).unwrap();
    let mut world = PlacementWorld {
        spawner_entity: true,
        ..PlacementWorld::default()
    };
    let mut seeds = [1_i64, 2].into_iter();
    assert!(place_mineshaft_corridor(
        &mut world,
        &mut corridor,
        MineshaftType::Mesa,
        &clip,
        &mut TraceRandom::default(),
        &mut || seeds.next().unwrap(),
    ));
    assert!(corridor.has_placed_spider);
    assert_eq!(world.configured_spawners.len(), 1);
}

#[test]
fn corridor_support_search_accepts_last_vanilla_lower_and_upper_probes() {
    let clip = BlockBox::new(pos(-10, -100, -10), pos(20, 200, 20)).unwrap();
    let corridor = || MineshaftCorridor {
        bounding_box: BlockBox::new(pos(0, 50, 0), pos(2, 52, 4)).unwrap(),
        generation_depth: 1,
        orientation: HorizontalDirection::South,
        has_rails: false,
        spider_corridor: false,
        has_placed_spider: false,
        sections: 1,
    };

    let lower_support = pos(0, 28, 2);
    let mut lower_world = PlacementWorld::default();
    lower_world.generic_non_replaceable.insert(lower_support);
    lower_world.sturdy_tops.insert(lower_support);
    assert!(place_mineshaft_corridor(
        &mut lower_world,
        &mut corridor(),
        MineshaftType::Normal,
        &clip,
        &mut TraceRandom::default(),
        &mut || 1,
    ));
    assert_eq!(
        lower_world
            .states
            .iter()
            .filter(|(_, state)| state.block == "minecraft:oak_log")
            .count(),
        20
    );
    assert_eq!(
        lower_world.states[&pos(0, 29, 2)].block,
        "minecraft:oak_log"
    );
    assert_eq!(
        lower_world.states[&pos(0, 48, 2)].block,
        "minecraft:oak_log"
    );

    let upper_support = pos(0, 100, 2);
    let mut upper_world = PlacementWorld::default();
    upper_world.generic_non_replaceable.insert(upper_support);
    upper_world.center_down.insert(upper_support);
    assert!(place_mineshaft_corridor(
        &mut upper_world,
        &mut corridor(),
        MineshaftType::Mesa,
        &clip,
        &mut TraceRandom::default(),
        &mut || 1,
    ));
    assert_eq!(
        upper_world
            .states
            .iter()
            .filter(|(_, state)| state.block == "minecraft:chain")
            .count(),
        49
    );
    assert_eq!(upper_world.states[&pos(0, 51, 2)].block, "minecraft:chain");
    assert_eq!(upper_world.states[&pos(0, 99, 2)].block, "minecraft:chain");
}

#[test]
fn corridor_support_search_does_not_probe_past_vanilla_limits() {
    let clip = BlockBox::new(pos(-10, -100, -10), pos(20, 200, 20)).unwrap();
    let corridor = || MineshaftCorridor {
        bounding_box: BlockBox::new(pos(0, 50, 0), pos(2, 52, 4)).unwrap(),
        generation_depth: 1,
        orientation: HorizontalDirection::South,
        has_rails: false,
        spider_corridor: false,
        has_placed_spider: false,
        sections: 1,
    };
    let lower_support = pos(0, 27, 2);
    let upper_support = pos(0, 101, 2);
    let mut world = PlacementWorld::default();
    world.generic_non_replaceable.insert(lower_support);
    world.generic_non_replaceable.insert(upper_support);
    world.sturdy_tops.insert(lower_support);
    world.center_down.insert(upper_support);
    assert!(place_mineshaft_corridor(
        &mut world,
        &mut corridor(),
        MineshaftType::Normal,
        &clip,
        &mut TraceRandom::default(),
        &mut || 1,
    ));
    assert!(
        world
            .states
            .values()
            .all(|state| state.block != "minecraft:oak_log" && state.block != "minecraft:chain")
    );
}

#[derive(Default)]
struct ZeroRandom {
    double_draws: usize,
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct PlacementWorld {
    states: BTreeMap<BlockPos, StructureState>,
    fluids: BTreeMap<BlockPos, FluidState>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    carts: Vec<MineshaftChestCartSpawn>,
    configured_spawners: Vec<BlockPos>,
    biome_probes: Vec<BlockPos>,
    generic_non_replaceable: BTreeSet<BlockPos>,
    sturdy_tops: BTreeSet<BlockPos>,
    center_down: BTreeSet<BlockPos>,
    falling: BTreeSet<BlockPos>,
    blocking_biome: bool,
    spawner_entity: bool,
}

impl PieceWorld for PlacementWorld {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:stone"))
    }
    fn fluid_at(&mut self, position: BlockPos) -> FluidState {
        self.fluids
            .get(&position)
            .copied()
            .unwrap_or(FluidState::Empty)
    }
    fn set_state(&mut self, position: BlockPos, state: StructureState, flags: u32) -> bool {
        self.writes.push((position, state.clone(), flags));
        self.states.insert(position, state);
        true
    }
    fn schedule_fluid_tick(&mut self, _position: BlockPos, _fluid: FluidState, _delay: u32) {}
    fn mark_shape_postprocessing(&mut self, _position: BlockPos) {}
    fn solid_render(&mut self, position: BlockPos) -> bool {
        self.states
            .get(&position)
            .is_none_or(|state| state.block != "minecraft:air")
    }
    fn is_loot_container(&mut self, _position: BlockPos) -> bool {
        false
    }
    fn install_loot(&mut self, _position: BlockPos, _table: &str, _seed: i64) {}
}

impl MineshaftWorld for PlacementWorld {
    fn mineshaft_blocking_biome(&mut self, position: BlockPos) -> bool {
        self.biome_probes.push(position);
        self.blocking_biome
    }
    fn ocean_floor_height(&mut self, _x: i32, _z: i32) -> i32 {
        100
    }
    fn structure_replaceable(&mut self, position: BlockPos, _state: &StructureState) -> bool {
        !self.generic_non_replaceable.contains(&position)
    }
    fn sturdy_top(&mut self, position: BlockPos) -> bool {
        self.sturdy_tops.contains(&position)
    }
    fn sturdy_face(&mut self, _position: BlockPos, _face: MineshaftFace) -> bool {
        true
    }
    fn supports_center_down(&mut self, position: BlockPos) -> bool {
        self.center_down.contains(&position)
    }
    fn falling_block(&mut self, position: BlockPos) -> bool {
        self.falling.contains(&position)
    }
    fn minimum_y(&self) -> i32 {
        -64
    }
    fn maximum_y(&self) -> i32 {
        320
    }
    fn create_mineshaft_chest_cart(&mut self, _position: [f64; 3]) -> bool {
        true
    }
    fn spawn_mineshaft_chest_cart(&mut self, request: MineshaftChestCartSpawn) {
        self.carts.push(request);
    }
    fn is_spawner_block_entity(&mut self, _position: BlockPos) -> bool {
        self.spawner_entity
    }
    fn configure_cave_spider_spawner(&mut self, position: BlockPos) {
        self.configured_spawners.push(position);
    }
}

#[derive(Default)]
struct TraceRandom {
    integer_draws: usize,
    float_draws: usize,
}

impl GenerationRandom for TraceRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        self.integer_draws += 1;
        0
    }
    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        0.0
    }
    fn next_f64(&mut self) -> f64 {
        0.0
    }
    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}

impl GenerationRandom for ZeroRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        0
    }
    fn next_f32(&mut self) -> f32 {
        0.0
    }
    fn next_f64(&mut self) -> f64 {
        self.double_draws += 1;
        0.0
    }
    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
