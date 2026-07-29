//! Canonical Region ring and node assignments used by every topology.

use crate::topology::TopologyError;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyRegionDescriptor {
    pub key: SimulationRegionKey,
    pub generation: ActivationGeneration,
    pub node: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyLayout {
    descriptors: Vec<TopologyRegionDescriptor>,
    node_count: u16,
}

impl TopologyLayout {
    pub fn ring(region_count: u16, node_count: u16) -> Result<Self, TopologyError> {
        if node_count == 0 {
            return Err(TopologyError::ZeroNodes);
        }
        if region_count < 2 {
            return Err(TopologyError::EmptyLayout);
        }
        let world = WorldId::new(1).expect("the locked topology world ID is non-zero");
        let dimension = DimensionId::new(
            ResourceId::minecraft("overworld")
                .expect("the locked topology dimension is a valid resource ID"),
        );
        let descriptors = (0..region_count)
            .map(|index| TopologyRegionDescriptor {
                key: SimulationRegionKey::new(
                    world,
                    dimension.clone(),
                    RegionCoord::new(i32::from(index), 0),
                    RegionMappingVersion::V1,
                ),
                generation: ActivationGeneration::INITIAL,
                node: index % node_count,
            })
            .collect();
        Self::new(descriptors, node_count)
    }

    pub fn new(
        descriptors: Vec<TopologyRegionDescriptor>,
        node_count: u16,
    ) -> Result<Self, TopologyError> {
        if node_count == 0 {
            return Err(TopologyError::ZeroNodes);
        }
        if descriptors.len() < 2 {
            return Err(TopologyError::EmptyLayout);
        }
        let mut indexed = Vec::with_capacity(descriptors.len());
        for descriptor in descriptors {
            if descriptor.node >= node_count {
                return Err(TopologyError::UnknownNode(descriptor.node));
            }
            indexed.push(descriptor);
        }
        indexed.sort_by(|left, right| left.key.cmp(&right.key));
        if let Some(pair) = indexed.windows(2).find(|pair| pair[0].key == pair[1].key) {
            return Err(TopologyError::DuplicateRegion(pair[0].key.clone()));
        }
        Ok(Self {
            descriptors: indexed,
            node_count,
        })
    }

    pub const fn node_count(&self) -> u16 {
        self.node_count
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &TopologyRegionDescriptor> {
        self.descriptors.iter()
    }

    pub fn descriptor(
        &self,
        key: &SimulationRegionKey,
    ) -> Result<&TopologyRegionDescriptor, TopologyError> {
        self.descriptors
            .binary_search_by(|descriptor| descriptor.key.cmp(key))
            .ok()
            .and_then(|index| self.descriptors.get(index))
            .ok_or_else(|| TopologyError::UnknownRegion(key.clone()))
    }

    pub fn successor(
        &self,
        key: &SimulationRegionKey,
    ) -> Result<&TopologyRegionDescriptor, TopologyError> {
        let index = self
            .descriptors
            .binary_search_by(|descriptor| descriptor.key.cmp(key))
            .map_err(|_| TopologyError::UnknownRegion(key.clone()))?;
        self.descriptors
            .get((index + 1) % self.descriptors.len())
            .ok_or_else(|| TopologyError::UnknownRegion(key.clone()))
    }

    pub fn predecessor(
        &self,
        key: &SimulationRegionKey,
    ) -> Result<&TopologyRegionDescriptor, TopologyError> {
        let index = self
            .descriptors
            .binary_search_by(|descriptor| descriptor.key.cmp(key))
            .map_err(|_| TopologyError::UnknownRegion(key.clone()))?;
        self.descriptors
            .get((index + self.descriptors.len() - 1) % self.descriptors.len())
            .ok_or_else(|| TopologyError::UnknownRegion(key.clone()))
    }

    pub fn with_all_on_node(&self, node: u16) -> Result<Self, TopologyError> {
        if node >= self.node_count {
            return Err(TopologyError::UnknownNode(node));
        }
        let descriptors = self
            .descriptors()
            .cloned()
            .map(|mut descriptor| {
                descriptor.node = node;
                descriptor
            })
            .collect();
        Self::new(descriptors, self.node_count)
    }

    pub fn recover_node(&self, failed: u16, survivor: u16) -> Result<Self, TopologyError> {
        if failed >= self.node_count {
            return Err(TopologyError::UnknownNode(failed));
        }
        if survivor >= self.node_count {
            return Err(TopologyError::UnknownNode(survivor));
        }
        let descriptors = self
            .descriptors()
            .cloned()
            .map(|mut descriptor| {
                if descriptor.node == failed {
                    descriptor.node = survivor;
                    descriptor.generation = descriptor
                        .generation
                        .checked_next()
                        .map_err(|_| TopologyError::GenerationExhausted)?;
                }
                Ok(descriptor)
            })
            .collect::<Result<Vec<_>, TopologyError>>()?;
        Self::new(descriptors, self.node_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_order_and_recovery_are_canonical() {
        let layout = TopologyLayout::ring(4, 3).unwrap();
        let keys = layout
            .descriptors()
            .map(|descriptor| descriptor.key.clone())
            .collect::<Vec<_>>();
        assert_eq!(layout.successor(&keys[3]).unwrap().key, keys[0]);
        assert_eq!(layout.predecessor(&keys[0]).unwrap().key, keys[3]);

        let recovered = layout.recover_node(1, 2).unwrap();
        let moved = recovered.descriptor(&keys[1]).unwrap();
        assert_eq!(moved.node, 2);
        assert_eq!(moved.generation.get(), 2);
    }
}
