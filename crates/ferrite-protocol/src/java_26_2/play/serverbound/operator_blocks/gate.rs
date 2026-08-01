use crate::java_26_2::play::serverbound::operator_blocks::packet::{
    OperatorBlockPacketKind, OperatorBlockRequest, StructureUpdate, TestInstanceAction,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OperatorBlockGates {
    pub operator_blocks: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OperatorBlockContext {
    pub instabuild: bool,
    pub command_game_master: bool,
}

impl OperatorBlockContext {
    #[must_use]
    pub const fn can_use_game_master_blocks(self) -> bool {
        self.instabuild && self.command_game_master
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandToolMessage {
    None,
    Success,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureOperationEffect {
    UpdateDataNoOperation,
    RunAndReport {
        update: StructureUpdate,
        success: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorBlockEffect {
    GenerateJigsaw {
        levels: i32,
        keep_jigsaws: bool,
    },
    UpdateCommandBlock {
        clear_last_output: bool,
        call_update_hook: bool,
        message: CommandToolMessage,
    },
    UpdateCommandMinecart {
        clear_last_output: bool,
        call_metadata_hook: bool,
        message: CommandToolMessage,
    },
    SetJigsawFieldsThenMarkAndPublish,
    WriteStructureFieldsThenOperateMarkAndPublish {
        operation: StructureOperationEffect,
    },
    SetTestModeThenStateMessageMarkAndPublish,
    ReplyTestInstanceStatusDirect {
        include_structure_size: bool,
        missing_test: bool,
    },
    InstallTestDataThenOperateAndPublishAirToCurrent {
        action: TestInstanceAction,
        operation_succeeded: bool,
        publication_flags: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorBlockDecision {
    OmitDisabled,
    RefuseUnauthorizedWithCommandMessage(OperatorBlockPacketKind),
    OmitUnauthorized(OperatorBlockPacketKind),
    OmitMissingOrWrongTarget(OperatorBlockPacketKind),
    Emit(OperatorBlockEffect),
}

impl OperatorBlockGates {
    #[must_use]
    pub const fn decide(
        self,
        request: OperatorBlockRequest,
        context: OperatorBlockContext,
    ) -> OperatorBlockDecision {
        if !self.operator_blocks {
            return OperatorBlockDecision::OmitDisabled;
        }
        let kind = request.kind();
        if !context.can_use_game_master_blocks() {
            return if kind.is_command_tool() {
                OperatorBlockDecision::RefuseUnauthorizedWithCommandMessage(kind)
            } else {
                OperatorBlockDecision::OmitUnauthorized(kind)
            };
        }
        if !request.target_matches() {
            return OperatorBlockDecision::OmitMissingOrWrongTarget(kind);
        }
        OperatorBlockDecision::Emit(effect(request))
    }
}

const fn effect(request: OperatorBlockRequest) -> OperatorBlockEffect {
    match request {
        OperatorBlockRequest::JigsawGenerate {
            levels,
            keep_jigsaws,
            ..
        } => OperatorBlockEffect::GenerateJigsaw {
            levels,
            keep_jigsaws,
        },
        OperatorBlockRequest::SetCommandBlock {
            command_nonempty,
            track_output,
            command_blocks_enabled,
            ..
        } => OperatorBlockEffect::UpdateCommandBlock {
            clear_last_output: !track_output,
            call_update_hook: command_blocks_enabled,
            message: command_message(command_nonempty, command_blocks_enabled),
        },
        OperatorBlockRequest::SetCommandMinecart {
            command_nonempty,
            track_output,
            command_blocks_enabled,
            ..
        } => OperatorBlockEffect::UpdateCommandMinecart {
            clear_last_output: !track_output,
            call_metadata_hook: command_blocks_enabled,
            message: command_message(command_nonempty, command_blocks_enabled),
        },
        OperatorBlockRequest::SetJigsawBlock { .. } => {
            OperatorBlockEffect::SetJigsawFieldsThenMarkAndPublish
        }
        OperatorBlockRequest::SetStructureBlock {
            update,
            name_valid,
            operation_succeeded,
            ..
        } => OperatorBlockEffect::WriteStructureFieldsThenOperateMarkAndPublish {
            operation: structure_operation(update, name_valid && operation_succeeded),
        },
        OperatorBlockRequest::SetTestBlock { .. } => {
            OperatorBlockEffect::SetTestModeThenStateMessageMarkAndPublish
        }
        OperatorBlockRequest::TestInstanceBlockAction {
            action,
            test_key_resolves,
            operation_succeeded,
            ..
        } => match action {
            TestInstanceAction::Init => OperatorBlockEffect::ReplyTestInstanceStatusDirect {
                include_structure_size: false,
                missing_test: !test_key_resolves,
            },
            TestInstanceAction::Query => OperatorBlockEffect::ReplyTestInstanceStatusDirect {
                include_structure_size: test_key_resolves && operation_succeeded,
                missing_test: !test_key_resolves,
            },
            TestInstanceAction::Set
            | TestInstanceAction::Reset
            | TestInstanceAction::Save
            | TestInstanceAction::Export
            | TestInstanceAction::Run => {
                OperatorBlockEffect::InstallTestDataThenOperateAndPublishAirToCurrent {
                    action,
                    operation_succeeded,
                    publication_flags: 3,
                }
            }
        },
    }
}

const fn command_message(
    command_nonempty: bool,
    command_blocks_enabled: bool,
) -> CommandToolMessage {
    if !command_nonempty {
        CommandToolMessage::None
    } else if command_blocks_enabled {
        CommandToolMessage::Success
    } else {
        CommandToolMessage::Disabled
    }
}

const fn structure_operation(update: StructureUpdate, success: bool) -> StructureOperationEffect {
    match update {
        StructureUpdate::UpdateData => StructureOperationEffect::UpdateDataNoOperation,
        StructureUpdate::SaveArea | StructureUpdate::LoadArea | StructureUpdate::ScanArea => {
            StructureOperationEffect::RunAndReport { update, success }
        }
    }
}
