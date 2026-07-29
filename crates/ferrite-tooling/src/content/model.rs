use ferrite_registry::bundle::CatalogClassification;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ReferenceLock {
    pub(crate) version: String,
    pub(crate) client: ArtifactLock,
    pub(crate) server: ArtifactLock,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ArtifactLock {
    pub(crate) sha1: String,
    pub(crate) size: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Catalog {
    pub(crate) category: Vec<Category>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Category {
    pub(crate) kind: String,
    pub(crate) expected_count: usize,
    pub(crate) ids_sha1: String,
    pub(crate) family: Vec<Family>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Family {
    pub(crate) name: String,
    pub(crate) classification: CatalogClassification,
    pub(crate) rules: Vec<String>,
    #[serde(default)]
    pub(crate) exact: Vec<String>,
    #[serde(default)]
    pub(crate) patterns: Vec<String>,
    #[serde(default)]
    pub(crate) block_items: bool,
    #[serde(default)]
    pub(crate) remaining: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BundleLock {
    pub(crate) schema_version: u16,
    pub(crate) reference_version: String,
    pub(crate) bundle_digest: String,
    pub(crate) content_manifest_digest: String,
    pub(crate) registries: usize,
    pub(crate) entries: usize,
}
