use std::collections::BTreeMap;

use ferrite_world::generation::portal::transfer::{
    TransferEntity, TransferOperation, TransferTarget, TransferTopology, transfer_passenger_graph,
};
use ferrite_world::generation::portal::{Rotation, Vec3};

fn entity(id: u64, passengers: Vec<u64>) -> TransferEntity {
    TransferEntity {
        id,
        position: Vec3 {
            x: id as f64,
            y: 10.0 + id as f64,
            z: -(id as f64),
        },
        velocity: Vec3 {
            x: 0.1 * id as f64,
            y: 0.0,
            z: 0.0,
        },
        rotation: Rotation {
            yaw: id as f32 * 10.0,
            pitch: id as f32,
        },
        passengers,
        server_player: false,
        destination_type_creatable: true,
        spectator_camera_tracks: None,
    }
}

fn target() -> TransferTarget {
    TransferTarget {
        position: Vec3 {
            x: 100.0,
            y: 70.0,
            z: 200.0,
        },
        velocity: Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        rotation: Rotation {
            yaw: 90.0,
            pitch: 20.0,
        },
    }
}

#[test]
fn same_level_moves_nested_passengers_before_root_with_relative_poses_and_callbacks() {
    let entities = BTreeMap::from([
        (1, entity(1, vec![2])),
        (2, entity(2, vec![3])),
        (3, entity(3, Vec::new())),
    ]);
    let outcome = transfer_passenger_graph(&entities, 1, target(), TransferTopology::SameLevel);
    assert!(outcome.root_transferred && !outcome.old_root_removed);
    assert_eq!(
        outcome.transferred,
        BTreeMap::from([(1, 1), (2, 2), (3, 3)])
    );
    let moved = outcome
        .operations
        .iter()
        .filter_map(|operation| match operation {
            TransferOperation::MoveSameLevel {
                entity,
                target,
                as_passenger,
            } => Some((*entity, *target, *as_passenger)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        moved.iter().map(|entry| entry.0).collect::<Vec<_>>(),
        [3, 2, 1]
    );
    assert!(moved[0].2 && moved[1].2 && !moved[2].2);
    assert_eq!(
        moved[1].1.position,
        Vec3 {
            x: 101.0,
            y: 71.0,
            z: 199.0
        }
    );
    assert_eq!(
        moved[1].1.rotation,
        Rotation {
            yaw: 100.0,
            pitch: 21.0
        }
    );
    let callbacks = outcome
        .operations
        .iter()
        .filter(|operation| matches!(operation, TransferOperation::PostTransitionCallback { .. }))
        .count();
    assert_eq!(callbacks, 3);
}

#[test]
fn cross_level_ejects_then_recurses_creates_restores_removes_adds_and_remounts() {
    let entities = BTreeMap::from([(1, entity(1, vec![2])), (2, entity(2, Vec::new()))]);
    let outcome = transfer_passenger_graph(&entities, 1, target(), TransferTopology::CrossLevel);
    assert!(outcome.root_transferred && outcome.old_root_removed);
    let new_passenger = outcome.transferred[&2];
    let new_root = outcome.transferred[&1];
    let eject_index = outcome
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                TransferOperation::EjectDirectPassengers { vehicle: 1, .. }
            )
        })
        .unwrap();
    let passenger_create = outcome
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                TransferOperation::CreateDimensionTravelInstance { old: 2, .. }
            )
        })
        .unwrap();
    let root_create = outcome
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                TransferOperation::CreateDimensionTravelInstance { old: 1, .. }
            )
        })
        .unwrap();
    let remount = outcome.operations.iter().position(|operation| matches!(operation, TransferOperation::Remount { passenger, vehicle } if *passenger == new_passenger && *vehicle == new_root)).unwrap();
    assert!(
        eject_index < passenger_create && passenger_create < root_create && root_create < remount
    );
    assert!(outcome.operations.iter().any(|operation| matches!(operation, TransferOperation::AddToDestination { entity, as_passenger: true, .. } if *entity == new_passenger)));
    assert!(outcome.operations.iter().any(|operation| matches!(operation, TransferOperation::AddToDestination { entity, as_passenger: false, .. } if *entity == new_root)));
}

#[test]
fn failed_root_creation_leaves_old_ejected_root_after_passengers_already_transfer() {
    let mut root = entity(1, vec![2]);
    root.destination_type_creatable = false;
    let entities = BTreeMap::from([(1, root), (2, entity(2, Vec::new()))]);
    let outcome = transfer_passenger_graph(&entities, 1, target(), TransferTopology::CrossLevel);
    assert!(!outcome.root_transferred && !outcome.old_root_removed);
    assert!(outcome.failed_type_creation.contains(&1));
    assert!(outcome.transferred.contains_key(&2));
    assert!(!outcome.transferred.contains_key(&1));
    assert!(
        !outcome
            .operations
            .iter()
            .any(|operation| matches!(operation, TransferOperation::Remount { .. }))
    );
}

#[test]
fn server_player_moves_same_instance_synchronizes_and_moves_spectator_camera() {
    let mut root = entity(1, Vec::new());
    root.server_player = true;
    let mut spectator = entity(2, Vec::new());
    spectator.server_player = true;
    spectator.spectator_camera_tracks = Some(1);
    let entities = BTreeMap::from([(1, root), (2, spectator)]);
    let outcome = transfer_passenger_graph(&entities, 1, target(), TransferTopology::CrossLevel);
    assert_eq!(outcome.transferred[&1], 1);
    assert!(!outcome.old_root_removed);
    assert!(outcome.operations.iter().any(|operation| matches!(
        operation,
        TransferOperation::MoveServerPlayerInstance {
            entity: 1,
            as_passenger: false,
            ..
        }
    )));
    assert!(outcome.operations.iter().any(|operation| matches!(
        operation,
        TransferOperation::SynchronizeServerPlayer { entity: 1 }
    )));
    assert!(outcome.operations.iter().any(|operation| matches!(
        operation,
        TransferOperation::RunDimensionCriteriaAndNetherTracking { entity: 1 }
    )));
    assert!(outcome.operations.iter().any(|operation| matches!(
        operation,
        TransferOperation::SpectatorCameraFollow {
            spectator: 2,
            target: 1
        }
    )));
}

#[test]
fn passenger_root_skips_detach_and_cycles_fail_closed() {
    let entities = BTreeMap::from([(1, entity(1, vec![2])), (2, entity(2, vec![1]))]);
    let outcome = transfer_passenger_graph(&entities, 2, target(), TransferTopology::SameLevel);
    assert!(!matches!(
        outcome.operations.first(),
        Some(TransferOperation::DetachRoot { .. })
    ));
    assert!(outcome.operations.len() < 10);
}

#[test]
fn missing_root_is_a_noop() {
    let outcome =
        transfer_passenger_graph(&BTreeMap::new(), 99, target(), TransferTopology::CrossLevel);
    assert_eq!(outcome, Default::default());
}
