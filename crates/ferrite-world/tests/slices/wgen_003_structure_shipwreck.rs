use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::PathBuf;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::nbt::NbtCompound;
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::processor::{Heightmap, ProcessorWorld, StructureState};
use ferrite_world::generation::structure::shipwreck::ShipwreckRuntime;
use ferrite_world::generation::structure::template_manager::{FileTemplateSource, TemplateManager};
use ferrite_world::generation::structure::template_place::{PlacedTemplateEntity, TemplateWorld};

#[test]
fn official_choices_preserve_locked_order_and_defer_long_z_templates() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = ShipwreckRuntime {
        templates: &mut manager,
    };
    let beached = runtime
        .generate_piece(pos(10, 0, 20), true, &mut ScriptRandom::new([0, 10]))
        .unwrap();
    assert_eq!(
        beached.template,
        "minecraft:shipwreck/rightsideup_backhalf_degraded"
    );
    assert!(!beached.height_adjusted);
    assert!(!beached.is_too_big_for_worldgen_region());

    let ocean = runtime
        .generate_piece(pos(-16, 0, 32), false, &mut ScriptRandom::new([3, 19]))
        .unwrap();
    assert_eq!(
        ocean.template,
        "minecraft:shipwreck/rightsideup_backhalf_degraded"
    );
    assert!(!ocean.is_too_big_for_worldgen_region());
    assert_eq!(ocean.position, pos(-16, 90, 32));
}

#[test]
fn beached_mast_defers_live_height_then_generic_and_marker_chests_each_draw() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = ShipwreckRuntime {
        templates: &mut manager,
    };
    let mut piece = runtime
        .generate_piece(pos(0, 0, 0), true, &mut ScriptRandom::new([0, 0]))
        .unwrap();
    let mut world = World {
        height: 80,
        ..World::default()
    };
    let clip = BlockBox::new(pos(-40, -100, -40), pos(80, 200, 80)).unwrap();
    let mut caller = ScriptRandom::new([2]);
    let mut seeds = [1_i64, 2, 3, 4, 5, 6].into_iter();
    let mut next_seed = || seeds.next().expect("three generic and three marker seeds");

    assert!(
        runtime
            .place(
                &mut world,
                &mut piece,
                &clip,
                319,
                &mut caller,
                &mut next_seed,
            )
            .unwrap()
    );
    assert_eq!(piece.position.y, 68);
    assert!(piece.height_adjusted);
    assert_eq!(world.height_calls.len(), 9 * 28);
    assert!(
        world
            .height_calls
            .iter()
            .all(|(map, _, _)| *map == Heightmap::WorldSurfaceWorldgen)
    );
    assert_eq!(world.loaded_nbt.len(), 3);
    assert_eq!(
        world
            .installed_loot
            .iter()
            .map(|(_, table, seed)| (table.as_str(), *seed))
            .collect::<Vec<_>>(),
        [
            ("minecraft:chests/shipwreck_supply", 4),
            ("minecraft:chests/shipwreck_map", 5),
            ("minecraft:chests/shipwreck_treasure", 6),
        ]
    );
    assert!(seeds.next().is_none());
}

fn local_resource_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/mc-reference/26.2/client-classes");
    root.join("data/minecraft/structure/shipwreck")
        .is_dir()
        .then_some(root)
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    height: i32,
    height_calls: Vec<(Heightmap, i32, i32)>,
    states: BTreeMap<BlockPos, StructureState>,
    loaded_nbt: Vec<(BlockPos, NbtCompound)>,
    installed_loot: Vec<(BlockPos, String, i64)>,
}

impl PieceWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:water"))
    }
    fn fluid_at(&mut self, _position: BlockPos) -> FluidState {
        FluidState::Water
    }
    fn set_state(&mut self, position: BlockPos, state: StructureState, _flags: u32) -> bool {
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
    fn height(&mut self, heightmap: Heightmap, x: i32, z: i32) -> i32 {
        self.height_calls.push((heightmap, x, z));
        self.height
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
        panic!("locked shipwreck templates have no entities");
    }
}

struct ScriptRandom {
    values: std::vec::IntoIter<u32>,
}

impl ScriptRandom {
    fn new(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            values: values.into_iter().collect::<Vec<_>>().into_iter(),
        }
    }
}

impl GenerationRandom for ScriptRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.values.next().unwrap_or_default().min(bound.get() - 1)
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
