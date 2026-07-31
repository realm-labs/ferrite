use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::PathBuf;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::igloo::{
    IglooPart, IglooPlacementRuntime, generate_igloo_pieces,
};
use ferrite_world::generation::structure::nbt::{NbtCompound, NbtValue};
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::processor::{Heightmap, ProcessorWorld, StructureState};
use ferrite_world::generation::structure::template_manager::{FileTemplateSource, TemplateManager};
use ferrite_world::generation::structure::template_place::{PlacedTemplateEntity, TemplateWorld};

#[test]
fn generation_orders_bottom_shaft_top_and_keeps_half_open_chance() {
    let pieces = generate_igloo_pieces(pos(10, 0, 20), &mut ScriptRandom::new([0, 7], [0.49]));
    assert_eq!(pieces.len(), 12);
    assert_eq!(pieces[0].part, IglooPart::Bottom);
    assert_eq!(pieces[0].original_position, pos(10, 57, 20));
    assert!(
        pieces[1..11]
            .iter()
            .all(|piece| piece.part == IglooPart::Middle)
    );
    assert_eq!(pieces[1].original_position.y, 90);
    assert_eq!(pieces[10].original_position.y, 63);
    assert_eq!(pieces[11].part, IglooPart::Top);

    let top_only = generate_igloo_pieces(pos(10, 0, 20), &mut ScriptRandom::new([0, 0], [0.5]));
    assert_eq!(top_only.len(), 1);
    assert_eq!(top_only[0].part, IglooPart::Top);
}

#[test]
fn official_top_uses_live_probe_and_repairs_snow_without_clip_gate() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let [top] = generate_igloo_pieces(pos(10, 0, 20), &mut ScriptRandom::new([0], [0.5]))
        .try_into()
        .unwrap();
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = IglooPlacementRuntime {
        templates: &mut manager,
    };
    let clip = BlockBox::new(pos(-100, -100, -100), pos(100, 200, 200)).unwrap();
    let mut world = World {
        surface_height: 100,
        ..World::default()
    };
    world
        .states
        .insert(pos(13, 98, 25), StructureState::new("minecraft:stone"));
    let mut no_seed = || panic!("top has no loot container");

    assert!(
        runtime
            .place(&mut world, &top, &clip, &mut ZeroRandom, &mut no_seed)
            .unwrap()
    );
    assert_eq!(world.height_probes, [(13, 20)]);
    // The 152 listed states include one NBT-bearing furnace, whose barrier
    // prewrite precedes the final state; top support repair is the last offer.
    assert_eq!(world.writes.len(), 154);
    assert_eq!(
        world.writes.last(),
        Some(&(
            pos(13, 99, 25),
            StructureState::new("minecraft:snow_block"),
            3,
        ))
    );
    assert_eq!(top.original_position, pos(10, 90, 20));
}

#[test]
fn official_bottom_loads_entities_and_marker_seed_overrides_generic_chest_seed() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let pieces = generate_igloo_pieces(pos(0, 0, 0), &mut ScriptRandom::new([0, 0, 0], [0.0]));
    let bottom = &pieces[0];
    assert_eq!(bottom.part, IglooPart::Bottom);
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = IglooPlacementRuntime {
        templates: &mut manager,
    };
    let clip = BlockBox::new(pos(-100, -100, -100), pos(100, 200, 100)).unwrap();
    let mut world = World {
        surface_height: 100,
        ..World::default()
    };
    let mut seeds = [11_i64, 22].into_iter();
    let mut next_seed = || seeds.next().expect("exactly two bottom chest seeds");

    assert!(
        runtime
            .place(&mut world, bottom, &clip, &mut ZeroRandom, &mut next_seed,)
            .unwrap()
    );
    assert_eq!(world.height_probes, [(3, 0)]);
    assert_eq!(world.entities.len(), 2);
    assert!(world.entities.iter().all(|(_, finalize)| !finalize));
    assert!(
        world
            .entities
            .iter()
            .all(|(entity, _)| !entity.nbt.contains_key("UUID") && entity.nbt.contains_key("Pos"))
    );
    let chest_nbt = world
        .loaded_nbt
        .iter()
        .find(|(_, nbt)| nbt.get("LootTableSeed") == Some(&NbtValue::Long(11)))
        .expect("generic chest load consumes the first seed");
    assert!(world.states[&chest_nbt.0].block == "minecraft:chest");
    assert_eq!(
        world.installed_loot,
        [(chest_nbt.0, "minecraft:chests/igloo_chest".into(), 22)]
    );
    assert!(seeds.next().is_none());
}

fn local_resource_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/mc-reference/26.2/client-classes");
    root.join("data/minecraft/structure/igloo")
        .is_dir()
        .then_some(root)
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    surface_height: i32,
    height_probes: Vec<(i32, i32)>,
    states: BTreeMap<BlockPos, StructureState>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    loaded_nbt: Vec<(BlockPos, NbtCompound)>,
    installed_loot: Vec<(BlockPos, String, i64)>,
    entities: Vec<(PlacedTemplateEntity, bool)>,
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
        self.height_probes.push((x, z));
        self.surface_height
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
    fn place_template_entity(&mut self, entity: PlacedTemplateEntity, finalize: bool) {
        self.entities.push((entity, finalize));
    }
}

struct ScriptRandom {
    integers: std::vec::IntoIter<u32>,
    doubles: std::vec::IntoIter<f64>,
}

impl ScriptRandom {
    fn new(
        integers: impl IntoIterator<Item = u32>,
        doubles: impl IntoIterator<Item = f64>,
    ) -> Self {
        Self {
            integers: integers.into_iter().collect::<Vec<_>>().into_iter(),
            doubles: doubles.into_iter().collect::<Vec<_>>().into_iter(),
        }
    }
}

impl GenerationRandom for ScriptRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.integers
            .next()
            .unwrap_or_default()
            .min(bound.get() - 1)
    }
    fn next_f32(&mut self) -> f32 {
        0.0
    }
    fn next_f64(&mut self) -> f64 {
        self.doubles.next().unwrap_or_default()
    }
    fn next_gaussian(&mut self) -> f64 {
        0.0
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
