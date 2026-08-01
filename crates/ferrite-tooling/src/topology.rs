use anyhow::{Context as _, Result, ensure};
use std::path::Path;
use std::process::Command;

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    run_cluster(
        workspace,
        "verify-topology",
        "three-process topology conformance",
    )?;
    run_cluster(workspace, "verify-faults", "multi-node fault injection")
}

fn run_cluster(workspace: &Path, command: &str, label: &str) -> Result<()> {
    let mut arguments = vec!["run", "-q", "-p", "ferrite-cluster", "--", command];
    if command == "verify-topology" {
        arguments.extend(["--ticks", "10000"]);
    }
    let status = Command::new("cargo")
        .args(arguments)
        .current_dir(workspace)
        .status()
        .with_context(|| format!("run {label}"))?;
    ensure!(status.success(), "{label} failed with {status}");
    Ok(())
}
