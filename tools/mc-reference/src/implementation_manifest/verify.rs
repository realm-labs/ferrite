use crate::Context;
use anyhow::{Context as _, Result, bail, ensure};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use toml::{Table, Value};

use super::{BASELINE_SHA256, OUTPUT_RELATIVE, WORLDGEN_EXACTNESS_SHA256, materialize};

const RECORD_ARRAYS: [(&str, &str); 7] = [
    ("worldgen_exactness", "id"),
    ("catalog_batch", "reference_kind"),
    ("gameplay_batch", "id"),
    ("deferred_observation", "slice"),
    ("surface_owner", "reference"),
    ("join_owner", "left+right"),
    ("protocol_batch", "reference_family"),
];
const PROGRESS_FIELDS: [&str; 5] = [
    "disposition",
    "evidence",
    "rationale",
    "attempted_alternatives",
    "unblock_conditions",
];

pub(super) fn run(context: &Context) -> Result<()> {
    let path = context.workspace.join(OUTPUT_RELATIVE);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let actual: Value =
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let expected =
        Value::try_from(materialize(context)?).context("serialize expected implementation map")?;

    verify_root(context, &actual, &expected)?;
    for (array, key) in RECORD_ARRAYS {
        verify_record_array(&actual, &expected, array, key)?;
    }
    verify_rule_reachability(&actual, &expected)?;
    verify_batch_graph(context, &actual)?;
    verify_progress(&context.workspace, &actual)?;
    render_counters(&actual)?;

    let digest = hex::encode(Sha256::digest(text.as_bytes()));
    println!("implementation manifest verified: sha256 {digest}");
    Ok(())
}

fn verify_rule_reachability(actual: &Value, expected: &Value) -> Result<()> {
    let actual_parents = referenced_rules(actual, "parents")?;
    let expected_parents = referenced_rules(expected, "parents")?;
    ensure!(
        actual_parents == expected_parents,
        "implementation parent-rule reachability is stale"
    );

    let actual_leaves = referenced_rules(actual, "leaves")?;
    let expected_leaves = referenced_rules(expected, "leaves")?;
    ensure!(
        actual_leaves == expected_leaves,
        "implementation leaf-rule reachability is stale"
    );
    ensure!(
        actual_parents.len() == 65,
        "implementation reaches {} parent rules; expected 65",
        actual_parents.len()
    );
    ensure!(
        actual_leaves.len() == 352,
        "implementation reaches {} leaf rules; expected 352",
        actual_leaves.len()
    );
    println!(
        "implementation rule reachability verified: {} parent rules, {} leaf rules",
        actual_parents.len(),
        actual_leaves.len()
    );
    Ok(())
}

fn referenced_rules(root: &Value, field: &str) -> Result<BTreeSet<String>> {
    let mut rules = BTreeSet::new();
    for record in record_tables(root, "gameplay_batch")? {
        rules.extend(string_array(record, field)?.into_iter().map(str::to_owned));
    }
    Ok(rules)
}

fn verify_root(context: &Context, actual: &Value, expected: &Value) -> Result<()> {
    for field in [
        "schema_version",
        "goal",
        "reference_version",
        "baseline",
        "baseline_sha256",
        "generator",
        "ordering",
        "totals",
    ] {
        ensure!(
            actual.get(field) == expected.get(field),
            "implementation manifest has stale root field {field}"
        );
    }
    let baseline = actual
        .get("baseline")
        .and_then(Value::as_str)
        .context("implementation manifest has no baseline path")?;
    let baseline_path = safe_workspace_path(&context.workspace, baseline)?;
    let baseline_bytes = fs::read(&baseline_path)
        .with_context(|| format!("read baseline {}", baseline_path.display()))?;
    let digest = hex::encode(Sha256::digest(&baseline_bytes));
    ensure!(
        digest == BASELINE_SHA256,
        "implementation baseline digest drifted: expected {BASELINE_SHA256}, found {digest}"
    );
    let exactness = record_tables(actual, "worldgen_exactness")?;
    ensure!(
        exactness.len() == 1,
        "implementation manifest must contain one worldgen exactness record"
    );
    let contract = required_string(exactness[0], "contract", "worldgen_exactness")?;
    let contract_path = safe_workspace_path(&context.workspace, contract)?;
    let contract_bytes = fs::read(&contract_path)
        .with_context(|| format!("read exactness contract {}", contract_path.display()))?;
    let contract_digest = hex::encode(Sha256::digest(&contract_bytes));
    ensure!(
        contract_digest == WORLDGEN_EXACTNESS_SHA256,
        "worldgen exactness contract digest drifted: expected {WORLDGEN_EXACTNESS_SHA256}, found {contract_digest}"
    );
    Ok(())
}

fn verify_record_array(
    actual: &Value,
    expected: &Value,
    array: &str,
    key_field: &str,
) -> Result<()> {
    let actual_records = keyed_records(actual, array, key_field)?;
    let expected_records = keyed_records(expected, array, key_field)?;
    let actual_keys = actual_records.keys().cloned().collect::<BTreeSet<_>>();
    let expected_keys = expected_records.keys().cloned().collect::<BTreeSet<_>>();
    let missing = expected_keys
        .difference(&actual_keys)
        .cloned()
        .collect::<Vec<_>>();
    let dead = actual_keys
        .difference(&expected_keys)
        .cloned()
        .collect::<Vec<_>>();
    ensure!(
        missing.is_empty(),
        "{array} misses reference mappings: {}",
        missing.join(", ")
    );
    ensure!(
        dead.is_empty(),
        "{array} contains dead reference mappings: {}",
        dead.join(", ")
    );
    for key in expected_keys {
        let actual_record = actual_records[&key];
        let expected_record = expected_records[&key];
        ensure!(
            immutable_record(actual_record) == immutable_record(expected_record),
            "{array} mapping {key} is stale"
        );
    }
    Ok(())
}

fn keyed_records<'a>(
    root: &'a Value,
    array: &str,
    key_field: &str,
) -> Result<BTreeMap<String, &'a Table>> {
    let records = root
        .get(array)
        .and_then(Value::as_array)
        .with_context(|| format!("implementation manifest has no {array} array"))?;
    let mut keyed = BTreeMap::new();
    for value in records {
        let record = value
            .as_table()
            .with_context(|| format!("{array} contains a non-table record"))?;
        let key = record_key(record, key_field)
            .with_context(|| format!("{array} record has no stable key"))?;
        ensure!(
            keyed.insert(key.clone(), record).is_none(),
            "{array} duplicates reference mapping {key}"
        );
    }
    Ok(keyed)
}

fn record_key(record: &Table, key_field: &str) -> Option<String> {
    if key_field == "left+right" {
        let left = record.get("left")?.as_str()?;
        let right = record.get("right")?.as_str()?;
        return Some(format!("{left}\0{right}"));
    }
    record
        .get(key_field)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn immutable_record(record: &Table) -> Table {
    let mut immutable = record.clone();
    for field in PROGRESS_FIELDS {
        immutable.remove(field);
    }
    immutable
}

fn verify_batch_graph(context: &Context, root: &Value) -> Result<()> {
    let fixed_ids = fixed_batch_ids(context)?;
    let mut generated = BTreeMap::<String, Vec<String>>::new();
    for array in ["catalog_batch", "gameplay_batch", "protocol_batch"] {
        for record in record_tables(root, array)? {
            let id = required_string(record, "id", array)?;
            ensure!(
                generated
                    .insert(
                        id.to_owned(),
                        string_array(record, "depends_on")?
                            .into_iter()
                            .map(str::to_owned)
                            .collect(),
                    )
                    .is_none(),
                "generated batch ID {id} is duplicated"
            );
            ensure!(
                owned_record_count(record, array)? > 0,
                "generated batch {id} owns no records"
            );
            for field in ["responsibility", "implementation_owner", "test_owner"] {
                ensure!(
                    !required_string(record, field, array)?.trim().is_empty(),
                    "generated batch {id} has empty {field}"
                );
            }
            validate_test_owner(required_string(record, "test_owner", array)?)?;
            let closes_in = required_string(record, "closes_in", array)?;
            ensure!(
                fixed_ids.contains(closes_in),
                "generated batch {id} closes in unknown fixed batch {closes_in}"
            );
            ensure!(
                !string_array(record, "depends_on")?.contains(&closes_in),
                "generated batch {id} depends on its own closure {closes_in}"
            );
            let phase = required_usize(record, "phase", array)?;
            ensure!(
                id.starts_with(&format!("G01-P{phase}-")),
                "generated batch {id} disagrees with phase {phase}"
            );
        }
    }
    let generated_ids = generated.keys().cloned().collect::<BTreeSet<_>>();
    for (id, dependencies) in &generated {
        for dependency in dependencies {
            ensure!(
                fixed_ids.contains(dependency) || generated_ids.contains(dependency),
                "generated batch {id} depends on unknown batch {dependency}"
            );
            ensure!(id != dependency, "generated batch {id} depends on itself");
        }
    }
    ensure!(
        !has_generated_cycle(&generated),
        "generated implementation batch graph contains a cycle"
    );
    Ok(())
}

fn fixed_batch_ids(context: &Context) -> Result<BTreeSet<String>> {
    let path = context
        .workspace
        .join("docs/goals/01-audited-minecraft-26.2-status.md");
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let pattern = Regex::new(r"`(G01-P\d+-B\d+)`")?;
    let ids = pattern
        .captures_iter(&text)
        .map(|capture| capture[1].to_owned())
        .collect::<BTreeSet<_>>();
    ensure!(!ids.is_empty(), "status ledger contains no fixed batch IDs");
    Ok(ids)
}

fn has_generated_cycle(graph: &BTreeMap<String, Vec<String>>) -> bool {
    fn visit(
        node: &str,
        graph: &BTreeMap<String, Vec<String>>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) -> bool {
        if visited.contains(node) {
            return false;
        }
        if !visiting.insert(node.to_owned()) {
            return true;
        }
        if let Some(dependencies) = graph.get(node) {
            for dependency in dependencies {
                if graph.contains_key(dependency) && visit(dependency, graph, visiting, visited) {
                    return true;
                }
            }
        }
        visiting.remove(node);
        visited.insert(node.to_owned());
        false
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    graph
        .keys()
        .any(|node| visit(node, graph, &mut visiting, &mut visited))
}

fn verify_progress(workspace: &Path, root: &Value) -> Result<()> {
    let mut active_batches = BTreeSet::new();
    for array in RECORD_ARRAYS.map(|(array, _)| array) {
        for record in record_tables(root, array)? {
            if let Some(test_owner) = record.get("test_owner").and_then(Value::as_str) {
                validate_test_owner(test_owner)?;
            }
            let disposition = required_string(record, "disposition", array)?;
            let evidence = string_array(record, "evidence")?;
            let rationale = optional_string_array(record, "rationale")?;
            let alternatives = optional_string_array(record, "attempted_alternatives")?;
            let unblock = optional_string_array(record, "unblock_conditions")?;
            let batch = progress_batch(record, array)?;
            match disposition {
                "Pending" => ensure!(
                    evidence.is_empty()
                        && rationale.is_empty()
                        && alternatives.is_empty()
                        && unblock.is_empty(),
                    "{array} {batch} is Pending but carries terminal evidence"
                ),
                "InProgress" => {
                    active_batches.insert(batch.clone());
                }
                "Implemented" => ensure!(
                    !evidence.is_empty(),
                    "{array} {batch} is Implemented without evidence"
                ),
                "Verified" => {
                    ensure!(
                        !evidence.is_empty(),
                        "{array} {batch} is Verified without evidence"
                    );
                    let test_owner = required_string(record, "test_owner", array)?;
                    let test_path = safe_workspace_path(workspace, test_owner)?;
                    ensure!(
                        test_path.is_file(),
                        "{array} {batch} is Verified but test owner {} does not exist",
                        test_path.display()
                    );
                }
                "DeferredExperiment" => ensure!(
                    array == "deferred_observation",
                    "{array} {batch} illegally defers a source-specified implementation record"
                ),
                "NotApplicable" => ensure!(
                    !evidence.is_empty() && !rationale.is_empty(),
                    "{array} {batch} is NotApplicable without evidence and rationale"
                ),
                "Blocked" => ensure!(
                    !rationale.is_empty() && !alternatives.is_empty() && !unblock.is_empty(),
                    "{array} {batch} is Blocked without reason, attempted alternatives, and unblock conditions"
                ),
                other => bail!("{array} {batch} has unknown disposition {other}"),
            }
            if array == "deferred_observation" {
                ensure!(
                    disposition == "DeferredExperiment",
                    "deferred observation {batch} must remain DeferredExperiment until replaced by evidence"
                );
                ensure!(
                    !required_string(record, "policy", array)?.trim().is_empty()
                        && !required_string(record, "replacement_condition", array)?
                            .trim()
                            .is_empty(),
                    "deferred observation {batch} has no policy or replacement condition"
                );
            }
        }
    }
    ensure!(
        active_batches.len() <= 1,
        "more than one implementation batch is InProgress: {}",
        active_batches.into_iter().collect::<Vec<_>>().join(", ")
    );
    Ok(())
}

fn render_counters(root: &Value) -> Result<()> {
    let catalog = weighted_dispositions(root, "catalog_batch", "reference_ids")?;
    let gameplay = weighted_array_dispositions(root, "gameplay_batch", "slices")?;
    let surfaces = unit_dispositions(root, "surface_owner")?;
    let joins = unit_dispositions(root, "join_owner")?;
    let protocol = unit_dispositions(root, "protocol_batch")?;
    let worldgen = unit_dispositions(root, "worldgen_exactness")?;
    let optional = record_tables(root, "protocol_batch")?
        .into_iter()
        .filter(|record| {
            record.get("implementation_mode").and_then(Value::as_str) == Some("ConfigurationGate")
        })
        .count();

    println!(
        "implementation catalog coverage: 9078 IDs in 32 batches; dispositions {}",
        display_counts(&catalog)
    );
    println!(
        "implementation gameplay coverage: 331 slices in 55 batches; dispositions {}",
        display_counts(&gameplay)
    );
    println!(
        "implementation behavior-surface coverage: 10 surfaces; dispositions {}",
        display_counts(&surfaces)
    );
    println!(
        "implementation cross-system-join coverage: 36 joins; dispositions {}",
        display_counts(&joins)
    );
    println!(
        "implementation protocol coverage: 58 families ({} required, {optional} optional gates); dispositions {}",
        58 - optional,
        display_counts(&protocol)
    );
    println!(
        "implementation worldgen exactness: dispositions {}",
        display_counts(&worldgen)
    );
    Ok(())
}

fn weighted_dispositions(
    root: &Value,
    array: &str,
    weight_field: &str,
) -> Result<BTreeMap<String, usize>> {
    let mut counts = BTreeMap::new();
    for record in record_tables(root, array)? {
        let disposition = required_string(record, "disposition", array)?.to_owned();
        *counts.entry(disposition).or_default() += required_usize(record, weight_field, array)?;
    }
    Ok(counts)
}

fn weighted_array_dispositions(
    root: &Value,
    array: &str,
    weight_field: &str,
) -> Result<BTreeMap<String, usize>> {
    let mut counts = BTreeMap::new();
    for record in record_tables(root, array)? {
        let disposition = required_string(record, "disposition", array)?.to_owned();
        let weight = record
            .get(weight_field)
            .and_then(Value::as_array)
            .with_context(|| format!("{array} record has no {weight_field} array"))?
            .len();
        *counts.entry(disposition).or_default() += weight;
    }
    Ok(counts)
}

fn unit_dispositions(root: &Value, array: &str) -> Result<BTreeMap<String, usize>> {
    let mut counts = BTreeMap::new();
    for record in record_tables(root, array)? {
        let disposition = required_string(record, "disposition", array)?.to_owned();
        *counts.entry(disposition).or_default() += 1;
    }
    Ok(counts)
}

fn display_counts(counts: &BTreeMap<String, usize>) -> String {
    let values = counts
        .iter()
        .map(|(status, count)| format!("{status}: {count}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{values}}}")
}

fn record_tables<'a>(root: &'a Value, array: &str) -> Result<Vec<&'a Table>> {
    root.get(array)
        .and_then(Value::as_array)
        .with_context(|| format!("implementation manifest has no {array} array"))?
        .iter()
        .map(|record| {
            record
                .as_table()
                .with_context(|| format!("{array} contains a non-table record"))
        })
        .collect()
}

fn required_string<'a>(record: &'a Table, field: &str, owner: &str) -> Result<&'a str> {
    record
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("{owner} record has no string {field}"))
}

fn required_usize(record: &Table, field: &str, owner: &str) -> Result<usize> {
    let value = record
        .get(field)
        .and_then(Value::as_integer)
        .with_context(|| format!("{owner} record has no integer {field}"))?;
    usize::try_from(value).with_context(|| format!("{owner} record has invalid {field}"))
}

fn string_array<'a>(record: &'a Table, field: &str) -> Result<Vec<&'a str>> {
    record
        .get(field)
        .and_then(Value::as_array)
        .with_context(|| format!("record has no {field} array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .with_context(|| format!("{field} contains a non-string value"))
        })
        .collect()
}

fn optional_string_array<'a>(record: &'a Table, field: &str) -> Result<Vec<&'a str>> {
    match record.get(field) {
        Some(_) => string_array(record, field),
        None => Ok(Vec::new()),
    }
}

fn owned_record_count(record: &Table, array: &str) -> Result<usize> {
    match array {
        "catalog_batch" => required_usize(record, "reference_ids", array),
        "gameplay_batch" => record
            .get("slices")
            .and_then(Value::as_array)
            .map(Vec::len)
            .context("gameplay_batch record has no slices array"),
        "protocol_batch" => required_usize(record, "packets", array),
        _ => bail!("unknown generated record array {array}"),
    }
}

fn progress_batch(record: &Table, array: &str) -> Result<String> {
    if let Some(id) = record.get("id").and_then(Value::as_str) {
        return Ok(id.to_owned());
    }
    if let Some(batch) = record.get("implementation_batch").and_then(Value::as_str) {
        return Ok(batch.to_owned());
    }
    if let Some(slice) = record.get("slice").and_then(Value::as_str) {
        return Ok(slice.to_owned());
    }
    bail!("{array} record has no batch identity")
}

fn validate_test_owner(path: &str) -> Result<()> {
    let candidate = Path::new(path);
    ensure!(
        !candidate.is_absolute(),
        "test owner path is absolute: {path}"
    );
    ensure!(
        !candidate
            .components()
            .any(|component| matches!(component, Component::ParentDir)),
        "test owner escapes the workspace: {path}"
    );
    ensure!(
        candidate.extension().and_then(|value| value.to_str()) == Some("rs"),
        "test owner is not a Rust source path: {path}"
    );
    Ok(())
}

fn safe_workspace_path(workspace: &Path, relative: &str) -> Result<PathBuf> {
    validate_relative_path(relative)?;
    Ok(workspace.join(relative))
}

fn validate_relative_path(path: &str) -> Result<()> {
    let candidate = Path::new(path);
    ensure!(
        !candidate.is_absolute(),
        "workspace path is absolute: {path}"
    );
    ensure!(
        !candidate.components().any(|component| matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )),
        "workspace path escapes the workspace: {path}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn immutable_records_ignore_only_progress_fields() {
        let value: Value = toml::from_str(
            r#"
            id = "G01-P1-D001"
            disposition = "Verified"
            evidence = ["test:x"]
            rationale = ["reason"]
            reference_ids = 1
            "#,
        )
        .unwrap();
        let immutable = immutable_record(value.as_table().unwrap());

        assert_eq!(
            immutable.get("id").and_then(Value::as_str),
            Some("G01-P1-D001")
        );
        assert_eq!(
            immutable.get("reference_ids").and_then(Value::as_integer),
            Some(1)
        );
        assert!(!immutable.contains_key("disposition"));
        assert!(!immutable.contains_key("evidence"));
        assert!(!immutable.contains_key("rationale"));
    }

    #[test]
    fn generated_cycle_detection_finds_cycles() {
        let cyclic = BTreeMap::from([
            ("a".to_owned(), vec!["b".to_owned()]),
            ("b".to_owned(), vec!["a".to_owned()]),
        ]);
        let acyclic = BTreeMap::from([
            ("a".to_owned(), vec![]),
            ("b".to_owned(), vec!["a".to_owned()]),
        ]);

        assert!(has_generated_cycle(&cyclic));
        assert!(!has_generated_cycle(&acyclic));
    }

    #[test]
    fn test_owner_paths_are_scoped_rust_files() {
        assert!(validate_test_owner("crates/ferrite-world/tests/chunk.rs").is_ok());
        assert!(validate_test_owner("../outside.rs").is_err());
        assert!(validate_test_owner("tests/vector.json").is_err());
    }

    #[test]
    fn counters_are_stably_rendered() {
        let counts = BTreeMap::from([("Pending".to_owned(), 4), ("Verified".to_owned(), 2)]);
        assert_eq!(display_counts(&counts), "{Pending: 4, Verified: 2}");
    }

    #[test]
    fn record_coverage_rejects_missing_duplicate_dead_and_stale_mappings() {
        let expected = manifest_with_catalog_records(&[("block", 1)]);
        let missing = manifest_with_catalog_records(&[]);
        let duplicate: Value = toml::from_str(
            r#"
            catalog_batch = [
                { reference_kind = "block", reference_ids = 1, disposition = "Pending", evidence = [] },
                { reference_kind = "block", reference_ids = 1, disposition = "Pending", evidence = [] },
            ]
            "#,
        )
        .unwrap();
        let dead = manifest_with_catalog_records(&[("block", 1), ("item", 1)]);
        let stale = manifest_with_catalog_records(&[("block", 2)]);

        assert!(
            verify_record_array(&missing, &expected, "catalog_batch", "reference_kind").is_err()
        );
        assert!(
            verify_record_array(&duplicate, &expected, "catalog_batch", "reference_kind").is_err()
        );
        assert!(verify_record_array(&dead, &expected, "catalog_batch", "reference_kind").is_err());
        assert!(verify_record_array(&stale, &expected, "catalog_batch", "reference_kind").is_err());
    }

    #[test]
    fn progress_rejects_false_completion_and_pending_evidence() {
        let directory = tempdir().unwrap();
        let verified_without_evidence = progress_manifest("Verified", &[]);
        let pending_with_evidence = progress_manifest("Pending", &["uncommitted"]);

        assert!(verify_progress(directory.path(), &verified_without_evidence).is_err());
        assert!(verify_progress(directory.path(), &pending_with_evidence).is_err());
    }

    fn manifest_with_catalog_records(records: &[(&str, i64)]) -> Value {
        let records = records
            .iter()
            .map(|(kind, count)| {
                Value::Table(Table::from_iter([
                    (
                        "reference_kind".to_owned(),
                        Value::String((*kind).to_owned()),
                    ),
                    ("reference_ids".to_owned(), Value::Integer(*count)),
                    (
                        "disposition".to_owned(),
                        Value::String("Pending".to_owned()),
                    ),
                    ("evidence".to_owned(), Value::Array(Vec::new())),
                ]))
            })
            .collect();
        Value::Table(Table::from_iter([(
            "catalog_batch".to_owned(),
            Value::Array(records),
        )]))
    }

    fn progress_manifest(disposition: &str, evidence: &[&str]) -> Value {
        let record = Value::Table(Table::from_iter([
            ("id".to_owned(), Value::String("G01-P1-D001".to_owned())),
            (
                "test_owner".to_owned(),
                Value::String("tests/catalog.rs".to_owned()),
            ),
            (
                "disposition".to_owned(),
                Value::String(disposition.to_owned()),
            ),
            (
                "evidence".to_owned(),
                Value::Array(
                    evidence
                        .iter()
                        .map(|value| Value::String((*value).to_owned()))
                        .collect(),
                ),
            ),
        ]));
        Value::Table(Table::from_iter([
            ("catalog_batch".to_owned(), Value::Array(vec![record])),
            ("gameplay_batch".to_owned(), Value::Array(Vec::new())),
            ("deferred_observation".to_owned(), Value::Array(Vec::new())),
            ("worldgen_exactness".to_owned(), Value::Array(Vec::new())),
            ("surface_owner".to_owned(), Value::Array(Vec::new())),
            ("join_owner".to_owned(), Value::Array(Vec::new())),
            ("protocol_batch".to_owned(), Value::Array(Vec::new())),
        ]))
    }
}
