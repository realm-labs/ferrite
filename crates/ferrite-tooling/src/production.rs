//! Machine verification for the formal-server production integration denominator.

use anyhow::{Context as _, Result, ensure};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

const MANIFEST_PATH: &str = "goals/minecraft-java-26.2/production-integration.toml";
const PACKET_SOURCE: &str = "crates/ferrite-protocol/src/java_26_2/play/serverbound/packet.rs";
const FORMAL_ENTRY: &str = "apps/ferrite-server -> NodeProcess -> MinecraftGateway";
const SERVICE_IDS: [&str; 11] = [
    "connection/base-custom-payload",
    "connection/configuration",
    "connection/login-admission",
    "connection/play-installation",
    "connection/status",
    "process/management-drain",
    "process/membership-placement",
    "region/tick-composition",
    "service/optional-c4-gates",
    "storage/production-continuity",
    "world/bootstrap-terrain",
];
const TARGET_GOALS: [&str; 5] = ["Goal 03", "Goal 04", "Goal 05", "Goal 06", "Goal 07"];
const ALL_STAGES: [IntegrationStage; 7] = [
    IntegrationStage::Ingress,
    IntegrationStage::Semantic,
    IntegrationStage::Authority,
    IntegrationStage::Continuity,
    IntegrationStage::Projection,
    IntegrationStage::FocusedTest,
    IntegrationStage::ClientAcceptance,
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductionManifest {
    schema_version: u32,
    goal: String,
    reference_version: String,
    formal_entry: String,
    packet_source: String,
    ordering: String,
    stages: Vec<IntegrationStage>,
    totals: ManifestTotals,
    service: Vec<ServiceRecord>,
    serverbound: Vec<ServerboundRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestTotals {
    service_rows: usize,
    serverbound_rows: usize,
    serverbound_packets: usize,
    integrated_rows: usize,
    partial_rows: usize,
    unsupported_rows: usize,
    planned_rows: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServiceRecord {
    #[serde(flatten)]
    integration: IntegrationRecord,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServerboundRecord {
    #[serde(flatten)]
    integration: IntegrationRecord,
    packets: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IntegrationRecord {
    id: String,
    responsibility: String,
    owner: String,
    status: IntegrationStatus,
    target_goals: Vec<String>,
    implemented_stages: Vec<IntegrationStage>,
    not_applicable_stages: Vec<IntegrationStage>,
    gaps: Vec<IntegrationStage>,
    evidence: Vec<String>,
    tests: Vec<String>,
    #[serde(default)]
    rationale: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum IntegrationStatus {
    Integrated,
    Partial,
    Unsupported,
    Planned,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum IntegrationStage {
    Ingress,
    Semantic,
    Authority,
    Continuity,
    Projection,
    FocusedTest,
    ClientAcceptance,
}

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    let manifest_path = workspace.join(MANIFEST_PATH);
    let manifest_text = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: ProductionManifest = toml::from_str(&manifest_text)
        .with_context(|| format!("parse {}", manifest_path.display()))?;

    verify_root(&manifest)?;
    verify_services(workspace, &manifest.service)?;
    verify_serverbound(workspace, &manifest.serverbound)?;
    verify_totals(&manifest)?;

    println!(
        "production integration manifest verified: {} service rows, {} serverbound rows, {} packets",
        manifest.service.len(),
        manifest.serverbound.len(),
        manifest.totals.serverbound_packets
    );
    Ok(())
}

fn verify_root(manifest: &ProductionManifest) -> Result<()> {
    ensure!(
        manifest.schema_version == 1,
        "unsupported production manifest schema"
    );
    ensure!(
        manifest.goal == "Goal 03",
        "production manifest goal is not Goal 03"
    );
    ensure!(
        manifest.reference_version == "26.2",
        "production manifest reference version is not 26.2"
    );
    ensure!(
        manifest.formal_entry == FORMAL_ENTRY,
        "production manifest formal entry is stale"
    );
    ensure!(
        manifest.packet_source == PACKET_SOURCE,
        "production manifest packet source is stale"
    );
    ensure!(
        manifest.ordering == "Service IDs and serverbound row IDs are sorted lexicographically.",
        "production manifest ordering contract is stale"
    );
    ensure!(
        manifest.stages == ALL_STAGES,
        "production manifest stage vocabulary or order is stale"
    );
    Ok(())
}

fn verify_services(workspace: &Path, services: &[ServiceRecord]) -> Result<()> {
    let expected = SERVICE_IDS.into_iter().collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    let mut previous = None;
    for service in services {
        verify_record(workspace, &service.integration)?;
        verify_order(&mut previous, &service.integration.id, "service")?;
        ensure!(
            actual.insert(service.integration.id.as_str()),
            "production service {} is duplicated",
            service.integration.id
        );
    }
    ensure!(
        actual == expected,
        "production services are stale; missing {:?}, dead {:?}",
        expected.difference(&actual).collect::<Vec<_>>(),
        actual.difference(&expected).collect::<Vec<_>>()
    );
    Ok(())
}

fn verify_serverbound(workspace: &Path, records: &[ServerboundRecord]) -> Result<()> {
    let source_path = workspace.join(PACKET_SOURCE);
    let source = fs::read_to_string(&source_path)
        .with_context(|| format!("read {}", source_path.display()))?;
    let expected = parse_packet_variants(&source)?;
    let mut actual = BTreeSet::new();
    let mut previous = None;
    for record in records {
        verify_record(workspace, &record.integration)?;
        verify_order(&mut previous, &record.integration.id, "serverbound")?;
        ensure!(
            !record.packets.is_empty(),
            "serverbound row {} owns no packets",
            record.integration.id
        );
        let mut packet_previous = None;
        for packet in &record.packets {
            verify_order(&mut packet_previous, packet, "packet")?;
            ensure!(
                actual.insert(packet.as_str()),
                "serverbound packet {packet} is mapped more than once"
            );
        }
    }
    let expected_refs = expected.iter().map(String::as_str).collect::<BTreeSet<_>>();
    ensure!(
        actual == expected_refs,
        "serverbound packet coverage is stale; missing {:?}, dead {:?}",
        expected_refs.difference(&actual).collect::<Vec<_>>(),
        actual.difference(&expected_refs).collect::<Vec<_>>()
    );
    Ok(())
}

fn verify_record(workspace: &Path, record: &IntegrationRecord) -> Result<()> {
    ensure!(
        valid_id(&record.id),
        "invalid production row ID {}",
        record.id
    );
    ensure!(
        !record.responsibility.trim().is_empty(),
        "production row {} has no responsibility",
        record.id
    );
    ensure!(
        !record.owner.trim().is_empty(),
        "production row {} has no owner",
        record.id
    );
    ensure!(
        !record.target_goals.is_empty(),
        "production row {} has no target Goal",
        record.id
    );
    let allowed_goals = TARGET_GOALS.into_iter().collect::<BTreeSet<_>>();
    let target_goals = unique_strings(&record.target_goals, "target Goal", &record.id)?;
    ensure!(
        target_goals
            .iter()
            .all(|goal| allowed_goals.contains(*goal)),
        "production row {} has an unknown target Goal",
        record.id
    );

    let implemented = unique_stages(&record.implemented_stages, "implemented", &record.id)?;
    let not_applicable =
        unique_stages(&record.not_applicable_stages, "not-applicable", &record.id)?;
    let gaps = unique_stages(&record.gaps, "gap", &record.id)?;
    ensure!(
        implemented.is_disjoint(&not_applicable)
            && implemented.is_disjoint(&gaps)
            && not_applicable.is_disjoint(&gaps),
        "production row {} assigns one stage more than once",
        record.id
    );
    let covered = implemented
        .union(&not_applicable)
        .copied()
        .chain(gaps.iter().copied())
        .collect::<BTreeSet<_>>();
    let all = ALL_STAGES.into_iter().collect::<BTreeSet<_>>();
    ensure!(
        covered == all,
        "production row {} does not classify every integration stage",
        record.id
    );

    match record.status {
        IntegrationStatus::Integrated => {
            ensure!(
                gaps.is_empty(),
                "Integrated production row {} still has gaps",
                record.id
            );
            ensure!(
                implemented.contains(&IntegrationStage::FocusedTest),
                "Integrated production row {} has no focused test stage",
                record.id
            );
        }
        IntegrationStatus::Partial => ensure!(
            !implemented.is_empty() && !gaps.is_empty(),
            "Partial production row {} must have implemented and missing stages",
            record.id
        ),
        IntegrationStatus::Planned => ensure!(
            !gaps.is_empty(),
            "Planned production row {} has no planned gaps",
            record.id
        ),
        IntegrationStatus::Unsupported => ensure!(
            gaps.is_empty() && !record.rationale.is_empty(),
            "Unsupported production row {} needs a complete explicit outcome and rationale",
            record.id
        ),
    }

    ensure!(
        !record.evidence.is_empty(),
        "production row {} has no evidence paths",
        record.id
    );
    for path in record.evidence.iter().chain(&record.tests) {
        verify_workspace_file(workspace, path, &record.id)?;
    }
    if implemented.contains(&IntegrationStage::FocusedTest) {
        ensure!(
            !record.tests.is_empty(),
            "production row {} claims focused tests without test owners",
            record.id
        );
    }
    if implemented.contains(&IntegrationStage::ClientAcceptance) {
        ensure!(
            record
                .evidence
                .iter()
                .any(|path| path.starts_with("docs/reports/goal-02/")),
            "production row {} claims client acceptance without Goal 02 evidence",
            record.id
        );
    }
    Ok(())
}

fn verify_totals(manifest: &ProductionManifest) -> Result<()> {
    let totals = &manifest.totals;
    ensure!(
        totals.service_rows == manifest.service.len(),
        "service row total is stale"
    );
    ensure!(
        totals.serverbound_rows == manifest.serverbound.len(),
        "serverbound row total is stale"
    );
    let packet_count = manifest
        .serverbound
        .iter()
        .map(|record| record.packets.len())
        .sum::<usize>();
    ensure!(
        totals.serverbound_packets == packet_count,
        "serverbound packet total is stale"
    );
    let status_counts = manifest
        .service
        .iter()
        .map(|record| record.integration.status)
        .chain(
            manifest
                .serverbound
                .iter()
                .map(|record| record.integration.status),
        )
        .fold(BTreeMap::new(), |mut counts, status| {
            *counts.entry(status).or_insert(0_usize) += 1;
            counts
        });
    for (status, expected) in [
        (IntegrationStatus::Integrated, totals.integrated_rows),
        (IntegrationStatus::Partial, totals.partial_rows),
        (IntegrationStatus::Unsupported, totals.unsupported_rows),
        (IntegrationStatus::Planned, totals.planned_rows),
    ] {
        ensure!(
            status_counts.get(&status).copied().unwrap_or_default() == expected,
            "{status:?} production row total is stale"
        );
    }
    ensure!(
        status_counts.values().sum::<usize>() == totals.service_rows + totals.serverbound_rows,
        "production status totals do not cover every row"
    );
    Ok(())
}

fn parse_packet_variants(source: &str) -> Result<BTreeSet<String>> {
    let mut in_enum = false;
    let mut variants = BTreeSet::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if !in_enum {
            if trimmed == "pub enum PlayServerboundEntryPacket {" {
                in_enum = true;
            }
            continue;
        }
        if trimmed == "}" {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#") {
            continue;
        }
        let name = trimmed
            .split(['(', '{', ','])
            .next()
            .unwrap_or_default()
            .trim();
        ensure!(
            !name.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_'),
            "invalid Play serverbound enum line: {trimmed}"
        );
        ensure!(
            variants.insert(name.to_owned()),
            "duplicate Play serverbound variant {name}"
        );
    }
    ensure!(in_enum, "PlayServerboundEntryPacket enum is missing");
    ensure!(
        !variants.is_empty(),
        "PlayServerboundEntryPacket has no variants"
    );
    Ok(variants)
}

fn unique_stages(
    values: &[IntegrationStage],
    label: &str,
    id: &str,
) -> Result<BTreeSet<IntegrationStage>> {
    let unique = values.iter().copied().collect::<BTreeSet<_>>();
    ensure!(
        unique.len() == values.len(),
        "production row {id} duplicates a {label} stage"
    );
    Ok(unique)
}

fn unique_strings<'a>(values: &'a [String], label: &str, id: &str) -> Result<BTreeSet<&'a str>> {
    let unique = values.iter().map(String::as_str).collect::<BTreeSet<_>>();
    ensure!(
        unique.len() == values.len(),
        "production row {id} duplicates a {label}"
    );
    Ok(unique)
}

fn verify_order<'a>(previous: &mut Option<&'a str>, value: &'a str, label: &str) -> Result<()> {
    if let Some(previous) = previous {
        ensure!(
            *previous < value,
            "{label} values are not strictly lexicographically sorted: {previous}, {value}"
        );
    }
    *previous = Some(value);
    Ok(())
}

fn verify_workspace_file(workspace: &Path, relative: &str, id: &str) -> Result<()> {
    let relative_path = Path::new(relative);
    ensure!(
        !relative_path.is_absolute()
            && !relative_path.components().any(|component| matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )),
        "production row {id} has unsafe evidence path {relative}"
    );
    let path = workspace.join(relative_path);
    ensure!(
        path.is_file(),
        "production row {id} evidence path does not exist: {}",
        path.display()
    );
    let resolved = path
        .canonicalize()
        .with_context(|| format!("resolve production evidence {}", path.display()))?;
    ensure!(
        resolved.starts_with(workspace),
        "production row {id} evidence escapes the workspace: {}",
        resolved.display()
    );
    Ok(())
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('/')
        && !id.ends_with('/')
        && id.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '/'
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_and_unit_serverbound_variants() {
        let source = "\
pub enum PlayServerboundEntryPacket {\n\
    Attack(Attack),\n\
    ClientTickEnd,\n\
    PlayerLoaded,\n\
}\n";
        assert_eq!(
            parse_packet_variants(source).unwrap(),
            ["Attack", "ClientTickEnd", "PlayerLoaded"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        );
    }

    #[test]
    fn stage_vocabulary_has_no_duplicates() {
        assert_eq!(
            ALL_STAGES.into_iter().collect::<BTreeSet<_>>().len(),
            ALL_STAGES.len()
        );
    }

    #[test]
    fn production_ids_are_responsibility_scoped() {
        assert!(valid_id("connection/play-installation"));
        assert!(!valid_id("Goal03/Play"));
        assert!(!valid_id("/connection/play"));
    }

    #[test]
    fn ordered_values_reject_duplicates_and_regressions() {
        let mut previous = None;
        verify_order(&mut previous, "a", "test").unwrap();
        verify_order(&mut previous, "b", "test").unwrap();
        assert!(verify_order(&mut previous, "b", "test").is_err());
        assert!(verify_order(&mut previous, "a", "test").is_err());
    }
}
