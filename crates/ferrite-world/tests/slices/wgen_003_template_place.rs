use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::nbt::{NbtCompound, NbtValue};
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::processor::{
    Heightmap, Processor, ProcessorSettings, ProcessorWorld, SettingsRandom, StructureState,
};
use ferrite_world::generation::structure::template::{
    StructureTemplate, TemplateBlock, TemplateEntity, TemplatePalette,
};
use ferrite_world::generation::structure::template_place::{
    PlacedTemplateEntity, TemplateMirror, TemplatePlaceSettings, TemplateRotation,
    TemplateTransform, TemplateWorld, data_markers, place_template,
};

#[test]
fn generic_template_transaction_orders_barrier_state_nbt_fluid_updates_and_entity() {
    let template = template();
    let transform = TemplateTransform {
        origin: pos(10, 5, 20),
        pivot: pos(0, 0, 0),
        mirror: TemplateMirror::None,
        rotation: TemplateRotation::Clockwise90,
    };
    let clip = BlockBox::new(pos(10, 5, 20), pos(10, 5, 21)).unwrap();
    let mut world = World {
        loot_containers: BTreeSet::from([pos(10, 5, 20)]),
        fluids: BTreeMap::from([(pos(10, 5, 21), FluidState::Water)]),
        ..World::default()
    };
    let seed_calls = Cell::new(0);
    let processors = [Processor::NoOp];
    let result = place_template(
        &mut world,
        &template,
        settings(transform, &clip, &processors),
        &mut ZeroRandom,
        || {
            seed_calls.set(seed_calls.get() + 1);
            99
        },
    )
    .unwrap();
    assert_eq!(result.attempted_blocks, 2);
    assert_eq!(result.written, [pos(10, 5, 20), pos(10, 5, 21)]);
    assert_eq!(result.placed_entities, 1);
    assert_eq!(seed_calls.get(), 1);
    assert_eq!(
        world
            .writes
            .iter()
            .map(|write| (write.0, write.1.block.as_str(), write.2))
            .collect::<Vec<_>>(),
        [
            (pos(10, 5, 20), "minecraft:barrier", 820),
            (pos(10, 5, 20), "minecraft:chest", 2),
            (pos(10, 5, 21), "minecraft:stone", 2),
        ]
    );
    assert_eq!(world.writes[1].1.properties["facing"], "east");
    assert_eq!(world.loaded_nbt[0].1["LootTableSeed"], NbtValue::Long(99));
    assert_eq!(world.reconciled.len(), 2);
    assert_eq!(world.reconciled[1].1, FluidState::Water);
    assert_eq!(world.finished, vec![result.written.clone()]);
    assert_eq!(world.entities[0].0.block_position, pos(10, 5, 21));
    assert_eq!(world.entities[0].0.position, [9.5, 5.0, 21.25]);
    assert!(world.entities[0].1);
}

#[test]
fn marker_filter_uses_raw_palette_transform_and_its_own_clip() {
    let template = template();
    let transform = TemplateTransform {
        origin: pos(10, 5, 20),
        pivot: pos(0, 0, 0),
        mirror: TemplateMirror::None,
        rotation: TemplateRotation::Clockwise90,
    };
    let full = BlockBox::new(pos(10, 5, 20), pos(10, 5, 22)).unwrap();
    assert_eq!(
        data_markers(&template, 0, transform, &full),
        [
            ferrite_world::generation::structure::template_place::DataMarker {
                position: pos(10, 5, 22),
                metadata: "treasure".into(),
            }
        ]
    );
    assert!(data_markers(&template, 0, transform, &BlockBox::point(pos(10, 5, 20))).is_empty());
}

#[test]
fn mirror_and_rotation_transform_directional_state_keys() {
    let transform = TemplateTransform {
        origin: pos(0, 0, 0),
        pivot: pos(0, 0, 0),
        mirror: TemplateMirror::FrontBack,
        rotation: TemplateRotation::Counterclockwise90,
    };
    assert_eq!(transform.position(pos(2, 3, 4)), pos(4, 3, 2));
    let mut state = StructureState::new("minecraft:redstone_wire");
    state.properties.insert("north".into(), "side".into());
    state.properties.insert("east".into(), "up".into());
    let transformed = transform.state(state);
    assert_eq!(transformed.properties["west"], "side");
    assert_eq!(transformed.properties["south"], "up");
}

fn template() -> StructureTemplate {
    let mut chest = StructureState::new("minecraft:chest");
    chest.properties.insert("facing".into(), "north".into());
    let states = vec![
        chest,
        StructureState::new("minecraft:stone"),
        StructureState::new("minecraft:structure_block"),
    ];
    let marker_nbt = NbtCompound::from([
        ("mode".into(), NbtValue::String("DATA".into())),
        ("metadata".into(), NbtValue::String("treasure".into())),
    ]);
    StructureTemplate {
        data_version: Some(4_699),
        size: [3, 1, 1],
        palettes: vec![TemplatePalette { states }],
        blocks: vec![
            TemplateBlock {
                position: pos(0, 0, 0),
                state_index: 0,
                nbt: Some(NbtCompound::new()),
            },
            TemplateBlock {
                position: pos(1, 0, 0),
                state_index: 1,
                nbt: None,
            },
            TemplateBlock {
                position: pos(2, 0, 0),
                state_index: 2,
                nbt: Some(marker_nbt),
            },
        ],
        entities: vec![TemplateEntity {
            block_position: pos(1, 0, 0),
            position: [1.25, 0.0, 0.5],
            nbt: NbtCompound::from([("id".into(), NbtValue::String("minecraft:pig".into()))]),
        }],
    }
}

fn settings<'a>(
    transform: TemplateTransform,
    clip: &'a BlockBox,
    processors: &'a [Processor],
) -> TemplatePlaceSettings<'a> {
    TemplatePlaceSettings {
        transform,
        clip,
        palette: 0,
        processors,
        processor_settings: ProcessorSettings {
            clip: Some(*clip),
            random: SettingsRandom::PositionDerived,
            keep_jigsaws: false,
        },
        reference_position: transform.origin,
        block_flags: 2,
        keep_liquids: true,
        known_shape: false,
        include_entities: true,
        finalize_entities: true,
    }
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    states: BTreeMap<BlockPos, StructureState>,
    fluids: BTreeMap<BlockPos, FluidState>,
    loot_containers: BTreeSet<BlockPos>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    loaded_nbt: Vec<(BlockPos, NbtCompound)>,
    reconciled: Vec<(BlockPos, FluidState)>,
    finished: Vec<Vec<BlockPos>>,
    entities: Vec<(PlacedTemplateEntity, bool)>,
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
        self.loot_containers.contains(&position)
    }

    fn install_loot(&mut self, _position: BlockPos, _table: &str, _seed: i64) {}
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

    fn positional_seed(&self, position: BlockPos) -> i64 {
        i64::from(position.x) ^ i64::from(position.z)
    }

    fn capped_seed(&self, _template_origin: BlockPos) -> i64 {
        0
    }
}

impl TemplateWorld for World {
    fn load_template_nbt(&mut self, position: BlockPos, nbt: NbtCompound) {
        self.loaded_nbt.push((position, nbt));
    }

    fn reconcile_template_fluid(
        &mut self,
        position: BlockPos,
        previous: FluidState,
        _placed: &StructureState,
    ) {
        self.reconciled.push((position, previous));
    }

    fn finish_template_updates(&mut self, positions: &[BlockPos], _known_shape: bool) {
        self.finished.push(positions.to_vec());
    }

    fn place_template_entity(&mut self, entity: PlacedTemplateEntity, finalize: bool) {
        self.entities.push((entity, finalize));
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
