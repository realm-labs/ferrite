use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::PathBuf;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::nbt::NbtCompound;
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::processor::{Heightmap, ProcessorWorld, StructureState};
use ferrite_world::generation::structure::ruined_portal::{
    HorizontalFace, RuinedPortalGenerationWorld, RuinedPortalRecord, RuinedPortalRuntime,
    RuinedPortalWorld, VerticalPlacement,
};
use ferrite_world::generation::structure::template_manager::{FileTemplateSource, TemplateManager};
use ferrite_world::generation::structure::template_place::{PlacedTemplateEntity, TemplateWorld};

#[test]
fn locked_templates_match_sizes_counts_chests_and_jigsaw_finals() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let expected = [
        ("portal_1", [6, 10, 6], 304, 1),
        ("portal_2", [9, 12, 9], 750, 1),
        ("portal_3", [8, 9, 9], 554, 1),
        ("portal_4", [8, 9, 9], 500, 1),
        ("portal_5", [10, 10, 7], 601, 1),
        ("portal_6", [5, 7, 7], 212, 0),
        ("portal_7", [9, 7, 9], 510, 0),
        ("portal_8", [14, 9, 9], 1_054, 0),
        ("portal_9", [10, 8, 9], 640, 0),
        ("portal_10", [12, 8, 10], 880, 0),
        ("giant_portal_1", [11, 17, 16], 2_400, 0),
        ("giant_portal_2", [11, 16, 16], 2_266, 0),
        ("giant_portal_3", [16, 16, 16], 3_433, 0),
    ];
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    for (name, size, blocks, jigsaws) in expected {
        let template = manager
            .require(&format!("minecraft:ruined_portal/{name}"))
            .unwrap()
            .template;
        assert_eq!(template.size, size, "{name}");
        assert_eq!(template.blocks.len(), blocks, "{name}");
        assert_eq!(template.palettes.len(), 1, "{name}");
        assert!(template.entities.is_empty(), "{name}");
        let palette = &template.palettes[0];
        assert_eq!(
            template
                .blocks
                .iter()
                .filter(|block| palette.states[block.state_index].block == "minecraft:chest")
                .count(),
            1,
            "{name}"
        );
        assert_eq!(
            template
                .blocks
                .iter()
                .filter(|block| palette.states[block.state_index].block == "minecraft:jigsaw")
                .count(),
            jigsaws,
            "{name}"
        );
    }
}

#[test]
fn desert_selection_skips_exact_probability_draws_and_uses_partial_burial_endpoint() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = RuinedPortalRuntime {
        templates: &mut manager,
    };
    let mut world = World {
        height: 101,
        opaque: true,
        ..World::default()
    };
    let mut random = ScriptRandom::new([9, 3, 6], [0.05, 0.5], [0.0]);
    let piece = runtime
        .generate_piece(
            &mut world,
            RuinedPortalRecord::Desert,
            pos(0, 0, 0),
            &mut random,
        )
        .unwrap();
    assert_eq!(piece.template, "minecraft:ruined_portal/portal_10");
    assert_eq!(piece.vertical_placement, VerticalPlacement::PartlyBuried);
    assert!(!piece.properties.air_pocket);
    assert!(piece.mirror_front_back);
    assert_eq!(piece.position.y, 100);
    assert_eq!(random.float_draws, 2);
    assert_eq!(random.integer_draws, 3);
    assert_eq!(world.opacity_calls.len(), 3);
}

#[test]
fn only_center_chunk_owns_full_template_jigsaw_final_and_apron() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = RuinedPortalRuntime {
        templates: &mut manager,
    };
    let mut world = World {
        height: 81,
        opaque: true,
        ..World::default()
    };
    let mut generation = ScriptRandom::new([0, 0], [0.5, 0.0], [0.0]);
    let mut piece = runtime
        .generate_piece(
            &mut world,
            RuinedPortalRecord::Desert,
            pos(0, 0, 0),
            &mut generation,
        )
        .unwrap();
    world.height = piece.bounding_box.minimum.y + 1;
    world.states.clear();
    world.writes.clear();
    world.opacity_calls.clear();
    let miss = BlockBox::point(pos(100, piece.position.y, 100));
    let mut no_seed = || panic!("nonowner cannot place chest");
    assert!(
        !runtime
            .place(
                &mut world,
                &mut piece,
                &miss,
                &mut ScriptRandom::new([], [], []),
                &mut no_seed,
            )
            .unwrap()
    );
    assert!(world.writes.is_empty());

    let owner = BlockBox::point(piece.bounding_box.center());
    let mut seeds = [91_i64].into_iter();
    let mut next_seed = || seeds.next().expect("one ruined portal chest seed");
    assert!(
        runtime
            .place(
                &mut world,
                &mut piece,
                &owner,
                &mut ScriptRandom::new(
                    std::iter::repeat_n(0, 20_000),
                    std::iter::repeat_n(1.0, 2_000),
                    std::iter::repeat_n(0.0, 2_000)
                ),
                &mut next_seed,
            )
            .unwrap()
    );
    // The chest and the temporary Jigsaw both load their locked NBT before
    // the inherited final-state pass overwrites the Jigsaw cell.
    assert_eq!(world.loaded_nbt.len(), 2);
    assert!(
        world
            .writes
            .iter()
            .any(|(_, state, flags)| { state.block == "minecraft:netherrack" && *flags == 3 })
    );
    assert!(
        world
            .writes
            .iter()
            .any(|(position, _, _)| !piece.bounding_box.contains(*position))
    );
    assert!(seeds.next().is_none());
}

fn local_resource_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/mc-reference/26.2/client-classes");
    root.join("data/minecraft/structure/ruined_portal")
        .is_dir()
        .then_some(root)
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    height: i32,
    opaque: bool,
    opacity_calls: Vec<(Heightmap, BlockPos)>,
    states: BTreeMap<BlockPos, StructureState>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    loaded_nbt: Vec<(BlockPos, NbtCompound)>,
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
    fn install_loot(&mut self, _position: BlockPos, _table: &str, _seed: i64) {}
}

impl ProcessorWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        PieceWorld::state_at(self, position)
    }
    fn height(&mut self, _heightmap: Heightmap, _x: i32, _z: i32) -> i32 {
        self.height
    }
    fn is_full_collision(&mut self, _position: BlockPos, _state: &StructureState) -> bool {
        true
    }
    fn positional_seed(&self, position: BlockPos) -> i64 {
        i64::from(position.x) * 31 + i64::from(position.y) * 17 + i64::from(position.z)
    }
    fn capped_seed(&self, _template_origin: BlockPos) -> i64 {
        0
    }
}

impl RuinedPortalGenerationWorld for World {
    fn minimum_y(&self) -> i32 {
        -64
    }
    fn sea_level(&self) -> i32 {
        63
    }
    fn opaque_in_generator_column(&mut self, heightmap: Heightmap, position: BlockPos) -> bool {
        self.opacity_calls.push((heightmap, position));
        self.opaque
    }
    fn cold_enough_to_snow(&mut self, _position: BlockPos, _sea_level: i32) -> bool {
        false
    }
}

impl TemplateWorld for World {
    fn load_template_nbt(&mut self, position: BlockPos, nbt: NbtCompound) {
        self.loaded_nbt.push((position, nbt));
    }
    fn place_template_entity(&mut self, _entity: PlacedTemplateEntity, _finalize: bool) {
        panic!("locked ruined portals have no entities");
    }
}

impl RuinedPortalWorld for World {
    fn minimum_y(&self) -> i32 {
        -64
    }
    fn supports_vine_face(&mut self, _position: BlockPos, _face: HorizontalFace) -> bool {
        true
    }
}

struct ScriptRandom {
    integers: std::vec::IntoIter<u32>,
    floats: std::vec::IntoIter<f32>,
    doubles: std::vec::IntoIter<f64>,
    integer_draws: usize,
    float_draws: usize,
}

impl ScriptRandom {
    fn new(
        integers: impl IntoIterator<Item = u32>,
        floats: impl IntoIterator<Item = f32>,
        doubles: impl IntoIterator<Item = f64>,
    ) -> Self {
        Self {
            integers: integers.into_iter().collect::<Vec<_>>().into_iter(),
            floats: floats.into_iter().collect::<Vec<_>>().into_iter(),
            doubles: doubles.into_iter().collect::<Vec<_>>().into_iter(),
            integer_draws: 0,
            float_draws: 0,
        }
    }
}

impl GenerationRandom for ScriptRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.integer_draws += 1;
        self.integers
            .next()
            .unwrap_or_default()
            .min(bound.get() - 1)
    }
    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        self.floats.next().unwrap_or_default()
    }
    fn next_f64(&mut self) -> f64 {
        self.doubles.next().unwrap_or_default()
    }
    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
