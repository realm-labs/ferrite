use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::piece::{FluidState, HorizontalDirection, PieceWorld};
use ferrite_world::generation::structure::processor::StructureState;
use ferrite_world::generation::structure::stronghold_catalog::{
    STRONGHOLD_BIOME_TAG, STRONGHOLD_BIOMES, STRONGHOLD_CORRIDOR_LOOT,
    STRONGHOLD_CORRIDOR_LOOT_TABLE, STRONGHOLD_CORRIDOR_ROLLS, STRONGHOLD_CROSSING_LOOT,
    STRONGHOLD_CROSSING_LOOT_TABLE, STRONGHOLD_CROSSING_ROLLS, STRONGHOLD_LIBRARY_LOOT,
    STRONGHOLD_LIBRARY_LOOT_TABLE, STRONGHOLD_LIBRARY_ROLLS, STRONGHOLD_PREFERRED_BIOME_TAG,
    STRONGHOLD_PREFERRED_BIOMES, STRONGHOLD_STEP, STRONGHOLD_TERRAIN_ADAPTATION, STRONGHOLDS_COUNT,
    STRONGHOLDS_DISTANCE, STRONGHOLDS_SALT, STRONGHOLDS_SPREAD,
};
use ferrite_world::generation::structure::stronghold_graph::{
    StrongholdDoor, StrongholdPiece, StrongholdPieceKind, generate_stronghold,
};
use ferrite_world::generation::structure::stronghold_place::{
    StrongholdWorld, place_stronghold_piece,
};

#[test]
fn fixed_seed_retries_until_portal_and_relocates_below_sea() {
    let graph = generate_stronghold(0, 0, 0, 63, -64);
    assert_eq!(graph.stub_position, BlockPos::new(0, 0, 0));
    assert!(!graph.pieces.is_empty());
    assert_eq!(graph.pieces[0].kind, StrongholdPieceKind::Start);
    assert_eq!(
        graph.pieces[graph.portal_room].kind,
        StrongholdPieceKind::PortalRoom
    );
    let union = graph
        .pieces
        .iter()
        .map(|piece| piece.bounding_box)
        .reduce(|left, right| left.union(right))
        .unwrap();
    assert!(union.maximum.y <= 52);
    assert_eq!(
        graph.locator_position,
        graph.pieces[graph.portal_room].bounding_box.center()
    );
    assert!(
        graph
            .pieces
            .iter()
            .all(|piece| piece.generation_depth <= 51)
    );
}

#[test]
fn stronghold_piece_catalog_preserves_all_thirteen_ids() {
    let kinds = [
        StrongholdPieceKind::Start,
        StrongholdPieceKind::Straight,
        StrongholdPieceKind::PrisonHall,
        StrongholdPieceKind::LeftTurn,
        StrongholdPieceKind::RightTurn,
        StrongholdPieceKind::RoomCrossing,
        StrongholdPieceKind::StraightStairsDown,
        StrongholdPieceKind::StairsDown,
        StrongholdPieceKind::FiveCrossing,
        StrongholdPieceKind::ChestCorridor,
        StrongholdPieceKind::Library,
        StrongholdPieceKind::PortalRoom,
        StrongholdPieceKind::FillerCorridor,
    ];
    assert_eq!(
        kinds.map(StrongholdPieceKind::id),
        [
            "minecraft:shstart",
            "minecraft:shs",
            "minecraft:shph",
            "minecraft:shlt",
            "minecraft:shrt",
            "minecraft:shrc",
            "minecraft:shssd",
            "minecraft:shsd",
            "minecraft:sh5c",
            "minecraft:shcc",
            "minecraft:shli",
            "minecraft:shpr",
            "minecraft:shfc",
        ]
    );
}

#[test]
fn stronghold_records_preserve_rings_biomes_and_three_loot_families() {
    assert_eq!(STRONGHOLD_BIOME_TAG, "#minecraft:has_structure/stronghold");
    assert_eq!(
        STRONGHOLD_PREFERRED_BIOME_TAG,
        "#minecraft:stronghold_biased_to"
    );
    assert_eq!(STRONGHOLD_STEP, "surface_structures");
    assert_eq!(STRONGHOLD_TERRAIN_ADAPTATION, "bury");
    assert_eq!(STRONGHOLD_BIOMES.len(), 55);
    assert_eq!(STRONGHOLD_PREFERRED_BIOMES.len(), 38);
    assert_eq!(
        (
            STRONGHOLDS_DISTANCE,
            STRONGHOLDS_SPREAD,
            STRONGHOLDS_COUNT,
            STRONGHOLDS_SALT
        ),
        (32, 3, 128, 0)
    );
    assert_eq!(
        STRONGHOLD_CORRIDOR_LOOT_TABLE,
        "minecraft:chests/stronghold_corridor"
    );
    assert_eq!(
        STRONGHOLD_CROSSING_LOOT_TABLE,
        "minecraft:chests/stronghold_crossing"
    );
    assert_eq!(
        STRONGHOLD_LIBRARY_LOOT_TABLE,
        "minecraft:chests/stronghold_library"
    );
    assert_eq!(STRONGHOLD_CORRIDOR_ROLLS, (2, 3));
    assert_eq!(STRONGHOLD_CROSSING_ROLLS, (1, 4));
    assert_eq!(STRONGHOLD_LIBRARY_ROLLS, (2, 10));
    assert_eq!(STRONGHOLD_CORRIDOR_LOOT.len(), 21);
    assert_eq!(STRONGHOLD_CROSSING_LOOT.len(), 8);
    assert_eq!(STRONGHOLD_LIBRARY_LOOT.len(), 5);
    assert_eq!(
        STRONGHOLD_LIBRARY_LOOT[4].function,
        Some("minecraft:enchant_with_levels:30:#minecraft:on_random_loot")
    );
}

#[test]
fn portal_redraws_all_eyes_and_latches_the_silverfish_spawner() {
    let mut piece = piece(StrongholdPieceKind::PortalRoom, [11, 8, 16]);
    piece.entry_door = StrongholdDoor::Grates;
    let clip = piece.bounding_box;
    let mut world = StrongholdTestWorld {
        spawner_entity: true,
        ..StrongholdTestWorld::default()
    };
    let mut random = OneRandom::default();
    place_stronghold_piece(&mut world, &mut piece, &clip, &mut random);

    assert_eq!(random.float_draws, 772);
    assert_eq!(block_count(&world, "minecraft:end_portal_frame"), 12);
    assert_eq!(block_count(&world, "minecraft:end_portal"), 9);
    assert_eq!(world.configured_spawners.len(), 1);
    assert!(piece.spawner_placed);

    place_stronghold_piece(&mut world, &mut piece, &clip, &mut random);
    assert_eq!(
        random.float_draws, 1_544,
        "all twelve eyes redraw on repeat"
    );
    assert_eq!(
        world.configured_spawners.len(),
        1,
        "spawner latch survives repeat"
    );
}

#[test]
fn chest_corridor_latches_after_admitted_write_without_block_entity() {
    let mut piece = piece(StrongholdPieceKind::ChestCorridor, [5, 5, 7]);
    piece.chest_pending = true;
    let clip = piece.bounding_box;
    let mut world = StrongholdTestWorld::default();
    let mut random = OneRandom::default();
    place_stronghold_piece(&mut world, &mut piece, &clip, &mut random);
    assert!(!piece.chest_pending);
    assert_eq!(block_count(&world, "minecraft:chest"), 1);
    assert!(world.loot.is_empty());
}

#[derive(Default)]
struct OneRandom {
    float_draws: usize,
}

impl GenerationRandom for OneRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        bound.get() - 1
    }

    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        1.0
    }

    fn next_f64(&mut self) -> f64 {
        1.0
    }

    fn next_gaussian(&mut self) -> f64 {
        1.0
    }
}

#[derive(Default)]
struct StrongholdTestWorld {
    states: BTreeMap<BlockPos, StructureState>,
    fluid_ticks: Vec<(BlockPos, FluidState, u32)>,
    postprocessing: Vec<BlockPos>,
    loot: Vec<(BlockPos, String, i64)>,
    configured_spawners: Vec<BlockPos>,
    container_entity: bool,
    spawner_entity: bool,
}

impl PieceWorld for StrongholdTestWorld {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:stone"))
    }

    fn fluid_at(&mut self, position: BlockPos) -> FluidState {
        match self.states.get(&position).map(|state| state.block.as_str()) {
            Some("minecraft:lava") => FluidState::Lava,
            Some("minecraft:water") => FluidState::Water,
            _ => FluidState::Empty,
        }
    }

    fn set_state(&mut self, position: BlockPos, state: StructureState, _flags: u32) -> bool {
        self.states.insert(position, state);
        true
    }

    fn schedule_fluid_tick(&mut self, position: BlockPos, fluid: FluidState, delay: u32) {
        self.fluid_ticks.push((position, fluid, delay));
    }

    fn mark_shape_postprocessing(&mut self, position: BlockPos) {
        self.postprocessing.push(position);
    }

    fn solid_render(&mut self, position: BlockPos) -> bool {
        self.states.get(&position).is_some_and(|state| {
            !matches!(state.block.as_str(), "minecraft:air" | "minecraft:cave_air")
        })
    }

    fn is_loot_container(&mut self, position: BlockPos) -> bool {
        self.container_entity
            && self
                .states
                .get(&position)
                .is_some_and(|state| state.block == "minecraft:chest")
    }

    fn install_loot(&mut self, position: BlockPos, table: &str, seed: i64) {
        self.loot.push((position, table.to_owned(), seed));
    }
}

impl StrongholdWorld for StrongholdTestWorld {
    fn is_silverfish_spawner_block_entity(&mut self, _position: BlockPos) -> bool {
        self.spawner_entity
    }

    fn configure_silverfish_spawner(
        &mut self,
        position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) {
        self.configured_spawners.push(position);
    }
}

fn piece(kind: StrongholdPieceKind, size: [i32; 3]) -> StrongholdPiece {
    StrongholdPiece {
        kind,
        bounding_box: BlockBox::new(
            BlockPos::new(0, 20, 0),
            BlockPos::new(size[0] - 1, 20 + size[1] - 1, size[2] - 1),
        )
        .unwrap(),
        generation_depth: 1,
        orientation: HorizontalDirection::South,
        entry_door: StrongholdDoor::Opening,
        source: false,
        left_child: false,
        right_child: false,
        room_type: 0,
        low_left: false,
        high_left: false,
        low_right: false,
        high_right: false,
        chest_pending: false,
        tall_library: false,
        spawner_placed: false,
        filler_steps: 0,
    }
}

fn block_count(world: &StrongholdTestWorld, block: &str) -> usize {
    world
        .states
        .values()
        .filter(|state| state.block == block)
        .count()
}
