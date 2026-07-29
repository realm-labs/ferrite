mod inventory;
mod policy;

use anyhow::{Context as _, Result, bail, ensure};
use inventory::{CacheInventory, active_sentinel, inspect, resolve_scoped_path};
use policy::{CachePolicy, NamespaceKind};
use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAINTENANCE_LOCK: &str = "target/ferrite-tooling/cache-maintenance-v1.lock";
const MAINTENANCE_STATE: &str = "target/ferrite-tooling/last-maintenance-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ApplyMode {
    DryRun,
    Apply,
}

pub(crate) fn inspect_command(workspace: &Path) -> Result<()> {
    let policy = policy::load(workspace)?;
    let inventory = inspect(workspace, &policy, SystemTime::now())?;
    print_inventory(&inventory, ApplyMode::DryRun);
    Ok(())
}

pub(crate) fn prune(workspace: &Path, mode: ApplyMode) -> Result<()> {
    let policy = policy::load(workspace)?;
    let _lock = MaintenanceLock::acquire(workspace)?;
    let inventory = inspect(workspace, &policy, SystemTime::now())?;
    apply_inventory(workspace, &inventory, mode)
}

pub(crate) fn maintain(workspace: &Path, mode: ApplyMode) -> Result<()> {
    let policy = policy::load(workspace)?;
    let _lock = MaintenanceLock::acquire(workspace)?;
    let now = SystemTime::now();
    if !maintenance_due(workspace, &policy, now)? {
        println!(
            "cache maintenance skipped: checked within the last {} hours",
            policy.check_interval_hours
        );
        return Ok(());
    }
    let inventory = inspect(workspace, &policy, now)?;
    apply_inventory(workspace, &inventory, mode)?;
    if mode == ApplyMode::Apply {
        write_maintenance_state(workspace, now)?;
    }
    Ok(())
}

pub(crate) fn run_isolated_cargo(
    workspace: &Path,
    namespace: &str,
    arguments: &[String],
) -> Result<()> {
    ensure!(
        !arguments.is_empty(),
        "isolated cargo requires Cargo arguments"
    );
    let policy = policy::load(workspace)?;
    let configured = policy
        .namespace
        .iter()
        .find(|candidate| candidate.name == namespace)
        .with_context(|| format!("unknown cache namespace {namespace}"))?;
    ensure!(
        configured.kind == NamespaceKind::Auxiliary,
        "namespace {namespace} is not an isolated auxiliary target"
    );
    let target = workspace.join("target");
    fs::create_dir_all(&target)?;
    let target = target.canonicalize()?;
    let namespace_root = resolve_scoped_path(workspace, &target, &configured.path)?;
    fs::create_dir_all(&namespace_root)?;
    let _active = ActiveBuild::acquire(&namespace_root)?;
    println!(
        "running isolated Cargo namespace {namespace}: CARGO_TARGET_DIR={}",
        namespace_root.display()
    );
    let status = Command::new("cargo")
        .args(arguments)
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", &namespace_root)
        .status()
        .context("run isolated Cargo command")?;
    ensure!(
        status.success(),
        "isolated Cargo command failed with {status}"
    );
    Ok(())
}

fn apply_inventory(workspace: &Path, inventory: &CacheInventory, mode: ApplyMode) -> Result<()> {
    print_inventory(inventory, mode);
    if mode == ApplyMode::DryRun {
        return Ok(());
    }
    let target = workspace.join("target").canonicalize()?;
    for candidate in &inventory.candidates {
        if !candidate.eligible {
            continue;
        }
        let resolved = candidate
            .path
            .canonicalize()
            .with_context(|| format!("resolve prune target {}", candidate.path.display()))?;
        ensure!(
            resolved.starts_with(&target) && resolved != target,
            "refusing unscoped prune target {}",
            resolved.display()
        );
        fs::remove_dir_all(&resolved)
            .with_context(|| format!("prune cache namespace {}", resolved.display()))?;
        println!(
            "pruned cache namespace {}: {} bytes from {}",
            candidate.name,
            candidate.bytes,
            resolved.display()
        );
    }
    Ok(())
}

fn print_inventory(inventory: &CacheInventory, mode: ApplyMode) {
    println!(
        "workspace target footprint: {} bytes; mode: {}",
        inventory.target_bytes,
        if mode == ApplyMode::Apply {
            "apply"
        } else {
            "dry-run"
        }
    );
    for candidate in &inventory.candidates {
        println!(
            "{}: path={} exists={} active={} eligible={} bytes={} age_hours={} reason={}",
            candidate.name,
            candidate.path.display(),
            candidate.exists,
            candidate.active,
            candidate.eligible,
            candidate.bytes,
            candidate.age.as_secs() / 3600,
            candidate.reason
        );
    }
}

fn maintenance_due(workspace: &Path, policy: &CachePolicy, now: SystemTime) -> Result<bool> {
    let path = workspace.join(MAINTENANCE_STATE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    };
    let last_seconds = text
        .trim()
        .parse::<u64>()
        .with_context(|| format!("parse {}", path.display()))?;
    let last = UNIX_EPOCH + Duration::from_secs(last_seconds);
    let elapsed = now.duration_since(last).unwrap_or_default();
    Ok(elapsed >= Duration::from_secs(policy.check_interval_hours * 3600))
}

fn write_maintenance_state(workspace: &Path, now: SystemTime) -> Result<()> {
    let path = workspace.join(MAINTENANCE_STATE);
    let parent = path.parent().context("maintenance state has no parent")?;
    fs::create_dir_all(parent)?;
    let seconds = now
        .duration_since(UNIX_EPOCH)
        .context("system clock predates Unix epoch")?
        .as_secs();
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(file, "{seconds}")?;
    file.sync_all()?;
    Ok(())
}

struct MaintenanceLock {
    _file: File,
}

impl MaintenanceLock {
    fn acquire(workspace: &Path) -> Result<Self> {
        let path = workspace.join(MAINTENANCE_LOCK);
        let parent = path.parent().context("maintenance lock has no parent")?;
        fs::create_dir_all(parent)?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("open {}", path.display()))?;
        file.try_lock()
            .with_context(|| format!("cache maintenance is already active: {}", path.display()))?;
        Ok(Self { _file: file })
    }
}

struct ActiveBuild {
    path: PathBuf,
    _file: File,
}

impl ActiveBuild {
    fn acquire(namespace_root: &Path) -> Result<Self> {
        let path = active_sentinel(namespace_root);
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .with_context(|| {
                format!(
                    "isolated target namespace is already active: {}",
                    namespace_root.display()
                )
            })?;
        Ok(Self { path, _file: file })
    }
}

impl Drop for ActiveBuild {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.path) {
            eprintln!(
                "failed to remove active-build marker {}: {error}",
                self.path.display()
            );
        }
    }
}

pub(crate) fn parse_apply_mode(arguments: &[String]) -> Result<ApplyMode> {
    match arguments {
        [] => Ok(ApplyMode::DryRun),
        [flag] if flag == "--apply" => Ok(ApplyMode::Apply),
        _ => bail!("expected no option for dry-run or exactly --apply"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn dry_run_preserves_and_apply_removes_only_eligible_candidates() {
        let directory = tempdir().unwrap();
        let workspace = directory.path().canonicalize().unwrap();
        fs::create_dir_all(workspace.join("target/coverage")).unwrap();
        fs::write(workspace.join("target/coverage/data"), [0_u8; 4]).unwrap();
        let inventory = CacheInventory {
            target_bytes: 4,
            candidates: vec![inventory::CacheCandidate {
                name: "coverage".to_owned(),
                path: workspace.join("target/coverage"),
                bytes: 4,
                age: Duration::from_secs(8 * 24 * 3600),
                exists: true,
                active: false,
                eligible: true,
                reason: "test".to_owned(),
            }],
        };

        apply_inventory(&workspace, &inventory, ApplyMode::DryRun).unwrap();
        assert!(workspace.join("target/coverage").exists());
        apply_inventory(&workspace, &inventory, ApplyMode::Apply).unwrap();
        assert!(!workspace.join("target/coverage").exists());
    }

    #[test]
    fn recent_maintenance_is_rate_limited() {
        let directory = tempdir().unwrap();
        let workspace = directory.path().canonicalize().unwrap();
        let policy = CachePolicy {
            schema_version: 1,
            workspace_id: "ferrite".to_owned(),
            check_interval_hours: 24,
            auxiliary_inactive_days: 7,
            dev_inactive_days: 14,
            dev_high_water_bytes: 1,
            protected_paths: Vec::new(),
            namespace: Vec::new(),
        };
        let now = UNIX_EPOCH + Duration::from_secs(1_000_000);
        write_maintenance_state(&workspace, now).unwrap();

        assert!(!maintenance_due(&workspace, &policy, now).unwrap());
        assert!(
            maintenance_due(&workspace, &policy, now + Duration::from_secs(24 * 3600)).unwrap()
        );
    }
}
