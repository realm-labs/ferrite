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
        Self::multiverse_ring(region_count, node_count, 1)
    }

    pub fn multiverse_ring(
        region_count: u16,
        node_count: u16,
        world_count: u16,
    ) -> Result<Self, TopologyError> {
        if node_count == 0 {
            return Err(TopologyError::ZeroNodes);
        }
        if region_count < 2 {
            return Err(TopologyError::EmptyLayout);
        }
        if world_count == 0 || region_count / world_count < 2 {
            return Err(TopologyError::InvalidWorldCount);
        }
        let dimension = DimensionId::new(
            ResourceId::minecraft("overworld")
                .expect("the locked topology dimension is a valid resource ID"),
        );
        let descriptors = (0..region_count)
            .map(|index| {
                let world_index = index % world_count;
                let coordinate = index / world_count;
                TopologyRegionDescriptor {
                    key: SimulationRegionKey::new(
                        WorldId::new(u128::from(world_index) + 1)
                            .expect("the topology world index is non-zero"),
                        dimension.clone(),
                        RegionCoord::new(i32::from(coordinate), 0),
                        RegionMappingVersion::V1,
                    ),
                    generation: ActivationGeneration::INITIAL,
                    node: (coordinate + world_index) % node_count,
                }
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
        let mut domain_start = 0;
        while domain_start < indexed.len() {
            let domain_end = indexed[domain_start + 1..]
                .iter()
                .position(|descriptor| !same_domain(&indexed[domain_start].key, &descriptor.key))
                .map_or(indexed.len(), |offset| domain_start + 1 + offset);
            if domain_end - domain_start < 2 {
                return Err(TopologyError::SingletonRegionDomain);
            }
            domain_start = domain_end;
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
        if let Some(next) = self.descriptors.get(index + 1)
            && same_domain(key, &next.key)
        {
            return Ok(next);
        }
        self.descriptors[..=index]
            .iter()
            .find(|descriptor| same_domain(key, &descriptor.key))
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
        if index > 0
            && let Some(previous) = self.descriptors.get(index - 1)
            && same_domain(key, &previous.key)
        {
            return Ok(previous);
        }
        self.descriptors[index..]
            .iter()
            .rfind(|descriptor| same_domain(key, &descriptor.key))
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

fn same_domain(left: &SimulationRegionKey, right: &SimulationRegionKey) -> bool {
    left.world() == right.world()
        && left.dimension() == right.dimension()
        && left.mapping_version() == right.mapping_version()
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

    #[test]
    fn independent_world_domains_wrap_without_crossing_endpoints() {
        let first = TopologyLayout::ring(4, 2).unwrap();
        let second_world = WorldId::new(2).unwrap();
        let mut descriptors = first.descriptors().cloned().collect::<Vec<_>>();
        descriptors.extend(first.descriptors().cloned().map(|mut descriptor| {
            descriptor.key = SimulationRegionKey::new(
                second_world,
                descriptor.key.dimension().clone(),
                descriptor.key.coordinate(),
                descriptor.key.mapping_version(),
            );
            descriptor
        }));
        let layout = TopologyLayout::new(descriptors, 2).unwrap();
        let keys = layout
            .descriptors()
            .map(|descriptor| descriptor.key.clone())
            .collect::<Vec<_>>();
        assert_eq!(layout.successor(&keys[3]).unwrap().key, keys[0]);
        assert_eq!(layout.predecessor(&keys[4]).unwrap().key, keys[7]);
    }

    #[test]
    fn multiverse_ring_balances_worlds_across_nodes() {
        let layout = TopologyLayout::multiverse_ring(8, 3, 2).unwrap();
        let counts = (0..3)
            .map(|node| {
                layout
                    .descriptors()
                    .filter(|descriptor| descriptor.node == node)
                    .count()
            })
            .collect::<Vec<_>>();
        assert_eq!(counts, vec![3, 3, 2]);
        assert!(matches!(
            TopologyLayout::multiverse_ring(3, 2, 2),
            Err(TopologyError::InvalidWorldCount)
        ));
    }

    #[test]
    fn singleton_world_domain_is_rejected() {
        let mut descriptors = TopologyLayout::ring(2, 2)
            .unwrap()
            .descriptors()
            .cloned()
            .collect::<Vec<_>>();
        descriptors[1].key = SimulationRegionKey::new(
            WorldId::new(2).unwrap(),
            descriptors[1].key.dimension().clone(),
            descriptors[1].key.coordinate(),
            descriptors[1].key.mapping_version(),
        );
        assert!(matches!(
            TopologyLayout::new(descriptors, 2),
            Err(TopologyError::SingletonRegionDomain)
        ));
    }
}
