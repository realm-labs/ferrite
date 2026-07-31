use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::{GenerationRandom, LegacyRandom};
use ferrite_world::generation::structure::monument_catalog::{
    MONUMENT_BIOME_RANGE, MONUMENT_BIOME_TAG, MONUMENT_MONSTERS, MONUMENT_START_BIOMES,
    MONUMENT_STEP, MONUMENT_SURROUNDING_BIOME_TAG, MONUMENT_SURROUNDING_BIOMES,
    MONUMENT_TERRAIN_ADAPTATION, MonumentSpawnOverride, OCEAN_MONUMENTS_PLACEMENT,
    OCEAN_MONUMENTS_SALT, OCEAN_MONUMENTS_SEPARATION, OCEAN_MONUMENTS_SPACING,
    OCEAN_MONUMENTS_SPREAD_TYPE, OCEAN_MONUMENTS_STRUCTURE, OCEAN_MONUMENTS_WEIGHT,
    monument_spawn_override_at,
};
use ferrite_world::generation::structure::monument_graph::{
    MonumentGraph, MonumentPieceKind, generate_monument,
};
use ferrite_world::generation::structure::monument_place::{
    MonumentElderSpawn, MonumentWorld, place_monument_child,
};
use ferrite_world::generation::structure::piece::{
    FluidState, HorizontalDirection, OrientedPiece, PieceWorld,
};
use ferrite_world::generation::structure::processor::StructureState;

#[test]
fn zero_stream_builds_connected_claimed_lattice_and_locked_child_order() {
    let mut random = ZeroRandom::default();
    let graph = generate_monument(0, 0, 17, &mut random);
    assert_eq!(graph.stub_position, pos(8, 17, 8));
    assert_eq!(graph.building.kind, MonumentPieceKind::Building);
    assert_eq!(graph.building.orientation, HorizontalDirection::North);
    assert_eq!(graph.building.bounding_box.minimum, pos(-29, 39, -29));
    assert_eq!(graph.building.bounding_box.maximum, pos(28, 61, 28));
    assert_eq!(graph.rooms.len(), 49);
    assert_eq!(graph.rooms[graph.source_room].index, 2);
    assert_eq!(graph.rooms[graph.core_room].index, 10);
    assert!(graph.rooms.iter().take(46).all(|room| room.claimed));
    assert!(source_connected(&graph));
    assert_eq!(graph.children[0].kind, MonumentPieceKind::Entry);
    assert_eq!(graph.children[1].kind, MonumentPieceKind::Core);
    assert_eq!(
        graph.children[graph.children.len() - 3..]
            .iter()
            .map(|child| child.kind)
            .collect::<Vec<_>>(),
        [
            MonumentPieceKind::Wing,
            MonumentPieceKind::Wing,
            MonumentPieceKind::Penthouse,
        ]
    );
    assert_ne!(
        graph.children[graph.children.len() - 3].design & 1,
        graph.children[graph.children.len() - 2].design & 1
    );
    assert_eq!(&random.bounds[..4], [4, 4, 46, 45]);
    assert_eq!(
        &random.bounds[2..47],
        (2..=46).rev().collect::<Vec<_>>().as_slice()
    );
}

#[test]
fn monument_piece_catalog_preserves_all_twelve_registered_ids() {
    let kinds = [
        MonumentPieceKind::Building,
        MonumentPieceKind::Entry,
        MonumentPieceKind::Core,
        MonumentPieceKind::DoubleX,
        MonumentPieceKind::DoubleXY,
        MonumentPieceKind::DoubleY,
        MonumentPieceKind::DoubleYZ,
        MonumentPieceKind::DoubleZ,
        MonumentPieceKind::Simple,
        MonumentPieceKind::SimpleTop,
        MonumentPieceKind::Wing,
        MonumentPieceKind::Penthouse,
    ];
    assert_eq!(
        kinds.map(MonumentPieceKind::id),
        [
            "minecraft:omb",
            "minecraft:omentry",
            "minecraft:omcr",
            "minecraft:omdxr",
            "minecraft:omdxyr",
            "minecraft:omdyr",
            "minecraft:omdyzr",
            "minecraft:omdzr",
            "minecraft:omsimple",
            "minecraft:omsimplet",
            "minecraft:omwr",
            "minecraft:ompenthouse",
        ]
    );
}

#[test]
fn monument_records_preserve_biome_set_and_full_box_spawn_overrides() {
    assert_eq!(
        MONUMENT_BIOME_TAG,
        "#minecraft:has_structure/ocean_monument"
    );
    assert_eq!(
        MONUMENT_SURROUNDING_BIOME_TAG,
        "#minecraft:required_ocean_monument_surrounding"
    );
    assert_eq!(MONUMENT_STEP, "surface_structures");
    assert_eq!(MONUMENT_TERRAIN_ADAPTATION, "none");
    assert_eq!(MONUMENT_BIOME_RANGE, 29);
    assert_eq!(MONUMENT_START_BIOMES.len(), 4);
    assert_eq!(MONUMENT_SURROUNDING_BIOMES.len(), 11);
    assert_eq!(OCEAN_MONUMENTS_STRUCTURE, "minecraft:monument");
    assert_eq!(OCEAN_MONUMENTS_WEIGHT, 1);
    assert_eq!(OCEAN_MONUMENTS_PLACEMENT, "random_spread");
    assert_eq!(OCEAN_MONUMENTS_SPREAD_TYPE, "triangular");
    assert_eq!(OCEAN_MONUMENTS_SPACING, 32);
    assert_eq!(OCEAN_MONUMENTS_SEPARATION, 5);
    assert_eq!(OCEAN_MONUMENTS_SALT, 10_387_313);
    assert_eq!(MONUMENT_MONSTERS[0].entity, "minecraft:guardian");
    assert_eq!(
        (
            MONUMENT_MONSTERS[0].weight,
            MONUMENT_MONSTERS[0].minimum,
            MONUMENT_MONSTERS[0].maximum
        ),
        (1, 2, 4)
    );

    let building = box_(pos(0, 39, 0), pos(57, 61, 57));
    assert_eq!(
        monument_spawn_override_at(building, "monster", pos(10, 45, 10)),
        Some(MonumentSpawnOverride::Monsters(&MONUMENT_MONSTERS))
    );
    assert_eq!(
        monument_spawn_override_at(building, "axolotls", pos(10, 45, 10)),
        Some(MonumentSpawnOverride::Empty)
    );
    assert_eq!(
        monument_spawn_override_at(building, "underground_water_creature", pos(10, 45, 10)),
        Some(MonumentSpawnOverride::Empty)
    );
    assert_eq!(
        monument_spawn_override_at(building, "water_creature", pos(10, 45, 10)),
        None
    );
    assert_eq!(
        monument_spawn_override_at(building, "monster", pos(58, 45, 10)),
        None
    );
}

#[test]
fn full_building_places_gold_three_elders_and_preserves_packed_ice() {
    let mut random = ZeroRandom::default();
    let graph = generate_monument(0, 0, 17, &mut random);
    let piece = OrientedPiece {
        bounds: graph.building.bounding_box,
        orientation: graph.building.orientation,
    };
    let preserved = piece.world_position(pos(28, 10, 10));
    let mut world = MonumentTestWorld::default();
    world
        .states
        .insert(preserved, StructureState::new("minecraft:packed_ice"));
    let clip = box_(pos(-50, -64, -50), pos(50, 100, 50));
    random.bounds.clear();
    place_monument_child(&mut world, &graph, &graph.building, &clip, &mut random);

    assert_eq!(
        world
            .states
            .values()
            .filter(|state| state.block == "minecraft:gold_block")
            .count(),
        8
    );
    assert_eq!(world.elders.len(), 3);
    assert!(world.elders.iter().all(|spawn| {
        spawn.healed_to_maximum
            && spawn.reason_structure
            && spawn.finalize_with_local_difficulty
            && spawn.include_passengers
    }));
    assert_eq!(world.states[&preserved].block, "minecraft:packed_ice");

    place_monument_child(&mut world, &graph, &graph.building, &clip, &mut random);
    assert_eq!(
        world.elders.len(),
        6,
        "elder placement deliberately has no latch"
    );
}

#[test]
fn ordinary_room_random_consumption_preserves_source_short_circuits() {
    let graph = graph_with(|child| child.kind == MonumentPieceKind::SimpleTop);
    let top = graph
        .children
        .iter()
        .find(|child| child.kind == MonumentPieceKind::SimpleTop)
        .expect("zero stream produces a simple-top room");
    let mut random = ZeroRandom::default();
    let mut world = MonumentTestWorld::default();
    place_monument_child(&mut world, &graph, top, &top.bounding_box, &mut random);
    assert_eq!(random.bounds, vec![3; 36]);

    let graph = graph_with(|child| child.kind == MonumentPieceKind::Simple && child.design == 0);
    let simple = graph
        .children
        .iter()
        .find(|child| child.kind == MonumentPieceKind::Simple && child.design == 0)
        .expect("zero stream produces a design-zero simple room");
    random.bounds.clear();
    place_monument_child(
        &mut world,
        &graph,
        simple,
        &simple.bounding_box,
        &mut random,
    );
    assert!(
        random.bounds.is_empty(),
        "design zero short-circuits before nextBoolean"
    );
}

fn graph_with(
    predicate: impl Fn(&ferrite_world::generation::structure::monument_graph::MonumentChild) -> bool,
) -> MonumentGraph {
    (0..10_000)
        .find_map(|seed| {
            let mut random = LegacyRandom::new(seed);
            let graph = generate_monument(0, 0, 17, &mut random);
            graph.children.iter().any(&predicate).then_some(graph)
        })
        .expect("search range contains the requested canonical room fitter result")
}

fn source_connected(graph: &MonumentGraph) -> bool {
    for start in 0..46 {
        let mut seen = BTreeSet::new();
        let mut pending = vec![start];
        while let Some(room) = pending.pop() {
            if !seen.insert(room) {
                continue;
            }
            if room == graph.source_room {
                break;
            }
            for direction in 0..6 {
                if graph.rooms[room].openings[direction]
                    && let Some(next) = graph.rooms[room].connections[direction]
                {
                    pending.push(next);
                }
            }
        }
        if !seen.contains(&graph.source_room) {
            return false;
        }
    }
    true
}

#[derive(Default)]
struct ZeroRandom {
    bounds: Vec<u32>,
}

impl GenerationRandom for ZeroRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
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

#[derive(Default)]
struct MonumentTestWorld {
    states: BTreeMap<BlockPos, StructureState>,
    fluid_ticks: Vec<(BlockPos, FluidState, u32)>,
    elders: Vec<MonumentElderSpawn>,
}

impl PieceWorld for MonumentTestWorld {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:water"))
    }

    fn fluid_at(&mut self, position: BlockPos) -> FluidState {
        match self.states.get(&position).map(|state| state.block.as_str()) {
            Some("minecraft:lava") => FluidState::Lava,
            Some("minecraft:air") => FluidState::Empty,
            _ => FluidState::Water,
        }
    }

    fn set_state(&mut self, position: BlockPos, state: StructureState, _flags: u32) -> bool {
        self.states.insert(position, state);
        true
    }

    fn schedule_fluid_tick(&mut self, position: BlockPos, fluid: FluidState, delay: u32) {
        self.fluid_ticks.push((position, fluid, delay));
    }

    fn mark_shape_postprocessing(&mut self, _position: BlockPos) {}

    fn solid_render(&mut self, position: BlockPos) -> bool {
        self.states
            .get(&position)
            .is_some_and(|state| state.block != "minecraft:air")
    }

    fn is_loot_container(&mut self, _position: BlockPos) -> bool {
        false
    }

    fn install_loot(&mut self, _position: BlockPos, _table: &str, _seed: i64) {}
}

impl MonumentWorld for MonumentTestWorld {
    fn sea_level(&self) -> i32 {
        63
    }

    fn minimum_y(&self) -> i32 {
        -64
    }

    fn monument_support_replaceable(
        &mut self,
        _position: BlockPos,
        state: &StructureState,
    ) -> bool {
        matches!(state.block.as_str(), "minecraft:air" | "minecraft:water")
    }

    fn spawn_elder_guardian(&mut self, request: MonumentElderSpawn) {
        self.elders.push(request);
    }
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos::new(x, y, z)
}

fn box_(minimum: BlockPos, maximum: BlockPos) -> ferrite_world::generation::structure::BlockBox {
    ferrite_world::generation::structure::BlockBox::new(minimum, maximum).unwrap()
}
