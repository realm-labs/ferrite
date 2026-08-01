#![forbid(unsafe_code)]

use anyhow::{Context as _, Result, bail, ensure};
use globset::{Glob, GlobSet, GlobSetBuilder};
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
use std::process::{self, Command as ProcessCommand};
use walkdir::WalkDir;
use zip::ZipArchive;

mod artifact;
mod catalog;
mod experiments;
mod implementation_manifest;
mod protocol;
mod surface;
mod symbols;
mod verification;

const REFERENCE_RELATIVE: &str = "docs/reference/minecraft-java-26.2";
const SYMBOL_CACHE_VERSION: &str = "v1";
const SYMBOL_CACHE_HEADER: &str = "mc-reference-symbol-cache-v1";
const JAVAP_BATCH_SIZE: usize = 64;

#[derive(Debug)]
pub enum Command {
    Fetch { version: String },
    Reports,
    Query { kind: String, id: String },
    Unreviewed { kind: Option<String> },
    Symbols,
    Coverage,
    Readiness,
    Protocol(ProtocolCommand),
    Surface(SurfaceCommand),
    Experiment(ExperimentCommand),
    ImplementationManifest(ImplementationManifestCommand),
    Verify { offline: bool },
}

#[derive(Debug)]
pub enum ProtocolCommand {
    Inventory,
    Coverage,
    Readiness,
    Catalog { write: bool },
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

#[derive(Debug)]
pub enum ImplementationManifestCommand {
    Render,
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
    expected_executable_count: usize,
    executable_paths_sha1: String,
    expected_redirect_count: usize,
    redirect_paths_sha1: String,
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

struct CompiledFamilySelector {
    exact: BTreeSet<String>,
    patterns: GlobSet,
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
        Command::Fetch { version } => artifact::fetch(context, &version),
        Command::Reports => artifact::reports(context),
        Command::Query { kind, id } => catalog::query(context, &kind, &id),
        Command::Unreviewed { kind } => catalog::unreviewed(context, kind.as_deref()),
        Command::Symbols => symbols::symbols(context),
        Command::Coverage => catalog::coverage(context).map(|_| ()),
        Command::Readiness => verification::readiness(context),
        Command::Protocol(command) => protocol::protocol(context, command),
        Command::Surface(command) => surface::surfaces(context, command),
        Command::Experiment(command) => experiments::experiments(context, command),
        Command::ImplementationManifest(command) => implementation_manifest::run(context, command),
        Command::Verify { offline } => verification::verify(context, offline),
    }
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
mod tests;
