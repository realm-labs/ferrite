use crate::{audit, cache, deployment, topology};
use anyhow::{Context as _, Result, ensure};
use std::path::Path;
use std::process::Command;

pub(crate) fn check(workspace: &Path) -> Result<()> {
    cache::maintain(workspace, cache::ApplyMode::Apply)?;
    audit::verify(workspace)?;
    deployment::verify(workspace)?;
    topology::verify(workspace)?;
    run(
        workspace,
        "behavior scenario validation",
        &[
            "run",
            "-q",
            "-p",
            "behavior-runner",
            "--",
            "validate",
            "tests/fixtures/scenarios/recording-smoke.toml",
        ],
    )?;
    run(
        workspace,
        "behavior scenario execution",
        &[
            "run",
            "-q",
            "-p",
            "behavior-runner",
            "--",
            "run",
            "tests/fixtures/scenarios/recording-smoke.toml",
        ],
    )?;
    run(
        workspace,
        "C0/C1 headless protocol conformance",
        &["run", "-q", "-p", "protocol-conformance", "--", "run"],
    )?;
    run(
        workspace,
        "C0/C1 loopback TCP smoke",
        &["run", "-q", "-p", "protocol-conformance", "--", "tcp-smoke"],
    )?;
    run(
        workspace,
        "C2 playable loopback TCP smoke",
        &["run", "-q", "-p", "protocol-conformance", "--", "c2-smoke"],
    )?;
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
