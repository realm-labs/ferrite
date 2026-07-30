use ferrite_protocol::java_26_2::play::registry::{MOB_EFFECT, PlayRegistries};
use ferrite_protocol::java_26_2::play::serverbound::anvil_beacon::anvil::{
    AnvilClientEdit, AnvilClientProjection, AnvilInputProjection, AnvilMenuState,
    AnvilRenameOutcome, filter_name, handle_rename,
};
use ferrite_protocol::java_26_2::play::serverbound::anvil_beacon::beacon::{
    BeaconAdmissionError, BeaconClientAction, BeaconClientEmission, BeaconClientProjection,
    BeaconCommitOutcome, BeaconDataField, BeaconEffect, BeaconMenuState, handle_set_beacon,
};
use ferrite_protocol::java_26_2::play::serverbound::anvil_beacon::packet::{RenameItem, SetBeacon};
use ferrite_protocol::java_26_2::play::serverbound::codec::{
    PlayServerboundEntryCodecError, decode_packet, decode_packet_with_registries, encode_packet,
    encode_packet_with_registries,
};
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use ferrite_protocol::java_26_2::value::identifier::Identifier;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut effects = [
        "speed",
        "slowness",
        "haste",
        "mining_fatigue",
        "strength",
        "instant_health",
        "instant_damage",
        "jump_boost",
        "nausea",
        "regeneration",
        "resistance",
        "fire_resistance",
        "water_breathing",
        "invisibility",
        "blindness",
        "night_vision",
        "hunger",
        "weakness",
        "poison",
        "wither",
        "health_boost",
        "absorption",
        "saturation",
        "glowing",
        "levitation",
        "luck",
        "unluck",
        "slow_falling",
        "conduit_power",
        "dolphins_grace",
        "bad_omen",
        "hero_of_the_village",
        "darkness",
        "trial_omen",
        "raid_omen",
        "wind_charged",
        "weaving",
        "oozing",
        "infested",
        "breath_of_the_nautilus",
    ]
    .into_iter()
    .map(|path| id(&format!("minecraft:{path}")))
    .collect::<Vec<_>>();
    assert_eq!(effects.len(), 40);
    let mut registries = PlayRegistries::default();
    registries.insert(id(MOB_EFFECT), std::mem::take(&mut effects));
    registries
}

fn rename(name: &str) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::RenameItem(RenameItem {
        name: name.to_owned(),
    })
}

fn beacon(primary: Option<&str>, secondary: Option<&str>) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::SetBeacon(SetBeacon {
        primary: primary.map(id),
        secondary: secondary.map(id),
    })
}

#[test]
fn c3_gold_serverbound_anvil_beacon_locks_both_packets() {
    let registries = registries();
    assert_eq!(
        encode_packet(rename("Forge")).unwrap(),
        [0x30, 0x05, b'F', b'o', b'r', b'g', b'e']
    );
    assert_eq!(
        encode_packet_with_registries(
            beacon(Some("minecraft:speed"), Some("minecraft:regeneration")),
            &registries,
        )
        .unwrap(),
        [0x34, 0x01, 0x00, 0x01, 0x09]
    );
}

#[test]
fn c3_anvil_beacon_codecs_enforce_utf_registry_and_complete_consumption() {
    let registries = registries();
    for packet in [
        rename("rename \u{1f680}"),
        beacon(None, None),
        beacon(
            Some("minecraft:breath_of_the_nautilus"),
            Some("minecraft:regeneration"),
        ),
    ] {
        let encoded = encode_packet_with_registries(packet.clone(), &registries).unwrap();
        assert_eq!(
            decode_packet_with_registries(&encoded, &registries).unwrap(),
            packet
        );
    }

    let malformed_utf = [0x30, 0x01, 0xff];
    assert_eq!(decode_packet(&malformed_utf).unwrap(), rename("\u{fffd}"));
    let noncanonical_presence = [0x34, 0x02, 0x00, 0x00];
    let decoded = decode_packet_with_registries(&noncanonical_presence, &registries).unwrap();
    assert_eq!(decoded, beacon(Some("minecraft:speed"), None));
    assert_eq!(
        encode_packet_with_registries(decoded, &registries).unwrap(),
        [0x34, 0x01, 0x00, 0x00]
    );

    assert!(matches!(
        decode_packet(&[0x34, 0x00, 0x00]),
        Err(PlayServerboundEntryCodecError::MissingRegistryContext { .. })
    ));
    assert!(decode_packet_with_registries(&[0x34, 0x01, 0x28, 0x00], &registries).is_err());
    assert!(decode_packet_with_registries(&[0x34, 0x01], &registries).is_err());
    assert!(decode_packet_with_registries(&[0x34, 0x01, 0x80], &registries).is_err());
    assert!(decode_packet_with_registries(&[0x34, 0x00, 0x00, 0x00], &registries).is_err());
    assert!(encode_packet(rename(&"a".repeat(32_768))).is_err());
}

#[test]
fn c3_anvil_rename_prediction_precedes_send_and_normalizes_default_name() {
    let mut client = AnvilClientProjection::new(true);
    assert_eq!(client.edit("Name"), AnvilClientEdit::IgnoredMissingInput);
    client.set_input(Some(AnvilInputProjection {
        hover_name: "Iron Sword".to_owned(),
        has_custom_name: false,
    }));
    assert_eq!(client.edit_text(), "Iron Sword");
    assert_eq!(
        client.edit("Iron Sword"),
        AnvilClientEdit::Unchanged,
        "the default hover name normalizes to the already-empty menu name"
    );
    assert_eq!(
        client.edit("New\u{a7} Name"),
        AnvilClientEdit::PredictedAndSend(RenameItem {
            name: "New\u{a7} Name".to_owned(),
        })
    );
    assert_eq!(client.accepted_name(), "New Name");
    assert_eq!(client.result_custom_name(), Some("New Name"));
    assert_eq!(client.recomputations(), 1);
    assert_eq!(
        client.edit("New Name"),
        AnvilClientEdit::Unchanged,
        "distinct wire edits collapsing under filtering do not send"
    );
    assert_eq!(
        client.edit(&"a".repeat(51)),
        AnvilClientEdit::RejectedClientLength
    );
}

#[test]
fn c3_anvil_rename_admission_filters_before_bound_blank_and_broadcast() {
    let packet = RenameItem {
        name: format!("{}\u{a7}\u{0}\u{7f}", "x".repeat(50)),
    };
    let mut menu = AnvilMenuState::new(true, true);
    let AnvilRenameOutcome::Applied(convergence) = handle_rename(Some(&mut menu), &packet) else {
        panic!("filtered 50-unit name should apply");
    };
    assert_eq!(convergence.accepted_name, "x".repeat(50));
    assert_eq!(convergence.result_custom_name, Some("x".repeat(50)));
    assert_eq!((convergence.recomputations, convergence.broadcasts), (1, 1));

    assert_eq!(
        handle_rename(
            Some(&mut menu),
            &RenameItem {
                name: format!("{}\u{a7}", "x".repeat(50)),
            },
        ),
        AnvilRenameOutcome::NoChange
    );
    assert_eq!(
        handle_rename(
            Some(&mut menu),
            &RenameItem {
                name: "x".repeat(51),
            },
        ),
        AnvilRenameOutcome::NoChange
    );
    let outcome = handle_rename(
        Some(&mut menu),
        &RenameItem {
            name: "\u{a0}\u{2007}\u{202f}".to_owned(),
        },
    );
    assert!(matches!(outcome, AnvilRenameOutcome::Applied(_)));
    assert_eq!(menu.result_custom_name, None);
    assert_eq!(filter_name("a\u{1f680}b"), "a\u{1f680}b");

    let mut invalid = AnvilMenuState::new(false, true);
    assert_eq!(
        handle_rename(Some(&mut invalid), &RenameItem { name: "x".into() }),
        AnvilRenameOutcome::IgnoredInvalidMenu
    );
    assert_eq!(
        handle_rename(None, &RenameItem { name: "x".into() }),
        AnvilRenameOutcome::IgnoredWrongMenu
    );
}

#[test]
fn c3_beacon_effect_mapping_separates_wire_ids_menu_data_and_filtered_state() {
    let registries = registries();
    let mut menu = BeaconMenuState::new(true, i32::MAX, 2);
    menu.beam_sections_nonempty = true;
    let packet = SetBeacon {
        primary: Some(id("minecraft:speed")),
        secondary: Some(id("minecraft:slowness")),
    };
    let BeaconCommitOutcome::Applied(commit) =
        handle_set_beacon(Some(&mut menu), &packet, &registries).unwrap()
    else {
        panic!("maximum level admits codec-valid secondary");
    };
    assert_eq!(commit.data_writes[0].field, BeaconDataField::Primary);
    assert_eq!(commit.data_writes[0].value, 1);
    assert_eq!(commit.data_writes[1].field, BeaconDataField::Secondary);
    assert_eq!(commit.data_writes[1].value, 2);
    assert_eq!(commit.primary, Some(BeaconEffect::Speed));
    assert_eq!(commit.secondary, None);
    assert_eq!(commit.remaining_payment, 1);
    assert!(commit.played_selection_sound);
    assert!(menu.chunk_unsaved);
    assert_eq!(menu.selection_sounds, 1);
    assert_eq!(menu.remaining_payment_on_close(), 1);
}

#[test]
fn c3_beacon_selection_admission_covers_payment_tiers_and_null_quirk() {
    let registries = registries();
    let packet = SetBeacon {
        primary: None,
        secondary: None,
    };
    assert_eq!(
        handle_set_beacon(None, &packet, &registries).unwrap(),
        BeaconCommitOutcome::IgnoredWrongMenu
    );
    let mut invalid = BeaconMenuState::new(false, 4, 1);
    assert_eq!(
        handle_set_beacon(Some(&mut invalid), &packet, &registries).unwrap(),
        BeaconCommitOutcome::IgnoredInvalidMenu
    );
    let mut unpaid = BeaconMenuState::new(true, 4, 0);
    assert_eq!(
        handle_set_beacon(Some(&mut unpaid), &packet, &registries).unwrap(),
        BeaconCommitOutcome::DisconnectGeneric
    );
    let mut forged = BeaconMenuState::new(true, 4, 2);
    assert!(matches!(
        handle_set_beacon(Some(&mut forged), &packet, &registries).unwrap(),
        BeaconCommitOutcome::Applied(_)
    ));
    assert_eq!(forged.payment_count, 1);

    let mut regeneration = BeaconMenuState::new(true, 4, 1);
    assert!(matches!(
        handle_set_beacon(
            Some(&mut regeneration),
            &SetBeacon {
                primary: None,
                secondary: Some(id("minecraft:regeneration")),
            },
            &registries,
        )
        .unwrap(),
        BeaconCommitOutcome::Applied(_)
    ));
    let mut null_primary = BeaconMenuState::new(true, 4, 1);
    assert_eq!(
        handle_set_beacon(
            Some(&mut null_primary),
            &SetBeacon {
                primary: None,
                secondary: Some(id("minecraft:speed")),
            },
            &registries,
        ),
        Err(BeaconAdmissionError::NullPrimaryEquality)
    );
    assert_eq!(null_primary.payment_count, 1);

    let mut tier_refusal = BeaconMenuState::new(true, 3, 1);
    assert_eq!(
        handle_set_beacon(
            Some(&mut tier_refusal),
            &SetBeacon {
                primary: Some(id("minecraft:regeneration")),
                secondary: None,
            },
            &registries,
        )
        .unwrap(),
        BeaconCommitOutcome::DisconnectGeneric
    );
}

#[test]
fn c3_anvil_beacon_order_sends_selection_before_close_without_prediction() {
    let mut client = BeaconClientProjection::new(4, true);
    assert!(client.choose_primary(BeaconEffect::Speed));
    assert!(client.choose_regeneration());
    assert!(client.choose_primary(BeaconEffect::Haste));
    assert_eq!(client.secondary, None);
    assert!(client.choose_primary_upgrade());
    assert_eq!(
        client.done(),
        BeaconClientAction::Emit(vec![
            BeaconClientEmission::SetBeacon(SetBeacon {
                primary: Some(id("minecraft:haste")),
                secondary: Some(id("minecraft:haste")),
            }),
            BeaconClientEmission::CloseContainer,
        ])
    );
    assert_eq!(
        BeaconClientProjection::cancel(),
        BeaconClientAction::Emit(vec![BeaconClientEmission::CloseContainer])
    );
    assert_eq!(
        BeaconClientProjection::new(4, false).done(),
        BeaconClientAction::Disabled
    );
    assert_eq!(
        BeaconClientProjection::new(3, true).done(),
        BeaconClientAction::Disabled
    );
}

#[test]
fn c3_anvil_beacon_end_to_end_decodes_into_current_menu_transactions() {
    let registries = registries();
    let rename_body = encode_packet(rename("A\u{a7}xe")).unwrap();
    let PlayServerboundEntryPacket::RenameItem(rename_packet) =
        decode_packet_with_registries(&rename_body, &registries).unwrap()
    else {
        panic!("expected rename packet");
    };
    let mut anvil = AnvilMenuState::new(true, true);
    assert!(matches!(
        handle_rename(Some(&mut anvil), &rename_packet),
        AnvilRenameOutcome::Applied(_)
    ));
    assert_eq!(anvil.accepted_name, "Axe");

    let beacon_body = encode_packet_with_registries(
        beacon(Some("minecraft:resistance"), Some("minecraft:regeneration")),
        &registries,
    )
    .unwrap();
    let PlayServerboundEntryPacket::SetBeacon(beacon_packet) =
        decode_packet_with_registries(&beacon_body, &registries).unwrap()
    else {
        panic!("expected beacon packet");
    };
    let mut menu = BeaconMenuState::new(true, 4, 1);
    assert!(matches!(
        handle_set_beacon(Some(&mut menu), &beacon_packet, &registries).unwrap(),
        BeaconCommitOutcome::Applied(_)
    ));
    assert_eq!(menu.primary, Some(BeaconEffect::Resistance));
    assert_eq!(menu.secondary, Some(BeaconEffect::Regeneration));
    assert_eq!(menu.payment_count, 0);
}
