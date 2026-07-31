use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::PathBuf;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::{GenerationRandom, LegacyRandom};
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::jigsaw::Rotation;
use ferrite_world::generation::structure::mansion_catalog::{
    MANSION_BIOMES, MANSION_COMMON_LOOT, MANSION_COMMON_ROLLS, MANSION_LOOT_TABLE,
    MANSION_RARE_LOOT, MANSION_RARE_ROLLS, MANSION_STEP, MANSION_SUPPLY_LOOT, MANSION_SUPPLY_ROLLS,
    MANSION_TEMPLATES, MANSION_TERRAIN_ADAPTATION, MANSION_TRIM_EMPTY_WEIGHT,
    MANSION_TRIM_TEMPLATE, MANSION_TRIM_WEIGHT, WOODLAND_MANSIONS_SALT,
    WOODLAND_MANSIONS_SEPARATION, WOODLAND_MANSIONS_SPACING, WOODLAND_MANSIONS_SPREAD_TYPE,
};
use ferrite_world::generation::structure::mansion_pieces::{
    MansionPieceSpec, generate_mansion_specs,
};
use ferrite_world::generation::structure::mansion_runtime::{
    MansionMobSpawn, MansionRuntime, MansionWorld, place_foundation,
};
use ferrite_world::generation::structure::nbt::NbtCompound;
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::processor::{Heightmap, ProcessorWorld, StructureState};
use ferrite_world::generation::structure::template_manager::{FileTemplateSource, TemplateManager};
use ferrite_world::generation::structure::template_place::{
    PlacedTemplateEntity, TemplateMirror, TemplateRotation, TemplateTransform, TemplateWorld,
    data_markers,
};

#[test]
fn locked_corpus_has_all_exact_sizes_cells_and_marker_totals() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let expected = expected_templates();
    assert_eq!(expected.len(), MANSION_TEMPLATES.len());
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let clip = BlockBox::new(pos(-100, -100, -100), pos(100, 100, 100)).unwrap();
    let mut markers = BTreeMap::<String, usize>::new();
    for ((name, size, blocks, block_nbt), catalog_name) in
        expected.into_iter().zip(MANSION_TEMPLATES)
    {
        assert_eq!(name, catalog_name);
        let template = manager
            .require(&format!("minecraft:woodland_mansion/{name}"))
            .unwrap()
            .template;
        assert_eq!(template.size, size, "{name}");
        assert_eq!(template.blocks.len(), blocks, "{name}");
        assert_eq!(
            template
                .blocks
                .iter()
                .filter(|block| block.nbt.is_some())
                .count(),
            block_nbt,
            "{name}"
        );
        assert_eq!(template.palettes.len(), 1, "{name}");
        assert!(template.entities.is_empty(), "{name}");
        assert!(
            template.palettes[0]
                .states
                .iter()
                .all(|state| state.block != "minecraft:structure_void"),
            "{name}"
        );
        for marker in data_markers(
            &template,
            0,
            TemplateTransform {
                origin: pos(0, 0, 0),
                pivot: pos(0, 0, 0),
                mirror: TemplateMirror::None,
                rotation: TemplateRotation::None,
            },
            &clip,
        ) {
            *markers.entry(marker.metadata).or_default() += 1;
        }
    }
    assert_eq!(markers.values().sum::<usize>(), 38);
    assert_eq!(markers["ChestWest"], 3);
    assert_eq!(markers["ChestSouth"], 5);
    assert_eq!(markers["ChestNorth"], 2);
    assert_eq!(markers["Warrior"], 20);
    assert_eq!(markers["Mage"], 4);
    assert_eq!(markers["Group of Allays"], 4);
}

#[test]
fn mansion_records_preserve_biomes_set_and_four_loot_pools() {
    assert_eq!(MANSION_STEP, "surface_structures");
    assert_eq!(MANSION_TERRAIN_ADAPTATION, "none");
    assert_eq!(
        MANSION_BIOMES,
        ["minecraft:dark_forest", "minecraft:pale_garden"]
    );
    assert_eq!(WOODLAND_MANSIONS_SPREAD_TYPE, "triangular");
    assert_eq!(
        (
            WOODLAND_MANSIONS_SPACING,
            WOODLAND_MANSIONS_SEPARATION,
            WOODLAND_MANSIONS_SALT,
        ),
        (80, 20, 10_387_319)
    );
    assert_eq!(MANSION_LOOT_TABLE, "minecraft:chests/woodland_mansion");
    assert_eq!(MANSION_RARE_ROLLS, (1, 3));
    assert_eq!(MANSION_SUPPLY_ROLLS, (1, 4));
    assert_eq!(MANSION_COMMON_ROLLS, 3);
    assert_eq!(
        MANSION_RARE_LOOT
            .iter()
            .map(|entry| entry.weight)
            .sum::<u32>(),
        107
    );
    assert_eq!(
        MANSION_SUPPLY_LOOT
            .iter()
            .map(|entry| entry.weight)
            .sum::<u32>(),
        175
    );
    assert_eq!(
        MANSION_COMMON_LOOT
            .iter()
            .map(|entry| entry.weight)
            .sum::<u32>(),
        4
    );
    assert_eq!((MANSION_TRIM_EMPTY_WEIGHT, MANSION_TRIM_WEIGHT), (1, 1));
    assert_eq!(
        MANSION_TRIM_TEMPLATE,
        "minecraft:vex_armor_trim_smithing_template"
    );
}

#[test]
fn graph_and_piece_schedule_are_repeatable_and_source_ordered() {
    let origin = pos(80, 70, -40);
    let mut first_random = LegacyRandom::new(0x1234_5678);
    let (first_layout, first) =
        generate_mansion_specs(origin, Rotation::Clockwise90, &mut first_random);
    let mut second_random = LegacyRandom::new(0x1234_5678);
    let (second_layout, second) =
        generate_mansion_specs(origin, Rotation::Clockwise90, &mut second_random);
    assert_eq!(first_layout, second_layout);
    assert_eq!(first, second);
    assert_eq!(first.first().unwrap().template, "entrance");
    assert_eq!(first.first().unwrap().position, pos(80, 70, -49));
    assert!(
        first
            .iter()
            .all(|piece| MANSION_TEMPLATES.contains(&piece.template.as_str()))
    );
    assert_eq!(first.len(), 529);
    let first_roof = first
        .iter()
        .position(|piece| piece.template == "roof")
        .unwrap();
    let first_corridor = first
        .iter()
        .position(|piece| piece.template == "corridor_floor")
        .unwrap();
    assert!(first_roof < first_corridor);
}

#[test]
fn chest_marker_rotates_facing_without_mirror_and_consumes_one_seed() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = MansionRuntime {
        templates: &mut manager,
    };
    let mut piece = runtime
        .create_piece(&MansionPieceSpec {
            template: "1x1_a4".into(),
            position: pos(10, 80, 10),
            rotation: Rotation::Clockwise90,
            mirror: TemplateMirror::FrontBack,
        })
        .unwrap();
    let clip = BlockBox::new(pos(-100, 0, -100), pos(100, 200, 100)).unwrap();
    let mut world = World::default();
    let mut seed_calls = 0;
    let mut seed = || {
        seed_calls += 1;
        77
    };
    assert!(
        runtime
            .place(&mut world, &mut piece, &clip, &mut ZeroRandom, &mut seed)
            .unwrap()
    );
    assert_eq!(seed_calls, 1);
    assert_eq!(world.loot.len(), 1);
    let chest = &world.states[&world.loot[0].0];
    assert_eq!(chest.properties["facing"], "north");
    assert_eq!(world.loot[0].1, "minecraft:chests/woodland_mansion");
    assert_eq!(world.loot[0].2, 77);
}

#[test]
fn allay_groups_use_world_rng_and_clear_each_successful_marker() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = MansionRuntime {
        templates: &mut manager,
    };
    let mut piece = runtime
        .create_piece(&MansionPieceSpec {
            template: "2x2_a1".into(),
            position: pos(0, 70, 0),
            rotation: Rotation::None,
            mirror: TemplateMirror::None,
        })
        .unwrap();
    let clip = BlockBox::new(pos(-20, 0, -20), pos(40, 120, 40)).unwrap();
    let mut world = World {
        world_draw: 2,
        spawn_success: true,
        ..World::default()
    };
    let mut no_seed = || panic!("2x2_a1 has no chest marker");
    runtime
        .place(&mut world, &mut piece, &clip, &mut ZeroRandom, &mut no_seed)
        .unwrap();
    assert_eq!(world.world_random_calls, 4);
    assert_eq!(world.mobs.len(), 13);
    assert!(world.mobs.iter().all(|spawn| {
        spawn.persistent
            && spawn.finalize_for_local_difficulty
            && spawn.structure_spawn_reason
            && spawn.add_with_passengers
    }));
    assert_eq!(world.marker_clear_writes, 13,);
}

#[test]
fn null_mob_factories_leave_marker_blocks_untouched() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = MansionRuntime {
        templates: &mut manager,
    };
    let mut piece = runtime
        .create_piece(&MansionPieceSpec {
            template: "2x2_a1".into(),
            position: pos(0, 70, 0),
            rotation: Rotation::None,
            mirror: TemplateMirror::None,
        })
        .unwrap();
    let clip = BlockBox::new(pos(-20, 0, -20), pos(40, 120, 40)).unwrap();
    let mut world = World::default();
    let mut no_seed = || panic!("2x2_a1 has no chest marker");
    runtime
        .place(&mut world, &mut piece, &clip, &mut ZeroRandom, &mut no_seed)
        .unwrap();
    assert_eq!(world.mobs.len(), 5);
    assert_eq!(world.marker_clear_writes, 0);
}

#[test]
fn foundation_requires_a_piece_seed_and_fills_air_and_liquid_until_solid() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = MansionRuntime {
        templates: &mut manager,
    };
    let piece = runtime
        .create_piece(&MansionPieceSpec {
            template: "wall_flat".into(),
            position: pos(0, 70, 0),
            rotation: Rotation::None,
            mirror: TemplateMirror::None,
        })
        .unwrap();
    let seed = piece.bounding_box.minimum;
    let mut world = World::default();
    world
        .states
        .insert(seed, StructureState::new("minecraft:dark_oak_planks"));
    world
        .fluids
        .insert(pos(seed.x, 69, seed.z), FluidState::Water);
    world.states.insert(
        pos(seed.x, 67, seed.z),
        StructureState::new("minecraft:stone"),
    );
    let clip = BlockBox::new(pos(seed.x, -64, seed.z), pos(seed.x + 1, 100, seed.z)).unwrap();
    place_foundation(&mut world, &[piece], &clip, -64);
    assert_eq!(
        world.states[&pos(seed.x, 69, seed.z)].block,
        "minecraft:cobblestone"
    );
    assert_eq!(
        world.states[&pos(seed.x, 68, seed.z)].block,
        "minecraft:cobblestone"
    );
    assert_eq!(
        world.states[&pos(seed.x, 67, seed.z)].block,
        "minecraft:stone"
    );
    assert!(!world.states.contains_key(&pos(seed.x + 1, 69, seed.z)));
}

fn expected_templates() -> Vec<(&'static str, [i32; 3], usize, usize)> {
    vec![
        ("entrance", [21, 19, 16], 6_288, 0),
        ("wall_flat", [2, 8, 8], 127, 0),
        ("wall_window", [2, 8, 8], 128, 0),
        ("wall_corner", [9, 8, 2], 32, 0),
        ("roof", [8, 1, 8], 64, 0),
        ("roof_front", [4, 4, 8], 126, 0),
        ("small_wall", [2, 4, 8], 62, 0),
        ("small_wall_corner", [2, 4, 2], 16, 0),
        ("roof_corner", [4, 4, 4], 56, 0),
        ("roof_inner_corner", [4, 4, 4], 50, 0),
        ("corridor_floor", [7, 8, 7], 392, 0),
        ("carpet_north", [5, 1, 2], 8, 0),
        ("carpet_east", [2, 1, 5], 8, 0),
        ("carpet_south_1", [8, 8, 3], 172, 0),
        ("carpet_west_1", [3, 8, 8], 172, 0),
        ("carpet_south_2", [8, 11, 3], 239, 0),
        ("carpet_west_2", [3, 11, 8], 238, 0),
        ("indoors_wall_1", [1, 8, 8], 64, 0),
        ("indoors_door_1", [1, 8, 8], 64, 0),
        ("indoors_wall_2", [1, 11, 8], 88, 0),
        ("indoors_door_2", [1, 11, 8], 88, 0),
        ("1x1_a1", [7, 8, 7], 392, 0),
        ("1x1_a2", [7, 8, 7], 392, 0),
        ("1x1_a3", [7, 8, 7], 392, 0),
        ("1x1_a4", [7, 8, 7], 392, 1),
        ("1x1_a5", [7, 8, 7], 392, 0),
        ("1x1_as1", [7, 8, 7], 392, 1),
        ("1x1_as2", [7, 8, 7], 392, 1),
        ("1x1_as3", [7, 8, 7], 392, 0),
        ("1x1_as4", [7, 8, 7], 392, 0),
        ("1x2_a1", [7, 8, 15], 840, 3),
        ("1x2_a2", [7, 8, 15], 840, 0),
        ("1x2_a3", [7, 8, 15], 840, 1),
        ("1x2_a4", [7, 8, 15], 840, 1),
        ("1x2_a5", [7, 8, 15], 840, 0),
        ("1x2_a6", [7, 8, 15], 840, 1),
        ("1x2_a7", [7, 8, 15], 840, 1),
        ("1x2_a8", [7, 8, 15], 840, 1),
        ("1x2_a9", [7, 8, 15], 840, 43),
        ("1x2_b1", [7, 8, 15], 840, 1),
        ("1x2_b2", [7, 8, 15], 840, 1),
        ("1x2_b3", [7, 8, 15], 840, 2),
        ("1x2_b4", [7, 8, 15], 840, 1),
        ("1x2_b5", [7, 8, 15], 840, 0),
        ("1x2_s1", [7, 8, 15], 840, 1),
        ("1x2_s2", [7, 8, 15], 840, 1),
        ("2x2_a1", [15, 8, 15], 1_800, 5),
        ("2x2_a2", [15, 8, 15], 1_800, 27),
        ("2x2_a3", [15, 8, 15], 1_800, 0),
        ("2x2_a4", [15, 8, 15], 1_800, 0),
        ("2x2_s1", [15, 11, 15], 2_475, 0),
        ("1x1_b1", [7, 11, 7], 539, 0),
        ("1x1_b2", [7, 11, 7], 539, 0),
        ("1x1_b3", [7, 11, 7], 539, 0),
        ("1x1_b4", [7, 11, 7], 539, 0),
        ("1x1_b5", [7, 11, 7], 539, 1),
        ("1x2_c1", [7, 11, 15], 1_155, 0),
        ("1x2_c2", [7, 11, 15], 1_155, 0),
        ("1x2_c3", [7, 11, 15], 1_155, 3),
        ("1x2_c4", [7, 11, 15], 1_155, 0),
        ("1x2_c_stairs", [7, 22, 15], 2_310, 0),
        ("1x2_d1", [7, 11, 15], 1_155, 3),
        ("1x2_d2", [7, 11, 15], 1_155, 1),
        ("1x2_d3", [7, 11, 15], 1_155, 5),
        ("1x2_d4", [7, 11, 15], 1_155, 0),
        ("1x2_d5", [7, 11, 15], 1_155, 0),
        ("1x2_d_stairs", [7, 22, 15], 2_310, 0),
        ("1x2_se1", [7, 11, 15], 1_155, 2),
        ("2x2_b1", [15, 11, 15], 2_475, 3),
        ("2x2_b2", [15, 11, 15], 2_475, 3),
        ("2x2_b3", [15, 11, 15], 2_475, 0),
        ("2x2_b4", [15, 11, 15], 2_475, 3),
        ("2x2_b5", [15, 11, 15], 2_475, 1),
    ]
}

fn local_resource_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/mc-reference/26.2/client-classes");
    root.join("data/minecraft/structure/woodland_mansion")
        .is_dir()
        .then_some(root)
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos::new(x, y, z)
}

#[derive(Default)]
struct World {
    states: BTreeMap<BlockPos, StructureState>,
    fluids: BTreeMap<BlockPos, FluidState>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    loot: Vec<(BlockPos, String, i64)>,
    mobs: Vec<MansionMobSpawn>,
    world_draw: u32,
    world_random_calls: usize,
    spawn_success: bool,
    clear_after_spawn: bool,
    marker_clear_writes: usize,
}

impl PieceWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:air"))
    }

    fn fluid_at(&mut self, position: BlockPos) -> FluidState {
        self.fluids
            .get(&position)
            .copied()
            .unwrap_or(FluidState::Empty)
    }

    fn set_state(&mut self, position: BlockPos, state: StructureState, flags: u32) -> bool {
        if self.clear_after_spawn && state.block == "minecraft:air" && flags == 2 {
            self.marker_clear_writes += 1;
            self.clear_after_spawn = false;
        }
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

impl ProcessorWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        PieceWorld::state_at(self, position)
    }

    fn height(&mut self, _heightmap: Heightmap, _x: i32, _z: i32) -> i32 {
        70
    }

    fn is_full_collision(&mut self, _position: BlockPos, _state: &StructureState) -> bool {
        true
    }

    fn positional_seed(&self, position: BlockPos) -> i64 {
        i64::from(position.x) ^ i64::from(position.y) ^ i64::from(position.z)
    }

    fn capped_seed(&self, _template_origin: BlockPos) -> i64 {
        0
    }
}

impl TemplateWorld for World {
    fn load_template_nbt(&mut self, _position: BlockPos, _nbt: NbtCompound) {}

    fn place_template_entity(&mut self, _entity: PlacedTemplateEntity, _finalize: bool) {
        panic!("mansion templates contain no entities");
    }
}

impl MansionWorld for World {
    fn mansion_world_random(&mut self, bound: NonZeroU32) -> u32 {
        self.world_random_calls += 1;
        self.world_draw % bound.get()
    }

    fn spawn_mansion_mob(&mut self, request: MansionMobSpawn) -> bool {
        self.mobs.push(request);
        self.clear_after_spawn = self.spawn_success;
        self.spawn_success
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
