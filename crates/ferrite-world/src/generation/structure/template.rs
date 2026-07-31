//! Decoded structure-template palettes, sparse blocks, and entity payloads.

use std::collections::BTreeSet;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::structure::nbt::{
    NbtCompound, NbtDecodeError, NbtValue, decode_gzip_compound,
};
use crate::generation::structure::processor::StructureState;

#[derive(Debug, Clone, PartialEq)]
pub struct StructureTemplate {
    pub data_version: Option<i32>,
    pub size: [i32; 3],
    pub palettes: Vec<TemplatePalette>,
    pub blocks: Vec<TemplateBlock>,
    pub entities: Vec<TemplateEntity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePalette {
    pub states: Vec<StructureState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateBlock {
    pub position: BlockPos,
    pub state_index: usize,
    pub nbt: Option<NbtCompound>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateEntity {
    pub block_position: BlockPos,
    pub position: [f64; 3],
    pub nbt: NbtCompound,
}

impl StructureTemplate {
    pub fn empty() -> Self {
        Self {
            data_version: None,
            size: [0; 3],
            palettes: Vec::new(),
            blocks: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn decode_gzip(bytes: &[u8]) -> Result<Self, TemplateDecodeError> {
        Self::decode(decode_gzip_compound(bytes)?)
    }

    pub fn decode(root: NbtCompound) -> Result<Self, TemplateDecodeError> {
        let size = integer_triplet(field(&root, "size")?, "size")?;
        if size.into_iter().any(|axis| axis < 0) {
            return Err(TemplateDecodeError::NegativeSize(size));
        }
        let palettes = decode_palettes(&root)?;
        if palettes.is_empty() {
            return Err(TemplateDecodeError::EmptyPalettes);
        }
        let state_count = palettes[0].states.len();
        if palettes
            .iter()
            .any(|palette| palette.states.len() != state_count)
        {
            return Err(TemplateDecodeError::PaletteLength);
        }
        let blocks = list(field(&root, "blocks")?, "blocks")?
            .iter()
            .map(|value| decode_block(value, state_count))
            .collect::<Result<Vec<_>, _>>()?;
        let entities = root
            .get("entities")
            .map(|value| {
                list(value, "entities")?
                    .iter()
                    .map(decode_entity)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            data_version: root.get("DataVersion").and_then(NbtValue::as_i32),
            size,
            palettes,
            blocks,
            entities,
        })
    }

    pub fn state(&self, palette: usize, block: &TemplateBlock) -> Option<&StructureState> {
        self.palettes.get(palette)?.states.get(block.state_index)
    }

    pub fn duplicate_positions(&self) -> BTreeSet<BlockPos> {
        let mut seen = BTreeSet::new();
        let mut duplicates = BTreeSet::new();
        for block in &self.blocks {
            if !seen.insert(block.position) {
                duplicates.insert(block.position);
            }
        }
        duplicates
    }

    pub fn volume(&self) -> usize {
        self.size
            .into_iter()
            .map(|axis| usize::try_from(axis).expect("decoded size is nonnegative"))
            .product()
    }
}

fn decode_palettes(root: &NbtCompound) -> Result<Vec<TemplatePalette>, TemplateDecodeError> {
    if let Some(value) = root.get("palettes") {
        return list(value, "palettes")?
            .iter()
            .map(|palette| decode_palette(list(palette, "palette")?))
            .collect();
    }
    Ok(vec![decode_palette(list(
        field(root, "palette")?,
        "palette",
    )?)?])
}

fn decode_palette(values: &[NbtValue]) -> Result<TemplatePalette, TemplateDecodeError> {
    let states = values
        .iter()
        .map(|value| {
            let state = compound(value, "palette state")?;
            let name = string(field(state, "Name")?, "Name")?;
            let mut decoded = StructureState::new(name);
            if let Some(properties) = state.get("Properties") {
                for (key, value) in compound(properties, "Properties")? {
                    decoded
                        .properties
                        .insert(key.clone(), string(value, "property")?.into());
                }
            }
            Ok(decoded)
        })
        .collect::<Result<Vec<_>, TemplateDecodeError>>()?;
    Ok(TemplatePalette { states })
}

fn decode_block(
    value: &NbtValue,
    state_count: usize,
) -> Result<TemplateBlock, TemplateDecodeError> {
    let block = compound(value, "block")?;
    let position = integer_triplet(field(block, "pos")?, "pos")?;
    let state = field(block, "state")?
        .as_i32()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(TemplateDecodeError::Integer("state"))?;
    if state >= state_count {
        return Err(TemplateDecodeError::StateIndex {
            index: state,
            states: state_count,
        });
    }
    Ok(TemplateBlock {
        position: BlockPos {
            x: position[0],
            y: position[1],
            z: position[2],
        },
        state_index: state,
        nbt: block
            .get("nbt")
            .map(|value| compound(value, "block nbt").cloned())
            .transpose()?,
    })
}

fn decode_entity(value: &NbtValue) -> Result<TemplateEntity, TemplateDecodeError> {
    let entity = compound(value, "entity")?;
    let block_position = integer_triplet(field(entity, "blockPos")?, "blockPos")?;
    let coordinates = list(field(entity, "pos")?, "pos")?;
    if coordinates.len() != 3 {
        return Err(TemplateDecodeError::Triplet("pos"));
    }
    let mut position = [0.0; 3];
    for (index, value) in coordinates.iter().enumerate() {
        position[index] = value.as_f64().ok_or(TemplateDecodeError::Number("pos"))?;
    }
    Ok(TemplateEntity {
        block_position: BlockPos {
            x: block_position[0],
            y: block_position[1],
            z: block_position[2],
        },
        position,
        nbt: compound(field(entity, "nbt")?, "entity nbt")?.clone(),
    })
}

fn integer_triplet(value: &NbtValue, field: &'static str) -> Result<[i32; 3], TemplateDecodeError> {
    let values = list(value, field)?;
    if values.len() != 3 {
        return Err(TemplateDecodeError::Triplet(field));
    }
    let mut result = [0; 3];
    for (index, value) in values.iter().enumerate() {
        result[index] = value.as_i32().ok_or(TemplateDecodeError::Integer(field))?;
    }
    Ok(result)
}

fn field<'a>(
    compound: &'a NbtCompound,
    name: &'static str,
) -> Result<&'a NbtValue, TemplateDecodeError> {
    compound.get(name).ok_or(TemplateDecodeError::Missing(name))
}

fn list<'a>(
    value: &'a NbtValue,
    field: &'static str,
) -> Result<&'a [NbtValue], TemplateDecodeError> {
    value.as_list().ok_or(TemplateDecodeError::List(field))
}

fn compound<'a>(
    value: &'a NbtValue,
    field: &'static str,
) -> Result<&'a NbtCompound, TemplateDecodeError> {
    value
        .as_compound()
        .ok_or(TemplateDecodeError::Compound(field))
}

fn string<'a>(value: &'a NbtValue, field: &'static str) -> Result<&'a str, TemplateDecodeError> {
    value.as_str().ok_or(TemplateDecodeError::String(field))
}

#[derive(Debug, Error)]
pub enum TemplateDecodeError {
    #[error(transparent)]
    Nbt(#[from] NbtDecodeError),
    #[error("template is missing {0}")]
    Missing(&'static str),
    #[error("template field {0} must be a list")]
    List(&'static str),
    #[error("template field {0} must be a compound")]
    Compound(&'static str),
    #[error("template field {0} must be a string")]
    String(&'static str),
    #[error("template field {0} must be an integer")]
    Integer(&'static str),
    #[error("template field {0} must be numeric")]
    Number(&'static str),
    #[error("template field {0} must contain exactly three values")]
    Triplet(&'static str),
    #[error("template has negative size {0:?}")]
    NegativeSize([i32; 3]),
    #[error("template has no palettes")]
    EmptyPalettes,
    #[error("template palettes have different lengths")]
    PaletteLength,
    #[error("template state index {index} is outside palette length {states}")]
    StateIndex { index: usize, states: usize },
}
