//! Named, reproducible synthetic capacity profiles and measurement reports.

mod config;
mod runner;

use crate::capacity::config::CapacityProfile;
use crate::capacity::runner::Sample;
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const BENCHMARK_ID: &str = "ferrite-region-capacity-v1";

pub(crate) fn run(mut arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let command = arguments
        .next()
        .ok_or("capacity requires verify or benchmark")?;
    match command.as_str() {
        "verify" => verify(parse_options(arguments, false)?),
        "benchmark" => benchmark(parse_options(arguments, true)?),
        _ => Err(format!("unknown capacity command {command}").into()),
    }
}

pub(crate) fn worker(mut arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let profile = match (arguments.next().as_deref(), arguments.next()) {
        (Some("--profile"), Some(profile)) => profile,
        _ => return Err("capacity-worker requires --profile <NAME>".into()),
    };
    if arguments.next().is_some() {
        return Err("capacity-worker received unexpected arguments".into());
    }
    let profile = config::select(Some(&profile))?
        .pop()
        .ok_or("capacity worker profile disappeared")?;
    serde_json::to_writer(std::io::stdout().lock(), &runner::run(&profile)?)?;
    Ok(())
}

struct Options {
    profile: Option<String>,
    output: Option<PathBuf>,
}

fn parse_options(
    mut arguments: impl Iterator<Item = String>,
    allow_output: bool,
) -> Result<Options, Box<dyn Error>> {
    let mut profile = None;
    let mut output = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--profile" => profile = Some(arguments.next().ok_or("--profile requires a name")?),
            "--output" if allow_output => {
                output = Some(PathBuf::from(
                    arguments.next().ok_or("--output requires a path")?,
                ));
            }
            _ => return Err(format!("unknown capacity argument {argument}").into()),
        }
    }
    Ok(Options { profile, output })
}

fn verify(options: Options) -> Result<(), Box<dyn Error>> {
    let profiles = config::select(options.profile.as_deref())?;
    for profile in &profiles {
        runner::verify(profile)?;
    }
    println!(
        "capacity profiles verified: schema=1 profiles={} names={}",
        profiles.len(),
        profiles
            .iter()
            .map(|profile| profile.name.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    Ok(())
}

fn benchmark(options: Options) -> Result<(), Box<dyn Error>> {
    let profiles = config::select(options.profile.as_deref())?;
    let mut results = Vec::with_capacity(profiles.len());
    for profile in profiles {
        eprintln!(
            "benchmarking capacity profile {} with {} isolated samples",
            profile.name, profile.samples
        );
        let samples = (0..profile.samples)
            .map(|_| run_isolated_sample(&profile.name))
            .collect::<Result<Vec<_>, _>>()?;
        results.push(summarize(profile, samples)?);
    }
    let report = CapacityReport {
        schema_version: 1,
        benchmark: BENCHMARK_ID,
        generated_unix_seconds: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        revision: git_output(&["rev-parse", "HEAD"]),
        worktree_dirty: !git_output(&["status", "--porcelain"]).is_empty(),
        runner: RunnerMetadata::capture(),
        command: "cargo ferrite cargo bench run --release -p ferrite-cluster -- capacity benchmark --output <PATH>".to_owned(),
        profiles: results,
        claim_boundary: "Synthetic Region topology measurements only; not a production player-capacity promise.".to_owned(),
    };
    let encoded = serde_json::to_string_pretty(&report)? + "\n";
    if let Some(path) = options.output {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, encoded)?;
        println!("capacity benchmark report written to {}", path.display());
    } else {
        print!("{encoded}");
    }
    Ok(())
}

fn run_isolated_sample(profile: &str) -> Result<Sample, Box<dyn Error>> {
    let output = Command::new(std::env::current_exe()?)
        .args(["capacity-worker", "--profile", profile])
        .stdin(Stdio::null())
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "capacity sample {profile} failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[derive(Serialize)]
struct CapacityReport {
    schema_version: u16,
    benchmark: &'static str,
    generated_unix_seconds: u64,
    revision: String,
    worktree_dirty: bool,
    runner: RunnerMetadata,
    command: String,
    profiles: Vec<ProfileReport>,
    claim_boundary: String,
}

#[derive(Serialize)]
struct RunnerMetadata {
    operating_system: &'static str,
    architecture: &'static str,
    cpu: String,
    logical_parallelism: usize,
    rustc: String,
    build_profile: &'static str,
}

impl RunnerMetadata {
    fn capture() -> Self {
        Self {
            operating_system: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
            cpu: cpu_identity(),
            logical_parallelism: std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
            rustc: command_output("rustc", &["--version"]),
            build_profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
        }
    }
}

#[derive(Serialize)]
struct ProfileReport {
    workload: CapacityProfile,
    samples: u16,
    balanced_tick_ns: MetricSummary,
    balanced_region_tick_ns: MetricSummary,
    hotspot_tick_ns: MetricSummary,
    hotspot_region_tick_ns: MetricSummary,
    hotspot_slowdown_ratio: f64,
    rss_delta_bytes: Option<MetricSummary>,
    storage_encode_ns: MetricSummary,
    durable_storage_bytes: u64,
    network_messages_per_tick: u64,
    network_bytes_per_tick: u64,
    cross_node_messages_per_tick: u64,
    network_node_pairs: u64,
    balanced_peak_queue_depth: u64,
    hotspot_peak_queue_depth: u64,
    hotspot_queue_utilization_basis_points: u64,
    rebalance_ns: MetricSummary,
    rebalance_moved_regions: u64,
    rebalance_moved_storage_bytes: u64,
    rebalance_final_skew_regions: u64,
    final_digest: String,
}

fn summarize(
    profile: CapacityProfile,
    samples: Vec<Sample>,
) -> Result<ProfileReport, Box<dyn Error>> {
    if samples.len() != usize::from(profile.samples) {
        return Err("capacity sample count mismatch".into());
    }
    let first = samples.first().ok_or("capacity samples are empty")?;
    for sample in &samples[1..] {
        if sample.durable_storage_bytes != first.durable_storage_bytes
            || sample.network_messages_per_tick != first.network_messages_per_tick
            || sample.network_bytes_per_tick != first.network_bytes_per_tick
            || sample.cross_node_messages_per_tick != first.cross_node_messages_per_tick
            || sample.network_node_pairs != first.network_node_pairs
            || sample.balanced_peak_queue_depth != first.balanced_peak_queue_depth
            || sample.hotspot_peak_queue_depth != first.hotspot_peak_queue_depth
            || sample.hotspot_queue_utilization_basis_points
                != first.hotspot_queue_utilization_basis_points
            || sample.rebalance_moved_regions != first.rebalance_moved_regions
            || sample.rebalance_moved_storage_bytes != first.rebalance_moved_storage_bytes
            || sample.rebalance_final_skew_regions != first.rebalance_final_skew_regions
            || sample.final_digest != first.final_digest
        {
            return Err(format!(
                "capacity profile {} produced unstable exact metrics",
                profile.name
            )
            .into());
        }
    }
    let balanced_tick_ns = metric(&samples, |sample| sample.balanced_tick_ns);
    let hotspot_tick_ns = metric(&samples, |sample| sample.hotspot_tick_ns);
    let slowdown = hotspot_tick_ns.mean / balanced_tick_ns.mean;
    Ok(ProfileReport {
        samples: profile.samples,
        balanced_region_tick_ns: metric(&samples, |sample| sample.balanced_region_tick_ns),
        hotspot_region_tick_ns: metric(&samples, |sample| sample.hotspot_region_tick_ns),
        rss_delta_bytes: optional_metric(&samples, |sample| sample.rss_delta_bytes),
        storage_encode_ns: metric(&samples, |sample| sample.storage_encode_ns),
        rebalance_ns: metric(&samples, |sample| sample.rebalance_ns),
        durable_storage_bytes: first.durable_storage_bytes,
        network_messages_per_tick: first.network_messages_per_tick,
        network_bytes_per_tick: first.network_bytes_per_tick,
        cross_node_messages_per_tick: first.cross_node_messages_per_tick,
        network_node_pairs: first.network_node_pairs,
        balanced_peak_queue_depth: first.balanced_peak_queue_depth,
        hotspot_peak_queue_depth: first.hotspot_peak_queue_depth,
        hotspot_queue_utilization_basis_points: first.hotspot_queue_utilization_basis_points,
        rebalance_moved_regions: first.rebalance_moved_regions,
        rebalance_moved_storage_bytes: first.rebalance_moved_storage_bytes,
        rebalance_final_skew_regions: first.rebalance_final_skew_regions,
        final_digest: first.final_digest.clone(),
        workload: profile,
        balanced_tick_ns,
        hotspot_tick_ns,
        hotspot_slowdown_ratio: slowdown,
    })
}

#[derive(Serialize)]
struct MetricSummary {
    minimum: u64,
    median: u64,
    p95: u64,
    maximum: u64,
    mean: f64,
    standard_deviation: f64,
    coefficient_of_variation_percent: f64,
}

fn metric(samples: &[Sample], select: impl Fn(&Sample) -> u64) -> MetricSummary {
    let mut values = samples.iter().map(select).collect::<Vec<_>>();
    values.sort_unstable();
    summarize_values(&values)
}

fn optional_metric(
    samples: &[Sample],
    select: impl Fn(&Sample) -> Option<u64>,
) -> Option<MetricSummary> {
    let mut values = samples.iter().filter_map(select).collect::<Vec<_>>();
    if values.len() != samples.len() {
        return None;
    }
    values.sort_unstable();
    Some(summarize_values(&values))
}

fn summarize_values(values: &[u64]) -> MetricSummary {
    let mean = values.iter().map(|&value| value as f64).sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|&value| {
            let delta = value as f64 - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    let p95_index = (values.len() * 95).div_ceil(100).saturating_sub(1);
    let standard_deviation = variance.sqrt();
    MetricSummary {
        minimum: values[0],
        median: values[values.len() / 2],
        p95: values[p95_index],
        maximum: values[values.len() - 1],
        mean,
        standard_deviation,
        coefficient_of_variation_percent: if mean == 0.0 {
            0.0
        } else {
            standard_deviation / mean * 100.0
        },
    }
}

fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("ferrite-cluster is nested under the workspace apps directory")
}

fn git_output(arguments: &[&str]) -> String {
    Command::new("git")
        .args(arguments)
        .current_dir(workspace())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_owned())
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn cpu_identity() -> String {
    let apple = command_output("sysctl", &["-n", "machdep.cpu.brand_string"]);
    if !apple.is_empty() {
        return apple;
    }
    std::env::consts::ARCH.to_owned()
}

fn command_output(program: &str, arguments: &[&str]) -> String {
    Command::new(program)
        .args(arguments)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_summary_uses_population_variance_and_nearest_rank_p95() {
        let samples = (1..=5)
            .map(|value| Sample {
                balanced_tick_ns: value,
                balanced_region_tick_ns: 0,
                hotspot_tick_ns: 0,
                hotspot_region_tick_ns: 0,
                rss_delta_bytes: None,
                storage_encode_ns: 0,
                durable_storage_bytes: 0,
                network_messages_per_tick: 0,
                network_bytes_per_tick: 0,
                cross_node_messages_per_tick: 0,
                network_node_pairs: 0,
                balanced_peak_queue_depth: 0,
                hotspot_peak_queue_depth: 0,
                hotspot_queue_utilization_basis_points: 0,
                rebalance_ns: 0,
                rebalance_moved_regions: 0,
                rebalance_moved_storage_bytes: 0,
                rebalance_final_skew_regions: 0,
                final_digest: String::new(),
            })
            .collect::<Vec<_>>();
        let summary = metric(&samples, |sample| sample.balanced_tick_ns);
        assert_eq!(summary.median, 3);
        assert_eq!(summary.p95, 5);
        assert!((summary.standard_deviation - 2.0_f64.sqrt()).abs() < f64::EPSILON);
    }
}
