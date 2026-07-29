use crate::protocol::load_protocol_completion;
use crate::verification::{documented_rule_ids, verify_cached_artifacts, verify_reports};
use crate::*;

pub(crate) fn surfaces(context: &Context, command: SurfaceCommand) -> Result<()> {
    match command {
        SurfaceCommand::Coverage => surface_coverage(context, false),
        SurfaceCommand::Readiness => surface_coverage(context, true),
        SurfaceCommand::Verify => surface_verify(context),
    }
}

fn load_behavior_surfaces(context: &Context) -> Result<BehaviorSurfaceFile> {
    let path = context.reference.join("behavior-surfaces.toml");
    toml::from_str(
        &fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?,
    )
    .with_context(|| format!("invalid {}", path.display()))
}

pub(crate) fn expected_surface_kinds() -> BTreeSet<BehaviorSurfaceKind> {
    BTreeSet::from([
        BehaviorSurfaceKind::TickScheduler,
        BehaviorSurfaceKind::NetworkIngress,
        BehaviorSurfaceKind::CommandAdministration,
        BehaviorSurfaceKind::ContentDispatch,
        BehaviorSurfaceKind::PlayerLifecycle,
        BehaviorSurfaceKind::WorldLifecycle,
        BehaviorSurfaceKind::PersistenceReload,
        BehaviorSurfaceKind::ClientProjection,
        BehaviorSurfaceKind::DataReload,
        BehaviorSurfaceKind::CrossSystemOrdering,
    ])
}

pub(crate) fn surface_coverage(context: &Context, require_ready: bool) -> Result<()> {
    let ledger = load_behavior_surfaces(context)?;
    ensure!(
        ledger.version == context.lock.version,
        "behavior-surface ledger targets {}, expected {}",
        ledger.version,
        context.lock.version
    );
    ensure!(
        !ledger.surface.is_empty(),
        "behavior-surface ledger is empty"
    );

    let rules = documented_rule_ids(context)?;
    let protocol = load_protocol_completion(context)?;
    let protocol_families = protocol
        .family
        .iter()
        .map(|family| family.id.as_str())
        .collect::<BTreeSet<_>>();
    let id_regex = Regex::new(r"^SURFACE-[A-Z0-9-]+-[0-9]{3}$")?;
    let mut ids = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    let mut statuses = BTreeMap::<BehaviorSurfaceStatus, usize>::new();
    let mut command_surface_status = None;
    let mut network_ingress_families = None;
    let mut cross_system_surface_status = None;

    for surface in &ledger.surface {
        ensure!(
            id_regex.is_match(&surface.id) && ids.insert(&surface.id),
            "duplicate or invalid behavior-surface ID {}",
            surface.id
        );
        ensure!(
            kinds.insert(surface.kind),
            "duplicate behavior-surface kind {:?}",
            surface.kind
        );
        if surface.kind == BehaviorSurfaceKind::CommandAdministration {
            command_surface_status = Some(surface.status);
        }
        if surface.kind == BehaviorSurfaceKind::NetworkIngress {
            network_ingress_families = Some(
                surface
                    .protocol_families
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
            );
        }
        if surface.kind == BehaviorSurfaceKind::CrossSystemOrdering {
            cross_system_surface_status = Some(surface.status);
        }
        ensure!(
            !surface.boundary.trim().is_empty()
                && !surface.triggers.is_empty()
                && !surface.inventory_sources.is_empty()
                && !surface.selectors.is_empty()
                && !surface.owners.is_empty()
                && !surface.state_domains.is_empty()
                && !surface.persistence.is_empty()
                && !surface.client_projection.is_empty()
                && !surface.evidence.is_empty(),
            "{} has an incomplete ownership boundary",
            surface.id
        );
        ensure!(
            surface
                .triggers
                .iter()
                .chain(&surface.selectors)
                .chain(&surface.state_domains)
                .chain(&surface.persistence)
                .chain(&surface.client_projection)
                .chain(&surface.evidence)
                .all(|value| !value.trim().is_empty()),
            "{} contains an empty boundary field",
            surface.id
        );
        for owner in &surface.owners {
            ensure!(
                rules.contains(owner),
                "{} references missing rule owner {owner}",
                surface.id
            );
        }
        for family in &surface.protocol_families {
            ensure!(
                protocol_families.contains(family.as_str()),
                "{} references missing protocol family {family}",
                surface.id
            );
        }
        match surface.status {
            BehaviorSurfaceStatus::Todo | BehaviorSurfaceStatus::InProgress => ensure!(
                !surface.unknowns.is_empty() && !surface.reproduction.is_empty(),
                "{} has no recoverable work description",
                surface.id
            ),
            BehaviorSurfaceStatus::Mapped => ensure!(
                surface.unknowns.is_empty()
                    && !surface.reproduction.is_empty()
                    && !surface.last_commit.trim().is_empty(),
                "{} is falsely mapped",
                surface.id
            ),
            BehaviorSurfaceStatus::SourceInconclusive => ensure!(
                !surface.unknowns.is_empty()
                    && !surface.reproduction.is_empty()
                    && !surface.last_commit.trim().is_empty(),
                "{} has no exact unknown, reproduction, or last conclusion",
                surface.id
            ),
        }
        *statuses.entry(surface.status).or_default() += 1;
    }

    ensure!(
        kinds == expected_surface_kinds(),
        "behavior-surface kinds differ from the required root inventory"
    );
    let expected_serverbound = protocol
        .family
        .iter()
        .filter(|family| family.direction == "serverbound")
        .map(|family| family.id.clone())
        .collect::<BTreeSet<_>>();
    validate_exact_protocol_family_partition(
        network_ingress_families
            .as_ref()
            .context("missing NetworkIngress protocol-family inventory")?,
        &expected_serverbound,
        "NetworkIngress",
    )?;
    let command_statuses = validate_command_roots(context, &rules)?;
    if command_surface_status == Some(BehaviorSurfaceStatus::Mapped) {
        ensure!(
            command_statuses.len() == 1
                && command_statuses.contains_key(&CommandRootStatus::Mapped),
            "CommandAdministration is falsely mapped while command-root work remains"
        );
    }
    let join_statuses = validate_cross_system_joins(context, &kinds, &rules)?;
    if cross_system_surface_status == Some(BehaviorSurfaceStatus::Mapped) {
        ensure!(
            !join_statuses.contains_key(&CrossSystemJoinStatus::InProgress),
            "CrossSystemOrdering is falsely mapped while join work remains"
        );
    }
    println!(
        "behavior-surface coverage complete: {} root surfaces; statuses {:?}",
        ledger.surface.len(),
        statuses
    );
    if require_ready {
        let todo = statuses
            .get(&BehaviorSurfaceStatus::Todo)
            .copied()
            .unwrap_or(0);
        let in_progress = statuses
            .get(&BehaviorSurfaceStatus::InProgress)
            .copied()
            .unwrap_or(0);
        ensure!(
            todo == 0 && in_progress == 0,
            "behavior-surface readiness blocked by {todo} Todo and {in_progress} InProgress roots"
        );
        println!("mc-reference behavior-surface readiness complete");
    }
    Ok(())
}

pub(crate) fn validate_exact_protocol_family_partition(
    actual: &BTreeSet<String>,
    expected: &BTreeSet<String>,
    label: &str,
) -> Result<()> {
    ensure!(
        actual == expected,
        "{label} protocol-family coverage differs: missing {:?}, extra {:?}",
        expected.difference(actual).collect::<Vec<_>>(),
        actual.difference(expected).collect::<Vec<_>>()
    );
    Ok(())
}

fn validate_cross_system_joins(
    context: &Context,
    surface_kinds: &BTreeSet<BehaviorSurfaceKind>,
    rules: &BTreeSet<String>,
) -> Result<BTreeMap<CrossSystemJoinStatus, usize>> {
    let path = context.reference.join("cross-system-joins.toml");
    let map: CrossSystemJoinMap = toml::from_str(
        &fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?,
    )
    .with_context(|| format!("invalid {}", path.display()))?;
    ensure!(
        map.version == context.lock.version,
        "cross-system join map targets {}, expected {}",
        map.version,
        context.lock.version
    );
    let statuses = validate_cross_system_join_map(&map, surface_kinds, rules)?;
    println!(
        "cross-system join matrix mapped: {} unordered pairs; statuses {:?}",
        map.join.len(),
        statuses
    );
    Ok(statuses)
}

pub(crate) fn validate_cross_system_join_map(
    map: &CrossSystemJoinMap,
    surface_kinds: &BTreeSet<BehaviorSurfaceKind>,
    rules: &BTreeSet<String>,
) -> Result<BTreeMap<CrossSystemJoinStatus, usize>> {
    let roots = surface_kinds
        .iter()
        .copied()
        .filter(|kind| *kind != BehaviorSurfaceKind::CrossSystemOrdering)
        .collect::<Vec<_>>();
    let mut expected = BTreeSet::new();
    for (index, left) in roots.iter().enumerate() {
        for right in roots.iter().skip(index + 1) {
            expected.insert((*left, *right));
        }
    }

    let mut actual = BTreeSet::new();
    let mut statuses = BTreeMap::new();
    for join in &map.join {
        ensure!(
            join.left < join.right,
            "cross-system join {:?}/{:?} is not in canonical order",
            join.left,
            join.right
        );
        ensure!(
            actual.insert((join.left, join.right)),
            "duplicate cross-system join {:?}/{:?}",
            join.left,
            join.right
        );
        match join.status {
            CrossSystemJoinStatus::Empty => ensure!(
                join.shared_domains.is_empty()
                    && join.owners.is_empty()
                    && join.remaining_work.is_empty(),
                "empty cross-system join {:?}/{:?} has ownership claims",
                join.left,
                join.right
            ),
            CrossSystemJoinStatus::InProgress | CrossSystemJoinStatus::SourceInconclusive => {
                ensure!(
                    !join.shared_domains.is_empty()
                        && !join.owners.is_empty()
                        && !join.remaining_work.is_empty(),
                    "cross-system join {:?}/{:?} has no recoverable ownership",
                    join.left,
                    join.right
                );
            }
            CrossSystemJoinStatus::Mapped => ensure!(
                !join.shared_domains.is_empty()
                    && !join.owners.is_empty()
                    && join.remaining_work.is_empty(),
                "cross-system join {:?}/{:?} is falsely mapped",
                join.left,
                join.right
            ),
        }
        for owner in &join.owners {
            ensure!(
                rules.contains(owner),
                "cross-system join {:?}/{:?} references missing owner {owner}",
                join.left,
                join.right
            );
        }
        *statuses.entry(join.status).or_default() += 1;
    }
    ensure!(
        actual == expected,
        "cross-system pair coverage differs: missing {:?}, extra {:?}",
        expected.difference(&actual).collect::<Vec<_>>(),
        actual.difference(&expected).collect::<Vec<_>>()
    );
    Ok(statuses)
}

fn validate_command_roots(
    context: &Context,
    rules: &BTreeSet<String>,
) -> Result<BTreeMap<CommandRootStatus, usize>> {
    let path = context.reference.join("command-roots.toml");
    let map: CommandRootMap = toml::from_str(
        &fs::read_to_string(&path).with_context(|| format!("missing {}", path.display()))?,
    )
    .with_context(|| format!("invalid {}", path.display()))?;
    ensure!(
        map.version == context.lock.version,
        "command-root map targets {}, expected {}",
        map.version,
        context.lock.version
    );
    let report = read_json(&context.cache.join("generated/reports/commands.json"))?;
    let official = report
        .get("children")
        .and_then(Value::as_object)
        .context("commands.json root has no children object")?
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let (executable_paths, redirect_paths) = command_report_paths(&report)?;
    ensure!(
        executable_paths.len() == map.inventory.expected_executable_count,
        "command executable count {} differs from lock {}",
        executable_paths.len(),
        map.inventory.expected_executable_count
    );
    ensure!(
        ids_digest(&executable_paths) == map.inventory.executable_paths_sha1,
        "command executable-path digest differs from lock"
    );
    ensure!(
        redirect_paths.len() == map.inventory.expected_redirect_count,
        "command redirect count {} differs from lock {}",
        redirect_paths.len(),
        map.inventory.expected_redirect_count
    );
    ensure!(
        ids_digest(&redirect_paths) == map.inventory.redirect_paths_sha1,
        "command redirect-path digest differs from lock"
    );
    validate_command_root_map(&map, &official, rules)?;
    let mut statuses = BTreeMap::<CommandRootStatus, usize>::new();
    for family in &map.family {
        *statuses.entry(family.status).or_default() += 1;
    }
    println!(
        "command-root inventory mapped: {} roots in {} recoverable families; statuses {:?}",
        official.len(),
        map.family.len(),
        statuses
    );
    Ok(statuses)
}

pub(crate) fn command_report_paths(report: &Value) -> Result<(BTreeSet<String>, BTreeSet<String>)> {
    fn visit(
        node: &Value,
        path: &mut Vec<String>,
        executable_paths: &mut BTreeSet<String>,
        redirect_paths: &mut BTreeSet<String>,
    ) -> Result<()> {
        let object = node
            .as_object()
            .with_context(|| format!("command node {} is not an object", path.join(" ")))?;
        if object.get("executable").and_then(Value::as_bool) == Some(true) {
            executable_paths.insert(path.join(" "));
        }
        if let Some(redirect) = object.get("redirect") {
            let target = redirect
                .as_array()
                .context("command redirect is not an array")?
                .iter()
                .map(|segment| {
                    segment
                        .as_str()
                        .map(str::to_owned)
                        .context("command redirect segment is not a string")
                })
                .collect::<Result<Vec<_>>>()?;
            redirect_paths.insert(format!("{} -> {}", path.join(" "), target.join(" ")));
        }
        if let Some(children) = object.get("children") {
            for (name, child) in children
                .as_object()
                .context("command children is not an object")?
            {
                path.push(name.clone());
                visit(child, path, executable_paths, redirect_paths)?;
                path.pop();
            }
        }
        Ok(())
    }

    let roots = report
        .get("children")
        .and_then(Value::as_object)
        .context("commands.json root has no children object")?;
    let mut executable_paths = BTreeSet::new();
    let mut redirect_paths = BTreeSet::new();
    for (root, node) in roots {
        visit(
            node,
            &mut vec![root.clone()],
            &mut executable_paths,
            &mut redirect_paths,
        )?;
    }
    Ok((executable_paths, redirect_paths))
}

pub(crate) fn validate_command_root_map(
    map: &CommandRootMap,
    official: &BTreeSet<String>,
    rules: &BTreeSet<String>,
) -> Result<()> {
    ensure!(
        official.len() == map.inventory.expected_count,
        "command-root count {} differs from lock {}",
        official.len(),
        map.inventory.expected_count
    );
    ensure!(
        ids_digest(official) == map.inventory.roots_sha1,
        "command-root digest differs from lock"
    );

    let mut family_names = BTreeSet::new();
    let mut mapped = BTreeSet::new();
    for family in &map.family {
        ensure!(
            !family.name.trim().is_empty() && family_names.insert(&family.name),
            "duplicate or empty command-root family {}",
            family.name
        );
        ensure!(
            !family.roots.is_empty()
                && !family.owners.is_empty()
                && !family.state_domains.is_empty(),
            "command-root family {} has incomplete ownership",
            family.name
        );
        match family.status {
            CommandRootStatus::InProgress | CommandRootStatus::SourceInconclusive => ensure!(
                !family.remaining_work.is_empty(),
                "command-root family {} has no recoverable work",
                family.name
            ),
            CommandRootStatus::Mapped => ensure!(
                family.remaining_work.is_empty(),
                "command-root family {} is falsely mapped",
                family.name
            ),
        }
        for owner in &family.owners {
            ensure!(
                rules.contains(owner),
                "command-root family {} references missing rule owner {owner}",
                family.name
            );
        }
        for root in &family.roots {
            ensure!(
                official.contains(root),
                "command-root family {} contains stale root {root}",
                family.name
            );
            ensure!(
                mapped.insert(root.clone()),
                "command root {root} belongs to multiple families"
            );
        }
    }
    ensure!(
        mapped.iter().eq(official.iter()),
        "command-root coverage differs: missing {:?}",
        official.difference(&mapped).collect::<Vec<_>>()
    );
    Ok(())
}

fn surface_verify(context: &Context) -> Result<()> {
    verify_cached_artifacts(context)?;
    verify_reports(context)?;
    surface_coverage(context, false)?;
    println!("mc-reference behavior-surface verification complete (offline)");
    Ok(())
}
