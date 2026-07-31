//! Strict decoding of the locked template-pool registry into executable elements.

use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::generation::structure::jigsaw::{
    Connector, ElementKind, Joint, PoolElement, Projection, TemplatePool,
};
use crate::generation::structure::nbt::NbtCompound;
use crate::generation::structure::template::StructureTemplate;
use crate::generation::structure::template_manager::{
    TemplateManager, TemplateManagerError, TemplateSource,
};
use crate::generation::worldgen_catalog::{WorldgenCatalog, WorldgenRecordKind};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PoolElementCounts {
    pub empty: usize,
    pub feature: usize,
    pub legacy_single: usize,
    pub list: usize,
    pub single: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TemplatePoolAudit {
    pub pools: usize,
    pub weighted_entries: usize,
    pub expanded_weight: usize,
    pub elements: PoolElementCounts,
    pub rigid: usize,
    pub terrain_matching: usize,
    pub referenced_templates: BTreeSet<String>,
    pub missing_templates: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct TemplatePoolCatalog {
    pools: BTreeMap<String, TemplatePool>,
    audit: TemplatePoolAudit,
}

impl TemplatePoolCatalog {
    pub fn decode<S>(
        catalog: WorldgenCatalog<'_>,
        templates: &mut TemplateManager<S>,
    ) -> Result<Self, PoolCatalogError>
    where
        S: TemplateSource,
    {
        let mut pools = BTreeMap::new();
        let mut audit = TemplatePoolAudit::default();
        for entry in catalog.entries(WorldgenRecordKind::TemplatePool) {
            let resource = entry.persistent_id().resource();
            let Some(path) = resource.path().strip_prefix("template_pool/") else {
                return Err(PoolCatalogError::PoolId(resource.to_string()));
            };
            let name = format!("{}:{path}", resource.namespace());
            let root = object(entry.value(), &name, "root")?;
            let fallback = string_field(root, &name, "fallback")?.to_owned();
            let elements = array_field(root, &name, "elements")?;
            let mut weighted = Vec::with_capacity(elements.len());
            for (index, value) in elements.iter().enumerate() {
                let field = format!("elements[{index}]");
                let weighted_object = object(value, &name, &field)?;
                let weight = unsigned_weight(weighted_object.get("weight"), &name, &field)?;
                let element_value = required(weighted_object, &name, &field, "element")?;
                let element = decode_element(element_value, &name, templates, &mut audit, true)?;
                audit.weighted_entries += 1;
                audit.expanded_weight += usize::from(weight);
                weighted.push((element, weight));
            }
            let pool = TemplatePool::new(fallback, weighted)
                .map_err(|error| PoolCatalogError::Jigsaw(name.clone(), error.to_string()))?;
            if pools.insert(name.clone(), pool).is_some() {
                return Err(PoolCatalogError::DuplicatePool(name));
            }
        }
        audit.pools = pools.len();
        Ok(Self { pools, audit })
    }

    pub fn pools(&self) -> &BTreeMap<String, TemplatePool> {
        &self.pools
    }

    pub fn audit(&self) -> &TemplatePoolAudit {
        &self.audit
    }

    pub fn into_pools(self) -> BTreeMap<String, TemplatePool> {
        self.pools
    }
}

fn decode_element<S>(
    value: &Value,
    pool: &str,
    templates: &mut TemplateManager<S>,
    audit: &mut TemplatePoolAudit,
    count_weighted_kind: bool,
) -> Result<PoolElement, PoolCatalogError>
where
    S: TemplateSource,
{
    let element = object(value, pool, "element")?;
    let kind = string_field(element, pool, "element_type")?;
    match kind {
        "minecraft:empty_pool_element" => {
            if count_weighted_kind {
                audit.elements.empty += 1;
            }
            Ok(PoolElement::empty())
        }
        "minecraft:feature_pool_element" => {
            if count_weighted_kind {
                audit.elements.feature += 1;
            }
            let projection = decode_projection(element, pool, audit, count_weighted_kind)?;
            let feature = string_field(element, pool, "feature")?.to_owned();
            Ok(PoolElement {
                kind: ElementKind::Feature { name: feature },
                projection,
                size: [0; 3],
                connectors: vec![feature_connector()],
                ground_level_delta: 0,
                processor_list: None,
            })
        }
        "minecraft:single_pool_element" | "minecraft:legacy_single_pool_element" => {
            let legacy = kind == "minecraft:legacy_single_pool_element";
            if legacy && count_weighted_kind {
                audit.elements.legacy_single += 1;
            } else if count_weighted_kind {
                audit.elements.single += 1;
            }
            let projection = decode_projection(element, pool, audit, count_weighted_kind)?;
            decode_single(element, pool, templates, audit, projection, legacy)
        }
        "minecraft:list_pool_element" => {
            if count_weighted_kind {
                audit.elements.list += 1;
            }
            let projection = decode_projection(element, pool, audit, count_weighted_kind)?;
            let children = array_field(element, pool, "elements")?;
            if children.is_empty() {
                return Err(invalid(pool, "elements", "a nonempty array"));
            }
            let mut decoded = children
                .iter()
                .map(|child| decode_element(child, pool, templates, audit, false))
                .collect::<Result<Vec<_>, _>>()?;
            for child in &mut decoded {
                force_projection(child, projection);
            }
            let size = decoded.iter().fold([0; 3], |mut maximum, child| {
                for (axis, value) in maximum.iter_mut().enumerate() {
                    *value = (*value).max(child.size[axis]);
                }
                maximum
            });
            let connectors = decoded[0].connectors.clone();
            let ground_level_delta = decoded[0].ground_level_delta;
            Ok(PoolElement {
                kind: ElementKind::List(decoded),
                projection,
                size,
                connectors,
                ground_level_delta,
                processor_list: None,
            })
        }
        _ => Err(PoolCatalogError::ElementType {
            pool: pool.to_owned(),
            kind: kind.to_owned(),
        }),
    }
}

fn decode_single<S>(
    element: &Map<String, Value>,
    pool: &str,
    templates: &mut TemplateManager<S>,
    audit: &mut TemplatePoolAudit,
    projection: Projection,
    legacy: bool,
) -> Result<PoolElement, PoolCatalogError>
where
    S: TemplateSource,
{
    let location = string_field(element, pool, "location")?.to_owned();
    audit.referenced_templates.insert(location.clone());
    let lookup = templates.get_or_create(&location)?;
    if lookup.missing {
        audit.missing_templates.insert(location.clone());
    }
    let connectors = template_connectors(&lookup.template, &location)?;
    let processors = match element.get("processors") {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Object(value)) if inline_processors_are_empty(value) => None,
        Some(_) => return Err(invalid(pool, "processors", "a resource ID or empty object")),
        None => None,
    };
    Ok(PoolElement {
        kind: ElementKind::Single {
            template: location,
            legacy,
        },
        projection,
        size: lookup.template.size,
        connectors,
        ground_level_delta: 1,
        processor_list: processors,
    })
}

fn inline_processors_are_empty(value: &Map<String, Value>) -> bool {
    value.len() == 1
        && value
            .get("processors")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
}

pub fn template_connectors(
    template: &StructureTemplate,
    name: &str,
) -> Result<Vec<Connector>, PoolCatalogError> {
    let mut connectors = Vec::new();
    for block in &template.blocks {
        let Some(state) = template.state(0, block) else {
            continue;
        };
        if state.block != "minecraft:jigsaw" {
            continue;
        }
        let orientation = state
            .properties
            .get("orientation")
            .ok_or_else(|| connector_error(name, block.position, "missing orientation"))?;
        let (front, top) = parse_orientation(orientation)
            .ok_or_else(|| connector_error(name, block.position, "invalid orientation"))?;
        let nbt = block
            .nbt
            .as_ref()
            .ok_or_else(|| connector_error(name, block.position, "missing block NBT"))?;
        connectors.push(Connector {
            local_position: block.position,
            front,
            top,
            joint: parse_joint(nbt, name, block.position)?,
            name: nbt_string(nbt, "name")
                .unwrap_or("minecraft:empty")
                .to_owned(),
            target: nbt_string(nbt, "target")
                .unwrap_or("minecraft:empty")
                .to_owned(),
            pool: nbt_string(nbt, "pool")
                .unwrap_or("minecraft:empty")
                .to_owned(),
            selection_priority: nbt_integer(nbt, "selection_priority").unwrap_or(0),
            placement_priority: nbt_integer(nbt, "placement_priority").unwrap_or(0),
        });
    }
    Ok(connectors)
}

fn parse_orientation(value: &str) -> Option<(Direction, Direction)> {
    let (front, top) = value.split_once('_')?;
    Some((parse_direction(front)?, parse_direction(top)?))
}

fn parse_direction(value: &str) -> Option<Direction> {
    match value {
        "down" => Some(Direction::Down),
        "up" => Some(Direction::Up),
        "north" => Some(Direction::North),
        "south" => Some(Direction::South),
        "west" => Some(Direction::West),
        "east" => Some(Direction::East),
        _ => None,
    }
}

fn parse_joint(
    nbt: &NbtCompound,
    name: &str,
    position: BlockPos,
) -> Result<Joint, PoolCatalogError> {
    match nbt_string(nbt, "joint").unwrap_or("rollable") {
        "rollable" => Ok(Joint::Rollable),
        "aligned" => Ok(Joint::Aligned),
        _ => Err(connector_error(name, position, "invalid joint")),
    }
}

fn nbt_string<'a>(nbt: &'a NbtCompound, field: &str) -> Option<&'a str> {
    nbt.get(field).and_then(|value| value.as_str())
}

fn nbt_integer(nbt: &NbtCompound, field: &str) -> Option<i32> {
    nbt.get(field).and_then(|value| value.as_i32())
}

fn feature_connector() -> Connector {
    Connector {
        local_position: BlockPos::new(0, 0, 0),
        front: Direction::Down,
        top: Direction::North,
        joint: Joint::Rollable,
        name: "minecraft:bottom".into(),
        target: "minecraft:empty".into(),
        pool: "minecraft:empty".into(),
        selection_priority: 0,
        placement_priority: 0,
    }
}

fn force_projection(element: &mut PoolElement, projection: Projection) {
    element.projection = projection;
    if let ElementKind::List(children) = &mut element.kind {
        for child in children {
            force_projection(child, projection);
        }
    }
}

fn decode_projection(
    element: &Map<String, Value>,
    pool: &str,
    audit: &mut TemplatePoolAudit,
    count: bool,
) -> Result<Projection, PoolCatalogError> {
    match string_field(element, pool, "projection")? {
        "rigid" => {
            if count {
                audit.rigid += 1;
            }
            Ok(Projection::Rigid)
        }
        "terrain_matching" => {
            if count {
                audit.terrain_matching += 1;
            }
            Ok(Projection::TerrainMatching)
        }
        _ => Err(invalid(pool, "projection", "rigid or terrain_matching")),
    }
}

fn unsigned_weight(
    value: Option<&Value>,
    pool: &str,
    field: &str,
) -> Result<u16, PoolCatalogError> {
    value
        .and_then(Value::as_u64)
        .and_then(|weight| u16::try_from(weight).ok())
        .filter(|weight| (1..=150).contains(weight))
        .ok_or_else(|| invalid(pool, &format!("{field}.weight"), "an integer in 1..=150"))
}

fn required<'a>(
    object: &'a Map<String, Value>,
    pool: &str,
    parent: &str,
    field: &str,
) -> Result<&'a Value, PoolCatalogError> {
    object
        .get(field)
        .ok_or_else(|| invalid(pool, &format!("{parent}.{field}"), "a present value"))
}

fn string_field<'a>(
    object: &'a Map<String, Value>,
    pool: &str,
    field: &str,
) -> Result<&'a str, PoolCatalogError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(pool, field, "a string"))
}

fn array_field<'a>(
    object: &'a Map<String, Value>,
    pool: &str,
    field: &str,
) -> Result<&'a [Value], PoolCatalogError> {
    object
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| invalid(pool, field, "an array"))
}

fn object<'a>(
    value: &'a Value,
    pool: &str,
    field: &str,
) -> Result<&'a Map<String, Value>, PoolCatalogError> {
    value
        .as_object()
        .ok_or_else(|| invalid(pool, field, "an object"))
}

fn invalid(pool: &str, field: &str, expected: &'static str) -> PoolCatalogError {
    PoolCatalogError::Invalid {
        pool: pool.to_owned(),
        field: field.to_owned(),
        expected,
    }
}

fn connector_error(name: &str, position: BlockPos, detail: &'static str) -> PoolCatalogError {
    PoolCatalogError::Connector {
        template: name.to_owned(),
        position,
        detail,
    }
}

#[derive(Debug, Error)]
pub enum PoolCatalogError {
    #[error("invalid template-pool persistent ID {0}")]
    PoolId(String),
    #[error("template pool {pool} field {field} must be {expected}")]
    Invalid {
        pool: String,
        field: String,
        expected: &'static str,
    },
    #[error("template pool {pool} has unsupported element type {kind}")]
    ElementType { pool: String, kind: String },
    #[error("template pool {0} is duplicated")]
    DuplicatePool(String),
    #[error("template pool {0} has invalid jigsaw data: {1}")]
    Jigsaw(String, String),
    #[error("template {template} connector at {position:?}: {detail}")]
    Connector {
        template: String,
        position: BlockPos,
        detail: &'static str,
    },
    #[error(transparent)]
    Template(#[from] TemplateManagerError),
}
