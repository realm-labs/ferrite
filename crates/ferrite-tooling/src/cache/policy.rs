use anyhow::{Context as _, Result, ensure};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

const POLICY_RELATIVE: &str = ".ferrite/cache-policy.toml";

#[derive(Debug, Clone, Deserialize)]
pub(super) struct CachePolicy {
    pub(super) schema_version: u32,
    pub(super) workspace_id: String,
    pub(super) check_interval_hours: u64,
    pub(super) auxiliary_inactive_days: u64,
    pub(super) dev_inactive_days: u64,
    pub(super) dev_high_water_bytes: u64,
    pub(super) protected_paths: Vec<String>,
    pub(super) namespace: Vec<NamespacePolicy>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct NamespacePolicy {
    pub(super) name: String,
    pub(super) path: String,
    pub(super) kind: NamespaceKind,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub(super) enum NamespaceKind {
    Dev,
    Auxiliary,
}

pub(super) fn load(workspace: &Path) -> Result<CachePolicy> {
    let path = workspace.join(POLICY_RELATIVE);
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let policy: CachePolicy =
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    validate(&policy)?;
    Ok(policy)
}

fn validate(policy: &CachePolicy) -> Result<()> {
    ensure!(
        policy.schema_version == 1,
        "unsupported cache policy schema"
    );
    ensure!(
        policy.workspace_id == "ferrite",
        "cache policy workspace marker is not ferrite"
    );
    ensure!(
        policy.check_interval_hours > 0,
        "cache check interval must be positive"
    );
    ensure!(
        policy.dev_high_water_bytes > 0,
        "dev high-water mark must be positive"
    );
    ensure!(
        !policy.namespace.is_empty(),
        "cache policy has no namespaces"
    );
    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut dev_count = 0;
    for namespace in &policy.namespace {
        ensure!(
            !namespace.name.trim().is_empty() && names.insert(&namespace.name),
            "cache namespace name is empty or duplicated"
        );
        validate_target_path(&namespace.path)?;
        ensure!(
            paths.insert(&namespace.path),
            "cache namespace path {} is duplicated",
            namespace.path
        );
        if namespace.kind == NamespaceKind::Dev {
            dev_count += 1;
        }
    }
    ensure!(
        dev_count == 1,
        "cache policy requires exactly one Dev namespace"
    );
    for protected in &policy.protected_paths {
        validate_target_path(protected)?;
    }
    Ok(())
}

pub(super) fn validate_target_path(path: &str) -> Result<()> {
    let candidate = Path::new(path);
    ensure!(!candidate.is_absolute(), "cache path is absolute: {path}");
    ensure!(
        candidate
            .components()
            .next()
            .is_some_and(|component| component.as_os_str() == "target"),
        "cache path is outside target/: {path}"
    );
    ensure!(
        !candidate.components().any(|component| matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )),
        "cache path escapes target/: {path}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_paths_outside_the_workspace_target() {
        assert!(validate_target_path("target/debugging").is_ok());
        assert!(validate_target_path("../target").is_err());
        assert!(validate_target_path("other/cache").is_err());
    }
}
