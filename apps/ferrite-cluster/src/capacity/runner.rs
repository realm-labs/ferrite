use crate::capacity::config::{CapacityProfile, balanced_layout, hotspot_layout};
use ferrite_region_runtime::lattice::remoting::{LatticeRemotingAdapter, LatticeTransportFrame};
use ferrite_region_runtime::topology::cluster::{
    TopologyCluster, TopologyTransport, digest_snapshots, repartition_snapshots,
};
use ferrite_region_runtime::topology::layout::{TopologyLayout, TopologyRegionDescriptor};
use ferrite_region_runtime::topology::partition::TopologyPartitionSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::process::Command;
use std::time::Instant;

const FRAME_LIMIT: usize = 16 * 1024;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Sample {
    pub balanced_tick_ns: u64,
    pub balanced_region_tick_ns: u64,
    pub hotspot_tick_ns: u64,
    pub hotspot_region_tick_ns: u64,
    pub rss_delta_bytes: Option<u64>,
    pub storage_encode_ns: u64,
    pub durable_storage_bytes: u64,
    pub network_messages_per_tick: u64,
    pub network_bytes_per_tick: u64,
    pub cross_node_messages_per_tick: u64,
    pub network_node_pairs: u64,
    pub balanced_peak_queue_depth: u64,
    pub hotspot_peak_queue_depth: u64,
    pub hotspot_queue_utilization_basis_points: u64,
    pub rebalance_ns: u64,
    pub rebalance_moved_regions: u64,
    pub rebalance_moved_storage_bytes: u64,
    pub rebalance_final_skew_regions: u64,
    pub final_digest: String,
}

struct Measurement {
    tick_ns: u64,
    region_tick_ns: u64,
    rss_delta_bytes: Option<u64>,
    storage_encode_ns: u64,
    durable_storage_bytes: u64,
    traffic: Traffic,
    layout: TopologyLayout,
    snapshots: Vec<TopologyPartitionSnapshot>,
    digest: [u8; 32],
}

#[derive(Default)]
struct Traffic {
    messages: u64,
    bytes: u64,
    cross_node_messages: u64,
    node_pairs: u64,
    peak_queue_depth: u64,
}

pub(super) fn run(profile: &CapacityProfile) -> Result<Sample, Box<dyn Error>> {
    let balanced = measure(profile, balanced_layout(profile)?, true)?;
    let hotspot = measure(profile, hotspot_layout(profile)?, false)?;
    if balanced.digest != hotspot.digest {
        return Err("balanced and hotspot topology state diverged".into());
    }
    let rebalance = measure_rebalance(profile, &hotspot.layout, &hotspot.snapshots)?;
    Ok(Sample {
        balanced_tick_ns: balanced.tick_ns,
        balanced_region_tick_ns: balanced.region_tick_ns,
        hotspot_tick_ns: hotspot.tick_ns,
        hotspot_region_tick_ns: hotspot.region_tick_ns,
        rss_delta_bytes: balanced.rss_delta_bytes,
        storage_encode_ns: balanced.storage_encode_ns,
        durable_storage_bytes: balanced.durable_storage_bytes,
        network_messages_per_tick: balanced.traffic.messages,
        network_bytes_per_tick: balanced.traffic.bytes,
        cross_node_messages_per_tick: balanced.traffic.cross_node_messages,
        network_node_pairs: balanced.traffic.node_pairs,
        balanced_peak_queue_depth: balanced.traffic.peak_queue_depth,
        hotspot_peak_queue_depth: hotspot.traffic.peak_queue_depth,
        hotspot_queue_utilization_basis_points: hotspot
            .traffic
            .peak_queue_depth
            .saturating_mul(10_000)
            / u64::try_from(profile.mailbox_capacity)?,
        rebalance_ns: rebalance.elapsed_ns,
        rebalance_moved_regions: rebalance.moved_regions,
        rebalance_moved_storage_bytes: rebalance.moved_storage_bytes,
        rebalance_final_skew_regions: rebalance.final_skew,
        final_digest: hex(&balanced.digest),
    })
}

pub(super) fn verify(profile: &CapacityProfile) -> Result<(), Box<dyn Error>> {
    let balanced = balanced_layout(profile)?;
    let hotspot = hotspot_layout(profile)?;
    let target = rebalance_layout(&hotspot)?;
    if node_skew(&balanced)? > 1
        || node_skew(&target)? > u64::from(profile.rebalance_max_skew_regions)
    {
        return Err(format!(
            "capacity profile {} misses its rebalance objective",
            profile.name
        )
        .into());
    }
    let peak = hotspot
        .descriptors()
        .fold(
            vec![0_usize; usize::from(profile.nodes)],
            |mut counts, descriptor| {
                counts[usize::from(descriptor.node)] += 1;
                counts
            },
        )
        .into_iter()
        .max()
        .unwrap_or(0);
    if peak > profile.mailbox_capacity {
        return Err(format!(
            "capacity profile {} can overflow its hotspot mailbox",
            profile.name
        )
        .into());
    }
    Ok(())
}

fn measure(
    profile: &CapacityProfile,
    layout: TopologyLayout,
    measure_memory: bool,
) -> Result<Measurement, Box<dyn Error>> {
    let rss_before = measure_memory.then(resident_bytes).flatten();
    let mut cluster = TopologyCluster::seeded(
        layout.clone(),
        profile.mailbox_capacity,
        TopologyTransport::LatticeInProcess,
    )?;
    cluster.run_to(profile.warmup_ticks)?;
    let rss_after = measure_memory.then(resident_bytes).flatten();
    let first_tick = profile.warmup_ticks + 1;
    let final_tick = profile
        .warmup_ticks
        .checked_add(profile.measured_ticks)
        .ok_or("capacity tick range overflow")?;
    let started = Instant::now();
    for tick in first_tick..=final_tick {
        cluster.run_tick_number(tick)?;
    }
    let elapsed_ns = nanos(started.elapsed().as_nanos());
    let digest = cluster.digest();
    std::hint::black_box(digest);

    let messages = cluster.emit_tick_number(final_tick + 1)?;
    let traffic = observe_traffic(&layout, &messages)?;
    let snapshots = cluster.snapshots();
    let storage_started = Instant::now();
    let durable_storage_bytes = snapshots
        .iter()
        .flat_map(|snapshot| snapshot.regions.iter())
        .try_fold(0_u64, |total, region| -> Result<u64, Box<dyn Error>> {
            let bytes = region.recovery_point()?.encode()?;
            Ok(total
                .checked_add(u64::try_from(bytes.len())?)
                .ok_or("durable storage byte count overflow")?)
        })?;
    let storage_encode_ns = nanos(storage_started.elapsed().as_nanos());
    let tick_ns = elapsed_ns / profile.measured_ticks;
    Ok(Measurement {
        tick_ns,
        region_tick_ns: tick_ns / u64::from(profile.regions),
        rss_delta_bytes: rss_before
            .zip(rss_after)
            .map(|(before, after)| after.saturating_sub(before)),
        storage_encode_ns,
        durable_storage_bytes,
        traffic,
        layout,
        snapshots,
        digest,
    })
}

fn observe_traffic(
    layout: &TopologyLayout,
    messages: &[ferrite_region_runtime::topology::partition::TopologyWireMessage],
) -> Result<Traffic, Box<dyn Error>> {
    let adapter = LatticeRemotingAdapter::new(FRAME_LIMIT)?;
    let mut queues = BTreeMap::<u16, u64>::new();
    let mut pairs = BTreeSet::new();
    let mut traffic = Traffic::default();
    for message in messages {
        traffic.messages += 1;
        traffic.bytes = traffic
            .bytes
            .checked_add(u64::try_from(message.bytes.len())?)
            .ok_or("network byte count overflow")?;
        let frame = LatticeTransportFrame::from_transport_payload(message.bytes.clone());
        let envelope = adapter.decode(&frame)?;
        let source = layout.descriptor(envelope.source())?.node;
        let target = layout.descriptor(envelope.target())?.node;
        *queues.entry(target).or_default() += 1;
        if source != target {
            traffic.cross_node_messages += 1;
            pairs.insert((source, target));
        }
    }
    traffic.node_pairs = u64::try_from(pairs.len())?;
    traffic.peak_queue_depth = queues.values().copied().max().unwrap_or(0);
    Ok(traffic)
}

struct RebalanceMeasurement {
    elapsed_ns: u64,
    moved_regions: u64,
    moved_storage_bytes: u64,
    final_skew: u64,
}

fn measure_rebalance(
    profile: &CapacityProfile,
    layout: &TopologyLayout,
    snapshots: &[TopologyPartitionSnapshot],
) -> Result<RebalanceMeasurement, Box<dyn Error>> {
    let target = rebalance_layout(layout)?;
    let moved = layout
        .descriptors()
        .filter(|descriptor| {
            target
                .descriptor(&descriptor.key)
                .is_ok_and(|candidate| candidate.node != descriptor.node)
        })
        .map(|descriptor| descriptor.key.clone())
        .collect::<BTreeSet<_>>();
    let moved_storage_bytes = snapshots
        .iter()
        .flat_map(|snapshot| snapshot.regions.iter())
        .filter(|region| moved.contains(&region.key))
        .try_fold(0_u64, |total, region| -> Result<u64, Box<dyn Error>> {
            let bytes = region.recovery_point()?.encode()?;
            Ok(total
                .checked_add(u64::try_from(bytes.len())?)
                .ok_or("rebalance byte count overflow")?)
        })?;
    let started = Instant::now();
    let rebalanced = repartition_snapshots(snapshots, &target)?;
    let elapsed_ns = nanos(started.elapsed().as_nanos());
    if digest_snapshots(&rebalanced) != digest_snapshots(snapshots) {
        return Err("rebalance changed canonical committed state".into());
    }
    let final_skew = node_skew(&target)?;
    if final_skew > u64::from(profile.rebalance_max_skew_regions) {
        return Err(format!("rebalance finished with {final_skew}-Region skew").into());
    }
    Ok(RebalanceMeasurement {
        elapsed_ns,
        moved_regions: u64::try_from(moved.len())?,
        moved_storage_bytes,
        final_skew,
    })
}

fn rebalance_layout(layout: &TopologyLayout) -> Result<TopologyLayout, Box<dyn Error>> {
    let region_count = u16::try_from(layout.len())?;
    let base = region_count / layout.node_count();
    let remainder = region_count % layout.node_count();
    let targets = (0..layout.node_count())
        .map(|node| base + u16::from(node < remainder))
        .collect::<Vec<_>>();
    let mut counts = vec![0_u16; usize::from(layout.node_count())];
    for descriptor in layout.descriptors() {
        counts[usize::from(descriptor.node)] += 1;
    }
    let mut destinations = VecDeque::new();
    for (node, (&count, &target)) in counts.iter().zip(&targets).enumerate() {
        for _ in count..target {
            destinations.push_back(u16::try_from(node)?);
        }
    }
    let mut remaining_excess = counts
        .iter()
        .zip(&targets)
        .map(|(&count, &target)| count.saturating_sub(target))
        .collect::<Vec<_>>();
    let descriptors = layout
        .descriptors()
        .cloned()
        .map(|mut descriptor| {
            let node = usize::from(descriptor.node);
            if remaining_excess[node] > 0 {
                descriptor.node = destinations
                    .pop_front()
                    .ok_or("rebalance destination accounting mismatch")?;
                descriptor.generation = descriptor.generation.checked_next()?;
                remaining_excess[node] -= 1;
            }
            Ok(descriptor)
        })
        .collect::<Result<Vec<TopologyRegionDescriptor>, Box<dyn Error>>>()?;
    if !destinations.is_empty() || remaining_excess.iter().any(|&count| count != 0) {
        return Err("rebalance source accounting mismatch".into());
    }
    Ok(TopologyLayout::new(descriptors, layout.node_count())?)
}

fn node_skew(layout: &TopologyLayout) -> Result<u64, Box<dyn Error>> {
    let mut counts = vec![0_u64; usize::from(layout.node_count())];
    for descriptor in layout.descriptors() {
        counts[usize::from(descriptor.node)] += 1;
    }
    let minimum = counts.iter().copied().min().ok_or("empty node counts")?;
    let maximum = counts.iter().copied().max().ok_or("empty node counts")?;
    Ok(maximum - minimum)
}

fn resident_bytes() -> Option<u64> {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let kibibytes = String::from_utf8(output.stdout)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    kibibytes.checked_mul(1024)
}

fn nanos(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_profile() -> CapacityProfile {
        CapacityProfile {
            name: "tiny-test".to_owned(),
            regions: 8,
            nodes: 3,
            worlds: 2,
            mailbox_capacity: 8,
            warmup_ticks: 2,
            measured_ticks: 3,
            samples: 3,
            hotspot_percent: 75,
            rebalance_max_skew_regions: 1,
        }
    }

    #[test]
    fn hotspot_rebalance_is_minimal_balanced_and_generation_fenced() {
        let profile = tiny_profile();
        let hotspot = hotspot_layout(&profile).unwrap();
        let balanced = rebalance_layout(&hotspot).unwrap();
        assert_eq!(node_skew(&balanced).unwrap(), 1);
        let moved = hotspot
            .descriptors()
            .filter(|descriptor| {
                balanced.descriptor(&descriptor.key).unwrap().node != descriptor.node
            })
            .count();
        assert_eq!(moved, 3);
        for descriptor in hotspot.descriptors() {
            let target = balanced.descriptor(&descriptor.key).unwrap();
            if target.node != descriptor.node {
                assert_eq!(target.generation.get(), descriptor.generation.get() + 1);
            }
        }
    }

    #[test]
    fn tiny_profile_exercises_every_metric_family() {
        let sample = run(&tiny_profile()).unwrap();
        assert_eq!(sample.network_messages_per_tick, 8);
        assert!(sample.network_bytes_per_tick > 0);
        assert!(sample.cross_node_messages_per_tick > 0);
        assert!(sample.hotspot_peak_queue_depth > sample.balanced_peak_queue_depth);
        assert_eq!(sample.rebalance_final_skew_regions, 1);
        assert_eq!(sample.final_digest.len(), 64);
    }
}
