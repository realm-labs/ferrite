//! Operator actions, structure operations, and GameTest convergence.

use crate::block::test_instance::BLOCK_UPDATE_FLAGS;
use crate::block::test_instance::data::{
    BlockEntityEffect, ErrorMarker, IntVector, QuarterRotation, TestAction, TestComponent,
    TestInstanceData, TestInstanceEntity, TestStatus,
};
use crate::block::test_instance::geometry::{
    BlockBox, BoundaryShell, PlacementPlan, boundary_shell, effective_rotation, plan_placement,
    structure_box, structure_position,
};
use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfiguredTest {
    pub key: ResourceId,
    pub structure: ResourceId,
    pub description: TestComponent,
    pub intrinsic_rotation: QuarterRotation,
    pub padding: i32,
    pub sky_access: bool,
    pub required: bool,
    pub setup_delay: u32,
    pub template_resolved: bool,
    pub template_size: Option<IntVector>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusResponse {
    pub description: TestComponent,
    pub size: Option<IntVector>,
    pub no_test_error: bool,
    pub positionless: bool,
}

pub fn status_response(action: TestAction, configured: Option<&ConfiguredTest>) -> StatusResponse {
    match configured {
        Some(test) => StatusResponse {
            description: test.description.clone(),
            size: (action == TestAction::Query && test.template_resolved)
                .then_some(test.template_size)
                .flatten(),
            no_test_error: false,
            positionless: true,
        },
        None => StatusResponse {
            description: TestComponent::literal("No test instance configured"),
            size: None,
            no_test_error: true,
            positionless: true,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportOutcome {
    Disabled,
    MissingCachedTemplate,
    FileSaveFailure(String),
    Success { absolute_path: String },
    PathValidationFailure(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionContext {
    pub has_game_master_permission: bool,
    pub matching_block_entity: bool,
    pub configured: Option<ConfiguredTest>,
    pub export_outcome: ExportOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestInstanceEffect {
    MoveToLevelThread,
    CheckGameMasterPermission,
    LookupMatchingBlockEntity,
    BlockEntity(BlockEntityEffect),
    SendRequesterStatus(StatusResponse),
    RemoveBarrierBlocks(BoundaryShell),
    Placement(PlacementPlan),
    CaptureTemplate {
        identifier: ResourceId,
        position: BlockPos,
        size: IntVector,
        include_entities: bool,
        empty_author: bool,
        save_to_disk: bool,
        ignore_air_and_structure_void: bool,
        result_ignored: bool,
    },
    RequestExportPath(ResourceId),
    ExportMethodReturned {
        failure: bool,
        ignored_by_handler: bool,
    },
    SendRequesterMessage {
        message: TestComponent,
        red: bool,
    },
    PropagatePathValidationFailure(String),
    ClearGlobalGameTestTicker,
    ForgetTrackedFailedTests,
    AnnounceRegisteredTestToRequester(ResourceId),
    CreateNoRetryGameTest {
        extra_rotation: QuarterRotation,
        position: BlockPos,
    },
    ReplaceWithFreshTestInstance {
        position: BlockPos,
        data: TestInstanceData,
    },
    PlaceBarrierBoundaryExceptTestInstances(BoundaryShell),
    StartTestAfterDelay(u32),
    DiscardPassingCleanupEntities(BlockBox),
    BroadcastResult {
        passed: bool,
        message: TestComponent,
    },
    FinalAirToCurrentStateUpdate {
        flags: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlan {
    pub accepted: bool,
    pub entity_replaced: bool,
    pub effects: Vec<TestInstanceEffect>,
}

pub fn handle_action(
    entity: &mut TestInstanceEntity,
    position: BlockPos,
    action: TestAction,
    packet_data: TestInstanceData,
    context: &ActionContext,
) -> ActionPlan {
    let mut effects = vec![TestInstanceEffect::MoveToLevelThread];
    effects.push(TestInstanceEffect::CheckGameMasterPermission);
    if !context.has_game_master_permission {
        return ActionPlan {
            accepted: false,
            entity_replaced: false,
            effects,
        };
    }
    effects.push(TestInstanceEffect::LookupMatchingBlockEntity);
    if !context.matching_block_entity {
        return ActionPlan {
            accepted: false,
            entity_replaced: false,
            effects,
        };
    }
    if matches!(action, TestAction::Init | TestAction::Query) {
        effects.push(TestInstanceEffect::SendRequesterStatus(status_response(
            action,
            context.configured.as_ref(),
        )));
        return ActionPlan {
            accepted: true,
            entity_replaced: false,
            effects,
        };
    }

    effects.extend(
        entity
            .set(packet_data, true)
            .into_iter()
            .map(TestInstanceEffect::BlockEntity),
    );
    let mut entity_replaced = false;
    let mut path_validation_aborted = false;
    match action {
        TestAction::Set => {}
        TestAction::Reset => {
            effects.extend(reset(entity, position, context.configured.as_ref()));
        }
        TestAction::Save => {
            effects.extend(save(position, &entity.data, context.configured.as_ref()).1);
        }
        TestAction::Export => {
            let (identifier, save_effects) =
                save(position, &entity.data, context.configured.as_ref());
            effects.extend(save_effects);
            if let Some(identifier) = identifier {
                let export = export(identifier, &context.export_outcome);
                path_validation_aborted = export.path_validation_aborted;
                effects.extend(export.effects);
            }
        }
        TestAction::Run => {
            let run = run(entity, position, context.configured.as_ref());
            entity_replaced = run.entity_replaced;
            effects.extend(run.effects);
        }
        TestAction::Init | TestAction::Query => unreachable!("queries return before mutation"),
    }
    if !path_validation_aborted {
        effects.push(TestInstanceEffect::FinalAirToCurrentStateUpdate {
            flags: BLOCK_UPDATE_FLAGS,
        });
    }
    ActionPlan {
        accepted: true,
        entity_replaced,
        effects,
    }
}

fn geometry_settings(configured: Option<&ConfiguredTest>) -> (QuarterRotation, i32, bool) {
    configured.map_or((QuarterRotation::None, 0, false), |test| {
        (test.intrinsic_rotation, test.padding, test.sky_access)
    })
}

fn reset(
    entity: &mut TestInstanceEntity,
    position: BlockPos,
    configured: Option<&ConfiguredTest>,
) -> Vec<TestInstanceEffect> {
    let (intrinsic, padding, sky_access) = geometry_settings(configured);
    let rotation = effective_rotation(intrinsic, entity.data.extra_rotation);
    let structure = structure_box(position, entity.data.size, rotation, padding);
    let mut effects = vec![TestInstanceEffect::RemoveBarrierBlocks(boundary_shell(
        structure, sky_access,
    ))];
    effects.extend(
        entity
            .clear_error_markers(true)
            .into_iter()
            .map(TestInstanceEffect::BlockEntity),
    );
    if let Some(test) = configured.filter(|test| test.template_resolved) {
        effects.push(TestInstanceEffect::Placement(plan_placement(
            position,
            &entity.data,
            test.structure.clone(),
            test.intrinsic_rotation,
            test.padding,
        )));
        effects.push(TestInstanceEffect::SendRequesterMessage {
            message: TestComponent::literal("Test instance reset"),
            red: false,
        });
    }
    effects.extend(
        entity
            .set_status(TestStatus::Cleared, true)
            .into_iter()
            .map(TestInstanceEffect::BlockEntity),
    );
    effects
}

fn save(
    position: BlockPos,
    data: &TestInstanceData,
    configured: Option<&ConfiguredTest>,
) -> (Option<ResourceId>, Vec<TestInstanceEffect>) {
    let identifier = configured
        .map(|test| test.structure.clone())
        .or_else(|| data.test_key.clone());
    let Some(identifier) = identifier else {
        return (
            None,
            vec![TestInstanceEffect::SendRequesterMessage {
                message: TestComponent::literal(format!(
                    "Unable to save test instance at {},{},{}",
                    position.x, position.y, position.z
                )),
                red: true,
            }],
        );
    };
    let padding = configured.map_or(0, |test| test.padding);
    (
        Some(identifier.clone()),
        vec![TestInstanceEffect::CaptureTemplate {
            identifier,
            position: structure_position(position, padding),
            size: data.size,
            include_entities: !data.ignore_entities,
            empty_author: true,
            save_to_disk: true,
            ignore_air_and_structure_void: true,
            result_ignored: true,
        }],
    )
}

struct ExportPlan {
    path_validation_aborted: bool,
    effects: Vec<TestInstanceEffect>,
}

fn export(identifier: ResourceId, outcome: &ExportOutcome) -> ExportPlan {
    let mut effects = vec![TestInstanceEffect::RequestExportPath(identifier)];
    let (path_validation_aborted, returned_failure, effect) = match outcome {
        ExportOutcome::Disabled => (
            false,
            Some(true),
            TestInstanceEffect::SendRequesterMessage {
                message: TestComponent::literal("Test instance export is disabled"),
                red: true,
            },
        ),
        ExportOutcome::MissingCachedTemplate => (
            false,
            Some(true),
            TestInstanceEffect::SendRequesterMessage {
                message: TestComponent::literal("Test instance template is not cached"),
                red: true,
            },
        ),
        ExportOutcome::FileSaveFailure(message) => (
            false,
            Some(true),
            TestInstanceEffect::SendRequesterMessage {
                message: TestComponent::literal(message.clone()),
                red: true,
            },
        ),
        ExportOutcome::Success { absolute_path } => (
            false,
            Some(false),
            TestInstanceEffect::SendRequesterMessage {
                message: TestComponent::literal(absolute_path.clone()),
                red: false,
            },
        ),
        ExportOutcome::PathValidationFailure(message) => (
            true,
            None,
            TestInstanceEffect::PropagatePathValidationFailure(message.clone()),
        ),
    };
    effects.push(effect);
    if let Some(failure) = returned_failure {
        effects.push(TestInstanceEffect::ExportMethodReturned {
            failure,
            ignored_by_handler: true,
        });
    }
    ExportPlan {
        path_validation_aborted,
        effects,
    }
}

struct RunPlan {
    entity_replaced: bool,
    effects: Vec<TestInstanceEffect>,
}

fn run(
    entity: &mut TestInstanceEntity,
    position: BlockPos,
    configured: Option<&ConfiguredTest>,
) -> RunPlan {
    let Some(test) = configured else {
        return RunPlan {
            entity_replaced: false,
            effects: vec![TestInstanceEffect::SendRequesterMessage {
                message: TestComponent::literal("No test instance configured"),
                red: true,
            }],
        };
    };
    if !test.template_resolved {
        return RunPlan {
            entity_replaced: false,
            effects: vec![TestInstanceEffect::SendRequesterMessage {
                message: TestComponent::literal("No test structure available"),
                red: true,
            }],
        };
    }

    let mut effects = vec![TestInstanceEffect::Placement(plan_placement(
        position,
        &entity.data,
        test.structure.clone(),
        test.intrinsic_rotation,
        test.padding,
    ))];
    effects.extend(
        entity
            .clear_error_markers(true)
            .into_iter()
            .map(TestInstanceEffect::BlockEntity),
    );
    effects.extend([
        TestInstanceEffect::ClearGlobalGameTestTicker,
        TestInstanceEffect::ForgetTrackedFailedTests,
        TestInstanceEffect::AnnounceRegisteredTestToRequester(test.key.clone()),
        TestInstanceEffect::CreateNoRetryGameTest {
            extra_rotation: entity.data.extra_rotation,
            position,
        },
    ]);
    let fresh_data = TestInstanceData {
        test_key: Some(test.key.clone()),
        size: test.template_size.unwrap_or(IntVector::new(1, 1, 1)),
        extra_rotation: entity.data.extra_rotation,
        ignore_entities: false,
        status: TestStatus::Cleared,
        error: None,
    };
    effects.push(TestInstanceEffect::ReplaceWithFreshTestInstance {
        position,
        data: fresh_data.clone(),
    });
    let second = plan_placement(
        position,
        &fresh_data,
        test.structure.clone(),
        test.intrinsic_rotation,
        test.padding,
    );
    let shell = boundary_shell(second.structure_box, test.sky_access);
    effects.extend([
        TestInstanceEffect::Placement(second),
        TestInstanceEffect::PlaceBarrierBoundaryExceptTestInstances(shell),
        TestInstanceEffect::StartTestAfterDelay(test.setup_delay),
    ]);
    RunPlan {
        entity_replaced: true,
        effects,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunResult {
    Passed {
        message: TestComponent,
    },
    Failed {
        message: TestComponent,
        positional_error: Option<ErrorMarker>,
    },
}

pub fn start_execution(entity: &mut TestInstanceEntity) -> Vec<TestInstanceEffect> {
    entity
        .set_status(TestStatus::Running, true)
        .into_iter()
        .map(TestInstanceEffect::BlockEntity)
        .collect()
}

pub fn report_result(
    entity: &mut TestInstanceEntity,
    cleanup_box: BlockBox,
    result: RunResult,
) -> Vec<TestInstanceEffect> {
    let mut effects = Vec::new();
    match result {
        RunResult::Passed { message } => {
            effects.extend(
                entity
                    .set_status(TestStatus::Finished, true)
                    .into_iter()
                    .map(TestInstanceEffect::BlockEntity),
            );
            effects.push(TestInstanceEffect::DiscardPassingCleanupEntities(
                cleanup_box.inflate(1),
            ));
            effects.push(TestInstanceEffect::BroadcastResult {
                passed: true,
                message,
            });
        }
        RunResult::Failed {
            message,
            positional_error,
        } => {
            effects.extend(
                entity
                    .set_error(message.clone(), true)
                    .into_iter()
                    .map(TestInstanceEffect::BlockEntity),
            );
            if let Some(marker) = positional_error {
                effects.extend(
                    entity
                        .add_error_marker(marker, true)
                        .into_iter()
                        .map(TestInstanceEffect::BlockEntity),
                );
            }
            effects.push(TestInstanceEffect::BroadcastResult {
                passed: false,
                message,
            });
        }
    }
    effects
}
