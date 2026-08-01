use ferrite_protocol::java_26_2::play::clientbound::chat_presentation::packet::MessageSignature;
use ferrite_protocol::java_26_2::play::serverbound::chat::admission::{
    ChatAdmission, ChatFilterPolicy, ChatFutureChain, ChatVisibility, NormalizedPlayerChat,
    TickThrottler, admit_chat, admit_signed_command, admit_unsigned_command, complete_suggestions,
    has_illegal_chat_character, normalize_suggestion, truncate_suggestions,
};
use ferrite_protocol::java_26_2::play::serverbound::chat::last_seen::{
    LAST_SEEN_WINDOW, LastSeenError, LastSeenTracker, LastSeenValidator, PendingResult, checksum,
};
use ferrite_protocol::java_26_2::play::serverbound::chat::packet::{
    ArgumentSignature, ChatAck, ChatCommand, ChatCommandSigned, ChatMessage, ChatSessionUpdate,
    CommandSuggestion, LastSeenUpdate, ProfilePublicKeyData,
};
use ferrite_protocol::java_26_2::play::serverbound::chat::session::{
    ChatSessionAction, ChatSessionState, ProfileKeyValidation, profile_key_signed_payload,
    verify_sha256_rsa,
};
use ferrite_protocol::java_26_2::play::serverbound::chat::signing::{
    MessageDecoder, SignableArgument, SignedArgumentError, SignedDecodeError, SignedMessageBody,
    SignedMessageLink, collect_signed_arguments, signed_payload, unsigned_command_allowed,
};
use ferrite_protocol::java_26_2::play::serverbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use rsa::pkcs8::EncodePublicKey;
use rsa::{BigUint, RsaPublicKey};

fn signature(byte: u8) -> MessageSignature {
    MessageSignature(Box::new([byte; 256]))
}

fn update(offset: i32, acknowledged: [u8; 3], checksum: i8) -> LastSeenUpdate {
    LastSeenUpdate {
        offset,
        acknowledged,
        checksum,
    }
}

fn signed_command() -> ChatCommandSigned {
    ChatCommandSigned {
        command: "say hi".to_owned(),
        timestamp_millis: 1_234,
        salt: -5,
        argument_signatures: Vec::new(),
        last_seen: update(0, [0; 3], 0),
    }
}

fn chat_message(message: &str) -> ChatMessage {
    ChatMessage {
        message: message.to_owned(),
        timestamp_millis: 1_234,
        salt: -5,
        signature: None,
        last_seen: update(0, [0; 3], 0),
    }
}

fn profile_key(expiry: i64, key: &[u8], signature: &[u8]) -> ProfilePublicKeyData {
    ProfilePublicKeyData {
        expires_at_millis: expiry,
        public_key: key.to_vec(),
        key_signature: signature.to_vec(),
    }
}

fn valid_public_key() -> Vec<u8> {
    let modulus = BigUint::from_bytes_be(&[0xff; 128]);
    let exponent = BigUint::from(65_537_u32);
    RsaPublicKey::new(modulus, exponent)
        .unwrap()
        .to_public_key_der()
        .unwrap()
        .as_bytes()
        .to_vec()
}

fn encode_var_i32(mut value: i32) -> Vec<u8> {
    let mut output = Vec::new();
    loop {
        if value & !0x7f == 0 {
            output.push(value as u8);
            return output;
        }
        output.push((value as u8 & 0x7f) | 0x80);
        value = ((value as u32) >> 7) as i32;
    }
}

#[test]
fn c3_gold_serverbound_chat_locks_all_six_minimal_frames() {
    let mut signed = vec![8, 0];
    signed.extend([0; 16]);
    signed.extend([0; 6]);
    let mut chat = vec![9, 0];
    chat.extend([0; 16]);
    chat.extend([0; 6]);
    let public_key = valid_public_key();
    let mut session = vec![10];
    session.extend([0; 24]);
    session.extend(encode_var_i32(public_key.len() as i32));
    session.extend(&public_key);
    session.push(0);
    let cases = [
        (
            PlayServerboundEntryPacket::ChatAck(ChatAck { offset: 0 }),
            vec![6, 0],
        ),
        (
            PlayServerboundEntryPacket::ChatCommand(ChatCommand {
                command: String::new(),
            }),
            vec![7, 0],
        ),
        (
            PlayServerboundEntryPacket::ChatCommandSigned(ChatCommandSigned {
                command: String::new(),
                timestamp_millis: 0,
                salt: 0,
                argument_signatures: Vec::new(),
                last_seen: update(0, [0; 3], 0),
            }),
            signed,
        ),
        (
            PlayServerboundEntryPacket::ChatMessage(ChatMessage {
                message: String::new(),
                timestamp_millis: 0,
                salt: 0,
                signature: None,
                last_seen: update(0, [0; 3], 0),
            }),
            chat,
        ),
        (
            PlayServerboundEntryPacket::ChatSessionUpdate(ChatSessionUpdate {
                session_id: 0,
                profile_key: profile_key(0, &public_key, &[]),
            }),
            session,
        ),
        (
            PlayServerboundEntryPacket::CommandSuggestion(CommandSuggestion {
                transaction_id: 0,
                input: String::new(),
            }),
            vec![15, 0, 0],
        ),
    ];
    for (packet, expected) in cases {
        assert_eq!(encode_packet(packet.clone()).unwrap(), expected);
        assert_eq!(decode_packet(&expected).unwrap(), packet);
    }
}

#[test]
fn c3_chat_codecs_preserve_signed_fields_signatures_bitsets_and_nonzero_presence() {
    let packet = PlayServerboundEntryPacket::ChatCommandSigned(ChatCommandSigned {
        command: "execute as 😀".to_owned(),
        timestamp_millis: i64::MIN,
        salt: i64::MAX,
        argument_signatures: vec![
            ArgumentSignature {
                name: "first".to_owned(),
                signature: signature(1),
            },
            ArgumentSignature {
                name: "first".to_owned(),
                signature: signature(2),
            },
        ],
        last_seen: update(i32::MIN, [0xff, 0xff, 0xff], -128),
    });
    let encoded = encode_packet(packet.clone()).unwrap();
    assert_eq!(decode_packet(&encoded).unwrap(), packet);

    let packet = PlayServerboundEntryPacket::ChatMessage(ChatMessage {
        message: "hello".to_owned(),
        timestamp_millis: i64::MAX,
        salt: i64::MIN,
        signature: Some(signature(0xab)),
        last_seen: update(i32::MAX, [1, 2, 3], 127),
    });
    let mut encoded = encode_packet(packet.clone()).unwrap();
    let presence = 1 + 1 + "hello".len() + 8 + 8;
    encoded[presence] = 0xfe;
    assert_eq!(decode_packet(&encoded).unwrap(), packet);

    let session = PlayServerboundEntryPacket::ChatSessionUpdate(ChatSessionUpdate {
        session_id: u128::MAX,
        profile_key: profile_key(i64::MIN, &valid_public_key(), &[2; 4_096]),
    });
    assert_eq!(
        decode_packet(&encode_packet(session.clone()).unwrap()).unwrap(),
        session
    );
    let suggestion = PlayServerboundEntryPacket::CommandSuggestion(CommandSuggestion {
        transaction_id: i32::MIN,
        input: "x".repeat(32_500),
    });
    assert_eq!(
        decode_packet(&encode_packet(suggestion.clone()).unwrap()).unwrap(),
        suggestion
    );
}

#[test]
fn c3_chat_codecs_reject_every_count_string_fixed_field_and_residual_boundary() {
    let mut nine_arguments = vec![8, 0];
    nine_arguments.extend([0; 16]);
    nine_arguments.push(9);
    assert!(decode_packet(&nine_arguments).is_err());

    assert!(
        encode_packet(PlayServerboundEntryPacket::ChatMessage(chat_message(
            &"x".repeat(257)
        )))
        .is_err()
    );
    assert!(
        encode_packet(PlayServerboundEntryPacket::CommandSuggestion(
            CommandSuggestion {
                transaction_id: 0,
                input: "x".repeat(32_501),
            }
        ))
        .is_err()
    );
    assert!(
        encode_packet(PlayServerboundEntryPacket::ChatSessionUpdate(
            ChatSessionUpdate {
                session_id: 0,
                profile_key: profile_key(0, &[0; 513], &[]),
            }
        ))
        .is_err()
    );
    assert!(
        encode_packet(PlayServerboundEntryPacket::ChatSessionUpdate(
            ChatSessionUpdate {
                session_id: 0,
                profile_key: profile_key(0, &[1, 2], &[]),
            }
        ))
        .is_err()
    );
    let mut invalid_der = vec![10];
    invalid_der.extend([0; 24]);
    invalid_der.extend([2, 1, 2, 0]);
    assert!(decode_packet(&invalid_der).is_err());
    assert!(!verify_sha256_rsa(&[1, 2], b"payload", &[0; 256]));

    let complete =
        encode_packet(PlayServerboundEntryPacket::ChatMessage(chat_message("ok"))).unwrap();
    for length in 1..complete.len() {
        assert!(decode_packet(&complete[..length]).is_err());
    }
    let mut trailing = complete;
    trailing.push(0);
    assert!(decode_packet(&trailing).is_err());
}

#[test]
fn c3_last_seen_window_applies_offset_bits_and_checksum_in_slot_order() {
    let first = signature(1);
    let second = signature(2);
    let mut validator = LastSeenValidator::default();
    assert_eq!(validator.tracked_len(), LAST_SEEN_WINDOW);
    assert_eq!(
        validator.add_pending(first.clone()),
        Ok(PendingResult::Added)
    );
    assert_eq!(
        validator.add_pending(first.clone()),
        Ok(PendingResult::ConsecutiveDuplicate)
    );
    assert_eq!(
        validator.add_pending(second.clone()),
        Ok(PendingResult::Added)
    );
    let acknowledged = [first.clone(), second.clone()];
    assert_eq!(
        validator.apply_update(update(2, [0, 0, 0x0c], checksum(&acknowledged))),
        Ok(acknowledged.to_vec())
    );
    assert_eq!((validator.tracked_len(), validator.pending_len()), (20, 0));
}

#[test]
fn c3_last_seen_validation_is_nontransactional_after_offset_slots_and_checksum() {
    let first = signature(3);
    let second = signature(4);
    let mut validator = LastSeenValidator::default();
    validator.add_pending(first.clone()).unwrap();
    validator.add_pending(second).unwrap();
    assert!(matches!(
        validator.apply_update(update(2, [0, 0, 0x04], 99)),
        Err(LastSeenError::Checksum { .. })
    ));
    assert_eq!((validator.tracked_len(), validator.pending_len()), (20, 0));
    assert_eq!(
        validator.apply_update(update(0, [0; 3], 0)),
        Err(LastSeenError::ClearedAcknowledged { slot: 18 })
    );

    let mut upper = LastSeenValidator::default();
    upper.add_pending(signature(5)).unwrap();
    assert_eq!(
        upper.apply_update(update(1, [0, 0, 0x10], 0)),
        Err(LastSeenError::BitsOutsideWindow)
    );
    assert_eq!(upper.tracked_len(), 20);
}

#[test]
fn c3_last_seen_ack_rejects_invalid_offsets_without_removal() {
    let mut validator = LastSeenValidator::default();
    validator.add_pending(signature(1)).unwrap();
    assert_eq!(
        validator.apply_ack(-1),
        Err(LastSeenError::NegativeOffset(-1))
    );
    assert!(matches!(
        validator.apply_ack(2),
        Err(LastSeenError::OffsetTooLarge { maximum: 1, .. })
    ));
    assert_eq!(validator.tracked_len(), 21);
    assert_eq!(validator.apply_ack(1), Ok(()));
    assert_eq!(validator.tracked_len(), 20);
}

#[test]
fn c3_client_last_seen_tracker_advances_distinct_messages_and_clears_offsets() {
    let mut tracker = LastSeenTracker::default();
    assert_eq!(
        tracker.add_processed(signature(1), true),
        PendingResult::Added
    );
    assert_eq!(
        tracker.add_processed(signature(1), false),
        PendingResult::ConsecutiveDuplicate
    );
    assert_eq!(
        tracker.add_processed(signature(2), false),
        PendingResult::Added
    );
    let update = tracker.generate_update();
    assert_eq!((update.offset, update.acknowledged), (2, [0, 0, 0x04]));
    assert_eq!(update.checksum, checksum(&[signature(1)]));

    for byte in 3..=67 {
        tracker.add_processed(signature(byte), true);
    }
    assert_eq!(tracker.take_ack_if_due(), Some(65));
    assert_eq!(tracker.take_ack_if_due(), None);
}

#[test]
fn c3_signature_payload_locks_uuid_index_epoch_seconds_content_and_last_seen_order() {
    let link = SignedMessageLink {
        sender: 0x0102_0304_0506_0708_1112_1314_1516_1718,
        session: 0x2122_2324_2526_2728_3132_3334_3536_3738,
        index: -7,
    };
    let body = SignedMessageBody {
        content: "hi".to_owned(),
        timestamp_millis: -1,
        salt: i64::MIN,
        last_seen: vec![signature(0xab)],
    };
    let payload = signed_payload(link, &body);
    assert_eq!(&payload[0..4], &1_i32.to_be_bytes());
    assert_eq!(&payload[4..20], &link.sender.to_be_bytes());
    assert_eq!(&payload[20..36], &link.session.to_be_bytes());
    assert_eq!(&payload[36..40], &(-7_i32).to_be_bytes());
    assert_eq!(&payload[40..48], &i64::MIN.to_be_bytes());
    assert_eq!(&payload[48..56], &(-1_i64).to_be_bytes());
    assert_eq!(&payload[56..60], &2_i32.to_be_bytes());
    assert_eq!(&payload[60..62], b"hi");
    assert_eq!(&payload[62..66], &1_i32.to_be_bytes());
    assert_eq!(&payload[66..], &[0xab; 256]);
}

#[test]
fn c3_message_decoder_preserves_missing_expired_breaking_and_equal_timestamp_rules() {
    let body = SignedMessageBody {
        content: "hello".to_owned(),
        timestamp_millis: 1_000,
        salt: 2,
        last_seen: Vec::new(),
    };
    assert_eq!(
        MessageDecoder::unsigned(true).decode(&body, None, 0, |_, _| true),
        Err(SignedDecodeError::MissingProfileKey)
    );
    assert!(
        !MessageDecoder::unsigned(false)
            .decode(&body, Some(&signature(1)), 0, |_, _| false)
            .unwrap()
            .signed
    );

    let mut decoder = MessageDecoder::authenticated(1, 2, 10_000);
    assert_eq!(
        decoder.decode(&body, None, 0, |_, _| true),
        Err(SignedDecodeError::MissingSignature)
    );
    assert_eq!(
        decoder
            .decode(&body, Some(&signature(1)), 0, |_, _| true)
            .unwrap()
            .link
            .unwrap()
            .index,
        0
    );
    assert!(
        decoder
            .decode(&body, Some(&signature(2)), 0, |_, _| true)
            .is_ok()
    );
    let earlier = SignedMessageBody {
        timestamp_millis: 999,
        ..body.clone()
    };
    assert_eq!(
        decoder.decode(&earlier, Some(&signature(3)), 0, |_, _| true),
        Err(SignedDecodeError::OutOfOrderTimestamp)
    );
    assert_eq!(
        decoder.decode(&body, Some(&signature(4)), 0, |_, _| true),
        Err(SignedDecodeError::BrokenChain)
    );

    let mut invalid = MessageDecoder::authenticated(1, 2, 10_000);
    assert_eq!(
        invalid.decode(&body, Some(&signature(1)), 0, |_, _| false),
        Err(SignedDecodeError::InvalidSignature)
    );
    assert_eq!(
        invalid.decode(&body, Some(&signature(1)), 0, |_, _| true),
        Err(SignedDecodeError::BrokenChain)
    );
}

#[test]
fn c3_signed_command_consumes_wire_order_replaces_duplicates_and_breaks_unknown_names() {
    let authoritative = [
        SignableArgument {
            name: "target",
            value: "Alice",
        },
        SignableArgument {
            name: "message",
            value: "hello",
        },
    ];
    let mut packet = signed_command();
    packet.argument_signatures = vec![
        ArgumentSignature {
            name: "target".to_owned(),
            signature: signature(1),
        },
        ArgumentSignature {
            name: "target".to_owned(),
            signature: signature(2),
        },
        ArgumentSignature {
            name: "message".to_owned(),
            signature: signature(3),
        },
    ];
    let mut indexes = Vec::new();
    let mut decoder = MessageDecoder::authenticated(1, 2, 10_000);
    let decoded = collect_signed_arguments(
        &mut decoder,
        &packet,
        &authoritative,
        &[],
        0,
        |payload, _| {
            indexes.push(i32::from_be_bytes(payload[36..40].try_into().unwrap()));
            true
        },
    )
    .unwrap();
    assert_eq!(indexes, [0, 1, 2]);
    assert_eq!(decoded["target"].outcome.link.unwrap().index, 1);
    assert_eq!(decoded["message"].outcome.link.unwrap().index, 2);

    packet.argument_signatures[0].name = "unknown".to_owned();
    assert_eq!(
        collect_signed_arguments(&mut decoder, &packet, &authoritative, &[], 0, |_, _| true),
        Err(SignedArgumentError::Mismatch)
    );
    let body = SignedMessageBody {
        content: "later".to_owned(),
        timestamp_millis: 2_000,
        salt: 0,
        last_seen: Vec::new(),
    };
    assert_eq!(
        decoder.decode(&body, Some(&signature(9)), 0, |_, _| true),
        Err(SignedDecodeError::BrokenChain)
    );
    assert!(!unsigned_command_allowed(true, 1));
    assert!(unsigned_command_allowed(true, 0));
}

#[test]
fn c3_chat_session_compares_key_data_validates_payload_and_installs_decoder_atomically() {
    let player = 0x0102_0304_0506_0708_1112_1314_1516_1718;
    let key = profile_key(100, &[1, 2, 3], &[4, 5]);
    let mut state = ChatSessionState::new(player, true);
    let first = ChatSessionUpdate {
        session_id: 7,
        profile_key: key.clone(),
    };
    let expected_payload = profile_key_signed_payload(player, &key);
    assert_eq!(
        state.apply_update(first.clone(), |payload, signature| {
            assert_eq!(
                (payload, signature),
                (expected_payload.as_slice(), &[4, 5][..])
            );
            ProfileKeyValidation::Valid
        }),
        ChatSessionAction::InstalledAndBroadcastInitializeChat
    );
    assert_eq!(state.installed().unwrap().session_id, 7);
    assert!(matches!(
        state.decoder(),
        MessageDecoder::Authenticated { .. }
    ));
    assert_eq!(
        state.apply_update(
            ChatSessionUpdate {
                session_id: 8,
                ..first.clone()
            },
            |_, _| panic!("equal key data must skip validation")
        ),
        ChatSessionAction::NoOpEqualKeyData
    );
    assert_eq!(state.installed().unwrap().session_id, 7);
    assert_eq!(
        state.apply_update(
            ChatSessionUpdate {
                session_id: 9,
                profile_key: profile_key(99, &[9], &[9]),
            },
            |_, _| ProfileKeyValidation::Valid
        ),
        ChatSessionAction::DisconnectExpiredPublicKey
    );
}

#[test]
fn c3_chat_session_missing_and_invalid_validator_do_not_mutate_first_key() {
    for (validation, action) in [
        (
            ProfileKeyValidation::ValidatorUnavailable,
            ChatSessionAction::WarnAndIgnoreMissingValidator,
        ),
        (
            ProfileKeyValidation::Invalid,
            ChatSessionAction::DisconnectInvalidPublicKey,
        ),
    ] {
        let mut state = ChatSessionState::new(1, false);
        assert_eq!(
            state.apply_update(
                ChatSessionUpdate {
                    session_id: 2,
                    profile_key: profile_key(i64::MIN, &[1], &[2]),
                },
                |_, _| validation
            ),
            action
        );
        assert_eq!(state.installed(), None);
        assert_eq!(state.decoder(), &MessageDecoder::unsigned(false));
    }
}

#[test]
fn c3_chat_admission_mutates_last_seen_before_character_and_visibility_failures() {
    let sig = signature(7);
    for (message, visibility, expected) in [
        (
            "bad\u{00a7}",
            ChatVisibility::Full,
            ChatAdmission::DisconnectIllegalCharacters,
        ),
        (
            "hidden",
            ChatVisibility::Hidden,
            ChatAdmission::DisabledByOptions,
        ),
    ] {
        let mut validator = LastSeenValidator::default();
        validator.add_pending(sig.clone()).unwrap();
        let mut packet = chat_message(message);
        packet.last_seen = update(1, [0, 0, 0x08], checksum(std::slice::from_ref(&sig)));
        assert_eq!(admit_chat(&mut validator, &packet, visibility), expected);
        assert_eq!((validator.tracked_len(), validator.pending_len()), (20, 0));
    }
    assert!(has_illegal_chat_character("\n"));
    assert!(has_illegal_chat_character("\u{007f}"));
    assert!(!has_illegal_chat_character("😀"));
    assert_eq!(admit_unsigned_command("say ok"), ChatAdmission::Scheduled);
    let mut validator = LastSeenValidator::default();
    assert_eq!(
        admit_signed_command(&mut validator, &signed_command()),
        ChatAdmission::Scheduled
    );
}

#[test]
fn c3_chat_and_command_throttlers_are_independent_tick_and_exemption_counters() {
    let mut chat = TickThrottler::new(10);
    let mut command = TickThrottler::new(10);
    for _ in 0..9 {
        assert!(!chat.charge(false));
    }
    assert_eq!((chat.counter(), command.counter()), (180, 0));
    assert!(chat.charge(false));
    assert!(!command.charge(true));
    assert_eq!(command.counter(), 20);
    chat.tick();
    command.tick();
    assert_eq!((chat.counter(), command.counter()), (199, 19));
    assert!(!TickThrottler::new(0).charge(false));
}

#[test]
fn c3_command_suggestion_strips_one_slash_preserves_id_and_caps_only_the_list() {
    let normalized = normalize_suggestion(&CommandSuggestion {
        transaction_id: -1,
        input: "//give".to_owned(),
    });
    assert_eq!(
        (normalized.transaction_id, normalized.parsed_input.as_str()),
        (-1, "/give")
    );
    let mut suggestions = (0..1_005).collect::<Vec<_>>();
    truncate_suggestions(&mut suggestions);
    assert_eq!(suggestions, (0..1_000).collect::<Vec<_>>());
}

#[test]
fn c3_last_seen_pending_limit_disconnects_only_after_retaining_the_4097th_entry() {
    let mut validator = LastSeenValidator::default();
    for value in 0..(4_096 - LAST_SEEN_WINDOW) {
        let mut bytes = [0_u8; 256];
        bytes[..8].copy_from_slice(&(value as u64).to_be_bytes());
        assert_eq!(
            validator.add_pending(MessageSignature(Box::new(bytes))),
            Ok(PendingResult::Added)
        );
    }
    assert_eq!(validator.tracked_len(), 4_096);
    let mut overflow = [0xff_u8; 256];
    overflow[..8].copy_from_slice(&u64::MAX.to_be_bytes());
    assert_eq!(
        validator.add_pending(MessageSignature(Box::new(overflow))),
        Err(LastSeenError::TooManyPending { tracked: 4_097 })
    );
    assert_eq!(validator.tracked_len(), 4_097);
}

#[test]
fn c3_suggestion_completion_preserves_range_and_transaction_while_capping_entries() {
    use ferrite_protocol::java_26_2::play::clientbound::completion::packet::SuggestionEntry;

    let entries = (0..1_005)
        .map(|index| SuggestionEntry {
            text: index.to_string(),
            tooltip: None,
        })
        .collect();
    let response = complete_suggestions(
        &CommandSuggestion {
            transaction_id: i32::MIN,
            input: "/x".to_owned(),
        },
        -7,
        i32::MAX,
        entries,
    );
    assert_eq!(
        (response.transaction, response.start, response.length),
        (i32::MIN, -7, i32::MAX)
    );
    assert_eq!(response.entries.len(), 1_000);
}

#[test]
fn c3_chat_future_chain_serializes_filter_completion_and_cancels_on_disconnect() {
    fn normalized(content: &str) -> NormalizedPlayerChat {
        NormalizedPlayerChat {
            sender: 7,
            signed_content: content.to_owned(),
            decorated_content: Some(format!("<{content}>")),
            filter_policy: ChatFilterPolicy::Pass,
        }
    }

    let mut chain = ChatFutureChain::default();
    let first = chain.append().unwrap();
    let second = chain.append().unwrap();
    assert!(chain.complete(second, normalized("second")).is_empty());
    assert_eq!(
        chain
            .complete(first, normalized("first"))
            .iter()
            .map(|event| event.signed_content.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    let third = chain.append().unwrap();
    chain.disconnect();
    assert!(chain.complete(third, normalized("third")).is_empty());
    assert_eq!(chain.append(), None);
}
