//! Persistent, deterministic inventory of registry content and provenance.

use crate::digest::ContentDigest;
use crate::provenance::ContentProvenance;
use crate::registry::{PersistentId, Registry, RegistryName};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::num::NonZeroU16;
use thiserror::Error;

const MANIFEST_DIGEST_DOMAIN: &[u8] = b"ferrite:content-manifest:v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ManifestSchemaVersion(NonZeroU16);

impl ManifestSchemaVersion {
    pub const V1: Self = Self(NonZeroU16::MIN);

    pub const fn new(value: u16) -> Result<Self, ManifestError> {
        match NonZeroU16::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(ManifestError::ZeroSchemaVersion),
        }
    }

    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryManifestEntry {
    persistent_id: PersistentId,
    content_digest: ContentDigest,
    provenance: ContentProvenance,
}

impl RegistryManifestEntry {
    pub const fn persistent_id(&self) -> &PersistentId {
        &self.persistent_id
    }

    pub const fn content_digest(&self) -> ContentDigest {
        self.content_digest
    }

    pub const fn provenance(&self) -> &ContentProvenance {
        &self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryManifest {
    name: RegistryName,
    entries: Vec<RegistryManifestEntry>,
}

impl RegistryManifest {
    pub fn from_registry<T>(registry: &Registry<T>) -> Self {
        Self {
            name: registry.name().clone(),
            entries: registry
                .entries()
                .map(|entry| RegistryManifestEntry {
                    persistent_id: entry.persistent_id().clone(),
                    content_digest: entry.content_digest(),
                    provenance: entry.provenance().clone(),
                })
                .collect(),
        }
    }

    pub const fn name(&self) -> &RegistryName {
        &self.name
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = &RegistryManifestEntry> {
        self.entries.iter()
    }

    fn validate(&self) -> Result<(), ManifestError> {
        let mut identities = BTreeSet::new();
        for entry in &self.entries {
            if !identities.insert(entry.persistent_id.clone()) {
                return Err(ManifestError::DuplicatePersistentId {
                    registry: self.name.clone(),
                    id: entry.persistent_id.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ContentManifestRepr", into = "ContentManifestRepr")]
pub struct ContentManifest {
    schema_version: ManifestSchemaVersion,
    registries: Vec<RegistryManifest>,
}

impl ContentManifest {
    pub fn new(
        schema_version: ManifestSchemaVersion,
        mut registries: Vec<RegistryManifest>,
    ) -> Result<Self, ManifestError> {
        registries.sort_by(|left, right| left.name.cmp(&right.name));
        for registry in &registries {
            registry.validate()?;
        }
        for pair in registries.windows(2) {
            if pair[0].name == pair[1].name {
                return Err(ManifestError::DuplicateRegistry {
                    registry: pair[0].name.clone(),
                });
            }
        }
        Ok(Self {
            schema_version,
            registries,
        })
    }

    pub const fn schema_version(&self) -> ManifestSchemaVersion {
        self.schema_version
    }

    pub fn registries(&self) -> impl ExactSizeIterator<Item = &RegistryManifest> {
        self.registries.iter()
    }

    pub fn digest(&self) -> ContentDigest {
        let mut hasher = blake3::Hasher::new();
        hasher.update(MANIFEST_DIGEST_DOMAIN);
        hasher.update(&self.schema_version.get().to_be_bytes());
        update_count(&mut hasher, self.registries.len());
        for registry in &self.registries {
            update_string(&mut hasher, &registry.name.to_string());
            update_count(&mut hasher, registry.entries.len());
            for entry in &registry.entries {
                update_string(&mut hasher, &entry.persistent_id.to_string());
                hasher.update(entry.content_digest.as_bytes());
                hasher.update(&[entry.provenance.kind().stable_tag()]);
                update_string(&mut hasher, &entry.provenance.provider().to_string());
                update_string(&mut hasher, entry.provenance.revision());
                hasher.update(entry.provenance.source_digest().as_bytes());
            }
        }
        ContentDigest::from_bytes(*hasher.finalize().as_bytes())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ContentManifestRepr {
    schema_version: ManifestSchemaVersion,
    registries: Vec<RegistryManifest>,
}

impl TryFrom<ContentManifestRepr> for ContentManifest {
    type Error = ManifestError;

    fn try_from(value: ContentManifestRepr) -> Result<Self, Self::Error> {
        Self::new(value.schema_version, value.registries)
    }
}

impl From<ContentManifest> for ContentManifestRepr {
    fn from(value: ContentManifest) -> Self {
        Self {
            schema_version: value.schema_version,
            registries: value.registries,
        }
    }
}

fn update_count(hasher: &mut blake3::Hasher, count: usize) {
    hasher.update(&(count as u64).to_be_bytes());
}

fn update_string(hasher: &mut blake3::Hasher, value: &str) {
    update_count(hasher, value.len());
    hasher.update(value.as_bytes());
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ManifestError {
    #[error("content manifest schema version cannot be zero")]
    ZeroSchemaVersion,
    #[error("duplicate registry manifest {registry}")]
    DuplicateRegistry { registry: RegistryName },
    #[error("duplicate persistent identity {id} in registry {registry}")]
    DuplicatePersistentId {
        registry: RegistryName,
        id: PersistentId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::ProvenanceKind;
    use crate::registry::{ContributionOrder, RegistryBuilder};
    use ferrite_foundation::resource::ResourceId;

    fn resource(path: &str) -> ResourceId {
        ResourceId::new("ferrite", path).unwrap()
    }

    fn registry(name: &str, insertion: &[(&str, u32)]) -> Registry<u32> {
        let provenance = ContentProvenance::new(
            ProvenanceKind::ProjectAuthored,
            resource("tests"),
            "v1",
            ContentDigest::blake3(b"fixture"),
        )
        .unwrap();
        let mut builder = RegistryBuilder::new(RegistryName::new(resource(name)));
        for (path, ordinal) in insertion {
            builder.contribute(
                ContributionOrder::new(0, resource("base"), *ordinal),
                PersistentId::new(resource(path)),
                *ordinal,
                ContentDigest::blake3(path.as_bytes()),
                provenance.clone(),
            );
        }
        builder.build().unwrap()
    }

    #[test]
    fn manifest_digest_ignores_input_insertion_and_registry_order() {
        let blocks = registry("blocks", &[("second", 1), ("first", 0)]);
        let items = registry("items", &[("item", 0)]);
        let first = ContentManifest::new(
            ManifestSchemaVersion::V1,
            vec![
                RegistryManifest::from_registry(&items),
                RegistryManifest::from_registry(&blocks),
            ],
        )
        .unwrap();

        let blocks_reordered = registry("blocks", &[("first", 0), ("second", 1)]);
        let items_reordered = registry("items", &[("item", 0)]);
        let second = ContentManifest::new(
            ManifestSchemaVersion::V1,
            vec![
                RegistryManifest::from_registry(&blocks_reordered),
                RegistryManifest::from_registry(&items_reordered),
            ],
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.digest(), second.digest());
    }

    #[test]
    fn serde_round_trip_preserves_digest_and_rejects_duplicate_registries() {
        let blocks = registry("blocks", &[("stone", 0)]);
        let manifest = ContentManifest::new(
            ManifestSchemaVersion::V1,
            vec![RegistryManifest::from_registry(&blocks)],
        )
        .unwrap();
        let encoded = serde_json::to_string(&manifest).unwrap();
        let decoded = serde_json::from_str::<ContentManifest>(&encoded).unwrap();
        assert_eq!(decoded.digest(), manifest.digest());

        let duplicate = format!(
            r#"{{"schema_version":1,"registries":[{registry},{registry}]}}"#,
            registry = serde_json::to_string(&RegistryManifest::from_registry(&blocks)).unwrap()
        );
        assert!(serde_json::from_str::<ContentManifest>(&duplicate).is_err());
    }
}
