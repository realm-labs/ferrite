use anyhow::{Context as _, Result, ensure};
use std::path::Path;
use std::process::Command;

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    ensure_clean(workspace)?;
    run(
        workspace,
        "cross-platform deterministic vectors",
        &[
            "test",
            "-p",
            "ferrite-replay",
            "--test",
            "cross_platform_vectors",
            "--",
            "--nocapture",
        ],
    )?;
    run(
        workspace,
        "playable topology equivalence",
        &[
            "run",
            "-q",
            "-p",
            "ferrite-cluster",
            "--",
            "verify-playable",
        ],
    )?;
    run(
        workspace,
        "canonical replay topology equivalence",
        &["run", "-q", "-p", "ferrite-cluster", "--", "verify-replay"],
    )?;
    run(
        workspace,
        "locked unmodified-client fixture",
        &[
            "run",
            "-q",
            "-p",
            "protocol-conformance",
            "--",
            "verify-vanilla-fixture",
            "--client-jar",
            "target/mc-reference/26.2/client.jar",
            "--registry-report",
            "target/mc-reference/26.2/generated/reports/registries.json",
        ],
    )?;
    run(
        workspace,
        "complete repository and implementation acceptance",
        &["ferrite", "task", "check"],
    )?;
    let diff = Command::new("git")
        .args(["diff", "--check"])
        .current_dir(workspace)
        .status()
        .context("run final diff check")?;
    ensure!(diff.success(), "final diff check failed with {diff}");
    ensure_clean(workspace)?;
    println!(
        "clean-checkout acceptance passed: revision={} platform={}/{}",
        git_output(workspace, &["rev-parse", "HEAD"])?,
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    Ok(())
}

fn run(workspace: &Path, label: &str, arguments: &[&str]) -> Result<()> {
    println!("running {label}: cargo {}", arguments.join(" "));
    let status = Command::new("cargo")
        .args(arguments)
        .current_dir(workspace)
        .status()
        .with_context(|| format!("run {label}"))?;
    ensure!(status.success(), "{label} failed with {status}");
    Ok(())
}

fn ensure_clean(workspace: &Path) -> Result<()> {
    let status = git_output(
        workspace,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    ensure!(
        status.is_empty(),
        "acceptance requires a clean checkout; found:\n{status}"
    );
    Ok(())
}

fn git_output(workspace: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(workspace)
        .output()
        .context("run Git acceptance query")?;
    ensure!(
        output.status.success(),
        "Git acceptance query failed with {}",
        output.status
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
