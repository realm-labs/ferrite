use anyhow::{Context as _, Result, bail, ensure};
use globset::{Glob, GlobSetBuilder};
use regex::Regex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use walkdir::WalkDir;
use zip::ZipArchive;

const REFERENCE_RELATIVE: &str = "docs/reference/minecraft-java-26.2";

#[derive(Debug)]
pub enum Command {
    Fetch { version: String },
    Reports,
    Query { kind: String, id: String },
    Symbols,
    Coverage,
    Readiness,
    Protocol(ProtocolCommand),
    Surface(SurfaceCommand),
    Experiment(ExperimentCommand),
    Verify { offline: bool },
}

#[derive(Debug)]
pub enum ProtocolCommand {
    Inventory,
    Coverage,
    Readiness,
    Verify,
}

#[derive(Debug)]
pub enum SurfaceCommand {
    Coverage,
    Readiness,
    Verify,
}

#[derive(Debug)]
pub enum ExperimentCommand {
    List,
    Run { id: String },
    Verify,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub workspace: PathBuf,
    pub reference: PathBuf,
    pub cache: PathBuf,
    lock: LockFile,
}

#[derive(Debug, Clone, Deserialize)]
struct LockFile {
    version: String,
    manifest_url: String,
    metadata: Artifact,
    client: Artifact,
    server: Artifact,
    java_major: u32,
    data_pack: String,
    resource_pack: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Artifact {
    url: String,
    sha1: String,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    versions: Vec<ManifestVersion>,
}

#[derive(Debug, Deserialize)]
struct ManifestVersion {
    id: String,
    url: String,
    sha1: String,
}

#[derive(Debug, Deserialize)]
struct VersionMetadata {
    downloads: BTreeMap<String, Download>,
}

#[derive(Debug, Deserialize)]
struct Download {
    url: String,
    sha1: String,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct Catalog {
    category: Vec<Category>,
}

#[derive(Debug, Deserialize)]
struct Category {
    kind: String,
    source: String,
    expected_count: usize,
    ids_sha1: String,
    family: Vec<Family>,
}

#[derive(Debug, Deserialize)]
struct Family {
    name: String,
    classification: Classification,
    rules: Vec<String>,
    #[serde(default)]
    exact: Vec<String>,
    #[serde(default)]
    patterns: Vec<String>,
    #[serde(default)]
    block_items: bool,
    #[serde(default)]
    remaining: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
enum Classification {
    BehaviorFamily,
    Special,
    DataOnly,
    Unreviewed,
}

#[derive(Debug, Deserialize)]
struct ExperimentFile {
    experiment: Vec<Experiment>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Experiment {
    id: String,
    rules: Vec<String>,
    mode: String,
    status: String,
    repeats: u32,
    initial_state: Vec<String>,
    action: Vec<TimedText>,
    observation: Vec<TimedText>,
    expected: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TimedText {
    tick: u64,
    value: String,
}

#[derive(Debug, Deserialize)]
struct ExperimentResult {
    passed: bool,
    observations: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CompletionFile {
    version: String,
    slice: Vec<CompletionSlice>,
    registry: Vec<RegistryScopeRecord>,
}

#[derive(Debug, Deserialize)]
struct CompletionSlice {
    id: String,
    subsystem: String,
    parents: Vec<String>,
    leaves: Vec<String>,
    registry_kinds: Vec<String>,
    selectors: Vec<String>,
    symbols: Vec<String>,
    data_paths: Vec<String>,
    status: CompletionStatus,
    unknowns: Vec<String>,
    reproduction: Vec<String>,
    experiments: Vec<String>,
    last_commit: String,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CompletionStatus {
    Todo,
    InProgress,
    SourceSpecified,
    DataOnlyVerified,
    SourceInconclusive,
}

#[derive(Debug, Deserialize)]
struct RegistryScopeRecord {
    id: String,
    scope: RegistryScope,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct ProtocolCompletionFile {
    version: String,
    inventory: ProtocolInventoryLock,
    family: Vec<ProtocolFamily>,
}

#[derive(Debug, Deserialize)]
struct ProtocolInventoryLock {
    expected_count: usize,
    entries_sha1: String,
}

#[derive(Debug, Deserialize)]
struct ProtocolFamily {
    id: String,
    level: ProtocolLevel,
    state: String,
    direction: String,
    patterns: Vec<String>,
    status: ProtocolStatus,
    responsibility: ProtocolResponsibility,
    owner: String,
    specification: String,
    evidence: Vec<String>,
    fields: Vec<String>,
    mappings: Vec<String>,
    transitions: Vec<String>,
    ordering: Vec<String>,
    vectors: Vec<String>,
    unknowns: Vec<String>,
    reproduction: Vec<String>,
    last_commit: String,
}

#[derive(Debug, Deserialize)]
struct BehaviorSurfaceFile {
    version: String,
    surface: Vec<BehaviorSurface>,
}

#[derive(Debug, Deserialize)]
struct CommandRootMap {
    version: String,
    inventory: CommandRootInventoryLock,
    family: Vec<CommandRootFamily>,
}

#[derive(Debug, Deserialize)]
struct CrossSystemJoinMap {
    version: String,
    join: Vec<CrossSystemJoin>,
}

#[derive(Debug, Deserialize)]
struct CrossSystemJoin {
    left: BehaviorSurfaceKind,
    right: BehaviorSurfaceKind,
    shared_domains: Vec<String>,
    owners: Vec<String>,
    status: CrossSystemJoinStatus,
    remaining_work: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CrossSystemJoinStatus {
    Empty,
    InProgress,
    Mapped,
    SourceInconclusive,
}

#[derive(Debug, Deserialize)]
struct CommandRootInventoryLock {
    expected_count: usize,
    roots_sha1: String,
}

#[derive(Debug, Deserialize)]
struct CommandRootFamily {
    name: String,
    roots: Vec<String>,
    owners: Vec<String>,
    state_domains: Vec<String>,
    status: CommandRootStatus,
    remaining_work: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CommandRootStatus {
    InProgress,
    Mapped,
    SourceInconclusive,
}

#[derive(Debug, Deserialize)]
struct BehaviorSurface {
    id: String,
    kind: BehaviorSurfaceKind,
    boundary: String,
    triggers: Vec<String>,
    inventory_sources: Vec<SurfaceInventorySource>,
    selectors: Vec<String>,
    owners: Vec<String>,
    state_domains: Vec<String>,
    persistence: Vec<String>,
    client_projection: Vec<String>,
    #[serde(default)]
    protocol_families: Vec<String>,
    status: BehaviorSurfaceStatus,
    evidence: Vec<String>,
    unknowns: Vec<String>,
    reproduction: Vec<String>,
    last_commit: String,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BehaviorSurfaceKind {
    TickScheduler,
    NetworkIngress,
    CommandAdministration,
    ContentDispatch,
    PlayerLifecycle,
    WorldLifecycle,
    PersistenceReload,
    ClientProjection,
    DataReload,
    CrossSystemOrdering,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BehaviorSurfaceStatus {
    Todo,
    InProgress,
    Mapped,
    SourceInconclusive,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SurfaceInventorySource {
    OfficialServerSymbols,
    OfficialClientSymbols,
    PacketReport,
    CommandReport,
    RegistryReport,
    BundledData,
    SaveStateFields,
    ManualCrossProduct,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProtocolLevel {
    C0,
    C1,
    C2,
    C3,
    C4,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProtocolStatus {
    Todo,
    InProgress,
    Specified,
    GatedOptional,
    NonServerResponsibility,
    SourceInconclusive,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
enum ProtocolResponsibility {
    Required,
    Optional,
    NonServer,
}

#[derive(Debug, Deserialize)]
enum RegistryScope {
    GameplayBehavior,
    GameplayData,
    ObservablePresentation,
    InternalOnly,
}

#[derive(Debug)]
struct MatchResult<'a> {
    category: &'a Category,
    family: &'a Family,
}

impl Context {
    pub fn discover() -> Result<Self> {
        let mut current = env::current_dir()?;
        loop {
            let reference = current.join(REFERENCE_RELATIVE);
            if reference.join("lock.toml").is_file() {
                let lock: LockFile =
                    toml::from_str(&fs::read_to_string(reference.join("lock.toml"))?)?;
                let cache = current.join("target/mc-reference").join(&lock.version);
                return Ok(Self {
                    workspace: current,
                    reference,
                    cache,
                    lock,
                });
            }
            if !current.pop() {
                bail!(
                    "run inside the Ferrite workspace; {REFERENCE_RELATIVE}/lock.toml was not found"
                );
            }
        }
    }
}

pub fn run(context: &Context, command: Command) -> Result<()> {
    match command {
        Command::Fetch { version } => fetch(context, &version),
        Command::Reports => reports(context),
        Command::Query { kind, id } => query(context, &kind, &id),
        Command::Symbols => symbols(context),
        Command::Coverage => coverage(context).map(|_| ()),
        Command::Readiness => readiness(context),
        Command::Protocol(command) => protocol(context, command),
        Command::Surface(command) => surfaces(context, command),
        Command::Experiment(command) => experiments(context, command),
        Command::Verify { offline } => verify(context, offline),
    }
}

fn fetch(context: &Context, version: &str) -> Result<()> {
    ensure!(
        version == context.lock.version,
        "only locked version {} is accepted",
        context.lock.version
    );
    fs::create_dir_all(&context.cache)?;
    let client = Client::builder()
        .user_agent("Ferrite mc-reference/0.1")
        .build()?;
    let manifest_bytes = get(&client, &context.lock.manifest_url)?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
    let metadata_is_current =
        manifest_metadata_is_current(&manifest, version, &context.lock.metadata)?;
    write_verified(
        &context.cache.join("version_manifest_v2.json"),
        &manifest_bytes,
        None,
        None,
    )?;
    if !metadata_is_current {
        eprintln!(
            "warning: the live manifest now points {version} at revised launcher metadata; \
             fetching the SHA-1-locked metadata instead"
        );
    }

    let metadata_bytes = get(&client, &context.lock.metadata.url)?;
    write_verified(
        &context.cache.join("version.json"),
        &metadata_bytes,
        Some(&context.lock.metadata.sha1),
        None,
    )?;
    let metadata: VersionMetadata = serde_json::from_slice(&metadata_bytes)?;
    for (name, locked) in [
        ("client", &context.lock.client),
        ("server", &context.lock.server),
    ] {
        let declared = metadata
            .downloads
            .get(name)
            .with_context(|| format!("metadata has no {name} download"))?;
        ensure!(
            declared.url == locked.url && declared.sha1 == locked.sha1,
            "{name} metadata differs from lock"
        );
        ensure!(
            locked.size == Some(declared.size),
            "{name} size differs from lock"
        );
        download_file(&client, locked, &context.cache.join(format!("{name}.jar")))?;
    }
    println!(
        "fetched and verified Minecraft Java {} (data pack {}, resource pack {})",
        context.lock.version, context.lock.data_pack, context.lock.resource_pack
    );
    Ok(())
}

fn manifest_metadata_is_current(
    manifest: &Manifest,
    version: &str,
    locked: &Artifact,
) -> Result<bool> {
    let entry = manifest
        .versions
        .iter()
        .find(|entry| entry.id == version)
        .with_context(|| format!("{version} is absent from the official manifest"))?;
    Ok(entry.url == locked.url && entry.sha1 == locked.sha1)
}

fn get(client: &Client, url: &str) -> Result<Vec<u8>> {
    let response = client.get(url).send()?.error_for_status()?;
    Ok(response.bytes()?.to_vec())
}

fn download_file(client: &Client, artifact: &Artifact, destination: &Path) -> Result<()> {
    if destination.is_file() && verify_file(destination, &artifact.sha1, artifact.size).is_ok() {
        return Ok(());
    }
    let part = destination.with_extension("jar.part");
    let mut response = client.get(&artifact.url).send()?.error_for_status()?;
    let mut output = File::create(&part)?;
    io::copy(&mut response, &mut output)?;
    output.flush()?;
    verify_file(&part, &artifact.sha1, artifact.size)?;
    fs::rename(part, destination)?;
    Ok(())
}

fn write_verified(path: &Path, bytes: &[u8], sha1: Option<&str>, size: Option<u64>) -> Result<()> {
    if let Some(expected) = sha1 {
        ensure!(
            sha1_bytes(bytes) == expected,
            "SHA-1 mismatch for {}",
            path.display()
        );
    }
    if let Some(expected) = size {
        ensure!(
            bytes.len() as u64 == expected,
            "size mismatch for {}",
            path.display()
        );
    }
    fs::write(path, bytes)?;
    Ok(())
}

fn verify_file(path: &Path, expected_sha1: &str, expected_size: Option<u64>) -> Result<()> {
    let file = File::open(path).with_context(|| format!("missing {}", path.display()))?;
    if let Some(expected) = expected_size {
        ensure!(
            file.metadata()?.len() == expected,
            "size mismatch for {}",
            path.display()
        );
    }
    let mut reader = BufReader::new(file);
    let mut hasher = Sha1::new();
    io::copy(&mut reader, &mut hasher)?;
    ensure!(
        hex::encode(hasher.finalize()) == expected_sha1,
        "SHA-1 mismatch for {}",
        path.display()
    );
    Ok(())
}

fn reports(context: &Context) -> Result<()> {
    verify_cached_artifacts(context)?;
    let output = context.cache.join("generated");
    fs::create_dir_all(&output)?;
    let java = env::var("MC_REF_JAVA").unwrap_or_else(|_| "java".into());
    check_java_major(&java, context.lock.java_major)?;
    let status = ProcessCommand::new(java)
        .current_dir(&context.cache)
        .arg("-DbundlerMainClass=net.minecraft.data.Main")
        .arg("-jar")
        .arg(context.cache.join("server.jar"))
        .arg("--reports")
        .arg("--output")
        .arg(&output)
        .status()?;
    ensure!(
        status.success(),
        "official report generator exited with {status}"
    );
    ensure!(
        output.join("reports/blocks.json").is_file(),
        "report generation did not produce blocks.json"
    );
    extract_server(context)?;
    println!("reports generated in {}", output.display());
    Ok(())
}

fn check_java_major(java: &str, expected: u32) -> Result<()> {
    let output = ProcessCommand::new(java).arg("-version").output()?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let regex = Regex::new(r#"version "(\d+)"#)?;
    let actual: u32 = regex
        .captures(&combined)
        .context("cannot parse java -version")?[1]
        .parse()?;
    ensure!(
        actual == expected,
        "Java {expected} required, found Java {actual}; set MC_REF_JAVA"
    );
    Ok(())
}

fn extract_server(context: &Context) -> Result<PathBuf> {
    let destination = context
        .cache
        .join(format!("server-{}.jar", context.lock.version));
    if destination.is_file() {
        return Ok(destination);
    }
    let input = File::open(context.cache.join("server.jar"))?;
    let mut archive = ZipArchive::new(input)?;
    let suffix = format!("/server-{}.jar", context.lock.version);
    let index = (0..archive.len())
        .find(|index| {
            archive
                .by_index(*index)
                .map(|file| file.name().ends_with(&suffix))
                .unwrap_or(false)
        })
        .context("bundled server jar not found")?;
    let mut member = archive.by_index(index)?;
    let mut output = File::create(&destination)?;
    io::copy(&mut member, &mut output)?;
    Ok(destination)
}

fn query(context: &Context, kind: &str, raw_id: &str) -> Result<()> {
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

fn coverage(context: &Context) -> Result<usize> {
    verify_cached_artifacts(context)?;
    let catalog = load_catalog(context)?;
    validate_rule_references(context, &catalog)?;
    let blocks = load_category_ids(context, "block")?;
    let mut total = 0;
    let mut unreviewed = 0;
    let mut unreviewed_families = BTreeMap::<(String, String), usize>::new();
    for category in &catalog.category {
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
            let matched = classify(&catalog, &category.kind, id, Some(&blocks))?;
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

fn validate_family_selectors(
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

fn load_category_ids(context: &Context, kind: &str) -> Result<BTreeSet<String>> {
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

fn registry_entry(registries: &Value, kind: &str, id: &str) -> Result<Value> {
    let registry = escape_pointer(&format!("minecraft:{}", registry_report_key(kind)));
    registries
        .pointer(&format!("/{registry}/entries/{}", escape_pointer(id)))
        .cloned()
        .with_context(|| format!("registry {kind} has no entry {id}"))
}

fn registry_ids(registries: &Value, kind: &str) -> Result<BTreeSet<String>> {
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

fn ids_from_files(directory: &Path, extension: &str) -> Result<BTreeSet<String>> {
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

fn classify<'a>(
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
    let mut matches = Vec::new();
    for family in &category.family {
        let mut matched = family
            .exact
            .iter()
            .any(|value| normalize_unchecked(value) == id);
        if !matched && !family.patterns.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pattern in &family.patterns {
                builder.add(Glob::new(&normalize_unchecked(pattern))?);
            }
            matched = builder.build()?.is_match(id);
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
        "{kind} {id} matched {} behavior families",
        matches.len()
    );
    Ok(MatchResult {
        category,
        family: matches[0],
    })
}

fn symbols(context: &Context) -> Result<()> {
    verify_cached_artifacts(context)?;
    let server = extract_server(context)?;
    let client = context.cache.join("client.jar");
    let javap = env::var("MC_REF_JAVAP").unwrap_or_else(|_| "javap".into());
    let symbol_regex = Regex::new(
        r"`(?P<class>net\.minecraft\.[A-Za-z0-9_.$]+)#(?P<member>[A-Za-z0-9_$<>]+)(?P<params>\([^`]*\))?`",
    )?;
    let mut symbols = BTreeSet::new();
    for file in markdown_files(&context.reference) {
        let text = fs::read_to_string(&file)?;
        for captures in symbol_regex.captures_iter(&text) {
            symbols.insert((
                captures["class"].to_string(),
                captures["member"].to_string(),
                captures.name("params").map(|m| m.as_str().to_string()),
            ));
        }
    }
    ensure!(
        !symbols.is_empty(),
        "no source symbols found in documentation"
    );
    let mut cache = BTreeMap::<String, String>::new();
    for (class, member, params) in &symbols {
        let output = if let Some(value) = cache.get(class) {
            value.clone()
        } else {
            let jar = if class.starts_with("net.minecraft.client.") {
                &client
            } else {
                &server
            };
            let output = ProcessCommand::new(&javap)
                .args(["-p", "-s", "-classpath"])
                .arg(jar)
                .arg(class)
                .output()?;
            ensure!(
                output.status.success(),
                "javap could not resolve {class}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let text = String::from_utf8(output.stdout)?;
            cache.insert(class.clone(), text.clone());
            text
        };
        ensure!(
            output.contains(member),
            "symbol not found: {class}#{member}"
        );
        if let Some(params) = params {
            ensure!(
                descriptor_matches(&output, member, params),
                "method overload not found: {class}#{member}{params}"
            );
        }
    }
    println!(
        "symbols verified: {} locators across {} classes",
        symbols.len(),
        cache.len()
    );
    Ok(())
}

fn descriptor_matches(output: &str, member: &str, parameters: &str) -> bool {
    let expected: String = parameters
        .trim_matches(['(', ')'])
        .split(',')
        .filter(|value| !value.trim().is_empty())
        .map(java_type_descriptor)
        .collect();
    let lines: Vec<_> = output.lines().collect();
    lines.windows(2).any(|pair| {
        pair[0].contains(&format!(" {member}("))
            && pair[1]
                .trim()
                .strip_prefix("descriptor: (")
                .and_then(|value| value.split_once(')'))
                .is_some_and(|(actual, _)| actual == expected)
    })
}

fn java_type_descriptor(value: &str) -> String {
    let value = value.trim();
    if let Some(component) = value.strip_suffix("[]") {
        return format!("[{}", java_type_descriptor(component));
    }
    match value {
        "boolean" => "Z".into(),
        "byte" => "B".into(),
        "char" => "C".into(),
        "short" => "S".into(),
        "int" => "I".into(),
        "long" => "J".into(),
        "float" => "F".into(),
        "double" => "D".into(),
        _ => format!("L{};", value.replace('.', "/")),
    }
}

fn experiments(context: &Context, command: ExperimentCommand) -> Result<()> {
    let definitions = load_experiments(context)?;
    match command {
        ExperimentCommand::List => {
            for experiment in definitions {
                println!(
                    "{}\t{}\t{}",
                    experiment.id, experiment.mode, experiment.status
                );
            }
            Ok(())
        }
        ExperimentCommand::Verify => validate_experiments(context, &definitions),
        ExperimentCommand::Run { id } => {
            let experiment = definitions
                .iter()
                .find(|experiment| experiment.id == id)
                .with_context(|| format!("unknown experiment {id}"))?;
            let run_directory = context.cache.join("experiments").join(&id);
            fs::create_dir_all(&run_directory)?;
            fs::write(
                run_directory.join("procedure.json"),
                serde_json::to_vec_pretty(experiment)?,
            )?;
            ensure!(
                experiment.status == "automated",
                "{} is {}; prepared {} but no automated result can be claimed",
                id,
                experiment.status,
                run_directory.join("procedure.json").display()
            );
            let runner = context
                .reference
                .join("experiments/runner")
                .join(format!("{id}.sh"));
            ensure!(
                runner.is_file(),
                "automated runner is not committed for {id}"
            );
            let result_path = run_directory.join("result.json");
            if result_path.exists() {
                fs::remove_file(&result_path)?;
            }
            let status = ProcessCommand::new("sh")
                .arg(runner)
                .current_dir(&run_directory)
                .env("MC_REF_CACHE", &context.cache)
                .env("MC_REF_EXPERIMENT_DIR", &run_directory)
                .env("MC_REF_SERVER_JAR", context.cache.join("server.jar"))
                .status()?;
            ensure!(status.success(), "experiment {id} failed");
            let result: ExperimentResult = serde_json::from_reader(BufReader::new(
                File::open(&result_path)
                    .with_context(|| format!("runner did not produce {}", result_path.display()))?,
            ))?;
            ensure!(
                result.passed && !result.observations.is_empty(),
                "experiment {id} did not pass with recorded observations"
            );
            println!(
                "experiment {id} passed with {} observations",
                result.observations.len()
            );
            Ok(())
        }
    }
}

fn validate_experiments(context: &Context, definitions: &[Experiment]) -> Result<()> {
    let rules = documented_rule_ids(context)?;
    let mut ids = BTreeSet::new();
    for experiment in definitions {
        ensure!(
            experiment.id.starts_with("EXP-"),
            "invalid experiment ID {}",
            experiment.id
        );
        ensure!(
            ids.insert(&experiment.id),
            "duplicate experiment ID {}",
            experiment.id
        );
        ensure!(
            experiment.repeats > 0,
            "{} repeats must be positive",
            experiment.id
        );
        ensure!(
            !experiment.initial_state.is_empty()
                && !experiment.action.is_empty()
                && !experiment.observation.is_empty()
                && !experiment.expected.is_empty(),
            "{} has an incomplete procedure",
            experiment.id
        );
        ensure!(
            experiment
                .action
                .windows(2)
                .all(|pair| pair[0].tick <= pair[1].tick),
            "{} actions are not tick ordered",
            experiment.id
        );
        ensure!(
            experiment
                .observation
                .windows(2)
                .all(|pair| pair[0].tick <= pair[1].tick),
            "{} observations are not tick ordered",
            experiment.id
        );
        ensure!(
            experiment.action.iter().all(|v| !v.value.trim().is_empty())
                && experiment
                    .observation
                    .iter()
                    .all(|v| !v.value.trim().is_empty()),
            "{} contains an empty step",
            experiment.id
        );
        for rule in &experiment.rules {
            ensure!(
                rules.contains(rule),
                "{} references missing rule {rule}",
                experiment.id
            );
        }
    }
    println!("experiment definitions verified: {}", definitions.len());
    Ok(())
}

fn verify(context: &Context, offline: bool) -> Result<()> {
    if !offline {
        let client = Client::builder()
            .user_agent("Ferrite mc-reference/0.1")
            .build()?;
        let manifest: Manifest =
            serde_json::from_slice(&get(&client, &context.lock.manifest_url)?)?;
        let metadata_is_current =
            manifest_metadata_is_current(&manifest, &context.lock.version, &context.lock.metadata)?;
        if metadata_is_current {
            println!("official manifest version and metadata pointer verified");
        } else {
            println!(
                "official manifest version verified; live metadata pointer has moved beyond the lock"
            );
        }
    }
    verify_cached_artifacts(context)?;
    verify_reports(context)?;
    validate_docs(context)?;
    validate_completion(context, false)?;
    symbols(context)?;
    coverage(context)?;
    experiments(context, ExperimentCommand::Verify)?;
    protocol_verify(context)?;
    surface_coverage(context, false)?;
    hygiene(context)?;
    println!(
        "mc-reference verification complete ({})",
        if offline { "offline" } else { "online" }
    );
    Ok(())
}

fn readiness(context: &Context) -> Result<()> {
    let completion_error = validate_completion(context, true).err();
    let surface_error = surface_coverage(context, true).err();
    match (completion_error, surface_error) {
        (None, None) => Ok(()),
        (Some(completion), None) => {
            bail!("gameplay readiness blocked: {completion:#}")
        }
        (None, Some(surface)) => {
            bail!("gameplay readiness blocked: {surface:#}")
        }
        (Some(completion), Some(surface)) => bail!(
            "gameplay readiness blocked by both ledgers:\n- completion: {completion:#}\n- surfaces: {surface:#}"
        ),
    }
}

fn protocol(context: &Context, command: ProtocolCommand) -> Result<()> {
    match command {
        ProtocolCommand::Inventory => protocol_inventory(context).map(|_| ()),
        ProtocolCommand::Coverage => protocol_coverage(context, false),
        ProtocolCommand::Readiness => protocol_coverage(context, true),
        ProtocolCommand::Verify => protocol_verify(context),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProtocolPacket {
    state: String,
    direction: String,
    identity: String,
    protocol_id: u64,
}

fn load_protocol_completion(context: &Context) -> Result<ProtocolCompletionFile> {
    let path = context.reference.join("protocol/completion.toml");
    toml::from_str(
        &fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?,
    )
    .with_context(|| format!("invalid {}", path.display()))
}

fn protocol_packets(context: &Context) -> Result<Vec<ProtocolPacket>> {
    let path = context.cache.join("generated/reports/packets.json");
    let report = read_json(&path)?;
    let mut packets = Vec::new();
    for (state, directions) in report
        .as_object()
        .context("packets.json root is not an object")?
    {
        for (direction, identities) in directions
            .as_object()
            .with_context(|| format!("packets.json state {state} is not an object"))?
        {
            let identities = identities
                .as_object()
                .with_context(|| format!("packets.json {state}/{direction} is not an object"))?;
            let mut ids = BTreeSet::new();
            for (identity, record) in identities {
                let protocol_id = record
                    .get("protocol_id")
                    .and_then(Value::as_u64)
                    .with_context(|| {
                        format!("{state}/{direction}/{identity} misses protocol_id")
                    })?;
                ensure!(
                    ids.insert(protocol_id),
                    "duplicate packet ID {protocol_id} in {state}/{direction}"
                );
                packets.push(ProtocolPacket {
                    state: state.clone(),
                    direction: direction.clone(),
                    identity: identity.clone(),
                    protocol_id,
                });
            }
            ensure!(
                ids.iter().copied().eq(0..ids.len() as u64),
                "packet IDs are not contiguous from zero in {state}/{direction}"
            );
        }
    }
    packets.sort();
    Ok(packets)
}

fn protocol_inventory(context: &Context) -> Result<Vec<ProtocolPacket>> {
    let completion = load_protocol_completion(context)?;
    ensure!(
        completion.version == context.lock.version,
        "protocol ledger version differs from lock"
    );
    let packets = protocol_packets(context)?;
    let mut bytes = Vec::new();
    let mut counts = BTreeMap::<(String, String), usize>::new();
    for packet in &packets {
        writeln!(
            bytes,
            "{}\t{}\t{}\t{}",
            packet.state, packet.direction, packet.identity, packet.protocol_id
        )?;
        *counts
            .entry((packet.state.clone(), packet.direction.clone()))
            .or_default() += 1;
    }
    ensure!(
        packets.len() == completion.inventory.expected_count,
        "protocol inventory expected {} packets, found {}",
        completion.inventory.expected_count,
        packets.len()
    );
    ensure!(
        sha1_bytes(&bytes) == completion.inventory.entries_sha1,
        "protocol inventory digest differs from completion ledger"
    );
    for ((state, direction), count) in counts {
        println!("{state:13} {direction:11} {count:3} packets");
    }
    println!(
        "protocol inventory verified: {} packets, digest {}",
        packets.len(),
        completion.inventory.entries_sha1
    );
    Ok(packets)
}

fn protocol_coverage(context: &Context, require_ready: bool) -> Result<()> {
    let completion = load_protocol_completion(context)?;
    let packets = protocol_inventory(context)?;
    ensure!(
        !completion.family.is_empty(),
        "protocol ledger has no packet families"
    );
    let mut family_ids = BTreeSet::new();
    let mut matched_family_ids = BTreeSet::new();
    for family in &completion.family {
        ensure!(
            !family.id.trim().is_empty() && family_ids.insert(&family.id),
            "duplicate or empty protocol family ID {}",
            family.id
        );
        ensure!(
            !family.owner.trim().is_empty(),
            "{} has no owner",
            family.id
        );
        ensure!(!family.evidence.is_empty(), "{} has no evidence", family.id);
        ensure!(
            !family.patterns.is_empty(),
            "{} has no packet selectors",
            family.id
        );
        if !matches!(
            family.status,
            ProtocolStatus::Todo | ProtocolStatus::InProgress
        ) {
            ensure!(
                !family.specification.is_empty()
                    && context
                        .reference
                        .join("protocol")
                        .join(&family.specification)
                        .is_file(),
                "{} references a missing protocol specification",
                family.id
            );
            ensure!(
                !family.last_commit.is_empty(),
                "{} complete conclusion has no commit",
                family.id
            );
        }
        match family.status {
            ProtocolStatus::Todo | ProtocolStatus::InProgress => {
                ensure!(
                    !family.unknowns.is_empty() && !family.reproduction.is_empty(),
                    "{} has no recoverable work description",
                    family.id
                );
            }
            ProtocolStatus::Specified => {
                ensure!(
                    family.responsibility == ProtocolResponsibility::Required,
                    "{} Specified responsibility is not Required",
                    family.id
                );
                ensure!(
                    !family.specification.is_empty()
                        && !family.fields.is_empty()
                        && !family.mappings.is_empty()
                        && !family.transitions.is_empty()
                        && !family.ordering.is_empty()
                        && !family.vectors.is_empty()
                        && !family.last_commit.is_empty(),
                    "{} is falsely complete",
                    family.id
                );
            }
            ProtocolStatus::GatedOptional => {
                ensure!(
                    family.responsibility == ProtocolResponsibility::Optional,
                    "{} optional status/responsibility disagree",
                    family.id
                );
                ensure!(
                    !family.specification.is_empty()
                        && !family.vectors.is_empty()
                        && !family.last_commit.is_empty(),
                    "{} optional path is not justified and tested",
                    family.id
                );
            }
            ProtocolStatus::NonServerResponsibility => {
                ensure!(
                    family.responsibility == ProtocolResponsibility::NonServer,
                    "{} non-server status/responsibility disagree",
                    family.id
                );
                ensure!(
                    !family.specification.is_empty()
                        && !family.mappings.is_empty()
                        && !family.last_commit.is_empty(),
                    "{} non-server path is not justified",
                    family.id
                );
            }
            ProtocolStatus::SourceInconclusive => {
                ensure!(
                    !family.specification.is_empty()
                        && !family.unknowns.is_empty()
                        && !family.reproduction.is_empty(),
                    "{} inconclusive path has no exact unknown/reproduction",
                    family.id
                );
            }
        }
    }
    for packet in &packets {
        let mut matches = Vec::new();
        for family in &completion.family {
            if family.state != packet.state || family.direction != packet.direction {
                continue;
            }
            let mut builder = GlobSetBuilder::new();
            for pattern in &family.patterns {
                builder.add(
                    Glob::new(pattern)
                        .with_context(|| format!("invalid selector in {}", family.id))?,
                );
            }
            if builder.build()?.is_match(&packet.identity) {
                matches.push(family);
            }
        }
        ensure!(
            matches.len() == 1,
            "{}/{}/{} matched {} protocol families",
            packet.state,
            packet.direction,
            packet.identity,
            matches.len()
        );
        matched_family_ids.insert(&matches[0].id);
    }
    ensure!(
        matched_family_ids == family_ids,
        "one or more protocol families match zero locked packets"
    );
    let mut statuses = BTreeMap::<ProtocolStatus, usize>::new();
    let mut levels = BTreeMap::<ProtocolLevel, usize>::new();
    for family in &completion.family {
        *statuses.entry(family.status).or_default() += 1;
        *levels.entry(family.level).or_default() += 1;
    }
    println!(
        "protocol coverage complete: {} packets in {} families; levels {:?}; statuses {:?}",
        packets.len(),
        completion.family.len(),
        levels,
        statuses
    );
    if require_ready {
        let todo = statuses.get(&ProtocolStatus::Todo).copied().unwrap_or(0);
        let in_progress = statuses
            .get(&ProtocolStatus::InProgress)
            .copied()
            .unwrap_or(0);
        ensure!(
            todo == 0,
            "protocol readiness blocked by {todo} Todo families"
        );
        ensure!(
            in_progress == 0,
            "protocol readiness blocked by {in_progress} InProgress families"
        );
        println!("mc-reference protocol readiness complete");
    }
    Ok(())
}

fn protocol_verify(context: &Context) -> Result<()> {
    verify_cached_artifacts(context)?;
    verify_reports(context)?;
    protocol_coverage(context, false)?;
    println!("mc-reference protocol verification complete (offline)");
    Ok(())
}

fn surfaces(context: &Context, command: SurfaceCommand) -> Result<()> {
    match command {
        SurfaceCommand::Coverage => surface_coverage(context, false),
        SurfaceCommand::Readiness => surface_coverage(context, true),
        SurfaceCommand::Verify => surface_verify(context),
    }
}

fn load_behavior_surfaces(context: &Context) -> Result<BehaviorSurfaceFile> {
    let path = context.reference.join("behavior-surfaces.toml");
    toml::from_str(
        &fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?,
    )
    .with_context(|| format!("invalid {}", path.display()))
}

fn expected_surface_kinds() -> BTreeSet<BehaviorSurfaceKind> {
    BTreeSet::from([
        BehaviorSurfaceKind::TickScheduler,
        BehaviorSurfaceKind::NetworkIngress,
        BehaviorSurfaceKind::CommandAdministration,
        BehaviorSurfaceKind::ContentDispatch,
        BehaviorSurfaceKind::PlayerLifecycle,
        BehaviorSurfaceKind::WorldLifecycle,
        BehaviorSurfaceKind::PersistenceReload,
        BehaviorSurfaceKind::ClientProjection,
        BehaviorSurfaceKind::DataReload,
        BehaviorSurfaceKind::CrossSystemOrdering,
    ])
}

fn surface_coverage(context: &Context, require_ready: bool) -> Result<()> {
    let ledger = load_behavior_surfaces(context)?;
    ensure!(
        ledger.version == context.lock.version,
        "behavior-surface ledger targets {}, expected {}",
        ledger.version,
        context.lock.version
    );
    ensure!(
        !ledger.surface.is_empty(),
        "behavior-surface ledger is empty"
    );

    let rules = documented_rule_ids(context)?;
    let protocol = load_protocol_completion(context)?;
    let protocol_families = protocol
        .family
        .iter()
        .map(|family| family.id.as_str())
        .collect::<BTreeSet<_>>();
    let id_regex = Regex::new(r"^SURFACE-[A-Z0-9-]+-[0-9]{3}$")?;
    let mut ids = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    let mut statuses = BTreeMap::<BehaviorSurfaceStatus, usize>::new();
    let mut command_surface_status = None;
    let mut network_ingress_families = None;
    let mut cross_system_surface_status = None;

    for surface in &ledger.surface {
        ensure!(
            id_regex.is_match(&surface.id) && ids.insert(&surface.id),
            "duplicate or invalid behavior-surface ID {}",
            surface.id
        );
        ensure!(
            kinds.insert(surface.kind),
            "duplicate behavior-surface kind {:?}",
            surface.kind
        );
        if surface.kind == BehaviorSurfaceKind::CommandAdministration {
            command_surface_status = Some(surface.status);
        }
        if surface.kind == BehaviorSurfaceKind::NetworkIngress {
            network_ingress_families = Some(
                surface
                    .protocol_families
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
            );
        }
        if surface.kind == BehaviorSurfaceKind::CrossSystemOrdering {
            cross_system_surface_status = Some(surface.status);
        }
        ensure!(
            !surface.boundary.trim().is_empty()
                && !surface.triggers.is_empty()
                && !surface.inventory_sources.is_empty()
                && !surface.selectors.is_empty()
                && !surface.owners.is_empty()
                && !surface.state_domains.is_empty()
                && !surface.persistence.is_empty()
                && !surface.client_projection.is_empty()
                && !surface.evidence.is_empty(),
            "{} has an incomplete ownership boundary",
            surface.id
        );
        ensure!(
            surface
                .triggers
                .iter()
                .chain(&surface.selectors)
                .chain(&surface.state_domains)
                .chain(&surface.persistence)
                .chain(&surface.client_projection)
                .chain(&surface.evidence)
                .all(|value| !value.trim().is_empty()),
            "{} contains an empty boundary field",
            surface.id
        );
        for owner in &surface.owners {
            ensure!(
                rules.contains(owner),
                "{} references missing rule owner {owner}",
                surface.id
            );
        }
        for family in &surface.protocol_families {
            ensure!(
                protocol_families.contains(family.as_str()),
                "{} references missing protocol family {family}",
                surface.id
            );
        }
        match surface.status {
            BehaviorSurfaceStatus::Todo | BehaviorSurfaceStatus::InProgress => ensure!(
                !surface.unknowns.is_empty() && !surface.reproduction.is_empty(),
                "{} has no recoverable work description",
                surface.id
            ),
            BehaviorSurfaceStatus::Mapped => ensure!(
                surface.unknowns.is_empty()
                    && !surface.reproduction.is_empty()
                    && !surface.last_commit.trim().is_empty(),
                "{} is falsely mapped",
                surface.id
            ),
            BehaviorSurfaceStatus::SourceInconclusive => ensure!(
                !surface.unknowns.is_empty()
                    && !surface.reproduction.is_empty()
                    && !surface.last_commit.trim().is_empty(),
                "{} has no exact unknown, reproduction, or last conclusion",
                surface.id
            ),
        }
        *statuses.entry(surface.status).or_default() += 1;
    }

    ensure!(
        kinds == expected_surface_kinds(),
        "behavior-surface kinds differ from the required root inventory"
    );
    let expected_serverbound = protocol
        .family
        .iter()
        .filter(|family| family.direction == "serverbound")
        .map(|family| family.id.clone())
        .collect::<BTreeSet<_>>();
    validate_exact_protocol_family_partition(
        network_ingress_families
            .as_ref()
            .context("missing NetworkIngress protocol-family inventory")?,
        &expected_serverbound,
        "NetworkIngress",
    )?;
    let command_statuses = validate_command_roots(context, &rules)?;
    if command_surface_status == Some(BehaviorSurfaceStatus::Mapped) {
        ensure!(
            command_statuses.len() == 1
                && command_statuses.contains_key(&CommandRootStatus::Mapped),
            "CommandAdministration is falsely mapped while command-root work remains"
        );
    }
    let join_statuses = validate_cross_system_joins(context, &kinds, &rules)?;
    if cross_system_surface_status == Some(BehaviorSurfaceStatus::Mapped) {
        ensure!(
            !join_statuses.contains_key(&CrossSystemJoinStatus::InProgress),
            "CrossSystemOrdering is falsely mapped while join work remains"
        );
    }
    println!(
        "behavior-surface coverage complete: {} root surfaces; statuses {:?}",
        ledger.surface.len(),
        statuses
    );
    if require_ready {
        let todo = statuses
            .get(&BehaviorSurfaceStatus::Todo)
            .copied()
            .unwrap_or(0);
        let in_progress = statuses
            .get(&BehaviorSurfaceStatus::InProgress)
            .copied()
            .unwrap_or(0);
        ensure!(
            todo == 0 && in_progress == 0,
            "behavior-surface readiness blocked by {todo} Todo and {in_progress} InProgress roots"
        );
        println!("mc-reference behavior-surface readiness complete");
    }
    Ok(())
}

fn validate_exact_protocol_family_partition(
    actual: &BTreeSet<String>,
    expected: &BTreeSet<String>,
    label: &str,
) -> Result<()> {
    ensure!(
        actual == expected,
        "{label} protocol-family coverage differs: missing {:?}, extra {:?}",
        expected.difference(actual).collect::<Vec<_>>(),
        actual.difference(expected).collect::<Vec<_>>()
    );
    Ok(())
}

fn validate_cross_system_joins(
    context: &Context,
    surface_kinds: &BTreeSet<BehaviorSurfaceKind>,
    rules: &BTreeSet<String>,
) -> Result<BTreeMap<CrossSystemJoinStatus, usize>> {
    let path = context.reference.join("cross-system-joins.toml");
    let map: CrossSystemJoinMap = toml::from_str(
        &fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?,
    )
    .with_context(|| format!("invalid {}", path.display()))?;
    ensure!(
        map.version == context.lock.version,
        "cross-system join map targets {}, expected {}",
        map.version,
        context.lock.version
    );
    let statuses = validate_cross_system_join_map(&map, surface_kinds, rules)?;
    println!(
        "cross-system join matrix mapped: {} unordered pairs; statuses {:?}",
        map.join.len(),
        statuses
    );
    Ok(statuses)
}

fn validate_cross_system_join_map(
    map: &CrossSystemJoinMap,
    surface_kinds: &BTreeSet<BehaviorSurfaceKind>,
    rules: &BTreeSet<String>,
) -> Result<BTreeMap<CrossSystemJoinStatus, usize>> {
    let roots = surface_kinds
        .iter()
        .copied()
        .filter(|kind| *kind != BehaviorSurfaceKind::CrossSystemOrdering)
        .collect::<Vec<_>>();
    let mut expected = BTreeSet::new();
    for (index, left) in roots.iter().enumerate() {
        for right in roots.iter().skip(index + 1) {
            expected.insert((*left, *right));
        }
    }

    let mut actual = BTreeSet::new();
    let mut statuses = BTreeMap::new();
    for join in &map.join {
        ensure!(
            join.left < join.right,
            "cross-system join {:?}/{:?} is not in canonical order",
            join.left,
            join.right
        );
        ensure!(
            actual.insert((join.left, join.right)),
            "duplicate cross-system join {:?}/{:?}",
            join.left,
            join.right
        );
        match join.status {
            CrossSystemJoinStatus::Empty => ensure!(
                join.shared_domains.is_empty()
                    && join.owners.is_empty()
                    && join.remaining_work.is_empty(),
                "empty cross-system join {:?}/{:?} has ownership claims",
                join.left,
                join.right
            ),
            CrossSystemJoinStatus::InProgress | CrossSystemJoinStatus::SourceInconclusive => {
                ensure!(
                    !join.shared_domains.is_empty()
                        && !join.owners.is_empty()
                        && !join.remaining_work.is_empty(),
                    "cross-system join {:?}/{:?} has no recoverable ownership",
                    join.left,
                    join.right
                );
            }
            CrossSystemJoinStatus::Mapped => ensure!(
                !join.shared_domains.is_empty()
                    && !join.owners.is_empty()
                    && join.remaining_work.is_empty(),
                "cross-system join {:?}/{:?} is falsely mapped",
                join.left,
                join.right
            ),
        }
        for owner in &join.owners {
            ensure!(
                rules.contains(owner),
                "cross-system join {:?}/{:?} references missing owner {owner}",
                join.left,
                join.right
            );
        }
        *statuses.entry(join.status).or_default() += 1;
    }
    ensure!(
        actual == expected,
        "cross-system pair coverage differs: missing {:?}, extra {:?}",
        expected.difference(&actual).collect::<Vec<_>>(),
        actual.difference(&expected).collect::<Vec<_>>()
    );
    Ok(statuses)
}

fn validate_command_roots(
    context: &Context,
    rules: &BTreeSet<String>,
) -> Result<BTreeMap<CommandRootStatus, usize>> {
    let path = context.reference.join("command-roots.toml");
    let map: CommandRootMap = toml::from_str(
        &fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?,
    )
    .with_context(|| format!("invalid {}", path.display()))?;
    ensure!(
        map.version == context.lock.version,
        "command-root map targets {}, expected {}",
        map.version,
        context.lock.version
    );
    let report = read_json(&context.cache.join("generated/reports/commands.json"))?;
    let official = report
        .get("children")
        .and_then(Value::as_object)
        .context("commands.json root has no children object")?
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    validate_command_root_map(&map, &official, rules)?;
    let mut statuses = BTreeMap::<CommandRootStatus, usize>::new();
    for family in &map.family {
        *statuses.entry(family.status).or_default() += 1;
    }
    println!(
        "command-root inventory mapped: {} roots in {} recoverable families; statuses {:?}",
        official.len(),
        map.family.len(),
        statuses
    );
    Ok(statuses)
}

fn validate_command_root_map(
    map: &CommandRootMap,
    official: &BTreeSet<String>,
    rules: &BTreeSet<String>,
) -> Result<()> {
    ensure!(
        official.len() == map.inventory.expected_count,
        "command-root count {} differs from lock {}",
        official.len(),
        map.inventory.expected_count
    );
    ensure!(
        ids_digest(&official) == map.inventory.roots_sha1,
        "command-root digest differs from lock"
    );

    let mut family_names = BTreeSet::new();
    let mut mapped = BTreeSet::new();
    for family in &map.family {
        ensure!(
            !family.name.trim().is_empty() && family_names.insert(&family.name),
            "duplicate or empty command-root family {}",
            family.name
        );
        ensure!(
            !family.roots.is_empty()
                && !family.owners.is_empty()
                && !family.state_domains.is_empty(),
            "command-root family {} has incomplete ownership",
            family.name
        );
        match family.status {
            CommandRootStatus::InProgress | CommandRootStatus::SourceInconclusive => ensure!(
                !family.remaining_work.is_empty(),
                "command-root family {} has no recoverable work",
                family.name
            ),
            CommandRootStatus::Mapped => ensure!(
                family.remaining_work.is_empty(),
                "command-root family {} is falsely mapped",
                family.name
            ),
        }
        for owner in &family.owners {
            ensure!(
                rules.contains(owner),
                "command-root family {} references missing rule owner {owner}",
                family.name
            );
        }
        for root in &family.roots {
            ensure!(
                official.contains(root),
                "command-root family {} contains stale root {root}",
                family.name
            );
            ensure!(
                mapped.insert(root.clone()),
                "command root {root} belongs to multiple families"
            );
        }
    }
    ensure!(
        mapped.iter().eq(official.iter()),
        "command-root coverage differs: missing {:?}",
        official.difference(&mapped).collect::<Vec<_>>()
    );
    Ok(())
}

fn surface_verify(context: &Context) -> Result<()> {
    verify_cached_artifacts(context)?;
    verify_reports(context)?;
    surface_coverage(context, false)?;
    println!("mc-reference behavior-surface verification complete (offline)");
    Ok(())
}

fn completion_slice_has_ownership(slice: &CompletionSlice) -> bool {
    !slice.id.trim().is_empty()
        && !slice.subsystem.trim().is_empty()
        && !slice.parents.is_empty()
        && !slice.leaves.is_empty()
        && !slice.selectors.is_empty()
        && (!slice.symbols.is_empty() || !slice.data_paths.is_empty())
}

fn validate_completion(context: &Context, require_complete: bool) -> Result<()> {
    let completion: CompletionFile = toml::from_str(&fs::read_to_string(
        context.reference.join("completion.toml"),
    )?)?;
    ensure!(
        completion.version == context.lock.version,
        "completion ledger targets {}, expected {}",
        completion.version,
        context.lock.version
    );

    let parent_regex = Regex::new(r"(?m)^## `([A-Z][A-Z0-9-]+)`")?;
    let leaf_regex = Regex::new(r"(?m)^## Leaf rule `([A-Z][A-Z0-9-]+)`")?;
    let mut parents = BTreeSet::new();
    let mut leaves = BTreeSet::new();
    for file in markdown_files(&context.reference) {
        let text = fs::read_to_string(file)?;
        parents.extend(
            parent_regex
                .captures_iter(&text)
                .map(|capture| capture[1].to_string()),
        );
        leaves.extend(
            leaf_regex
                .captures_iter(&text)
                .map(|capture| capture[1].to_string()),
        );
    }
    let experiments: BTreeSet<_> = load_experiments(context)?
        .into_iter()
        .map(|experiment| experiment.id)
        .collect();

    let mut slice_ids = BTreeSet::new();
    let mut covered_parents = BTreeSet::new();
    let mut covered_leaves = BTreeSet::new();
    let mut statuses = BTreeMap::<CompletionStatus, usize>::new();
    for slice in &completion.slice {
        ensure!(
            slice_ids.insert(&slice.id),
            "duplicate completion slice {}",
            slice.id
        );
        ensure!(
            completion_slice_has_ownership(slice),
            "completion slice {} has incomplete ownership fields",
            slice.id
        );
        ensure!(
            slice
                .registry_kinds
                .iter()
                .all(|registry| !registry.trim().is_empty()),
            "completion slice {} has an empty registry kind",
            slice.id
        );
        for parent in &slice.parents {
            ensure!(
                parents.contains(parent),
                "completion slice {} references unknown parent {parent}",
                slice.id
            );
            covered_parents.insert(parent.clone());
        }
        for leaf in &slice.leaves {
            ensure!(
                leaves.contains(leaf),
                "completion slice {} references unknown leaf {leaf}",
                slice.id
            );
            ensure!(
                covered_leaves.insert(leaf.clone()),
                "leaf {leaf} is owned by multiple completion slices"
            );
        }
        for experiment in &slice.experiments {
            ensure!(
                experiments.contains(experiment),
                "completion slice {} references unknown experiment {experiment}",
                slice.id
            );
        }
        if matches!(
            slice.status,
            CompletionStatus::SourceSpecified
                | CompletionStatus::DataOnlyVerified
                | CompletionStatus::SourceInconclusive
        ) {
            ensure!(
                !slice.last_commit.trim().is_empty(),
                "completed slice {} has no last_commit",
                slice.id
            );
        }
        if slice.status == CompletionStatus::SourceInconclusive {
            ensure!(
                !slice.unknowns.is_empty() && !slice.reproduction.is_empty(),
                "SourceInconclusive slice {} needs exact unknowns and reproduction",
                slice.id
            );
        }
        *statuses.entry(slice.status).or_default() += 1;
    }
    ensure!(
        covered_parents == parents,
        "completion parent coverage differs: missing {:?}",
        parents.difference(&covered_parents).collect::<Vec<_>>()
    );
    ensure!(
        covered_leaves == leaves,
        "completion leaf coverage differs: missing {:?}",
        leaves.difference(&covered_leaves).collect::<Vec<_>>()
    );

    let official = read_json(&context.cache.join("generated/reports/registries.json"))?
        .as_object()
        .context("registries.json is not an object")?
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut scoped = BTreeSet::new();
    for registry in &completion.registry {
        ensure!(
            scoped.insert(registry.id.clone()),
            "duplicate registry scope {}",
            registry.id
        );
        ensure!(
            !registry.reason.trim().is_empty(),
            "registry {} has no scope reason",
            registry.id
        );
        let _scope = &registry.scope;
    }
    ensure!(
        scoped == official,
        "registry scope differs: missing {:?}, stale {:?}",
        official.difference(&scoped).collect::<Vec<_>>(),
        scoped.difference(&official).collect::<Vec<_>>()
    );

    let catalog = load_catalog(context)?;
    let blocks = load_category_ids(context, "block")?;
    let mut unreviewed = 0;
    for category in &catalog.category {
        let ids = load_category_ids(context, &category.kind)?;
        validate_family_selectors(category, &ids, &blocks)?;
        for id in &ids {
            if classify(&catalog, &category.kind, id, Some(&blocks))?
                .family
                .classification
                == Classification::Unreviewed
            {
                unreviewed += 1;
            }
        }
    }

    let todo = statuses.get(&CompletionStatus::Todo).copied().unwrap_or(0);
    let in_progress = statuses
        .get(&CompletionStatus::InProgress)
        .copied()
        .unwrap_or(0);
    let source_specified = statuses
        .get(&CompletionStatus::SourceSpecified)
        .copied()
        .unwrap_or(0);
    let data_only = statuses
        .get(&CompletionStatus::DataOnlyVerified)
        .copied()
        .unwrap_or(0);
    let source_inconclusive = statuses
        .get(&CompletionStatus::SourceInconclusive)
        .copied()
        .unwrap_or(0);
    println!(
        "readiness ledger: {} slices (Todo {todo}, InProgress {in_progress}, SourceSpecified {source_specified}, DataOnlyVerified {data_only}, SourceInconclusive {source_inconclusive}), {} parent rules, {} leaf rules, {} registries; {unreviewed} unreviewed catalog IDs",
        completion.slice.len(),
        parents.len(),
        leaves.len(),
        scoped.len()
    );
    if require_complete {
        ensure!(todo == 0, "readiness blocked by {todo} Todo slices");
        ensure!(
            in_progress == 0,
            "readiness blocked by {in_progress} InProgress slices"
        );
        ensure!(
            unreviewed == 0,
            "readiness blocked by {unreviewed} unreviewed catalog IDs"
        );
        println!("mc-reference source readiness complete");
    } else {
        println!("completion ledger consistency verified");
    }
    Ok(())
}

fn verify_cached_artifacts(context: &Context) -> Result<()> {
    verify_file(
        &context.cache.join("version.json"),
        &context.lock.metadata.sha1,
        context.lock.metadata.size,
    )?;
    verify_file(
        &context.cache.join("client.jar"),
        &context.lock.client.sha1,
        context.lock.client.size,
    )?;
    verify_file(
        &context.cache.join("server.jar"),
        &context.lock.server.sha1,
        context.lock.server.size,
    )?;
    Ok(())
}

fn verify_reports(context: &Context) -> Result<()> {
    for file in ["blocks.json", "registries.json", "commands.json"] {
        ensure!(
            context.cache.join("generated/reports").join(file).is_file(),
            "missing generated report {file}"
        );
    }
    Ok(())
}

fn validate_docs(context: &Context) -> Result<()> {
    let rule_regex = Regex::new(r"(?m)^## `([A-Z][A-Z0-9-]+)`")?;
    let leaf_regex = Regex::new(r"(?m)^## Leaf rule `([A-Z][A-Z0-9-]+)`")?;
    let link_regex = Regex::new(r"\]\(([^)]+)\)")?;
    let required = [
        "Parent",
        "FidelityClass",
        "EvidenceStatus",
        "SourceConclusion",
        "Applies when",
        "Authoritative state",
        "Transition and ordering",
        "Branches and aborts",
        "Constants and randomness",
        "Side effects",
        "Gates",
        "Boundary cases and quirks",
        "Evidence",
        "Test vectors",
    ];
    let mut ids = BTreeSet::new();
    let mut parent_ids = BTreeSet::new();
    let mut referenced_parents = BTreeSet::new();
    let mut leaves = 0;
    for file in markdown_files(&context.reference) {
        let text = fs::read_to_string(&file)?;
        for captures in rule_regex.captures_iter(&text) {
            parent_ids.insert(captures[1].to_string());
            ensure!(
                ids.insert(captures[1].to_string()),
                "duplicate rule ID {}",
                &captures[1]
            );
        }
        for captures in leaf_regex.captures_iter(&text) {
            ensure!(
                ids.insert(captures[1].to_string()),
                "duplicate rule ID {}",
                &captures[1]
            );
            leaves += 1;
            let start = captures.get(0).unwrap().start();
            let end = text[start + 1..]
                .find("\n## ")
                .map(|v| start + 1 + v)
                .unwrap_or(text.len());
            let section = &text[start..end];
            for field in required {
                ensure!(
                    section.contains(&format!("**{field}:**")),
                    "{} in {} misses {field}",
                    &captures[1],
                    file.display()
                );
            }
        }
        for line in text.lines().filter(|line| line.starts_with("**Parent:**")) {
            for captures in Regex::new(r"`([A-Z]+-\d+)`")?.captures_iter(line) {
                referenced_parents.insert(captures[1].to_string());
            }
        }
        for captures in link_regex.captures_iter(&text) {
            let link = captures[1].trim().trim_matches(['<', '>']);
            if link.starts_with("https://") || link.starts_with("http://") {
                reqwest::Url::parse(link).with_context(|| {
                    format!("invalid external link {link} in {}", file.display())
                })?;
                if link.contains("minecraft.wiki/") {
                    ensure!(
                        link.contains("oldid="),
                        "community Wiki link is not revision-pinned in {}: {link}",
                        file.display()
                    );
                }
                continue;
            }
            if link.starts_with('#') || link.starts_with("mailto:") {
                continue;
            }
            let target = link.split('#').next().unwrap_or(link);
            if target.is_empty() {
                continue;
            }
            let resolved = file.parent().unwrap_or(&context.reference).join(target);
            let exists = resolved.is_file() || resolved.join("README.md").is_file();
            ensure!(exists, "broken internal link in {}: {link}", file.display());
        }
    }
    ensure!(parent_ids.len() == 65, "expected 65 stable parent rules");
    ensure!(
        referenced_parents == parent_ids,
        "leaf parent coverage differs: missing {:?}, unknown {:?}",
        parent_ids
            .difference(&referenced_parents)
            .collect::<Vec<_>>(),
        referenced_parents
            .difference(&parent_ids)
            .collect::<Vec<_>>()
    );
    ensure!(leaves > 0, "no leaf rules found");
    println!(
        "documentation schema verified: {} IDs including {} leaf rules",
        ids.len(),
        leaves
    );
    Ok(())
}

fn validate_rule_references(context: &Context, catalog: &Catalog) -> Result<()> {
    let ids = documented_rule_ids(context)?;
    for category in &catalog.category {
        ensure!(
            !category.family.is_empty(),
            "{} has no behavior families",
            category.kind
        );
        let remaining = category
            .family
            .iter()
            .filter(|family| family.remaining)
            .count();
        ensure!(
            remaining <= 1,
            "{} has multiple remaining families",
            category.kind
        );
        for family in &category.family {
            ensure!(
                !family.rules.is_empty(),
                "{}/{} has no rule references",
                category.kind,
                family.name
            );
            for rule in &family.rules {
                ensure!(
                    ids.contains(rule),
                    "{}/{} references missing rule {rule}",
                    category.kind,
                    family.name
                );
            }
        }
    }
    Ok(())
}

fn documented_rule_ids(context: &Context) -> Result<BTreeSet<String>> {
    let regex = Regex::new(r"`([A-Z]{2,}(?:-[A-Z0-9]+)+)`")?;
    let mut ids = BTreeSet::new();
    for file in markdown_files(&context.reference) {
        let text = fs::read_to_string(file)?;
        for captures in regex.captures_iter(&text) {
            if captures[1].starts_with("EXP-") {
                continue;
            }
            ids.insert(captures[1].to_string());
        }
    }
    Ok(ids)
}

fn load_catalog(context: &Context) -> Result<Catalog> {
    Ok(toml::from_str(&fs::read_to_string(
        context.reference.join("catalog/catalog.toml"),
    )?)?)
}

fn load_experiments(context: &Context) -> Result<Vec<Experiment>> {
    let mut experiments = Vec::new();
    for entry in fs::read_dir(context.reference.join("experiments"))? {
        let path = entry?.path();
        if path.extension().and_then(|v| v.to_str()) == Some("toml") {
            experiments
                .extend(toml::from_str::<ExperimentFile>(&fs::read_to_string(path)?)?.experiment);
        }
    }
    Ok(experiments)
}

fn hygiene(context: &Context) -> Result<()> {
    let forbidden_extensions = ["jar", "class", "mca", "mcr"];
    for entry in WalkDir::new(&context.workspace)
        .into_iter()
        .filter_entry(|entry| entry.file_name() != "target")
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if path
                .components()
                .any(|component| component.as_os_str() == ".git")
            {
                continue;
            }
            let extension = path.extension().and_then(|v| v.to_str()).unwrap_or("");
            ensure!(
                !forbidden_extensions.contains(&extension),
                "forbidden generated artifact in repository: {}",
                path.display()
            );
        }
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_reader(BufReader::new(
        File::open(path)
            .with_context(|| format!("missing {}; run mc-ref reports", path.display()))?,
    ))?)
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|v| v.to_str()) == Some("md"))
        .map(|entry| entry.into_path())
        .collect()
}

fn normalize_id(id: &str) -> Result<String> {
    let value = normalize_unchecked(id);
    let regex = Regex::new(r"^[a-z0-9_.-]+:[a-z0-9_./-]+$")?;
    ensure!(regex.is_match(&value), "invalid namespaced ID {id}");
    Ok(value)
}

fn normalize_unchecked(id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    }
}

fn strip_namespace(id: &str) -> &str {
    id.split_once(':').map(|(_, path)| path).unwrap_or(id)
}
fn escape_pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
fn sha1_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha1::digest(bytes))
}
fn ids_digest(ids: &BTreeSet<String>) -> String {
    let mut hasher = Sha1::new();
    for id in ids {
        hasher.update(id.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn ids_are_namespaced() {
        assert_eq!(normalize_id("stone").unwrap(), "minecraft:stone");
        assert_eq!(
            normalize_id("minecraft:oak_log").unwrap(),
            "minecraft:oak_log"
        );
        assert!(normalize_id("Bad ID").is_err());
    }

    #[test]
    fn digest_is_sorted_and_newline_terminated() {
        let ids = BTreeSet::from(["minecraft:b".into(), "minecraft:a".into()]);
        assert_eq!(ids_digest(&ids), sha1_bytes(b"minecraft:a\nminecraft:b\n"));
    }

    #[test]
    fn parses_manifest() {
        let manifest: Manifest =
            serde_json::from_str(r#"{"versions":[{"id":"26.2","url":"u","sha1":"s"}]}"#).unwrap();
        assert_eq!(manifest.versions[0].id, "26.2");
    }

    #[test]
    fn accepts_live_manifest_metadata_drift_for_a_locked_version() {
        let manifest: Manifest = serde_json::from_str(
            r#"{"versions":[{"id":"26.2","url":"revised","sha1":"revised"}]}"#,
        )
        .unwrap();
        let locked = Artifact {
            url: "locked".into(),
            sha1: "locked".into(),
            size: None,
        };
        assert!(!manifest_metadata_is_current(&manifest, "26.2", &locked).unwrap());
        assert!(manifest_metadata_is_current(&manifest, "missing", &locked).is_err());
    }

    #[test]
    fn command_root_map_requires_an_exact_owned_partition() {
        let official = BTreeSet::from(["help".to_string()]);
        let rules = BTreeSet::from(["SIM-001".to_string()]);
        let mut map = CommandRootMap {
            version: "26.2".into(),
            inventory: CommandRootInventoryLock {
                expected_count: 1,
                roots_sha1: ids_digest(&official),
            },
            family: vec![CommandRootFamily {
                name: "informational".into(),
                roots: vec!["help".into()],
                owners: vec!["SIM-001".into()],
                state_domains: vec!["feedback".into()],
                status: CommandRootStatus::InProgress,
                remaining_work: vec!["audit leaves".into()],
            }],
        };
        validate_command_root_map(&map, &official, &rules).unwrap();
        map.family[0].roots.push("stale".into());
        assert!(validate_command_root_map(&map, &official, &rules).is_err());
    }

    #[test]
    fn cross_system_join_map_requires_every_unordered_root_pair() {
        let surfaces = BTreeSet::from([
            BehaviorSurfaceKind::TickScheduler,
            BehaviorSurfaceKind::NetworkIngress,
            BehaviorSurfaceKind::CrossSystemOrdering,
        ]);
        let rules = BTreeSet::from(["SIM-001".to_string()]);
        let mut map = CrossSystemJoinMap {
            version: "26.2".into(),
            join: vec![CrossSystemJoin {
                left: BehaviorSurfaceKind::TickScheduler,
                right: BehaviorSurfaceKind::NetworkIngress,
                shared_domains: vec!["server thread".into()],
                owners: vec!["SIM-001".into()],
                status: CrossSystemJoinStatus::InProgress,
                remaining_work: vec!["specify ordering".into()],
            }],
        };
        validate_cross_system_join_map(&map, &surfaces, &rules).unwrap();
        map.join[0].right = BehaviorSurfaceKind::CrossSystemOrdering;
        assert!(validate_cross_system_join_map(&map, &surfaces, &rules).is_err());
    }

    #[test]
    fn network_ingress_requires_exact_serverbound_family_partition() {
        let expected = BTreeSet::from(["required".to_string(), "optional".to_string()]);
        validate_exact_protocol_family_partition(&expected, &expected, "NetworkIngress").unwrap();
        let incomplete = BTreeSet::from(["required".to_string()]);
        assert!(
            validate_exact_protocol_family_partition(&incomplete, &expected, "NetworkIngress")
                .is_err()
        );
    }

    #[test]
    fn catalog_requires_exactly_one_family() {
        let catalog = Catalog {
            category: vec![Category {
                kind: "block".into(),
                source: "reports/blocks.json".into(),
                expected_count: 1,
                ids_sha1: "x".into(),
                family: vec![Family {
                    name: "generic".into(),
                    classification: Classification::DataOnly,
                    rules: vec!["BLK-001".into()],
                    exact: vec![],
                    patterns: vec![],
                    block_items: false,
                    remaining: true,
                }],
            }],
        };
        assert_eq!(
            classify(&catalog, "block", "minecraft:stone", None)
                .unwrap()
                .family
                .name,
            "generic"
        );
    }

    #[test]
    fn catalog_rejects_stale_exact_ids_and_zero_match_patterns() {
        let category = Category {
            kind: "entity_type".into(),
            source: "reports/registries.json#minecraft:entity_type".into(),
            expected_count: 1,
            ids_sha1: "x".into(),
            family: vec![Family {
                name: "projectile".into(),
                classification: Classification::BehaviorFamily,
                rules: vec!["ENT-004".into()],
                exact: vec!["removed_projectile".into()],
                patterns: vec!["*_missing_pattern".into()],
                block_items: false,
                remaining: false,
            }],
        };
        let ids = BTreeSet::from(["minecraft:arrow".to_string()]);
        assert!(validate_family_selectors(&category, &ids, &BTreeSet::new()).is_err());
    }

    #[test]
    fn catalog_rejects_special_remaining_fallbacks() {
        let category = Category {
            kind: "item".into(),
            source: "reports/minecraft/components/item/<id>.json".into(),
            expected_count: 1,
            ids_sha1: "x".into(),
            family: vec![Family {
                name: "remaining-special-items".into(),
                classification: Classification::Special,
                rules: vec!["ITM-001".into()],
                exact: vec![],
                patterns: vec![],
                block_items: false,
                remaining: true,
            }],
        };
        let ids = BTreeSet::from(["minecraft:stick".to_string()]);
        assert!(validate_family_selectors(&category, &ids, &BTreeSet::new()).is_err());
    }

    #[test]
    fn catalog_rejects_unapproved_data_only_fallbacks() {
        let category = Category {
            kind: "worldgen".into(),
            source: "data/minecraft/worldgen/**".into(),
            expected_count: 1,
            ids_sha1: "x".into(),
            family: vec![Family {
                name: "remaining-worldgen".into(),
                classification: Classification::DataOnly,
                rules: vec!["WGEN-001".into()],
                exact: vec![],
                patterns: vec![],
                block_items: false,
                remaining: true,
            }],
        };
        let ids = BTreeSet::from(["minecraft:worldgen/example".to_string()]);
        assert!(validate_family_selectors(&category, &ids, &BTreeSet::new()).is_err());
    }

    #[test]
    fn unreviewed_fallback_is_not_reported_as_data_only() {
        let catalog = Catalog {
            category: vec![Category {
                kind: "block".into(),
                source: "reports/blocks.json".into(),
                expected_count: 1,
                ids_sha1: "x".into(),
                family: vec![Family {
                    name: "unreviewed-block".into(),
                    classification: Classification::Unreviewed,
                    rules: vec!["BLK-001".into()],
                    exact: vec![],
                    patterns: vec![],
                    block_items: false,
                    remaining: true,
                }],
            }],
        };
        let matched = classify(&catalog, "block", "minecraft:stone", None).unwrap();
        assert_eq!(matched.family.classification, Classification::Unreviewed);
    }

    #[test]
    fn verifies_cached_artifact_hash_and_size() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("artifact.bin");
        fs::write(&path, b"locked").unwrap();
        verify_file(&path, &sha1_bytes(b"locked"), Some(6)).unwrap();
        assert!(verify_file(&path, &sha1_bytes(b"changed"), Some(6)).is_err());
        assert!(verify_file(&path, &sha1_bytes(b"locked"), Some(7)).is_err());
    }

    #[test]
    fn parses_report_id_paths() {
        let directory = tempdir().unwrap();
        let nested = directory.path().join("boats");
        fs::create_dir(&nested).unwrap();
        fs::write(directory.path().join("stone.json"), b"{}").unwrap();
        fs::write(nested.join("oak.json"), b"{}").unwrap();
        let ids = ids_from_files(directory.path(), "json").unwrap();
        assert_eq!(
            ids,
            BTreeSet::from([
                "minecraft:stone".to_string(),
                "minecraft:boats/oak".to_string()
            ])
        );
    }

    #[test]
    fn generic_registry_queries_support_new_catalog_kinds() {
        let registries = serde_json::json!({
            "minecraft:ticket_type": {
                "entries": {
                    "minecraft:portal": { "protocol_id": 6 },
                    "minecraft:forced": { "protocol_id": 5 }
                }
            },
            "minecraft:worldgen/density_function_type": {
                "entries": {
                    "minecraft:constant": { "protocol_id": 0 }
                }
            },
            "minecraft:worldgen/material_condition": {
                "entries": {
                    "minecraft:stone_depth": { "protocol_id": 10 }
                }
            },
            "minecraft:worldgen/material_rule": {
                "entries": {
                    "minecraft:sequence": { "protocol_id": 2 }
                }
            },
            "minecraft:worldgen/structure_type": {
                "entries": {
                    "minecraft:buried_treasure": { "protocol_id": 0 }
                }
            },
            "minecraft:worldgen/pool_alias_binding": {
                "entries": {
                    "minecraft:direct": { "protocol_id": 2 }
                }
            },
            "minecraft:worldgen/structure_pool_element": {
                "entries": {
                    "minecraft:list_pool_element": { "protocol_id": 1 }
                }
            },
            "minecraft:worldgen/structure_processor": {
                "entries": {
                    "minecraft:rule": { "protocol_id": 10 }
                }
            }
        });
        assert_eq!(
            registry_ids(&registries, "ticket_type").unwrap(),
            BTreeSet::from([
                "minecraft:forced".to_string(),
                "minecraft:portal".to_string()
            ])
        );
        assert_eq!(
            registry_entry(&registries, "ticket_type", "minecraft:portal").unwrap()["protocol_id"],
            6
        );
        assert_eq!(
            registry_ids(&registries, "density_function_type").unwrap(),
            BTreeSet::from(["minecraft:constant".to_string()])
        );
        assert_eq!(
            registry_entry(&registries, "density_function_type", "minecraft:constant").unwrap()["protocol_id"],
            0
        );
        assert_eq!(
            registry_ids(&registries, "material_condition").unwrap(),
            BTreeSet::from(["minecraft:stone_depth".to_string()])
        );
        assert_eq!(
            registry_entry(&registries, "material_rule", "minecraft:sequence").unwrap()["protocol_id"],
            2
        );
        assert_eq!(
            registry_ids(&registries, "structure_type").unwrap(),
            BTreeSet::from(["minecraft:buried_treasure".to_string()])
        );
        assert_eq!(
            registry_entry(&registries, "structure_type", "minecraft:buried_treasure").unwrap()["protocol_id"],
            0
        );
        assert_eq!(
            registry_entry(&registries, "pool_alias_binding", "minecraft:direct").unwrap()["protocol_id"],
            2
        );
        assert_eq!(
            registry_ids(&registries, "structure_pool_element").unwrap(),
            BTreeSet::from(["minecraft:list_pool_element".to_string()])
        );
        assert_eq!(
            registry_entry(&registries, "structure_processor", "minecraft:rule").unwrap()["protocol_id"],
            10
        );
        assert!(registry_entry(&registries, "ticket_type", "minecraft:removed").is_err());
    }

    #[test]
    fn data_backed_catalog_kinds_have_locked_jar_paths() {
        assert_eq!(
            server_data_prefix("sulfur_cube_archetype"),
            Some("data/minecraft/sulfur_cube_archetype")
        );
        assert_eq!(server_data_prefix("entity_type"), None);
    }

    #[test]
    fn parses_experiment_definition_schema() {
        let file: ExperimentFile = toml::from_str(
            r#"
                [[experiment]]
                id = "EXP-TST-001"
                rules = ["SIM-001"]
                mode = "gametest"
                status = "planned"
                repeats = 1
                initial_state = ["empty"]
                action = [{ tick = 0, value = "act" }]
                observation = [{ tick = 1, value = "observe" }]
                expected = ["done"]
            "#,
        )
        .unwrap();
        assert_eq!(file.experiment[0].id, "EXP-TST-001");
        assert_eq!(file.experiment[0].observation[0].tick, 1);
    }

    #[test]
    fn parses_completion_ledger_schema() {
        let completion: CompletionFile = toml::from_str(
            r#"
                version = "26.2"
                [[slice]]
                id = "TST-SLICE-001"
                subsystem = "test"
                parents = ["SIM-001"]
                leaves = ["SIM-PIPELINE-001"]
                registry_kinds = []
                selectors = ["minecraft:stone"]
                symbols = ["net.minecraft.Test#tick"]
                data_paths = []
                status = "SourceInconclusive"
                unknowns = ["Client presentation is outside the server source boundary."]
                reproduction = ["Observe one client tick after the server event."]
                experiments = ["EXP-SIM-001"]
                last_commit = "deadbee"

                [[registry]]
                id = "minecraft:block"
                scope = "GameplayBehavior"
                reason = "Blocks select gameplay behavior."
            "#,
        )
        .unwrap();
        assert_eq!(completion.version, "26.2");
        assert!(completion_slice_has_ownership(&completion.slice[0]));
        assert!(completion.slice[0].registry_kinds.is_empty());
        assert_eq!(
            completion.slice[0].status,
            CompletionStatus::SourceInconclusive
        );
        assert_eq!(completion.registry[0].id, "minecraft:block");
    }

    #[test]
    fn parses_protocol_completion_ledger_schema() {
        let parsed: ProtocolCompletionFile = toml::from_str(
            r#"
version = "26.2"
[inventory]
expected_count = 1
entries_sha1 = "abc"
[[family]]
id = "PROTO-STATUS-001"
level = "C0"
state = "status"
direction = "serverbound"
patterns = ["minecraft:status_request"]
status = "Todo"
responsibility = "Required"
owner = "protocol/handshake-and-status"
specification = ""
evidence = ["OFF-REPORT-001"]
fields = []
mappings = []
transitions = []
ordering = []
vectors = []
unknowns = ["field layout"]
reproduction = ["trace codec"]
last_commit = ""
"#,
        )
        .unwrap();
        assert_eq!(parsed.inventory.expected_count, 1);
        assert_eq!(parsed.family.len(), 1);
        assert_eq!(parsed.family[0].level, ProtocolLevel::C0);
    }

    #[test]
    fn parses_behavior_surface_ledger_schema() {
        let parsed: BehaviorSurfaceFile = toml::from_str(
            r#"
version = "26.2"
[[surface]]
id = "SURFACE-TICK-SCHEDULER-001"
kind = "TickScheduler"
boundary = "server tick"
triggers = ["fixed tick"]
inventory_sources = ["OfficialServerSymbols"]
selectors = ["tick roots"]
owners = ["SIM-001"]
state_domains = ["world state"]
persistence = ["clock continuity"]
client_projection = ["time update"]
protocol_families = []
status = "Mapped"
evidence = ["OFF-SERVER-001"]
unknowns = []
reproduction = ["run the tick vector"]
last_commit = "deadbee"
"#,
        )
        .unwrap();
        assert_eq!(parsed.version, "26.2");
        assert_eq!(parsed.surface.len(), 1);
        assert_eq!(parsed.surface[0].kind, BehaviorSurfaceKind::TickScheduler);
        assert_eq!(parsed.surface[0].status, BehaviorSurfaceStatus::Mapped);
        assert_eq!(expected_surface_kinds().len(), 10);
    }

    #[test]
    fn matches_jvm_descriptors_instead_of_generic_declarations() {
        let javap = "  public void tick(net.minecraft.server.level.ServerLevel, E);\n    descriptor: (Lnet/minecraft/server/level/ServerLevel;Lnet/minecraft/world/entity/LivingEntity;)V\n";
        assert!(descriptor_matches(
            javap,
            "tick",
            "(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity)"
        ));
    }
}
