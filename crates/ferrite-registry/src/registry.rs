//! Deterministic contribution assembly and runtime ID assignment.

use crate::digest::ContentDigest;
use crate::provenance::ContentProvenance;
use ferrite_foundation::resource::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegistryName(ResourceId);

impl RegistryName {
    pub const fn new(identifier: ResourceId) -> Self {
        Self(identifier)
    }

    pub const fn resource(&self) -> &ResourceId {
        &self.0
    }
}

impl Display for RegistryName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersistentId(ResourceId);

impl PersistentId {
    pub const fn new(identifier: ResourceId) -> Self {
        Self(identifier)
    }

    pub const fn resource(&self) -> &ResourceId {
        &self.0
    }
}

impl Display for PersistentId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

/// Process-local dense identity. It intentionally has no serialization implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeId(u32);

impl RuntimeId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContributionOrder {
    stage: u16,
    contributor: ResourceId,
    ordinal: u32,
}

impl ContributionOrder {
    pub const fn new(stage: u16, contributor: ResourceId, ordinal: u32) -> Self {
        Self {
            stage,
            contributor,
            ordinal,
        }
    }

    pub const fn stage(&self) -> u16 {
        self.stage
    }

    pub const fn contributor(&self) -> &ResourceId {
        &self.contributor
    }

    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }
}

#[derive(Debug)]
struct Contribution<T> {
    persistent_id: PersistentId,
    value: T,
    content_digest: ContentDigest,
    provenance: ContentProvenance,
    order: ContributionOrder,
}

#[derive(Debug)]
pub struct RegistryBuilder<T> {
    name: RegistryName,
    contributions: Vec<Contribution<T>>,
}

impl<T> RegistryBuilder<T> {
    pub const fn new(name: RegistryName) -> Self {
        Self {
            name,
            contributions: Vec::new(),
        }
    }

    pub fn contribute(
        &mut self,
        order: ContributionOrder,
        persistent_id: PersistentId,
        value: T,
        content_digest: ContentDigest,
        provenance: ContentProvenance,
    ) -> &mut Self {
        self.contributions.push(Contribution {
            persistent_id,
            value,
            content_digest,
            provenance,
            order,
        });
        self
    }

    pub fn build(mut self) -> Result<Registry<T>, RegistryBuildError> {
        self.contributions.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| left.persistent_id.cmp(&right.persistent_id))
        });

        let mut persistent_ids = BTreeSet::new();
        let mut contribution_orders = BTreeSet::new();
        let mut entries = Vec::with_capacity(self.contributions.len());
        let mut runtime_by_persistent = BTreeMap::new();

        for (index, contribution) in self.contributions.into_iter().enumerate() {
            if !contribution_orders.insert(contribution.order.clone()) {
                return Err(RegistryBuildError::DuplicateContributionOrder {
                    order: contribution.order,
                });
            }
            if !persistent_ids.insert(contribution.persistent_id.clone()) {
                return Err(RegistryBuildError::DuplicatePersistentId {
                    id: contribution.persistent_id,
                });
            }
            let runtime_value =
                u32::try_from(index).map_err(|_| RegistryBuildError::RuntimeIdExhausted)?;
            let runtime_id = RuntimeId::new(runtime_value);
            runtime_by_persistent.insert(contribution.persistent_id.clone(), runtime_id);
            entries.push(RegistryEntry {
                persistent_id: contribution.persistent_id,
                runtime_id,
                value: contribution.value,
                content_digest: contribution.content_digest,
                provenance: contribution.provenance,
            });
        }

        Ok(Registry {
            name: self.name,
            entries,
            runtime_by_persistent,
        })
    }
}

#[derive(Debug)]
pub struct Registry<T> {
    name: RegistryName,
    entries: Vec<RegistryEntry<T>>,
    runtime_by_persistent: BTreeMap<PersistentId, RuntimeId>,
}

impl<T> Registry<T> {
    pub const fn name(&self) -> &RegistryName {
        &self.name
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get_by_runtime(&self, id: RuntimeId) -> Option<&RegistryEntry<T>> {
        let index = usize::try_from(id.get()).ok()?;
        self.entries.get(index)
    }

    pub fn get_by_persistent(&self, id: &PersistentId) -> Option<&RegistryEntry<T>> {
        self.runtime_id(id)
            .and_then(|runtime_id| self.get_by_runtime(runtime_id))
    }

    pub fn runtime_id(&self, id: &PersistentId) -> Option<RuntimeId> {
        self.runtime_by_persistent.get(id).copied()
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = &RegistryEntry<T>> {
        self.entries.iter()
    }
}

#[derive(Debug)]
pub struct RegistryEntry<T> {
    persistent_id: PersistentId,
    runtime_id: RuntimeId,
    value: T,
    content_digest: ContentDigest,
    provenance: ContentProvenance,
}

impl<T> RegistryEntry<T> {
    pub const fn persistent_id(&self) -> &PersistentId {
        &self.persistent_id
    }

    pub const fn runtime_id(&self) -> RuntimeId {
        self.runtime_id
    }

    pub const fn value(&self) -> &T {
        &self.value
    }

    pub const fn content_digest(&self) -> ContentDigest {
        self.content_digest
    }

    pub const fn provenance(&self) -> &ContentProvenance {
        &self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegistryBuildError {
    #[error("duplicate persistent registry identity {id}")]
    DuplicatePersistentId { id: PersistentId },
    #[error("duplicate contribution order {order:?}")]
    DuplicateContributionOrder { order: ContributionOrder },
    #[error("registry contains more entries than a u32 runtime ID can address")]
    RuntimeIdExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::ProvenanceKind;

    fn resource(path: &str) -> ResourceId {
        ResourceId::new("ferrite", path).unwrap()
    }

    fn provenance() -> ContentProvenance {
        ContentProvenance::new(
            ProvenanceKind::ProjectAuthored,
            resource("tests"),
            "v1",
            ContentDigest::blake3(b"fixture"),
        )
        .unwrap()
    }

    #[test]
    fn contribution_order_not_insertion_order_assigns_runtime_ids() {
        let mut builder = RegistryBuilder::new(RegistryName::new(resource("blocks")));
        builder
            .contribute(
                ContributionOrder::new(0, resource("base"), 1),
                PersistentId::new(resource("second")),
                2,
                ContentDigest::blake3(b"second"),
                provenance(),
            )
            .contribute(
                ContributionOrder::new(0, resource("base"), 0),
                PersistentId::new(resource("first")),
                1,
                ContentDigest::blake3(b"first"),
                provenance(),
            );
        let registry = builder.build().unwrap();
        assert_eq!(
            registry
                .runtime_id(&PersistentId::new(resource("first")))
                .unwrap()
                .get(),
            0
        );
        assert_eq!(
            registry.get_by_runtime(RuntimeId::new(1)).unwrap().value(),
            &2
        );
    }

    #[test]
    fn duplicate_identity_or_order_is_rejected() {
        let duplicate_id = PersistentId::new(resource("same"));
        let mut builder = RegistryBuilder::new(RegistryName::new(resource("blocks")));
        for ordinal in 0..2 {
            builder.contribute(
                ContributionOrder::new(0, resource("base"), ordinal),
                duplicate_id.clone(),
                ordinal,
                ContentDigest::blake3(&[ordinal as u8]),
                provenance(),
            );
        }
        assert!(matches!(
            builder.build(),
            Err(RegistryBuildError::DuplicatePersistentId { .. })
        ));

        let mut builder = RegistryBuilder::new(RegistryName::new(resource("items")));
        for path in ["first", "second"] {
            builder.contribute(
                ContributionOrder::new(0, resource("base"), 0),
                PersistentId::new(resource(path)),
                path,
                ContentDigest::blake3(path.as_bytes()),
                provenance(),
            );
        }
        assert!(matches!(
            builder.build(),
            Err(RegistryBuildError::DuplicateContributionOrder { .. })
        ));
    }
}
