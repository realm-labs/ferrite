use ferrite_foundation::coordinate::BlockPos;
use ferrite_protocol::java_26_2::play::serverbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use ferrite_protocol::java_26_2::play::serverbound::sign_update::packet::SignUpdate;
use ferrite_protocol::java_26_2::play::serverbound::sign_update::transaction::{
    FilteredLine, PendingSignSubmission, SignCompletionState, SignEditorProjection, SignEntity,
    SignLine, SignText, SignUpdateOutcome, complete_sign_update, strip_legacy_formatting,
    tick_editor_authorization,
};

fn packet(lines: [&str; 4]) -> SignUpdate {
    SignUpdate {
        position: BlockPos::new(0, 0, 0),
        front_text: false,
        lines: lines.map(str::to_owned),
    }
}

fn text(prefix: &str) -> SignText {
    SignText {
        lines: std::array::from_fn(|index| SignLine {
            literal: format!("{prefix}{index}"),
            filtered_literal: Some(format!("filtered-{prefix}{index}")),
            style: format!("style-{prefix}{index}"),
        }),
    }
}

fn sign(editor: Option<u128>) -> SignEntity {
    SignEntity {
        waxed: false,
        has_level: true,
        allowed_editor: editor,
        front: text("front-"),
        back: text("back-"),
        changed_calls: 0,
        block_update_flags: Vec::new(),
    }
}

fn filtered(raw: [&str; 4], filtered: [&str; 4]) -> [FilteredLine; 4] {
    std::array::from_fn(|index| FilteredLine {
        raw: raw[index].to_owned(),
        filtered_or_empty: filtered[index].to_owned(),
    })
}

#[test]
fn c3_gold_serverbound_sign_update_locks_empty_back_text() {
    let expected = vec![61, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let wrapped = PlayServerboundEntryPacket::SignUpdate(packet(["", "", "", ""]));
    assert_eq!(encode_packet(wrapped.clone()).unwrap(), expected);
    assert_eq!(decode_packet(&expected).unwrap(), wrapped);
}

#[test]
fn c3_sign_update_codec_preserves_position_side_order_and_nonzero_boolean() {
    for value in [
        SignUpdate {
            position: BlockPos::new(-33_554_432, -2_048, -33_554_432),
            front_text: false,
            lines: ["one", "two", "three", "four"].map(str::to_owned),
        },
        SignUpdate {
            position: BlockPos::new(33_554_431, 2_047, 33_554_431),
            front_text: true,
            lines: ["😀", "", "x", "last"].map(str::to_owned),
        },
    ] {
        let wrapped = PlayServerboundEntryPacket::SignUpdate(value);
        assert_eq!(
            decode_packet(&encode_packet(wrapped.clone()).unwrap()).unwrap(),
            wrapped
        );
    }

    let mut noncanonical = encode_packet(PlayServerboundEntryPacket::SignUpdate(SignUpdate {
        front_text: true,
        ..packet(["", "", "", ""])
    }))
    .unwrap();
    noncanonical[9] = 0xfe;
    assert!(matches!(
        decode_packet(&noncanonical).unwrap(),
        PlayServerboundEntryPacket::SignUpdate(SignUpdate {
            front_text: true,
            ..
        })
    ));
}

#[test]
fn c3_sign_member_encoder_is_asymmetric_with_the_384_unit_server_decoder() {
    let accepted = PlayServerboundEntryPacket::SignUpdate(packet([&"x".repeat(384), "", "", ""]));
    let encoded = encode_packet(accepted.clone()).unwrap();
    assert_eq!(decode_packet(&encoded).unwrap(), accepted);

    let member_only =
        PlayServerboundEntryPacket::SignUpdate(packet([&"x".repeat(385), "", "", ""]));
    let encoded = encode_packet(member_only).unwrap();
    assert!(decode_packet(&encoded).is_err());
    assert!(
        encode_packet(PlayServerboundEntryPacket::SignUpdate(packet([
            &"x".repeat(32_768),
            "",
            "",
            "",
        ])))
        .is_err()
    );
}

#[test]
fn c3_sign_codec_replacement_decodes_malformed_utf8_and_faults_framing() {
    let mut malformed = vec![61];
    malformed.extend([0; 9]);
    malformed.extend([2, 0xc3, 0x28, 0, 0, 0]);
    assert_eq!(
        decode_packet(&malformed).unwrap(),
        PlayServerboundEntryPacket::SignUpdate(packet(["�(", "", "", ""]))
    );

    let encoded = encode_packet(PlayServerboundEntryPacket::SignUpdate(packet([
        "one", "two", "three", "four",
    ])))
    .unwrap();
    for length in 1..encoded.len() {
        assert!(decode_packet(&encoded[..length]).is_err());
    }
    let mut trailing = encoded;
    trailing.push(0);
    assert!(decode_packet(&trailing).is_err());
}

#[test]
fn c3_sign_submission_strips_only_recognized_legacy_formatting_in_line_order() {
    assert_eq!(strip_legacy_formatting("a§1b§Kc§zd§"), "abc§zd§");
    let submission = PendingSignSubmission::from_packet(SignUpdate {
        position: BlockPos::new(1, 2, 3),
        front_text: true,
        lines: ["§aone", "two§R", "§xthree", "four"].map(str::to_owned),
    });
    assert_eq!(submission.position, BlockPos::new(1, 2, 3));
    assert!(submission.front_text);
    assert_eq!(submission.stripped_lines, ["one", "two", "§xthree", "four"]);
}

#[test]
fn c3_sign_editor_removed_submits_current_lines_once_only_when_connected() {
    let mut editor = SignEditorProjection::new(
        BlockPos::new(1, 2, 3),
        true,
        ["one", "two", "three", "four"].map(str::to_owned),
    );
    editor.lines_mut()[2] = "edited".to_owned();
    assert_eq!(
        editor.removed(true),
        Some(SignUpdate {
            position: BlockPos::new(1, 2, 3),
            front_text: true,
            lines: ["one", "two", "edited", "four"].map(str::to_owned),
        })
    );
    assert_eq!(editor.removed(true), None);

    let mut disconnected = SignEditorProjection::new(
        BlockPos::new(0, 0, 0),
        false,
        [String::new(), String::new(), String::new(), String::new()],
    );
    assert_eq!(disconnected.removed(false), None);
    assert_eq!(disconnected.removed(true), None);
}

#[test]
fn c3_sign_success_replaces_selected_side_preserves_styles_and_emits_two_updates() {
    let mut state = SignCompletionState::Sign(Box::new(sign(Some(7))));
    let old_back = match &state {
        SignCompletionState::Sign(sign) => sign.back.clone(),
        _ => unreachable!(),
    };
    let submission = PendingSignSubmission::from_packet(SignUpdate {
        front_text: true,
        ..packet(["raw0", "raw1", "raw2", "raw3"])
    });
    assert_eq!(
        complete_sign_update(
            &mut state,
            &submission,
            filtered(
                ["raw0", "raw1", "raw2", "raw3"],
                ["safe0", "", "safe2", "safe3"],
            ),
            7,
            false,
        ),
        SignUpdateOutcome::Applied
    );
    let SignCompletionState::Sign(sign) = state else {
        unreachable!();
    };
    assert_eq!(sign.back, old_back);
    assert_eq!(
        sign.front
            .lines
            .map(|line| (line.literal, line.filtered_literal, line.style)),
        [
            (
                "raw0".to_owned(),
                Some("safe0".to_owned()),
                "style-front-0".to_owned()
            ),
            (
                "raw1".to_owned(),
                Some(String::new()),
                "style-front-1".to_owned()
            ),
            (
                "raw2".to_owned(),
                Some("safe2".to_owned()),
                "style-front-2".to_owned()
            ),
            (
                "raw3".to_owned(),
                Some("safe3".to_owned()),
                "style-front-3".to_owned()
            ),
        ]
    );
    assert_eq!(
        (sign.changed_calls, sign.block_update_flags),
        (1, vec![3, 3])
    );
    assert_eq!(sign.allowed_editor, None);
}

#[test]
fn c3_sign_filtering_enabled_stores_only_filtered_literals_on_the_selected_back() {
    let mut state = SignCompletionState::Sign(Box::new(sign(Some(9))));
    let submission = PendingSignSubmission::from_packet(packet(["a", "b", "c", "d"]));
    assert_eq!(
        complete_sign_update(
            &mut state,
            &submission,
            filtered(["a", "b", "c", "d"], ["A", "", "C", "D"]),
            9,
            true,
        ),
        SignUpdateOutcome::Applied
    );
    let SignCompletionState::Sign(sign) = state else {
        unreachable!();
    };
    assert_eq!(
        sign.back
            .lines
            .map(|line| (line.literal, line.filtered_literal)),
        [
            ("A".to_owned(), None),
            (String::new(), None),
            ("C".to_owned(), None),
            ("D".to_owned(), None),
        ]
    );
}

#[test]
fn c3_sign_completion_rechecks_loaded_entity_wax_level_and_exact_editor_without_cleanup() {
    let submission = PendingSignSubmission::from_packet(packet(["a", "b", "c", "d"]));
    let lines = filtered(["a", "b", "c", "d"], ["a", "b", "c", "d"]);
    for (mut state, outcome) in [
        (
            SignCompletionState::Unloaded,
            SignUpdateOutcome::IgnoredUnloaded,
        ),
        (
            SignCompletionState::MissingBlockEntity,
            SignUpdateOutcome::IgnoredMissingSign,
        ),
        (
            SignCompletionState::OtherBlockEntity,
            SignUpdateOutcome::IgnoredMissingSign,
        ),
        (
            SignCompletionState::Sign(Box::new(SignEntity {
                waxed: true,
                ..sign(Some(7))
            })),
            SignUpdateOutcome::RejectedAuthorization,
        ),
        (
            SignCompletionState::Sign(Box::new(SignEntity {
                has_level: false,
                ..sign(Some(7))
            })),
            SignUpdateOutcome::RejectedAuthorization,
        ),
        (
            SignCompletionState::Sign(Box::new(sign(Some(8)))),
            SignUpdateOutcome::RejectedAuthorization,
        ),
    ] {
        assert_eq!(
            complete_sign_update(&mut state, &submission, lines.clone(), 7, false),
            outcome
        );
        if let SignCompletionState::Sign(sign) = state {
            assert_eq!(
                sign.allowed_editor,
                if sign.waxed || !sign.has_level {
                    Some(7)
                } else {
                    Some(8)
                }
            );
            assert_eq!((sign.changed_calls, sign.block_update_flags.len()), (0, 0));
        }
    }
}

#[test]
fn c3_sign_tick_clears_only_stale_editor_authorization() {
    let mut entity = sign(Some(7));
    tick_editor_authorization(&mut entity, true);
    assert_eq!(entity.allowed_editor, Some(7));
    tick_editor_authorization(&mut entity, false);
    assert_eq!(entity.allowed_editor, None);
    tick_editor_authorization(&mut entity, false);
    assert_eq!(entity.allowed_editor, None);
}

#[test]
fn c3_sign_decode_filter_and_current_state_commit_join_end_to_end() {
    let wire = encode_packet(PlayServerboundEntryPacket::SignUpdate(SignUpdate {
        position: BlockPos::new(-4, 5, 6),
        front_text: true,
        lines: ["§aone", "two", "three", "four"].map(str::to_owned),
    }))
    .unwrap();
    let PlayServerboundEntryPacket::SignUpdate(decoded) = decode_packet(&wire).unwrap() else {
        panic!("decoded packet must retain its identity");
    };
    let submission = PendingSignSubmission::from_packet(decoded);
    assert_eq!(submission.stripped_lines[0], "one");
    let mut state = SignCompletionState::Sign(Box::new(sign(Some(42))));
    assert_eq!(
        complete_sign_update(
            &mut state,
            &submission,
            filtered(
                ["one", "two", "three", "four"],
                ["one", "two", "three", "four"],
            ),
            42,
            false,
        ),
        SignUpdateOutcome::Applied
    );
}
