use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::debug_projection::gate::{
    DebugProjectionContext, DebugProjectionDecision, DebugProjectionEffect, DebugProjectionGates,
    DebugProjectionLifecycleEffect, DebugProjectionLifecycleEvent, DebugProjectionTarget,
    lifecycle_effect,
};
use ferrite_protocol::java_26_2::play::clientbound::debug_projection::packet::{
    DebugProjectionPacket, DebugProjectionPacketKind, DebugRetention, DebugSubscription,
    DebugValueState,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::wire::compression::{CompressionMode, encode_packet};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;

fn context() -> DebugProjectionContext {
    DebugProjectionContext {
        service_registered: true,
        authorized: true,
        requested: true,
        target_tracked: true,
        entity_resolved: true,
        dedicated_server: true,
    }
}

fn packets() -> [DebugProjectionPacket; 5] {
    [
        DebugProjectionPacket::BlockValue {
            subscription: DebugSubscription::PointsOfInterest,
            state: DebugValueState::Replace,
        },
        DebugProjectionPacket::ChunkValue {
            subscription: DebugSubscription::Structures,
            state: DebugValueState::Clear,
        },
        DebugProjectionPacket::EntityValue {
            subscription: DebugSubscription::Brains,
            state: DebugValueState::Replace,
        },
        DebugProjectionPacket::Event {
            subscription: DebugSubscription::GameEvents,
        },
        DebugProjectionPacket::Sample,
    ]
}

#[test]
fn c4_debug_projection_inventory_locks_all_five_catalog_entries() {
    assert_eq!(DebugProjectionPacketKind::ALL.len(), 5);
    let ids = DebugProjectionPacketKind::ALL
        .into_iter()
        .map(DebugProjectionPacketKind::wire_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids, BTreeSet::from([26, 27, 28, 29, 30]));
    for packet in DebugProjectionPacketKind::ALL {
        let descriptor = PacketCatalog::by_wire_id(
            ConnectionState::Play,
            PacketDirection::Clientbound,
            packet.wire_id(),
        )
        .unwrap();
        assert_eq!(descriptor.identity(), packet.identity());
    }
}

#[test]
fn c4_debug_subscription_registry_locks_ids_names_and_retention() {
    assert_eq!(DebugSubscription::ALL.len(), 16);
    for (raw_id, subscription) in DebugSubscription::ALL.into_iter().enumerate() {
        assert_eq!(subscription.raw_id(), i32::try_from(raw_id).unwrap());
        assert!(subscription.identity().starts_with("minecraft:"));
    }
    assert_eq!(
        DebugSubscription::DedicatedServerTickTime.retention(),
        DebugRetention::SampleOnly
    );
    assert_eq!(
        DebugSubscription::EntityBlockIntersections.retention(),
        DebugRetention::Expiring { ticks: 100 }
    );
    assert_eq!(
        DebugSubscription::RedstoneWireOrientations.retention(),
        DebugRetention::Expiring { ticks: 200 }
    );
    assert_eq!(
        DebugSubscription::NeighborUpdates.retention(),
        DebugRetention::Expiring { ticks: 200 }
    );
    assert_eq!(
        DebugSubscription::GameEvents.retention(),
        DebugRetention::Expiring { ticks: 60 }
    );
}

#[test]
fn c4_debug_projection_default_gate_omits_every_packet() {
    let gates = DebugProjectionGates::default();
    for packet in packets() {
        assert_eq!(
            gates.decide(packet, context()),
            DebugProjectionDecision::OmitDisabled(packet.kind())
        );
    }
}

#[test]
fn c4_debug_projection_enabled_gate_degrades_without_registered_service() {
    let gates = DebugProjectionGates { diagnostics: true };
    let unavailable = DebugProjectionContext {
        service_registered: false,
        ..context()
    };
    for packet in packets() {
        assert_eq!(
            gates.decide(packet, unavailable),
            DebugProjectionDecision::DegradeServiceUnavailable
        );
    }
}

#[test]
fn c4_debug_projection_refuses_or_omits_invalid_audiences() {
    let gates = DebugProjectionGates { diagnostics: true };
    let packet = packets()[0];
    assert_eq!(
        gates.decide(
            packet,
            DebugProjectionContext {
                authorized: false,
                ..context()
            }
        ),
        DebugProjectionDecision::RefuseUnauthorized
    );
    assert_eq!(
        gates.decide(
            packet,
            DebugProjectionContext {
                requested: false,
                ..context()
            }
        ),
        DebugProjectionDecision::OmitUnrequested
    );
    assert_eq!(
        gates.decide(
            packet,
            DebugProjectionContext {
                target_tracked: false,
                ..context()
            }
        ),
        DebugProjectionDecision::OmitUntrackedTarget
    );
    assert_eq!(
        gates.decide(
            packets()[2],
            DebugProjectionContext {
                entity_resolved: false,
                ..context()
            }
        ),
        DebugProjectionDecision::OmitMissingEntity
    );
    assert_eq!(
        gates.decide(
            DebugProjectionPacket::BlockValue {
                subscription: DebugSubscription::DedicatedServerTickTime,
                state: DebugValueState::Clear,
            },
            context()
        ),
        DebugProjectionDecision::RefuseSampleOnlyValue
    );
    assert_eq!(
        gates.decide(
            DebugProjectionPacket::Sample,
            DebugProjectionContext {
                dedicated_server: false,
                ..context()
            }
        ),
        DebugProjectionDecision::OmitUnsupportedEnvironment
    );
}

#[test]
fn c4_debug_projection_effects_remain_connection_local() {
    let gates = DebugProjectionGates { diagnostics: true };
    let expected = [
        DebugProjectionEffect::ReplaceValue {
            target: DebugProjectionTarget::Block,
            retention: DebugRetention::Persistent,
        },
        DebugProjectionEffect::ClearValue {
            target: DebugProjectionTarget::Chunk,
        },
        DebugProjectionEffect::ReplaceValue {
            target: DebugProjectionTarget::Entity,
            retention: DebugRetention::Persistent,
        },
        DebugProjectionEffect::AppendEvent {
            retention: DebugRetention::Expiring { ticks: 60 },
        },
        DebugProjectionEffect::LogSampleImmediately,
    ];
    for (packet, effect) in packets().into_iter().zip(expected) {
        assert_eq!(
            gates.decide(packet, context()),
            DebugProjectionDecision::Emit(effect)
        );
    }
}

#[test]
fn c4_debug_expiry_and_lifecycle_clear_at_the_exact_boundaries() {
    let retention = DebugSubscription::GameEvents.retention();
    assert!(!retention.is_expired(1_000, 1_059));
    assert!(retention.is_expired(1_000, 1_060));
    assert!(retention.is_expired(1_000, 1_061));
    assert!(!DebugRetention::Persistent.is_expired(1_000, i64::MAX));
    for event in [
        DebugProjectionLifecycleEvent::Reconfiguration,
        DebugProjectionLifecycleEvent::Reconnect,
        DebugProjectionLifecycleEvent::Disconnect,
    ] {
        assert_eq!(
            lifecycle_effect(event),
            DebugProjectionLifecycleEffect::ClearProjectionAndRequestedSubscriptions
        );
    }
}

#[test]
fn c4_gold_debug_sample_empty_and_required_family_boundary_fail_closed() {
    let body = [
        u8::try_from(DebugProjectionPacketKind::Sample.wire_id()).unwrap(),
        0,
        u8::try_from(DebugSubscription::DedicatedServerTickTime.raw_id()).unwrap(),
    ];
    assert_eq!(
        encode_packet(
            &body,
            CompressionMode::enabled(256).unwrap(),
            FrameLimits::default(),
        )
        .unwrap(),
        [4, 0, 30, 0, 0]
    );

    let registries = PlayRegistries::default();
    let values = RejectComponentValues;
    assert!(matches!(
        decode_required_packet(
            &[DebugProjectionPacketKind::BlockValue.wire_id() as u8],
            PlayDecodeContext {
                registries: &registries,
                component_values: &values,
                dimension_section_count: 24,
            },
        ),
        Err(PlayClientboundCodecError::UnsupportedPacketIdentity {
            identity: "minecraft:debug/block_value"
        })
    ));
}
