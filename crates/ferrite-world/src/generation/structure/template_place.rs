//! Generic structure-template transform, processor, block-NBT, and entity transaction.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::nbt::{NbtCompound, NbtValue};
use crate::generation::structure::piece::{FluidState, PieceWorld};
use crate::generation::structure::processor::{
    Processor, ProcessorSettings, ProcessorWorld, StructureBlock, StructureState, process_blocks,
};
use crate::generation::structure::template::{StructureTemplate, TemplateEntity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateMirror {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateRotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, Copy)]
pub struct TemplateTransform {
    pub origin: BlockPos,
    pub pivot: BlockPos,
    pub mirror: TemplateMirror,
    pub rotation: TemplateRotation,
}

impl TemplateTransform {
    pub fn position(self, local: BlockPos) -> BlockPos {
        let mirrored = match self.mirror {
            TemplateMirror::None => local,
            TemplateMirror::LeftRight => BlockPos {
                z: self.pivot.z.wrapping_mul(2).wrapping_sub(local.z),
                ..local
            },
            TemplateMirror::FrontBack => BlockPos {
                x: self.pivot.x.wrapping_mul(2).wrapping_sub(local.x),
                ..local
            },
        };
        let dx = mirrored.x.wrapping_sub(self.pivot.x);
        let dz = mirrored.z.wrapping_sub(self.pivot.z);
        let (x, z) = match self.rotation {
            TemplateRotation::None => (mirrored.x, mirrored.z),
            TemplateRotation::Clockwise90 => {
                (self.pivot.x.wrapping_sub(dz), self.pivot.z.wrapping_add(dx))
            }
            TemplateRotation::Clockwise180 => {
                (self.pivot.x.wrapping_sub(dx), self.pivot.z.wrapping_sub(dz))
            }
            TemplateRotation::Counterclockwise90 => {
                (self.pivot.x.wrapping_add(dz), self.pivot.z.wrapping_sub(dx))
            }
        };
        BlockPos {
            x: self.origin.x.wrapping_add(x),
            y: self.origin.y.wrapping_add(local.y),
            z: self.origin.z.wrapping_add(z),
        }
    }

    pub fn fractional_position(self, local: [f64; 3]) -> [f64; 3] {
        let pivot_x = f64::from(self.pivot.x);
        let pivot_z = f64::from(self.pivot.z);
        let (mirrored_x, mirrored_z) = match self.mirror {
            TemplateMirror::None => (local[0], local[2]),
            TemplateMirror::LeftRight => (local[0], 2.0 * pivot_z - local[2]),
            TemplateMirror::FrontBack => (2.0 * pivot_x - local[0], local[2]),
        };
        let dx = mirrored_x - pivot_x;
        let dz = mirrored_z - pivot_z;
        let (x, z) = match self.rotation {
            TemplateRotation::None => (mirrored_x, mirrored_z),
            TemplateRotation::Clockwise90 => (pivot_x - dz, pivot_z + dx),
            TemplateRotation::Clockwise180 => (pivot_x - dx, pivot_z - dz),
            TemplateRotation::Counterclockwise90 => (pivot_x + dz, pivot_z - dx),
        };
        [
            f64::from(self.origin.x) + x,
            f64::from(self.origin.y) + local[1],
            f64::from(self.origin.z) + z,
        ]
    }

    pub fn state(self, mut state: StructureState) -> StructureState {
        if let Some(facing) = state.properties.get_mut("facing") {
            *facing = transform_facing(facing, self.mirror, self.rotation).into();
        }
        transform_directional_properties(&mut state, self.mirror, self.rotation);
        state
    }
}

pub trait TemplateWorld: PieceWorld + ProcessorWorld {
    fn load_template_nbt(&mut self, position: BlockPos, nbt: NbtCompound);

    fn reconcile_template_fluid(
        &mut self,
        _position: BlockPos,
        _previous: FluidState,
        _placed: &StructureState,
    ) {
    }

    fn finish_template_updates(&mut self, _positions: &[BlockPos], _known_shape: bool) {}

    fn place_template_entity(&mut self, entity: PlacedTemplateEntity, finalize: bool);
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlacedTemplateEntity {
    pub block_position: BlockPos,
    pub position: [f64; 3],
    pub yaw: Option<f32>,
    pub pitch: Option<f32>,
    pub nbt: NbtCompound,
}

#[derive(Debug, Clone, Copy)]
pub struct TemplatePlaceSettings<'a> {
    pub transform: TemplateTransform,
    pub clip: &'a BlockBox,
    pub palette: usize,
    pub processors: &'a [Processor],
    pub processor_settings: ProcessorSettings,
    pub reference_position: BlockPos,
    pub block_flags: u32,
    pub keep_liquids: bool,
    pub known_shape: bool,
    pub include_entities: bool,
    pub finalize_entities: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePlaceResult {
    pub written: Vec<BlockPos>,
    pub attempted_blocks: usize,
    pub placed_entities: usize,
}

pub fn place_template<W, R, F>(
    world: &mut W,
    template: &StructureTemplate,
    settings: TemplatePlaceSettings<'_>,
    random: &mut R,
    mut loot_seed: F,
) -> Option<TemplatePlaceResult>
where
    W: TemplateWorld,
    R: GenerationRandom,
    F: FnMut() -> i64,
{
    let palette = template.palettes.get(settings.palette)?;
    let cells = template
        .blocks
        .iter()
        .map(|block| {
            Some(StructureBlock {
                raw_position: block.position,
                position: settings.transform.position(block.position),
                state: settings
                    .transform
                    .state(palette.states.get(block.state_index)?.clone()),
                nbt: block.nbt.clone(),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let processed = process_blocks(
        world,
        settings.processors,
        &cells,
        settings.transform.origin,
        settings.reference_position,
        settings.processor_settings,
        random,
    );
    let mut written = Vec::new();
    let mut attempted_blocks = 0;
    for cell in processed.processed {
        if !settings.clip.contains(cell.position) {
            continue;
        }
        attempted_blocks += 1;
        let previous_fluid = PieceWorld::fluid_at(world, cell.position);
        if cell.nbt.is_some() {
            PieceWorld::set_state(
                world,
                cell.position,
                StructureState::new("minecraft:barrier"),
                820,
            );
        }
        if !PieceWorld::set_state(
            world,
            cell.position,
            cell.state.clone(),
            settings.block_flags,
        ) {
            continue;
        }
        if let Some(mut nbt) = cell.nbt {
            if PieceWorld::is_loot_container(world, cell.position) {
                nbt.insert("LootTableSeed".into(), NbtValue::Long(loot_seed()));
            }
            world.load_template_nbt(cell.position, nbt);
        }
        if settings.keep_liquids {
            world.reconcile_template_fluid(cell.position, previous_fluid, &cell.state);
        }
        written.push(cell.position);
    }
    world.finish_template_updates(&written, settings.known_shape);
    let mut placed_entities = 0;
    if settings.include_entities {
        for entity in &template.entities {
            let placed = transform_entity(entity, settings.transform);
            if settings.clip.contains(placed.block_position) {
                world.place_template_entity(placed, settings.finalize_entities);
                placed_entities += 1;
            }
        }
    }
    Some(TemplatePlaceResult {
        written,
        attempted_blocks,
        placed_entities,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMarker {
    pub position: BlockPos,
    pub metadata: String,
}

pub fn data_markers(
    template: &StructureTemplate,
    palette: usize,
    transform: TemplateTransform,
    clip: &BlockBox,
) -> Vec<DataMarker> {
    let Some(palette) = template.palettes.get(palette) else {
        return Vec::new();
    };
    template
        .blocks
        .iter()
        .filter_map(|block| {
            let state = palette.states.get(block.state_index)?;
            if state.block != "minecraft:structure_block" {
                return None;
            }
            let nbt = block.nbt.as_ref()?;
            if nbt.get("mode")?.as_str()? != "DATA" {
                return None;
            }
            let position = transform.position(block.position);
            clip.contains(position).then(|| DataMarker {
                position,
                metadata: nbt
                    .get("metadata")
                    .and_then(NbtValue::as_str)
                    .unwrap_or_default()
                    .into(),
            })
        })
        .collect()
}

fn transform_entity(entity: &TemplateEntity, transform: TemplateTransform) -> PlacedTemplateEntity {
    let position = transform.fractional_position(entity.position);
    let mut nbt = entity.nbt.clone();
    nbt.remove("UUID");
    nbt.insert(
        "Pos".into(),
        NbtValue::List(position.into_iter().map(NbtValue::Double).collect()),
    );
    let (yaw, pitch) = entity_rotation(&nbt, transform);
    if let (Some(yaw), Some(pitch)) = (yaw, pitch) {
        nbt.insert(
            "Rotation".into(),
            NbtValue::List(vec![NbtValue::Float(yaw), NbtValue::Float(pitch)]),
        );
    }
    PlacedTemplateEntity {
        block_position: transform.position(entity.block_position),
        position,
        yaw,
        pitch,
        nbt,
    }
}

fn entity_rotation(nbt: &NbtCompound, transform: TemplateTransform) -> (Option<f32>, Option<f32>) {
    let Some(rotation) = nbt.get("Rotation").and_then(NbtValue::as_list) else {
        return (None, None);
    };
    if rotation.len() != 2 {
        return (None, None);
    }
    let (Some(yaw), Some(pitch)) = (rotation[0].as_f64(), rotation[1].as_f64()) else {
        return (None, None);
    };
    let mirrored = match transform.mirror {
        TemplateMirror::None => yaw,
        TemplateMirror::LeftRight => 180.0 - yaw,
        TemplateMirror::FrontBack => -yaw,
    };
    let turns = match transform.rotation {
        TemplateRotation::None => 0.0,
        TemplateRotation::Clockwise90 => 90.0,
        TemplateRotation::Clockwise180 => 180.0,
        TemplateRotation::Counterclockwise90 => -90.0,
    };
    (Some((mirrored + turns) as f32), Some(pitch as f32))
}

fn transform_facing(facing: &str, mirror: TemplateMirror, rotation: TemplateRotation) -> &str {
    let mut index = match facing {
        "north" => 0_i32,
        "east" => 1,
        "south" => 2,
        "west" => 3,
        _ => return facing,
    };
    index = match mirror {
        TemplateMirror::None => index,
        TemplateMirror::LeftRight => (2 - index).rem_euclid(4),
        TemplateMirror::FrontBack => (-index).rem_euclid(4),
    };
    let turns = match rotation {
        TemplateRotation::None => 0,
        TemplateRotation::Clockwise90 => 1,
        TemplateRotation::Clockwise180 => 2,
        TemplateRotation::Counterclockwise90 => 3,
    };
    ["north", "east", "south", "west"][(index + turns).rem_euclid(4) as usize]
}

fn transform_directional_properties(
    state: &mut StructureState,
    mirror: TemplateMirror,
    rotation: TemplateRotation,
) {
    let properties = ["north", "east", "south", "west"]
        .map(|name| state.properties.remove(name).map(|value| (name, value)));
    for (name, value) in properties.into_iter().flatten() {
        state
            .properties
            .insert(transform_facing(name, mirror, rotation).into(), value);
    }
}
