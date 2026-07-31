use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::PathBuf;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::nbt::{NbtCompound, NbtValue};
use ferrite_world::generation::structure::ocean_ruin::{
    OceanRuinDrownedSpawn, OceanRuinRuntime, OceanRuinTemperature, OceanRuinWorld,
    generate_ocean_ruin_pieces,
};
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::processor::{Heightmap, ProcessorWorld, StructureState};
use ferrite_world::generation::structure::template_manager::{FileTemplateSource, TemplateManager};
use ferrite_world::generation::structure::template_place::{PlacedTemplateEntity, TemplateWorld};

#[test]
fn selection_keeps_inclusive_large_threshold_and_cold_overlay_order() {
    let cold = generate_ocean_ruin_pieces(
        pos(0, 0, 0),
        OceanRuinTemperature::Cold,
        &mut ScriptRandom::new([2, 3], [0.31]),
    );
    assert_eq!(cold.len(), 3);
    assert_eq!(
        cold.iter()
            .map(|piece| piece.template.as_str())
            .collect::<Vec<_>>(),
        [
            "minecraft:underwater_ruin/brick_4",
            "minecraft:underwater_ruin/cracked_4",
            "minecraft:underwater_ruin/mossy_4",
        ]
    );
    assert_eq!(
        cold.iter().map(|piece| piece.integrity).collect::<Vec<_>>(),
        [0.8, 0.7, 0.5]
    );

    let clustered = generate_ocean_ruin_pieces(
        pos(0, 0, 0),
        OceanRuinTemperature::Warm,
        &mut ScriptRandom::new(std::iter::repeat_n(0, 40), [0.3, 0.9]),
    );
    assert_eq!(
        clustered[0].template,
        "minecraft:underwater_ruin/big_warm_4"
    );
    assert!(clustered[0].large);
    assert_eq!(clustered.len(), 5);
    assert!(
        clustered[1..]
            .iter()
            .all(|piece| !piece.large && piece.integrity == 0.8)
    );
}

#[test]
fn all_locked_inputs_are_dense_single_palette_without_entities() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let names = locked_names();
    assert_eq!(names.len(), 48);
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    for name in names {
        let template = manager.require(&name).unwrap().template;
        assert_eq!(template.palettes.len(), 1, "{name}");
        assert_eq!(template.blocks.len(), template.volume(), "{name}");
        assert!(template.entities.is_empty(), "{name}");
        assert!(matches!(template.size, [6, 7, 7] | [16, 16, 16]), "{name}");
    }
}

#[test]
fn live_restack_caps_archaeology_and_markers_bypass_integrity() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut manager = TemplateManager::new(FileTemplateSource::new(root));
    let mut runtime = OceanRuinRuntime {
        templates: &mut manager,
    };
    let [mut piece] = generate_ocean_ruin_pieces(
        pos(0, 0, 0),
        OceanRuinTemperature::Warm,
        &mut ScriptRandom::new([0, 1], [0.5]),
    )
    .try_into()
    .unwrap();
    assert_eq!(piece.template, "minecraft:underwater_ruin/warm_2");
    let clip = BlockBox::new(pos(-20, -20, -20), pos(30, 100, 30)).unwrap();
    let mut world = World {
        ocean_floor: 50,
        solid_y: 45,
        spawn_success: true,
        ..World::default()
    };
    let mut seeds = [77_i64].into_iter();
    let mut next_seed = || seeds.next().expect("one marker chest seed");

    assert!(
        runtime
            .place(
                &mut world,
                &mut piece,
                &clip,
                &mut ScriptRandom::new([], []),
                &mut next_seed,
            )
            .unwrap()
    );
    assert_eq!(piece.position.y, 46);
    assert_eq!(world.spawns.len(), 2);
    assert!(world.spawns.iter().all(|spawn| spawn.persistent
        && spawn.finalize_structure_spawn
        && spawn.offer_with_passengers));
    assert!(
        world
            .spawns
            .iter()
            .all(|spawn| world.states[&spawn.position].block == "minecraft:water")
    );
    assert_eq!(
        world.installed_loot,
        [(
            pos(3, 47, 4),
            "minecraft:chests/underwater_ruin_small".into(),
            77,
        )]
    );
    let archaeology = world
        .loaded_nbt
        .iter()
        .filter(|(_, nbt)| {
            nbt.get("LootTable")
                == Some(&NbtValue::String(
                    "minecraft:archaeology/ocean_ruin_warm".into(),
                ))
        })
        .count();
    assert_eq!(archaeology, 5);
    assert!(seeds.next().is_none());
}

fn locked_names() -> Vec<String> {
    let mut names = (1..=8)
        .flat_map(|suffix| {
            ["brick", "cracked", "mossy", "warm"]
                .map(move |prefix| format!("minecraft:underwater_ruin/{prefix}_{suffix}"))
        })
        .collect::<Vec<_>>();
    for suffix in [1, 2, 3, 8] {
        names.extend(
            ["brick", "cracked", "mossy"]
                .map(|prefix| format!("minecraft:underwater_ruin/big_{prefix}_{suffix}")),
        );
    }
    names.extend((4..=7).map(|suffix| format!("minecraft:underwater_ruin/big_warm_{suffix}")));
    names
}

fn local_resource_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/mc-reference/26.2/client-classes");
    root.join("data/minecraft/structure/underwater_ruin")
        .is_dir()
        .then_some(root)
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    ocean_floor: i32,
    solid_y: i32,
    states: BTreeMap<BlockPos, StructureState>,
    loaded_nbt: Vec<(BlockPos, NbtCompound)>,
    installed_loot: Vec<(BlockPos, String, i64)>,
    spawns: Vec<OceanRuinDrownedSpawn>,
    spawn_success: bool,
}

impl PieceWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states.get(&position).cloned().unwrap_or_else(|| {
            if position.y > self.solid_y {
                StructureState::new("minecraft:water")
            } else {
                StructureState::new("minecraft:stone")
            }
        })
    }
    fn fluid_at(&mut self, position: BlockPos) -> FluidState {
        if position.y > self.solid_y {
            FluidState::Water
        } else {
            FluidState::Empty
        }
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
    fn height(&mut self, heightmap: Heightmap, _x: i32, _z: i32) -> i32 {
        assert_eq!(heightmap, Heightmap::OceanFloorWorldgen);
        self.ocean_floor
    }
    fn is_full_collision(&mut self, _position: BlockPos, _state: &StructureState) -> bool {
        true
    }
    fn positional_seed(&self, position: BlockPos) -> i64 {
        i64::from(position.x) * 31 + i64::from(position.y) * 17 + i64::from(position.z)
    }
    fn capped_seed(&self, template_origin: BlockPos) -> i64 {
        i64::from(template_origin.x) ^ i64::from(template_origin.y) ^ i64::from(template_origin.z)
    }
}

impl TemplateWorld for World {
    fn load_template_nbt(&mut self, position: BlockPos, nbt: NbtCompound) {
        self.loaded_nbt.push((position, nbt));
    }
    fn place_template_entity(&mut self, _entity: PlacedTemplateEntity, _finalize: bool) {
        panic!("locked ocean ruins have no template entities");
    }
}

impl OceanRuinWorld for World {
    fn minimum_y(&self) -> i32 {
        -64
    }
    fn sea_level(&self) -> i32 {
        63
    }
    fn spawn_ocean_ruin_drowned(&mut self, request: OceanRuinDrownedSpawn) -> bool {
        self.spawns.push(request);
        self.spawn_success
    }
}

struct ScriptRandom {
    integers: std::vec::IntoIter<u32>,
    floats: std::vec::IntoIter<f32>,
}

impl ScriptRandom {
    fn new(integers: impl IntoIterator<Item = u32>, floats: impl IntoIterator<Item = f32>) -> Self {
        Self {
            integers: integers.into_iter().collect::<Vec<_>>().into_iter(),
            floats: floats.into_iter().collect::<Vec<_>>().into_iter(),
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
        self.floats.next().unwrap_or_default()
    }
    fn next_f64(&mut self) -> f64 {
        0.0
    }
    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
