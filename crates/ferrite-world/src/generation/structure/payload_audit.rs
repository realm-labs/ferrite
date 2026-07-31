//! Physical payload audit for the six locked jigsaw structure families.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::generation::structure::jigsaw::Joint;
use crate::generation::structure::pool_catalog::{PoolCatalogError, template_connectors};
use crate::generation::structure::template::StructureTemplate;
use crate::generation::structure::template_manager::{
    FileTemplateSource, TemplateManager, TemplateManagerError,
};

const FAMILIES: [&str; 6] = [
    "ancient_city",
    "bastion",
    "pillager_outpost",
    "trail_ruins",
    "trial_chambers",
    "village",
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TemplatePayloadCounts {
    pub templates: usize,
    pub volume: usize,
    pub encoded_blocks: usize,
    pub absent_blocks: usize,
    pub explicit_air: usize,
    pub jigsaws: usize,
    pub other_block_nbt: usize,
    pub structure_void: usize,
    pub structure_blocks: usize,
    pub entities: usize,
    pub duplicate_positions: usize,
    pub connectors: usize,
    pub aligned_connectors: usize,
    pub rollable_connectors: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JigsawPayloadAudit {
    pub total: TemplatePayloadCounts,
    pub families: BTreeMap<String, TemplatePayloadCounts>,
    pub physical_templates: BTreeSet<String>,
    pub missing_references: BTreeSet<String>,
    pub unreferenced_templates: BTreeSet<String>,
    pub block_ids: BTreeSet<String>,
    pub exact_states: BTreeSet<String>,
    pub connector_pools: BTreeSet<String>,
    pub connector_final_states: BTreeSet<String>,
    pub selection_priorities: BTreeMap<i32, usize>,
    pub placement_priorities: BTreeMap<i32, usize>,
}

pub fn audit_locked_jigsaw_payload(
    resource_root: impl Into<PathBuf>,
    referenced_templates: &BTreeSet<String>,
) -> Result<JigsawPayloadAudit, PayloadAuditError> {
    let resource_root = resource_root.into();
    let structure_root = resource_root.join("data/minecraft/structure");
    let mut paths = Vec::new();
    for family in FAMILIES {
        collect_nbt_files(&structure_root.join(family), &mut paths)?;
    }
    paths.sort();

    let mut manager = TemplateManager::new(FileTemplateSource::new(resource_root));
    let mut audit = JigsawPayloadAudit::default();
    for path in paths {
        let relative = path
            .strip_prefix(&structure_root)
            .map_err(|_| PayloadAuditError::OutsideRoot(path.clone()))?;
        let template_path = resource_path(relative)?;
        let id = format!("minecraft:{template_path}");
        let family = template_path
            .split('/')
            .next()
            .expect("physical template has a family");
        let template = manager.require(&id)?.template;
        let counts = audit_template(&template, &id, &mut audit)?;
        merge_counts(&mut audit.total, &counts);
        merge_counts(
            audit.families.entry(family.to_owned()).or_default(),
            &counts,
        );
        audit.physical_templates.insert(id);
    }
    audit.missing_references = referenced_templates
        .difference(&audit.physical_templates)
        .cloned()
        .collect();
    audit.unreferenced_templates = audit
        .physical_templates
        .difference(referenced_templates)
        .cloned()
        .collect();
    Ok(audit)
}

fn audit_template(
    template: &StructureTemplate,
    name: &str,
    audit: &mut JigsawPayloadAudit,
) -> Result<TemplatePayloadCounts, PayloadAuditError> {
    let mut counts = TemplatePayloadCounts {
        templates: 1,
        volume: template.volume(),
        encoded_blocks: template.blocks.len(),
        absent_blocks: template.volume().saturating_sub(template.blocks.len()),
        entities: template.entities.len(),
        duplicate_positions: template.duplicate_positions().len(),
        ..TemplatePayloadCounts::default()
    };
    let palette = template
        .palettes
        .first()
        .ok_or_else(|| PayloadAuditError::Palette(name.to_owned()))?;
    for block in &template.blocks {
        let state = palette
            .states
            .get(block.state_index)
            .ok_or_else(|| PayloadAuditError::Palette(name.to_owned()))?;
        audit.block_ids.insert(state.block.clone());
        audit.exact_states.insert(format!(
            "{}{:?}",
            state.block,
            state.properties.iter().collect::<Vec<_>>()
        ));
        match state.block.as_str() {
            "minecraft:air" => counts.explicit_air += 1,
            "minecraft:jigsaw" => counts.jigsaws += 1,
            "minecraft:structure_void" => counts.structure_void += 1,
            "minecraft:structure_block" => counts.structure_blocks += 1,
            _ => {}
        }
        if block.nbt.is_some() && state.block != "minecraft:jigsaw" {
            counts.other_block_nbt += 1;
        }
        if state.block == "minecraft:jigsaw"
            && let Some(final_state) = block
                .nbt
                .as_ref()
                .and_then(|nbt| nbt.get("final_state"))
                .and_then(|value| value.as_str())
        {
            audit.connector_final_states.insert(final_state.to_owned());
        }
    }
    let connectors = template_connectors(template, name)?;
    counts.connectors = connectors.len();
    for connector in connectors {
        match connector.joint {
            Joint::Aligned => counts.aligned_connectors += 1,
            Joint::Rollable => counts.rollable_connectors += 1,
        }
        audit.connector_pools.insert(connector.pool);
        *audit
            .selection_priorities
            .entry(connector.selection_priority)
            .or_default() += 1;
        *audit
            .placement_priorities
            .entry(connector.placement_priority)
            .or_default() += 1;
    }
    Ok(counts)
}

fn merge_counts(total: &mut TemplatePayloadCounts, value: &TemplatePayloadCounts) {
    total.templates += value.templates;
    total.volume += value.volume;
    total.encoded_blocks += value.encoded_blocks;
    total.absent_blocks += value.absent_blocks;
    total.explicit_air += value.explicit_air;
    total.jigsaws += value.jigsaws;
    total.other_block_nbt += value.other_block_nbt;
    total.structure_void += value.structure_void;
    total.structure_blocks += value.structure_blocks;
    total.entities += value.entities;
    total.duplicate_positions += value.duplicate_positions;
    total.connectors += value.connectors;
    total.aligned_connectors += value.aligned_connectors;
    total.rollable_connectors += value.rollable_connectors;
}

fn collect_nbt_files(path: &Path, output: &mut Vec<PathBuf>) -> Result<(), PayloadAuditError> {
    for entry in fs::read_dir(path).map_err(|error| PayloadAuditError::Read {
        path: path.to_owned(),
        error,
    })? {
        let entry = entry.map_err(|error| PayloadAuditError::Read {
            path: path.to_owned(),
            error,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_nbt_files(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "nbt") {
            output.push(path);
        }
    }
    Ok(())
}

fn resource_path(path: &Path) -> Result<String, PayloadAuditError> {
    let mut parts = path
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| PayloadAuditError::Path(path.to_owned()))?;
    let last = parts
        .last_mut()
        .ok_or_else(|| PayloadAuditError::Path(path.to_owned()))?;
    *last = last
        .strip_suffix(".nbt")
        .ok_or_else(|| PayloadAuditError::Path(path.to_owned()))?;
    Ok(parts.join("/"))
}

#[derive(Debug, Error)]
pub enum PayloadAuditError {
    #[error("read payload directory {}: {error}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        error: std::io::Error,
    },
    #[error("payload path {} is outside the structure root", .0.display())]
    OutsideRoot(PathBuf),
    #[error("invalid payload path {}", .0.display())]
    Path(PathBuf),
    #[error("payload template {0} has no usable first palette")]
    Palette(String),
    #[error(transparent)]
    Template(#[from] TemplateManagerError),
    #[error(transparent)]
    Connector(#[from] PoolCatalogError),
}
