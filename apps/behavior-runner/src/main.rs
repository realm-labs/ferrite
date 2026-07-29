#![forbid(unsafe_code)]

//! Audited behavior scenario runner.

use anyhow::{Context as _, Result, bail};
use ferrite_testkit::recording::RecordingTarget;
use ferrite_testkit::scenario::Scenario;
use std::env;
use std::path::Path;

fn main() -> Result<()> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [command, path] if command == "validate" => validate(Path::new(path)),
        [command, path] if command == "run" => execute(Path::new(path)),
        _ => bail!("usage: behavior-runner <validate|run> <scenario.toml>"),
    }
}

fn validate(path: &Path) -> Result<()> {
    let scenario = Scenario::read(path)
        .with_context(|| format!("validate behavior scenario {}", path.display()))?;
    println!(
        "validated {} with {} steps",
        scenario.id(),
        scenario.steps().len()
    );
    Ok(())
}

fn execute(path: &Path) -> Result<()> {
    let scenario = Scenario::read(path)
        .with_context(|| format!("read behavior scenario {}", path.display()))?;
    let report = ferrite_testkit::scenario::run(&scenario, &mut RecordingTarget::default())
        .with_context(|| format!("run behavior scenario {}", scenario.id()))?;
    println!(
        "passed {}: {} steps through tick {}, snapshot {}",
        scenario.id(),
        report.executed_steps(),
        report.final_tick(),
        report.final_snapshot().digest()
    );
    Ok(())
}
