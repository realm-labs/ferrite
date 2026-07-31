use std::collections::BTreeSet;
use std::fs;
use std::num::NonZeroU32;
use std::path::PathBuf;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::resource::ResourceId;
use ferrite_registry::bundle::ContentBundle;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::block_tags::FileBlockTagResolver;
use ferrite_world::generation::structure::jigsaw::{
    ElementKind, PoolElement, Projection, Rotation,
};
use ferrite_world::generation::structure::nbt::NbtCompound;
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::pool_catalog::TemplatePoolCatalog;
use ferrite_world::generation::structure::pool_place::{
    PoolElementWorld, PoolPlacementRuntime, PoolPlacementSettings, positive_box_transform,
};
use ferrite_world::generation::structure::processor::{
    Heightmap, Processor, ProcessorWorld, StructureState,
};
use ferrite_world::generation::structure::processor_catalog::ProcessorCatalog;
use ferrite_world::generation::structure::template_manager::{
    FileTemplateSource, TemplateManager, TemplateSource, TemplateSourceError,
};
use ferrite_world::generation::structure::template_place::{PlacedTemplateEntity, TemplateWorld};
use ferrite_world::generation::worldgen_catalog::WorldgenCatalog;

#[test]
fn positive_box_transform_matches_graph_connector_rotation() {
    let origin = pos(50, 7, -20);
    let size = [5, 3, 9];
    for rotation in Rotation::ALL {
        let transform = positive_box_transform(origin, size, rotation);
        for local in [pos(0, 0, 0), pos(4, 2, 8), pos(2, 1, 3)] {
            let rotated = rotation.rotate_local(local, size);
            assert_eq!(
                transform.position(local),
                pos(
                    origin.x + rotated.x,
                    origin.y + rotated.y,
                    origin.z + rotated.z,
                )
            );
        }
    }
}

#[test]
fn feature_and_list_elements_ignore_clip_and_short_circuit_in_order() {
    let mut templates = TemplateManager::new(MissingSource);
    let processors = ProcessorCatalog::empty();
    let mut runtime = PoolPlacementRuntime {
        templates: &mut templates,
        processors: &processors,
    };
    let children = vec![feature("first"), feature("stop"), feature("unreachable")];
    let list = PoolElement {
        kind: ElementKind::List(children),
        projection: Projection::Rigid,
        size: [0; 3],
        connectors: Vec::new(),
        ground_level_delta: 0,
        processor_list: None,
    };
    let clip = BlockBox::point(pos(0, 0, 0));
    let settings = PoolPlacementSettings {
        origin: pos(100, 5, 100),
        rotation: Rotation::None,
        clip: &clip,
        reference_position: pos(0, 0, 0),
        keep_jigsaws: false,
        keep_liquids: true,
    };
    let mut world = World::default();
    let mut random = ZeroRandom;
    let mut seed = || 0;

    assert!(
        !runtime
            .place(&mut world, &list, settings, &mut random, &mut seed)
            .unwrap()
    );
    assert_eq!(world.features, ["first", "stop"]);
    assert!(
        runtime
            .place(
                &mut world,
                &PoolElement::empty(),
                settings,
                &mut random,
                &mut seed,
            )
            .unwrap()
    );
}

#[test]
fn processor_chain_orders_legacy_filter_after_projection() {
    let mut templates = TemplateManager::new(MissingSource);
    let processors = ProcessorCatalog::empty();
    let runtime = PoolPlacementRuntime {
        templates: &mut templates,
        processors: &processors,
    };
    let element = PoolElement {
        kind: ElementKind::Single {
            template: "minecraft:test".into(),
            legacy: true,
        },
        projection: Projection::TerrainMatching,
        size: [1; 3],
        connectors: Vec::new(),
        ground_level_delta: 1,
        processor_list: None,
    };
    let chain = runtime.processor_chain(&element, true).unwrap();
    assert!(matches!(chain[0], Processor::BlockIgnore(_)));
    assert!(matches!(chain[1], Processor::JigsawReplacement));
    assert!(matches!(chain[2], Processor::Gravity { offset: -1, .. }));
    let Processor::BlockIgnore(last) = &chain[3] else {
        panic!("legacy filter must be last");
    };
    assert_eq!(
        last,
        &BTreeSet::from([
            "minecraft:air".to_owned(),
            "minecraft:structure_block".to_owned(),
        ])
    );
}

#[test]
fn official_outpost_virtual_plate_runs_end_to_end_without_writing_air() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let Some(root) = local_resource_root() else {
        return;
    };
    let worldgen = WorldgenCatalog::from_bundle(&bundle).unwrap();
    let mut templates = TemplateManager::new(FileTemplateSource::new(&root));
    let pools = TemplatePoolCatalog::decode(worldgen, &mut templates).unwrap();
    let mut tags = FileBlockTagResolver::new(&root);
    let processors = ProcessorCatalog::decode(worldgen, &mut tags).unwrap();
    let mut runtime = PoolPlacementRuntime {
        templates: &mut templates,
        processors: &processors,
    };
    let element = &pools.pools()["minecraft:pillager_outpost/base_plates"].expanded()[0];
    assert!(matches!(
        element.kind,
        ElementKind::Single { legacy: true, .. }
    ));
    let clip = element
        .box_at(pos(0, 0, 0), Rotation::None)
        .expect("real base plate has a box");
    let mut world = World::default();
    let mut random = ZeroRandom;
    let mut seed = || panic!("virtual plate has no loot container");

    assert!(
        runtime
            .place(
                &mut world,
                element,
                PoolPlacementSettings {
                    origin: pos(0, 0, 0),
                    rotation: Rotation::None,
                    clip: &clip,
                    reference_position: pos(0, 0, 0),
                    keep_jigsaws: false,
                    keep_liquids: true,
                },
                &mut random,
                &mut seed,
            )
            .unwrap()
    );
    assert_eq!(world.writes, 0);
}

fn feature(name: &str) -> PoolElement {
    PoolElement {
        kind: ElementKind::Feature { name: name.into() },
        projection: Projection::Rigid,
        size: [0; 3],
        connectors: Vec::new(),
        ground_level_delta: 0,
        processor_list: None,
    }
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

struct MissingSource;

impl TemplateSource for MissingSource {
    fn load_template(&self, _id: &ResourceId) -> Result<Option<Vec<u8>>, TemplateSourceError> {
        Ok(None)
    }
}

#[derive(Default)]
struct World {
    features: Vec<String>,
    writes: usize,
}

impl PieceWorld for World {
    fn state_at(&mut self, _position: BlockPos) -> StructureState {
        StructureState::new("minecraft:air")
    }

    fn fluid_at(&mut self, _position: BlockPos) -> FluidState {
        FluidState::Empty
    }

    fn set_state(&mut self, _position: BlockPos, _state: StructureState, _flags: u32) -> bool {
        self.writes += 1;
        true
    }

    fn schedule_fluid_tick(&mut self, _position: BlockPos, _fluid: FluidState, _delay: u32) {}
    fn mark_shape_postprocessing(&mut self, _position: BlockPos) {}
    fn solid_render(&mut self, _position: BlockPos) -> bool {
        false
    }
    fn is_loot_container(&mut self, _position: BlockPos) -> bool {
        false
    }
    fn install_loot(&mut self, _position: BlockPos, _table: &str, _seed: i64) {}
}

fn local_bundle() -> Option<ContentBundle> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/ferrite-content/26.2/content-bundle.json");
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn local_resource_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/mc-reference/26.2/client-classes");
    root.join("data/minecraft/structure")
        .is_dir()
        .then_some(root)
}

impl ProcessorWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        PieceWorld::state_at(self, position)
    }
    fn height(&mut self, _heightmap: Heightmap, _x: i32, _z: i32) -> i32 {
        0
    }
    fn is_full_collision(&mut self, _position: BlockPos, _state: &StructureState) -> bool {
        true
    }
    fn positional_seed(&self, _position: BlockPos) -> i64 {
        0
    }
    fn capped_seed(&self, _template_origin: BlockPos) -> i64 {
        0
    }
}

impl TemplateWorld for World {
    fn load_template_nbt(&mut self, _position: BlockPos, _nbt: NbtCompound) {}
    fn place_template_entity(&mut self, _entity: PlacedTemplateEntity, _finalize: bool) {}
}

impl PoolElementWorld for World {
    fn place_pool_feature(
        &mut self,
        name: &str,
        _position: BlockPos,
        _random: &mut dyn GenerationRandom,
    ) -> bool {
        self.features.push(name.to_owned());
        name != "stop"
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
