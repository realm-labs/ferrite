use anyhow::{Context as _, Result, ensure};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;

const REPORT_PATH: &str = "docs/reports/goal-01/g01-p10-b4-capacity-benchmarks.json";
const PROFILE_PATH: &str = "benchmarks/capacity-profiles.toml";

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "ferrite-cluster",
            "--",
            "capacity",
            "verify",
        ])
        .current_dir(workspace)
        .status()
        .context("verify named capacity profiles")?;
    ensure!(
        status.success(),
        "named capacity profile verification failed with {status}"
    );
    verify_report(
        &fs::read_to_string(workspace.join(PROFILE_PATH))?,
        &fs::read_to_string(workspace.join(REPORT_PATH))?,
    )?;
    println!("capacity benchmark report verified: {REPORT_PATH}");
    Ok(())
}

#[derive(Deserialize)]
struct ProfileDocument {
    profile: Vec<Workload>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Workload {
    name: String,
    regions: u64,
    nodes: u64,
    worlds: u64,
    mailbox_capacity: u64,
    warmup_ticks: u64,
    measured_ticks: u64,
    samples: u64,
    hotspot_percent: u64,
    rebalance_max_skew_regions: u64,
}

#[derive(Deserialize)]
struct CapacityReport {
    schema_version: u64,
    benchmark: String,
    revision: String,
    worktree_dirty: bool,
    runner: Runner,
    profiles: Vec<ProfileReport>,
    claim_boundary: String,
}

#[derive(Deserialize)]
struct Runner {
    build_profile: String,
}

#[derive(Deserialize)]
struct ProfileReport {
    workload: Workload,
    samples: u64,
    balanced_tick_ns: Metric,
    balanced_region_tick_ns: Metric,
    hotspot_tick_ns: Metric,
    hotspot_region_tick_ns: Metric,
    rss_delta_bytes: Option<Metric>,
    storage_encode_ns: Metric,
    durable_storage_bytes: u64,
    network_messages_per_tick: u64,
    network_bytes_per_tick: u64,
    cross_node_messages_per_tick: u64,
    balanced_peak_queue_depth: u64,
    hotspot_peak_queue_depth: u64,
    hotspot_queue_utilization_basis_points: u64,
    rebalance_ns: Metric,
    rebalance_moved_regions: u64,
    rebalance_moved_storage_bytes: u64,
    rebalance_final_skew_regions: u64,
    final_digest: String,
}

#[derive(Deserialize)]
struct Metric {
    minimum: u64,
    median: u64,
    p95: u64,
    maximum: u64,
    mean: f64,
    standard_deviation: f64,
    coefficient_of_variation_percent: f64,
}

fn verify_report(profile_text: &str, report_text: &str) -> Result<()> {
    let expected: ProfileDocument = toml::from_str(profile_text)?;
    let report: CapacityReport = serde_json::from_str(report_text)?;
    ensure!(
        report.schema_version == 1,
        "capacity report schema must be 1"
    );
    ensure!(
        report.benchmark == "ferrite-region-capacity-v1",
        "capacity report benchmark identity drifted"
    );
    ensure!(
        !report.worktree_dirty,
        "capacity report was measured from a dirty worktree"
    );
    ensure!(
        report.runner.build_profile == "release",
        "capacity report is not release-optimized"
    );
    ensure!(
        is_hex(&report.revision, 40),
        "capacity report revision is not a Git SHA-1"
    );
    ensure!(
        report
            .claim_boundary
            .contains("not a production player-capacity promise"),
        "capacity report lost its claim boundary"
    );
    ensure!(
        report.profiles.len() == expected.profile.len(),
        "capacity report profile count drifted"
    );
    for (profile, expected) in report.profiles.iter().zip(&expected.profile) {
        ensure!(
            profile.workload == *expected,
            "capacity workload {} drifted",
            expected.name
        );
        ensure!(
            profile.samples == expected.samples,
            "capacity sample count drifted"
        );
        for metric in [
            &profile.balanced_tick_ns,
            &profile.balanced_region_tick_ns,
            &profile.hotspot_tick_ns,
            &profile.hotspot_region_tick_ns,
            &profile.storage_encode_ns,
            &profile.rebalance_ns,
        ] {
            verify_metric(metric)?;
        }
        if let Some(metric) = &profile.rss_delta_bytes {
            verify_metric(metric)?;
        }
        ensure!(
            profile.durable_storage_bytes > 0,
            "capacity storage measurement is empty"
        );
        ensure!(
            profile.network_messages_per_tick == expected.regions,
            "network message count drifted"
        );
        ensure!(
            profile.network_bytes_per_tick > 0,
            "network byte measurement is empty"
        );
        ensure!(
            profile.cross_node_messages_per_tick <= profile.network_messages_per_tick,
            "cross-node message count exceeds total messages"
        );
        ensure!(
            profile.balanced_peak_queue_depth <= expected.mailbox_capacity
                && profile.hotspot_peak_queue_depth <= expected.mailbox_capacity,
            "capacity queue depth exceeds the configured bound"
        );
        ensure!(
            profile.hotspot_peak_queue_depth >= profile.balanced_peak_queue_depth,
            "hotspot queue pressure is below the balanced profile"
        );
        ensure!(
            profile.hotspot_queue_utilization_basis_points
                == profile.hotspot_peak_queue_depth * 10_000 / expected.mailbox_capacity,
            "hotspot queue utilization is inconsistent"
        );
        ensure!(
            profile.rebalance_moved_regions <= expected.regions,
            "rebalance moved too many Regions"
        );
        ensure!(
            profile.rebalance_moved_storage_bytes <= profile.durable_storage_bytes,
            "rebalance moved storage exceeds the durable footprint"
        );
        ensure!(
            profile.rebalance_final_skew_regions <= expected.rebalance_max_skew_regions,
            "rebalance objective was not met"
        );
        ensure!(
            is_hex(&profile.final_digest, 64),
            "capacity digest is not canonical"
        );
    }
    Ok(())
}

fn verify_metric(metric: &Metric) -> Result<()> {
    ensure!(
        metric.minimum <= metric.median
            && metric.median <= metric.p95
            && metric.p95 <= metric.maximum,
        "capacity metric quantiles are not ordered"
    );
    ensure!(
        metric.mean.is_finite()
            && metric.standard_deviation.is_finite()
            && metric.coefficient_of_variation_percent.is_finite()
            && metric.mean >= 0.0
            && metric.standard_deviation >= 0.0
            && metric.coefficient_of_variation_percent >= 0.0,
        "capacity metric variance is invalid"
    );
    Ok(())
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_capacity_report_matches_profiles_and_claim_boundary() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        verify_report(
            &fs::read_to_string(workspace.join(PROFILE_PATH)).unwrap(),
            &fs::read_to_string(workspace.join(REPORT_PATH)).unwrap(),
        )
        .unwrap();
    }
}
