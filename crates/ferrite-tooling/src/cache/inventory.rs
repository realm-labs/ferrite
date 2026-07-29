use crate::cache::policy::{CachePolicy, NamespaceKind, NamespacePolicy};
use anyhow::{Context as _, Result, ensure};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const ACTIVE_SENTINEL: &str = ".ferrite-active";
const CARGO_LOCKS: [&str; 3] = [".cargo-build-lock", ".cargo-artifact-lock", ".cargo-lock"];

#[derive(Debug)]
pub(super) struct CacheInventory {
    pub(super) target_bytes: u64,
    pub(super) candidates: Vec<CacheCandidate>,
}

#[derive(Debug)]
pub(super) struct CacheCandidate {
    pub(super) name: String,
    pub(super) path: PathBuf,
    pub(super) bytes: u64,
    pub(super) age: Duration,
    pub(super) exists: bool,
    pub(super) active: bool,
    pub(super) eligible: bool,
    pub(super) reason: String,
}

pub(super) fn inspect(
    workspace: &Path,
    policy: &CachePolicy,
    now: SystemTime,
) -> Result<CacheInventory> {
    let target = workspace.join("target");
    fs::create_dir_all(&target)?;
    let target = target
        .canonicalize()
        .with_context(|| format!("resolve target root {}", target.display()))?;
    let protected = resolved_protected(workspace, policy, &target)?;
    let target_bytes = directory_size(&target, &protected)?;
    let mut candidates = Vec::new();
    for namespace in &policy.namespace {
        candidates.push(inspect_namespace(
            workspace,
            &target,
            &protected,
            policy,
            namespace,
            target_bytes,
            now,
        )?);
    }
    Ok(CacheInventory {
        target_bytes,
        candidates,
    })
}

fn inspect_namespace(
    workspace: &Path,
    target: &Path,
    protected: &[PathBuf],
    policy: &CachePolicy,
    namespace: &NamespacePolicy,
    target_bytes: u64,
    now: SystemTime,
) -> Result<CacheCandidate> {
    let path = resolve_scoped_path(workspace, target, &namespace.path)?;
    ensure!(
        protected
            .iter()
            .all(|protected| !path.starts_with(protected) && !protected.starts_with(&path)),
        "cache namespace {} intersects a protected path",
        namespace.name
    );
    if !path.exists() {
        return Ok(CacheCandidate {
            name: namespace.name.clone(),
            path,
            bytes: 0,
            age: Duration::ZERO,
            exists: false,
            active: false,
            eligible: false,
            reason: "namespace does not exist".to_owned(),
        });
    }
    let path = path
        .canonicalize()
        .with_context(|| format!("resolve cache namespace {}", path.display()))?;
    ensure!(
        path.starts_with(target),
        "resolved cache namespace escapes target: {}",
        path.display()
    );
    let bytes = directory_size(&path, protected)?;
    let modified = latest_modified(&path)?;
    let age = now.duration_since(modified).unwrap_or_default();
    let active = path.join(ACTIVE_SENTINEL).exists() || cargo_lock_is_active(&path)?;
    let minimum_age = match namespace.kind {
        NamespaceKind::Dev => Duration::from_secs(policy.dev_inactive_days * 24 * 60 * 60),
        NamespaceKind::Auxiliary => {
            Duration::from_secs(policy.auxiliary_inactive_days * 24 * 60 * 60)
        }
    };
    let above_water =
        namespace.kind != NamespaceKind::Dev || target_bytes > policy.dev_high_water_bytes;
    let eligible = !active && age >= minimum_age && above_water;
    let reason = if active {
        "active build marker or Cargo lock".to_owned()
    } else if age < minimum_age {
        format!(
            "inactive for {}h; requires {}h",
            age.as_secs() / 3600,
            minimum_age.as_secs() / 3600
        )
    } else if !above_water {
        format!(
            "target is {target_bytes} bytes; dev high-water is {} bytes",
            policy.dev_high_water_bytes
        )
    } else {
        "eligible by age, activity, and high-water policy".to_owned()
    };
    Ok(CacheCandidate {
        name: namespace.name.clone(),
        path,
        bytes,
        age,
        exists: true,
        active,
        eligible,
        reason,
    })
}

pub(super) fn active_sentinel(namespace_root: &Path) -> PathBuf {
    namespace_root.join(ACTIVE_SENTINEL)
}

pub(super) fn resolve_scoped_path(
    workspace: &Path,
    target: &Path,
    relative: &str,
) -> Result<PathBuf> {
    let path = workspace.join(relative);
    let parent = path.parent().context("cache namespace has no parent")?;
    fs::create_dir_all(parent)?;
    let parent = parent
        .canonicalize()
        .with_context(|| format!("resolve cache parent {}", parent.display()))?;
    let file_name = path
        .file_name()
        .context("cache namespace has no file name")?;
    let resolved = parent.join(file_name);
    ensure!(
        resolved.starts_with(target),
        "cache namespace escapes target: {}",
        resolved.display()
    );
    Ok(resolved)
}

fn resolved_protected(
    workspace: &Path,
    policy: &CachePolicy,
    target: &Path,
) -> Result<Vec<PathBuf>> {
    policy
        .protected_paths
        .iter()
        .map(|relative| resolve_scoped_path(workspace, target, relative))
        .collect()
}

fn directory_size(root: &Path, protected: &[PathBuf]) -> Result<u64> {
    if protected.iter().any(|path| root == path) {
        return Ok(0);
    }
    let metadata =
        fs::symlink_metadata(root).with_context(|| format!("inspect {}", root.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut bytes = 0_u64;
    for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry?;
        bytes = bytes
            .checked_add(directory_size(&entry.path(), protected)?)
            .context("cache size overflow")?;
    }
    Ok(bytes)
}

fn latest_modified(root: &Path) -> Result<SystemTime> {
    let metadata =
        fs::symlink_metadata(root).with_context(|| format!("inspect {}", root.display()))?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        return metadata.modified().context("read cache modification time");
    }
    let mut latest = metadata
        .modified()
        .context("read cache modification time")?;
    for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let modified = latest_modified(&entry?.path())?;
        latest = latest.max(modified);
    }
    Ok(latest)
}

fn cargo_lock_is_active(root: &Path) -> Result<bool> {
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(&path)? {
                pending.push(entry?.path());
            }
            continue;
        }
        let name = path.file_name().and_then(|value| value.to_str());
        if !name.is_some_and(|name| CARGO_LOCKS.contains(&name)) {
            continue;
        }
        let file = open_lock(&path)?;
        if file.try_lock().is_err() {
            return Ok(true);
        }
    }
    Ok(false)
}

fn open_lock(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .with_context(|| format!("open Cargo lock {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::policy::{CachePolicy, NamespaceKind, NamespacePolicy};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn protected_reference_cache_is_excluded_from_size() {
        let directory = tempdir().unwrap();
        let workspace = directory.path().canonicalize().unwrap();
        fs::create_dir_all(workspace.join("target/mc-reference/26.2")).unwrap();
        fs::write(
            workspace.join("target/mc-reference/26.2/server.jar"),
            [0_u8; 8],
        )
        .unwrap();
        fs::create_dir_all(workspace.join("target/coverage")).unwrap();
        fs::write(workspace.join("target/coverage/data"), [0_u8; 3]).unwrap();
        let policy = test_policy();

        let inventory = inspect(&workspace, &policy, SystemTime::now()).unwrap();

        assert_eq!(inventory.target_bytes, 3);
    }

    #[test]
    fn namespace_cannot_cover_a_protected_cache() {
        let directory = tempdir().unwrap();
        let workspace = directory.path().canonicalize().unwrap();
        fs::create_dir_all(workspace.join("target/mc-reference/26.2")).unwrap();
        let mut policy = test_policy();
        policy.namespace[0].path = "target/mc-reference".to_owned();

        assert!(inspect(&workspace, &policy, SystemTime::now()).is_err());
    }

    #[test]
    fn active_namespace_is_never_eligible() {
        let directory = tempdir().unwrap();
        let workspace = directory.path().canonicalize().unwrap();
        fs::create_dir_all(workspace.join("target/coverage")).unwrap();
        fs::write(workspace.join("target/coverage/.ferrite-active"), b"active").unwrap();
        let policy = test_policy();

        let inventory = inspect(&workspace, &policy, SystemTime::now()).unwrap();

        assert!(inventory.candidates[0].active);
        assert!(!inventory.candidates[0].eligible);
    }

    fn test_policy() -> CachePolicy {
        CachePolicy {
            schema_version: 1,
            workspace_id: "ferrite".to_owned(),
            check_interval_hours: 24,
            auxiliary_inactive_days: 0,
            dev_inactive_days: 0,
            dev_high_water_bytes: 1,
            protected_paths: vec!["target/mc-reference/26.2".to_owned()],
            namespace: vec![NamespacePolicy {
                name: "coverage".to_owned(),
                path: "target/coverage".to_owned(),
                kind: NamespaceKind::Auxiliary,
            }],
        }
    }
}
