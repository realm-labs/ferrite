use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::completion::codec::CompletionCodecError;
use ferrite_protocol::java_26_2::play::clientbound::completion::packet::{
    CommandSuggestions, CustomChatCompletions, CustomCompletionAction, SuggestionEntry,
};
use ferrite_protocol::java_26_2::play::clientbound::completion::projection::{
    CommandCompletionOutcome, CompletionClientProjection, CompletionProjectionError,
    CompletionUiError,
};
use ferrite_protocol::java_26_2::play::clientbound::completion::publication::{
    parse_completion_request, publish_completion,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;
use ferrite_protocol::java_26_2::wire::error::WireError;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn literal(value: &str) -> TextComponentNbt {
    TextComponentNbt::literal(value).unwrap()
}

fn suggestion(text: impl Into<String>) -> SuggestionEntry {
    SuggestionEntry {
        text: text.into(),
        tooltip: None,
    }
}

fn command(transaction: i32, start: i32, length: i32) -> CommandSuggestions {
    CommandSuggestions {
        transaction,
        start,
        length,
        entries: vec![suggestion("entry")],
    }
}

fn assert_roundtrip(packet: PlayClientboundPacket) {
    let registries = PlayRegistries::default();
    let body = encode_packet(&packet, &registries).unwrap();
    assert_eq!(decode_packet(&body, context(&registries)).unwrap(), packet);
}

#[test]
fn c3_gold_clientbound_completions_locks_both_packet_bodies() {
    let registries = PlayRegistries::default();
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::CommandSuggestions(Box::new(CommandSuggestions {
                transaction: 0,
                start: 0,
                length: 0,
                entries: Vec::new(),
            })),
            &registries,
        )
        .unwrap(),
        [0x0f, 0, 0, 0, 0]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::CustomChatCompletions(Box::new(CustomChatCompletions {
                action: CustomCompletionAction::Add,
                entries: Vec::new(),
            })),
            &registries,
        )
        .unwrap(),
        [0x17, 0, 0]
    );
}

#[test]
fn c3_completion_codecs_preserve_signed_ranges_entries_tooltips_and_actions() {
    assert_roundtrip(PlayClientboundPacket::CommandSuggestions(Box::new(
        CommandSuggestions {
            transaction: i32::MIN,
            start: i32::MAX,
            length: 1,
            entries: vec![
                SuggestionEntry {
                    text: "😀".repeat(16_383),
                    tooltip: Some(literal("tooltip")),
                },
                suggestion("second"),
            ],
        },
    )));
    for action in [
        CustomCompletionAction::Add,
        CustomCompletionAction::Remove,
        CustomCompletionAction::Set,
    ] {
        assert_roundtrip(PlayClientboundPacket::CustomChatCompletions(Box::new(
            CustomChatCompletions {
                action,
                entries: vec!["duplicate".to_owned(), "duplicate".to_owned()],
            },
        )));
    }

    let registries = PlayRegistries::default();
    let noncanonical_tooltip_boolean = [0x0f, 0, 0, 0, 1, 0, 2, 8, 0, 0];
    let decoded = decode_packet(&noncanonical_tooltip_boolean, context(&registries)).unwrap();
    assert_eq!(
        encode_packet(&decoded, &registries).unwrap(),
        [0x0f, 0, 0, 0, 1, 0, 1, 8, 0, 0]
    );
}

#[test]
fn c3_completion_codecs_fault_strict_actions_counts_strings_components_and_framing() {
    let registries = PlayRegistries::default();
    assert_eq!(
        decode_packet(&[0x17, 3, 0], context(&registries)),
        Err(PlayClientboundCodecError::Completion(
            CompletionCodecError::UnknownAction { ordinal: 3 }
        ))
    );
    assert_eq!(
        decode_packet(
            &[0x0f, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x0f],
            context(&registries),
        ),
        Err(PlayClientboundCodecError::Completion(
            CompletionCodecError::Wire(WireError::NegativeLength {
                field: "command suggestions",
                value: -1,
            })
        ))
    );
    assert!(decode_packet(&[0x0f, 0, 0, 0, 1, 0, 1, 0], context(&registries)).is_err());
    assert!(decode_packet(&[0x17, 0, 0, 0], context(&registries)).is_err());

    let overlong = PlayClientboundPacket::CustomChatCompletions(Box::new(CustomChatCompletions {
        action: CustomCompletionAction::Add,
        entries: vec!["x".repeat(32_768)],
    }));
    assert!(matches!(
        encode_packet(&overlong, &registries),
        Err(PlayClientboundCodecError::Completion(
            CompletionCodecError::Wire(WireError::UtfCodeUnitLimit { .. })
        ))
    ));
}

#[test]
fn c3_command_completion_converts_before_latest_correlation_and_reproduces_sentinel_fault() {
    let mut client = CompletionClientProjection::default();
    let first = client.begin_request("first");
    let second = client.begin_request("second");
    assert_eq!(first.transaction, 0);
    assert_eq!(second.transaction, 1);
    assert!(!first.canceled_previous);
    assert!(second.canceled_previous);

    let stale = client
        .apply_command(&command(first.transaction, i32::MAX, 1))
        .unwrap();
    let CommandCompletionOutcome::IgnoredStale(converted) = stale else {
        panic!("old transaction must be stale");
    };
    assert_eq!(converted.range.end, i32::MIN);
    assert!(converted.validate_for_input("input").is_err());
    assert!(client.has_pending_future());

    assert!(matches!(
        client.apply_command(&command(second.transaction, 0, 0)),
        Ok(CommandCompletionOutcome::Completed(_))
    ));
    assert_eq!(client.pending_transaction(), -1);
    assert!(!client.has_pending_future());
    assert!(matches!(
        client.apply_command(&command(second.transaction, 0, 0)),
        Ok(CommandCompletionOutcome::IgnoredStale(_))
    ));
    assert_eq!(
        client.apply_command(&command(-1, 0, 0)),
        Err(CompletionProjectionError::MissingPendingFuture)
    );
}

#[test]
fn c3_command_request_ids_wrap_and_a_live_negative_one_future_is_valid() {
    let mut wrapping = CompletionClientProjection::with_transaction_counter(i32::MAX);
    assert_eq!(wrapping.begin_request("wrapped").transaction, i32::MIN);

    let mut negative_one = CompletionClientProjection::with_transaction_counter(-2);
    let request = negative_one.begin_request("sentinel as live ID");
    assert_eq!(request.transaction, -1);
    assert!(matches!(
        negative_one.apply_command(&command(-1, 0, 0)),
        Ok(CommandCompletionOutcome::Completed(_))
    ));
}

#[test]
fn c3_command_completion_defers_range_failures_until_ui_application() {
    let mut client = CompletionClientProjection::default();
    let request = client.begin_request("😀");
    let CommandCompletionOutcome::Completed(result) = client
        .apply_command(&command(request.transaction, 0, 2))
        .unwrap()
    else {
        panic!("current transaction must complete");
    };
    result.validate_for_input("😀").unwrap();
    assert_eq!(
        result.validate_for_input("x"),
        Err(CompletionUiError::InvalidRange {
            start: 0,
            end: 2,
            input_length: 1,
        })
    );
}

#[test]
fn c3_custom_completions_are_receive_ordered_sets_union_current_player_names() {
    let mut client = CompletionClientProjection::default();
    let pending = client.begin_request("command").transaction;
    client.apply_custom(&CustomChatCompletions {
        action: CustomCompletionAction::Add,
        entries: vec!["custom".to_owned(), "custom".to_owned()],
    });
    assert_eq!(
        client.chat_candidates(["alice", "custom"]),
        BTreeSet::from(["alice".to_owned(), "custom".to_owned(),])
    );
    client.apply_custom(&CustomChatCompletions {
        action: CustomCompletionAction::Remove,
        entries: vec!["missing".to_owned(), "custom".to_owned()],
    });
    assert!(client.custom_entries().is_empty());
    assert_eq!(
        client.chat_candidates(["alice"]),
        BTreeSet::from(["alice".to_owned()])
    );
    client.apply_custom(&CustomChatCompletions {
        action: CustomCompletionAction::Set,
        entries: vec!["replacement".to_owned(), "replacement".to_owned()],
    });
    assert_eq!(
        client.custom_entries(),
        &BTreeSet::from(["replacement".to_owned()])
    );
    client.apply_custom(&CustomChatCompletions {
        action: CustomCompletionAction::Set,
        entries: Vec::new(),
    });
    assert_eq!(
        client.chat_candidates(["alice"]),
        BTreeSet::from(["alice".to_owned()])
    );
    assert_eq!(client.pending_transaction(), pending);
    assert!(client.has_pending_future());
}

#[test]
fn c3_command_completion_publication_strips_once_caps_and_allows_reordered_futures() {
    let mut client = CompletionClientProjection::default();
    let first = client.begin_request("/first");
    let second = client.begin_request("//second");
    let parsed_first = parse_completion_request(&first, str::to_owned);
    let parsed_second = parse_completion_request(&second, str::to_owned);
    assert_eq!(parsed_first.parsed, "first");
    assert_eq!(parsed_second.parsed, "/second");

    let second_response = publish_completion(
        &parsed_second,
        -10,
        i32::MAX,
        (0..1_001).map(|index| suggestion(index.to_string())),
    );
    let first_response = publish_completion(&parsed_first, 2, 3, [suggestion("first")]);
    assert_eq!(second_response.transaction, second.transaction);
    assert_eq!(second_response.entries.len(), 1_000);
    assert_eq!(second_response.entries[999].text, "999");
    assert_eq!(first_response.transaction, first.transaction);
    assert_eq!(first_response.entries[0].text, "first");
}

#[test]
fn c3_completion_projection_requires_an_installed_play_level() {
    for packet in [
        PlayClientboundPacket::CommandSuggestions(Box::new(command(0, 0, 0))),
        PlayClientboundPacket::CustomChatCompletions(Box::new(CustomChatCompletions {
            action: CustomCompletionAction::Set,
            entries: Vec::new(),
        })),
    ] {
        assert_eq!(
            PlayEntryProjection::default().apply(packet),
            Err(PlayProjectionError::LevelNotInstalled)
        );
    }
}
