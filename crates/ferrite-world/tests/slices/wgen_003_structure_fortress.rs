use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::fortress_catalog::{
    FORTRESS_BIOME_TAG, FORTRESS_BIOMES, FORTRESS_LOOT_TABLE, FORTRESS_MONSTERS,
    FORTRESS_PRIMARY_LOOT, FORTRESS_PRIMARY_LOOT_ROLLS, FORTRESS_STEP, FORTRESS_TERRAIN_ADAPTATION,
    FORTRESS_TRIM_EMPTY_WEIGHT, FORTRESS_TRIM_TEMPLATE, FORTRESS_TRIM_TEMPLATE_WEIGHT,
    NETHER_COMPLEXES, NETHER_COMPLEXES_SALT, NETHER_COMPLEXES_SEPARATION, NETHER_COMPLEXES_SPACING,
    fortress_monster_spawns_at,
};
use ferrite_world::generation::structure::fortress_graph::{
    FortressPiece, FortressPieceKind, generate_fortress,
};
use ferrite_world::generation::structure::fortress_place::{FortressWorld, place_fortress_piece};
use ferrite_world::generation::structure::piece::{
    FluidState, HorizontalDirection, OrientedPiece, PieceWorld,
};
use ferrite_world::generation::structure::processor::StructureState;

#[test]
fn zero_stream_walks_three_bridge_frontiers_to_the_range_boundary() {
    let mut random = ZeroRandom::default();
    let graph = generate_fortress(0, 0, &mut random);
    assert_eq!(graph.stub_position, pos(0, 64, 0));
    assert_eq!(graph.vertical_offset, -16);
    assert_eq!(graph.pieces.len(), 18);
    assert_eq!(graph.pieces[0].kind, FortressPieceKind::Start);
    assert_eq!(graph.pieces[0].orientation, HorizontalDirection::North);
    assert_eq!(
        graph.pieces[0].bounding_box,
        box_(pos(2, 48, 2), pos(20, 57, 20))
    );
    assert_eq!(
        graph.pieces[1].bounding_box,
        box_(pos(9, 48, -17), pos(13, 57, 1))
    );
    assert_eq!(
        graph.pieces[2].bounding_box,
        box_(pos(-17, 48, 9), pos(1, 57, 13))
    );
    assert_eq!(
        graph.pieces[3].bounding_box,
        box_(pos(21, 48, 9), pos(39, 57, 13))
    );
    assert!(
        graph
            .pieces
            .iter()
            .skip(1)
            .all(|piece| piece.kind == FortressPieceKind::BridgeStraight)
    );
    assert!(
        graph
            .pieces
            .iter()
            .all(|piece| piece.bounding_box.minimum.y == 48 && piece.bounding_box.maximum.y == 57)
    );
    assert_eq!(random.bounds.first(), Some(&4));
    assert_eq!(
        random
            .bounds
            .iter()
            .filter(|bound| **bound == 65_536)
            .count(),
        6
    );

    let mut weighted = ScriptRandom::new([0, 65, 60, 0, 0, 40, 0]);
    let graph = generate_fortress(0, 0, &mut weighted);
    assert_eq!(graph.pieces[1].kind, FortressPieceKind::CastleEntrance);
    assert_eq!(graph.pieces[2].kind, FortressPieceKind::MonsterThrone);
    assert_eq!(graph.pieces[3].kind, FortressPieceKind::BridgeStraight);
    assert_eq!(graph.pieces[4].kind, FortressPieceKind::CastleRightTurn);
    assert!(graph.pieces[4].chest_pending);
    assert_eq!(&weighted.bounds[..7], [4, 70, 65, 65, 3, 72, 3]);
}

#[test]
fn fortress_piece_catalog_preserves_all_locked_ids() {
    let kinds = [
        FortressPieceKind::Start,
        FortressPieceKind::BridgeStraight,
        FortressPieceKind::BridgeCrossing,
        FortressPieceKind::RoomCrossing,
        FortressPieceKind::StairsRoom,
        FortressPieceKind::MonsterThrone,
        FortressPieceKind::CastleEntrance,
        FortressPieceKind::CastleSmallCorridor,
        FortressPieceKind::CastleSmallCrossing,
        FortressPieceKind::CastleRightTurn,
        FortressPieceKind::CastleLeftTurn,
        FortressPieceKind::CastleCorridorStairs,
        FortressPieceKind::CastleTBalcony,
        FortressPieceKind::CastleStalkRoom,
        FortressPieceKind::BridgeEndFiller,
    ];
    assert_eq!(
        kinds.map(FortressPieceKind::id),
        [
            "minecraft:nestart",
            "minecraft:nebs",
            "minecraft:nebcr",
            "minecraft:nerc",
            "minecraft:nesr",
            "minecraft:nemt",
            "minecraft:nece",
            "minecraft:nesc",
            "minecraft:nescsc",
            "minecraft:nescrt",
            "minecraft:nesclt",
            "minecraft:neccs",
            "minecraft:nectb",
            "minecraft:necsr",
            "minecraft:nebef",
        ]
    );
}

#[test]
fn fortress_records_preserve_biomes_set_piece_spawn_override_and_loot_link() {
    assert_eq!(
        FORTRESS_BIOME_TAG,
        "#minecraft:has_structure/nether_fortress"
    );
    assert_eq!(FORTRESS_STEP, "underground_decoration");
    assert_eq!(FORTRESS_TERRAIN_ADAPTATION, "none");
    assert_eq!(
        FORTRESS_BIOMES,
        [
            "minecraft:nether_wastes",
            "minecraft:soul_sand_valley",
            "minecraft:crimson_forest",
            "minecraft:warped_forest",
            "minecraft:basalt_deltas",
        ]
    );
    assert_eq!(
        (
            NETHER_COMPLEXES_SPACING,
            NETHER_COMPLEXES_SEPARATION,
            NETHER_COMPLEXES_SALT
        ),
        (27, 4, 30_084_232)
    );
    assert_eq!(
        NETHER_COMPLEXES.map(|entry| (entry.structure, entry.weight)),
        [("minecraft:fortress", 2), ("minecraft:bastion_remnant", 3)]
    );
    assert_eq!(
        FORTRESS_MONSTERS
            .map(|entry| { (entry.entity, entry.weight, entry.minimum, entry.maximum) }),
        [
            ("minecraft:blaze", 10, 2, 3),
            ("minecraft:zombified_piglin", 5, 4, 4),
            ("minecraft:wither_skeleton", 8, 5, 5),
            ("minecraft:skeleton", 2, 5, 5),
            ("minecraft:magma_cube", 3, 4, 4),
        ]
    );
    let piece = piece(FortressPieceKind::BridgeStraight);
    assert_eq!(
        fortress_monster_spawns_at(std::slice::from_ref(&piece), pos(2, 45, 2)),
        Some(FORTRESS_MONSTERS.as_slice())
    );
    assert_eq!(
        fortress_monster_spawns_at(std::slice::from_ref(&piece), pos(5, 45, 2)),
        None
    );
    assert_eq!(FORTRESS_LOOT_TABLE, "minecraft:chests/nether_bridge");
    assert_eq!(FORTRESS_PRIMARY_LOOT_ROLLS, (2, 4));
    assert_eq!(FORTRESS_PRIMARY_LOOT.len(), 13);
    assert_eq!(FORTRESS_PRIMARY_LOOT[0].item, "minecraft:diamond");
    assert_eq!(
        FORTRESS_PRIMARY_LOOT[12],
        ferrite_world::generation::structure::fortress_catalog::FortressLootEntry {
            item: "minecraft:obsidian",
            weight: 2,
            minimum: 2,
            maximum: 4,
        }
    );
    assert_eq!(FORTRESS_TRIM_EMPTY_WEIGHT, 14);
    assert_eq!(
        FORTRESS_TRIM_TEMPLATE,
        "minecraft:rib_armor_trim_smithing_template"
    );
    assert_eq!(FORTRESS_TRIM_TEMPLATE_WEIGHT, 1);
}

#[test]
fn all_fifteen_piece_families_place_without_consuming_the_caller_stream() {
    let kinds = [
        FortressPieceKind::Start,
        FortressPieceKind::BridgeStraight,
        FortressPieceKind::BridgeCrossing,
        FortressPieceKind::RoomCrossing,
        FortressPieceKind::StairsRoom,
        FortressPieceKind::MonsterThrone,
        FortressPieceKind::CastleEntrance,
        FortressPieceKind::CastleSmallCorridor,
        FortressPieceKind::CastleSmallCrossing,
        FortressPieceKind::CastleRightTurn,
        FortressPieceKind::CastleLeftTurn,
        FortressPieceKind::CastleCorridorStairs,
        FortressPieceKind::CastleTBalcony,
        FortressPieceKind::CastleStalkRoom,
        FortressPieceKind::BridgeEndFiller,
    ];
    let clip = box_(pos(-100, -100, -100), pos(200, 200, 200));
    let mut random = CountingRandom::default();
    for kind in kinds {
        let mut piece = piece(kind);
        let mut world = FortressTestWorld::default();
        place_fortress_piece(&mut world, &mut piece, &clip, &mut random, &mut || 17);
        assert!(
            !world.writes.is_empty(),
            "{} emitted no geometry",
            kind.id()
        );
    }
    assert_eq!(random.draws, 0);
}

#[test]
fn throne_commits_spawner_latch_before_typed_block_entity_configuration() {
    let clip = box_(pos(-20, -20, -20), pos(40, 100, 40));
    let mut wrong_piece = piece(FortressPieceKind::MonsterThrone);
    let mut wrong_entity = FortressTestWorld::default();
    place_fortress_piece(
        &mut wrong_entity,
        &mut wrong_piece,
        &clip,
        &mut CountingRandom::default(),
        &mut || 1,
    );
    assert!(wrong_piece.spawner_placed);
    assert!(wrong_entity.configured_spawners.is_empty());
    assert_eq!(
        wrong_entity.states[&pos(3, 45, 5)].block,
        "minecraft:spawner"
    );

    let mut typed_piece = piece(FortressPieceKind::MonsterThrone);
    let mut typed_entity = FortressTestWorld {
        spawner_entity: true,
        ..FortressTestWorld::default()
    };
    place_fortress_piece(
        &mut typed_entity,
        &mut typed_piece,
        &clip,
        &mut CountingRandom::default(),
        &mut || 1,
    );
    assert_eq!(typed_entity.configured_spawners, [pos(3, 45, 5)]);
}

#[test]
fn turn_chest_latch_only_consumes_seed_for_a_resulting_container() {
    let clip = box_(pos(-20, -20, -20), pos(40, 100, 40));
    let mut wrong_piece = piece(FortressPieceKind::CastleRightTurn);
    wrong_piece.chest_pending = true;
    let mut wrong_entity = FortressTestWorld::default();
    let mut wrong_seed_calls = 0;
    place_fortress_piece(
        &mut wrong_entity,
        &mut wrong_piece,
        &clip,
        &mut CountingRandom::default(),
        &mut || {
            wrong_seed_calls += 1;
            11
        },
    );
    assert!(!wrong_piece.chest_pending);
    assert_eq!(wrong_seed_calls, 0);
    assert!(wrong_entity.loot.is_empty());

    let mut typed_piece = piece(FortressPieceKind::CastleLeftTurn);
    typed_piece.chest_pending = true;
    let mut typed_entity = FortressTestWorld {
        container_entity: true,
        ..FortressTestWorld::default()
    };
    let mut seeds = [91_i64].into_iter();
    place_fortress_piece(
        &mut typed_entity,
        &mut typed_piece,
        &clip,
        &mut CountingRandom::default(),
        &mut || seeds.next().unwrap(),
    );
    assert_eq!(
        typed_entity.loot,
        [(
            pos(3, 42, 3),
            "minecraft:chests/nether_bridge".to_owned(),
            91
        )]
    );
    assert!(seeds.next().is_none());
}

#[test]
fn entrance_schedules_explicit_lava_tick_even_when_the_offer_is_rejected() {
    let clip = box_(pos(-20, -20, -20), pos(40, 100, 40));
    let target = pos(6, 45, 6);
    let mut rejected = FortressTestWorld::default();
    rejected.rejected_writes.insert(target);
    place_fortress_piece(
        &mut rejected,
        &mut piece(FortressPieceKind::CastleEntrance),
        &clip,
        &mut CountingRandom::default(),
        &mut || 1,
    );
    assert_eq!(rejected.fluid_ticks, [(target, FluidState::Lava, 0)]);

    let mut accepted = FortressTestWorld::default();
    place_fortress_piece(
        &mut accepted,
        &mut piece(FortressPieceKind::CastleEntrance),
        &clip,
        &mut CountingRandom::default(),
        &mut || 1,
    );
    assert_eq!(
        accepted
            .fluid_ticks
            .iter()
            .filter(|(position, fluid, _)| *position == target && *fluid == FluidState::Lava)
            .count(),
        2
    );
}

#[test]
fn filler_replays_its_private_seed_and_ignores_caller_random() {
    let clip = box_(pos(-20, -20, -20), pos(40, 100, 40));
    let mut left_piece = piece(FortressPieceKind::BridgeEndFiller);
    left_piece.filler_seed = -123_456_789;
    let mut right_piece = left_piece.clone();
    let mut left = FortressTestWorld::default();
    let mut right = FortressTestWorld::default();
    let mut caller = CountingRandom::default();
    place_fortress_piece(&mut left, &mut left_piece, &clip, &mut caller, &mut || 1);
    place_fortress_piece(&mut right, &mut right_piece, &clip, &mut caller, &mut || 2);
    assert_eq!(left.writes, right.writes);
    assert_eq!(caller.draws, 0);
}

#[test]
fn fortress_piece_orientation_applies_source_mirror_then_rotation() {
    let clip = box_(pos(-40, -20, -40), pos(40, 100, 40));
    for (orientation, stair_facing, fence_axis) in [
        (HorizontalDirection::North, "south", ["north", "south"]),
        (HorizontalDirection::South, "north", ["north", "south"]),
        (HorizontalDirection::West, "east", ["west", "east"]),
        (HorizontalDirection::East, "west", ["east", "west"]),
    ] {
        let oriented =
            OrientedPiece::from_anchor(pos(0, 40, 0), pos(0, 0, 0), [5, 14, 10], orientation);
        let mut piece = piece(FortressPieceKind::CastleCorridorStairs);
        piece.bounding_box = oriented.bounds;
        piece.orientation = orientation;
        let mut world = FortressTestWorld::default();
        place_fortress_piece(
            &mut world,
            &mut piece,
            &clip,
            &mut CountingRandom::default(),
            &mut || 1,
        );
        let stair = oriented.world_position(pos(1, 8, 0));
        assert_eq!(world.states[&stair].properties["facing"], stair_facing);
        let fence = oriented.world_position(pos(0, 9, 0));
        assert!(
            fence_axis
                .iter()
                .all(|direction| world.states[&fence].properties[*direction] == "true")
        );
    }
}

#[derive(Default)]
struct ZeroRandom {
    bounds: Vec<u32>,
}

struct ScriptRandom {
    values: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl ScriptRandom {
    fn new(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            values: values.into_iter().collect(),
            bounds: Vec::new(),
        }
    }
}

impl GenerationRandom for ScriptRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.values.pop_front().unwrap_or(0);
        assert!(value < bound.get());
        value
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
struct CountingRandom {
    draws: usize,
}

impl GenerationRandom for CountingRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        self.draws += 1;
        0
    }

    fn next_f32(&mut self) -> f32 {
        self.draws += 1;
        0.0
    }

    fn next_f64(&mut self) -> f64 {
        self.draws += 1;
        0.0
    }

    fn next_gaussian(&mut self) -> f64 {
        self.draws += 1;
        0.0
    }
}

#[derive(Default)]
struct FortressTestWorld {
    states: BTreeMap<BlockPos, StructureState>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    rejected_writes: BTreeSet<BlockPos>,
    fluid_ticks: Vec<(BlockPos, FluidState, u32)>,
    postprocessing: Vec<BlockPos>,
    loot: Vec<(BlockPos, String, i64)>,
    configured_spawners: Vec<BlockPos>,
    container_entity: bool,
    spawner_entity: bool,
}

impl PieceWorld for FortressTestWorld {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:netherrack"))
    }

    fn fluid_at(&mut self, position: BlockPos) -> FluidState {
        match self.states.get(&position).map(|state| state.block.as_str()) {
            Some("minecraft:lava") => FluidState::Lava,
            Some("minecraft:water") => FluidState::Water,
            _ => FluidState::Empty,
        }
    }

    fn set_state(&mut self, position: BlockPos, state: StructureState, flags: u32) -> bool {
        self.writes.push((position, state.clone(), flags));
        if self.rejected_writes.contains(&position) {
            return false;
        }
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
            !matches!(
                state.block.as_str(),
                "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
            )
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

impl FortressWorld for FortressTestWorld {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn fortress_support_replaceable(
        &mut self,
        _position: BlockPos,
        state: &StructureState,
    ) -> bool {
        matches!(
            state.block.as_str(),
            "minecraft:air"
                | "minecraft:cave_air"
                | "minecraft:void_air"
                | "minecraft:water"
                | "minecraft:lava"
                | "minecraft:glow_lichen"
                | "minecraft:seagrass"
                | "minecraft:tall_seagrass"
        )
    }

    fn is_blaze_spawner_block_entity(&mut self, _position: BlockPos) -> bool {
        self.spawner_entity
    }

    fn configure_blaze_spawner(&mut self, position: BlockPos, _random: &mut impl GenerationRandom) {
        self.configured_spawners.push(position);
    }
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos::new(x, y, z)
}

fn box_(minimum: BlockPos, maximum: BlockPos) -> BlockBox {
    BlockBox::new(minimum, maximum).unwrap()
}

fn piece(kind: FortressPieceKind) -> FortressPiece {
    let [width, height, depth] = match kind {
        FortressPieceKind::Start | FortressPieceKind::BridgeCrossing => [19, 10, 19],
        FortressPieceKind::BridgeStraight => [5, 10, 19],
        FortressPieceKind::RoomCrossing => [7, 9, 7],
        FortressPieceKind::StairsRoom => [7, 11, 7],
        FortressPieceKind::MonsterThrone => [7, 8, 9],
        FortressPieceKind::CastleEntrance | FortressPieceKind::CastleStalkRoom => [13, 14, 13],
        FortressPieceKind::CastleSmallCorridor
        | FortressPieceKind::CastleSmallCrossing
        | FortressPieceKind::CastleRightTurn
        | FortressPieceKind::CastleLeftTurn => [5, 7, 5],
        FortressPieceKind::CastleCorridorStairs => [5, 14, 10],
        FortressPieceKind::CastleTBalcony => [9, 7, 9],
        FortressPieceKind::BridgeEndFiller => [5, 10, 8],
    };
    FortressPiece {
        kind,
        bounding_box: box_(pos(0, 40, 0), pos(width - 1, 40 + height - 1, depth - 1)),
        generation_depth: 1,
        orientation: HorizontalDirection::South,
        chest_pending: false,
        spawner_placed: false,
        filler_seed: 0,
    }
}
