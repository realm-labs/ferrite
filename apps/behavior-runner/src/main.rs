#![forbid(unsafe_code)]

//! Audited behavior scenario runner.

use anyhow::{Context as _, Result, bail};
use ferrite_testkit::recording::RecordingTarget;
use ferrite_testkit::scenario::Scenario;
use ferrite_testkit::worldgen_oracle::anvil::normalize_official_chunk;
use ferrite_testkit::worldgen_oracle::compare::compare_chunks;
use ferrite_testkit::worldgen_oracle::ferrite::generate_current_ferrite_chunk;
use ferrite_testkit::worldgen_oracle::model::SemanticChunk;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [command, path] if command == "validate" => validate(Path::new(path)),
        [command, path] if command == "run" => execute(Path::new(path)),
        [command, world, dimension, x, z, output] if command == "worldgen-normalize-official" => {
            normalize_official(
                Path::new(world),
                dimension,
                x.parse().context("parse chunk X")?,
                z.parse().context("parse chunk Z")?,
                Path::new(output),
            )
        }
        [command, official, ferrite] if command == "worldgen-compare" => {
            compare_worldgen(Path::new(official), Path::new(ferrite))
        }
        [command, dimension, seed, x, z, output] if command == "worldgen-normalize-ferrite" => {
            normalize_ferrite(
                dimension,
                seed.parse().context("parse world seed")?,
                x.parse().context("parse chunk X")?,
                z.parse().context("parse chunk Z")?,
                Path::new(output),
            )
        }
        _ => bail!(
            "usage: behavior-runner <validate|run> <scenario.toml>\n\
             or: behavior-runner worldgen-normalize-official \
             <world-root> <dimension> <chunk-x> <chunk-z> <output.json>\n\
             or: behavior-runner worldgen-normalize-ferrite \
             <dimension> <seed> <chunk-x> <chunk-z> <output.json>\n\
             or: behavior-runner worldgen-compare <official.json> <ferrite.json>"
        ),
    }
}

fn normalize_ferrite(
    dimension: &str,
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    output: &Path,
) -> Result<()> {
    let chunk = generate_current_ferrite_chunk(dimension, seed, chunk_x, chunk_z)
        .with_context(|| format!("normalize Ferrite chunk {dimension}/{chunk_x}/{chunk_z}"))?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    fs::write(output, serde_json::to_vec_pretty(&chunk)?)
        .with_context(|| format!("write normalized chunk {}", output.display()))?;
    println!(
        "normalized Ferrite {dimension}/{chunk_x}/{chunk_z}: {}",
        chunk.canonical_digest()
    );
    Ok(())
}

fn normalize_official(
    world: &Path,
    dimension: &str,
    chunk_x: i32,
    chunk_z: i32,
    output: &Path,
) -> Result<()> {
    let chunk = normalize_official_chunk(world, dimension, chunk_x, chunk_z)
        .with_context(|| format!("normalize official chunk {dimension}/{chunk_x}/{chunk_z}"))?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    fs::write(output, serde_json::to_vec_pretty(&chunk)?)
        .with_context(|| format!("write normalized chunk {}", output.display()))?;
    println!(
        "normalized official {dimension}/{chunk_x}/{chunk_z}: {}",
        chunk.canonical_digest()
    );
    Ok(())
}

fn compare_worldgen(official: &Path, ferrite: &Path) -> Result<()> {
    let official = read_semantic_chunk(official)?;
    let ferrite = read_semantic_chunk(ferrite)?;
    match compare_chunks(&official, &ferrite) {
        Ok(()) => {
            println!(
                "worldgen semantic identity: {}",
                official.canonical_digest()
            );
            Ok(())
        }
        Err(divergence) => {
            println!("{}", serde_json::to_string_pretty(&divergence)?);
            bail!(
                "worldgen semantic divergence at stage {} field {}",
                divergence.stage,
                divergence.field
            )
        }
    }
}

fn read_semantic_chunk(path: &Path) -> Result<SemanticChunk> {
    let bytes =
        fs::read(path).with_context(|| format!("read semantic chunk {}", path.display()))?;
    let chunk: SemanticChunk = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse semantic chunk {}", path.display()))?;
    chunk
        .validate_shape()
        .map_err(anyhow::Error::msg)
        .with_context(|| format!("validate semantic chunk {}", path.display()))?;
    Ok(chunk)
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
