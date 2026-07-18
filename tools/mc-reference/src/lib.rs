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
    Experiment(ExperimentCommand),
    Verify { offline: bool },
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
    let entry = manifest
        .versions
        .iter()
        .find(|entry| entry.id == version)
        .with_context(|| format!("{version} is absent from the official manifest"))?;
    ensure!(
        entry.url == context.lock.metadata.url,
        "metadata URL differs from lock"
    );
    ensure!(
        entry.sha1 == context.lock.metadata.sha1,
        "metadata SHA-1 differs from lock"
    );
    write_verified(
        &context.cache.join("version_manifest_v2.json"),
        &manifest_bytes,
        None,
        None,
    )?;

    let metadata_bytes = get(&client, &entry.url)?;
    write_verified(
        &context.cache.join("version.json"),
        &metadata_bytes,
        Some(&entry.sha1),
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
        "entity_type" | "mob_effect" | "menu" | "recipe_serializer" | "potion" => {
            let value = read_json(&reports.join("registries.json"))?;
            Ok(value
                .pointer(&format!("/minecraft:{kind}/entries/{}", escape_pointer(id)))
                .cloned()
                .unwrap_or(Value::Null))
        }
        "recipe" | "loot_table" | "advancement" | "damage_type" | "enchantment"
        | "dimension_type" | "worldgen" => read_server_data_json(context, kind, id),
        _ => bail!("query output is not implemented for {kind}"),
    }
}

fn read_server_data_json(context: &Context, kind: &str, id: &str) -> Result<Value> {
    let server = extract_server(context)?;
    let prefix = if kind == "worldgen" {
        "data/minecraft/worldgen"
    } else {
        // These are the singular directory names used by Data Pack 107.1.
        match kind {
            "recipe" => "data/minecraft/recipe",
            "loot_table" => "data/minecraft/loot_table",
            "advancement" => "data/minecraft/advancement",
            "damage_type" => "data/minecraft/damage_type",
            "enchantment" => "data/minecraft/enchantment",
            "dimension_type" => "data/minecraft/dimension_type",
            _ => bail!("no data path for {kind}"),
        }
    };
    let path = format!("{prefix}/{}.json", strip_namespace(id));
    let mut archive = ZipArchive::new(File::open(server)?)?;
    let mut entry = archive
        .by_name(&path)
        .with_context(|| format!("locked data has no {path}"))?;
    Ok(serde_json::from_reader(&mut entry)?)
}

fn query_tags(context: &Context, kind: &str, id: &str) -> Result<Vec<String>> {
    let tag_kind = match kind {
        "block" | "item" | "entity_type" | "mob_effect" | "damage_type" | "enchantment" => kind,
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
            }
        }
        total += ids.len();
        println!("{:<18} {:>5} IDs  {}", category.kind, ids.len(), digest);
    }
    println!(
        "coverage complete: {total} locked IDs, zero unclassified or ambiguous; {unreviewed} explicitly unreviewed"
    );
    Ok(total)
}

fn validate_family_selectors(
    category: &Category,
    ids: &BTreeSet<String>,
    blocks: &BTreeSet<String>,
) -> Result<()> {
    for family in &category.family {
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
    match kind {
        "block" => Ok(read_json(&reports.join("blocks.json"))?
            .as_object()
            .context("blocks.json is not an object")?
            .keys()
            .cloned()
            .collect()),
        "item" => ids_from_files(&reports.join("minecraft/components/item"), "json"),
        "entity_type" | "mob_effect" | "menu" | "recipe_serializer" | "potion" => {
            let value = read_json(&reports.join("registries.json"))?;
            Ok(value
                .pointer(&format!("/minecraft:{kind}/entries"))
                .and_then(Value::as_object)
                .with_context(|| format!("registry {kind} missing"))?
                .keys()
                .cloned()
                .collect())
        }
        "recipe" | "loot_table" | "advancement" | "damage_type" | "enchantment"
        | "dimension_type" | "worldgen" => ids_from_server_data(&server, kind),
        _ => bail!("unknown query kind {kind}"),
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
    let prefix = if kind == "worldgen" {
        "data/minecraft/worldgen/".to_string()
    } else {
        format!("data/minecraft/{kind}/")
    };
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
        let entry = manifest
            .versions
            .iter()
            .find(|entry| entry.id == context.lock.version)
            .context("locked version absent")?;
        ensure!(
            entry.sha1 == context.lock.metadata.sha1 && entry.url == context.lock.metadata.url,
            "official manifest no longer agrees with lock"
        );
        println!("official manifest lock verified");
    }
    verify_cached_artifacts(context)?;
    verify_reports(context)?;
    validate_docs(context)?;
    symbols(context)?;
    coverage(context)?;
    experiments(context, ExperimentCommand::Verify)?;
    hygiene(context)?;
    println!(
        "mc-reference verification complete ({})",
        if offline { "offline" } else { "online" }
    );
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
    fn matches_jvm_descriptors_instead_of_generic_declarations() {
        let javap = "  public void tick(net.minecraft.server.level.ServerLevel, E);\n    descriptor: (Lnet/minecraft/server/level/ServerLevel;Lnet/minecraft/world/entity/LivingEntity;)V\n";
        assert!(descriptor_matches(
            javap,
            "tick",
            "(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity)"
        ));
    }
}
