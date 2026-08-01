use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::serverbound::admin_state::gate::{
    AdminStateContext, AdminStateDecision, AdminStateEffect, AdminStateGates, AdminStateService,
};
use ferrite_protocol::java_26_2::play::serverbound::admin_state::packet::{
    AdminStatePacketKind, AdminStateRequest, CreativeStackClass, Difficulty, GameMode,
};
use ferrite_protocol::java_26_2::play::serverbound::codec::{
    PlayServerboundEntryCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

fn context() -> AdminStateContext {
    AdminStateContext {
        command_game_master: true,
        singleplayer_owner: false,
        infinite_materials: true,
        difficulty_locked: false,
        hardcore: false,
    }
}

fn requests() -> [AdminStateRequest; 7] {
    [
        AdminStateRequest::BlockEntityTagQuery {
            target_exists: true,
        },
        AdminStateRequest::ChangeDifficulty {
            requested: Difficulty::Normal,
        },
        AdminStateRequest::ChangeGameMode {
            requested: GameMode::Creative,
        },
        AdminStateRequest::EntityTagQuery {
            target_exists: true,
        },
        AdminStateRequest::LockDifficulty,
        AdminStateRequest::SetCreativeModeSlot {
            slot: 1,
            stack: CreativeStackClass::Item,
            feature_enabled: true,
            count_within_maximum: true,
            drop_throttle: 0,
        },
        AdminStateRequest::SetGameRule,
    ]
}

fn all_gates() -> AdminStateGates {
    AdminStateGates {
        tag_queries: true,
        difficulty: true,
        game_mode: true,
        creative_inventory: true,
        game_rules: true,
    }
}

#[test]
fn c4_admin_state_inventory_locks_all_seven_catalog_entries() {
    assert_eq!(AdminStatePacketKind::ALL.len(), 7);
    assert_eq!(
        AdminStatePacketKind::ALL
            .into_iter()
            .map(AdminStatePacketKind::wire_id)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([2, 4, 5, 25, 29, 56, 57])
    );
    for packet in AdminStatePacketKind::ALL {
        let descriptor = PacketCatalog::by_wire_id(
            ConnectionState::Play,
            PacketDirection::Serverbound,
            packet.wire_id(),
        )
        .unwrap();
        assert_eq!(descriptor.identity(), packet.identity());
    }
}

#[test]
fn c4_admin_state_simple_zero_goldens_lock_outer_fields() {
    let expected = [
        vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![4, 0],
        vec![5, 0],
        vec![25, 0, 0],
        vec![29, 0],
        vec![56, 0, 0, 0],
        vec![57, 0],
    ];
    let mut actual = Vec::new();
    for packet in AdminStatePacketKind::ALL {
        let mut writer = WireWriter::new(16);
        writer.write_var_i32(packet.wire_id()).unwrap();
        match packet {
            AdminStatePacketKind::BlockEntityTagQuery => {
                writer.write_var_i32(0).unwrap();
                writer.write_i64(0).unwrap();
            }
            AdminStatePacketKind::ChangeDifficulty
            | AdminStatePacketKind::ChangeGameMode
            | AdminStatePacketKind::EntityTagQuery
            | AdminStatePacketKind::SetGameRule => writer.write_var_i32(0).unwrap(),
            AdminStatePacketKind::LockDifficulty => writer.write_bool(false).unwrap(),
            AdminStatePacketKind::SetCreativeModeSlot => {
                writer.write_i16(0).unwrap();
                writer.write_var_i32(0).unwrap();
            }
        }
        if matches!(packet, AdminStatePacketKind::EntityTagQuery) {
            writer.write_var_i32(0).unwrap();
        }
        actual.push(writer.into_inner());
    }
    assert_eq!(actual, expected);
}

#[test]
fn c4_admin_state_default_gates_omit_every_service() {
    let gates = AdminStateGates::default();
    let expected = [
        AdminStateService::TagQueries,
        AdminStateService::Difficulty,
        AdminStateService::GameMode,
        AdminStateService::TagQueries,
        AdminStateService::Difficulty,
        AdminStateService::CreativeInventory,
        AdminStateService::GameRules,
    ];
    for (request, service) in requests().into_iter().zip(expected) {
        assert_eq!(
            gates.decide(request, context()),
            AdminStateDecision::OmitDisabled(service)
        );
    }
}

#[test]
fn c4_admin_state_enum_mappers_preserve_wrapping_and_zero_fallback() {
    assert_eq!(Difficulty::from_wrapping_raw(-1), Difficulty::Hard);
    assert_eq!(Difficulty::from_wrapping_raw(4), Difficulty::Peaceful);
    assert_eq!(Difficulty::from_wrapping_raw(i32::MAX), Difficulty::Hard);
    assert_eq!(GameMode::from_zero_fallback_raw(3), GameMode::Spectator);
    assert_eq!(GameMode::from_zero_fallback_raw(-1), GameMode::Survival);
    assert_eq!(GameMode::from_zero_fallback_raw(4), GameMode::Survival);
}

#[test]
fn c4_admin_permissions_distinguish_warning_and_silent_denials() {
    let denied = AdminStateContext::default();
    assert_eq!(
        all_gates().decide(requests()[1], denied),
        AdminStateDecision::RefuseUnauthorizedWithWarning(AdminStatePacketKind::ChangeDifficulty)
    );
    assert_eq!(
        all_gates().decide(requests()[2], denied),
        AdminStateDecision::RefuseUnauthorizedWithWarning(AdminStatePacketKind::ChangeGameMode)
    );
    for request in [requests()[0], requests()[3], requests()[4], requests()[6]] {
        assert_eq!(
            all_gates().decide(request, denied),
            AdminStateDecision::OmitUnauthorized(request.kind())
        );
    }
    let owner = AdminStateContext {
        singleplayer_owner: true,
        ..denied
    };
    assert!(matches!(
        all_gates().decide(requests()[1], owner),
        AdminStateDecision::Emit(_)
    ));
    assert!(matches!(
        all_gates().decide(requests()[4], owner),
        AdminStateDecision::Emit(_)
    ));
}

#[test]
fn c4_admin_difficulty_queries_and_gamerules_keep_exact_semantics() {
    assert_eq!(
        all_gates().decide(
            AdminStateRequest::ChangeDifficulty {
                requested: Difficulty::Easy,
            },
            AdminStateContext {
                hardcore: true,
                ..context()
            }
        ),
        AdminStateDecision::Emit(AdminStateEffect::UpdateDifficultyAndBroadcast {
            effective: Difficulty::Hard,
        })
    );
    assert_eq!(
        all_gates().decide(
            requests()[1],
            AdminStateContext {
                difficulty_locked: true,
                ..context()
            }
        ),
        AdminStateDecision::Emit(AdminStateEffect::NoopLockedDifficulty)
    );
    assert_eq!(
        all_gates().decide(
            AdminStateRequest::BlockEntityTagQuery {
                target_exists: false,
            },
            context()
        ),
        AdminStateDecision::Emit(AdminStateEffect::ReplyBlockEntityTag { present: false })
    );
    assert_eq!(
        all_gates().decide(
            AdminStateRequest::EntityTagQuery {
                target_exists: false,
            },
            context()
        ),
        AdminStateDecision::OmitMissingEntity
    );
    assert_eq!(
        all_gates().decide(AdminStateRequest::SetGameRule, context()),
        AdminStateDecision::Emit(AdminStateEffect::ApplyGameRulesSequentially)
    );
}

#[test]
fn c4_creative_slot_locks_slots_air_and_drop_throttle_boundaries() {
    let decide = |slot, stack, drop_throttle| {
        all_gates().decide(
            AdminStateRequest::SetCreativeModeSlot {
                slot,
                stack,
                feature_enabled: true,
                count_within_maximum: true,
                drop_throttle,
            },
            context(),
        )
    };
    assert_eq!(
        decide(1, CreativeStackClass::EmptyOrAir, 0),
        AdminStateDecision::Emit(AdminStateEffect::ClearInventorySlotAndRemoteMirror)
    );
    assert_eq!(
        decide(45, CreativeStackClass::Item, 0),
        AdminStateDecision::Emit(AdminStateEffect::SetInventorySlotAndRemoteMirror)
    );
    assert_eq!(
        decide(0, CreativeStackClass::Item, 0),
        AdminStateDecision::OmitInvalidCreativeSlot
    );
    assert_eq!(
        decide(-1, CreativeStackClass::EmptyOrAir, 1_479),
        AdminStateDecision::Emit(AdminStateEffect::ConsumeEmptyDropThrottle)
    );
    assert_eq!(
        decide(-1, CreativeStackClass::Item, 1_479),
        AdminStateDecision::Emit(AdminStateEffect::DropItemAndConsumeThrottle)
    );
    assert_eq!(
        decide(-1, CreativeStackClass::Item, 1_480),
        AdminStateDecision::OmitDropThrottled
    );
}

#[test]
fn c4_admin_state_required_decoder_remains_fail_closed() {
    for packet in AdminStatePacketKind::ALL {
        assert!(matches!(
            decode_required_packet(&[u8::try_from(packet.wire_id()).unwrap()]),
            Err(PlayServerboundEntryCodecError::UnsupportedPacketIdentity { identity })
                if identity == packet.identity()
        ));
    }
}
