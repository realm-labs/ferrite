//! Deterministic same-level and cross-level passenger-graph transfer transactions.

use std::collections::{BTreeMap, BTreeSet};

use super::{Rotation, Vec3};

pub type EntityId = u64;

#[derive(Clone, Debug, PartialEq)]
pub struct TransferEntity {
    pub id: EntityId,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Rotation,
    pub passengers: Vec<EntityId>,
    pub server_player: bool,
    pub destination_type_creatable: bool,
    pub spectator_camera_tracks: Option<EntityId>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransferTarget {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Rotation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferTopology {
    SameLevel,
    CrossLevel,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransferOperation {
    DetachRoot {
        entity: EntityId,
    },
    EjectDirectPassengers {
        vehicle: EntityId,
        passengers: Vec<EntityId>,
    },
    MoveSameLevel {
        entity: EntityId,
        target: TransferTarget,
        as_passenger: bool,
    },
    CreateDimensionTravelInstance {
        old: EntityId,
        new: EntityId,
    },
    RestoreState {
        old: EntityId,
        new: EntityId,
    },
    RemoveChangedDimension {
        entity: EntityId,
    },
    AddToDestination {
        entity: EntityId,
        target: TransferTarget,
        as_passenger: bool,
    },
    MoveServerPlayerInstance {
        entity: EntityId,
        target: TransferTarget,
        as_passenger: bool,
    },
    SynchronizeServerPlayer {
        entity: EntityId,
    },
    RunDimensionCriteriaAndNetherTracking {
        entity: EntityId,
    },
    Remount {
        passenger: EntityId,
        vehicle: EntityId,
    },
    PostTransitionCallback {
        entity: EntityId,
    },
    SpectatorCameraFollow {
        spectator: EntityId,
        target: EntityId,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransferOutcome {
    pub operations: Vec<TransferOperation>,
    pub transferred: BTreeMap<EntityId, EntityId>,
    pub failed_type_creation: BTreeSet<EntityId>,
    pub root_transferred: bool,
    pub old_root_removed: bool,
}

pub fn transfer_passenger_graph(
    entities: &BTreeMap<EntityId, TransferEntity>,
    root: EntityId,
    target: TransferTarget,
    topology: TransferTopology,
) -> TransferOutcome {
    let mut outcome = TransferOutcome::default();
    if !entities.contains_key(&root) {
        return outcome;
    }
    let root_is_passenger = entities
        .values()
        .any(|entity| entity.passengers.contains(&root));
    if !root_is_passenger {
        outcome
            .operations
            .push(TransferOperation::DetachRoot { entity: root });
    }
    let mut next_instance = entities
        .keys()
        .next_back()
        .copied()
        .unwrap_or(0)
        .wrapping_add(1);
    let transferred = match topology {
        TransferTopology::SameLevel => transfer_same_level(
            entities,
            root,
            target,
            false,
            &mut outcome,
            &mut BTreeSet::new(),
        ),
        TransferTopology::CrossLevel => transfer_cross_level(
            entities,
            root,
            target,
            false,
            &mut outcome,
            &mut next_instance,
            &mut BTreeSet::new(),
        ),
    };
    outcome.root_transferred = transferred.is_some();
    outcome.old_root_removed = match topology {
        TransferTopology::SameLevel => false,
        TransferTopology::CrossLevel => {
            transferred.is_some()
                && entities
                    .get(&root)
                    .is_some_and(|entity| !entity.server_player)
        }
    };
    append_camera_follows(entities, &mut outcome);
    outcome
}

fn transfer_same_level(
    entities: &BTreeMap<EntityId, TransferEntity>,
    id: EntityId,
    target: TransferTarget,
    as_passenger: bool,
    outcome: &mut TransferOutcome,
    visiting: &mut BTreeSet<EntityId>,
) -> Option<EntityId> {
    if !visiting.insert(id) {
        return None;
    }
    let entity = entities.get(&id)?;
    for passenger in &entity.passengers {
        if let Some(passenger_entity) = entities.get(passenger) {
            let passenger_target = relative_passenger_target(entity, passenger_entity, target);
            let _ = transfer_same_level(
                entities,
                *passenger,
                passenger_target,
                true,
                outcome,
                visiting,
            );
        }
    }
    outcome.operations.push(TransferOperation::MoveSameLevel {
        entity: id,
        target,
        as_passenger,
    });
    outcome
        .operations
        .push(TransferOperation::PostTransitionCallback { entity: id });
    outcome.transferred.insert(id, id);
    visiting.remove(&id);
    Some(id)
}

fn transfer_cross_level(
    entities: &BTreeMap<EntityId, TransferEntity>,
    id: EntityId,
    target: TransferTarget,
    as_passenger: bool,
    outcome: &mut TransferOutcome,
    next_instance: &mut EntityId,
    visiting: &mut BTreeSet<EntityId>,
) -> Option<EntityId> {
    if !visiting.insert(id) {
        return None;
    }
    let entity = entities.get(&id)?;
    if !entity.passengers.is_empty() {
        outcome
            .operations
            .push(TransferOperation::EjectDirectPassengers {
                vehicle: id,
                passengers: entity.passengers.clone(),
            });
    }
    let mut moved_passengers = Vec::new();
    for passenger in &entity.passengers {
        if let Some(passenger_entity) = entities.get(passenger) {
            let passenger_target = relative_passenger_target(entity, passenger_entity, target);
            if let Some(new_passenger) = transfer_cross_level(
                entities,
                *passenger,
                passenger_target,
                true,
                outcome,
                next_instance,
                visiting,
            ) {
                moved_passengers.push(new_passenger);
            }
        }
    }
    let moved = if entity.server_player {
        outcome
            .operations
            .push(TransferOperation::MoveServerPlayerInstance {
                entity: id,
                target,
                as_passenger,
            });
        outcome
            .operations
            .push(TransferOperation::SynchronizeServerPlayer { entity: id });
        outcome
            .operations
            .push(TransferOperation::RunDimensionCriteriaAndNetherTracking { entity: id });
        id
    } else {
        if !entity.destination_type_creatable {
            outcome.failed_type_creation.insert(id);
            visiting.remove(&id);
            return None;
        }
        let new = *next_instance;
        *next_instance = next_instance.wrapping_add(1);
        outcome
            .operations
            .push(TransferOperation::CreateDimensionTravelInstance { old: id, new });
        outcome
            .operations
            .push(TransferOperation::RestoreState { old: id, new });
        outcome
            .operations
            .push(TransferOperation::RemoveChangedDimension { entity: id });
        outcome
            .operations
            .push(TransferOperation::AddToDestination {
                entity: new,
                target,
                as_passenger,
            });
        new
    };
    for passenger in moved_passengers {
        outcome.operations.push(TransferOperation::Remount {
            passenger,
            vehicle: moved,
        });
    }
    outcome
        .operations
        .push(TransferOperation::PostTransitionCallback { entity: moved });
    outcome.transferred.insert(id, moved);
    visiting.remove(&id);
    Some(moved)
}

fn relative_passenger_target(
    vehicle: &TransferEntity,
    passenger: &TransferEntity,
    vehicle_target: TransferTarget,
) -> TransferTarget {
    TransferTarget {
        position: Vec3 {
            x: vehicle_target.position.x + passenger.position.x - vehicle.position.x,
            y: vehicle_target.position.y + passenger.position.y - vehicle.position.y,
            z: vehicle_target.position.z + passenger.position.z - vehicle.position.z,
        },
        velocity: passenger.velocity,
        rotation: Rotation {
            yaw: vehicle_target.rotation.yaw + passenger.rotation.yaw - vehicle.rotation.yaw,
            pitch: vehicle_target.rotation.pitch + passenger.rotation.pitch
                - vehicle.rotation.pitch,
        },
    }
}

fn append_camera_follows(
    entities: &BTreeMap<EntityId, TransferEntity>,
    outcome: &mut TransferOutcome,
) {
    for spectator in entities.values().filter(|entity| entity.server_player) {
        let Some(old_target) = spectator.spectator_camera_tracks else {
            continue;
        };
        let Some(new_target) = outcome.transferred.get(&old_target).copied() else {
            continue;
        };
        outcome
            .operations
            .push(TransferOperation::SpectatorCameraFollow {
                spectator: spectator.id,
                target: new_target,
            });
    }
}
