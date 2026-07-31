use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::PathBuf;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::end_city::{
    EndCityElytraFrameSpawn, EndCityFrameFacing, EndCityRuntime, EndCityShulkerSpawn, EndCityWorld,
};
use ferrite_world::generation::structure::end_city_graph::{
    end_city_start_anchor, generate_end_city,
};
use ferrite_world::generation::structure::jigsaw::Rotation;
use ferrite_world::generation::structure::nbt::NbtCompound;
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::processor::{Heightmap, ProcessorWorld, StructureState};
use ferrite_world::generation::structure::template_manager::{FileTemplateSource, TemplateManager};
use ferrite_world::generation::structure::template_place::{PlacedTemplateEntity, TemplateWorld};

#[test]
fn all_locked_templates_match_exact_inventory_and_tower_floor_is_present() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let expected = [
        ("base_floor", [10, 4, 10], 400),
        ("base_roof", [12, 2, 12], 288),
        ("second_floor_1", [12, 8, 12], 1_152),
        ("second_floor_2", [12, 8, 12], 1_152),
        ("second_roof", [14, 2, 14], 392),
        ("third_floor_1", [14, 8, 14], 1_568),
        ("third_floor_2", [14, 8, 14], 1_568),
        ("third_roof", [16, 2, 16], 512),
        ("tower_base", [7, 7, 7], 202),
        ("tower_floor", [7, 4, 7], 196),
        ("tower_piece", [7, 4, 7], 196),
        ("tower_top", [9, 5, 9], 405),
        ("bridge_end", [5, 6, 2], 60),
        ("bridge_piece", [5, 6, 4], 120),
        ("bridge_gentle_stairs", [5, 7, 8], 280),
        ("bridge_steep_stairs", [5, 7, 4], 140),
        ("fat_tower_base", [13, 4, 13], 676),
        ("fat_tower_middle", [13, 8, 13], 1_352),
        ("fat_tower_top", [17, 6, 17], 1_734),
        ("ship", [13, 24, 29], 9_048),
    ];
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    for (name, size, blocks) in expected {
        let template = manager
            .require(&format!("minecraft:end_city/{name}"))
            .unwrap()
            .template;
        assert_eq!(template.size, size, "{name}");
        assert_eq!(template.blocks.len(), blocks, "{name}");
        assert_eq!(template.palettes.len(), 1, "{name}");
        assert!(template.entities.is_empty(), "{name}");
    }
}

#[test]
fn overwrite_mode_writes_listed_air_and_connected_offsets_follow_parent_rotation() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = EndCityRuntime {
        templates: &mut manager,
    };
    let mut overwrite = runtime
        .create_piece("base_floor", pos(10, 70, 20), Rotation::Clockwise90, true)
        .unwrap();
    let child = runtime
        .connect_piece(
            &overwrite,
            "second_floor_1",
            pos(-1, 0, -1),
            Rotation::Clockwise90,
            false,
        )
        .unwrap();
    assert_eq!(child.position, pos(11, 70, 19));
    let clip = BlockBox::new(pos(-100, -100, -100), pos(200, 200, 200)).unwrap();
    let mut world = World::default();
    let mut no_seed = || panic!("base floor has no chest");
    runtime
        .place(
            &mut world,
            &mut overwrite,
            &clip,
            &mut ZeroRandom,
            &mut no_seed,
        )
        .unwrap();
    assert_eq!(world.writes.len(), 398);

    world.writes.clear();
    overwrite.overwrite = false;
    runtime
        .place(
            &mut world,
            &mut overwrite,
            &clip,
            &mut ZeroRandom,
            &mut no_seed,
        )
        .unwrap();
    assert_eq!(world.writes.len(), 148);
}

#[test]
fn ship_chests_draw_twice_and_markers_create_three_shulkers_and_elytra_frame() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = EndCityRuntime {
        templates: &mut manager,
    };
    let mut ship = runtime
        .create_piece("ship", pos(0, 80, 0), Rotation::Clockwise90, true)
        .unwrap();
    let clip = BlockBox::new(pos(-50, 0, -50), pos(50, 150, 50)).unwrap();
    let mut world = World::default();
    let mut seeds = [1_i64, 2, 3, 4].into_iter();
    let mut next_seed = || seeds.next().expect("two generic and two marker seeds");

    assert!(
        runtime
            .place(
                &mut world,
                &mut ship,
                &clip,
                &mut ZeroRandom,
                &mut next_seed
            )
            .unwrap()
    );
    assert_eq!(
        world
            .installed_loot
            .iter()
            .map(|(_, table, seed)| (table.as_str(), *seed))
            .collect::<Vec<_>>(),
        [
            ("minecraft:chests/end_city_treasure", 3),
            ("minecraft:chests/end_city_treasure", 4),
        ]
    );
    assert_eq!(world.shulkers.len(), 3);
    assert!(
        world
            .shulkers
            .iter()
            .all(|spawn| spawn.structure_creation && !spawn.finalize_spawn)
    );
    assert_eq!(world.frames.len(), 1);
    assert_eq!(world.frames[0].facing, EndCityFrameFacing::West);
    assert_eq!(world.frames[0].item, "minecraft:elytra");
    assert!(!world.frames[0].play_item_sound);
    assert!(seeds.next().is_none());
}

#[test]
fn start_height_precedes_graph_and_zero_stream_builds_the_fixed_base_and_capped_tower() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut world = World::default();
    let mut heights = [(7, 7), (12, 7), (7, 12), (12, 12)]
        .into_iter()
        .map(|position| (position, 60))
        .collect::<BTreeMap<_, _>>();
    world.height_overrides.append(&mut heights);
    let (anchor, rotation) =
        end_city_start_anchor(&mut world, pos(0, 0, 0), &mut ZeroRandom).unwrap();
    assert_eq!(anchor, pos(7, 60, 7));
    assert_eq!(rotation, Rotation::None);
    assert_eq!(world.height_queries, [(7, 7), (12, 7), (7, 12), (12, 12)]);

    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = EndCityRuntime {
        templates: &mut manager,
    };
    let pieces = generate_end_city(&mut runtime, anchor, rotation, &mut ZeroRandom).unwrap();
    assert_eq!(pieces.len(), 8);
    assert_eq!(
        pieces
            .iter()
            .map(|piece| piece.template.rsplit('/').next().unwrap())
            .collect::<Vec<_>>(),
        [
            "base_floor",
            "second_floor_1",
            "third_floor_1",
            "third_roof",
            "tower_base",
            "tower_piece",
            "tower_piece",
            "tower_top",
        ]
    );
}

fn local_resource_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/mc-reference/26.2/client-classes");
    root.join("data/minecraft/structure/end_city")
        .is_dir()
        .then_some(root)
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    states: BTreeMap<BlockPos, StructureState>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    loaded_nbt: Vec<(BlockPos, NbtCompound)>,
    installed_loot: Vec<(BlockPos, String, i64)>,
    shulkers: Vec<EndCityShulkerSpawn>,
    frames: Vec<EndCityElytraFrameSpawn>,
    height_overrides: BTreeMap<(i32, i32), i32>,
    height_queries: Vec<(i32, i32)>,
}

impl PieceWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:end_stone"))
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
        self.installed_loot.push((position, table.into(), seed));
    }
}

impl ProcessorWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        PieceWorld::state_at(self, position)
    }
    fn height(&mut self, _heightmap: Heightmap, x: i32, z: i32) -> i32 {
        self.height_queries.push((x, z));
        self.height_overrides.get(&(x, z)).copied().unwrap_or(80)
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
    fn load_template_nbt(&mut self, position: BlockPos, nbt: NbtCompound) {
        self.loaded_nbt.push((position, nbt));
    }
    fn place_template_entity(&mut self, _entity: PlacedTemplateEntity, _finalize: bool) {
        panic!("end-city raw template entities are ignored");
    }
}

impl EndCityWorld for World {
    fn is_spawnable_bounds(&self, _position: BlockPos) -> bool {
        true
    }
    fn spawn_end_city_shulker(&mut self, request: EndCityShulkerSpawn) {
        self.shulkers.push(request);
    }
    fn spawn_end_city_elytra_frame(&mut self, request: EndCityElytraFrameSpawn) {
        self.frames.push(request);
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
