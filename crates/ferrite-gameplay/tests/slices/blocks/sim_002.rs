use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::block::test_instance::client::{
    BEAM_FINAL_HEIGHT, BeamColor, EditorButton, MARKER_ALPHA, MARKER_INFLATION, TestInstanceEditor,
    UseWithoutItem, beam_projection, bounds_projection, combined_render_admission,
    marker_projections, truncate_utf16, use_without_item,
};
use ferrite_gameplay::block::test_instance::data::{
    BlockEntityEffect, ErrorMarker, IntVector, QuarterRotation, SET_CHANGED_EFFECTS, TestAction,
    TestComponent, TestInstanceData, TestInstanceEntity, TestStatus,
};
use ferrite_gameplay::block::test_instance::geometry::{
    BlockBox, ChunkRectangle, PlacementEffect, boundary_shell, effective_rotation,
    placement_start_corner, plan_placement, structure_box, structure_position, test_box,
    transformed_size,
};
use ferrite_gameplay::block::test_instance::operations::{
    ActionContext, ConfiguredTest, ExportOutcome, RunResult, TestInstanceEffect, handle_action,
    report_result, start_execution,
};
use ferrite_gameplay::block::test_instance::{
    BLOCK_ENTITY_PROTOCOL_ID, BLOCK_PROPERTIES, BLOCK_STATE_ID, BLOCK_UPDATE_FLAGS, POI_TICKETS,
    POI_VALID_RANGE, TEMPLATE_WRITE_FLAGS,
};

const POSITION: BlockPos = BlockPos::new(10, 20, 30);

fn minecraft(path: &str) -> ResourceId {
    ResourceId::minecraft(path).unwrap()
}

fn component(text: &str) -> TestComponent {
    TestComponent::literal(text)
}

fn data() -> TestInstanceData {
    TestInstanceData {
        test_key: Some(minecraft("always_pass")),
        size: IntVector::new(2, 3, 4),
        extra_rotation: QuarterRotation::Clockwise90,
        ignore_entities: true,
        status: TestStatus::Running,
        error: Some(component("old error")),
    }
}

fn configured(template_resolved: bool) -> ConfiguredTest {
    ConfiguredTest {
        key: minecraft("always_pass"),
        structure: minecraft("gametest/always_pass"),
        description: component("Always passes"),
        intrinsic_rotation: QuarterRotation::Clockwise90,
        padding: 1,
        sky_access: false,
        required: true,
        setup_delay: 7,
        template_resolved,
        template_size: Some(IntVector::new(5, 6, 7)),
    }
}

fn context(configured: Option<ConfiguredTest>) -> ActionContext {
    ActionContext {
        has_game_master_permission: true,
        matching_block_entity: true,
        configured,
        export_outcome: ExportOutcome::Success {
            absolute_path: "C:\\exports\\always_pass.snbt".to_owned(),
        },
    }
}

#[test]
fn identity_properties_status_and_action_wire_fallbacks_are_locked() {
    assert_eq!(BLOCK_STATE_ID, 21_742);
    assert_eq!(BLOCK_ENTITY_PROTOCOL_ID, 46);
    assert_eq!(BLOCK_UPDATE_FLAGS, 3);
    assert_eq!(TEMPLATE_WRITE_FLAGS, 818);
    assert_eq!(POI_TICKETS, 0);
    assert_eq!(POI_VALID_RANGE, 1);
    let properties = std::hint::black_box(BLOCK_PROPERTIES);
    assert_eq!(properties.destroy_time, -1.0);
    assert_eq!(properties.explosion_resistance, 3_600_000.0);
    assert!(!properties.has_loot_table);
    assert!(!properties.occluding);
    assert!(!properties.view_blocking);
    assert!(properties.full_collision_cube);
    assert_eq!(properties.item_stack_limit, 64);
    assert!(properties.item_epic);
    assert!(!properties.item_has_special_data_components);
    assert!(properties.block_and_item_share_cube_all_texture);
    assert!(properties.dragon_immune);
    assert!(properties.wither_immune);

    assert_eq!(TestStatus::Cleared.wire_id(), 0);
    assert_eq!(TestStatus::Running.wire_id(), 1);
    assert_eq!(TestStatus::Finished.wire_id(), 2);
    assert_eq!(TestStatus::from_wire_id(-1), TestStatus::Cleared);
    assert_eq!(TestStatus::from_wire_id(3), TestStatus::Cleared);
    for action in [
        TestAction::Init,
        TestAction::Query,
        TestAction::Set,
        TestAction::Reset,
        TestAction::Save,
        TestAction::Export,
        TestAction::Run,
    ] {
        assert_eq!(TestAction::from_wire_id(action.wire_id()), action);
    }
    assert_eq!(TestAction::from_wire_id(-1), TestAction::Init);
    assert_eq!(TestAction::from_wire_id(i32::MAX), TestAction::Init);
}

#[test]
fn six_field_data_marker_and_persistence_transitions_preserve_update_quirks() {
    let original = data();
    let finished = original.with_status(TestStatus::Finished);
    assert_eq!(finished.test_key, original.test_key);
    assert_eq!(finished.size, original.size);
    assert_eq!(finished.status, TestStatus::Finished);
    assert_eq!(finished.error, None);
    let failed = original.with_error("failure");
    assert_eq!(failed.status, TestStatus::Finished);
    assert_eq!(failed.error, Some(component("failure")));

    let marker = ErrorMarker {
        position: POSITION,
        message: component("marker"),
    };
    let mut entity = TestInstanceEntity::default();
    assert_eq!(entity.set(original.clone(), false), []);
    assert_eq!(entity.set(original.clone(), true), SET_CHANGED_EFFECTS);
    assert_eq!(
        entity.add_error_marker(marker.clone(), true),
        SET_CHANGED_EFFECTS
    );
    assert_eq!(entity.clear_error_markers(true), SET_CHANGED_EFFECTS);
    assert_eq!(entity.clear_error_markers(true), []);

    entity.add_error_marker(marker.clone(), false);
    let saved = entity.save();
    assert_eq!(saved.data, original);
    assert_eq!(saved.errors, Some(vec![marker.clone()]));
    assert_eq!(entity.update_payload(), saved);
    entity.clear_error_markers(false);
    assert_eq!(entity.save().errors, None);

    let retained = entity.data.clone();
    entity.error_markers.push(marker.clone());
    assert_eq!(entity.load(None, None, true), []);
    assert_eq!(entity.data, retained);
    assert!(entity.error_markers.is_empty());
    assert_eq!(
        entity.load(
            Some(TestInstanceData::default()),
            Some(vec![marker.clone()]),
            true,
        ),
        SET_CHANGED_EFFECTS
    );
    assert_eq!(entity.data, TestInstanceData::default());
    assert_eq!(entity.error_markers, [marker]);

    let opaque = TestComponent::opaque_adapter_payload(vec![10, 0, 8, 0, 0]);
    entity.set_error(opaque.clone(), false);
    entity.add_error_marker(
        ErrorMarker {
            position: POSITION,
            message: opaque.clone(),
        },
        false,
    );
    let opaque_saved = entity.save();
    assert_eq!(opaque_saved.data.error, Some(opaque.clone()));
    assert_eq!(
        opaque_saved
            .errors
            .unwrap()
            .last()
            .map(|marker| &marker.message),
        Some(&opaque)
    );
}

#[test]
fn signed_geometry_uses_effective_rotation_raw_size_and_normalized_boxes() {
    assert_eq!(
        effective_rotation(QuarterRotation::Clockwise90, QuarterRotation::Clockwise90),
        QuarterRotation::Clockwise180
    );
    assert_eq!(
        effective_rotation(
            QuarterRotation::Clockwise180,
            QuarterRotation::Counterclockwise90
        ),
        QuarterRotation::Clockwise90
    );
    assert_eq!(
        transformed_size(IntVector::new(2, 3, 4), QuarterRotation::Clockwise90),
        IntVector::new(4, 3, 2)
    );
    assert_eq!(structure_position(POSITION, 1), BlockPos::new(11, 22, 32));
    assert_eq!(
        structure_box(
            POSITION,
            IntVector::new(2, 3, 4),
            QuarterRotation::Clockwise90,
            1,
        ),
        BlockBox {
            minimum: BlockPos::new(11, 22, 32),
            maximum: BlockPos::new(14, 24, 33),
        }
    );
    assert_eq!(
        test_box(
            POSITION,
            IntVector::new(2, 3, 4),
            QuarterRotation::Clockwise90,
            1,
        ),
        BlockBox {
            minimum: BlockPos::new(10, 21, 31),
            maximum: BlockPos::new(15, 25, 34),
        }
    );
    let start = structure_position(POSITION, 1);
    assert_eq!(
        placement_start_corner(start, IntVector::new(2, 3, 4), QuarterRotation::Clockwise90,),
        BlockPos::new(14, 22, 32)
    );
    assert_eq!(
        placement_start_corner(
            start,
            IntVector::new(2, 3, 4),
            QuarterRotation::Clockwise180,
        ),
        BlockPos::new(12, 22, 35)
    );
    let forged = structure_box(
        POSITION,
        IntVector::new(0, -2, i32::MAX),
        QuarterRotation::None,
        0,
    );
    assert_eq!(forged.minimum.x, 9);
    assert_eq!(forged.maximum.x, 10);
    assert_eq!(forged.minimum.y, 18);
    assert_eq!(forged.maximum.y, 21);
    assert_eq!(
        forged.minimum.z,
        31_i32.wrapping_add(i32::MAX.wrapping_sub(1))
    );
    assert_eq!(forged.maximum.z, 31);
}

#[test]
fn placement_plan_force_loads_clears_twice_discards_and_places_in_order() {
    let plan = plan_placement(
        POSITION,
        &data(),
        minecraft("gametest/always_pass"),
        QuarterRotation::Clockwise90,
        1,
    );
    assert_eq!(plan.effects.len(), 7);
    assert!(matches!(
        plan.effects[0],
        PlacementEffect::PermanentlyForceChunks(ChunkRectangle { .. })
    ));
    assert!(matches!(
        plan.effects[1],
        PlacementEffect::ClearBlocksToAir {
            flags: 818,
            explicit_neighbor_update_per_cell: true,
            ..
        }
    ));
    assert_eq!(
        plan.effects[2],
        PlacementEffect::ClearScheduledBlockTicks(plan.test_box)
    );
    assert_eq!(
        plan.effects[3],
        PlacementEffect::ClearBlockEvents(plan.test_box)
    );
    assert_eq!(
        plan.effects[4],
        PlacementEffect::DiscardNonPlayerEntities(plan.test_box)
    );
    assert_eq!(
        plan.effects[5],
        PlacementEffect::RepeatDiscardNonPlayerEntities(plan.test_box)
    );
    let PlacementEffect::PlaceTemplate {
        origin,
        pivot,
        rotation,
        ignore_entities,
        known_shape,
        use_level_rng,
        flags,
        ..
    } = &plan.effects[6]
    else {
        panic!("last effect must place the template");
    };
    assert_eq!(origin, pivot);
    assert_eq!(*rotation, QuarterRotation::Clockwise180);
    assert!(*ignore_entities);
    assert!(*known_shape);
    assert!(*use_level_rng);
    assert_eq!(*flags, 818);
}

#[test]
fn action_admission_queries_and_set_lock_thread_permission_and_duplicate_publication() {
    let packet = data();
    let mut entity = TestInstanceEntity::default();
    let denied = handle_action(
        &mut entity,
        POSITION,
        TestAction::Set,
        packet.clone(),
        &ActionContext {
            has_game_master_permission: false,
            matching_block_entity: true,
            ..context(Some(configured(true)))
        },
    );
    assert!(!denied.accepted);
    assert_eq!(
        denied.effects,
        [
            TestInstanceEffect::MoveToLevelThread,
            TestInstanceEffect::CheckGameMasterPermission,
        ]
    );
    assert_eq!(entity.data, TestInstanceData::default());

    let missing = handle_action(
        &mut entity,
        POSITION,
        TestAction::Set,
        packet.clone(),
        &ActionContext {
            matching_block_entity: false,
            ..context(Some(configured(true)))
        },
    );
    assert!(!missing.accepted);
    assert_eq!(
        missing.effects[2],
        TestInstanceEffect::LookupMatchingBlockEntity
    );

    let query = handle_action(
        &mut entity,
        POSITION,
        TestAction::Query,
        packet.clone(),
        &context(Some(configured(true))),
    );
    assert!(query.accepted);
    assert_eq!(entity.data, TestInstanceData::default());
    let TestInstanceEffect::SendRequesterStatus(response) = &query.effects[3] else {
        panic!("query must respond only to the requester");
    };
    assert_eq!(response.description, component("Always passes"));
    assert_eq!(response.size, Some(IntVector::new(5, 6, 7)));
    assert!(response.positionless);

    let init = handle_action(
        &mut entity,
        POSITION,
        TestAction::Init,
        packet.clone(),
        &context(Some(configured(true))),
    );
    let TestInstanceEffect::SendRequesterStatus(response) = &init.effects[3] else {
        panic!("init must respond");
    };
    assert_eq!(response.size, None);
    let missing_template_query = handle_action(
        &mut entity,
        POSITION,
        TestAction::Query,
        packet.clone(),
        &context(Some(configured(false))),
    );
    let TestInstanceEffect::SendRequesterStatus(response) = &missing_template_query.effects[3]
    else {
        panic!("missing template query must still describe the registered test");
    };
    assert_eq!(response.description, component("Always passes"));
    assert_eq!(response.size, None);
    let no_test_query = handle_action(
        &mut entity,
        POSITION,
        TestAction::Query,
        packet.clone(),
        &context(None),
    );
    let TestInstanceEffect::SendRequesterStatus(response) = &no_test_query.effects[3] else {
        panic!("missing registry entry must return a red no-test status");
    };
    assert!(response.no_test_error);
    assert_eq!(response.size, None);

    let retained_marker = ErrorMarker {
        position: POSITION,
        message: component("retained by SET"),
    };
    entity.error_markers.push(retained_marker.clone());
    let set = handle_action(
        &mut entity,
        POSITION,
        TestAction::Set,
        packet.clone(),
        &context(Some(configured(true))),
    );
    assert!(set.accepted);
    assert_eq!(entity.data, packet);
    assert_eq!(entity.error_markers, [retained_marker]);
    assert_eq!(
        set.effects,
        [
            TestInstanceEffect::MoveToLevelThread,
            TestInstanceEffect::CheckGameMasterPermission,
            TestInstanceEffect::LookupMatchingBlockEntity,
            TestInstanceEffect::BlockEntity(BlockEntityEffect::MarkChunkDirty),
            TestInstanceEffect::BlockEntity(BlockEntityEffect::PublishAirToCurrentState {
                flags: 3
            }),
            TestInstanceEffect::FinalAirToCurrentStateUpdate { flags: 3 },
        ]
    );
}

#[test]
fn reset_save_and_export_preserve_silent_ignored_and_inverted_outcomes() {
    let marker = ErrorMarker {
        position: POSITION,
        message: component("old marker"),
    };
    let mut entity = TestInstanceEntity {
        data: TestInstanceData::default(),
        error_markers: vec![marker],
    };
    let reset = handle_action(
        &mut entity,
        POSITION,
        TestAction::Reset,
        data(),
        &context(Some(configured(true))),
    );
    assert!(entity.error_markers.is_empty());
    assert_eq!(entity.data.status, TestStatus::Cleared);
    assert!(
        reset
            .effects
            .iter()
            .any(|effect| matches!(effect, TestInstanceEffect::RemoveBarrierBlocks(_)))
    );
    assert!(
        reset
            .effects
            .iter()
            .any(|effect| matches!(effect, TestInstanceEffect::Placement(_)))
    );

    let mut missing_template_entity = TestInstanceEntity::default();
    let missing_template = handle_action(
        &mut missing_template_entity,
        POSITION,
        TestAction::Reset,
        data(),
        &context(Some(configured(false))),
    );
    assert!(
        !missing_template
            .effects
            .iter()
            .any(|effect| matches!(effect, TestInstanceEffect::Placement(_)))
    );
    assert!(!missing_template.effects.iter().any(|effect| matches!(
        effect,
        TestInstanceEffect::SendRequesterMessage { red: false, .. }
    )));

    let no_key = TestInstanceData {
        test_key: None,
        ..TestInstanceData::default()
    };
    let mut save_entity = TestInstanceEntity::default();
    let failed_save = handle_action(
        &mut save_entity,
        POSITION,
        TestAction::Save,
        no_key,
        &context(None),
    );
    assert!(failed_save.effects.iter().any(|effect| matches!(
        effect,
        TestInstanceEffect::SendRequesterMessage { red: true, .. }
    )));

    let mut export_entity = TestInstanceEntity::default();
    let export = handle_action(
        &mut export_entity,
        POSITION,
        TestAction::Export,
        data(),
        &context(Some(configured(true))),
    );
    let capture_index = export
        .effects
        .iter()
        .position(|effect| matches!(effect, TestInstanceEffect::CaptureTemplate { .. }))
        .unwrap();
    let path_index = export
        .effects
        .iter()
        .position(|effect| matches!(effect, TestInstanceEffect::RequestExportPath(_)))
        .unwrap();
    assert!(capture_index < path_index);
    let TestInstanceEffect::CaptureTemplate {
        identifier,
        position,
        size,
        include_entities,
        empty_author,
        save_to_disk,
        ignore_air_and_structure_void,
        result_ignored,
    } = &export.effects[capture_index]
    else {
        panic!("export must run SAVE first");
    };
    assert_eq!(identifier, &minecraft("gametest/always_pass"));
    assert_eq!(*position, structure_position(POSITION, 1));
    assert_eq!(*size, IntVector::new(2, 3, 4));
    assert!(!include_entities);
    assert!(*empty_author);
    assert!(*save_to_disk);
    assert!(*ignore_air_and_structure_void);
    assert!(*result_ignored);
    assert!(export.effects.iter().any(|effect| {
        matches!(
            effect,
            TestInstanceEffect::ExportMethodReturned {
                failure: false,
                ignored_by_handler: true,
            }
        )
    }));

    let mut path_abort_entity = TestInstanceEntity::default();
    let path_abort = handle_action(
        &mut path_abort_entity,
        POSITION,
        TestAction::Export,
        data(),
        &ActionContext {
            export_outcome: ExportOutcome::PathValidationFailure("invalid path".to_owned()),
            ..context(Some(configured(true)))
        },
    );
    assert!(matches!(
        path_abort.effects.last(),
        Some(TestInstanceEffect::PropagatePathValidationFailure(_))
    ));

    let mut disabled_entity = TestInstanceEntity::default();
    let disabled = handle_action(
        &mut disabled_entity,
        POSITION,
        TestAction::Export,
        data(),
        &ActionContext {
            export_outcome: ExportOutcome::Disabled,
            ..context(Some(configured(true)))
        },
    );
    assert!(disabled.effects.iter().any(|effect| matches!(
        effect,
        TestInstanceEffect::ExportMethodReturned {
            failure: true,
            ignored_by_handler: true,
        }
    )));
}

#[test]
fn successful_run_places_twice_replaces_entity_and_resets_global_state() {
    let marker = ErrorMarker {
        position: POSITION,
        message: component("old"),
    };
    let mut entity = TestInstanceEntity {
        data: TestInstanceData::default(),
        error_markers: vec![marker],
    };
    let run = handle_action(
        &mut entity,
        POSITION,
        TestAction::Run,
        data(),
        &context(Some(configured(true))),
    );
    assert!(run.entity_replaced);
    assert!(entity.error_markers.is_empty());
    assert_eq!(
        run.effects
            .iter()
            .filter(|effect| matches!(effect, TestInstanceEffect::Placement(_)))
            .count(),
        2
    );
    let replacement = run
        .effects
        .iter()
        .find_map(|effect| match effect {
            TestInstanceEffect::ReplaceWithFreshTestInstance { position, data }
                if *position == POSITION =>
            {
                Some(data)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(replacement.size, IntVector::new(5, 6, 7));
    assert_eq!(replacement.extra_rotation, QuarterRotation::Clockwise90);
    assert!(!replacement.ignore_entities);
    assert_eq!(replacement.status, TestStatus::Cleared);
    assert_eq!(replacement.error, None);
    let clear_global = run
        .effects
        .iter()
        .position(|effect| matches!(effect, TestInstanceEffect::ClearGlobalGameTestTicker))
        .unwrap();
    let replacement_index = run
        .effects
        .iter()
        .position(|effect| {
            matches!(
                effect,
                TestInstanceEffect::ReplaceWithFreshTestInstance { .. }
            )
        })
        .unwrap();
    assert!(clear_global < replacement_index);
    assert!(run.effects.iter().any(|effect| matches!(
        effect,
        TestInstanceEffect::PlaceBarrierBoundaryExceptTestInstances(_)
    )));
    assert!(
        run.effects
            .contains(&TestInstanceEffect::StartTestAfterDelay(7))
    );

    let mut fallback_config = configured(true);
    fallback_config.template_size = None;
    let fallback = handle_action(
        &mut TestInstanceEntity::default(),
        POSITION,
        TestAction::Run,
        data(),
        &context(Some(fallback_config)),
    );
    assert!(fallback.effects.iter().any(|effect| matches!(
        effect,
        TestInstanceEffect::ReplaceWithFreshTestInstance {
            data: TestInstanceData {
                size: IntVector { x: 1, y: 1, z: 1 },
                ..
            },
            ..
        }
    )));
}

#[test]
fn failed_run_retains_packet_record_and_markers_while_results_converge() {
    let marker = ErrorMarker {
        position: POSITION,
        message: component("retained"),
    };
    let packet = data();
    let mut entity = TestInstanceEntity {
        data: TestInstanceData::default(),
        error_markers: vec![marker.clone()],
    };
    let failed_run = handle_action(
        &mut entity,
        POSITION,
        TestAction::Run,
        packet.clone(),
        &context(None),
    );
    assert!(!failed_run.entity_replaced);
    assert_eq!(entity.data, packet);
    assert_eq!(
        entity.error_markers.as_slice(),
        std::slice::from_ref(&marker)
    );

    assert_eq!(start_execution(&mut entity).len(), 2);
    assert_eq!(entity.data.status, TestStatus::Running);
    assert_eq!(entity.data.error, None);
    let cleanup = BlockBox {
        minimum: POSITION,
        maximum: BlockPos::new(12, 22, 32),
    };
    let passed = report_result(
        &mut entity,
        cleanup,
        RunResult::Passed {
            message: component("passed"),
        },
    );
    assert_eq!(entity.data.status, TestStatus::Finished);
    assert_eq!(entity.data.error, None);
    assert!(
        passed
            .iter()
            .any(|effect| matches!(effect, TestInstanceEffect::DiscardPassingCleanupEntities(_)))
    );

    let positional = ErrorMarker {
        position: BlockPos::new(100, 64, -100),
        message: component("absolute"),
    };
    let failed = report_result(
        &mut entity,
        cleanup,
        RunResult::Failed {
            message: component("failure"),
            positional_error: Some(positional.clone()),
        },
    );
    assert_eq!(entity.data.status, TestStatus::Finished);
    assert_eq!(entity.data.error, Some(component("failure")));
    assert_eq!(entity.error_markers.last(), Some(&positional));
    assert_eq!(
        failed
            .iter()
            .filter(|effect| {
                matches!(
                    effect,
                    TestInstanceEffect::BlockEntity(BlockEntityEffect::PublishAirToCurrentState {
                        flags: 3
                    })
                )
            })
            .count(),
        2
    );
}

#[test]
fn editor_is_local_clamped_snapshot_only_and_response_race_compatible() {
    assert_eq!(use_without_item(true, false, true), UseWithoutItem::Pass);
    assert_eq!(
        use_without_item(true, true, false),
        UseWithoutItem::Success {
            open_local_editor: false
        }
    );
    assert_eq!(
        use_without_item(true, true, true),
        UseWithoutItem::Success {
            open_local_editor: true
        }
    );

    let (mut editor, init) = TestInstanceEditor::open(&data(), QuarterRotation::Clockwise90, false);
    assert_eq!(init.action, TestAction::Init);
    assert_eq!(editor.selected_rotation, QuarterRotation::Clockwise180);
    assert!(!editor.include_entities);
    assert!(!editor.export_visible);
    let invalid = editor.edit_identifier("Bad ID");
    assert!(invalid.local_invalid_identifier);
    assert_eq!(invalid.packet.action, TestAction::Query);
    assert_eq!(invalid.packet.data.test_key, None);
    assert_eq!(editor.description, component("Invalid test identifier"));
    let valid = editor.edit_identifier("always_pass");
    assert!(!valid.local_invalid_identifier);
    assert_eq!(valid.packet.data.test_key, Some(minecraft("always_pass")));
    editor.set_size_text(0, "999");
    editor.set_size_text(1, "-40");
    editor.set_size_text(2, "invalid");
    editor.set_size_text(2, "1234567890123456");
    assert_eq!(editor.size_z.encode_utf16().count(), 15);
    editor.set_size_text(2, "invalid");
    let outgoing = editor.packet(TestAction::Run);
    assert_eq!(outgoing.data.size, IntVector::new(48, 1, 1));
    assert_eq!(outgoing.data.status, TestStatus::Cleared);
    assert_eq!(outgoing.data.error, None);
    assert!(!editor.save_or_export_active());
    editor.selected_rotation = QuarterRotation::None;
    assert!(editor.save_or_export_active());
    assert_eq!(
        editor.submit(EditorButton::Cancel),
        ferrite_gameplay::block::test_instance::client::EditorSubmission {
            close_screen: true,
            packet: None,
        }
    );
    assert_eq!(
        editor.submit(EditorButton::Done).packet.unwrap().action,
        TestAction::Set
    );
    editor.receive_status(
        &ferrite_gameplay::block::test_instance::operations::StatusResponse {
            description: component("older response"),
            size: Some(IntVector::new(9, 8, 7)),
            no_test_error: false,
            positionless: true,
        },
        Some(&component("current synchronized error")),
    );
    assert_eq!(
        editor.description,
        TestComponent::sequence([
            component("current synchronized error"),
            component(": "),
            component("older response"),
        ])
    );
    assert_eq!(
        (&editor.size_x, &editor.size_y, &editor.size_z),
        (&"9".to_owned(), &"8".to_owned(), &"7".to_owned())
    );
    assert_eq!(truncate_utf16("a😀b", 3), "a😀");
}

#[test]
fn beams_bounds_markers_and_combined_admission_follow_independent_gates() {
    let mut projected = data();
    projected.status = TestStatus::Cleared;
    assert_eq!(beam_projection(&projected, None), None);
    projected.status = TestStatus::Running;
    projected.error = None;
    let running = beam_projection(&projected, None).unwrap();
    assert_eq!(running.color, BeamColor::Gray);
    assert_eq!(running.final_height, BEAM_FINAL_HEIGHT);
    assert!(!running.permission_gated);
    assert!(running.ordinary_beacon_animation);
    assert!(running.distance_scaling);
    assert!(running.horizontal_distance_admission);
    projected.status = TestStatus::Finished;
    assert_eq!(
        beam_projection(&projected, None).unwrap().color,
        BeamColor::Green
    );
    projected.error = Some(component("failure"));
    let mut optional = configured(true);
    optional.required = false;
    assert_eq!(
        beam_projection(&projected, Some(&optional)).unwrap().color,
        BeamColor::Orange
    );
    assert_eq!(
        beam_projection(&projected, None).unwrap().color,
        BeamColor::Red
    );

    assert_eq!(
        bounds_projection(&projected, Some(&optional), false, false),
        None
    );
    let bounds = bounds_projection(&projected, Some(&optional), false, true).unwrap();
    assert!(bounds.always_box);
    assert!(bounds.opaque_light_gray);
    assert!(!bounds.invisible_cells);
    projected.size.y = 0;
    assert_eq!(
        bounds_projection(&projected, Some(&optional), true, false),
        None
    );

    let markers = marker_projections(&[ErrorMarker {
        position: POSITION,
        message: component("visible"),
    }]);
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].cube_inflation, MARKER_INFLATION);
    assert_eq!(markers[0].cube_alpha, MARKER_ALPHA);
    assert!(markers[0].red_filled_cube);
    assert!(markers[0].always_on_top);
    assert!(!markers[0].permission_gated);
    assert_eq!(
        combined_render_admission(false, false),
        ferrite_gameplay::block::test_instance::client::CombinedRenderAdmission {
            offscreen_eligible: true,
            admitted: false,
        }
    );
    assert!(combined_render_admission(true, false).admitted);
    assert!(combined_render_admission(false, true).admitted);
    assert!(
        !boundary_shell(
            BlockBox {
                minimum: POSITION,
                maximum: POSITION,
            },
            true,
        )
        .include_ceiling
    );
    assert_eq!(
        boundary_shell(
            BlockBox {
                minimum: POSITION,
                maximum: POSITION,
            },
            false,
        )
        .outside_distance,
        1
    );
}
