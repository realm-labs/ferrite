use crate::content::artifact::{self, sha1_lines};
use crate::content::classify::Classifier;
use crate::content::model::{BundleLock, Catalog, ReferenceLock};
use crate::content::source::CatalogSources;
use anyhow::{Context as _, Result, ensure};
use ferrite_foundation::resource::ResourceId;
use ferrite_registry::bundle::{
    BundleEntry, BundleRegistry, BundleSchemaVersion, CatalogClassification, CatalogFamily,
    ContentBundle, FamilyName, Sha1Digest, SourceArtifact,
};
use ferrite_registry::digest::ContentDigest;
use ferrite_registry::provenance::{ContentProvenance, ProvenanceKind};
use ferrite_registry::registry::{PersistentId, RegistryName};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const REFERENCE_LOCK: &str = "docs/reference/minecraft-java-26.2/lock.toml";
const CATALOG: &str = "docs/reference/minecraft-java-26.2/catalog/catalog.toml";
pub(crate) const BUNDLE_LOCK: &str = "docs/reference/minecraft-java-26.2/content-bundle.lock.toml";

pub(crate) struct ImportedBundle {
    pub(crate) bundle: ContentBundle,
    pub(crate) registries: usize,
    pub(crate) entries: usize,
}

pub(crate) fn import(workspace: &Path, source: &Path) -> Result<ImportedBundle> {
    let lock_bytes = fs::read(workspace.join(REFERENCE_LOCK)).context("read Minecraft lock")?;
    let lock: ReferenceLock =
        toml::from_str(std::str::from_utf8(&lock_bytes).context("Minecraft lock is not UTF-8")?)
            .context("parse Minecraft lock")?;
    let catalog: Catalog = toml::from_str(
        &fs::read_to_string(workspace.join(CATALOG)).context("read catalog contract")?,
    )
    .context("parse catalog contract")?;

    let client = artifact::verify("client", &source.join("client.jar"), &lock.client)?;
    let server = artifact::verify("server", &source.join("server.jar"), &lock.server)?;
    let source_digest = server.content_digest();
    let artifacts = vec![client, server];
    let sources = CatalogSources::open(source, &lock.version)?;
    let block_ids = sources.block_ids()?.into_iter().collect::<BTreeSet<_>>();
    let provider = ResourceId::new("ferrite", "import/minecraft_java_26_2")?;
    let provenance = ContentProvenance::new(
        ProvenanceKind::LocalOfficialArtifact,
        provider,
        lock.version.clone(),
        source_digest,
    )?;

    let mut kinds = BTreeSet::new();
    let mut registries = Vec::with_capacity(catalog.category.len());
    let mut total_entries = 0_usize;
    for category in &catalog.category {
        ensure!(
            kinds.insert(category.kind.clone()),
            "duplicate catalog category {}",
            category.kind
        );
        let values = sources
            .load(&category.kind)
            .with_context(|| format!("load {} source values", category.kind))?;
        let ids = values.keys().cloned().collect::<BTreeSet<_>>();
        ensure!(
            ids.len() == category.expected_count,
            "{} count drift: expected {}, got {}",
            category.kind,
            category.expected_count,
            ids.len()
        );
        let actual_ids_sha1 = sha1_lines(ids.iter().map(String::as_str));
        ensure!(
            actual_ids_sha1 == category.ids_sha1,
            "{} identity digest drift: expected {}, got {actual_ids_sha1}",
            category.kind,
            category.ids_sha1
        );

        let classifier = Classifier::compile(category, &ids, &block_ids)?;
        let families = category
            .family
            .iter()
            .map(|family| {
                CatalogFamily::new(
                    FamilyName::new(&family.name)?,
                    family.classification,
                    family.rules.clone(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut entries = Vec::with_capacity(values.len());
        for (id, value) in values {
            let family = classifier.classify(&id, &block_ids)?;
            ensure!(
                family.classification != CatalogClassification::Unreviewed,
                "{} {id} remains unreviewed in family {}",
                category.kind,
                family.name
            );
            entries.push(BundleEntry::new(
                PersistentId::new(id.parse::<ResourceId>()?),
                FamilyName::new(&family.name)?,
                value,
                provenance.clone(),
            )?);
        }
        total_entries += entries.len();
        registries.push(BundleRegistry::new(
            RegistryName::new(ResourceId::new("minecraft", &category.kind)?),
            Sha1Digest::new(&category.ids_sha1)?,
            families,
            entries,
        )?);
    }

    let bundle = ContentBundle::new(
        BundleSchemaVersion::V1,
        lock.version,
        ContentDigest::blake3(&lock_bytes),
        artifacts,
        registries,
    )?;
    let registry_count = bundle.registries().len();
    Ok(ImportedBundle {
        registries: registry_count,
        entries: total_entries,
        bundle,
    })
}

pub(crate) fn load(path: &Path) -> Result<ImportedBundle> {
    let file = fs::File::open(path).with_context(|| format!("open bundle {}", path.display()))?;
    let bundle: ContentBundle = serde_json::from_reader(file)
        .with_context(|| format!("validate bundle {}", path.display()))?;
    let registries = bundle.registries().len();
    let entries = bundle
        .registries()
        .map(|registry| registry.entries().len())
        .sum();
    Ok(ImportedBundle {
        bundle,
        registries,
        entries,
    })
}

pub(crate) fn read_bundle_lock(workspace: &Path) -> Result<Option<BundleLock>> {
    let path = workspace.join(BUNDLE_LOCK);
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(
        toml::from_str(&fs::read_to_string(&path)?)
            .with_context(|| format!("parse {}", path.display()))?,
    ))
}

pub(crate) fn verify_lock(imported: &ImportedBundle, lock: &BundleLock) -> Result<()> {
    ensure!(
        imported.bundle.schema_version().get() == lock.schema_version,
        "bundle schema drift: expected {}, got {}",
        lock.schema_version,
        imported.bundle.schema_version().get()
    );
    ensure!(
        imported.bundle.reference_version() == lock.reference_version,
        "bundle reference version drift"
    );
    ensure!(
        imported.registries == lock.registries,
        "bundle registry count drift: expected {}, got {}",
        lock.registries,
        imported.registries
    );
    ensure!(
        imported.entries == lock.entries,
        "bundle entry count drift: expected {}, got {}",
        lock.entries,
        imported.entries
    );
    let actual_bundle = imported.bundle.digest()?;
    let expected_bundle = lock.bundle_digest.parse::<ContentDigest>()?;
    ensure!(
        actual_bundle == expected_bundle,
        "generated bundle digest drift: expected {expected_bundle}, got {actual_bundle}"
    );
    let actual_manifest = imported.bundle.content_manifest()?.digest();
    let expected_manifest = lock.content_manifest_digest.parse::<ContentDigest>()?;
    ensure!(
        actual_manifest == expected_manifest,
        "content manifest digest drift: expected {expected_manifest}, got {actual_manifest}"
    );
    Ok(())
}

pub(crate) fn print_candidate(imported: &ImportedBundle) -> Result<()> {
    println!(
        "schema_version = {}\nreference_version = {:?}\nbundle_digest = {:?}\ncontent_manifest_digest = {:?}\nregistries = {}\nentries = {}",
        imported.bundle.schema_version().get(),
        imported.bundle.reference_version(),
        imported.bundle.digest()?.to_string(),
        imported.bundle.content_manifest()?.digest().to_string(),
        imported.registries,
        imported.entries
    );
    Ok(())
}

pub(crate) fn artifact_summary(artifact: &SourceArtifact) -> String {
    format!(
        "{} sha1={} bytes={} blake3={}",
        artifact.name(),
        artifact.sha1(),
        artifact.size(),
        artifact.content_digest()
    )
}
