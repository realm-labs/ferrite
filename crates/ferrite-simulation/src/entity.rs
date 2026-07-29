//! Region-local ECS ownership without exposing runtime entity handles.

use bevy_ecs::component::Mutable;
use bevy_ecs::prelude::{Component, Entity, World};
use ferrite_foundation::identity::{DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::SimulationRegionKey;
use std::any::TypeId;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
#[component(immutable)]
pub struct StableIdentity(StableEntityId);

impl StableIdentity {
    pub const fn new(id: StableEntityId) -> Self {
        Self(id)
    }

    pub const fn get(self) -> StableEntityId {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Component)]
#[component(immutable)]
pub struct InWorldDimension {
    world: WorldId,
    dimension: DimensionId,
}

impl InWorldDimension {
    pub const fn new(world: WorldId, dimension: DimensionId) -> Self {
        Self { world, dimension }
    }

    pub const fn world(&self) -> WorldId {
        self.world
    }

    pub const fn dimension(&self) -> &DimensionId {
        &self.dimension
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Component)]
#[component(immutable)]
pub struct RegionMembership(SimulationRegionKey);

impl RegionMembership {
    pub const fn new(region: SimulationRegionKey) -> Self {
        Self(region)
    }

    pub const fn region(&self) -> &SimulationRegionKey {
        &self.0
    }
}

pub struct RegionEntityPartition {
    key: SimulationRegionKey,
    world: World,
    identities: BTreeMap<StableEntityId, Entity>,
}

impl RegionEntityPartition {
    pub fn new(key: SimulationRegionKey) -> Self {
        Self {
            key,
            world: World::new(),
            identities: BTreeMap::new(),
        }
    }

    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    pub fn len(&self) -> usize {
        self.identities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.identities.is_empty()
    }

    pub fn view(&self) -> RegionEntityView<'_> {
        RegionEntityView { partition: self }
    }

    pub fn spawn(&mut self, stable_id: StableEntityId) -> Result<(), RegionEntityError> {
        if self.identities.contains_key(&stable_id) {
            return Err(RegionEntityError::DuplicateStableId(stable_id));
        }
        let entity = self
            .world
            .spawn((
                StableIdentity::new(stable_id),
                InWorldDimension::new(self.key.world(), self.key.dimension().clone()),
                RegionMembership::new(self.key.clone()),
            ))
            .id();
        self.identities.insert(stable_id, entity);
        Ok(())
    }

    pub fn despawn(&mut self, stable_id: StableEntityId) -> Result<(), RegionEntityError> {
        let entity = self.entity(stable_id)?;
        if self.world.despawn(entity) {
            self.identities.remove(&stable_id);
            Ok(())
        } else {
            Err(RegionEntityError::IdentityMapCorrupt(stable_id))
        }
    }

    pub fn insert_component<T: Component>(
        &mut self,
        stable_id: StableEntityId,
        component: T,
    ) -> Result<(), RegionEntityError> {
        if protected_component::<T>() {
            return Err(RegionEntityError::ProtectedComponent(
                std::any::type_name::<T>(),
            ));
        }
        let entity = self.entity(stable_id)?;
        self.world.entity_mut(entity).insert(component);
        Ok(())
    }

    pub fn update_component<T: Component<Mutability = Mutable>, R>(
        &mut self,
        stable_id: StableEntityId,
        update: impl FnOnce(&mut T) -> R,
    ) -> Result<R, RegionEntityError> {
        let entity = self.entity(stable_id)?;
        let mut component =
            self.world
                .get_mut::<T>(entity)
                .ok_or(RegionEntityError::MissingComponent {
                    stable_id,
                    component: std::any::type_name::<T>(),
                })?;
        Ok(update(&mut component))
    }

    fn entity(&self, stable_id: StableEntityId) -> Result<Entity, RegionEntityError> {
        self.identities
            .get(&stable_id)
            .copied()
            .ok_or(RegionEntityError::UnknownStableId(stable_id))
    }
}

fn protected_component<T: Component>() -> bool {
    let component = TypeId::of::<T>();
    component == TypeId::of::<StableIdentity>()
        || component == TypeId::of::<InWorldDimension>()
        || component == TypeId::of::<RegionMembership>()
}

#[derive(Clone, Copy)]
pub struct RegionEntityView<'a> {
    partition: &'a RegionEntityPartition,
}

impl<'a> RegionEntityView<'a> {
    pub const fn key(self) -> &'a SimulationRegionKey {
        &self.partition.key
    }

    pub fn contains(self, stable_id: StableEntityId) -> bool {
        self.partition.identities.contains_key(&stable_id)
    }

    pub fn component<T: Component>(self, stable_id: StableEntityId) -> Option<&'a T> {
        let entity = self.partition.identities.get(&stable_id)?;
        self.partition.world.get::<T>(*entity)
    }

    pub fn stable_ids(self) -> impl ExactSizeIterator<Item = StableEntityId> + 'a {
        self.partition.identities.keys().copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegionEntityError {
    #[error("stable entity {0} already exists in this Region")]
    DuplicateStableId(StableEntityId),
    #[error("stable entity {0} does not exist in this Region")]
    UnknownStableId(StableEntityId),
    #[error("stable entity {stable_id} has no component {component}")]
    MissingComponent {
        stable_id: StableEntityId,
        component: &'static str,
    },
    #[error("stable entity {0} was mapped to a missing ECS entity")]
    IdentityMapCorrupt(StableEntityId),
    #[error("Region ownership component {0} cannot be inserted or replaced")]
    ProtectedComponent(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};
    use ferrite_foundation::resource::ResourceId;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
    struct Health(u32);

    fn key(coordinate: RegionCoord) -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            coordinate,
            RegionMappingVersion::V1,
        )
    }

    #[test]
    fn stable_ids_hide_ecs_handles_and_iterate_deterministically() {
        let mut entities = RegionEntityPartition::new(key(RegionCoord::new(0, 0)));
        let first = StableEntityId::new(2).unwrap();
        let second = StableEntityId::new(1).unwrap();
        entities.spawn(first).unwrap();
        entities.insert_component(first, Health(20)).unwrap();
        entities.spawn(second).unwrap();
        entities.insert_component(second, Health(10)).unwrap();
        let view = entities.view();
        assert_eq!(view.stable_ids().collect::<Vec<_>>(), [second, first]);
        assert_eq!(view.component::<Health>(first), Some(&Health(20)));
        assert_eq!(
            view.component::<RegionMembership>(first).unwrap().region(),
            entities.key()
        );
    }

    #[test]
    fn duplicate_spawns_and_missing_components_fail_without_aliasing() {
        let mut entities = RegionEntityPartition::new(key(RegionCoord::new(0, 0)));
        let id = StableEntityId::new(1).unwrap();
        entities.spawn(id).unwrap();
        entities.insert_component(id, Health(20)).unwrap();
        assert!(entities.spawn(id).is_err());
        assert!(
            entities
                .insert_component(id, StableIdentity::new(id))
                .is_err()
        );
        assert_eq!(
            entities.update_component::<Health, _>(id, |health| {
                health.0 -= 3;
                health.0
            }),
            Ok(17)
        );
        assert!(
            entities
                .update_component::<Health, _>(StableEntityId::new(2).unwrap(), |_| ())
                .is_err()
        );
        assert_eq!(entities.len(), 1);
    }

    #[test]
    fn partitions_do_not_share_runtime_entities() {
        let id = StableEntityId::new(1).unwrap();
        let mut left = RegionEntityPartition::new(key(RegionCoord::new(0, 0)));
        let right = RegionEntityPartition::new(key(RegionCoord::new(1, 0)));
        left.spawn(id).unwrap();
        left.insert_component(id, Health(20)).unwrap();
        assert!(left.view().contains(id));
        assert!(!right.view().contains(id));
    }

    #[test]
    fn despawn_removes_stable_identity_atomically() {
        let id = StableEntityId::new(1).unwrap();
        let mut entities = RegionEntityPartition::new(key(RegionCoord::new(0, 0)));
        entities.spawn(id).unwrap();
        entities.despawn(id).unwrap();
        assert!(!entities.view().contains(id));
        assert!(entities.despawn(id).is_err());
    }
}
