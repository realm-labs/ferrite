use crate::{architecture, content, source_policy};
use anyhow::{Context as _, Result, bail, ensure};
use serde::Deserialize;
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use toml::{Table, Value};

const IMPLEMENTATION_MANIFEST: &str = "goals/minecraft-java-26.2/implementation.toml";
const FORBIDDEN_ARTIFACT_EXTENSIONS: &[&str] = &["class", "jar", "mca", "mcr", "part"];
const GRADLE_WRAPPER_RELATIVE: &str = "tools/ferrite-client-mcp/gradle/wrapper/gradle-wrapper.jar";
const GRADLE_WRAPPER_SHA256: &str =
    "497c8c2a7e5031f6aa847f88104aa80a93532ec32ee17bdb8d1d2f67a194a9c7";

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    architecture::verify(workspace)?;
    source_policy::verify(workspace)?;
    verify_public_api(workspace)?;
    verify_generated_artifacts(workspace)?;
    run_manifest_verifier(workspace)?;
    verify_complete_manifest(workspace)?;
    content::verify(workspace)?;
    run_rustdoc(workspace)?;
    println!("Ferrite Goal 01 architecture and content audits passed");
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    workspace_members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Package {
    id: String,
    targets: Vec<Target>,
}

#[derive(Debug, Deserialize)]
struct Target {
    kind: Vec<String>,
    src_path: PathBuf,
}

fn verify_public_api(workspace: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(workspace)
        .output()
        .context("run cargo metadata for public API audit")?;
    ensure!(
        output.status.success(),
        "cargo metadata for public API audit failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Metadata =
        serde_json::from_slice(&output.stdout).context("parse public API cargo metadata")?;
    let members = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut roots = BTreeSet::new();
    for package in metadata
        .packages
        .iter()
        .filter(|package| members.contains(package.id.as_str()))
    {
        for target in &package.targets {
            if target
                .kind
                .iter()
                .any(|kind| kind == "lib" || kind == "bin")
            {
                roots.insert(target.src_path.clone());
            }
        }
    }
    ensure!(!roots.is_empty(), "public API audit found no crate roots");
    for root in &roots {
        let source = fs::read_to_string(root)
            .with_context(|| format!("read crate root {}", root.display()))?;
        ensure!(
            source
                .lines()
                .take(5)
                .any(|line| line.trim() == "#![forbid(unsafe_code)]"),
            "crate root {} does not forbid unsafe code",
            root.display()
        );
    }
    println!(
        "public API source boundary verified: {} library and binary crate roots forbid unsafe code",
        roots.len()
    );
    Ok(())
}

fn verify_generated_artifacts(workspace: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(workspace)
        .output()
        .context("list tracked files for generated-artifact audit")?;
    ensure!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut tracked = 0_usize;
    for raw in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
    {
        let relative = std::str::from_utf8(raw).context("tracked path is not UTF-8")?;
        let path = Path::new(relative);
        tracked += 1;
        ensure!(
            !path.components().any(|component| {
                matches!(component, Component::Normal(name) if name == OsStr::new("target") || name == OsStr::new("generated"))
            }),
            "tracked generated directory entry: {relative}"
        );
        let extension = path.extension().and_then(OsStr::to_str).unwrap_or("");
        ensure!(
            !FORBIDDEN_ARTIFACT_EXTENSIONS.contains(&extension)
                || verified_gradle_wrapper(workspace, relative)?,
            "tracked generated artifact: {relative}"
        );
        ensure!(
            path.file_name() != Some(OsStr::new("content-bundle.json")),
            "generated content bundle is tracked: {relative}"
        );
    }

    let build_script = workspace.join("crates/ferrite-protocol/build.rs");
    let build_source = fs::read_to_string(&build_script)
        .with_context(|| format!("read {}", build_script.display()))?;
    ensure!(
        build_source.contains("OUT_DIR")
            && build_source.contains("minecraft_java_26_2_packet_catalog.rs")
            && build_source.contains("minecraft_java_26_2_entity_metadata_accessors.rs"),
        "protocol generation is not contained in Cargo OUT_DIR"
    );
    for relative in [
        "crates/ferrite-protocol/reference/minecraft-java-26.2-packets.toml",
        "crates/ferrite-protocol/reference/minecraft-java-26.2-entity-metadata-accessors.tsv",
        "docs/reference/minecraft-java-26.2/content-bundle.lock.toml",
    ] {
        ensure!(
            workspace.join(relative).is_file(),
            "reviewed generated-input lock is missing: {relative}"
        );
    }
    println!(
        "generated-artifact boundary verified: {tracked} tracked paths, generated Rust confined to OUT_DIR"
    );
    Ok(())
}

fn verified_gradle_wrapper(workspace: &Path, relative: &str) -> Result<bool> {
    if relative != GRADLE_WRAPPER_RELATIVE {
        return Ok(false);
    }
    let bytes = fs::read(workspace.join(relative))
        .with_context(|| format!("read reviewed Gradle wrapper {relative}"))?;
    let digest = hex::encode(Sha256::digest(bytes));
    ensure!(
        digest == GRADLE_WRAPPER_SHA256,
        "Gradle wrapper digest drifted: expected {GRADLE_WRAPPER_SHA256}, found {digest}"
    );
    Ok(true)
}

fn run_manifest_verifier(workspace: &Path) -> Result<()> {
    run_cargo(
        workspace,
        "implementation manifest verification",
        &[
            "run",
            "-q",
            "-p",
            "mc-reference",
            "--bin",
            "mc-ref",
            "--",
            "implementation-manifest",
            "verify",
        ],
        None,
    )
}

fn verify_complete_manifest(workspace: &Path) -> Result<()> {
    let path = workspace.join(IMPLEMENTATION_MANIFEST);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let root: Value = toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;

    let catalog = verified_weight(&root, "catalog_batch", Weight::Integer("reference_ids"))?;
    let gameplay = verified_weight(&root, "gameplay_batch", Weight::Array("slices"))?;
    let surfaces = verified_weight(&root, "surface_owner", Weight::Unit)?;
    let joins = verified_weight(&root, "join_owner", Weight::Unit)?;
    let protocol = verified_weight(&root, "protocol_batch", Weight::Integer("packets"))?;
    let worldgen_exactness = verified_weight(&root, "worldgen_exactness", Weight::Unit)?;
    let protocol_families = records(&root, "protocol_batch")?.len();
    let optional_families = records(&root, "protocol_batch")?
        .into_iter()
        .filter(|record| {
            record.get("implementation_mode").and_then(Value::as_str) == Some("ConfigurationGate")
        })
        .count();
    let deferred = records(&root, "deferred_observation")?;
    ensure!(
        deferred.iter().all(|record| {
            record.get("disposition").and_then(Value::as_str) == Some("DeferredExperiment")
        }),
        "a deferred observation lost its DeferredExperiment disposition"
    );

    ensure!(
        catalog == 9_078,
        "verified catalog coverage is {catalog}; expected 9078"
    );
    ensure!(
        gameplay == 331,
        "verified gameplay coverage is {gameplay}; expected 331"
    );
    ensure!(
        surfaces == 10,
        "verified surface coverage is {surfaces}; expected 10"
    );
    ensure!(
        joins == 36,
        "verified join coverage is {joins}; expected 36"
    );
    ensure!(
        protocol == 256,
        "verified protocol packet coverage is {protocol}; expected 256"
    );
    ensure!(
        worldgen_exactness == 1,
        "verified worldgen exactness coverage is {worldgen_exactness}; expected 1"
    );
    ensure!(
        protocol_families == 58,
        "protocol family coverage is {protocol_families}; expected 58"
    );
    ensure!(
        optional_families == 14,
        "optional protocol family coverage is {optional_families}; expected 14"
    );
    ensure!(
        deferred.len() == 4,
        "deferred observation count is {}; expected 4",
        deferred.len()
    );
    println!(
        "terminal manifest dispositions verified: 9078 catalog IDs, 331 slices, 58 protocol families, 10 surfaces, 36 joins, 4 deferred observations, and worldgen exactness"
    );
    Ok(())
}

enum Weight {
    Integer(&'static str),
    Array(&'static str),
    Unit,
}

fn verified_weight(root: &Value, array: &str, weight: Weight) -> Result<usize> {
    let mut total = 0_usize;
    for record in records(root, array)? {
        let disposition = record
            .get("disposition")
            .and_then(Value::as_str)
            .with_context(|| format!("{array} record has no disposition"))?;
        ensure!(
            disposition == "Verified",
            "{array} contains non-Verified disposition {disposition}"
        );
        total += match weight {
            Weight::Integer(field) => usize::try_from(
                record
                    .get(field)
                    .and_then(Value::as_integer)
                    .with_context(|| format!("{array} record has no integer {field}"))?,
            )
            .with_context(|| format!("{array} record has invalid {field}"))?,
            Weight::Array(field) => record
                .get(field)
                .and_then(Value::as_array)
                .with_context(|| format!("{array} record has no {field} array"))?
                .len(),
            Weight::Unit => 1,
        };
    }
    Ok(total)
}

fn records<'a>(root: &'a Value, array: &str) -> Result<Vec<&'a Table>> {
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

fn run_rustdoc(workspace: &Path) -> Result<()> {
    let flags = match std::env::var("RUSTDOCFLAGS") {
        Ok(existing) if !existing.trim().is_empty() => format!("{existing} -D warnings"),
        _ => "-D warnings".to_owned(),
    };
    run_cargo(
        workspace,
        "public API rustdoc",
        &["doc", "--workspace", "--all-features", "--no-deps"],
        Some(("RUSTDOCFLAGS", flags.as_str())),
    )
}

fn run_cargo(
    workspace: &Path,
    label: &str,
    arguments: &[&str],
    environment: Option<(&str, &str)>,
) -> Result<()> {
    println!("running {label}: cargo {}", arguments.join(" "));
    let mut command = Command::new("cargo");
    command.args(arguments).current_dir(workspace);
    if let Some((name, value)) = environment {
        command.env(name, value);
    }
    let status = command
        .status()
        .with_context(|| format!("run Cargo {label}"))?;
    if !status.success() {
        bail!("{label} failed with {status}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verified_weights_reject_incomplete_records() {
        let complete: Value = toml::from_str(
            r#"
            [[entry]]
            disposition = "Verified"
            values = ["a", "b"]
            "#,
        )
        .unwrap();
        assert_eq!(
            verified_weight(&complete, "entry", Weight::Array("values")).unwrap(),
            2
        );

        let pending: Value = toml::from_str(
            r#"
            [[entry]]
            disposition = "Pending"
            "#,
        )
        .unwrap();
        assert!(verified_weight(&pending, "entry", Weight::Unit).is_err());
    }
}
