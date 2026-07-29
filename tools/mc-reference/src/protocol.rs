use crate::verification::{verify_cached_artifacts, verify_reports};
use crate::*;
use std::fmt::Write as _;

const RUNTIME_CATALOG_RELATIVE: &str =
    "crates/ferrite-protocol/reference/minecraft-java-26.2-packets.toml";
const RUNTIME_CATALOG_SCHEMA: u32 = 1;

pub(crate) fn protocol(context: &Context, command: ProtocolCommand) -> Result<()> {
    match command {
        ProtocolCommand::Inventory => protocol_inventory(context).map(|_| ()),
        ProtocolCommand::Coverage => protocol_coverage(context, false),
        ProtocolCommand::Readiness => protocol_coverage(context, true),
        ProtocolCommand::Catalog { write } => protocol_catalog(context, write),
        ProtocolCommand::Verify => protocol_verify(context),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProtocolPacket {
    state: String,
    direction: String,
    identity: String,
    protocol_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeCatalogLane {
    state: String,
    direction: String,
    identities: Vec<String>,
}

pub(crate) fn load_protocol_completion(context: &Context) -> Result<ProtocolCompletionFile> {
    let path = context.reference.join("protocol/completion.toml");
    toml::from_str(
        &fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?,
    )
    .with_context(|| format!("invalid {}", path.display()))
}

fn protocol_packets(context: &Context) -> Result<Vec<ProtocolPacket>> {
    let path = context.cache.join("generated/reports/packets.json");
    let report = read_json(&path)?;
    let mut packets = Vec::new();
    for (state, directions) in report
        .as_object()
        .context("packets.json root is not an object")?
    {
        for (direction, identities) in directions
            .as_object()
            .with_context(|| format!("packets.json state {state} is not an object"))?
        {
            let identities = identities
                .as_object()
                .with_context(|| format!("packets.json {state}/{direction} is not an object"))?;
            let mut ids = BTreeSet::new();
            for (identity, record) in identities {
                let protocol_id = record
                    .get("protocol_id")
                    .and_then(Value::as_u64)
                    .with_context(|| {
                        format!("{state}/{direction}/{identity} misses protocol_id")
                    })?;
                ensure!(
                    ids.insert(protocol_id),
                    "duplicate packet ID {protocol_id} in {state}/{direction}"
                );
                packets.push(ProtocolPacket {
                    state: state.clone(),
                    direction: direction.clone(),
                    identity: identity.clone(),
                    protocol_id,
                });
            }
            ensure!(
                ids.iter().copied().eq(0..ids.len() as u64),
                "packet IDs are not contiguous from zero in {state}/{direction}"
            );
        }
    }
    packets.sort();
    Ok(packets)
}

fn protocol_inventory(context: &Context) -> Result<Vec<ProtocolPacket>> {
    let completion = load_protocol_completion(context)?;
    ensure!(
        completion.version == context.lock.version,
        "protocol ledger version differs from lock"
    );
    let packets = protocol_packets(context)?;
    let mut bytes = Vec::new();
    let mut counts = BTreeMap::<(String, String), usize>::new();
    for packet in &packets {
        writeln!(
            bytes,
            "{}\t{}\t{}\t{}",
            packet.state, packet.direction, packet.identity, packet.protocol_id
        )?;
        *counts
            .entry((packet.state.clone(), packet.direction.clone()))
            .or_default() += 1;
    }
    ensure!(
        packets.len() == completion.inventory.expected_count,
        "protocol inventory expected {} packets, found {}",
        completion.inventory.expected_count,
        packets.len()
    );
    ensure!(
        sha1_bytes(&bytes) == completion.inventory.entries_sha1,
        "protocol inventory digest differs from completion ledger"
    );
    for ((state, direction), count) in counts {
        println!("{state:13} {direction:11} {count:3} packets");
    }
    println!(
        "protocol inventory verified: {} packets, digest {}",
        packets.len(),
        completion.inventory.entries_sha1
    );
    Ok(packets)
}

fn protocol_coverage(context: &Context, require_ready: bool) -> Result<()> {
    let completion = load_protocol_completion(context)?;
    let packets = protocol_inventory(context)?;
    ensure!(
        !completion.family.is_empty(),
        "protocol ledger has no packet families"
    );
    let mut family_ids = BTreeSet::new();
    let mut matched_family_ids = BTreeSet::new();
    for family in &completion.family {
        ensure!(
            !family.id.trim().is_empty() && family_ids.insert(&family.id),
            "duplicate or empty protocol family ID {}",
            family.id
        );
        ensure!(
            !family.owner.trim().is_empty(),
            "{} has no owner",
            family.id
        );
        ensure!(!family.evidence.is_empty(), "{} has no evidence", family.id);
        ensure!(
            !family.patterns.is_empty(),
            "{} has no packet selectors",
            family.id
        );
        if !matches!(
            family.status,
            ProtocolStatus::Todo | ProtocolStatus::InProgress
        ) {
            ensure!(
                !family.specification.is_empty()
                    && context
                        .reference
                        .join("protocol")
                        .join(&family.specification)
                        .is_file(),
                "{} references a missing protocol specification",
                family.id
            );
            ensure!(
                !family.last_commit.is_empty(),
                "{} complete conclusion has no commit",
                family.id
            );
        }
        match family.status {
            ProtocolStatus::Todo | ProtocolStatus::InProgress => {
                ensure!(
                    !family.unknowns.is_empty() && !family.reproduction.is_empty(),
                    "{} has no recoverable work description",
                    family.id
                );
            }
            ProtocolStatus::Specified => {
                ensure!(
                    family.responsibility == ProtocolResponsibility::Required,
                    "{} Specified responsibility is not Required",
                    family.id
                );
                ensure!(
                    !family.specification.is_empty()
                        && !family.fields.is_empty()
                        && !family.mappings.is_empty()
                        && !family.transitions.is_empty()
                        && !family.ordering.is_empty()
                        && !family.vectors.is_empty()
                        && !family.last_commit.is_empty(),
                    "{} is falsely complete",
                    family.id
                );
            }
            ProtocolStatus::GatedOptional => {
                ensure!(
                    family.responsibility == ProtocolResponsibility::Optional,
                    "{} optional status/responsibility disagree",
                    family.id
                );
                ensure!(
                    !family.specification.is_empty()
                        && !family.vectors.is_empty()
                        && !family.last_commit.is_empty(),
                    "{} optional path is not justified and tested",
                    family.id
                );
            }
            ProtocolStatus::NonServerResponsibility => {
                ensure!(
                    family.responsibility == ProtocolResponsibility::NonServer,
                    "{} non-server status/responsibility disagree",
                    family.id
                );
                ensure!(
                    !family.specification.is_empty()
                        && !family.mappings.is_empty()
                        && !family.last_commit.is_empty(),
                    "{} non-server path is not justified",
                    family.id
                );
            }
            ProtocolStatus::SourceInconclusive => {
                ensure!(
                    !family.specification.is_empty()
                        && !family.unknowns.is_empty()
                        && !family.reproduction.is_empty(),
                    "{} inconclusive path has no exact unknown/reproduction",
                    family.id
                );
            }
        }
    }
    for packet in &packets {
        let mut matches = Vec::new();
        for family in &completion.family {
            if family.state != packet.state || family.direction != packet.direction {
                continue;
            }
            let mut builder = GlobSetBuilder::new();
            for pattern in &family.patterns {
                builder.add(
                    Glob::new(pattern)
                        .with_context(|| format!("invalid selector in {}", family.id))?,
                );
            }
            if builder.build()?.is_match(&packet.identity) {
                matches.push(family);
            }
        }
        ensure!(
            matches.len() == 1,
            "{}/{}/{} matched {} protocol families",
            packet.state,
            packet.direction,
            packet.identity,
            matches.len()
        );
        matched_family_ids.insert(&matches[0].id);
    }
    ensure!(
        matched_family_ids == family_ids,
        "one or more protocol families match zero locked packets"
    );
    let mut statuses = BTreeMap::<ProtocolStatus, usize>::new();
    let mut levels = BTreeMap::<ProtocolLevel, usize>::new();
    for family in &completion.family {
        *statuses.entry(family.status).or_default() += 1;
        *levels.entry(family.level).or_default() += 1;
    }
    println!(
        "protocol coverage complete: {} packets in {} families; levels {:?}; statuses {:?}",
        packets.len(),
        completion.family.len(),
        levels,
        statuses
    );
    if require_ready {
        let todo = statuses.get(&ProtocolStatus::Todo).copied().unwrap_or(0);
        let in_progress = statuses
            .get(&ProtocolStatus::InProgress)
            .copied()
            .unwrap_or(0);
        ensure!(
            todo == 0,
            "protocol readiness blocked by {todo} Todo families"
        );
        ensure!(
            in_progress == 0,
            "protocol readiness blocked by {in_progress} InProgress families"
        );
        println!("mc-reference protocol readiness complete");
    }
    Ok(())
}

pub(crate) fn protocol_verify(context: &Context) -> Result<()> {
    verify_cached_artifacts(context)?;
    verify_reports(context)?;
    protocol_coverage(context, false)?;
    protocol_catalog(context, false)?;
    println!("mc-reference protocol verification complete (offline)");
    Ok(())
}

fn protocol_catalog(context: &Context, write: bool) -> Result<()> {
    let rendered = render_runtime_catalog(context)?;
    let path = context.workspace.join(RUNTIME_CATALOG_RELATIVE);
    if write {
        let parent = path
            .parent()
            .context("runtime packet catalog has no parent")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("create packet catalog directory {}", parent.display()))?;
        fs::write(&path, &rendered)
            .with_context(|| format!("write runtime packet catalog {}", path.display()))?;
        println!("wrote runtime packet catalog {}", path.display());
    } else {
        let committed = fs::read_to_string(&path)
            .with_context(|| format!("missing runtime packet catalog {}", path.display()))?;
        let committed = committed.replace("\r\n", "\n");
        ensure!(
            committed == rendered,
            "runtime packet catalog differs from locked packets report; run \
             `cargo run -p mc-reference --bin mc-ref -- protocol catalog --write`"
        );
        println!(
            "runtime packet catalog verified: {}",
            path.strip_prefix(&context.workspace)
                .unwrap_or(&path)
                .display()
        );
    }
    Ok(())
}

fn render_runtime_catalog(context: &Context) -> Result<String> {
    let completion = load_protocol_completion(context)?;
    let packets = protocol_packets(context)?;
    let mut digest_input = Vec::new();
    for packet in &packets {
        writeln!(
            digest_input,
            "{}\t{}\t{}\t{}",
            packet.state, packet.direction, packet.identity, packet.protocol_id
        )?;
    }
    ensure!(
        packets.len() == completion.inventory.expected_count,
        "runtime catalog expected {} packets, found {}",
        completion.inventory.expected_count,
        packets.len()
    );
    ensure!(
        sha1_bytes(&digest_input) == completion.inventory.entries_sha1,
        "runtime catalog source report differs from the protocol inventory digest"
    );
    let version_report = read_json(&context.cache.join("client-classes/version.json"))?;
    let protocol_version = version_report
        .get("protocol_version")
        .and_then(Value::as_u64)
        .context("client version report misses protocol_version")?;
    let mut grouped = BTreeMap::<(String, String), Vec<ProtocolPacket>>::new();
    for packet in packets {
        grouped
            .entry((packet.state.clone(), packet.direction.clone()))
            .or_default()
            .push(packet);
    }
    let mut lanes = Vec::new();
    for ((state, direction), mut packets) in grouped {
        packets.sort_by_key(|packet| packet.protocol_id);
        for (expected, packet) in packets.iter().enumerate() {
            ensure!(
                packet.protocol_id == expected as u64,
                "runtime packet lane {state}/{direction} is not contiguous"
            );
            ensure!(
                normalize_id(&packet.identity)? == packet.identity,
                "runtime packet identity is not canonical: {}",
                packet.identity
            );
        }
        lanes.push(RuntimeCatalogLane {
            state,
            direction,
            identities: packets.into_iter().map(|packet| packet.identity).collect(),
        });
    }
    render_catalog_toml(
        &completion.version,
        protocol_version,
        &completion.inventory.entries_sha1,
        &lanes,
    )
}

fn render_catalog_toml(
    minecraft_version: &str,
    protocol_version: u64,
    entries_sha1: &str,
    lanes: &[RuntimeCatalogLane],
) -> Result<String> {
    ensure!(
        minecraft_version
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.'),
        "invalid catalog Minecraft version"
    );
    ensure!(
        entries_sha1.len() == 40
            && entries_sha1
                .chars()
                .all(|character| character.is_ascii_hexdigit()),
        "invalid catalog packet digest"
    );
    let mut output = String::new();
    writeln!(output, "# ferrite-minecraft-packet-catalog-v1")?;
    writeln!(
        output,
        "# Generated from OFF-REPORT-001; do not edit packet identities by hand."
    )?;
    writeln!(output, "schema = {RUNTIME_CATALOG_SCHEMA}")?;
    writeln!(output, "minecraft_version = \"{minecraft_version}\"")?;
    writeln!(output, "protocol_version = {protocol_version}")?;
    writeln!(
        output,
        "entries_sha1 = \"{}\"",
        entries_sha1.to_ascii_lowercase()
    )?;
    for lane in lanes {
        ensure!(
            matches!(
                lane.state.as_str(),
                "configuration" | "handshake" | "login" | "play" | "status"
            ),
            "invalid runtime packet state {}",
            lane.state
        );
        ensure!(
            matches!(lane.direction.as_str(), "clientbound" | "serverbound"),
            "invalid runtime packet direction {}",
            lane.direction
        );
        writeln!(output)?;
        writeln!(output, "[[lane]]")?;
        writeln!(output, "state = \"{}\"", lane.state)?;
        writeln!(output, "direction = \"{}\"", lane.direction)?;
        writeln!(output, "identities = [")?;
        for identity in &lane.identities {
            ensure!(
                normalize_id(identity)? == *identity,
                "invalid runtime packet identity {identity}"
            );
            writeln!(output, "    \"{identity}\",")?;
        }
        writeln!(output, "]")?;
    }
    Ok(output)
}

#[cfg(test)]
mod runtime_catalog_tests {
    use crate::protocol::{RuntimeCatalogLane, render_catalog_toml};

    #[test]
    fn runtime_catalog_rendering_is_canonical() {
        let lanes = vec![RuntimeCatalogLane {
            state: "status".to_owned(),
            direction: "serverbound".to_owned(),
            identities: vec![
                "minecraft:status_request".to_owned(),
                "minecraft:ping_request".to_owned(),
            ],
        }];
        let rendered = render_catalog_toml(
            "26.2",
            776,
            "f34b0956b6399c749d4638cd6d3c9226685f41fa",
            &lanes,
        )
        .unwrap();
        assert!(rendered.starts_with("# ferrite-minecraft-packet-catalog-v1\n"));
        assert!(rendered.contains("protocol_version = 776\n"));
        assert!(
            rendered.find("minecraft:status_request").unwrap()
                < rendered.find("minecraft:ping_request").unwrap()
        );
    }
}
