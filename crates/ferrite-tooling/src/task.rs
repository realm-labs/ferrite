use crate::{architecture, cache};
use anyhow::{Context as _, Result, ensure};
use std::path::Path;
use std::process::Command;

pub(crate) fn check(workspace: &Path) -> Result<()> {
    cache::maintain(workspace, cache::ApplyMode::Apply)?;
    architecture::verify(workspace)?;
    run(workspace, "format", &["fmt", "--all", "--", "--check"])?;
    run(
        workspace,
        "Clippy",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run(
        workspace,
        "workspace tests",
        &["test", "--workspace", "--all-features"],
    )?;
    run(
        workspace,
        "offline reference and implementation coverage",
        &[
            "run",
            "-q",
            "-p",
            "mc-reference",
            "--bin",
            "mc-ref",
            "--",
            "verify",
            "--offline",
        ],
    )?;
    println!("Ferrite repository checks passed");
    Ok(())
}

fn run(workspace: &Path, label: &str, arguments: &[&str]) -> Result<()> {
    println!("running {label}: cargo {}", arguments.join(" "));
    let status = Command::new("cargo")
        .args(arguments)
        .current_dir(workspace)
        .status()
        .with_context(|| format!("run Cargo {label}"))?;
    ensure!(status.success(), "{label} failed with {status}");
    Ok(())
}
