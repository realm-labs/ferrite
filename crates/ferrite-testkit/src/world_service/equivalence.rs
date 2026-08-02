//! Executable audit of the intentionally deferred worldgen-equivalence observation.

use std::fs;
use std::path::Path;

use toml::Value;

use crate::world_service::conformance::architectural_generation_digest;
use crate::world_service::fixtures::content_manifest;

const EXPERIMENTS: [&str; 3] = ["EXP-WGEN-001", "EXP-WGEN-005", "EXP-WGEN-006"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenEquivalenceReport {
    pub experiments: usize,
    pub planned_repeats: i64,
    pub source_specified_slices: i64,
    pub source_inconclusive_slices: i64,
    pub project_seed_is_deterministic: bool,
    pub distinct_project_seeds_diverge: bool,
    pub same_seed_semantic_identity_required: bool,
    pub differential_oracle_implemented: bool,
    pub declared_population_verified: bool,
    pub statistical_thresholds_can_close_compatibility: bool,
}

#[must_use]
pub fn run_worldgen_equivalence_boundary() -> WorldgenEquivalenceReport {
    let definitions =
        parse("../../docs/reference/minecraft-java-26.2/experiments/definitions.toml");
    let experiment_records = definitions
        .get("experiment")
        .and_then(Value::as_array)
        .expect("experiment definitions contain records");
    let mut planned_repeats = 0;
    for identity in EXPERIMENTS {
        let experiment = find_by(experiment_records, "id", identity);
        assert_eq!(string(experiment, "mode"), "dedicated");
        assert_eq!(string(experiment, "status"), "planned");
        assert_eq!(array_strings(experiment, "rules"), ["WGEN-PIPELINE-001"]);
        assert!(!array(experiment, "initial_state").is_empty());
        assert!(!array(experiment, "action").is_empty());
        assert!(!array(experiment, "observation").is_empty());
        assert!(!array(experiment, "expected").is_empty());
        planned_repeats += integer(experiment, "repeats");
    }
    assert_eq!(planned_repeats, 8_200);

    let implementation = parse("../../goals/minecraft-java-26.2/implementation.toml");
    let deferred_records = implementation
        .get("deferred_observation")
        .and_then(Value::as_array)
        .expect("implementation manifest contains deferred observations");
    let deferred = find_by(deferred_records, "slice", "WGEN-PIPELINE-EQUIVALENCE-001");
    assert_eq!(string(deferred, "disposition"), "DeferredExperiment");
    assert_eq!(array_strings(deferred, "experiments"), EXPERIMENTS);
    assert_eq!(
        string(deferred, "policy"),
        "Require same-input normalized semantic identity; statistical similarity is diagnostic only."
    );
    assert_eq!(
        string(deferred, "replacement_condition"),
        "Resolve only when committed official/Ferrite populations have zero unexplained semantic divergence."
    );
    let exactness = implementation
        .get("worldgen_exactness")
        .and_then(Value::as_array)
        .expect("implementation manifest contains worldgen exactness")
        .first()
        .expect("implementation manifest contains one worldgen exactness record");
    assert_eq!(string(exactness, "id"), "G01-P8-B4");
    assert_eq!(
        string(exactness, "acceptance"),
        "ZeroUnexplainedSemanticDivergence"
    );
    assert_eq!(string(exactness, "disposition"), "Implemented");

    let gameplay = implementation
        .get("gameplay_batch")
        .and_then(Value::as_array)
        .expect("implementation manifest contains gameplay batches");
    let world = gameplay
        .iter()
        .filter(|record| string(record, "subsystem") == "world")
        .collect::<Vec<_>>();
    assert!(
        world
            .iter()
            .all(|record| string(record, "disposition") == "Verified")
    );
    let source_specified_slices = world
        .iter()
        .map(|record| integer(record, "source_specified"))
        .sum();
    let source_inconclusive_slices = world
        .iter()
        .map(|record| integer(record, "source_inconclusive"))
        .sum();

    let manifest = content_manifest();
    let first = architectural_generation_digest(manifest, 0x2602);
    let replay = architectural_generation_digest(manifest, 0x2602);
    let distinct = architectural_generation_digest(manifest, 0x2603);
    WorldgenEquivalenceReport {
        experiments: EXPERIMENTS.len(),
        planned_repeats,
        source_specified_slices,
        source_inconclusive_slices,
        project_seed_is_deterministic: first == replay,
        distinct_project_seeds_diverge: first != distinct,
        same_seed_semantic_identity_required: true,
        differential_oracle_implemented: true,
        declared_population_verified: false,
        statistical_thresholds_can_close_compatibility: false,
    }
}

fn parse(relative: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
        .parse()
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
}

fn find_by<'a>(records: &'a [Value], field: &str, expected: &str) -> &'a Value {
    records
        .iter()
        .find(|record| string(record, field) == expected)
        .unwrap_or_else(|| panic!("missing record with {field}={expected}"))
}

fn string<'a>(record: &'a Value, field: &str) -> &'a str {
    record
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("record field {field} is not a string"))
}

fn integer(record: &Value, field: &str) -> i64 {
    record
        .get(field)
        .and_then(Value::as_integer)
        .unwrap_or_else(|| panic!("record field {field} is not an integer"))
}

fn array<'a>(record: &'a Value, field: &str) -> &'a [Value] {
    record
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("record field {field} is not an array"))
}

fn array_strings<'a>(record: &'a Value, field: &str) -> Vec<&'a str> {
    array(record, field)
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("record field {field} contains a non-string"))
        })
        .collect()
}
