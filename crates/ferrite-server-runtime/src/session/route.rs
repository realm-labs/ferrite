use std::collections::BTreeMap;

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{DimensionId, WorldId};
use ferrite_foundation::region::{RegionMapping, SimulationRegionKey};
use ferrite_protocol::semantic::VirtualHost;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialWorldRoute {
    pub world: WorldId,
    pub dimension: DimensionId,
    pub spawn_chunk: ChunkPos,
    pub mapping: RegionMapping,
}

impl InitialWorldRoute {
    #[must_use]
    pub fn region(&self) -> SimulationRegionKey {
        self.mapping
            .region_for_chunk(self.world, self.dimension.clone(), self.spawn_chunk)
    }
}

#[derive(Debug, Clone)]
pub struct VirtualHostRoutes {
    fallback: InitialWorldRoute,
    maximum_routes: usize,
    exact: BTreeMap<(String, u16), InitialWorldRoute>,
}

impl VirtualHostRoutes {
    pub fn new(
        fallback: InitialWorldRoute,
        maximum_routes: usize,
    ) -> Result<Self, RouteTableError> {
        if maximum_routes == 0 {
            return Err(RouteTableError::ZeroCapacity);
        }
        Ok(Self {
            fallback,
            maximum_routes,
            exact: BTreeMap::new(),
        })
    }

    pub fn insert(
        &mut self,
        host: String,
        port: u16,
        route: InitialWorldRoute,
    ) -> Result<(), RouteTableError> {
        let key = (host, port);
        if self.exact.contains_key(&key) {
            return Err(RouteTableError::Duplicate { host: key.0, port });
        }
        if self.exact.len() == self.maximum_routes {
            return Err(RouteTableError::Full {
                capacity: self.maximum_routes,
            });
        }
        self.exact.insert(key, route);
        Ok(())
    }

    #[must_use]
    pub fn resolve(&self, host: &VirtualHost) -> &InitialWorldRoute {
        self.exact
            .get(&(host.host.clone(), host.port))
            .unwrap_or(&self.fallback)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RouteTableError {
    #[error("virtual-host route capacity cannot be zero")]
    ZeroCapacity,
    #[error("virtual-host route table reached its {capacity}-route bound")]
    Full { capacity: usize },
    #[error("virtual-host route {host}:{port} is duplicated")]
    Duplicate { host: String, port: u16 },
}
