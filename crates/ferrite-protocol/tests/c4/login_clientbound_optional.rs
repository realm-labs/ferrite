use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::login::clientbound::codec::{
    LoginClientboundCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::login::clientbound::optional::{
    LoginClientboundGates, LoginClientboundOptionalService, OptionalLoginClientboundCodecError,
    OptionalLoginClientboundContext, OptionalLoginClientboundDecision,
    OptionalLoginClientboundEffect, OptionalLoginClientboundPacket,
    OptionalLoginClientboundPacketKind, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::wire::error::WireError;
use ferrite_protocol::java_26_2::wire::frame::MAX_FRAME_LENGTH;
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

fn identifier(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn context(valid_hello_received: bool, memory_connection: bool) -> OptionalLoginClientboundContext {
    OptionalLoginClientboundContext {
        valid_hello_received,
        memory_connection,
    }
}

fn packets() -> [OptionalLoginClientboundPacket; 3] {
    [
        OptionalLoginClientboundPacket::EncryptionHello {
            server_id: String::new(),
            public_key: vec![1, 2, 3],
            challenge: vec![4, 5, 6, 7],
            authenticate: true,
        },
        OptionalLoginClientboundPacket::CustomQuery {
            transaction_id: -7,
            channel: identifier("ferrite:query"),
            payload: vec![0, 1, 255],
        },
        OptionalLoginClientboundPacket::CookieRequest {
            key: identifier("ferrite:cookie"),
        },
    ]
}

#[test]
fn c4_login_clientbound_inventory_locks_all_three_catalog_entries() {
    assert_eq!(OptionalLoginClientboundPacketKind::ALL.len(), 3);
    let ids = OptionalLoginClientboundPacketKind::ALL
        .into_iter()
        .map(OptionalLoginClientboundPacketKind::wire_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, BTreeSet::from([1, 4, 5]));
    for packet in OptionalLoginClientboundPacketKind::ALL {
        let descriptor = PacketCatalog::by_wire_id(
            ConnectionState::Login,
            PacketDirection::Clientbound,
            packet.wire_id(),
        )
        .unwrap();
        assert_eq!(descriptor.identity(), packet.identity());
    }
}

#[test]
fn c4_login_clientbound_optional_codec_round_trips_exact_fields() {
    for packet in packets() {
        assert_eq!(
            decode_packet(&encode_packet(&packet).unwrap()).unwrap(),
            packet
        );
    }
}

#[test]
fn c4_login_clientbound_codec_preserves_nonzero_boolean_and_field_bounds() {
    let mut hello = WireWriter::new(MAX_FRAME_LENGTH);
    hello.write_var_i32(1).unwrap();
    hello.write_utf("", 20).unwrap();
    hello.write_byte_array(&[1], 1).unwrap();
    hello.write_byte_array(&[2], 1).unwrap();
    hello.write_u8(255).unwrap();
    assert!(matches!(
        decode_packet(&hello.into_inner()).unwrap(),
        OptionalLoginClientboundPacket::EncryptionHello {
            authenticate: true,
            ..
        }
    ));

    let oversized_server_id = OptionalLoginClientboundPacket::EncryptionHello {
        server_id: "x".repeat(21),
        public_key: Vec::new(),
        challenge: Vec::new(),
        authenticate: true,
    };
    assert!(matches!(
        encode_packet(&oversized_server_id),
        Err(OptionalLoginClientboundCodecError::Wire(
            WireError::UtfCodeUnitLimit { maximum: 20, .. }
        ))
    ));

    let oversized_query = OptionalLoginClientboundPacket::CustomQuery {
        transaction_id: 0,
        channel: identifier("ferrite:query"),
        payload: vec![0; 1_048_577],
    };
    assert!(matches!(
        encode_packet(&oversized_query),
        Err(OptionalLoginClientboundCodecError::Wire(
            WireError::LengthLimit {
                maximum: 1_048_576,
                ..
            }
        ))
    ));
}

#[test]
fn c4_required_and_optional_login_decoders_remain_fail_closed_by_family() {
    assert!(matches!(
        decode_required_packet(&[1]),
        Err(LoginClientboundCodecError::UnsupportedPacketIdentity {
            identity: "minecraft:hello"
        })
    ));
    assert!(matches!(
        decode_packet(&[2]),
        Err(OptionalLoginClientboundCodecError::RequiredPacketIdentity {
            identity: "minecraft:login_finished"
        })
    ));
}

#[test]
fn c4_default_offline_gate_omits_all_optional_login_output() {
    let gates = LoginClientboundGates::default();
    for packet in packets() {
        assert_eq!(
            gates.decide(&packet, context(true, false)),
            OptionalLoginClientboundDecision::OmitDisabled(packet.kind().service())
        );
    }
}

#[test]
fn c4_encryption_hello_requires_valid_hello_and_non_memory_transport() {
    let gates = LoginClientboundGates {
        online_authentication: true,
        ..LoginClientboundGates::default()
    };
    let hello = &packets()[0];
    assert_eq!(
        gates.decide(hello, context(false, false)),
        OptionalLoginClientboundDecision::RefuseEncryptionBeforeValidHello
    );
    assert_eq!(
        gates.decide(hello, context(true, true)),
        OptionalLoginClientboundDecision::OmitEncryptionForMemoryConnection
    );
    assert_eq!(
        gates.decide(hello, context(true, false)),
        OptionalLoginClientboundDecision::Emit(OptionalLoginClientboundEffect::EnterKeyStage)
    );
}

#[test]
fn c4_query_and_cookie_emit_only_correlation_effects_for_owned_services() {
    let gates = LoginClientboundGates {
        custom_query: true,
        cookies: true,
        ..LoginClientboundGates::default()
    };
    let [hello, query, cookie] = packets();
    assert_eq!(
        gates.decide(&hello, context(true, false)),
        OptionalLoginClientboundDecision::OmitDisabled(
            LoginClientboundOptionalService::OnlineAuthentication
        )
    );
    assert_eq!(
        gates.decide(&query, context(true, false)),
        OptionalLoginClientboundDecision::Emit(
            OptionalLoginClientboundEffect::RegisterCorrelatedQuery
        )
    );
    assert_eq!(
        gates.decide(&cookie, context(true, false)),
        OptionalLoginClientboundDecision::Emit(
            OptionalLoginClientboundEffect::RegisterCookieRequest
        )
    );
}
