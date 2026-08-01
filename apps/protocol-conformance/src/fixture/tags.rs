use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use ferrite_protocol::java_26_2::configuration::clientbound::packet::{
    RegistryTags, TagDefinition,
};
use ferrite_protocol::java_26_2::configuration::registry::SYNCHRONIZED_REGISTRY_IDENTITIES;
use serde_json::{Map, Value};

use crate::DynError;
use crate::fixture::{data_entries, identifier};

const MAX_TAG_FILES: usize = 4_096;
const MAX_TAG_DEPTH: usize = 16;

pub(super) fn load(
    report: &Map<String, Value>,
    data_root: &Path,
) -> Result<Vec<RegistryTags>, DynError> {
    let registries = registry_ids(report, data_root)?;
    let definitions = tag_definitions(data_root, &registries)?;
    definitions
        .into_iter()
        .map(|(registry, tags)| {
            let ids = registries
                .get(&registry)
                .ok_or_else(|| format!("tag registry {registry} is absent from report"))?;
            let mut resolved = BTreeMap::new();
            let mut visiting = BTreeSet::new();
            let mut output = Vec::with_capacity(tags.len());
            for tag in tags.keys() {
                let members = resolve_tag(tag, &tags, ids, &mut resolved, &mut visiting)?;
                output.push(TagDefinition {
                    id: identifier(tag)?,
                    members,
                });
            }
            Ok(RegistryTags {
                registry: identifier(&registry)?,
                tags: output,
            })
        })
        .collect()
}

fn registry_ids(
    report: &Map<String, Value>,
    data_root: &Path,
) -> Result<BTreeMap<String, BTreeMap<String, i32>>, DynError> {
    let mut registries = report
        .iter()
        .filter_map(|(registry, value)| {
            value
                .get("entries")
                .and_then(Value::as_object)
                .map(|entries| (registry, entries))
        })
        .map(|(registry, entries)| {
            let ids = entries
                .iter()
                .map(|(entry, value)| {
                    let raw_id = value
                        .get("protocol_id")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| format!("{registry}/{entry} has no protocol_id"))?;
                    Ok((entry.clone(), i32::try_from(raw_id)?))
                })
                .collect::<Result<BTreeMap<_, _>, DynError>>()?;
            Ok((registry.clone(), ids))
        })
        .collect::<Result<BTreeMap<_, _>, DynError>>()?;
    for identity in SYNCHRONIZED_REGISTRY_IDENTITIES {
        if !registries.contains_key(identity) {
            let ids = data_entries(data_root, identity)?
                .into_iter()
                .enumerate()
                .map(|(raw_id, entry)| Ok((entry, i32::try_from(raw_id)?)))
                .collect::<Result<BTreeMap<_, _>, DynError>>()?;
            registries.insert(identity.to_owned(), ids);
        }
    }
    Ok(registries)
}

fn tag_definitions(
    data_root: &Path,
    registries: &BTreeMap<String, BTreeMap<String, i32>>,
) -> Result<BTreeMap<String, BTreeMap<String, Vec<String>>>, DynError> {
    let tag_root = data_root.join("tags");
    let mut files = Vec::new();
    collect_files(&tag_root, 0, &mut files)?;
    files.sort();
    let mut definitions = BTreeMap::<String, BTreeMap<String, Vec<String>>>::new();
    for file in files {
        let relative = file.strip_prefix(&tag_root)?;
        let path = slash_path(relative)?;
        let path = path
            .strip_suffix(".json")
            .ok_or("tag file did not retain its JSON suffix")?;
        let Some((registry, tag)) = split_registry_path(path, registries) else {
            continue;
        };
        let document: Value = serde_json::from_slice(&fs::read(&file)?)?;
        let values = document
            .get("values")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("tag {registry}/{tag} has no values array"))?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(normalize_identifier)
                    .ok_or_else(|| format!("tag {registry}/{tag} has a non-string member").into())
            })
            .collect::<Result<Vec<_>, DynError>>()?;
        let prior = definitions
            .entry(registry)
            .or_default()
            .insert(tag.clone(), values);
        if prior.is_some() {
            return Err(format!("tag {tag} is defined more than once").into());
        }
    }
    Ok(definitions)
}

fn collect_files(
    directory: &Path,
    depth: usize,
    output: &mut Vec<PathBuf>,
) -> Result<(), DynError> {
    if depth > MAX_TAG_DEPTH {
        return Err(format!("tag data exceeds directory depth {MAX_TAG_DEPTH}").into());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files(&path, depth + 1, output)?;
        } else if file_type.is_file() && path.extension().is_some_and(|value| value == "json") {
            if output.len() == MAX_TAG_FILES {
                return Err(format!("tag data exceeds {MAX_TAG_FILES} files").into());
            }
            output.push(path);
        }
    }
    Ok(())
}

fn slash_path(path: &Path) -> Result<String, DynError> {
    let parts = path
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| "tag path is not UTF-8".into())
        })
        .collect::<Result<Vec<String>, DynError>>()?;
    Ok(parts.join("/"))
}

fn split_registry_path(
    path: &str,
    registries: &BTreeMap<String, BTreeMap<String, i32>>,
) -> Option<(String, String)> {
    let parts = path.split('/').collect::<Vec<_>>();
    for split in (1..parts.len()).rev() {
        let registry = format!("minecraft:{}", parts[..split].join("/"));
        if registries.contains_key(&registry) {
            return Some((registry, format!("minecraft:{}", parts[split..].join("/"))));
        }
    }
    None
}

fn resolve_tag(
    tag: &str,
    definitions: &BTreeMap<String, Vec<String>>,
    ids: &BTreeMap<String, i32>,
    resolved: &mut BTreeMap<String, Vec<i32>>,
    visiting: &mut BTreeSet<String>,
) -> Result<Vec<i32>, DynError> {
    if let Some(members) = resolved.get(tag) {
        return Ok(members.clone());
    }
    if !visiting.insert(tag.to_owned()) {
        return Err(format!("tag reference cycle includes {tag}").into());
    }
    let values = definitions
        .get(tag)
        .ok_or_else(|| format!("required tag reference {tag} is absent"))?;
    let mut members = Vec::new();
    let mut seen = BTreeSet::new();
    for value in values {
        if let Some(reference) = value.strip_prefix('#') {
            for member in resolve_tag(reference, definitions, ids, resolved, visiting)? {
                if seen.insert(member) {
                    members.push(member);
                }
            }
        } else {
            let member = *ids
                .get(value)
                .ok_or_else(|| format!("tag {tag} references absent registry entry {value}"))?;
            if seen.insert(member) {
                members.push(member);
            }
        }
    }
    visiting.remove(tag);
    resolved.insert(tag.to_owned(), members.clone());
    Ok(members)
}

fn normalize_identifier(value: &str) -> String {
    let (prefix, value) = value
        .strip_prefix('#')
        .map_or(("", value), |value| ("#", value));
    if value.contains(':') {
        format!("{prefix}{value}")
    } else {
        format!("{prefix}minecraft:{value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_vanilla_tag_closure_is_complete() {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let version_root = workspace.join("target/mc-reference/26.2");
        let registries = version_root.join("generated/reports/registries.json");
        let server_data = version_root.join("server-classes/data/minecraft");
        if !registries.is_file() || !server_data.is_dir() {
            eprintln!(
                "locked local Minecraft artifacts are absent; `mc-ref verify --offline` owns that gate"
            );
            return;
        }
        let report: Value = serde_json::from_slice(&fs::read(registries).unwrap()).unwrap();
        let tags = load(report.as_object().unwrap(), &server_data).unwrap();
        assert_eq!(tags.len(), 15);
        assert_eq!(
            tags.iter()
                .map(|registry| registry.tags.len())
                .sum::<usize>(),
            697
        );
        let item = tags
            .iter()
            .find(|registry| registry.registry.to_string() == "minecraft:item")
            .unwrap();
        assert_eq!(item.tags.len(), 224);
        assert_eq!(
            item.tags
                .iter()
                .find(|tag| {
                    tag.id.to_string() == "minecraft:sulfur_cube_archetype/fast_sliding"
                })
                .unwrap()
                .members
                .len(),
            3
        );
    }
}
