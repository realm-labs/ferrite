use anyhow::{Context as _, Result, ensure};
use std::path::Path;
use std::process::Command;

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "ferrite-cluster",
            "--",
            "verify-topology",
            "--ticks",
            "10000",
        ])
        .current_dir(workspace)
        .status()
        .context("run three-process topology conformance")?;
    ensure!(
        status.success(),
        "three-process topology conformance failed with {status}"
    );
    Ok(())
}
