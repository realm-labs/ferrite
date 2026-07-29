use anyhow::{Context as _, Result, bail, ensure};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    workspace_members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Package {
    id: String,
    name: String,
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
struct Dependency {
    name: String,
    kind: Option<String>,
}

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    verify_profiles(workspace)?;
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(workspace)
        .output()
        .context("run cargo metadata")?;
    ensure!(
        output.status.success(),
        "cargo metadata failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Metadata =
        serde_json::from_slice(&output.stdout).context("parse cargo metadata")?;
    let edge_count = validate(&metadata)?;
    println!(
        "workspace dependency direction verified: {} packages, {edge_count} workspace edges",
        metadata.workspace_members.len()
    );
    Ok(())
}

fn verify_profiles(workspace: &Path) -> Result<()> {
    let path = workspace.join("Cargo.toml");
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let manifest: toml::Value =
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    let profile = manifest
        .get("profile")
        .and_then(toml::Value::as_table)
        .context("workspace manifest has no profile table")?;
    let dev = profile
        .get("dev")
        .and_then(toml::Value::as_table)
        .context("workspace manifest has no dev profile")?;
    ensure!(
        dev.get("debug").and_then(toml::Value::as_str) == Some("line-tables-only"),
        "dev profile must use line-tables-only debug information"
    );
    let dependency_debug = dev
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|packages| packages.get("*"))
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("debug"))
        .and_then(toml::Value::as_bool);
    ensure!(
        dependency_debug == Some(false),
        "dev dependency debug information must be disabled"
    );
    let debugging = profile
        .get("debugging")
        .and_then(toml::Value::as_table)
        .context("workspace manifest has no debugging profile")?;
    ensure!(
        debugging.get("inherits").and_then(toml::Value::as_str) == Some("dev")
            && debugging.get("debug").and_then(toml::Value::as_bool) == Some(true),
        "debugging profile must inherit dev and enable full debug information"
    );
    Ok(())
}

fn validate(metadata: &Metadata) -> Result<usize> {
    let expected = policy();
    let workspace_ids = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let packages = metadata
        .packages
        .iter()
        .filter(|package| workspace_ids.contains(package.id.as_str()))
        .collect::<Vec<_>>();
    let names = packages
        .iter()
        .map(|package| package.name.as_str())
        .collect::<BTreeSet<_>>();
    let expected_names = expected.keys().copied().collect::<BTreeSet<_>>();
    ensure!(
        names == expected_names,
        "workspace package policy is stale; missing policy {:?}, dead policy {:?}",
        names.difference(&expected_names).collect::<Vec<_>>(),
        expected_names.difference(&names).collect::<Vec<_>>()
    );

    let mut graph = BTreeMap::<String, Vec<String>>::new();
    let mut edge_count = 0;
    for package in packages {
        let allowed = &expected[package.name.as_str()];
        let mut edges = Vec::new();
        for dependency in &package.dependencies {
            if !names.contains(dependency.name.as_str()) {
                if dependency.name.starts_with("lattice-")
                    && package.name != "ferrite-region-runtime"
                {
                    bail!(
                        "{} leaks Lattice dependency {} outside ferrite-region-runtime",
                        package.name,
                        dependency.name
                    );
                }
                continue;
            }
            ensure!(
                dependency.name != "mc-reference",
                "{} depends on development-only mc-reference",
                package.name
            );
            let development = dependency.kind.as_deref() == Some("dev");
            let allowed_testkit = development && dependency.name == "ferrite-testkit";
            ensure!(
                allowed.contains(dependency.name.as_str()) || allowed_testkit,
                "{} has forbidden {}dependency on {}",
                package.name,
                if development { "development " } else { "" },
                dependency.name
            );
            edges.push(dependency.name.clone());
            edge_count += 1;
        }
        edges.sort();
        edges.dedup();
        graph.insert(package.name.clone(), edges);
    }
    ensure!(
        !has_cycle(&graph),
        "workspace dependency graph contains a cycle"
    );
    Ok(edge_count)
}

fn policy() -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    BTreeMap::from([
        (
            "behavior-runner",
            set(&["ferrite-gameplay", "ferrite-replay", "ferrite-testkit"]),
        ),
        (
            "ferrite-cluster",
            set(&["ferrite-region-runtime", "ferrite-server-runtime"]),
        ),
        ("ferrite-foundation", set(&[])),
        (
            "ferrite-gameplay",
            set(&[
                "ferrite-foundation",
                "ferrite-registry",
                "ferrite-simulation",
                "ferrite-world",
            ]),
        ),
        (
            "ferrite-persistence",
            set(&["ferrite-foundation", "ferrite-registry", "ferrite-world"]),
        ),
        (
            "ferrite-protocol",
            set(&["ferrite-foundation", "ferrite-registry"]),
        ),
        (
            "ferrite-region-runtime",
            set(&[
                "ferrite-foundation",
                "ferrite-persistence",
                "ferrite-simulation",
                "ferrite-world",
            ]),
        ),
        ("ferrite-registry", set(&["ferrite-foundation"])),
        (
            "ferrite-replay",
            set(&[
                "ferrite-foundation",
                "ferrite-gameplay",
                "ferrite-simulation",
                "ferrite-world",
            ]),
        ),
        ("ferrite-server", set(&["ferrite-server-runtime"])),
        (
            "ferrite-server-runtime",
            set(&[
                "ferrite-foundation",
                "ferrite-gameplay",
                "ferrite-persistence",
                "ferrite-protocol",
                "ferrite-region-runtime",
                "ferrite-registry",
                "ferrite-replay",
                "ferrite-simulation",
                "ferrite-world",
            ]),
        ),
        (
            "ferrite-simulation",
            set(&["ferrite-foundation", "ferrite-world"]),
        ),
        (
            "ferrite-testkit",
            set(&[
                "ferrite-foundation",
                "ferrite-gameplay",
                "ferrite-persistence",
                "ferrite-protocol",
                "ferrite-region-runtime",
                "ferrite-registry",
                "ferrite-replay",
                "ferrite-server-runtime",
                "ferrite-simulation",
                "ferrite-world",
            ]),
        ),
        (
            "ferrite-tooling",
            set(&["ferrite-foundation", "ferrite-registry"]),
        ),
        (
            "ferrite-world",
            set(&["ferrite-foundation", "ferrite-registry"]),
        ),
        ("mc-reference", set(&[])),
        (
            "protocol-conformance",
            set(&["ferrite-protocol", "ferrite-testkit"]),
        ),
        (
            "world-inspector",
            set(&["ferrite-persistence", "ferrite-world"]),
        ),
    ])
}

fn set(values: &[&'static str]) -> BTreeSet<&'static str> {
    values.iter().copied().collect()
}

fn has_cycle(graph: &BTreeMap<String, Vec<String>>) -> bool {
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
                if visit(dependency, graph, visiting, visited) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_dependency_cycles() {
        let graph = BTreeMap::from([
            ("a".to_owned(), vec!["b".to_owned()]),
            ("b".to_owned(), vec!["a".to_owned()]),
        ]);
        assert!(has_cycle(&graph));
    }

    #[test]
    fn allowed_policy_is_acyclic() {
        let graph = policy()
            .into_iter()
            .map(|(name, dependencies)| {
                (
                    name.to_owned(),
                    dependencies.into_iter().map(str::to_owned).collect(),
                )
            })
            .collect();
        assert!(!has_cycle(&graph));
    }

    #[test]
    fn profile_verifier_rejects_missing_full_symbol_profile() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("Cargo.toml"),
            r#"
            [profile.dev]
            debug = "line-tables-only"

            [profile.dev.package."*"]
            debug = false
            "#,
        )
        .unwrap();

        assert!(verify_profiles(directory.path()).is_err());
    }
}
