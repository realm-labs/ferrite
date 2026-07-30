use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_server_runtime::phase6::model::{
    ActionOutcome, PlayerActionHeader, PlayerMutation, PlayerPayload, PlayerPersistentState,
    ProjectionKind, ResyncReason,
};
use ferrite_server_runtime::phase6::runtime::{Phase6RegionRuntime, Phase6RuntimeError};

fn region() -> SimulationRegionKey {
    SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(0, 0),
        RegionMappingVersion::V1,
    )
}

fn player(value: u128) -> StableEntityId {
    StableEntityId::new(value).unwrap()
}

fn runtime(projection_capacity: usize) -> Phase6RegionRuntime {
    Phase6RegionRuntime::new(
        region(),
        ActivationGeneration::INITIAL,
        8,
        projection_capacity,
    )
    .unwrap()
}

fn header(player: StableEntityId, session_epoch: u64, sequence: u64) -> PlayerActionHeader {
    PlayerActionHeader {
        region: region(),
        generation: ActivationGeneration::INITIAL,
        player,
        session_epoch,
        sequence,
    }
}

fn mutation(state: &PlayerPersistentState, marker: u8) -> PlayerMutation {
    PlayerMutation {
        inventory: PlayerPayload::new(vec![marker; usize::from(marker) + 1]).unwrap(),
        selected_slot: marker % 9,
        experience_points: u32::from(marker) * 10,
        experience_level: u32::from(marker),
        food_level: 20 - i32::from(marker.min(20)),
        saturation_bits: f32::from(marker).to_bits(),
        exhaustion_bits: (f32::from(marker) / 2.0).to_bits(),
        progression: PlayerPayload::new(vec![marker.wrapping_add(1)]).unwrap(),
        ..PlayerMutation::from_state(state)
    }
}

fn join_and_drain(runtime: &mut Phase6RegionRuntime, player: StableEntityId) -> u64 {
    let epoch = runtime
        .join(player, PlayerPersistentState::default())
        .unwrap();
    let initial = runtime.drain_projections(player, usize::MAX).unwrap();
    assert!(matches!(
        initial.as_slice(),
        [projection]
            if projection.player == player
                && projection.session_epoch == epoch
                && matches!(
                    projection.kind,
                    ProjectionKind::FullState {
                        reason: ResyncReason::Join,
                        ..
                    }
                )
    ));
    epoch
}

#[test]
fn region_generation_session_and_player_ownership_fail_before_mutation() {
    let owner = player(1);
    let stranger = player(2);
    let mut runtime = runtime(4);
    let epoch = join_and_drain(&mut runtime, owner);
    let before = runtime.state(owner).unwrap();

    let mut wrong_region = header(owner, epoch, 1);
    wrong_region.region = SimulationRegionKey::new(
        WorldId::new(1).unwrap(),
        DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
        RegionCoord::new(1, 0),
        RegionMappingVersion::V1,
    );
    assert!(matches!(
        runtime.apply_player_action(&wrong_region, mutation(&before, 1)),
        Err(Phase6RuntimeError::WrongRegion)
    ));

    let mut stale_generation = header(owner, epoch, 1);
    stale_generation.generation = ActivationGeneration::new(2).unwrap();
    assert!(matches!(
        runtime.apply_player_action(&stale_generation, mutation(&before, 1)),
        Err(Phase6RuntimeError::StaleGeneration { .. })
    ));
    assert!(matches!(
        runtime.apply_player_action(&header(owner, epoch + 1, 1), mutation(&before, 1)),
        Err(Phase6RuntimeError::StaleSession { .. })
    ));
    assert!(matches!(
        runtime.apply_player_action(&header(stranger, 1, 1), mutation(&before, 1)),
        Err(Phase6RuntimeError::UnknownPlayer(id)) if id == stranger
    ));
    assert_eq!(runtime.state(owner), Some(before));
    assert_eq!(runtime.projection_len(owner), Some(0));
}

#[test]
fn ordered_actions_commit_once_and_inventory_revision_mismatch_resynchronizes() {
    let owner = player(1);
    let mut runtime = runtime(8);
    let epoch = join_and_drain(&mut runtime, owner);
    let initial = runtime.state(owner).unwrap();
    let committed = runtime
        .apply_player_action(&header(owner, epoch, 1), mutation(&initial, 3))
        .unwrap();
    assert!(matches!(
        committed,
        ActionOutcome::Committed {
            full_resync: false,
            ..
        }
    ));
    let state = runtime.state(owner).unwrap();
    assert_eq!(state.inventory_revision, 1);
    assert_eq!(state.inventory.bytes(), &[3; 4]);
    assert_eq!(state.last_action_sequence, 1);
    assert_eq!(
        runtime
            .apply_player_action(&header(owner, epoch, 1), mutation(&state, 4))
            .unwrap(),
        ActionOutcome::AlreadyApplied
    );
    assert!(matches!(
        runtime.apply_player_action(&header(owner, epoch, 3), mutation(&state, 4)),
        Err(Phase6RuntimeError::ActionSequenceGap {
            expected: 2,
            actual: 3
        })
    ));

    let mut stale = mutation(&state, 4);
    stale.expected_inventory_revision = 0;
    let rejected = runtime
        .apply_player_action(&header(owner, epoch, 2), stale)
        .unwrap();
    assert!(matches!(
        rejected,
        ActionOutcome::RejectedAndResynchronized {
            reason: ResyncReason::InventoryRevision,
            ..
        }
    ));
    let unchanged = runtime.state(owner).unwrap();
    assert_eq!(unchanged.inventory_revision, 1);
    assert_eq!(unchanged.inventory.bytes(), &[3; 4]);
    assert_eq!(unchanged.last_action_sequence, 2);
    assert!(
        runtime
            .drain_projections(owner, usize::MAX)
            .unwrap()
            .iter()
            .any(|projection| matches!(
                projection.kind,
                ProjectionKind::FullState {
                    reason: ResyncReason::InventoryRevision,
                    inventory_revision: 1,
                    ..
                }
            ))
    );
}

#[test]
fn stale_menu_state_commits_then_full_syncs_while_wrong_container_is_ignored() {
    let owner = player(1);
    let mut runtime = runtime(8);
    let epoch = join_and_drain(&mut runtime, owner);
    runtime.open_menu(&header(owner, epoch, 1), 12).unwrap();
    let initial = runtime.state(owner).unwrap();

    let stale = runtime
        .apply_menu_action(&header(owner, epoch, 1), 12, 9, mutation(&initial, 5))
        .unwrap();
    assert!(matches!(
        stale,
        ActionOutcome::Committed {
            full_resync: true,
            ..
        }
    ));
    assert_eq!(runtime.menu(owner).unwrap().state_id, 1);
    assert!(
        runtime
            .drain_projections(owner, usize::MAX)
            .unwrap()
            .iter()
            .any(|projection| matches!(
                projection.kind,
                ProjectionKind::FullState {
                    reason: ResyncReason::MenuState,
                    menu: Some(menu),
                    ..
                } if menu.container_id == 12 && menu.state_id == 1
            ))
    );

    let state = runtime.state(owner).unwrap();
    assert_eq!(
        runtime
            .apply_menu_action(&header(owner, epoch, 2), 13, 1, mutation(&state, 6))
            .unwrap(),
        ActionOutcome::IgnoredWrongContainer
    );
    let unchanged = runtime.state(owner).unwrap();
    assert_eq!(unchanged.inventory.bytes(), &[5; 6]);
    assert_eq!(unchanged.last_action_sequence, 2);
    assert_eq!(runtime.projection_len(owner), Some(0));
}

#[test]
fn per_player_projection_capacity_is_atomic_and_never_blocks_another_player() {
    let first = player(1);
    let second = player(2);
    let mut runtime = runtime(1);
    let first_epoch = join_and_drain(&mut runtime, first);
    let second_epoch = join_and_drain(&mut runtime, second);

    let first_initial = runtime.state(first).unwrap();
    runtime
        .apply_player_action(&header(first, first_epoch, 1), mutation(&first_initial, 1))
        .unwrap();
    let first_committed = runtime.state(first).unwrap();
    assert!(matches!(
        runtime.apply_player_action(
            &header(first, first_epoch, 2),
            mutation(&first_committed, 2)
        ),
        Err(Phase6RuntimeError::ProjectionCapacity {
            player: id,
            capacity: 1
        }) if id == first
    ));
    assert_eq!(runtime.state(first), Some(first_committed));

    let second_initial = runtime.state(second).unwrap();
    runtime
        .apply_player_action(
            &header(second, second_epoch, 1),
            mutation(&second_initial, 7),
        )
        .unwrap();
    assert_eq!(runtime.state(second).unwrap().inventory.bytes(), &[7; 8]);
    assert_eq!(runtime.projection_len(first), Some(1));
    assert_eq!(runtime.projection_len(second), Some(1));
}

#[test]
fn invalid_player_fields_do_not_consume_action_or_projection_revisions() {
    let owner = player(1);
    let mut runtime = runtime(4);
    let epoch = join_and_drain(&mut runtime, owner);
    let initial = runtime.state(owner).unwrap();
    let mut invalid = mutation(&initial, 1);
    invalid.selected_slot = 9;
    assert!(matches!(
        runtime.apply_player_action(&header(owner, epoch, 1), invalid),
        Err(Phase6RuntimeError::Continuity(_))
    ));
    assert_eq!(runtime.state(owner), Some(initial.clone()));
    assert_eq!(runtime.projection_len(owner), Some(0));

    assert_eq!(
        runtime
            .apply_player_action(&header(owner, epoch, 1), mutation(&initial, 1))
            .unwrap(),
        ActionOutcome::Committed {
            projection_revision: 2,
            full_resync: false,
        }
    );
}

#[test]
fn continuity_restores_authority_but_replaces_transport_menu_and_projection_state() {
    let owner = player(1);
    let mut source = runtime(8);
    let epoch = join_and_drain(&mut source, owner);
    source.open_menu(&header(owner, epoch, 1), 4).unwrap();
    let initial = source.state(owner).unwrap();
    source
        .apply_menu_action(&header(owner, epoch, 1), 4, 0, mutation(&initial, 8))
        .unwrap();
    let committed = source.state(owner).unwrap();
    let records = source.capture_continuity().unwrap();

    let next_generation = ActivationGeneration::new(2).unwrap();
    let mut restored =
        Phase6RegionRuntime::restore(region(), next_generation, 8, 8, &records).unwrap();
    assert_eq!(restored.state(owner).unwrap().inventory.bytes(), &[8; 9]);
    assert_eq!(restored.state(owner).unwrap().inventory_revision, 1);
    assert_eq!(restored.menu(owner), None);
    let restored_epoch = restored.session_epoch(owner).unwrap();
    assert_eq!(restored_epoch, epoch + 1);
    let projection = restored.drain_projections(owner, 1).unwrap();
    assert!(matches!(
        projection.as_slice(),
        [event]
            if event.session_epoch == restored_epoch
                && matches!(
                    event.kind,
                    ProjectionKind::FullState {
                        reason: ResyncReason::Reload,
                        menu: None,
                        ..
                    }
                )
    ));

    let mut old_generation = header(owner, restored_epoch, 2);
    old_generation.generation = ActivationGeneration::INITIAL;
    assert!(matches!(
        restored.apply_player_action(&old_generation, mutation(&committed, 9)),
        Err(Phase6RuntimeError::StaleGeneration { .. })
    ));
    let replay = PlayerActionHeader {
        generation: next_generation,
        session_epoch: restored_epoch,
        ..header(owner, restored_epoch, 1)
    };
    assert_eq!(
        restored
            .apply_player_action(&replay, mutation(&committed, 9))
            .unwrap(),
        ActionOutcome::AlreadyApplied
    );
}

#[test]
fn continuity_records_are_stably_ordered_and_validate_player_fields() {
    let mut runtime = runtime(4);
    let second = player(2);
    let first = player(1);
    join_and_drain(&mut runtime, second);
    join_and_drain(&mut runtime, first);
    let records = runtime.capture_continuity().unwrap();
    assert_eq!(records.len(), 2);
    assert!(records[0].key() < records[1].key());

    let invalid = PlayerPersistentState {
        selected_slot: 9,
        ..PlayerPersistentState::default()
    };
    assert!(matches!(
        runtime.join(player(3), invalid),
        Err(Phase6RuntimeError::Continuity(_))
    ));
}
