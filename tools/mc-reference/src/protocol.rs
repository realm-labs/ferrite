use crate::verification::{verify_cached_artifacts, verify_reports};
use crate::*;

pub(crate) fn protocol(context: &Context, command: ProtocolCommand) -> Result<()> {
    match command {
        ProtocolCommand::Inventory => protocol_inventory(context).map(|_| ()),
        ProtocolCommand::Coverage => protocol_coverage(context, false),
        ProtocolCommand::Readiness => protocol_coverage(context, true),
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
    println!("mc-reference protocol verification complete (offline)");
    Ok(())
}
