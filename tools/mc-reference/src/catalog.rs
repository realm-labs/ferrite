use crate::artifact::extract_server;
use crate::verification::{load_catalog, validate_rule_references, verify_cached_artifacts};
use crate::*;

pub(crate) fn query(context: &Context, kind: &str, raw_id: &str) -> Result<()> {
    verify_cached_artifacts(context)?;
    let kind = match kind {
        "entity" => "entity_type",
        "effect" => "mob_effect",
        value => value,
    };
    let id = normalize_id(raw_id)?;
    let ids = load_category_ids(context, kind)?;
    ensure!(
        ids.contains(&id),
        "{id} is not present in locked {kind} data"
    );
    let catalog = load_catalog(context)?;
    let blocks = if kind == "item" {
        Some(load_category_ids(context, "block")?)
    } else {
        None
    };
    let matched = classify(&catalog, kind, &id, blocks.as_ref())?;
    let value = query_value(context, kind, &id)?;
    let tags = query_tags(context, kind, &id)?;
    let rendered = serde_json::json!({
        "version": context.lock.version,
        "kind": kind,
        "id": id,
        "classification": matched.family.classification,
        "behavior_family": matched.family.name,
        "rules": matched.family.rules,
        "source": matched.category.source,
        "direct_tags": tags,
        "locked_data": value,
    });
    println!("{}", serde_json::to_string_pretty(&rendered)?);
    Ok(())
}

pub(crate) fn unreviewed(context: &Context, raw_kind: Option<&str>) -> Result<()> {
    verify_cached_artifacts(context)?;
    let requested_kind = raw_kind.map(|kind| match kind {
        "entity" => "entity_type",
        "effect" => "mob_effect",
        value => value,
    });
    let catalog = load_catalog(context)?;
    validate_rule_references(context, &catalog)?;
    if let Some(kind) = requested_kind {
        ensure!(
            catalog
                .category
                .iter()
                .any(|category| category.kind == kind),
            "catalog has no registry kind {kind}"
        );
    }
    let blocks = load_category_ids(context, "block")?;
    let mut total = 0;
    let stdout = io::stdout();
    let mut output = io::BufWriter::new(stdout.lock());
    for category in &catalog.category {
        if requested_kind.is_some_and(|kind| category.kind != kind) {
            continue;
        }
        let selectors = compile_family_selectors(category)?;
        let ids = load_category_ids(context, &category.kind)?;
        ensure!(
            ids.len() == category.expected_count,
            "{} count: expected {}, got {}",
            category.kind,
            category.expected_count,
            ids.len()
        );
        let digest = ids_digest(&ids);
        ensure!(
            digest == category.ids_sha1,
            "{} ID snapshot changed: expected {}, got {}",
            category.kind,
            category.ids_sha1,
            digest
        );
        validate_family_selectors(category, &ids, &blocks)?;
        for id in &ids {
            let matched = classify_in_category(category, &selectors, id, Some(&blocks))?;
            if matched.family.classification == Classification::Unreviewed {
                if let Err(error) =
                    writeln!(output, "{}\t{}\t{}", category.kind, id, matched.family.name)
                {
                    if error.kind() == io::ErrorKind::BrokenPipe {
                        return Ok(());
                    }
                    return Err(error.into());
                }
                total += 1;
            }
        }
    }
    if let Err(error) = output.flush() {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return Ok(());
        }
        return Err(error.into());
    }
    eprintln!("unreviewed inventory: {total} IDs");
    Ok(())
}

fn query_value(context: &Context, kind: &str, id: &str) -> Result<Value> {
    let reports = context.cache.join("generated/reports");
    if server_data_prefix(kind).is_some() {
        return read_server_data_json(context, kind, id);
    }
    match kind {
        "block" => Ok(read_json(&reports.join("blocks.json"))?
            .get(id)
            .cloned()
            .unwrap_or(Value::Null)),
        "item" => read_json(
            &reports
                .join("minecraft/components/item")
                .join(format!("{}.json", strip_namespace(id))),
        ),
        _ => {
            let value = read_json(&reports.join("registries.json"))?;
            registry_entry(&value, kind, id)
        }
    }
}

fn read_server_data_json(context: &Context, kind: &str, id: &str) -> Result<Value> {
    let server = extract_server(context)?;
    let prefix = server_data_prefix(kind).with_context(|| format!("no data path for {kind}"))?;
    let path = format!("{prefix}/{}.json", strip_namespace(id));
    let mut archive = ZipArchive::new(File::open(server)?)?;
    let mut entry = archive
        .by_name(&path)
        .with_context(|| format!("locked data has no {path}"))?;
    Ok(serde_json::from_reader(&mut entry)?)
}

fn query_tags(context: &Context, kind: &str, id: &str) -> Result<Vec<String>> {
    let tag_kind = match kind {
        "block" | "item" | "entity_type" | "mob_effect" | "damage_type" | "enchantment"
        | "fluid" => kind,
        _ => return Ok(Vec::new()),
    };
    let prefix = format!("data/minecraft/tags/{tag_kind}/");
    let server = extract_server(context)?;
    let mut archive = ZipArchive::new(File::open(server)?)?;
    let mut tags = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_string();
        if !name.starts_with(&prefix) || !name.ends_with(".json") {
            continue;
        }
        let value: Value = serde_json::from_reader(&mut entry)?;
        let directly_contains =
            value
                .get("values")
                .and_then(Value::as_array)
                .is_some_and(|values| {
                    values.iter().any(|value| match value {
                        Value::String(value) => value == id,
                        Value::Object(value) => value.get("id").and_then(Value::as_str) == Some(id),
                        _ => false,
                    })
                });
        if directly_contains {
            tags.push(format!("minecraft:{}", &name[prefix.len()..name.len() - 5]));
        }
    }
    tags.sort();
    Ok(tags)
}

pub(crate) fn coverage(context: &Context) -> Result<usize> {
    verify_cached_artifacts(context)?;
    let catalog = load_catalog(context)?;
    validate_rule_references(context, &catalog)?;
    let blocks = load_category_ids(context, "block")?;
    let mut total = 0;
    let mut unreviewed = 0;
    let mut unreviewed_families = BTreeMap::<(String, String), usize>::new();
    for category in &catalog.category {
        let selectors = compile_family_selectors(category)?;
        let ids = load_category_ids(context, &category.kind)?;
        ensure!(
            ids.len() == category.expected_count,
            "{} count: expected {}, got {}",
            category.kind,
            category.expected_count,
            ids.len()
        );
        let digest = ids_digest(&ids);
        ensure!(
            digest == category.ids_sha1,
            "{} ID snapshot changed: expected {}, got {}",
            category.kind,
            category.ids_sha1,
            digest
        );
        validate_family_selectors(category, &ids, &blocks)?;
        for id in &ids {
            let matched = classify_in_category(category, &selectors, id, Some(&blocks))?;
            if matched.family.classification == Classification::Unreviewed {
                unreviewed += 1;
                *unreviewed_families
                    .entry((category.kind.clone(), matched.family.name.clone()))
                    .or_default() += 1;
            }
        }
        total += ids.len();
        println!("{:<18} {:>5} IDs  {}", category.kind, ids.len(), digest);
    }
    println!(
        "coverage complete: {total} locked IDs, zero unclassified or ambiguous; {unreviewed} explicitly unreviewed"
    );
    for ((kind, family), count) in unreviewed_families {
        println!("unreviewed {kind}/{family}: {count} IDs");
    }
    Ok(total)
}

pub(crate) fn validate_family_selectors(
    category: &Category,
    ids: &BTreeSet<String>,
    blocks: &BTreeSet<String>,
) -> Result<()> {
    for family in &category.family {
        ensure!(
            !(family.remaining && family.classification == Classification::Special),
            "{}/{} is a Special fallback; Special families require an explicit selector and unaudited fallbacks must remain Unreviewed",
            category.kind,
            family.name
        );
        if family.remaining && family.classification == Classification::DataOnly {
            ensure!(
                matches!(
                    category.kind.as_str(),
                    "potion"
                        | "recipe"
                        | "loot_table"
                        | "advancement"
                        | "damage_type"
                        | "enchantment"
                ),
                "{}/{} is not approved for a DataOnly fallback; audit and split it or keep it Unreviewed",
                category.kind,
                family.name
            );
        }
        for exact in &family.exact {
            let exact = normalize_unchecked(exact);
            ensure!(
                ids.contains(&exact),
                "{}/{} has stale exact ID {exact}",
                category.kind,
                family.name
            );
        }
        for pattern in &family.patterns {
            let normalized = normalize_unchecked(pattern);
            let matcher = Glob::new(&normalized)?.compile_matcher();
            ensure!(
                ids.iter().any(|id| matcher.is_match(id)),
                "{}/{} pattern {normalized} matches zero locked IDs",
                category.kind,
                family.name
            );
        }
        if family.block_items {
            ensure!(
                category.kind == "item" && ids.iter().any(|id| blocks.contains(id)),
                "{}/{} block_items selector matches zero locked IDs",
                category.kind,
                family.name
            );
        }
        ensure!(
            family.remaining
                || !family.exact.is_empty()
                || !family.patterns.is_empty()
                || family.block_items,
            "{}/{} has no selector",
            category.kind,
            family.name
        );
    }
    Ok(())
}

pub(crate) fn load_category_ids(context: &Context, kind: &str) -> Result<BTreeSet<String>> {
    let reports = context.cache.join("generated/reports");
    let server = extract_server(context)?;
    if server_data_prefix(kind).is_some() {
        return ids_from_server_data(&server, kind);
    }
    match kind {
        "block" => Ok(read_json(&reports.join("blocks.json"))?
            .as_object()
            .context("blocks.json is not an object")?
            .keys()
            .cloned()
            .collect()),
        "item" => ids_from_files(&reports.join("minecraft/components/item"), "json"),
        _ => {
            let value = read_json(&reports.join("registries.json"))?;
            registry_ids(&value, kind)
        }
    }
}

pub(crate) fn registry_entry(registries: &Value, kind: &str, id: &str) -> Result<Value> {
    let registry = escape_pointer(&format!("minecraft:{}", registry_report_key(kind)));
    registries
        .pointer(&format!("/{registry}/entries/{}", escape_pointer(id)))
        .cloned()
        .with_context(|| format!("registry {kind} has no entry {id}"))
}

pub(crate) fn registry_ids(registries: &Value, kind: &str) -> Result<BTreeSet<String>> {
    let registry = escape_pointer(&format!("minecraft:{}", registry_report_key(kind)));
    Ok(registries
        .pointer(&format!("/{registry}/entries"))
        .and_then(Value::as_object)
        .with_context(|| format!("registry {kind} missing"))?
        .keys()
        .cloned()
        .collect())
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

pub(crate) fn ids_from_files(directory: &Path, extension: &str) -> Result<BTreeSet<String>> {
    ensure!(
        directory.is_dir(),
        "missing generated report directory {}",
        directory.display()
    );
    let mut ids = BTreeSet::new();
    for entry in WalkDir::new(directory)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|v| v.to_str()) == Some(extension))
    {
        let relative = entry.path().strip_prefix(directory)?.with_extension("");
        ids.insert(format!(
            "minecraft:{}",
            relative.to_string_lossy().replace('\\', "/")
        ));
    }
    Ok(ids)
}

fn ids_from_server_data(server: &Path, kind: &str) -> Result<BTreeSet<String>> {
    let input = File::open(server)?;
    let mut archive = ZipArchive::new(input)?;
    let prefix = format!(
        "{}/",
        server_data_prefix(kind).with_context(|| format!("no data path for {kind}"))?
    );
    let mut ids = BTreeSet::new();
    for index in 0..archive.len() {
        let name = archive.by_index(index)?.name().to_string();
        if name.starts_with(&prefix) && name.ends_with(".json") {
            let relative = &name[prefix.len()..name.len() - 5];
            ids.insert(format!("minecraft:{relative}"));
        }
    }
    Ok(ids)
}

pub(crate) fn server_data_prefix(kind: &str) -> Option<&'static str> {
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

pub(crate) fn classify<'a>(
    catalog: &'a Catalog,
    kind: &str,
    id: &str,
    blocks: Option<&BTreeSet<String>>,
) -> Result<MatchResult<'a>> {
    let category = catalog
        .category
        .iter()
        .find(|category| category.kind == kind)
        .with_context(|| format!("catalog has no {kind} category"))?;
    let selectors = compile_family_selectors(category)?;
    classify_in_category(category, &selectors, id, blocks)
}

pub(crate) fn compile_family_selectors(category: &Category) -> Result<Vec<CompiledFamilySelector>> {
    category
        .family
        .iter()
        .map(|family| {
            let mut builder = GlobSetBuilder::new();
            for pattern in &family.patterns {
                builder.add(Glob::new(&normalize_unchecked(pattern))?);
            }
            Ok(CompiledFamilySelector {
                exact: family
                    .exact
                    .iter()
                    .map(|value| normalize_unchecked(value))
                    .collect(),
                patterns: builder.build()?,
            })
        })
        .collect()
}

pub(crate) fn classify_in_category<'a>(
    category: &'a Category,
    selectors: &[CompiledFamilySelector],
    id: &str,
    blocks: Option<&BTreeSet<String>>,
) -> Result<MatchResult<'a>> {
    ensure!(
        selectors.len() == category.family.len(),
        "{} compiled selector count differs from its families",
        category.kind
    );
    let mut matches = Vec::new();
    for (family, selector) in category.family.iter().zip(selectors) {
        let mut matched = selector.exact.contains(id);
        if !matched {
            matched = selector.patterns.is_match(id);
        }
        if !matched && family.block_items && matches.is_empty() {
            matched = blocks.is_some_and(|blocks| blocks.contains(id));
        }
        if !matched && family.remaining {
            matched = matches.is_empty();
        }
        if matched {
            matches.push(family);
        }
    }
    ensure!(
        matches.len() == 1,
        "{} {id} matched {} behavior families",
        category.kind,
        matches.len()
    );
    Ok(MatchResult {
        category,
        family: matches[0],
    })
}
