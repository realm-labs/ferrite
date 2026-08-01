use std::cell::Cell;
use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::clientbound::chat_presentation::codec::ChatPresentationCodecError;
use ferrite_protocol::java_26_2::play::clientbound::chat_presentation::packet::{
    BoundChatType, ChatDecoration, ChatParameter, ChatTypeHolder, DeleteChat, DirectChatType,
    DisguisedChat, FilterMask, MessageSignature, PackedMessageSignature, PlayerChat,
    SignedMessageBodyPacked, SystemChat,
};
use ferrite_protocol::java_26_2::play::clientbound::chat_presentation::projection::{
    ChatClientProjection, ChatPresentationAction, ChatPresentationPolicy, ChatProjectionError,
    ChatTrust, ChatVisibility, DisplayKind, LastSeenTracker, MessageSignatureCache,
    SenderChatState, ValidationEvidence,
};
use ferrite_protocol::java_26_2::play::clientbound::chat_presentation::publication::{
    AuthoredChat, ChatPublicationConnection, PublishedChatPacket, SystemRecipient,
    publish_player_chat, publish_system_chat,
};
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::{CHAT_TYPE, PlayRegistries};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::{NbtQuota, NetworkNbt, TextComponentNbt};

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn literal(value: &str) -> TextComponentNbt {
    TextComponentNbt::literal(value).unwrap()
}

fn empty_style() -> NetworkNbt {
    NetworkNbt::from_bytes(vec![10, 0], NbtQuota::Trusted).unwrap()
}

fn empty_decoration() -> ChatDecoration {
    ChatDecoration {
        translation_key: String::new(),
        parameters: Vec::new(),
        style: empty_style(),
    }
}

fn direct_bound() -> BoundChatType {
    BoundChatType {
        holder: ChatTypeHolder::Direct(Box::new(DirectChatType {
            chat: empty_decoration(),
            narration: empty_decoration(),
        })),
        name: literal(""),
        target: None,
    }
}

fn signature(byte: u8) -> MessageSignature {
    MessageSignature(Box::new([byte; 256]))
}

fn indexed_signature(value: u16) -> MessageSignature {
    let mut bytes = [0; 256];
    bytes[..2].copy_from_slice(&value.to_be_bytes());
    MessageSignature(Box::new(bytes))
}

fn minimal_player() -> PlayerChat {
    PlayerChat {
        global_index: 0,
        sender: 0,
        message_index: 0,
        signature: None,
        body: SignedMessageBodyPacked {
            content: String::new(),
            timestamp_ms: 0,
            salt: 0,
            last_seen: Vec::new(),
        },
        unsigned_content: None,
        filter_mask: FilterMask::Pass,
        chat_type: direct_bound(),
    }
}

fn policy(now_ms: i64, gui_tick: u64) -> ChatPresentationPolicy {
    ChatPresentationPolicy {
        now_ms,
        gui_tick,
        local_profile: 99,
        integrated_server: false,
        enforces_secure_chat: false,
        secure_only: false,
        visibility: ChatVisibility::Full,
        friends_only: false,
        local_receiver_allowed: true,
        delay_seconds: 0.0,
        paused: false,
    }
}

fn sender(session: bool) -> SenderChatState {
    SenderChatState {
        profile_name: "sender".to_owned(),
        session_id: session.then_some(11),
        session_expired: false,
        validator_poisoned: false,
        blocked: false,
        friend: true,
    }
}

fn assert_roundtrip(packet: PlayClientboundPacket, registries: &PlayRegistries) {
    let encoded = encode_packet(&packet, registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(registries)).unwrap(),
        packet
    );
}

#[test]
fn c3_gold_clientbound_chat_presentation_locks_all_four_packet_bodies() {
    let registries = PlayRegistries::default();
    let packets = [
        (
            PlayClientboundPacket::DeleteChat(DeleteChat {
                signature: PackedMessageSignature::CacheIndex(0),
            }),
            "1f01",
        ),
        (
            PlayClientboundPacket::DisguisedChat(DisguisedChat {
                message: literal(""),
                chat_type: direct_bound(),
            }),
            "210800000000000a0000000a0008000000",
        ),
        (
            PlayClientboundPacket::PlayerChat(Box::new(minimal_player())),
            "410000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000a0008000000",
        ),
        (
            PlayClientboundPacket::SystemChat(SystemChat {
                content: literal(""),
                overlay: false,
            }),
            "7908000000",
        ),
    ];
    for (packet, expected) in packets {
        assert_eq!(
            encode_packet(&packet, &registries).unwrap(),
            decode_hex(expected)
        );
    }
}

#[test]
fn c3_chat_presentation_requires_an_installed_play_level() {
    for packet in [
        PlayClientboundPacket::DeleteChat(DeleteChat {
            signature: PackedMessageSignature::CacheIndex(0),
        }),
        PlayClientboundPacket::SystemChat(SystemChat {
            content: literal("system"),
            overlay: false,
        }),
    ] {
        assert_eq!(
            PlayEntryProjection::default().apply(packet),
            Err(PlayProjectionError::LevelNotInstalled)
        );
    }
}

#[test]
fn c3_chat_codecs_roundtrip_packed_registered_optional_and_partial_forms() {
    let mut tables = BTreeMap::new();
    tables.insert(id(CHAT_TYPE), vec![id("minecraft:chat")]);
    let registries = PlayRegistries::new(tables);
    assert_roundtrip(
        PlayClientboundPacket::DeleteChat(DeleteChat {
            signature: PackedMessageSignature::Full(signature(0xab)),
        }),
        &registries,
    );
    let packet = PlayerChat {
        global_index: i32::MIN,
        sender: u128::MAX,
        message_index: i32::MAX,
        signature: Some(signature(1)),
        body: SignedMessageBodyPacked {
            content: "😀".repeat(128),
            timestamp_ms: i64::MIN,
            salt: i64::MAX,
            last_seen: vec![
                PackedMessageSignature::CacheIndex(-2),
                PackedMessageSignature::Full(signature(2)),
            ],
        },
        unsigned_content: Some(literal("unsigned")),
        filter_mask: FilterMask::PartiallyFiltered(vec![i64::MIN, i64::MAX]),
        chat_type: BoundChatType {
            holder: ChatTypeHolder::Registered(id("minecraft:chat")),
            name: literal("name"),
            target: Some(literal("target")),
        },
    };
    assert_roundtrip(
        PlayClientboundPacket::PlayerChat(Box::new(packet)),
        &registries,
    );
    assert_roundtrip(
        PlayClientboundPacket::SystemChat(SystemChat {
            content: literal("system"),
            overlay: true,
        }),
        &registries,
    );
}

#[test]
fn c3_chat_codec_normalizes_booleans_parameters_and_fails_strict_boundaries() {
    let registries = PlayRegistries::default();
    let mut system = encode_packet(
        &PlayClientboundPacket::SystemChat(SystemChat {
            content: literal(""),
            overlay: false,
        }),
        &registries,
    )
    .unwrap();
    *system.last_mut().unwrap() = 2;
    let PlayClientboundPacket::SystemChat(decoded) =
        decode_packet(&system, context(&registries)).unwrap()
    else {
        panic!("system chat expected");
    };
    assert!(decoded.overlay);

    let mut disguised = vec![0x21, 8, 0, 0, 0, 0, 1, 7, 10, 0];
    disguised.extend_from_slice(&[0, 0, 10, 0, 8, 0, 0, 0]);
    let PlayClientboundPacket::DisguisedChat(decoded) =
        decode_packet(&disguised, context(&registries)).unwrap()
    else {
        panic!("disguised chat expected");
    };
    let ChatTypeHolder::Direct(direct) = decoded.chat_type.holder else {
        panic!("direct chat type expected");
    };
    assert_eq!(direct.chat.parameters, vec![ChatParameter::Sender]);

    let mut invalid_filter = encode_packet(
        &PlayClientboundPacket::PlayerChat(Box::new(minimal_player())),
        &registries,
    )
    .unwrap();
    let filter_index = invalid_filter.len() - 14;
    invalid_filter[filter_index] = 3;
    assert!(matches!(
        decode_packet(&invalid_filter, context(&registries)),
        Err(PlayClientboundCodecError::ChatPresentation(
            ChatPresentationCodecError::UnknownFilterMask { ordinal: 3 }
        ))
    ));

    let delete = PlayClientboundPacket::DeleteChat(DeleteChat {
        signature: PackedMessageSignature::CacheIndex(0),
    });
    assert_eq!(delete.chat_skippable(), Some(false));
    assert_eq!(
        PlayClientboundPacket::PlayerChat(Box::new(minimal_player())).chat_skippable(),
        Some(true)
    );

    let mut too_many = minimal_player();
    too_many.body.last_seen = vec![PackedMessageSignature::CacheIndex(0); 21];
    assert!(
        encode_packet(
            &PlayClientboundPacket::PlayerChat(Box::new(too_many)),
            &registries,
        )
        .is_err()
    );
    assert!(decode_packet(&[0x21, 8, 0, 0, 1], context(&registries)).is_err());

    for malformed in [vec![0x1f, 0], vec![0x21], {
        let mut trailing = system;
        trailing.push(0);
        trailing
    }] {
        assert!(decode_packet(&malformed, context(&registries)).is_err());
    }
}

#[test]
fn c3_signature_cache_uses_old_snapshot_tail_order_dedup_and_index_faults() {
    let mut cache = MessageSignatureCache::default();
    assert_eq!(
        cache
            .unpack(&PackedMessageSignature::CacheIndex(0))
            .unwrap(),
        None
    );
    assert_eq!(
        cache.unpack(&PackedMessageSignature::CacheIndex(-1)),
        Err(ChatProjectionError::InvalidCacheIndex { index: -1 })
    );
    cache.push_batch([signature(3)]);
    cache.push_batch([signature(1), signature(2), signature(1)]);
    assert_eq!(cache.entries(), &[signature(2), signature(1), signature(3)]);
    assert_eq!(
        cache.pack(&signature(1)),
        PackedMessageSignature::CacheIndex(1)
    );
    assert!(matches!(
        cache.pack(&signature(9)),
        PackedMessageSignature::Full(_)
    ));

    let mut tracker = LastSeenTracker::default();
    for byte in 0..=64 {
        tracker.record(signature(byte), true);
    }
    assert!(tracker.acknowledgement_required());
    assert_eq!(tracker.entries().len(), 20);
    tracker.record(signature(64), false);
    assert_eq!(tracker.offset(), 65);

    let mut bounded = MessageSignatureCache::default();
    bounded.push_batch((0..129).map(indexed_signature));
    assert_eq!(bounded.entries().len(), 128);
    assert_eq!(bounded.entries()[0], indexed_signature(128));
    assert_eq!(bounded.entries()[127], indexed_signature(1));
}

#[test]
fn c3_player_chat_index_cache_sender_and_validator_prefixes_are_ordered() {
    let mut client = ChatClientProjection::default();
    client.install_sender(1, sender(true));
    let mut packet = minimal_player();
    packet.sender = 1;
    packet.signature = Some(signature(1));
    packet.body.content = "hello".to_owned();
    packet.body.timestamp_ms = 1_000;
    assert_eq!(
        client
            .apply_player(&packet, &policy(1_000, 0), ValidationEvidence::default())
            .unwrap()
            .action,
        ChatPresentationAction::Displayed
    );
    assert_eq!(client.cache.entries(), &[signature(1)]);

    packet.global_index = 2;
    packet.signature = Some(signature(2));
    assert_eq!(
        client.apply_player(&packet, &policy(1_000, 1), ValidationEvidence::default()),
        Err(ChatProjectionError::BadGlobalIndex {
            expected: 1,
            actual: 2,
        })
    );
    assert_eq!(client.next_global_index, 2);
    assert_eq!(client.cache.entries(), &[signature(1)]);

    packet.body.last_seen = vec![PackedMessageSignature::CacheIndex(0)];
    let invalid = ValidationEvidence {
        signature_valid: false,
        ..ValidationEvidence::default()
    };
    assert_eq!(
        client
            .apply_player(&packet, &policy(1_000, 2), invalid)
            .unwrap()
            .action,
        ChatPresentationAction::Displayed
    );
    assert_eq!(client.cache.entries(), &[signature(2), signature(1)]);
    assert!(client.senders.get(&1).unwrap().validator_poisoned);

    packet.global_index = 3;
    packet.signature = Some(signature(3));
    packet.body.last_seen.clear();
    let before = client.displayed.len();
    client
        .apply_player(&packet, &policy(1_000, 3), ValidationEvidence::default())
        .unwrap();
    assert_eq!(client.displayed.len(), before + 1);
    assert_eq!(
        client.displayed.last().unwrap().kind,
        DisplayKind::ValidationError
    );

    packet.global_index = 4;
    packet.signature = Some(signature(4));
    let mut delayed_error = policy(1_100, 4);
    delayed_error.delay_seconds = 1.0;
    assert_eq!(
        client
            .apply_player(&packet, &delayed_error, ValidationEvidence::default())
            .unwrap()
            .action,
        ChatPresentationAction::Queued
    );
    assert_eq!(
        client
            .apply_delete(
                &DeleteChat {
                    signature: PackedMessageSignature::CacheIndex(0),
                },
                4,
            )
            .unwrap()
            .action,
        ChatPresentationAction::Noop
    );
    assert_eq!(client.queued_len(), 1);
}

#[test]
fn c3_player_chat_unresolved_missing_sender_and_unsigned_fallback_keep_distinct_prefixes() {
    let mut client = ChatClientProjection::default();
    let mut packet = minimal_player();
    packet.sender = 1;
    packet.signature = Some(signature(1));
    packet.body.content = "message".to_owned();
    packet.body.timestamp_ms = 1_000;
    packet.body.last_seen = vec![PackedMessageSignature::CacheIndex(0)];
    assert_eq!(
        client.apply_player(&packet, &policy(1_000, 0), ValidationEvidence::default()),
        Err(ChatProjectionError::UnresolvedPackedSignature)
    );
    assert_eq!(client.next_global_index, 1);
    assert!(client.cache.entries().is_empty());

    packet.global_index = 1;
    packet.body.last_seen.clear();
    assert_eq!(
        client
            .apply_player(&packet, &policy(1_000, 1), ValidationEvidence::default())
            .unwrap()
            .action,
        ChatPresentationAction::Displayed
    );
    assert_eq!(client.cache.entries(), &[signature(1)]);
    assert_eq!(client.last_seen.entries().back(), Some(&None));

    client.install_sender(1, sender(false));
    packet.global_index = 2;
    packet.signature = Some(signature(2));
    client
        .apply_player(&packet, &policy(1_000, 2), ValidationEvidence::default())
        .unwrap();
    let displayed = client.displayed.last().unwrap();
    assert_eq!(displayed.kind, DisplayKind::Player);
    assert_eq!(displayed.trust, ChatTrust::NotSecure);
    assert!(displayed.signature.is_none());
    assert_eq!(client.last_seen.offset(), 1);

    packet.global_index = 3;
    packet.signature = Some(signature(3));
    let mut enforced = policy(1_000, 3);
    enforced.enforces_secure_chat = true;
    client
        .apply_player(&packet, &enforced, ValidationEvidence::default())
        .unwrap();
    assert_eq!(
        client.displayed.last().unwrap().kind,
        DisplayKind::ValidationError
    );
    assert_eq!(client.last_seen.entries().back(), Some(&None));
}

#[test]
fn c3_player_chat_trust_secure_only_social_filter_and_partial_mask_are_exact() {
    let mut client = ChatClientProjection::default();
    client.install_sender(1, sender(true));
    let mut packet = minimal_player();
    packet.sender = 1;
    packet.signature = Some(signature(1));
    packet.body.content = "abcd".to_owned();
    packet.body.timestamp_ms = 1_000;
    packet.unsigned_content = Some(literal("unsigned"));
    packet.filter_mask = FilterMask::PartiallyFiltered(vec![0b1010]);
    let modified = ValidationEvidence {
        decorated_contains_signed: false,
        unsigned_uses_default_font: false,
        ..ValidationEvidence::default()
    };
    let mut secure_only = policy(1_000, 0);
    secure_only.secure_only = true;
    let outcome = client
        .apply_player(&packet, &secure_only, modified)
        .unwrap();
    assert_eq!(outcome.action, ChatPresentationAction::Displayed);
    let displayed = client.displayed.last().unwrap();
    assert_eq!(displayed.content, "a#c#");
    assert_eq!(displayed.trust, ChatTrust::Modified);
    assert!(!displayed.used_unsigned_content);

    packet.global_index = 1;
    packet.signature = Some(signature(2));
    packet.filter_mask = FilterMask::FullyFiltered;
    assert_eq!(
        client
            .apply_player(&packet, &policy(1_000, 1), ValidationEvidence::default(),)
            .unwrap()
            .action,
        ChatPresentationAction::Suppressed
    );
    assert_eq!(client.last_seen.entries().back(), Some(&None));

    packet.global_index = 2;
    packet.signature = Some(signature(3));
    packet.filter_mask = FilterMask::PartiallyFiltered(vec![1 << 9]);
    assert_eq!(
        client.apply_player(&packet, &policy(1_000, 2), ValidationEvidence::default()),
        Err(ChatProjectionError::FilterPositionOutOfRange {
            position: 9,
            length: 4,
        })
    );
    assert_eq!(client.cache.entries()[0], signature(3));
}

#[test]
fn c3_chat_delay_last_seen_and_deletion_follow_locked_timing_and_prefixes() {
    let mut client = ChatClientProjection::default();
    client.install_sender(1, sender(true));
    let mut first = minimal_player();
    first.sender = 1;
    first.signature = Some(signature(1));
    first.body.content = "first".to_owned();
    first.body.timestamp_ms = 1_000;
    client
        .apply_player(&first, &policy(1_000, 0), ValidationEvidence::default())
        .unwrap();
    assert_eq!(client.last_seen.offset(), 1);
    assert_eq!(
        client
            .apply_delete(
                &DeleteChat {
                    signature: PackedMessageSignature::CacheIndex(0),
                },
                30,
            )
            .unwrap()
            .action,
        ChatPresentationAction::DeleteScheduled
    );
    assert!(client.tick(1_100, 59, false).is_empty());
    assert_eq!(
        client.tick(1_100, 60, false)[0].action,
        ChatPresentationAction::Deleted
    );
    assert_eq!(client.displayed[0].kind, DisplayKind::DeletedMarker);

    let mut second = first;
    second.global_index = 1;
    second.signature = Some(signature(2));
    second.body.content = "second".to_owned();
    let mut delayed = policy(1_100, 61);
    delayed.delay_seconds = 1.0;
    assert_eq!(
        client
            .apply_player(&second, &delayed, ValidationEvidence::default())
            .unwrap()
            .action,
        ChatPresentationAction::Queued
    );
    assert_eq!(client.queued_len(), 1);
    assert_eq!(
        client
            .apply_delete(
                &DeleteChat {
                    signature: PackedMessageSignature::CacheIndex(0),
                },
                61,
            )
            .unwrap()
            .action,
        ChatPresentationAction::Deleted
    );
    assert_eq!(client.queued_len(), 0);
}

#[test]
fn c3_chat_delay_drains_suppressed_until_shown_and_zero_flushes_the_whole_queue() {
    let mut client = ChatClientProjection::default();
    client.install_sender(1, sender(true));
    let mut packet = minimal_player();
    packet.sender = 1;
    packet.signature = Some(signature(1));
    packet.body.timestamp_ms = 1_000;
    client
        .apply_player(&packet, &policy(1_000, 0), ValidationEvidence::default())
        .unwrap();

    let mut delayed = policy(1_100, 1);
    delayed.delay_seconds = 1.0;
    packet.global_index = 1;
    packet.signature = Some(signature(2));
    packet.filter_mask = FilterMask::FullyFiltered;
    client
        .apply_player(&packet, &delayed, ValidationEvidence::default())
        .unwrap();
    packet.global_index = 2;
    packet.signature = Some(signature(3));
    packet.filter_mask = FilterMask::Pass;
    client
        .apply_player(&packet, &delayed, ValidationEvidence::default())
        .unwrap();
    assert!(client.tick(1_999, 2, false).is_empty());
    let drained = client.tick(2_000, 2, false);
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].action, ChatPresentationAction::Suppressed);
    assert_eq!(drained[1].action, ChatPresentationAction::Displayed);

    delayed.now_ms = 2_100;
    for (global_index, byte) in [(3, 4), (4, 5)] {
        packet.global_index = global_index;
        packet.signature = Some(signature(byte));
        client
            .apply_player(&packet, &delayed, ValidationEvidence::default())
            .unwrap();
    }
    assert_eq!(client.queued_len(), 2);
    assert_eq!(client.set_delay_seconds(0.0, 2_100, 3, false).len(), 2);
    assert_eq!(client.queued_len(), 0);

    packet.global_index = 5;
    packet.signature = Some(signature(6));
    delayed.now_ms = 100;
    client
        .apply_player(&packet, &delayed, ValidationEvidence::default())
        .unwrap();
    assert!(client.tick(1_000, 4, true).is_empty());
    assert_eq!(client.tick(1_050, 5, false).len(), 1);
}

#[test]
fn c3_disguised_and_system_chat_keep_delay_visibility_social_and_overlay_paths_separate() {
    let mut client = ChatClientProjection::default();
    let mut blocked = sender(false);
    blocked.blocked = true;
    client.install_sender(1, blocked);
    let mut delayed = policy(100, 0);
    delayed.delay_seconds = 1.0;
    assert_eq!(
        client
            .apply_disguised(
                &DisguisedChat {
                    message: literal("message"),
                    chat_type: direct_bound(),
                },
                &delayed,
            )
            .action,
        ChatPresentationAction::Queued
    );
    assert_eq!(
        client.tick(1_000, 20, false)[0].action,
        ChatPresentationAction::Displayed
    );

    let mut hidden = policy(1_001, 21);
    hidden.visibility = ChatVisibility::Hidden;
    assert_eq!(
        client
            .apply_system(
                &SystemChat {
                    content: literal("overlay"),
                    overlay: true,
                },
                "overlay",
                &hidden,
            )
            .action,
        ChatPresentationAction::Displayed
    );
    hidden.visibility = ChatVisibility::Full;
    assert_eq!(
        client
            .apply_system(
                &SystemChat {
                    content: literal("<sender> blocked"),
                    overlay: false,
                },
                "<sender> blocked",
                &hidden,
            )
            .action,
        ChatPresentationAction::Suppressed
    );
    client.remove_sender(1);
    assert_eq!(
        client
            .apply_system(
                &SystemChat {
                    content: literal("<sender> still blocked"),
                    overlay: false,
                },
                "<sender> still blocked",
                &hidden,
            )
            .action,
        ChatPresentationAction::Suppressed
    );
}

#[test]
fn c3_chat_publication_is_per_connection_visibility_filter_index_cache_and_pending_ordered() {
    let authored = AuthoredChat {
        sender: 1,
        message_index: 7,
        signature: Some(signature(1)),
        body_content: "message".to_owned(),
        timestamp_ms: 10,
        salt: 11,
        last_seen: vec![signature(9)],
        unsigned_content: None,
        filter_mask: FilterMask::PartiallyFiltered(vec![1]),
        decorated: literal("message"),
        chat_type: direct_bound(),
    };
    let mut recipients = BTreeMap::from([
        (
            1,
            ChatPublicationConnection {
                visibility: ChatVisibility::Full,
                filters_message: false,
                next_global_index: 4,
                cache: MessageSignatureCache::default(),
                pending_signatures: 0,
            },
        ),
        (
            2,
            ChatPublicationConnection {
                visibility: ChatVisibility::Full,
                filters_message: true,
                next_global_index: 9,
                cache: MessageSignatureCache::default(),
                pending_signatures: 4_096,
            },
        ),
        (
            3,
            ChatPublicationConnection {
                visibility: ChatVisibility::System,
                filters_message: false,
                next_global_index: 0,
                cache: MessageSignatureCache::default(),
                pending_signatures: 0,
            },
        ),
    ]);
    let result = publish_player_chat(&authored, &mut recipients);
    assert_eq!(result.deliveries.len(), 2);
    let PublishedChatPacket::Player(first) = &result.deliveries[0].packet else {
        panic!("player chat expected");
    };
    assert_eq!(first.global_index, 4);
    assert_eq!(first.filter_mask, FilterMask::Pass);
    let PublishedChatPacket::Player(second) = &result.deliveries[1].packet else {
        panic!("player chat expected");
    };
    assert_eq!(second.global_index, 9);
    assert!(matches!(
        second.filter_mask,
        FilterMask::PartiallyFiltered(_)
    ));
    assert!(result.deliveries[1].disconnect_after_send);
    assert_eq!(recipients.get(&1).unwrap().cache.entries(), &[signature(1)]);

    let mut fully_filtered = authored;
    fully_filtered.filter_mask = FilterMask::FullyFiltered;
    let result = publish_player_chat(&fully_filtered, &mut recipients);
    assert_eq!(result.deliveries.len(), 1);
    assert!(result.notify_sender_fully_filtered);
}

#[test]
fn c3_system_publication_retries_visible_nonoverlay_with_bounded_fallback_only() {
    let recipients = BTreeMap::from([
        (
            1,
            SystemRecipient {
                visibility: ChatVisibility::Full,
                send_succeeds: false,
            },
        ),
        (
            2,
            SystemRecipient {
                visibility: ChatVisibility::Hidden,
                send_succeeds: false,
            },
        ),
    ]);
    let fallback_length = Cell::new(0);
    let long = "x".repeat(300);
    let deliveries =
        publish_system_chat(&literal("message"), &long, false, &recipients, |preview| {
            fallback_length.set(preview.chars().count());
            literal("fallback")
        });
    assert_eq!(deliveries.len(), 1);
    assert!(deliveries[0].fallback);
    assert_eq!(fallback_length.get(), 256);
    assert!(
        publish_system_chat(&literal("overlay"), "overlay", true, &recipients, |_| {
            literal("unused")
        })
        .is_empty()
    );
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}
