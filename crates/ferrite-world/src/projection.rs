//! Immutable, protocol-neutral snapshots for client terrain projection.

use std::collections::BTreeMap;
use std::sync::Arc;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::resource::ResourceId;
use thiserror::Error;

use crate::chunk::{ChunkLayout, ChunkRevision};
use crate::id::BlockStateId;
use crate::section::ChunkSection;

pub const LIGHT_LAYER_BYTES: usize = 2_048;
const HEIGHTMAP_COLUMNS: usize = 16 * 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClientHeightmap {
    WorldSurface,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntitySnapshot {
    pub position: BlockPos,
    pub kind: ResourceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LightLayer {
    Empty,
    Data(Box<[u8; LIGHT_LAYER_BYTES]>),
}

impl LightLayer {
    #[must_use]
    pub fn full_brightness() -> Self {
        Self::Data(Box::new([0xff; LIGHT_LAYER_BYTES]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightSnapshot {
    sky: Box<[LightLayer]>,
    block: Box<[LightLayer]>,
}

/// Durable light authority owned by the same column as block and biome state.
///
/// The two boundary layers required by the Java chunk-light packet are retained
/// explicitly so persistence, simulation, and projection cannot disagree about
/// an implicit default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLightState {
    sky: Box<[LightLayer]>,
    block: Box<[LightLayer]>,
}

impl ChunkLightState {
    pub fn new(
        sky: Vec<LightLayer>,
        block: Vec<LightLayer>,
        section_count: u16,
    ) -> Result<Self, ChunkProjectionError> {
        let snapshot = LightSnapshot::new(sky, block, section_count)?;
        Ok(Self {
            sky: snapshot.sky,
            block: snapshot.block,
        })
    }

    pub fn snapshot(&self, section_count: u16) -> Result<LightSnapshot, ChunkProjectionError> {
        LightSnapshot::new(self.sky.to_vec(), self.block.to_vec(), section_count)
    }

    #[must_use]
    pub fn sky(&self) -> &[LightLayer] {
        &self.sky
    }

    #[must_use]
    pub fn block(&self) -> &[LightLayer] {
        &self.block
    }
}

impl LightSnapshot {
    pub fn new(
        sky: Vec<LightLayer>,
        block: Vec<LightLayer>,
        section_count: u16,
    ) -> Result<Self, ChunkProjectionError> {
        let expected = usize::from(section_count)
            .checked_add(2)
            .ok_or(ChunkProjectionError::LightSectionCountOverflow)?;
        if sky.len() != expected || block.len() != expected {
            return Err(ChunkProjectionError::LightLayerCount {
                expected,
                sky: sky.len(),
                block: block.len(),
            });
        }
        Ok(Self {
            sky: sky.into_boxed_slice(),
            block: block.into_boxed_slice(),
        })
    }

    pub fn full_sky(section_count: u16) -> Result<Self, ChunkProjectionError> {
        let count = usize::from(section_count)
            .checked_add(2)
            .ok_or(ChunkProjectionError::LightSectionCountOverflow)?;
        Self::new(
            vec![LightLayer::full_brightness(); count],
            vec![LightLayer::Empty; count],
            section_count,
        )
    }

    #[must_use]
    pub fn sky(&self) -> &[LightLayer] {
        &self.sky
    }

    #[must_use]
    pub fn block(&self) -> &[LightLayer] {
        &self.block
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSnapshot {
    inner: Arc<ChunkSnapshotInner>,
}

#[derive(Debug, PartialEq, Eq)]
struct ChunkSnapshotInner {
    position: ChunkPos,
    layout: ChunkLayout,
    revision: ChunkRevision,
    sections: Box<[ChunkSection]>,
    heightmaps: BTreeMap<ClientHeightmap, Box<[i32; HEIGHTMAP_COLUMNS]>>,
    block_entities: Box<[BlockEntitySnapshot]>,
    light: LightSnapshot,
}

impl ChunkSnapshot {
    pub fn new(
        position: ChunkPos,
        layout: ChunkLayout,
        revision: ChunkRevision,
        sections: Vec<ChunkSection>,
        block_entities: Vec<BlockEntitySnapshot>,
        light: LightSnapshot,
        mut heightmap_includes: impl FnMut(ClientHeightmap, BlockStateId) -> bool,
    ) -> Result<Self, ChunkProjectionError> {
        let expected = usize::from(layout.sections().count());
        if sections.len() != expected {
            return Err(ChunkProjectionError::SectionCount {
                expected,
                actual: sections.len(),
            });
        }
        validate_block_entities(position, layout, &block_entities)?;
        let heightmaps = [
            ClientHeightmap::WorldSurface,
            ClientHeightmap::MotionBlocking,
            ClientHeightmap::MotionBlockingNoLeaves,
        ]
        .into_iter()
        .map(|kind| {
            (
                kind,
                Box::new(derive_heightmap(layout, &sections, |state| {
                    heightmap_includes(kind, state)
                })),
            )
        })
        .collect();
        Ok(Self {
            inner: Arc::new(ChunkSnapshotInner {
                position,
                layout,
                revision,
                sections: sections.into_boxed_slice(),
                heightmaps,
                block_entities: block_entities.into_boxed_slice(),
                light,
            }),
        })
    }

    #[must_use]
    pub fn position(&self) -> ChunkPos {
        self.inner.position
    }

    #[must_use]
    pub fn layout(&self) -> ChunkLayout {
        self.inner.layout
    }

    #[must_use]
    pub fn revision(&self) -> ChunkRevision {
        self.inner.revision
    }

    #[must_use]
    pub fn sections(&self) -> &[ChunkSection] {
        &self.inner.sections
    }

    #[must_use]
    pub fn heightmaps(&self) -> &BTreeMap<ClientHeightmap, Box<[i32; HEIGHTMAP_COLUMNS]>> {
        &self.inner.heightmaps
    }

    #[must_use]
    pub fn block_entities(&self) -> &[BlockEntitySnapshot] {
        &self.inner.block_entities
    }

    #[must_use]
    pub fn light(&self) -> &LightSnapshot {
        &self.inner.light
    }
}

fn derive_heightmap(
    layout: ChunkLayout,
    sections: &[ChunkSection],
    mut includes: impl FnMut(BlockStateId) -> bool,
) -> [i32; HEIGHTMAP_COLUMNS] {
    let minimum_y = layout.sections().minimum() * 16;
    let maximum_y = layout.sections().maximum_exclusive() * 16;
    let mut heights = [minimum_y; HEIGHTMAP_COLUMNS];
    for z in 0..16 {
        for x in 0..16 {
            let column = z * 16 + x;
            'vertical: for y in (minimum_y..maximum_y).rev() {
                let section_index = ((y.div_euclid(16)) - layout.sections().minimum()) as usize;
                let local_y = y.rem_euclid(16) as u8;
                let local =
                    ferrite_foundation::coordinate::LocalBlockPos::new(x as u8, local_y, z as u8)
                        .expect("heightmap coordinates are inside a section");
                if includes(sections[section_index].block(local)) {
                    heights[column] = y + 1;
                    break 'vertical;
                }
            }
        }
    }
    heights
}

fn validate_block_entities(
    chunk: ChunkPos,
    layout: ChunkLayout,
    block_entities: &[BlockEntitySnapshot],
) -> Result<(), ChunkProjectionError> {
    let mut previous = None;
    for entity in block_entities {
        if entity.position.chunk() != chunk {
            return Err(ChunkProjectionError::WrongBlockEntityChunk {
                expected: chunk,
                actual: entity.position.chunk(),
            });
        }
        if !layout.sections().contains(entity.position.section().y) {
            return Err(ChunkProjectionError::BlockEntityOutsideVerticalRange {
                position: entity.position,
            });
        }
        if previous.is_some_and(|position| position >= entity.position) {
            return Err(ChunkProjectionError::BlockEntitiesNotOrdered);
        }
        previous = Some(entity.position);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChunkProjectionError {
    #[error("chunk snapshot has {actual} sections, expected {expected}")]
    SectionCount { expected: usize, actual: usize },
    #[error("light section count overflows addressable memory")]
    LightSectionCountOverflow,
    #[error("light snapshot has {sky} sky and {block} block layers, expected {expected} each")]
    LightLayerCount {
        expected: usize,
        sky: usize,
        block: usize,
    },
    #[error("block entity belongs to chunk {actual:?}, expected {expected:?}")]
    WrongBlockEntityChunk {
        expected: ChunkPos,
        actual: ChunkPos,
    },
    #[error("block entity at {position:?} is outside the chunk vertical range")]
    BlockEntityOutsideVerticalRange { position: BlockPos },
    #[error("block entities must be unique and ordered by position")]
    BlockEntitiesNotOrdered,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::{ChunkColumn, VerticalSectionRange};
    use crate::id::BiomeId;

    #[test]
    fn snapshot_contains_all_sections_and_client_heightmaps() {
        let layout = ChunkLayout::new(
            VerticalSectionRange::new(-1, 3).unwrap(),
            BlockStateId::new(0),
            BiomeId::new(1),
        );
        let mut chunk = ChunkColumn::new(ChunkPos::new(2, -3), layout);
        chunk
            .set_uniform_section(-1, BlockStateId::new(1), BiomeId::new(1))
            .unwrap();
        let snapshot = chunk
            .snapshot(
                LightSnapshot::full_sky(layout.sections().count()).unwrap(),
                |_, state| state != BlockStateId::new(0),
            )
            .unwrap();
        assert_eq!(snapshot.sections().len(), 3);
        assert_eq!(snapshot.heightmaps().len(), 3);
        assert!(
            snapshot
                .heightmaps()
                .values()
                .all(|heightmap| heightmap.iter().all(|height| *height == 0))
        );
        assert_eq!(snapshot.light().sky().len(), 5);
    }
}
