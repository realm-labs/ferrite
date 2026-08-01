use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::clientbound::debug_projection::packet::DebugSubscription;
use ferrite_protocol::java_26_2::play::serverbound::codec::{
    PlayServerboundEntryCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::play::serverbound::debug_subscription::gate::{
    DebugSubscriptionRequestContext, DebugSubscriptionRequestDecision,
    DebugSubscriptionRequestGates,
};
use ferrite_protocol::java_26_2::play::serverbound::debug_subscription::packet::{
    DebugSubscriptionRequest, DebugSubscriptionRequestKind, DebugSubscriptionSet,
    DebugSubscriptionSetError,
};
use ferrite_protocol::java_26_2::play::serverbound::debug_subscription::state::{
    DebugSubscriptionAuthorization, DebugSubscriptionLifecycleEvent, DebugSubscriptionRuntime,
    DebugSynchronizerTransition, apply_lifecycle,
};
use ferrite_protocol::java_26_2::wire::compression::{CompressionMode, encode_packet};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

#[test]
fn c4_debug_subscription_request_locks_catalog_identity() {
    assert_eq!(DebugSubscriptionRequestKind::ALL.len(), 1);
    let kind = DebugSubscriptionRequestKind::Replace;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Play,
        PacketDirection::Serverbound,
        kind.wire_id(),
    )
    .unwrap();
    assert_eq!(kind.wire_id(), 23);
    assert_eq!(descriptor.identity(), kind.identity());
}

#[test]
fn c4_gold_debug_subscription_empty_set_matches_locked_frame() {
    let mut writer = WireWriter::new(2);
    writer
        .write_var_i32(DebugSubscriptionRequestKind::Replace.wire_id())
        .unwrap();
    writer.write_var_i32(0).unwrap();
    assert_eq!(
        encode_packet(
            &writer.into_inner(),
            CompressionMode::enabled(256).unwrap(),
            FrameLimits::default(),
        )
        .unwrap(),
        [3, 0, 23, 0]
    );
}

#[test]
fn c4_debug_subscription_set_is_strict_capped_and_collapses_duplicates() {
    let all = DebugSubscription::ALL.map(DebugSubscription::raw_id);
    let subscriptions = DebugSubscriptionSet::from_raw_ids(&all).unwrap();
    assert_eq!(subscriptions.len(), 16);
    for subscription in DebugSubscription::ALL {
        assert!(subscriptions.contains(subscription));
    }

    let duplicates = DebugSubscriptionSet::from_raw_ids(&[2, 2, 2]).unwrap();
    assert_eq!(duplicates.len(), 1);
    assert!(duplicates.contains(DebugSubscription::Brains));
    assert_eq!(
        DebugSubscriptionSet::from_raw_ids(&[16]),
        Err(DebugSubscriptionSetError::UnknownRawId { raw_id: 16 })
    );
    assert_eq!(
        DebugSubscriptionSet::from_raw_ids(&[0; 33]),
        Err(DebugSubscriptionSetError::TooManyEncoded { count: 33 })
    );
}

#[test]
fn c4_debug_subscription_gate_is_default_closed_and_requires_registered_service() {
    let requested = DebugSubscriptionSet::from_raw_ids(&[0, 2]).unwrap();
    let request = DebugSubscriptionRequest { requested };
    assert_eq!(
        DebugSubscriptionRequestGates::default().decide(
            request,
            DebugSubscriptionRequestContext {
                service_registered: true,
            },
        ),
        DebugSubscriptionRequestDecision::OmitDisabled(DebugSubscriptionRequestKind::Replace)
    );
    assert_eq!(
        DebugSubscriptionRequestGates { diagnostics: true }
            .decide(request, DebugSubscriptionRequestContext::default(),),
        DebugSubscriptionRequestDecision::DegradeServiceUnavailable
    );
    assert_eq!(
        DebugSubscriptionRequestGates { diagnostics: true }.decide(
            request,
            DebugSubscriptionRequestContext {
                service_registered: true,
            },
        ),
        DebugSubscriptionRequestDecision::ReplaceOnLevelThread { requested }
    );
}

#[test]
fn c4_debug_subscription_replaces_the_whole_requested_set() {
    let mut runtime = DebugSubscriptionRuntime::default();
    runtime.replace_requested(DebugSubscriptionSet::from_raw_ids(&[1, 2]).unwrap());
    runtime.replace_requested(DebugSubscriptionSet::from_raw_ids(&[3]).unwrap());
    assert_eq!(runtime.requested().len(), 1);
    assert!(runtime.requested().contains(DebugSubscription::Breezes));
    assert!(!runtime.requested().contains(DebugSubscription::Bees));
}

#[test]
fn c4_unauthorized_requests_are_retained_while_effective_membership_tracks_permission() {
    let mut runtime = DebugSubscriptionRuntime::default();
    let requested = DebugSubscriptionSet::from_raw_ids(&[0, 2]).unwrap();
    runtime.replace_requested(requested);

    let denied = runtime.rebuild_effective(DebugSubscriptionAuthorization::default());
    assert_eq!(denied.effective, DebugSubscriptionSet::empty());
    assert_eq!(denied.transition, DebugSynchronizerTransition::Unchanged);
    assert_eq!(runtime.requested(), requested);

    let admitted = runtime.rebuild_effective(DebugSubscriptionAuthorization {
        operator: true,
        ide_singleplayer_owner: false,
    });
    assert_eq!(admitted.effective, requested);
    assert_eq!(
        admitted.transition,
        DebugSynchronizerTransition::WakeAndSeed
    );

    let revoked = runtime.rebuild_effective(DebugSubscriptionAuthorization::default());
    assert_eq!(revoked.effective, DebugSubscriptionSet::empty());
    assert_eq!(
        revoked.transition,
        DebugSynchronizerTransition::SleepAndClear
    );
    assert_eq!(runtime.requested(), requested);
}

#[test]
fn c4_ide_singleplayer_owner_is_the_only_nonoperator_exception() {
    let mut runtime = DebugSubscriptionRuntime::default();
    let requested = DebugSubscriptionSet::from_raw_ids(&[15]).unwrap();
    runtime.replace_requested(requested);
    let rebuilt = runtime.rebuild_effective(DebugSubscriptionAuthorization {
        operator: false,
        ide_singleplayer_owner: true,
    });
    assert_eq!(rebuilt.effective, requested);
    assert_eq!(rebuilt.transition, DebugSynchronizerTransition::WakeAndSeed);
}

#[test]
fn c4_debug_subscription_membership_changes_without_an_acknowledgement() {
    let mut runtime = DebugSubscriptionRuntime::default();
    runtime.replace_requested(DebugSubscriptionSet::from_raw_ids(&[1]).unwrap());
    runtime.rebuild_effective(DebugSubscriptionAuthorization {
        operator: true,
        ide_singleplayer_owner: false,
    });
    runtime.replace_requested(DebugSubscriptionSet::from_raw_ids(&[2]).unwrap());
    let rebuilt = runtime.rebuild_effective(DebugSubscriptionAuthorization {
        operator: true,
        ide_singleplayer_owner: false,
    });
    assert_eq!(
        rebuilt.transition,
        DebugSynchronizerTransition::ReplaceMembership
    );
    assert!(rebuilt.effective.contains(DebugSubscription::Brains));
}

#[test]
fn c4_debug_subscription_is_discarded_with_the_old_player_object() {
    for event in [
        DebugSubscriptionLifecycleEvent::Disconnect,
        DebugSubscriptionLifecycleEvent::ReconfigurationRemoval,
    ] {
        let mut runtime = DebugSubscriptionRuntime::default();
        runtime.replace_requested(DebugSubscriptionSet::from_raw_ids(&[1]).unwrap());
        runtime.rebuild_effective(DebugSubscriptionAuthorization {
            operator: true,
            ide_singleplayer_owner: false,
        });
        assert_eq!(
            apply_lifecycle(&mut runtime, event),
            DebugSynchronizerTransition::SleepAndClear
        );
        assert!(runtime.requested().is_empty());
        assert!(runtime.effective().is_empty());
    }
}

#[test]
fn c4_debug_subscription_required_decoder_remains_fail_closed() {
    assert!(matches!(
        decode_required_packet(&[DebugSubscriptionRequestKind::Replace.wire_id() as u8]),
        Err(PlayServerboundEntryCodecError::UnsupportedPacketIdentity {
            identity: "minecraft:debug_subscription_request"
        })
    ));
}
