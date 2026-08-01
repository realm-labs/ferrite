use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::serverbound::codec::{
    PlayServerboundEntryCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::play::serverbound::operator_blocks::gate::{
    CommandToolMessage, OperatorBlockContext, OperatorBlockDecision, OperatorBlockEffect,
    OperatorBlockGates, StructureOperationEffect,
};
use ferrite_protocol::java_26_2::play::serverbound::operator_blocks::packet::{
    CommandBlockMode, JigsawJoint, OperatorBlockPacketKind, OperatorBlockRequest, StructureMode,
    StructureUpdate, TestBlockMode, TestInstanceAction, clamp_structure_integrity,
    clamp_structure_offset, clamp_structure_size,
};

fn context() -> OperatorBlockContext {
    OperatorBlockContext {
        instabuild: true,
        command_game_master: true,
    }
}

fn requests() -> [OperatorBlockRequest; 7] {
    [
        OperatorBlockRequest::JigsawGenerate {
            target_matches: true,
            levels: -1,
            keep_jigsaws: true,
        },
        OperatorBlockRequest::SetCommandBlock {
            target_matches: true,
            command_nonempty: true,
            track_output: true,
            command_blocks_enabled: true,
        },
        OperatorBlockRequest::SetCommandMinecart {
            target_matches: true,
            command_nonempty: true,
            track_output: true,
            command_blocks_enabled: true,
        },
        OperatorBlockRequest::SetJigsawBlock {
            target_matches: true,
        },
        OperatorBlockRequest::SetStructureBlock {
            target_matches: true,
            update: StructureUpdate::SaveArea,
            name_valid: true,
            operation_succeeded: true,
        },
        OperatorBlockRequest::SetTestBlock {
            target_matches: true,
        },
        OperatorBlockRequest::TestInstanceBlockAction {
            target_matches: true,
            action: TestInstanceAction::Set,
            test_key_resolves: true,
            operation_succeeded: true,
        },
    ]
}

#[test]
fn c4_operator_block_inventory_locks_all_seven_entries() {
    assert_eq!(OperatorBlockPacketKind::ALL.len(), 7);
    assert_eq!(
        OperatorBlockPacketKind::ALL
            .into_iter()
            .map(OperatorBlockPacketKind::wire_id)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([27, 54, 55, 58, 59, 60, 65])
    );
    for kind in OperatorBlockPacketKind::ALL {
        let descriptor = PacketCatalog::by_wire_id(
            ConnectionState::Play,
            PacketDirection::Serverbound,
            kind.wire_id(),
        )
        .unwrap();
        assert_eq!(descriptor.identity(), kind.identity());
    }
}

#[test]
fn c4_operator_block_enum_domains_clamps_and_fallbacks_are_exact() {
    assert_eq!(
        CommandBlockMode::from_strict_raw(0),
        Some(CommandBlockMode::Sequence)
    );
    assert_eq!(
        CommandBlockMode::from_strict_raw(2),
        Some(CommandBlockMode::Redstone)
    );
    assert_eq!(CommandBlockMode::from_strict_raw(3), None);
    assert_eq!(
        StructureUpdate::from_strict_raw(3),
        Some(StructureUpdate::ScanArea)
    );
    assert_eq!(StructureUpdate::from_strict_raw(-1), None);
    assert_eq!(StructureMode::from_strict_raw(3), Some(StructureMode::Data));
    assert_eq!(StructureMode::from_strict_raw(4), None);
    assert_eq!(
        JigsawJoint::from_fallback_name("rollable"),
        JigsawJoint::Rollable
    );
    assert_eq!(
        JigsawJoint::from_fallback_name("aligned"),
        JigsawJoint::Aligned
    );
    assert_eq!(
        JigsawJoint::from_fallback_name("unknown"),
        JigsawJoint::Aligned
    );
    assert_eq!(
        TestBlockMode::from_zero_fallback_raw(3),
        TestBlockMode::Accept
    );
    assert_eq!(
        TestBlockMode::from_zero_fallback_raw(4),
        TestBlockMode::Start
    );
    assert_eq!(
        TestInstanceAction::from_zero_fallback_raw(6),
        TestInstanceAction::Run
    );
    assert_eq!(
        TestInstanceAction::from_zero_fallback_raw(7),
        TestInstanceAction::Init
    );
    assert_eq!(clamp_structure_offset(-49), -48);
    assert_eq!(clamp_structure_offset(49), 48);
    assert_eq!(clamp_structure_size(-1), 0);
    assert_eq!(clamp_structure_size(49), 48);
    assert_eq!(clamp_structure_integrity(-0.1), 0.0);
    assert_eq!(clamp_structure_integrity(1.1), 1.0);
    assert!(clamp_structure_integrity(f32::NAN).is_nan());
}

#[test]
fn c4_operator_block_gate_is_default_closed() {
    for request in requests() {
        assert_eq!(
            OperatorBlockGates::default().decide(request, context()),
            OperatorBlockDecision::OmitDisabled
        );
    }
}

#[test]
fn c4_operator_permissions_require_both_instabuild_and_game_master() {
    let gates = OperatorBlockGates {
        operator_blocks: true,
    };
    for denied in [
        OperatorBlockContext::default(),
        OperatorBlockContext {
            instabuild: true,
            command_game_master: false,
        },
        OperatorBlockContext {
            instabuild: false,
            command_game_master: true,
        },
    ] {
        for request in requests() {
            let expected = if request.kind().is_command_tool() {
                OperatorBlockDecision::RefuseUnauthorizedWithCommandMessage(request.kind())
            } else {
                OperatorBlockDecision::OmitUnauthorized(request.kind())
            };
            assert_eq!(gates.decide(request, denied), expected);
        }
    }
}

#[test]
fn c4_operator_blocks_require_a_handler_time_matching_target() {
    let gates = OperatorBlockGates {
        operator_blocks: true,
    };
    for request in [
        OperatorBlockRequest::JigsawGenerate {
            target_matches: false,
            levels: i32::MAX,
            keep_jigsaws: false,
        },
        OperatorBlockRequest::SetCommandMinecart {
            target_matches: false,
            command_nonempty: true,
            track_output: false,
            command_blocks_enabled: true,
        },
        OperatorBlockRequest::SetStructureBlock {
            target_matches: false,
            update: StructureUpdate::UpdateData,
            name_valid: true,
            operation_succeeded: true,
        },
    ] {
        assert_eq!(
            gates.decide(request, context()),
            OperatorBlockDecision::OmitMissingOrWrongTarget(request.kind())
        );
    }
}

#[test]
fn c4_command_tools_mutate_while_disabled_and_clear_output_with_exact_messages() {
    let gates = OperatorBlockGates {
        operator_blocks: true,
    };
    for (request, effect) in [
        (
            OperatorBlockRequest::SetCommandBlock {
                target_matches: true,
                command_nonempty: true,
                track_output: false,
                command_blocks_enabled: false,
            },
            OperatorBlockEffect::UpdateCommandBlock {
                clear_last_output: true,
                call_update_hook: false,
                message: CommandToolMessage::Disabled,
            },
        ),
        (
            OperatorBlockRequest::SetCommandMinecart {
                target_matches: true,
                command_nonempty: false,
                track_output: true,
                command_blocks_enabled: true,
            },
            OperatorBlockEffect::UpdateCommandMinecart {
                clear_last_output: false,
                call_metadata_hook: true,
                message: CommandToolMessage::None,
            },
        ),
    ] {
        assert_eq!(
            gates.decide(request, context()),
            OperatorBlockDecision::Emit(effect)
        );
    }
}

#[test]
fn c4_structure_and_jigsaw_tools_preserve_raw_operation_boundaries() {
    let gates = OperatorBlockGates {
        operator_blocks: true,
    };
    assert_eq!(
        gates.decide(requests()[0], context()),
        OperatorBlockDecision::Emit(OperatorBlockEffect::GenerateJigsaw {
            levels: -1,
            keep_jigsaws: true,
        })
    );
    assert_eq!(
        gates.decide(requests()[3], context()),
        OperatorBlockDecision::Emit(OperatorBlockEffect::SetJigsawFieldsThenMarkAndPublish)
    );
    assert_eq!(
        gates.decide(
            OperatorBlockRequest::SetStructureBlock {
                target_matches: true,
                update: StructureUpdate::UpdateData,
                name_valid: false,
                operation_succeeded: false,
            },
            context(),
        ),
        OperatorBlockDecision::Emit(
            OperatorBlockEffect::WriteStructureFieldsThenOperateMarkAndPublish {
                operation: StructureOperationEffect::UpdateDataNoOperation,
            }
        )
    );
    assert_eq!(
        gates.decide(
            OperatorBlockRequest::SetStructureBlock {
                target_matches: true,
                update: StructureUpdate::LoadArea,
                name_valid: false,
                operation_succeeded: true,
            },
            context(),
        ),
        OperatorBlockDecision::Emit(
            OperatorBlockEffect::WriteStructureFieldsThenOperateMarkAndPublish {
                operation: StructureOperationEffect::RunAndReport {
                    update: StructureUpdate::LoadArea,
                    success: false,
                },
            }
        )
    );
}

#[test]
fn c4_test_tools_separate_direct_query_from_install_then_publish_mutation() {
    let gates = OperatorBlockGates {
        operator_blocks: true,
    };
    assert_eq!(
        gates.decide(requests()[5], context()),
        OperatorBlockDecision::Emit(OperatorBlockEffect::SetTestModeThenStateMessageMarkAndPublish)
    );
    assert_eq!(
        gates.decide(
            OperatorBlockRequest::TestInstanceBlockAction {
                target_matches: true,
                action: TestInstanceAction::Query,
                test_key_resolves: true,
                operation_succeeded: true,
            },
            context(),
        ),
        OperatorBlockDecision::Emit(OperatorBlockEffect::ReplyTestInstanceStatusDirect {
            include_structure_size: true,
            missing_test: false,
        })
    );
    assert_eq!(
        gates.decide(
            OperatorBlockRequest::TestInstanceBlockAction {
                target_matches: true,
                action: TestInstanceAction::Run,
                test_key_resolves: true,
                operation_succeeded: false,
            },
            context(),
        ),
        OperatorBlockDecision::Emit(
            OperatorBlockEffect::InstallTestDataThenOperateAndPublishAirToCurrent {
                action: TestInstanceAction::Run,
                operation_succeeded: false,
                publication_flags: 3,
            }
        )
    );
    assert!(!TestInstanceAction::Query.installs_data());
    assert!(TestInstanceAction::Run.installs_data());
}

#[test]
fn c4_operator_block_required_decoder_remains_fail_closed() {
    for packet in OperatorBlockPacketKind::ALL {
        assert!(matches!(
            decode_required_packet(&[u8::try_from(packet.wire_id()).unwrap()]),
            Err(PlayServerboundEntryCodecError::UnsupportedPacketIdentity { identity })
                if identity == packet.identity()
        ));
    }
}
