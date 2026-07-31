//! Strict decoding of structure processor-list records and their block tags.

use std::collections::BTreeMap;

use serde_json::{Map, Value};
use thiserror::Error;

use crate::generation::structure::block_tags::{BlockTagError, BlockTagResolver};
use crate::generation::structure::nbt::{NbtCompound, NbtValue};
use crate::generation::structure::processor::{
    Axis, BlockPredicate, Heightmap, LimitProvider, NbtModifier, PositionPredicate, Processor,
    ProcessorRule, StructureState,
};
use crate::generation::worldgen_catalog::{WorldgenCatalog, WorldgenRecordKind};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProcessorAudit {
    pub lists: usize,
    pub top_level: usize,
    pub rules: usize,
    pub rule_processors: usize,
    pub protected: usize,
    pub block_rot: usize,
    pub capped: usize,
    pub input_always: usize,
    pub input_block: usize,
    pub input_state: usize,
    pub input_random_block: usize,
    pub input_tag: usize,
    pub location_always: usize,
    pub location_block: usize,
    pub position_always: usize,
    pub position_axis_linear: usize,
    pub modifier_passthrough: usize,
    pub modifier_append_loot: usize,
}

#[derive(Debug, Clone)]
pub struct ProcessorCatalog {
    lists: BTreeMap<String, Vec<Processor>>,
    audit: ProcessorAudit,
}

impl ProcessorCatalog {
    pub fn empty() -> Self {
        Self {
            lists: BTreeMap::new(),
            audit: ProcessorAudit::default(),
        }
    }

    pub fn decode(
        catalog: WorldgenCatalog<'_>,
        tags: &mut impl BlockTagResolver,
    ) -> Result<Self, ProcessorCatalogError> {
        let mut lists = BTreeMap::new();
        let mut audit = ProcessorAudit::default();
        for entry in catalog.entries(WorldgenRecordKind::ProcessorList) {
            let resource = entry.persistent_id().resource();
            let Some(path) = resource.path().strip_prefix("processor_list/") else {
                return Err(ProcessorCatalogError::ListId(resource.to_string()));
            };
            let name = format!("{}:{path}", resource.namespace());
            let root = object(entry.value(), &name, "root")?;
            let values = array_field(root, &name, "processors")?;
            let processors = values
                .iter()
                .map(|value| decode_processor(value, &name, tags, &mut audit, true))
                .collect::<Result<Vec<_>, _>>()?;
            if lists.insert(name.clone(), processors).is_some() {
                return Err(ProcessorCatalogError::Duplicate(name));
            }
        }
        audit.lists = lists.len();
        Ok(Self { lists, audit })
    }

    pub fn get(&self, name: &str) -> Option<&[Processor]> {
        self.lists.get(name).map(Vec::as_slice)
    }

    pub fn lists(&self) -> &BTreeMap<String, Vec<Processor>> {
        &self.lists
    }

    pub const fn audit(&self) -> ProcessorAudit {
        self.audit
    }
}

fn decode_processor(
    value: &Value,
    list: &str,
    tags: &mut impl BlockTagResolver,
    audit: &mut ProcessorAudit,
    top_level: bool,
) -> Result<Processor, ProcessorCatalogError> {
    let processor = object(value, list, "processor")?;
    let kind = string_field(processor, list, "processor_type")?;
    if top_level {
        audit.top_level += 1;
    }
    match kind {
        "minecraft:nop" => Ok(Processor::NoOp),
        "minecraft:block_ignore" => {
            let blocks = array_field(processor, list, "blocks")?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| invalid(list, "blocks", "an array of block IDs"))
                })
                .collect::<Result<_, _>>()?;
            Ok(Processor::BlockIgnore(blocks))
        }
        "minecraft:protected_blocks" => {
            if top_level {
                audit.protected += 1;
            }
            let tag = string_field(processor, list, "value")?;
            Ok(Processor::ProtectedBlocks(tags.resolve_block_tag(tag)?))
        }
        "minecraft:block_rot" => {
            if top_level {
                audit.block_rot += 1;
            }
            let integrity = float_field(processor, list, "integrity")?;
            let rottable = processor
                .get("rottable_blocks")
                .map(|value| {
                    let tag = value
                        .as_str()
                        .ok_or_else(|| invalid(list, "rottable_blocks", "a block tag"))?;
                    Ok::<_, ProcessorCatalogError>(tags.resolve_block_tag(tag)?)
                })
                .transpose()?;
            Ok(Processor::BlockRot {
                integrity,
                rottable,
            })
        }
        "minecraft:gravity" => Ok(Processor::Gravity {
            heightmap: decode_heightmap(string_field(processor, list, "heightmap")?, list)?,
            offset: optional_i32(processor, list, "offset")?.unwrap_or(0),
        }),
        "minecraft:lava_submerged_block" => Ok(Processor::LavaSubmerged),
        "minecraft:jigsaw_replacement" => Ok(Processor::JigsawReplacement),
        "minecraft:blackstone_replace" => Ok(Processor::BlackstoneReplace),
        "minecraft:block_age" => Ok(Processor::BlockAge {
            mossiness: float_field(processor, list, "mossiness")?,
        }),
        "minecraft:rule" => {
            if top_level {
                audit.rule_processors += 1;
            }
            let rules = array_field(processor, list, "rules")?
                .iter()
                .map(|rule| decode_rule(rule, list, tags, audit))
                .collect::<Result<_, _>>()?;
            Ok(Processor::Rule(rules))
        }
        "minecraft:capped" => {
            if top_level {
                audit.capped += 1;
            }
            let delegate = required(processor, list, "delegate")?;
            let limit = decode_limit(required(processor, list, "limit")?, list)?;
            Ok(Processor::Capped {
                delegate: Box::new(decode_processor(delegate, list, tags, audit, false)?),
                limit,
            })
        }
        _ => Err(ProcessorCatalogError::ProcessorType {
            list: list.to_owned(),
            kind: kind.to_owned(),
        }),
    }
}

fn decode_rule(
    value: &Value,
    list: &str,
    tags: &mut impl BlockTagResolver,
    audit: &mut ProcessorAudit,
) -> Result<ProcessorRule, ProcessorCatalogError> {
    let rule = object(value, list, "rule")?;
    let input = decode_block_predicate(required(rule, list, "input_predicate")?, list, tags)?;
    count_input(&input, audit);
    let location = rule
        .get("location_predicate")
        .map(|value| decode_block_predicate(value, list, tags))
        .transpose()?
        .unwrap_or(BlockPredicate::Always);
    count_location(&location, audit);
    let position = rule
        .get("position_predicate")
        .map(|value| decode_position_predicate(value, list))
        .transpose()?
        .unwrap_or(PositionPredicate::Always);
    match position {
        PositionPredicate::Always => audit.position_always += 1,
        PositionPredicate::AxisAlignedLinear { .. } => audit.position_axis_linear += 1,
        PositionPredicate::Linear { .. } => {}
    }
    let modifier = rule
        .get("block_entity_modifier")
        .map(|value| decode_modifier(value, list))
        .transpose()?
        .unwrap_or(NbtModifier::Passthrough);
    match modifier {
        NbtModifier::Passthrough => audit.modifier_passthrough += 1,
        NbtModifier::AppendLoot(_) => audit.modifier_append_loot += 1,
        NbtModifier::Clear | NbtModifier::AppendStatic(_) => {}
    }
    audit.rules += 1;
    Ok(ProcessorRule {
        input,
        location,
        position,
        output: decode_state(required(rule, list, "output_state")?, list)?,
        modifier,
    })
}

fn decode_block_predicate(
    value: &Value,
    list: &str,
    tags: &mut impl BlockTagResolver,
) -> Result<BlockPredicate, ProcessorCatalogError> {
    let predicate = object(value, list, "predicate")?;
    let kind = string_field(predicate, list, "predicate_type")?;
    match kind {
        "minecraft:always_true" => Ok(BlockPredicate::Always),
        "minecraft:block_match" => Ok(BlockPredicate::Block(
            string_field(predicate, list, "block")?.to_owned(),
        )),
        "minecraft:blockstate_match" => Ok(BlockPredicate::State(decode_state(
            required(predicate, list, "block_state")?,
            list,
        )?)),
        "minecraft:random_block_match" => Ok(BlockPredicate::RandomBlock {
            block: string_field(predicate, list, "block")?.to_owned(),
            probability: float_field(predicate, list, "probability")?,
        }),
        "minecraft:random_blockstate_match" => Ok(BlockPredicate::RandomState {
            state: decode_state(required(predicate, list, "block_state")?, list)?,
            probability: float_field(predicate, list, "probability")?,
        }),
        "minecraft:tag_match" => Ok(BlockPredicate::Tag(
            tags.resolve_block_tag(string_field(predicate, list, "tag")?)?,
        )),
        _ => Err(ProcessorCatalogError::PredicateType {
            list: list.to_owned(),
            kind: kind.to_owned(),
        }),
    }
}

fn decode_position_predicate(
    value: &Value,
    list: &str,
) -> Result<PositionPredicate, ProcessorCatalogError> {
    let predicate = object(value, list, "position predicate")?;
    let kind = string_field(predicate, list, "predicate_type")?;
    if kind == "minecraft:always_true" {
        return Ok(PositionPredicate::Always);
    }
    let minimum_distance = optional_i32(predicate, list, "min_dist")?.unwrap_or(0);
    let maximum_distance = optional_i32(predicate, list, "max_dist")?.unwrap_or(0);
    let minimum_chance = optional_f32(predicate, list, "min_chance")?.unwrap_or(0.0);
    let maximum_chance = optional_f32(predicate, list, "max_chance")?.unwrap_or(0.0);
    match kind {
        "minecraft:linear_pos" => Ok(PositionPredicate::Linear {
            minimum_distance,
            maximum_distance,
            minimum_chance,
            maximum_chance,
        }),
        "minecraft:axis_aligned_linear_pos" => Ok(PositionPredicate::AxisAlignedLinear {
            axis: decode_axis(
                predicate.get("axis").and_then(Value::as_str).unwrap_or("y"),
                list,
            )?,
            minimum_distance,
            maximum_distance,
            minimum_chance,
            maximum_chance,
        }),
        _ => Err(ProcessorCatalogError::PositionType {
            list: list.to_owned(),
            kind: kind.to_owned(),
        }),
    }
}

fn decode_modifier(value: &Value, list: &str) -> Result<NbtModifier, ProcessorCatalogError> {
    let modifier = object(value, list, "block entity modifier")?;
    match string_field(modifier, list, "type")? {
        "minecraft:passthrough" => Ok(NbtModifier::Passthrough),
        "minecraft:clear" => Ok(NbtModifier::Clear),
        "minecraft:append_static" => {
            let data = object(required(modifier, list, "data")?, list, "modifier data")?;
            Ok(NbtModifier::AppendStatic(json_compound(data, list)?))
        }
        "minecraft:append_loot" => Ok(NbtModifier::AppendLoot(
            string_field(modifier, list, "loot_table")?.to_owned(),
        )),
        kind => Err(ProcessorCatalogError::ModifierType {
            list: list.to_owned(),
            kind: kind.to_owned(),
        }),
    }
}

fn decode_state(value: &Value, list: &str) -> Result<StructureState, ProcessorCatalogError> {
    let value = object(value, list, "block state")?;
    let mut state = StructureState::new(string_field(value, list, "Name")?);
    if let Some(properties) = value.get("Properties") {
        for (name, value) in object(properties, list, "Properties")? {
            let value = value
                .as_str()
                .ok_or_else(|| invalid(list, "Properties", "string-valued properties"))?;
            state.properties.insert(name.clone(), value.to_owned());
        }
    }
    Ok(state)
}

fn decode_limit(value: &Value, list: &str) -> Result<LimitProvider, ProcessorCatalogError> {
    if let Some(value) = value.as_u64().and_then(|value| u32::try_from(value).ok()) {
        return Ok(LimitProvider::Constant(value));
    }
    let provider = object(value, list, "limit")?;
    let kind = string_field(provider, list, "type")?;
    let body = provider
        .get("value")
        .and_then(Value::as_object)
        .unwrap_or(provider);
    match kind {
        "minecraft:constant" => Ok(LimitProvider::Constant(unsigned_field(
            body, list, "value",
        )?)),
        "minecraft:uniform" => Ok(LimitProvider::Uniform {
            minimum: unsigned_field(body, list, "min_inclusive")?,
            maximum: unsigned_field(body, list, "max_inclusive")?,
        }),
        _ => Err(invalid(
            list,
            "limit.type",
            "minecraft:constant or minecraft:uniform",
        )),
    }
}

fn json_compound(
    value: &Map<String, Value>,
    list: &str,
) -> Result<NbtCompound, ProcessorCatalogError> {
    value
        .iter()
        .map(|(name, value)| Ok((name.clone(), json_nbt(value, list)?)))
        .collect()
}

fn json_nbt(value: &Value, list: &str) -> Result<NbtValue, ProcessorCatalogError> {
    match value {
        Value::Bool(value) => Ok(NbtValue::Byte(i8::from(*value))),
        Value::Number(value) if value.is_i64() => value
            .as_i64()
            .map(NbtValue::Long)
            .ok_or_else(|| invalid(list, "modifier.data", "NBT-compatible JSON")),
        Value::Number(value) => value
            .as_f64()
            .map(NbtValue::Double)
            .ok_or_else(|| invalid(list, "modifier.data", "NBT-compatible JSON")),
        Value::String(value) => Ok(NbtValue::String(value.clone())),
        Value::Array(values) => values
            .iter()
            .map(|value| json_nbt(value, list))
            .collect::<Result<Vec<_>, _>>()
            .map(NbtValue::List),
        Value::Object(value) => json_compound(value, list).map(NbtValue::Compound),
        Value::Null => Err(invalid(list, "modifier.data", "NBT-compatible JSON")),
    }
}

fn count_input(predicate: &BlockPredicate, audit: &mut ProcessorAudit) {
    match predicate {
        BlockPredicate::Always => audit.input_always += 1,
        BlockPredicate::Block(_) => audit.input_block += 1,
        BlockPredicate::State(_) | BlockPredicate::RandomState { .. } => audit.input_state += 1,
        BlockPredicate::RandomBlock { .. } => audit.input_random_block += 1,
        BlockPredicate::Tag(_) => audit.input_tag += 1,
    }
}

fn count_location(predicate: &BlockPredicate, audit: &mut ProcessorAudit) {
    match predicate {
        BlockPredicate::Always => audit.location_always += 1,
        BlockPredicate::Block(_) => audit.location_block += 1,
        BlockPredicate::State(_)
        | BlockPredicate::RandomBlock { .. }
        | BlockPredicate::RandomState { .. }
        | BlockPredicate::Tag(_) => {}
    }
}

fn decode_heightmap(value: &str, list: &str) -> Result<Heightmap, ProcessorCatalogError> {
    match value {
        "WORLD_SURFACE_WG" => Ok(Heightmap::WorldSurfaceWorldgen),
        "OCEAN_FLOOR_WG" => Ok(Heightmap::OceanFloorWorldgen),
        _ => Err(invalid(
            list,
            "heightmap",
            "WORLD_SURFACE_WG or OCEAN_FLOOR_WG",
        )),
    }
}

fn decode_axis(value: &str, list: &str) -> Result<Axis, ProcessorCatalogError> {
    match value {
        "x" => Ok(Axis::X),
        "y" => Ok(Axis::Y),
        "z" => Ok(Axis::Z),
        _ => Err(invalid(list, "axis", "x, y, or z")),
    }
}

fn required<'a>(
    value: &'a Map<String, Value>,
    list: &str,
    field: &str,
) -> Result<&'a Value, ProcessorCatalogError> {
    value
        .get(field)
        .ok_or_else(|| invalid(list, field, "a present value"))
}

fn string_field<'a>(
    value: &'a Map<String, Value>,
    list: &str,
    field: &str,
) -> Result<&'a str, ProcessorCatalogError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(list, field, "a string"))
}

fn array_field<'a>(
    value: &'a Map<String, Value>,
    list: &str,
    field: &str,
) -> Result<&'a [Value], ProcessorCatalogError> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| invalid(list, field, "an array"))
}

fn object<'a>(
    value: &'a Value,
    list: &str,
    field: &str,
) -> Result<&'a Map<String, Value>, ProcessorCatalogError> {
    value
        .as_object()
        .ok_or_else(|| invalid(list, field, "an object"))
}

fn float_field(
    value: &Map<String, Value>,
    list: &str,
    field: &str,
) -> Result<f32, ProcessorCatalogError> {
    optional_f32(value, list, field)?.ok_or_else(|| invalid(list, field, "a number"))
}

fn optional_f32(
    value: &Map<String, Value>,
    list: &str,
    field: &str,
) -> Result<Option<f32>, ProcessorCatalogError> {
    value
        .get(field)
        .map(|value| {
            value
                .as_f64()
                .map(|value| value as f32)
                .ok_or_else(|| invalid(list, field, "a number"))
        })
        .transpose()
}

fn optional_i32(
    value: &Map<String, Value>,
    list: &str,
    field: &str,
) -> Result<Option<i32>, ProcessorCatalogError> {
    value
        .get(field)
        .map(|value| {
            value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or_else(|| invalid(list, field, "a 32-bit integer"))
        })
        .transpose()
}

fn unsigned_field(
    value: &Map<String, Value>,
    list: &str,
    field: &str,
) -> Result<u32, ProcessorCatalogError> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| invalid(list, field, "a nonnegative 32-bit integer"))
}

fn invalid(list: &str, field: &str, expected: &'static str) -> ProcessorCatalogError {
    ProcessorCatalogError::Invalid {
        list: list.to_owned(),
        field: field.to_owned(),
        expected,
    }
}

#[derive(Debug, Error)]
pub enum ProcessorCatalogError {
    #[error("invalid processor-list persistent ID {0}")]
    ListId(String),
    #[error("processor list {0} is duplicated")]
    Duplicate(String),
    #[error("processor list {list} field {field} must be {expected}")]
    Invalid {
        list: String,
        field: String,
        expected: &'static str,
    },
    #[error("processor list {list} has unsupported processor type {kind}")]
    ProcessorType { list: String, kind: String },
    #[error("processor list {list} has unsupported predicate type {kind}")]
    PredicateType { list: String, kind: String },
    #[error("processor list {list} has unsupported position predicate {kind}")]
    PositionType { list: String, kind: String },
    #[error("processor list {list} has unsupported block-entity modifier {kind}")]
    ModifierType { list: String, kind: String },
    #[error(transparent)]
    Tag(#[from] BlockTagError),
}
