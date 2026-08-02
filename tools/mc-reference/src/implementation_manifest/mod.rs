use crate::{Context, ImplementationManifestCommand};
use anyhow::{Context as _, Result, ensure};
use globset::{Glob, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

mod verify;

const OUTPUT_RELATIVE: &str = "goals/minecraft-java-26.2/implementation.toml";
const BASELINE_RELATIVE: &str = "goals/minecraft-java-26.2/reference-baseline.toml";
const WORLDGEN_EXACTNESS_RELATIVE: &str = "goals/minecraft-java-26.2/worldgen-exactness.toml";
const WORLDGEN_EXACTNESS_SHA256: &str =
    "a63d04184be0fa73cebe8a2ef715b0932b44f3d055955f95ea83de4b97a510e6";
const BASELINE_SHA256: &str = "31f5e58c029337aaf4c7bc8bba253a5ce8ecd6edbee30cd41989e94a9345c678";
const GENERATOR: &str =
    "cargo run -q -p mc-reference --bin mc-ref -- implementation-manifest render";

#[derive(Debug, Deserialize)]
struct Catalog {
    category: Vec<CatalogCategory>,
}

#[derive(Debug, Deserialize)]
struct CatalogCategory {
    kind: String,
    expected_count: usize,
    ids_sha1: String,
    family: Vec<CatalogFamily>,
}

#[derive(Debug, Deserialize)]
struct CatalogFamily {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Completion {
    slice: Vec<CompletionSlice>,
}

#[derive(Debug, Deserialize)]
struct CompletionSlice {
    id: String,
    subsystem: String,
    parents: Vec<String>,
    leaves: Vec<String>,
    status: String,
}

#[derive(Debug, Deserialize)]
struct SurfaceMap {
    surface: Vec<Surface>,
}

#[derive(Debug, Deserialize)]
struct Surface {
    id: String,
    kind: String,
}

#[derive(Debug, Deserialize)]
struct JoinMap {
    join: Vec<Join>,
}

#[derive(Debug, Deserialize)]
struct Join {
    left: String,
    right: String,
}

#[derive(Debug, Deserialize)]
struct ProtocolCompletion {
    family: Vec<ProtocolFamily>,
}

#[derive(Debug, Deserialize)]
struct ProtocolFamily {
    id: String,
    level: String,
    state: String,
    direction: String,
    patterns: Vec<String>,
    status: String,
    responsibility: String,
}

#[derive(Debug, Clone)]
struct Packet {
    state: String,
    direction: String,
    identity: String,
}

#[derive(Debug, Serialize)]
struct ImplementationManifest {
    schema_version: u32,
    goal: &'static str,
    reference_version: &'static str,
    baseline: &'static str,
    baseline_sha256: &'static str,
    generator: &'static str,
    ordering: &'static str,
    totals: Totals,
    worldgen_exactness: Vec<WorldgenExactness>,
    catalog_batch: Vec<CatalogBatch>,
    gameplay_batch: Vec<GameplayBatch>,
    deferred_observation: Vec<DeferredObservation>,
    surface_owner: Vec<SurfaceOwner>,
    join_owner: Vec<JoinOwner>,
    protocol_batch: Vec<ProtocolBatch>,
}

#[derive(Debug, Deserialize)]
struct WorldgenExactnessContract {
    normalization_schema: String,
    acceptance: String,
    oracle_batch: String,
    population_batch: String,
    semantic_fields: Vec<String>,
    population: Vec<WorldgenPopulation>,
}

#[derive(Debug, Deserialize)]
struct WorldgenPopulation {
    id: String,
    request_plans: Vec<String>,
}

#[derive(Debug, Serialize)]
struct WorldgenExactness {
    id: String,
    responsibility: String,
    closes_in: String,
    implementation_owner: &'static str,
    test_owner: &'static str,
    disposition: &'static str,
    evidence: Vec<String>,
    contract: &'static str,
    contract_sha256: &'static str,
    normalization_schema: String,
    acceptance: String,
    semantic_fields: Vec<String>,
    populations: Vec<String>,
    request_plans: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Totals {
    catalog_ids: usize,
    catalog_batches: usize,
    catalog_families: usize,
    gameplay_slices: usize,
    gameplay_batches: usize,
    source_specified_slices: usize,
    source_inconclusive_slices: usize,
    deferred_observations: usize,
    behavior_surfaces: usize,
    cross_system_joins: usize,
    protocol_packets: usize,
    protocol_families: usize,
    required_protocol_families: usize,
    optional_protocol_families: usize,
}

#[derive(Debug, Serialize)]
struct CatalogBatch {
    id: String,
    phase: u8,
    responsibility: String,
    depends_on: Vec<String>,
    closes_in: String,
    implementation_owner: &'static str,
    test_owner: String,
    disposition: &'static str,
    evidence: Vec<String>,
    reference_kind: String,
    reference_ids: usize,
    reference_digest: String,
    reference_families: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GameplayBatch {
    id: String,
    phase: u8,
    responsibility: String,
    depends_on: Vec<String>,
    closes_in: String,
    implementation_owner: &'static str,
    test_owner: String,
    disposition: &'static str,
    evidence: Vec<String>,
    subsystem: String,
    primary_parent: String,
    slices: Vec<String>,
    parents: Vec<String>,
    leaves: Vec<String>,
    source_specified: usize,
    source_inconclusive: usize,
}

#[derive(Debug, Serialize)]
struct DeferredObservation {
    slice: &'static str,
    source_part_batch: String,
    experiments: Vec<&'static str>,
    implementation_owner: &'static str,
    test_owner: String,
    disposition: &'static str,
    evidence: Vec<String>,
    policy: &'static str,
    replacement_condition: &'static str,
}

#[derive(Debug, Serialize)]
struct SurfaceOwner {
    reference: String,
    kind: String,
    implementation_batch: String,
    implementation_owner: &'static str,
    test_owner: String,
    disposition: &'static str,
    evidence: Vec<String>,
}

#[derive(Debug, Serialize)]
struct JoinOwner {
    left: String,
    right: String,
    implementation_batch: String,
    implementation_owner: &'static str,
    test_owner: String,
    disposition: &'static str,
    evidence: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProtocolBatch {
    id: String,
    phase: u8,
    responsibility: String,
    depends_on: Vec<String>,
    closes_in: String,
    implementation_owner: &'static str,
    test_owner: String,
    disposition: &'static str,
    evidence: Vec<String>,
    reference_family: String,
    level: String,
    state: String,
    direction: String,
    source_status: String,
    source_responsibility: String,
    implementation_mode: &'static str,
    packets: usize,
}

pub(crate) fn run(context: &Context, command: ImplementationManifestCommand) -> Result<()> {
    match command {
        ImplementationManifestCommand::Render => render(context),
        ImplementationManifestCommand::MigrateWorldgenExactness => {
            migrate_worldgen_exactness(context)
        }
        ImplementationManifestCommand::Verify => verify::run(context),
    }
}

fn materialize(context: &Context) -> Result<ImplementationManifest> {
    let catalog: Catalog = read_toml(&context.reference.join("catalog/catalog.toml"))?;
    let completion: Completion = read_toml(&context.reference.join("completion.toml"))?;
    let surfaces: SurfaceMap = read_toml(&context.reference.join("behavior-surfaces.toml"))?;
    let joins: JoinMap = read_toml(&context.reference.join("cross-system-joins.toml"))?;
    let protocol: ProtocolCompletion =
        read_toml(&context.reference.join("protocol/completion.toml"))?;
    let packets = read_packets(context)?;
    let worldgen_contract: WorldgenExactnessContract =
        read_toml(&context.workspace.join(WORLDGEN_EXACTNESS_RELATIVE))?;

    let catalog_batch = catalog_batches(catalog.category);
    let gameplay_batch = gameplay_batches(completion.slice)?;
    let deferred_observation = deferred_observations(&gameplay_batch)?;
    let surface_owner = surface_owners(surfaces.surface);
    let join_owner = join_owners(joins.join);
    let protocol_batch = protocol_batches(protocol.family, &packets)?;
    let worldgen_exactness = vec![worldgen_exactness(worldgen_contract)];

    let totals = Totals {
        catalog_ids: catalog_batch.iter().map(|batch| batch.reference_ids).sum(),
        catalog_batches: catalog_batch.len(),
        catalog_families: catalog_batch
            .iter()
            .map(|batch| batch.reference_families.len())
            .sum(),
        gameplay_slices: gameplay_batch.iter().map(|batch| batch.slices.len()).sum(),
        gameplay_batches: gameplay_batch.len(),
        source_specified_slices: gameplay_batch
            .iter()
            .map(|batch| batch.source_specified)
            .sum(),
        source_inconclusive_slices: gameplay_batch
            .iter()
            .map(|batch| batch.source_inconclusive)
            .sum(),
        deferred_observations: deferred_observation.len(),
        behavior_surfaces: surface_owner.len(),
        cross_system_joins: join_owner.len(),
        protocol_packets: protocol_batch.iter().map(|batch| batch.packets).sum(),
        protocol_families: protocol_batch.len(),
        required_protocol_families: protocol_batch
            .iter()
            .filter(|batch| batch.source_responsibility == "Required")
            .count(),
        optional_protocol_families: protocol_batch
            .iter()
            .filter(|batch| batch.source_responsibility == "Optional")
            .count(),
    };
    validate_totals(&totals)?;

    Ok(ImplementationManifest {
        schema_version: 2,
        goal: "Goal 01",
        reference_version: "26.2",
        baseline: BASELINE_RELATIVE,
        baseline_sha256: BASELINE_SHA256,
        generator: GENERATOR,
        ordering: "Batch IDs and records are sorted by phase, reference owner, and reference ID.",
        totals,
        worldgen_exactness,
        catalog_batch,
        gameplay_batch,
        deferred_observation,
        surface_owner,
        join_owner,
        protocol_batch,
    })
}

fn worldgen_exactness(contract: WorldgenExactnessContract) -> WorldgenExactness {
    let mut populations = contract
        .population
        .iter()
        .map(|population| population.id.clone())
        .collect::<Vec<_>>();
    populations.sort();
    let mut request_plans = contract
        .population
        .into_iter()
        .flat_map(|population| population.request_plans)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    request_plans.sort();
    WorldgenExactness {
        id: contract.oracle_batch,
        responsibility: "Build the locked official 26.2/Ferrite semantic differential oracle"
            .to_owned(),
        closes_in: contract.population_batch,
        implementation_owner: "ferrite-testkit",
        test_owner: "apps/behavior-runner/tests/worldgen_differential_oracle.rs",
        disposition: "Pending",
        evidence: Vec::new(),
        contract: WORLDGEN_EXACTNESS_RELATIVE,
        contract_sha256: WORLDGEN_EXACTNESS_SHA256,
        normalization_schema: contract.normalization_schema,
        acceptance: contract.acceptance,
        semantic_fields: contract.semantic_fields,
        populations,
        request_plans,
    }
}

fn migrate_worldgen_exactness(context: &Context) -> Result<()> {
    let output = context.workspace.join(OUTPUT_RELATIVE);
    let existing = fs::read_to_string(&output)
        .with_context(|| format!("read implementation manifest {}", output.display()))?;
    let parsed: toml::Value = toml::from_str(&existing).context("parse implementation manifest")?;
    if parsed
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        == Some(2)
    {
        ensure!(
            parsed.get("worldgen_exactness").is_some(),
            "schema 2 implementation manifest has no worldgen_exactness record"
        );
        println!(
            "worldgen exactness migration unchanged: {}",
            output.display()
        );
        return Ok(());
    }
    ensure!(
        parsed
            .get("schema_version")
            .and_then(toml::Value::as_integer)
            == Some(1),
        "worldgen exactness migration requires implementation schema 1"
    );
    ensure!(
        parsed.get("worldgen_exactness").is_none(),
        "schema 1 implementation manifest already contains worldgen_exactness"
    );
    let mut record = materialize(context)?
        .worldgen_exactness
        .into_iter()
        .next()
        .context("materialized implementation manifest has no worldgen record")?;
    record.disposition = "InProgress";
    #[derive(Serialize)]
    struct MigrationFragment {
        worldgen_exactness: Vec<WorldgenExactness>,
    }
    let fragment = toml::to_string_pretty(&MigrationFragment {
        worldgen_exactness: vec![record],
    })?;
    let migrated = existing.replacen("schema_version = 1", "schema_version = 2", 1);
    ensure!(
        migrated != existing,
        "schema_version root line was not found"
    );
    let migrated = migrated
        .replace(
            "Do not claim block-for-block same-seed world-generation identity.",
            "Require same-input normalized semantic identity; statistical similarity is diagnostic only.",
        )
        .replace(
            "Replace only with committed, named statistical equivalence thresholds.",
            "Resolve only when committed official/Ferrite populations have zero unexplained semantic divergence.",
        );
    let insertion = migrated
        .find("[[catalog_batch]]")
        .context("implementation manifest has no catalog batch insertion point")?;
    let mut output_text = String::with_capacity(migrated.len() + fragment.len() + 2);
    output_text.push_str(&migrated[..insertion]);
    output_text.push_str(&fragment);
    output_text.push('\n');
    output_text.push_str(&migrated[insertion..]);
    fs::write(&output, output_text).with_context(|| {
        format!(
            "write migrated implementation manifest {}",
            output.display()
        )
    })?;
    println!("worldgen exactness migration written: {}", output.display());
    Ok(())
}

fn render(context: &Context) -> Result<()> {
    let manifest = materialize(context)?;
    let body = toml::to_string_pretty(&manifest).context("serialize implementation manifest")?;
    let rendered = format!(
        "# @generated by `{GENERATOR}`; do not edit by hand.\n\
         # Reference documents remain normative; this file records implementation ownership only.\n\n\
         {body}"
    );
    let output = context.workspace.join(OUTPUT_RELATIVE);
    if let Ok(existing) = fs::read_to_string(&output) {
        if existing == rendered {
            println!("implementation manifest unchanged: {}", output.display());
            return Ok(());
        }
        ensure!(
            !contains_implementation_progress(&existing)?,
            "refusing to overwrite implementation progress in {}; migrate the changed reference mapping in a dedicated batch",
            output.display()
        );
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, rendered)
        .with_context(|| format!("write implementation manifest {}", output.display()))?;
    println!("implementation manifest rendered: {}", output.display());
    Ok(())
}

fn contains_implementation_progress(text: &str) -> Result<bool> {
    let manifest: toml::Value =
        toml::from_str(text).context("parse existing implementation manifest")?;
    for array in [
        "catalog_batch",
        "gameplay_batch",
        "deferred_observation",
        "worldgen_exactness",
        "surface_owner",
        "join_owner",
        "protocol_batch",
    ] {
        let records = manifest
            .get(array)
            .and_then(toml::Value::as_array)
            .with_context(|| format!("existing implementation manifest has no {array}"))?;
        for record in records {
            let record = record
                .as_table()
                .with_context(|| format!("existing {array} contains a non-table record"))?;
            let initial_disposition = if array == "deferred_observation" {
                "DeferredExperiment"
            } else {
                "Pending"
            };
            if record.get("disposition").and_then(toml::Value::as_str) != Some(initial_disposition)
            {
                return Ok(true);
            }
            for field in [
                "evidence",
                "rationale",
                "attempted_alternatives",
                "unblock_conditions",
            ] {
                if record
                    .get(field)
                    .and_then(toml::Value::as_array)
                    .is_some_and(|values| !values.is_empty())
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn read_toml<T>(path: &std::path::Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn catalog_batches(categories: Vec<CatalogCategory>) -> Vec<CatalogBatch> {
    let mut categories = categories;
    categories.sort_by(|left, right| left.kind.cmp(&right.kind));
    categories
        .into_iter()
        .enumerate()
        .map(|(index, mut category)| {
            category
                .family
                .sort_by(|left, right| left.name.cmp(&right.name));
            let slug = slug(&category.kind);
            CatalogBatch {
                id: format!("G01-P1-D{:03}", index + 1),
                phase: 1,
                responsibility: format!("Import and lower the {} catalog", category.kind),
                depends_on: vec!["G01-P1-B3".to_owned()],
                closes_in: "G01-P1-B4".to_owned(),
                implementation_owner: "ferrite-registry",
                test_owner: format!("crates/ferrite-registry/tests/catalog/{slug}.rs"),
                disposition: "Pending",
                evidence: Vec::new(),
                reference_kind: category.kind,
                reference_ids: category.expected_count,
                reference_digest: category.ids_sha1,
                reference_families: category
                    .family
                    .into_iter()
                    .map(|family| family.name)
                    .collect(),
            }
        })
        .collect()
}

fn gameplay_batches(slices: Vec<CompletionSlice>) -> Result<Vec<GameplayBatch>> {
    let mut groups = BTreeMap::<(u8, String, String), Vec<CompletionSlice>>::new();
    for slice in slices {
        let phase = subsystem_phase(&slice.subsystem)?;
        let primary_parent = slice
            .parents
            .first()
            .with_context(|| format!("{} has no parent rule", slice.id))?
            .clone();
        groups
            .entry((phase, slice.subsystem.clone(), primary_parent))
            .or_default()
            .push(slice);
    }

    let mut phase_indexes = BTreeMap::<u8, usize>::new();
    let mut batches = Vec::new();
    for ((phase, subsystem, primary_parent), mut slices) in groups {
        slices.sort_by(|left, right| left.id.cmp(&right.id));
        let index = phase_indexes.entry(phase).or_default();
        *index += 1;
        let source_specified = slices
            .iter()
            .filter(|slice| slice.status == "SourceSpecified")
            .count();
        let source_inconclusive = slices
            .iter()
            .filter(|slice| slice.status == "SourceInconclusive")
            .count();
        ensure!(
            source_specified + source_inconclusive == slices.len(),
            "{subsystem}/{primary_parent} contains a non-ready source slice"
        );
        let parents = sorted_union(slices.iter().flat_map(|slice| slice.parents.iter()));
        let leaves = sorted_union(slices.iter().flat_map(|slice| slice.leaves.iter()));
        let test_root = gameplay_test_root(&subsystem);
        batches.push(GameplayBatch {
            id: format!("G01-P{phase}-S{index:03}"),
            phase,
            responsibility: format!(
                "Implement {subsystem} slices primarily owned by {primary_parent}"
            ),
            depends_on: vec![phase_dependency(phase).to_owned()],
            closes_in: phase_closure(phase).to_owned(),
            implementation_owner: gameplay_implementation_owner(&subsystem),
            test_owner: format!("{test_root}/{}.rs", slug(&primary_parent)),
            disposition: "Pending",
            evidence: Vec::new(),
            subsystem,
            primary_parent,
            slices: slices.into_iter().map(|slice| slice.id).collect(),
            parents,
            leaves,
            source_specified,
            source_inconclusive,
        });
    }
    Ok(batches)
}

fn deferred_observations(batches: &[GameplayBatch]) -> Result<Vec<DeferredObservation>> {
    let definitions = [
        (
            "SIM-SCHEDULED-TICKS-001",
            vec!["EXP-SIM-002"],
            "Do not claim a vanilla cross-chunk restored-tick tie-break.",
            "Replace only with a committed EXP-SIM-002 observation.",
        ),
        (
            "ENV-LIGHTING-001",
            vec!["EXP-ENV-004"],
            "Do not claim a universal mutation-to-render latency bound.",
            "Replace only with a profile-scoped EXP-ENV-004 observation.",
        ),
        (
            "PLY-BLOCK-BREAK-001",
            vec!["EXP-PLY-003"],
            "Preserve the specified ACK-before-block-update order without claiming a rendered transient.",
            "Replace only with a committed EXP-PLY-003 frame observation.",
        ),
        (
            "WGEN-PIPELINE-EQUIVALENCE-001",
            vec!["EXP-WGEN-001", "EXP-WGEN-005", "EXP-WGEN-006"],
            "Require same-input normalized semantic identity; statistical similarity is diagnostic only.",
            "Resolve only when committed official/Ferrite populations have zero unexplained semantic divergence.",
        ),
    ];
    definitions
        .into_iter()
        .map(
            |(slice, experiments, policy, replacement_condition)| -> Result<_> {
                let source_part_batch = batches
                    .iter()
                    .find(|batch| batch.slices.iter().any(|candidate| candidate == slice))
                    .with_context(|| format!("deferred slice {slice} has no gameplay batch"))?
                    .id
                    .clone();
                Ok(DeferredObservation {
                    slice,
                    source_part_batch,
                    experiments,
                    implementation_owner: gameplay_implementation_owner(
                        batches
                            .iter()
                            .find(|batch| batch.slices.iter().any(|candidate| candidate == slice))
                            .map(|batch| batch.subsystem.as_str())
                            .context("deferred source batch disappeared")?,
                    ),
                    test_owner: format!(
                        "apps/behavior-runner/tests/experiments/{}.rs",
                        slug(slice)
                    ),
                    disposition: "DeferredExperiment",
                    evidence: Vec::new(),
                    policy,
                    replacement_condition,
                })
            },
        )
        .collect()
}

fn surface_owners(mut surfaces: Vec<Surface>) -> Vec<SurfaceOwner> {
    surfaces.sort_by(|left, right| left.id.cmp(&right.id));
    surfaces
        .into_iter()
        .map(|surface| {
            let phase = surface_phase(&surface.kind);
            SurfaceOwner {
                test_owner: format!(
                    "apps/behavior-runner/tests/surfaces/{}.rs",
                    slug(&surface.kind)
                ),
                implementation_batch: phase_closure(phase).to_owned(),
                implementation_owner: surface_implementation_owner(&surface.kind),
                reference: surface.id,
                kind: surface.kind,
                disposition: "Pending",
                evidence: Vec::new(),
            }
        })
        .collect()
}

fn join_owners(mut joins: Vec<Join>) -> Vec<JoinOwner> {
    joins.sort_by(|left, right| (&left.left, &left.right).cmp(&(&right.left, &right.right)));
    joins
        .into_iter()
        .map(|join| {
            let phase = surface_phase(&join.left).max(surface_phase(&join.right));
            JoinOwner {
                test_owner: format!(
                    "apps/behavior-runner/tests/joins/{}_{}.rs",
                    slug(&join.left),
                    slug(&join.right)
                ),
                implementation_batch: phase_closure(phase).to_owned(),
                implementation_owner: "ferrite-server-runtime",
                left: join.left,
                right: join.right,
                disposition: "Pending",
                evidence: Vec::new(),
            }
        })
        .collect()
}

fn protocol_batches(
    mut families: Vec<ProtocolFamily>,
    packets: &[Packet],
) -> Result<Vec<ProtocolBatch>> {
    families.sort_by(|left, right| left.id.cmp(&right.id));
    let mut indexes = BTreeMap::<(u8, bool), usize>::new();
    let mut batches = Vec::new();
    for family in families {
        let phase = protocol_phase(&family);
        let optional = family.responsibility == "Optional";
        let index = indexes.entry((phase, optional)).or_default();
        *index += 1;
        let class = if optional { 'O' } else { 'F' };
        let mut builder = GlobSetBuilder::new();
        for pattern in &family.patterns {
            builder.add(
                Glob::new(pattern).with_context(|| format!("invalid selector in {}", family.id))?,
            );
        }
        let selectors = builder.build()?;
        let packet_count = packets
            .iter()
            .filter(|packet| {
                packet.state == family.state
                    && packet.direction == family.direction
                    && selectors.is_match(&packet.identity)
            })
            .count();
        ensure!(packet_count > 0, "{} matches no packets", family.id);
        let family_slug = family
            .id
            .strip_prefix("PROTO-")
            .unwrap_or(&family.id)
            .trim_end_matches("-001");
        batches.push(ProtocolBatch {
            id: format!("G01-P{phase}-{class}{index:03}"),
            phase,
            responsibility: format!("Implement protocol family {}", family.id),
            depends_on: vec![phase_dependency(phase).to_owned()],
            closes_in: phase_closure(phase).to_owned(),
            implementation_owner: "ferrite-protocol",
            test_owner: format!(
                "crates/ferrite-protocol/tests/{}/{}.rs",
                family.level.to_ascii_lowercase(),
                slug(family_slug)
            ),
            disposition: "Pending",
            evidence: Vec::new(),
            reference_family: family.id,
            level: family.level,
            state: family.state,
            direction: family.direction,
            source_status: family.status,
            source_responsibility: family.responsibility,
            implementation_mode: if optional {
                "ConfigurationGate"
            } else {
                "Required"
            },
            packets: packet_count,
        });
    }
    batches.sort_by(|left, right| {
        (left.phase, &left.id, &left.reference_family).cmp(&(
            right.phase,
            &right.id,
            &right.reference_family,
        ))
    });
    Ok(batches)
}

fn read_packets(context: &Context) -> Result<Vec<Packet>> {
    let path = context.cache.join("generated/reports/packets.json");
    let root: Value = serde_json::from_str(
        &fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))?;
    let mut packets = Vec::new();
    for (state, directions) in root
        .as_object()
        .context("packets.json root is not an object")?
    {
        for (direction, identities) in directions
            .as_object()
            .with_context(|| format!("packets.json state {state} is not an object"))?
        {
            for identity in identities
                .as_object()
                .with_context(|| format!("packets.json {state}/{direction} is not an object"))?
                .keys()
            {
                packets.push(Packet {
                    state: state.clone(),
                    direction: direction.clone(),
                    identity: identity.clone(),
                });
            }
        }
    }
    packets.sort_by(|left, right| {
        (&left.state, &left.direction, &left.identity).cmp(&(
            &right.state,
            &right.direction,
            &right.identity,
        ))
    });
    Ok(packets)
}

fn validate_totals(totals: &Totals) -> Result<()> {
    ensure!(totals.catalog_ids == 9078, "expected 9,078 catalog IDs");
    ensure!(
        totals.gameplay_slices == 331,
        "expected 331 gameplay slices"
    );
    ensure!(
        totals.source_specified_slices == 327,
        "expected 327 source-specified slices"
    );
    ensure!(
        totals.source_inconclusive_slices == 4,
        "expected four source-inconclusive slices"
    );
    ensure!(
        totals.deferred_observations == 4,
        "expected four deferred observations"
    );
    ensure!(totals.behavior_surfaces == 10, "expected ten surfaces");
    ensure!(totals.cross_system_joins == 36, "expected 36 joins");
    ensure!(totals.protocol_packets == 256, "expected 256 packets");
    ensure!(totals.protocol_families == 58, "expected 58 families");
    ensure!(
        totals.required_protocol_families == 44,
        "expected 44 required families"
    );
    ensure!(
        totals.optional_protocol_families == 14,
        "expected 14 optional families"
    );
    Ok(())
}

fn subsystem_phase(subsystem: &str) -> Result<u8> {
    match subsystem {
        "simulation" | "blocks" | "environment" | "redstone" => Ok(5),
        "player" | "items" => Ok(6),
        "entities" | "mobs" => Ok(7),
        "world" => Ok(8),
        "client" => Ok(9),
        _ => anyhow::bail!("unmapped gameplay subsystem {subsystem}"),
    }
}

fn gameplay_test_root(subsystem: &str) -> String {
    match subsystem {
        "simulation" => "crates/ferrite-simulation/tests/slices".to_owned(),
        "world" => "crates/ferrite-world/tests/slices".to_owned(),
        "client" => "apps/behavior-runner/tests/client".to_owned(),
        _ => format!("crates/ferrite-gameplay/tests/slices/{subsystem}"),
    }
}

fn gameplay_implementation_owner(subsystem: &str) -> &'static str {
    match subsystem {
        "simulation" => "ferrite-simulation",
        "world" => "ferrite-world",
        "client" => "behavior-runner",
        _ => "ferrite-gameplay",
    }
}

fn protocol_phase(family: &ProtocolFamily) -> u8 {
    match family.level.as_str() {
        "C0" | "C1" => 3,
        "C2" => 4,
        "C4" => 9,
        "C3" if contains_any(
            &family.id,
            &[
                "CONTAINER",
                "INVENTORY",
                "RECIPE",
                "MERCHANT",
                "ANVIL",
                "BEACON",
                "SPECIAL-SCREENS",
                "PROGRESSION",
            ],
        ) =>
        {
            6
        }
        "C3" if contains_any(&family.id, &["ENTITY", "COMBAT"]) => 7,
        "C3" => 9,
        _ => 9,
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn surface_phase(kind: &str) -> u8 {
    match kind {
        "TickScheduler" => 5,
        "NetworkIngress" => 4,
        "PlayerLifecycle" => 6,
        "ContentDispatch" | "WorldLifecycle" | "PersistenceReload" => 8,
        "CommandAdministration" | "ClientProjection" | "DataReload" | "CrossSystemOrdering" => 9,
        _ => 9,
    }
}

fn surface_implementation_owner(kind: &str) -> &'static str {
    match kind {
        "TickScheduler" => "ferrite-simulation",
        "NetworkIngress" | "ClientProjection" => "ferrite-protocol",
        "PlayerLifecycle" | "CommandAdministration" | "ContentDispatch" => "ferrite-gameplay",
        "WorldLifecycle" => "ferrite-world",
        "PersistenceReload" => "ferrite-persistence",
        "DataReload" => "ferrite-registry",
        "CrossSystemOrdering" => "ferrite-server-runtime",
        _ => "behavior-runner",
    }
}

fn phase_dependency(phase: u8) -> &'static str {
    match phase {
        1 => "G01-P1-B3",
        3 => "G01-P3-B2",
        4 => "G01-P3-B5",
        5 => "G01-P4-B5",
        6 => "G01-P5-B2",
        7 => "G01-P6-B2",
        8 => "G01-P7-B2",
        9 => "G01-P8-B3",
        _ => "G01-P0-B4",
    }
}

fn phase_closure(phase: u8) -> &'static str {
    match phase {
        1 => "G01-P1-B4",
        3 => "G01-P3-B5",
        4 => "G01-P4-B5",
        5 => "G01-P5-B2",
        6 => "G01-P6-B2",
        7 => "G01-P7-B2",
        8 => "G01-P8-B2",
        9 => "G01-P9-B1",
        _ => "G01-P10-B1",
    }
}

fn sorted_union<'a>(values: impl Iterator<Item = &'a String>) -> Vec<String> {
    values
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn slug(value: &str) -> String {
    let mut result = String::new();
    let mut previous_was_lower_or_digit = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            if character.is_ascii_uppercase() && previous_was_lower_or_digit {
                result.push('_');
            }
            result.push(character.to_ascii_lowercase());
            previous_was_lower_or_digit =
                character.is_ascii_lowercase() || character.is_ascii_digit();
        } else {
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
            previous_was_lower_or_digit = false;
        }
    }
    result.trim_matches('_').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_batches_are_sorted_and_concrete() {
        let batches = catalog_batches(vec![
            CatalogCategory {
                kind: "worldgen/feature".to_owned(),
                expected_count: 2,
                ids_sha1: "b".to_owned(),
                family: vec![CatalogFamily {
                    name: "tree".to_owned(),
                }],
            },
            CatalogCategory {
                kind: "block".to_owned(),
                expected_count: 1,
                ids_sha1: "a".to_owned(),
                family: vec![CatalogFamily {
                    name: "stone".to_owned(),
                }],
            },
        ]);

        assert_eq!(batches[0].id, "G01-P1-D001");
        assert_eq!(batches[0].reference_kind, "block");
        assert_eq!(batches[1].id, "G01-P1-D002");
        assert_eq!(batches[1].reference_kind, "worldgen/feature");
        assert_eq!(
            batches[1].test_owner,
            "crates/ferrite-registry/tests/catalog/worldgen_feature.rs"
        );
    }

    #[test]
    fn gameplay_batches_partition_by_phase_subsystem_and_primary_parent() {
        let batches = gameplay_batches(vec![
            completion_slice("BLK-B-001", "blocks", "BLK-002", "SourceSpecified"),
            completion_slice("SIM-A-001", "simulation", "SIM-001", "SourceSpecified"),
            completion_slice("SIM-A-002", "simulation", "SIM-001", "SourceInconclusive"),
        ])
        .unwrap();

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].id, "G01-P5-S001");
        assert_eq!(batches[0].primary_parent, "BLK-002");
        assert_eq!(batches[1].id, "G01-P5-S002");
        assert_eq!(batches[1].source_specified, 1);
        assert_eq!(batches[1].source_inconclusive, 1);
    }

    #[test]
    fn protocol_batches_count_packets_and_assign_optional_gates() {
        let families = vec![
            protocol_family(
                "PROTO-PLAY-CLIENTBOUND-SOUND-001",
                "C3",
                "Specified",
                "Required",
                vec!["minecraft:*sound*"],
            ),
            protocol_family(
                "PROTO-PLAY-CLIENTBOUND-LIVE-TAGS-001",
                "C4",
                "GatedOptional",
                "Optional",
                vec!["minecraft:update_tags"],
            ),
        ];
        let packets = vec![
            packet("minecraft:sound"),
            packet("minecraft:sound_entity"),
            packet("minecraft:update_tags"),
        ];

        let batches = protocol_batches(families, &packets).unwrap();

        assert_eq!(batches[0].id, "G01-P9-F001");
        assert_eq!(batches[0].packets, 2);
        assert_eq!(batches[0].implementation_mode, "Required");
        assert_eq!(batches[1].id, "G01-P9-O001");
        assert_eq!(batches[1].packets, 1);
        assert_eq!(batches[1].implementation_mode, "ConfigurationGate");
    }

    #[test]
    fn slugs_are_readable_paths() {
        assert_eq!(slug("PROTO-WORLD/Feature-001"), "proto_world_feature_001");
        assert_eq!(slug("TickScheduler"), "tick_scheduler");
    }

    #[test]
    fn renderer_detects_progress_before_overwrite() {
        let initial = progress_fixture("Pending", "DeferredExperiment", false);
        let active = progress_fixture("InProgress", "DeferredExperiment", false);
        let evidenced = progress_fixture("Pending", "DeferredExperiment", true);

        assert!(!contains_implementation_progress(&initial).unwrap());
        assert!(contains_implementation_progress(&active).unwrap());
        assert!(contains_implementation_progress(&evidenced).unwrap());
    }

    fn completion_slice(id: &str, subsystem: &str, parent: &str, status: &str) -> CompletionSlice {
        CompletionSlice {
            id: id.to_owned(),
            subsystem: subsystem.to_owned(),
            parents: vec![parent.to_owned()],
            leaves: vec![format!("{parent}-LEAF")],
            status: status.to_owned(),
        }
    }

    fn protocol_family(
        id: &str,
        level: &str,
        status: &str,
        responsibility: &str,
        patterns: Vec<&str>,
    ) -> ProtocolFamily {
        ProtocolFamily {
            id: id.to_owned(),
            level: level.to_owned(),
            state: "play".to_owned(),
            direction: "clientbound".to_owned(),
            patterns: patterns.into_iter().map(str::to_owned).collect(),
            status: status.to_owned(),
            responsibility: responsibility.to_owned(),
        }
    }

    fn packet(identity: &str) -> Packet {
        Packet {
            state: "play".to_owned(),
            direction: "clientbound".to_owned(),
            identity: identity.to_owned(),
        }
    }

    fn progress_fixture(
        implementation_disposition: &str,
        deferred_disposition: &str,
        with_evidence: bool,
    ) -> String {
        let evidence = if with_evidence {
            r#"["test:path"]"#
        } else {
            "[]"
        };
        format!(
            r#"
            catalog_batch = [{{ disposition = "{implementation_disposition}", evidence = {evidence} }}]
            gameplay_batch = []
            deferred_observation = [{{ disposition = "{deferred_disposition}", evidence = [] }}]
            worldgen_exactness = []
            surface_owner = []
            join_owner = []
            protocol_batch = []
            "#
        )
    }
}
