//! Versioned authoritative structure starts and per-chunk references.

use std::collections::BTreeSet;

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::resource::ResourceId;
use thiserror::Error;

pub const STRUCTURE_STATE_VERSION_V1: u16 = 1;
pub const MAX_STRUCTURE_STARTS_PER_CHUNK: usize = 256;
pub const MAX_STRUCTURE_REFERENCES_PER_CHUNK: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructureBounds {
    pub minimum_x: i32,
    pub minimum_y: i32,
    pub minimum_z: i32,
    pub maximum_x: i32,
    pub maximum_y: i32,
    pub maximum_z: i32,
}

impl StructureBounds {
    pub fn new(
        minimum_x: i32,
        minimum_y: i32,
        minimum_z: i32,
        maximum_x: i32,
        maximum_y: i32,
        maximum_z: i32,
    ) -> Result<Self, StructureStateError> {
        if minimum_x > maximum_x || minimum_y > maximum_y || minimum_z > maximum_z {
            return Err(StructureStateError::InvertedBounds);
        }
        Ok(Self {
            minimum_x,
            minimum_y,
            minimum_z,
            maximum_x,
            maximum_y,
            maximum_z,
        })
    }

    #[must_use]
    pub fn intersects_chunk(self, chunk: ChunkPos) -> bool {
        let minimum_x = i64::from(chunk.x) * 16;
        let minimum_z = i64::from(chunk.z) * 16;
        let maximum_x = minimum_x + 15;
        let maximum_z = minimum_z + 15;
        i64::from(self.maximum_x) >= minimum_x
            && i64::from(self.minimum_x) <= maximum_x
            && i64::from(self.maximum_z) >= minimum_z
            && i64::from(self.minimum_z) <= maximum_z
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructurePlacement {
    pub structure: ResourceId,
    pub start_chunk: ChunkPos,
    pub bounds: StructureBounds,
    pub placement_seed: u64,
}

impl StructurePlacement {
    #[must_use]
    pub const fn new(
        structure: ResourceId,
        start_chunk: ChunkPos,
        bounds: StructureBounds,
        placement_seed: u64,
    ) -> Self {
        Self {
            structure,
            start_chunk,
            bounds,
            placement_seed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkStructureState {
    version: u16,
    starts: Box<[StructurePlacement]>,
    references: Box<[StructurePlacement]>,
}

impl ChunkStructureState {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            version: STRUCTURE_STATE_VERSION_V1,
            starts: Box::new([]),
            references: Box::new([]),
        }
    }

    pub fn v1(
        owner: ChunkPos,
        starts: impl IntoIterator<Item = StructurePlacement>,
        references: impl IntoIterator<Item = StructurePlacement>,
    ) -> Result<Self, StructureStateError> {
        let starts = bounded_sorted(
            starts,
            MAX_STRUCTURE_STARTS_PER_CHUNK,
            StructureStateError::TooManyStarts,
        )?;
        let references = bounded_sorted(
            references,
            MAX_STRUCTURE_REFERENCES_PER_CHUNK,
            StructureStateError::TooManyReferences,
        )?;
        if starts
            .iter()
            .any(|start| start.start_chunk != owner || !start.bounds.intersects_chunk(owner))
        {
            return Err(StructureStateError::StartOwnedByAnotherChunk);
        }
        if references
            .iter()
            .any(|reference| !reference.bounds.intersects_chunk(owner))
        {
            return Err(StructureStateError::ReferenceDoesNotIntersect);
        }
        Ok(Self {
            version: STRUCTURE_STATE_VERSION_V1,
            starts,
            references,
        })
    }

    pub fn from_durable_parts(
        version: u16,
        owner: ChunkPos,
        starts: Vec<StructurePlacement>,
        references: Vec<StructurePlacement>,
    ) -> Result<Self, StructureStateError> {
        if version != STRUCTURE_STATE_VERSION_V1 {
            return Err(StructureStateError::UnsupportedVersion(version));
        }
        let start_count = starts.len();
        let reference_count = references.len();
        let state = Self::v1(owner, starts, references)?;
        if state.starts.len() != start_count || state.references.len() != reference_count {
            return Err(StructureStateError::DuplicatePlacement);
        }
        Ok(state)
    }

    pub const fn version(&self) -> u16 {
        self.version
    }

    pub fn starts(&self) -> &[StructurePlacement] {
        &self.starts
    }

    pub fn references(&self) -> &[StructurePlacement] {
        &self.references
    }
}

fn bounded_sorted(
    values: impl IntoIterator<Item = StructurePlacement>,
    maximum: usize,
    overflow: StructureStateError,
) -> Result<Box<[StructurePlacement]>, StructureStateError> {
    let values = values.into_iter().collect::<BTreeSet<_>>();
    if values.len() > maximum {
        return Err(overflow);
    }
    Ok(values.into_iter().collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum StructureStateError {
    #[error("structure bounds are inverted")]
    InvertedBounds,
    #[error("structure state version {0} is unsupported")]
    UnsupportedVersion(u16),
    #[error("chunk contains too many structure starts")]
    TooManyStarts,
    #[error("chunk contains too many structure references")]
    TooManyReferences,
    #[error("structure start is owned by another chunk")]
    StartOwnedByAnotherChunk,
    #[error("structure reference does not intersect its owning chunk")]
    ReferenceDoesNotIntersect,
    #[error("durable structure state contains a duplicate placement")]
    DuplicatePlacement,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_sorts_deduplicates_and_validates_ownership() {
        let owner = ChunkPos::new(2, -3);
        let placement = StructurePlacement::new(
            ResourceId::new("ferrite", "waystone_ruin").unwrap(),
            owner,
            StructureBounds::new(46, 70, -34, 49, 74, -31).unwrap(),
            7,
        );
        let state = ChunkStructureState::v1(
            owner,
            [placement.clone(), placement.clone()],
            [placement.clone(), placement.clone()],
        )
        .unwrap();
        assert_eq!(state.starts(), std::slice::from_ref(&placement));
        assert_eq!(state.references(), std::slice::from_ref(&placement));
    }
}
