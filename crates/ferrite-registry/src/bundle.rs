//! Validated project-owned schema for locally generated content bundles.

use crate::digest::ContentDigest;
use crate::manifest::{
    ContentManifest, ManifestError, ManifestSchemaVersion, RegistryManifest, RegistryManifestEntry,
};
use crate::provenance::ContentProvenance;
use crate::registry::{PersistentId, RegistryName};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};
use std::num::NonZeroU16;
use thiserror::Error;

const MAX_REFERENCE_VERSION_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BundleSchemaVersion(NonZeroU16);

impl BundleSchemaVersion {
    pub const V1: Self = Self(NonZeroU16::MIN);

    pub const fn new(value: u16) -> Result<Self, BundleError> {
        match NonZeroU16::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(BundleError::ZeroSchemaVersion),
        }
    }

    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Sha1Digest(String);

impl Sha1Digest {
    pub fn new(value: impl Into<String>) -> Result<Self, BundleError> {
        let value = value.into();
        if value.len() != 40
            || !value
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(BundleError::InvalidSha1 { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Sha1Digest {
    type Error = BundleError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Sha1Digest> for String {
    fn from(value: Sha1Digest) -> Self {
        value.0
    }
}

impl Display for Sha1Digest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FamilyName(String);

impl FamilyName {
    pub fn new(value: impl Into<String>) -> Result<Self, BundleError> {
        let value = value.into();
        if value.is_empty()
            || !value
                .chars()
                .all(|character| matches!(character, 'a'..='z' | '0'..='9' | '-' | '_' | '/'))
        {
            return Err(BundleError::InvalidFamilyName { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FamilyName {
    type Error = BundleError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<FamilyName> for String {
    fn from(value: FamilyName) -> Self {
        value.0
    }
}

impl Display for FamilyName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CatalogClassification {
    BehaviorFamily,
    Special,
    DataOnly,
    Unreviewed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CatalogFamilyRepr", into = "CatalogFamilyRepr")]
pub struct CatalogFamily {
    name: FamilyName,
    classification: CatalogClassification,
    rules: Vec<String>,
}

impl CatalogFamily {
    pub fn new(
        name: FamilyName,
        classification: CatalogClassification,
        mut rules: Vec<String>,
    ) -> Result<Self, BundleError> {
        if rules.is_empty() {
            return Err(BundleError::EmptyFamilyRules { family: name });
        }
        if let Some(rule) = rules.iter().find(|rule| !valid_rule_id(rule)) {
            return Err(BundleError::InvalidRuleId { rule: rule.clone() });
        }
        rules.sort();
        rules.dedup();
        Ok(Self {
            name,
            classification,
            rules,
        })
    }

    pub const fn name(&self) -> &FamilyName {
        &self.name
    }

    pub const fn classification(&self) -> CatalogClassification {
        self.classification
    }

    pub fn rules(&self) -> impl ExactSizeIterator<Item = &str> {
        self.rules.iter().map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CatalogFamilyRepr {
    name: FamilyName,
    classification: CatalogClassification,
    rules: Vec<String>,
}

impl TryFrom<CatalogFamilyRepr> for CatalogFamily {
    type Error = BundleError;

    fn try_from(value: CatalogFamilyRepr) -> Result<Self, Self::Error> {
        Self::new(value.name, value.classification, value.rules)
    }
}

impl From<CatalogFamily> for CatalogFamilyRepr {
    fn from(value: CatalogFamily) -> Self {
        Self {
            name: value.name,
            classification: value.classification,
            rules: value.rules,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceArtifact {
    name: FamilyName,
    sha1: Sha1Digest,
    size: u64,
    content_digest: ContentDigest,
}

impl SourceArtifact {
    pub const fn new(
        name: FamilyName,
        sha1: Sha1Digest,
        size: u64,
        content_digest: ContentDigest,
    ) -> Self {
        Self {
            name,
            sha1,
            size,
            content_digest,
        }
    }

    pub const fn name(&self) -> &FamilyName {
        &self.name
    }

    pub const fn sha1(&self) -> &Sha1Digest {
        &self.sha1
    }

    pub const fn size(&self) -> u64 {
        self.size
    }

    pub const fn content_digest(&self) -> ContentDigest {
        self.content_digest
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "BundleEntryRepr", into = "BundleEntryRepr")]
pub struct BundleEntry {
    persistent_id: PersistentId,
    family: FamilyName,
    value: Value,
    content_digest: ContentDigest,
    provenance: ContentProvenance,
}

impl BundleEntry {
    pub fn new(
        persistent_id: PersistentId,
        family: FamilyName,
        value: Value,
        provenance: ContentProvenance,
    ) -> Result<Self, BundleError> {
        let content_digest = digest_value(&value)?;
        Ok(Self {
            persistent_id,
            family,
            value,
            content_digest,
            provenance,
        })
    }

    pub const fn persistent_id(&self) -> &PersistentId {
        &self.persistent_id
    }

    pub const fn family(&self) -> &FamilyName {
        &self.family
    }

    pub const fn value(&self) -> &Value {
        &self.value
    }

    pub const fn content_digest(&self) -> ContentDigest {
        self.content_digest
    }

    pub const fn provenance(&self) -> &ContentProvenance {
        &self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct BundleEntryRepr {
    persistent_id: PersistentId,
    family: FamilyName,
    value: Value,
    content_digest: ContentDigest,
    provenance: ContentProvenance,
}

impl TryFrom<BundleEntryRepr> for BundleEntry {
    type Error = BundleError;

    fn try_from(value: BundleEntryRepr) -> Result<Self, Self::Error> {
        let entry = Self::new(
            value.persistent_id,
            value.family,
            value.value,
            value.provenance,
        )?;
        if entry.content_digest != value.content_digest {
            return Err(BundleError::ContentDigestMismatch {
                id: entry.persistent_id,
            });
        }
        Ok(entry)
    }
}

impl From<BundleEntry> for BundleEntryRepr {
    fn from(value: BundleEntry) -> Self {
        Self {
            persistent_id: value.persistent_id,
            family: value.family,
            value: value.value,
            content_digest: value.content_digest,
            provenance: value.provenance,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "BundleRegistryRepr", into = "BundleRegistryRepr")]
pub struct BundleRegistry {
    name: RegistryName,
    ids_sha1: Sha1Digest,
    families: Vec<CatalogFamily>,
    entries: Vec<BundleEntry>,
}

impl BundleRegistry {
    pub fn new(
        name: RegistryName,
        ids_sha1: Sha1Digest,
        mut families: Vec<CatalogFamily>,
        mut entries: Vec<BundleEntry>,
    ) -> Result<Self, BundleError> {
        families.sort_by(|left, right| left.name.cmp(&right.name));
        entries.sort_by(|left, right| left.persistent_id.cmp(&right.persistent_id));

        reject_duplicate_families(&families)?;
        reject_duplicate_entries(&name, &entries)?;
        let family_names = families
            .iter()
            .map(|family| family.name.clone())
            .collect::<BTreeSet<_>>();
        for entry in &entries {
            if !family_names.contains(&entry.family) {
                return Err(BundleError::UnknownFamily {
                    registry: name,
                    family: entry.family.clone(),
                });
            }
        }
        Ok(Self {
            name,
            ids_sha1,
            families,
            entries,
        })
    }

    pub const fn name(&self) -> &RegistryName {
        &self.name
    }

    pub const fn ids_sha1(&self) -> &Sha1Digest {
        &self.ids_sha1
    }

    pub fn families(&self) -> impl ExactSizeIterator<Item = &CatalogFamily> {
        self.families.iter()
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = &BundleEntry> {
        self.entries.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct BundleRegistryRepr {
    name: RegistryName,
    ids_sha1: Sha1Digest,
    families: Vec<CatalogFamily>,
    entries: Vec<BundleEntry>,
}

impl TryFrom<BundleRegistryRepr> for BundleRegistry {
    type Error = BundleError;

    fn try_from(value: BundleRegistryRepr) -> Result<Self, Self::Error> {
        Self::new(value.name, value.ids_sha1, value.families, value.entries)
    }
}

impl From<BundleRegistry> for BundleRegistryRepr {
    fn from(value: BundleRegistry) -> Self {
        Self {
            name: value.name,
            ids_sha1: value.ids_sha1,
            families: value.families,
            entries: value.entries,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ContentBundleRepr", into = "ContentBundleRepr")]
pub struct ContentBundle {
    schema_version: BundleSchemaVersion,
    reference_version: String,
    lock_digest: ContentDigest,
    source_artifacts: Vec<SourceArtifact>,
    registries: Vec<BundleRegistry>,
}

impl ContentBundle {
    pub fn new(
        schema_version: BundleSchemaVersion,
        reference_version: impl Into<String>,
        lock_digest: ContentDigest,
        mut source_artifacts: Vec<SourceArtifact>,
        mut registries: Vec<BundleRegistry>,
    ) -> Result<Self, BundleError> {
        let reference_version = reference_version.into();
        validate_reference_version(&reference_version)?;
        source_artifacts.sort_by(|left, right| left.name.cmp(&right.name));
        registries.sort_by(|left, right| left.name.cmp(&right.name));
        reject_duplicate_artifacts(&source_artifacts)?;
        reject_duplicate_registries(&registries)?;
        Ok(Self {
            schema_version,
            reference_version,
            lock_digest,
            source_artifacts,
            registries,
        })
    }

    pub const fn schema_version(&self) -> BundleSchemaVersion {
        self.schema_version
    }

    pub fn reference_version(&self) -> &str {
        &self.reference_version
    }

    pub const fn lock_digest(&self) -> ContentDigest {
        self.lock_digest
    }

    pub fn source_artifacts(&self) -> impl ExactSizeIterator<Item = &SourceArtifact> {
        self.source_artifacts.iter()
    }

    pub fn registries(&self) -> impl ExactSizeIterator<Item = &BundleRegistry> {
        self.registries.iter()
    }

    pub fn digest(&self) -> Result<ContentDigest, BundleError> {
        let value = serde_json::to_value(self).map_err(BundleError::Encode)?;
        digest_value(&value)
    }

    pub fn content_manifest(&self) -> Result<ContentManifest, BundleError> {
        let registries = self
            .registries
            .iter()
            .map(|registry| {
                let entries = registry
                    .entries
                    .iter()
                    .map(|entry| {
                        RegistryManifestEntry::new(
                            entry.persistent_id.clone(),
                            entry.content_digest,
                            entry.provenance.clone(),
                        )
                    })
                    .collect();
                RegistryManifest::new(registry.name.clone(), entries)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ContentManifest::new(ManifestSchemaVersion::V1, registries)?)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ContentBundleRepr {
    schema_version: BundleSchemaVersion,
    reference_version: String,
    lock_digest: ContentDigest,
    source_artifacts: Vec<SourceArtifact>,
    registries: Vec<BundleRegistry>,
}

impl TryFrom<ContentBundleRepr> for ContentBundle {
    type Error = BundleError;

    fn try_from(value: ContentBundleRepr) -> Result<Self, Self::Error> {
        Self::new(
            value.schema_version,
            value.reference_version,
            value.lock_digest,
            value.source_artifacts,
            value.registries,
        )
    }
}

impl From<ContentBundle> for ContentBundleRepr {
    fn from(value: ContentBundle) -> Self {
        Self {
            schema_version: value.schema_version,
            reference_version: value.reference_version,
            lock_digest: value.lock_digest,
            source_artifacts: value.source_artifacts,
            registries: value.registries,
        }
    }
}

fn digest_value(value: &Value) -> Result<ContentDigest, BundleError> {
    let mut bytes = Vec::new();
    write_canonical_json(value, &mut bytes)?;
    Ok(ContentDigest::blake3(&bytes))
}

fn write_canonical_json(value: &Value, output: &mut Vec<u8>) -> Result<(), BundleError> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_writer(output, value).map_err(BundleError::Encode)?;
        }
        Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        Value::Object(values) => {
            output.push(b'{');
            let mut fields = values.iter().collect::<Vec<_>>();
            fields.sort_by_key(|(key, _)| *key);
            for (index, (key, value)) in fields.into_iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key).map_err(BundleError::Encode)?;
                output.push(b':');
                write_canonical_json(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn valid_rule_id(rule: &str) -> bool {
    !rule.is_empty()
        && rule
            .chars()
            .all(|character| matches!(character, 'A'..='Z' | '0'..='9' | '-'))
}

fn validate_reference_version(version: &str) -> Result<(), BundleError> {
    if version.is_empty()
        || version.len() > MAX_REFERENCE_VERSION_BYTES
        || version.chars().any(char::is_control)
    {
        return Err(BundleError::InvalidReferenceVersion {
            value: version.to_owned(),
        });
    }
    Ok(())
}

fn reject_duplicate_families(families: &[CatalogFamily]) -> Result<(), BundleError> {
    for pair in families.windows(2) {
        if pair[0].name == pair[1].name {
            return Err(BundleError::DuplicateFamily {
                family: pair[0].name.clone(),
            });
        }
    }
    Ok(())
}

fn reject_duplicate_entries(
    registry: &RegistryName,
    entries: &[BundleEntry],
) -> Result<(), BundleError> {
    for pair in entries.windows(2) {
        if pair[0].persistent_id == pair[1].persistent_id {
            return Err(BundleError::DuplicateEntry {
                registry: registry.clone(),
                id: pair[0].persistent_id.clone(),
            });
        }
    }
    Ok(())
}

fn reject_duplicate_artifacts(artifacts: &[SourceArtifact]) -> Result<(), BundleError> {
    for pair in artifacts.windows(2) {
        if pair[0].name == pair[1].name {
            return Err(BundleError::DuplicateArtifact {
                name: pair[0].name.clone(),
            });
        }
    }
    Ok(())
}

fn reject_duplicate_registries(registries: &[BundleRegistry]) -> Result<(), BundleError> {
    for pair in registries.windows(2) {
        if pair[0].name == pair[1].name {
            return Err(BundleError::DuplicateRegistry {
                registry: pair[0].name.clone(),
            });
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum BundleError {
    #[error("content bundle schema version cannot be zero")]
    ZeroSchemaVersion,
    #[error("invalid SHA-1 digest {value:?}")]
    InvalidSha1 { value: String },
    #[error("invalid catalog family name {value:?}")]
    InvalidFamilyName { value: String },
    #[error("catalog family {family} has no rule ownership")]
    EmptyFamilyRules { family: FamilyName },
    #[error("invalid catalog rule identity {rule:?}")]
    InvalidRuleId { rule: String },
    #[error("content digest does not match canonical payload for {id}")]
    ContentDigestMismatch { id: PersistentId },
    #[error("duplicate catalog family {family}")]
    DuplicateFamily { family: FamilyName },
    #[error("registry {registry} entry refers to unknown family {family}")]
    UnknownFamily {
        registry: RegistryName,
        family: FamilyName,
    },
    #[error("duplicate persistent entry {id} in bundle registry {registry}")]
    DuplicateEntry {
        registry: RegistryName,
        id: PersistentId,
    },
    #[error("invalid reference version {value:?}")]
    InvalidReferenceVersion { value: String },
    #[error("duplicate source artifact {name}")]
    DuplicateArtifact { name: FamilyName },
    #[error("duplicate bundle registry {registry}")]
    DuplicateRegistry { registry: RegistryName },
    #[error("encode canonical content bundle JSON: {0}")]
    Encode(serde_json::Error),
    #[error(transparent)]
    Manifest(#[from] ManifestError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{ContentProvenance, ProvenanceKind};
    use ferrite_foundation::resource::ResourceId;

    fn fixture_bundle(value: Value) -> ContentBundle {
        let source_digest = ContentDigest::blake3(b"source");
        let provenance = ContentProvenance::new(
            ProvenanceKind::ProjectAuthored,
            ResourceId::new("ferrite", "fixture").unwrap(),
            "v1",
            source_digest,
        )
        .unwrap();
        let entry = BundleEntry::new(
            PersistentId::new(ResourceId::minecraft("stone").unwrap()),
            FamilyName::new("stone-family").unwrap(),
            value,
            provenance,
        )
        .unwrap();
        let family = CatalogFamily::new(
            FamilyName::new("stone-family").unwrap(),
            CatalogClassification::BehaviorFamily,
            vec!["BLK-001".to_owned()],
        )
        .unwrap();
        let registry = BundleRegistry::new(
            RegistryName::new(ResourceId::minecraft("block").unwrap()),
            Sha1Digest::new("0000000000000000000000000000000000000000").unwrap(),
            vec![family],
            vec![entry],
        )
        .unwrap();
        ContentBundle::new(
            BundleSchemaVersion::V1,
            "fixture",
            ContentDigest::blake3(b"lock"),
            vec![SourceArtifact::new(
                FamilyName::new("server").unwrap(),
                Sha1Digest::new("0000000000000000000000000000000000000000").unwrap(),
                6,
                source_digest,
            )],
            vec![registry],
        )
        .unwrap()
    }

    #[test]
    fn canonical_digest_sorts_json_object_fields() {
        let first = fixture_bundle(serde_json::json!({"z": 1, "a": 2}));
        let second = fixture_bundle(serde_json::json!({"a": 2, "z": 1}));
        assert_eq!(first.digest().unwrap(), second.digest().unwrap());
        assert_eq!(
            first.content_manifest().unwrap().digest(),
            second.content_manifest().unwrap().digest()
        );
    }

    #[test]
    fn deserialization_revalidates_payload_digest() {
        let bundle = fixture_bundle(serde_json::json!({"value": 1}));
        let encoded = serde_json::to_string(&bundle).unwrap();
        let corrupted = encoded.replace("\"value\":1", "\"value\":2");
        assert!(serde_json::from_str::<ContentBundle>(&corrupted).is_err());
    }
}
