use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::login::serverbound::codec::{
    LoginServerboundCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::login::serverbound::optional::{
    LoginServerboundGate, LoginServerboundGates, LoginServerboundOptionalService,
    OptionalLoginServerDecision, OptionalLoginServerGateError, OptionalLoginServerTask,
    OptionalLoginServerboundCodecError, OptionalLoginServerboundPacket,
    OptionalLoginServerboundPacketKind, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::wire::error::WireError;

fn identifier(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn packets() -> [OptionalLoginServerboundPacket; 3] {
    [
        OptionalLoginServerboundPacket::Key {
            encrypted_secret: vec![1, 2, 3],
            encrypted_challenge: vec![4, 5, 6],
        },
        OptionalLoginServerboundPacket::null_custom_query_answer(-7),
        OptionalLoginServerboundPacket::CookieResponse {
            key: identifier("ferrite:cookie"),
            value: Some(vec![0, 1, 255]),
        },
    ]
}

#[test]
fn c4_login_serverbound_inventory_locks_all_three_catalog_entries() {
    assert_eq!(OptionalLoginServerboundPacketKind::ALL.len(), 3);
    let ids = OptionalLoginServerboundPacketKind::ALL
        .into_iter()
        .map(OptionalLoginServerboundPacketKind::wire_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, BTreeSet::from([1, 2, 4]));
    for packet in OptionalLoginServerboundPacketKind::ALL {
        let descriptor = PacketCatalog::by_wire_id(
            ConnectionState::Login,
            PacketDirection::Serverbound,
            packet.wire_id(),
        )
        .unwrap();
        assert_eq!(descriptor.identity(), packet.identity());
    }
}

#[test]
fn c4_login_serverbound_optional_codec_round_trips_exact_fields() {
    for packet in packets() {
        assert_eq!(
            decode_packet(&encode_packet(&packet).unwrap()).unwrap(),
            packet
        );
    }
    let absent = OptionalLoginServerboundPacket::CookieResponse {
        key: identifier("ferrite:absent"),
        value: None,
    };
    assert_eq!(
        decode_packet(&encode_packet(&absent).unwrap()).unwrap(),
        absent
    );
}

#[test]
fn c4_login_serverbound_codec_enforces_answer_and_cookie_bounds() {
    let oversized_answer = OptionalLoginServerboundPacket::CustomQueryAnswer {
        transaction_id: 0,
        remainder: vec![0; 1_048_577],
    };
    assert!(matches!(
        encode_packet(&oversized_answer),
        Err(OptionalLoginServerboundCodecError::Wire(
            WireError::LengthLimit {
                maximum: 1_048_576,
                ..
            }
        ))
    ));

    let oversized_cookie = OptionalLoginServerboundPacket::CookieResponse {
        key: identifier("ferrite:cookie"),
        value: Some(vec![0; 5_121]),
    };
    assert!(matches!(
        encode_packet(&oversized_cookie),
        Err(OptionalLoginServerboundCodecError::Wire(
            WireError::LengthLimit { maximum: 5_120, .. }
        ))
    ));
}

#[test]
fn c4_required_and_optional_login_input_decoders_remain_fail_closed_by_family() {
    assert!(matches!(
        decode_required_packet(&[1]),
        Err(LoginServerboundCodecError::UnsupportedPacketIdentity {
            identity: "minecraft:key"
        })
    ));
    assert!(matches!(
        decode_packet(&[3]),
        Err(OptionalLoginServerboundCodecError::RequiredPacketIdentity {
            identity: "minecraft:login_acknowledged"
        })
    ));
}

#[test]
fn c4_default_login_server_gate_rejects_every_optional_service() {
    let services = [
        LoginServerboundOptionalService::OnlineAuthentication,
        LoginServerboundOptionalService::CustomQuery,
        LoginServerboundOptionalService::Cookies,
    ];
    for (packet, service) in packets().into_iter().zip(services) {
        let mut gate = LoginServerboundGate::new(
            LoginServerboundGates::default(),
            OptionalLoginServerTask::None,
        );
        assert_eq!(
            gate.apply(packet),
            Err(OptionalLoginServerGateError::Disabled { service })
        );
    }
}

#[test]
fn c4_key_is_legal_only_in_key_and_installs_encryption_after_verification() {
    let gates = LoginServerboundGates {
        online_authentication: true,
        ..LoginServerboundGates::default()
    };
    let mut unsolicited = LoginServerboundGate::new(gates, OptionalLoginServerTask::None);
    assert!(matches!(
        unsolicited.apply(packets()[0].clone()),
        Err(OptionalLoginServerGateError::UnexpectedTask { .. })
    ));

    let expected_challenge = vec![9, 8, 7, 6];
    let mut gate = LoginServerboundGate::new(
        gates,
        OptionalLoginServerTask::Key {
            expected_challenge: expected_challenge.clone(),
        },
    );
    assert_eq!(
        gate.apply(packets()[0].clone()),
        Ok(OptionalLoginServerDecision::DecryptAndVerifyKey {
            encrypted_secret: vec![1, 2, 3],
            encrypted_challenge: vec![4, 5, 6],
            expected_challenge,
        })
    );
    assert_eq!(
        gate.task(),
        &OptionalLoginServerTask::KeyVerificationPending
    );
    assert_eq!(
        gate.key_verified(),
        Ok(OptionalLoginServerDecision::InstallEncryptionThenAuthenticate)
    );
    assert_eq!(gate.task(), &OptionalLoginServerTask::Authenticating);
    assert!(matches!(
        gate.key_verified(),
        Err(OptionalLoginServerGateError::UnexpectedKeyVerification { .. })
    ));
}

#[test]
fn c4_custom_query_answer_requires_matching_owned_transaction_and_consumes_it() {
    let gates = LoginServerboundGates {
        custom_query: true,
        ..LoginServerboundGates::default()
    };
    let mut gate = LoginServerboundGate::new(
        gates,
        OptionalLoginServerTask::CustomQuery { transaction_id: 7 },
    );
    assert!(matches!(
        gate.apply(OptionalLoginServerboundPacket::null_custom_query_answer(8)),
        Err(OptionalLoginServerGateError::UnexpectedTask { .. })
    ));
    assert_eq!(
        gate.apply(OptionalLoginServerboundPacket::null_custom_query_answer(7)),
        Ok(OptionalLoginServerDecision::CustomQueryAnswer { remainder: vec![0] })
    );
    assert_eq!(gate.task(), &OptionalLoginServerTask::None);
}

#[test]
fn c4_cookie_response_requires_matching_owned_key_and_consumes_it() {
    let key = identifier("ferrite:cookie");
    let mut gate = LoginServerboundGate::new(
        LoginServerboundGates {
            cookies: true,
            ..LoginServerboundGates::default()
        },
        OptionalLoginServerTask::CookieRequest { key: key.clone() },
    );
    assert!(matches!(
        gate.apply(OptionalLoginServerboundPacket::CookieResponse {
            key: identifier("ferrite:wrong"),
            value: None,
        }),
        Err(OptionalLoginServerGateError::CookieKeyMismatch { .. })
    ));
    assert_eq!(
        gate.apply(OptionalLoginServerboundPacket::CookieResponse {
            key,
            value: Some(vec![1]),
        }),
        Ok(OptionalLoginServerDecision::CookieResponse {
            value: Some(vec![1])
        })
    );
    assert_eq!(gate.task(), &OptionalLoginServerTask::None);
}
