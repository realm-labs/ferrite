//! Auditable origin metadata for registry content.

use crate::digest::ContentDigest;
use ferrite_foundation::resource::ResourceId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_REVISION_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProvenanceKind {
    ProjectAuthored,
    LocalOfficialArtifact,
    GeneratedBundle,
}

impl ProvenanceKind {
    pub const fn stable_tag(self) -> u8 {
        match self {
            Self::ProjectAuthored => 0,
            Self::LocalOfficialArtifact => 1,
            Self::GeneratedBundle => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "ContentProvenanceRepr", into = "ContentProvenanceRepr")]
pub struct ContentProvenance {
    kind: ProvenanceKind,
    provider: ResourceId,
    revision: String,
    source_digest: ContentDigest,
}

impl ContentProvenance {
    pub fn new(
        kind: ProvenanceKind,
        provider: ResourceId,
        revision: impl Into<String>,
        source_digest: ContentDigest,
    ) -> Result<Self, ProvenanceError> {
        let revision = revision.into();
        validate_revision(&revision)?;
        Ok(Self {
            kind,
            provider,
            revision,
            source_digest,
        })
    }

    pub const fn kind(&self) -> ProvenanceKind {
        self.kind
    }

    pub const fn provider(&self) -> &ResourceId {
        &self.provider
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub const fn source_digest(&self) -> ContentDigest {
        self.source_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ContentProvenanceRepr {
    kind: ProvenanceKind,
    provider: ResourceId,
    revision: String,
    source_digest: ContentDigest,
}

impl TryFrom<ContentProvenanceRepr> for ContentProvenance {
    type Error = ProvenanceError;

    fn try_from(value: ContentProvenanceRepr) -> Result<Self, Self::Error> {
        Self::new(
            value.kind,
            value.provider,
            value.revision,
            value.source_digest,
        )
    }
}

impl From<ContentProvenance> for ContentProvenanceRepr {
    fn from(value: ContentProvenance) -> Self {
        Self {
            kind: value.kind,
            provider: value.provider,
            revision: value.revision,
            source_digest: value.source_digest,
        }
    }
}

fn validate_revision(revision: &str) -> Result<(), ProvenanceError> {
    if revision.is_empty() {
        return Err(ProvenanceError::EmptyRevision);
    }
    if revision.len() > MAX_REVISION_BYTES {
        return Err(ProvenanceError::RevisionTooLong {
            actual: revision.len(),
            maximum: MAX_REVISION_BYTES,
        });
    }
    if revision.chars().any(char::is_control) {
        return Err(ProvenanceError::ControlCharacter);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ProvenanceError {
    #[error("content provenance revision cannot be empty")]
    EmptyRevision,
    #[error("content provenance revision is {actual} bytes; maximum is {maximum}")]
    RevisionTooLong { actual: usize, maximum: usize },
    #[error("content provenance revision contains a control character")]
    ControlCharacter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revisions_are_bounded_and_revalidated() {
        let provider = ResourceId::new("ferrite", "fixture").unwrap();
        let digest = ContentDigest::blake3(b"fixture");
        let provenance =
            ContentProvenance::new(ProvenanceKind::ProjectAuthored, provider, "v1", digest)
                .unwrap();
        let encoded = serde_json::to_string(&provenance).unwrap();
        assert_eq!(
            serde_json::from_str::<ContentProvenance>(&encoded).unwrap(),
            provenance
        );
        assert!(
            ContentProvenance::new(
                ProvenanceKind::ProjectAuthored,
                ResourceId::new("ferrite", "fixture").unwrap(),
                "",
                digest
            )
            .is_err()
        );
    }
}
