use anyhow::{Context as _, Result, ensure};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub(crate) struct CatalogSources {
    reports: PathBuf,
    blocks: Value,
    registries: Value,
    server_data_jar: Vec<u8>,
}

impl CatalogSources {
    pub(crate) fn open(cache: &Path, version: &str) -> Result<Self> {
        let reports = cache.join("generated/reports");
        ensure!(
            reports.is_dir(),
            "missing generated reports {}; run `mc-ref reports`",
            reports.display()
        );
        let blocks = read_json(&reports.join("blocks.json"))?;
        ensure!(blocks.is_object(), "blocks.json root is not an object");
        let registries = read_json(&reports.join("registries.json"))?;
        ensure!(
            registries.is_object(),
            "registries.json root is not an object"
        );
        let server_data_jar = read_nested_server(&cache.join("server.jar"), version)?;
        Ok(Self {
            reports,
            blocks,
            registries,
            server_data_jar,
        })
    }

    pub(crate) fn load(&self, kind: &str) -> Result<BTreeMap<String, Value>> {
        if let Some(prefix) = server_data_prefix(kind) {
            return self.load_server_data(prefix);
        }
        match kind {
            "block" => object_entries(&self.blocks, "blocks.json"),
            "item" => self.load_item_components(),
            _ => self.load_registry(kind),
        }
    }

    pub(crate) fn block_ids(&self) -> Result<Vec<String>> {
        Ok(object_entries(&self.blocks, "blocks.json")?
            .into_keys()
            .collect())
    }

    fn load_item_components(&self) -> Result<BTreeMap<String, Value>> {
        let root = self.reports.join("minecraft/components/item");
        ensure!(
            root.is_dir(),
            "missing item component report directory {}",
            root.display()
        );
        let mut paths = Vec::new();
        collect_json_paths(&root, &mut paths)?;
        paths.sort();
        let mut values = BTreeMap::new();
        for path in paths {
            let relative = path
                .strip_prefix(&root)
                .with_context(|| format!("strip item report root from {}", path.display()))?
                .with_extension("");
            let id = format!(
                "minecraft:{}",
                relative.to_string_lossy().replace('\\', "/")
            );
            let previous = values.insert(id.clone(), read_json(&path)?);
            ensure!(previous.is_none(), "duplicate item report identity {id}");
        }
        Ok(values)
    }

    fn load_registry(&self, kind: &str) -> Result<BTreeMap<String, Value>> {
        let key = format!("minecraft:{}", registry_report_key(kind));
        let entries = self
            .registries
            .get(&key)
            .and_then(|registry| registry.get("entries"))
            .and_then(Value::as_object)
            .with_context(|| format!("registries.json has no {key} entries"))?;
        Ok(entries
            .iter()
            .map(|(id, value)| (id.clone(), value.clone()))
            .collect())
    }

    fn load_server_data(&self, prefix: &str) -> Result<BTreeMap<String, Value>> {
        let mut archive = ZipArchive::new(Cursor::new(&self.server_data_jar))
            .context("open nested server data JAR")?;
        let prefix = format!("{prefix}/");
        let mut values = BTreeMap::new();
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .with_context(|| format!("read nested server entry {index}"))?;
            let name = entry.name().to_owned();
            if !name.starts_with(&prefix) || !name.ends_with(".json") {
                continue;
            }
            let relative = &name[prefix.len()..name.len() - 5];
            let id = format!("minecraft:{relative}");
            let value = serde_json::from_reader(&mut entry)
                .with_context(|| format!("parse nested server data {name}"))?;
            let previous = values.insert(id.clone(), value);
            ensure!(
                previous.is_none(),
                "duplicate nested server data identity {id}"
            );
        }
        ensure!(
            !values.is_empty(),
            "nested server data prefix {prefix} contains no JSON"
        );
        Ok(values)
    }
}

fn read_nested_server(path: &Path, version: &str) -> Result<Vec<u8>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut archive = ZipArchive::new(file).context("open locked bundled server JAR")?;
    let expected = format!("META-INF/versions/{version}/server-{version}.jar");
    let mut entry = archive
        .by_name(&expected)
        .with_context(|| format!("bundled server has no {expected}"))?;
    let mut bytes = Vec::with_capacity(usize::try_from(entry.size()).unwrap_or(0));
    entry
        .read_to_end(&mut bytes)
        .with_context(|| format!("read {expected}"))?;
    ensure!(!bytes.is_empty(), "nested server JAR is empty");
    Ok(bytes)
}

fn read_json(path: &Path) -> Result<Value> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    serde_json::from_reader(file).with_context(|| format!("parse {}", path.display()))
}

fn object_entries(value: &Value, source: &str) -> Result<BTreeMap<String, Value>> {
    Ok(value
        .as_object()
        .with_context(|| format!("{source} root is not an object"))?
        .iter()
        .map(|(id, value)| (id.clone(), value.clone()))
        .collect())
}

fn collect_json_paths(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("read report directory {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_json_paths(&path, paths)?;
        } else if file_type.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("json")
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn registry_report_key(kind: &str) -> &str {
    match kind {
        "density_function_type" => "worldgen/density_function_type",
        "material_condition" => "worldgen/material_condition",
        "material_rule" => "worldgen/material_rule",
        "pool_alias_binding" => "worldgen/pool_alias_binding",
        "structure_processor" => "worldgen/structure_processor",
        "structure_pool_element" => "worldgen/structure_pool_element",
        "structure_type" => "worldgen/structure_type",
        _ => kind,
    }
}

fn server_data_prefix(kind: &str) -> Option<&'static str> {
    match kind {
        "recipe" => Some("data/minecraft/recipe"),
        "loot_table" => Some("data/minecraft/loot_table"),
        "advancement" => Some("data/minecraft/advancement"),
        "damage_type" => Some("data/minecraft/damage_type"),
        "enchantment" => Some("data/minecraft/enchantment"),
        "dimension_type" => Some("data/minecraft/dimension_type"),
        "sulfur_cube_archetype" => Some("data/minecraft/sulfur_cube_archetype"),
        "worldgen" => Some("data/minecraft/worldgen"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_registry_aliases_are_explicit() {
        assert_eq!(
            registry_report_key("density_function_type"),
            "worldgen/density_function_type"
        );
        assert_eq!(registry_report_key("entity_type"), "entity_type");
    }

    #[test]
    fn server_data_categories_are_bounded() {
        assert_eq!(
            server_data_prefix("advancement"),
            Some("data/minecraft/advancement")
        );
        assert_eq!(server_data_prefix("block"), None);
    }
}
