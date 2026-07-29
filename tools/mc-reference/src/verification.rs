use crate::artifact::{get, manifest_metadata_is_current, verify_file};
use crate::catalog::{
    classify_in_category, compile_family_selectors, coverage, load_category_ids,
    validate_family_selectors,
};
use crate::experiments::experiments;
use crate::protocol::protocol_verify;
use crate::surface::surface_coverage;
use crate::symbols::symbols;
use crate::*;

pub(crate) fn verify(context: &Context, offline: bool) -> Result<()> {
    if !offline {
        let client = Client::builder()
            .user_agent("Ferrite mc-reference/0.1")
            .build()?;
        let manifest: Manifest =
            serde_json::from_slice(&get(&client, &context.lock.manifest_url)?)?;
        let metadata_is_current =
            manifest_metadata_is_current(&manifest, &context.lock.version, &context.lock.metadata)?;
        if metadata_is_current {
            println!("official manifest version and metadata pointer verified");
        } else {
            println!(
                "official manifest version verified; live metadata pointer has moved beyond the lock"
            );
        }
    }
    verify_cached_artifacts(context)?;
    verify_reports(context)?;
    validate_docs(context)?;
    validate_completion(context, false)?;
    symbols(context)?;
    coverage(context)?;
    experiments(context, ExperimentCommand::Verify)?;
    protocol_verify(context)?;
    surface_coverage(context, false)?;
    hygiene(context)?;
    implementation_manifest::run(context, ImplementationManifestCommand::Verify)?;
    println!(
        "mc-reference verification complete ({})",
        if offline { "offline" } else { "online" }
    );
    Ok(())
}

pub(crate) fn readiness(context: &Context) -> Result<()> {
    let completion_error = validate_completion(context, true).err();
    let surface_error = surface_coverage(context, true).err();
    match (completion_error, surface_error) {
        (None, None) => Ok(()),
        (Some(completion), None) => {
            bail!("gameplay readiness blocked: {completion:#}")
        }
        (None, Some(surface)) => {
            bail!("gameplay readiness blocked: {surface:#}")
        }
        (Some(completion), Some(surface)) => bail!(
            "gameplay readiness blocked by both ledgers:\n- completion: {completion:#}\n- surfaces: {surface:#}"
        ),
    }
}

pub(crate) fn completion_slice_has_ownership(slice: &CompletionSlice) -> bool {
    !slice.id.trim().is_empty()
        && !slice.subsystem.trim().is_empty()
        && !slice.parents.is_empty()
        && !slice.leaves.is_empty()
        && !slice.selectors.is_empty()
        && (!slice.symbols.is_empty() || !slice.data_paths.is_empty())
}

fn validate_completion(context: &Context, require_complete: bool) -> Result<()> {
    let completion: CompletionFile = toml::from_str(&fs::read_to_string(
        context.reference.join("completion.toml"),
    )?)?;
    ensure!(
        completion.version == context.lock.version,
        "completion ledger targets {}, expected {}",
        completion.version,
        context.lock.version
    );

    let parent_regex = Regex::new(r"(?m)^## `([A-Z][A-Z0-9-]+)`")?;
    let leaf_regex = Regex::new(r"(?m)^## Leaf rule `([A-Z][A-Z0-9-]+)`")?;
    let mut parents = BTreeSet::new();
    let mut leaves = BTreeSet::new();
    for file in markdown_files(&context.reference) {
        let text = fs::read_to_string(file)?;
        parents.extend(
            parent_regex
                .captures_iter(&text)
                .map(|capture| capture[1].to_string()),
        );
        leaves.extend(
            leaf_regex
                .captures_iter(&text)
                .map(|capture| capture[1].to_string()),
        );
    }
    let experiments: BTreeSet<_> = load_experiments(context)?
        .into_iter()
        .map(|experiment| experiment.id)
        .collect();

    let mut slice_ids = BTreeSet::new();
    let mut covered_parents = BTreeSet::new();
    let mut covered_leaves = BTreeSet::new();
    let mut statuses = BTreeMap::<CompletionStatus, usize>::new();
    for slice in &completion.slice {
        ensure!(
            slice_ids.insert(&slice.id),
            "duplicate completion slice {}",
            slice.id
        );
        ensure!(
            completion_slice_has_ownership(slice),
            "completion slice {} has incomplete ownership fields",
            slice.id
        );
        ensure!(
            slice
                .registry_kinds
                .iter()
                .all(|registry| !registry.trim().is_empty()),
            "completion slice {} has an empty registry kind",
            slice.id
        );
        for parent in &slice.parents {
            ensure!(
                parents.contains(parent),
                "completion slice {} references unknown parent {parent}",
                slice.id
            );
            covered_parents.insert(parent.clone());
        }
        for leaf in &slice.leaves {
            ensure!(
                leaves.contains(leaf),
                "completion slice {} references unknown leaf {leaf}",
                slice.id
            );
            ensure!(
                covered_leaves.insert(leaf.clone()),
                "leaf {leaf} is owned by multiple completion slices"
            );
        }
        for experiment in &slice.experiments {
            ensure!(
                experiments.contains(experiment),
                "completion slice {} references unknown experiment {experiment}",
                slice.id
            );
        }
        if matches!(
            slice.status,
            CompletionStatus::SourceSpecified
                | CompletionStatus::DataOnlyVerified
                | CompletionStatus::SourceInconclusive
        ) {
            ensure!(
                !slice.last_commit.trim().is_empty(),
                "completed slice {} has no last_commit",
                slice.id
            );
        }
        if slice.status == CompletionStatus::SourceInconclusive {
            ensure!(
                !slice.unknowns.is_empty() && !slice.reproduction.is_empty(),
                "SourceInconclusive slice {} needs exact unknowns and reproduction",
                slice.id
            );
        }
        *statuses.entry(slice.status).or_default() += 1;
    }
    ensure!(
        covered_parents == parents,
        "completion parent coverage differs: missing {:?}",
        parents.difference(&covered_parents).collect::<Vec<_>>()
    );
    ensure!(
        covered_leaves == leaves,
        "completion leaf coverage differs: missing {:?}",
        leaves.difference(&covered_leaves).collect::<Vec<_>>()
    );

    let official = read_json(&context.cache.join("generated/reports/registries.json"))?
        .as_object()
        .context("registries.json is not an object")?
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut scoped = BTreeSet::new();
    for registry in &completion.registry {
        ensure!(
            scoped.insert(registry.id.clone()),
            "duplicate registry scope {}",
            registry.id
        );
        ensure!(
            !registry.reason.trim().is_empty(),
            "registry {} has no scope reason",
            registry.id
        );
        let _scope = &registry.scope;
    }
    ensure!(
        scoped == official,
        "registry scope differs: missing {:?}, stale {:?}",
        official.difference(&scoped).collect::<Vec<_>>(),
        scoped.difference(&official).collect::<Vec<_>>()
    );

    let catalog = load_catalog(context)?;
    let blocks = load_category_ids(context, "block")?;
    let mut unreviewed = 0;
    for category in &catalog.category {
        let selectors = compile_family_selectors(category)?;
        let ids = load_category_ids(context, &category.kind)?;
        validate_family_selectors(category, &ids, &blocks)?;
        for id in &ids {
            if classify_in_category(category, &selectors, id, Some(&blocks))?
                .family
                .classification
                == Classification::Unreviewed
            {
                unreviewed += 1;
            }
        }
    }

    let todo = statuses.get(&CompletionStatus::Todo).copied().unwrap_or(0);
    let in_progress = statuses
        .get(&CompletionStatus::InProgress)
        .copied()
        .unwrap_or(0);
    let source_specified = statuses
        .get(&CompletionStatus::SourceSpecified)
        .copied()
        .unwrap_or(0);
    let data_only = statuses
        .get(&CompletionStatus::DataOnlyVerified)
        .copied()
        .unwrap_or(0);
    let source_inconclusive = statuses
        .get(&CompletionStatus::SourceInconclusive)
        .copied()
        .unwrap_or(0);
    println!(
        "readiness ledger: {} slices (Todo {todo}, InProgress {in_progress}, SourceSpecified {source_specified}, DataOnlyVerified {data_only}, SourceInconclusive {source_inconclusive}), {} parent rules, {} leaf rules, {} registries; {unreviewed} unreviewed catalog IDs",
        completion.slice.len(),
        parents.len(),
        leaves.len(),
        scoped.len()
    );
    if require_complete {
        ensure!(todo == 0, "readiness blocked by {todo} Todo slices");
        ensure!(
            in_progress == 0,
            "readiness blocked by {in_progress} InProgress slices"
        );
        ensure!(
            unreviewed == 0,
            "readiness blocked by {unreviewed} unreviewed catalog IDs"
        );
        println!("mc-reference source readiness complete");
    } else {
        println!("completion ledger consistency verified");
    }
    Ok(())
}

pub(crate) fn verify_cached_artifacts(context: &Context) -> Result<()> {
    verify_file(
        &context.cache.join("version.json"),
        &context.lock.metadata.sha1,
        context.lock.metadata.size,
    )?;
    verify_file(
        &context.cache.join("client.jar"),
        &context.lock.client.sha1,
        context.lock.client.size,
    )?;
    verify_file(
        &context.cache.join("server.jar"),
        &context.lock.server.sha1,
        context.lock.server.size,
    )?;
    Ok(())
}

pub(crate) fn verify_reports(context: &Context) -> Result<()> {
    for file in ["blocks.json", "registries.json", "commands.json"] {
        ensure!(
            context.cache.join("generated/reports").join(file).is_file(),
            "missing generated report {file}"
        );
    }
    Ok(())
}

fn validate_docs(context: &Context) -> Result<()> {
    let rule_regex = Regex::new(r"(?m)^## `([A-Z][A-Z0-9-]+)`")?;
    let leaf_regex = Regex::new(r"(?m)^## Leaf rule `([A-Z][A-Z0-9-]+)`")?;
    let link_regex = Regex::new(r"\]\(([^)]+)\)")?;
    let parent_reference_regex = Regex::new(r"`([A-Z]+-\d+)`")?;
    let required = [
        "Parent",
        "FidelityClass",
        "EvidenceStatus",
        "SourceConclusion",
        "Applies when",
        "Authoritative state",
        "Transition and ordering",
        "Branches and aborts",
        "Constants and randomness",
        "Side effects",
        "Gates",
        "Boundary cases and quirks",
        "Evidence",
        "Test vectors",
    ];
    let mut ids = BTreeSet::new();
    let mut parent_ids = BTreeSet::new();
    let mut referenced_parents = BTreeSet::new();
    let mut leaves = 0;
    for file in markdown_files(&context.reference) {
        let text = fs::read_to_string(&file)?;
        for captures in rule_regex.captures_iter(&text) {
            parent_ids.insert(captures[1].to_string());
            ensure!(
                ids.insert(captures[1].to_string()),
                "duplicate rule ID {}",
                &captures[1]
            );
        }
        for captures in leaf_regex.captures_iter(&text) {
            ensure!(
                ids.insert(captures[1].to_string()),
                "duplicate rule ID {}",
                &captures[1]
            );
            leaves += 1;
            let start = captures.get(0).unwrap().start();
            let end = text[start + 1..]
                .find("\n## ")
                .map(|v| start + 1 + v)
                .unwrap_or(text.len());
            let section = &text[start..end];
            for field in required {
                ensure!(
                    section.contains(&format!("**{field}:**")),
                    "{} in {} misses {field}",
                    &captures[1],
                    file.display()
                );
            }
        }
        for line in text.lines().filter(|line| line.starts_with("**Parent:**")) {
            for captures in parent_reference_regex.captures_iter(line) {
                referenced_parents.insert(captures[1].to_string());
            }
        }
        for captures in link_regex.captures_iter(&text) {
            let link = captures[1].trim().trim_matches(['<', '>']);
            if link.starts_with("https://") || link.starts_with("http://") {
                reqwest::Url::parse(link).with_context(|| {
                    format!("invalid external link {link} in {}", file.display())
                })?;
                if link.contains("minecraft.wiki/") {
                    ensure!(
                        link.contains("oldid="),
                        "community Wiki link is not revision-pinned in {}: {link}",
                        file.display()
                    );
                }
                continue;
            }
            if link.starts_with('#') || link.starts_with("mailto:") {
                continue;
            }
            let target = link.split('#').next().unwrap_or(link);
            if target.is_empty() {
                continue;
            }
            let resolved = file.parent().unwrap_or(&context.reference).join(target);
            let exists = resolved.is_file() || resolved.join("README.md").is_file();
            ensure!(exists, "broken internal link in {}: {link}", file.display());
        }
    }
    ensure!(parent_ids.len() == 65, "expected 65 stable parent rules");
    ensure!(
        referenced_parents == parent_ids,
        "leaf parent coverage differs: missing {:?}, unknown {:?}",
        parent_ids
            .difference(&referenced_parents)
            .collect::<Vec<_>>(),
        referenced_parents
            .difference(&parent_ids)
            .collect::<Vec<_>>()
    );
    ensure!(leaves > 0, "no leaf rules found");
    println!(
        "documentation schema verified: {} IDs including {} leaf rules",
        ids.len(),
        leaves
    );
    Ok(())
}

pub(crate) fn validate_rule_references(context: &Context, catalog: &Catalog) -> Result<()> {
    let ids = documented_rule_ids(context)?;
    for category in &catalog.category {
        ensure!(
            !category.family.is_empty(),
            "{} has no behavior families",
            category.kind
        );
        let remaining = category
            .family
            .iter()
            .filter(|family| family.remaining)
            .count();
        ensure!(
            remaining <= 1,
            "{} has multiple remaining families",
            category.kind
        );
        for family in &category.family {
            ensure!(
                !family.rules.is_empty(),
                "{}/{} has no rule references",
                category.kind,
                family.name
            );
            for rule in &family.rules {
                ensure!(
                    ids.contains(rule),
                    "{}/{} references missing rule {rule}",
                    category.kind,
                    family.name
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn documented_rule_ids(context: &Context) -> Result<BTreeSet<String>> {
    let regex = Regex::new(r"`([A-Z]{2,}(?:-[A-Z0-9]+)+)`")?;
    let mut ids = BTreeSet::new();
    for file in markdown_files(&context.reference) {
        let text = fs::read_to_string(file)?;
        for captures in regex.captures_iter(&text) {
            if captures[1].starts_with("EXP-") {
                continue;
            }
            ids.insert(captures[1].to_string());
        }
    }
    Ok(ids)
}

pub(crate) fn load_catalog(context: &Context) -> Result<Catalog> {
    Ok(toml::from_str(&fs::read_to_string(
        context.reference.join("catalog/catalog.toml"),
    )?)?)
}

pub(crate) fn load_experiments(context: &Context) -> Result<Vec<Experiment>> {
    let mut experiments = Vec::new();
    for entry in fs::read_dir(context.reference.join("experiments"))? {
        let path = entry?.path();
        if path.extension().and_then(|v| v.to_str()) == Some("toml") {
            experiments
                .extend(toml::from_str::<ExperimentFile>(&fs::read_to_string(path)?)?.experiment);
        }
    }
    Ok(experiments)
}

fn hygiene(context: &Context) -> Result<()> {
    let forbidden_extensions = ["jar", "class", "mca", "mcr"];
    for entry in WalkDir::new(&context.workspace)
        .into_iter()
        .filter_entry(|entry| entry.file_name() != "target")
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            let path = entry.path();
            if path
                .components()
                .any(|component| component.as_os_str() == ".git")
            {
                continue;
            }
            let extension = path.extension().and_then(|v| v.to_str()).unwrap_or("");
            ensure!(
                !forbidden_extensions.contains(&extension),
                "forbidden generated artifact in repository: {}",
                path.display()
            );
        }
    }
    Ok(())
}
