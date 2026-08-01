use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::title_tab::packet::{
    ClearTitles, SelectAdvancementsTab, SetActionBarText, SetSubtitleText, SetTitleText,
    SetTitlesAnimation, TabList,
};
use ferrite_protocol::java_26_2::play::clientbound::title_tab::projection::{
    AdvancementTabObject, TitleTabProjection,
};
use ferrite_protocol::java_26_2::play::clientbound::title_tab::publication::{
    AdvancementTabDefinition, AdvancementTabPublisher, CanonicalTitleTimes, ResolvedTitleKind,
    ResolvedTitlePacket, publish_animation, publish_clear, publish_resolved,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn component(value: &str) -> TextComponentNbt {
    TextComponentNbt::literal(value).unwrap()
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn assert_roundtrip(packet: PlayClientboundPacket) {
    let registries = PlayRegistries::default();
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );
}

#[test]
fn c3_gold_clientbound_title_tab_locks_all_seven_empty_bodies() {
    let registries = PlayRegistries::default();
    let empty = component("");
    let cases = [
        (
            PlayClientboundPacket::ClearTitles(ClearTitles { reset_times: false }),
            vec![14, 0],
        ),
        (
            PlayClientboundPacket::SelectAdvancementsTab(SelectAdvancementsTab { tab: None }),
            vec![85, 0],
        ),
        (
            PlayClientboundPacket::SetActionBarText(SetActionBarText {
                text: empty.clone(),
            }),
            vec![87, 8, 0, 0],
        ),
        (
            PlayClientboundPacket::SetSubtitleText(SetSubtitleText {
                text: empty.clone(),
            }),
            vec![112, 8, 0, 0],
        ),
        (
            PlayClientboundPacket::SetTitleText(SetTitleText {
                text: empty.clone(),
            }),
            vec![114, 8, 0, 0],
        ),
        (
            PlayClientboundPacket::SetTitlesAnimation(SetTitlesAnimation {
                fade_in: 0,
                stay: 0,
                fade_out: 0,
            }),
            vec![115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            PlayClientboundPacket::TabList(TabList {
                header: empty.clone(),
                footer: empty,
            }),
            vec![122, 8, 0, 0, 8, 0, 0],
        ),
    ];
    for (packet, expected) in cases {
        assert_eq!(encode_packet(&packet, &registries).unwrap(), expected);
        assert_eq!(
            decode_packet(&expected, context(&registries)).unwrap(),
            packet
        );
    }
}

#[test]
fn c3_title_tab_codecs_roundtrip_components_identifiers_and_signed_times() {
    assert_roundtrip(PlayClientboundPacket::ClearTitles(ClearTitles {
        reset_times: true,
    }));
    assert_roundtrip(PlayClientboundPacket::SelectAdvancementsTab(
        SelectAdvancementsTab {
            tab: Some(id("ferrite:root")),
        },
    ));
    assert_roundtrip(PlayClientboundPacket::SetActionBarText(SetActionBarText {
        text: component("action"),
    }));
    assert_roundtrip(PlayClientboundPacket::SetSubtitleText(SetSubtitleText {
        text: TextComponentNbt::translatable("ferrite.subtitle").unwrap(),
    }));
    assert_roundtrip(PlayClientboundPacket::SetTitleText(SetTitleText {
        text: component("title"),
    }));
    assert_roundtrip(PlayClientboundPacket::SetTitlesAnimation(
        SetTitlesAnimation {
            fade_in: i32::MIN,
            stay: -1,
            fade_out: i32::MAX,
        },
    ));
    assert_roundtrip(PlayClientboundPacket::TabList(TabList {
        header: component("header"),
        footer: component("footer"),
    }));
}

#[test]
fn c3_title_tab_codecs_normalize_booleans_and_fault_before_projection() {
    let registries = PlayRegistries::default();
    let clear = decode_packet(&[14, 2], context(&registries)).unwrap();
    assert_eq!(
        clear,
        PlayClientboundPacket::ClearTitles(ClearTitles { reset_times: true })
    );
    assert_eq!(encode_packet(&clear, &registries).unwrap(), [14, 1]);

    let select = decode_packet(
        &[
            85, 0xff, 12, b'f', b'e', b'r', b'r', b'i', b't', b'e', b':', b'r', b'o', b'o', b't',
        ],
        context(&registries),
    )
    .unwrap();
    assert_eq!(
        select,
        PlayClientboundPacket::SelectAdvancementsTab(SelectAdvancementsTab {
            tab: Some(id("ferrite:root")),
        })
    );

    assert!(decode_packet(&[87, 0], context(&registries)).is_err());
    assert!(decode_packet(&[85, 1, 1, b' '], context(&registries)).is_err());
    assert!(decode_packet(&[115], context(&registries)).is_err());
    let mut trailing = encode_packet(&clear, &registries).unwrap();
    trailing.push(0);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

#[test]
fn c3_action_bar_replaces_restarts_and_expires_independently() {
    let mut projection = TitleTabProjection::default();
    projection.apply_title(SetTitleText {
        text: component("title"),
    });
    projection.apply_action_bar(SetActionBarText {
        text: component("first"),
    });
    assert_eq!(projection.action_bar_remaining, 60);
    assert!(!projection.action_bar_animated_color);
    assert_eq!(projection.title_remaining, 100);
    for _ in 0..17 {
        projection.client_tick();
    }
    projection.apply_action_bar(SetActionBarText {
        text: component(""),
    });
    assert_eq!(projection.action_bar, Some(component("")));
    assert_eq!(projection.action_bar_remaining, 60);
    for _ in 0..59 {
        assert!(!projection.client_tick().action_bar_expired);
    }
    assert!(projection.client_tick().action_bar_expired);
    assert_eq!(projection.action_bar_remaining, 0);
    assert_eq!(projection.action_bar, Some(component("")));
}

#[test]
fn c3_title_timing_selectively_replaces_restarts_active_and_wraps() {
    let mut projection = TitleTabProjection::default();
    projection.apply_animation(SetTitlesAnimation {
        fade_in: 5,
        stay: -1,
        fade_out: 7,
    });
    assert_eq!(
        (projection.fade_in, projection.stay, projection.fade_out),
        (5, 70, 7)
    );
    assert_eq!(projection.title_remaining, 0);

    projection.apply_subtitle(SetSubtitleText {
        text: component("subtitle"),
    });
    projection.apply_title(SetTitleText {
        text: component("title"),
    });
    assert_eq!(projection.title_remaining, 82);
    projection.client_tick();
    projection.apply_animation(SetTitlesAnimation {
        fade_in: -1,
        stay: 3,
        fade_out: -1,
    });
    assert_eq!(projection.title_remaining, 15);

    projection.apply_clear(ClearTitles { reset_times: false });
    projection.apply_animation(SetTitlesAnimation {
        fade_in: i32::MAX,
        stay: 1,
        fade_out: 0,
    });
    assert_eq!(projection.title_remaining, 0);
    projection.apply_title(SetTitleText {
        text: component("wrapped"),
    });
    assert_eq!(projection.title_remaining, i32::MIN);
    assert_eq!(projection.title, Some(component("wrapped")));
}

#[test]
fn c3_clear_preserves_action_bar_and_optionally_resets_title_defaults() {
    let mut projection = TitleTabProjection::default();
    projection.apply_action_bar(SetActionBarText {
        text: component("action"),
    });
    projection.apply_animation(SetTitlesAnimation {
        fade_in: 1,
        stay: 2,
        fade_out: 3,
    });
    projection.apply_title(SetTitleText {
        text: component("title"),
    });
    projection.apply_subtitle(SetSubtitleText {
        text: component("subtitle"),
    });
    projection.apply_clear(ClearTitles { reset_times: false });
    assert_eq!(
        (projection.fade_in, projection.stay, projection.fade_out),
        (1, 2, 3)
    );
    assert_eq!(projection.action_bar, Some(component("action")));
    assert_eq!(projection.action_bar_remaining, 60);
    assert!(projection.title.is_none());
    assert!(projection.subtitle.is_none());

    projection.apply_clear(ClearTitles { reset_times: true });
    assert_eq!(
        (projection.fade_in, projection.stay, projection.fade_out),
        (10, 70, 20)
    );
}

#[test]
fn c3_title_expiry_clears_title_and_subtitle_without_response() {
    let mut projection = TitleTabProjection::default();
    projection.apply_animation(SetTitlesAnimation {
        fade_in: 0,
        stay: 1,
        fade_out: 0,
    });
    projection.apply_subtitle(SetSubtitleText {
        text: component("subtitle"),
    });
    projection.apply_title(SetTitleText {
        text: component("title"),
    });
    let tick = projection.client_tick();
    assert!(tick.title_expired);
    assert!(projection.title.is_none());
    assert!(projection.subtitle.is_none());
}

#[test]
fn c3_advancement_tab_correction_resolves_handler_time_object_identity_without_echo() {
    let root = id("ferrite:root");
    let mut projection = TitleTabProjection::default();
    let mut tabs = BTreeMap::new();
    let packet = SelectAdvancementsTab {
        tab: Some(root.clone()),
    };
    assert!(!projection.apply_select(&packet, &tabs));
    tabs.insert(root.clone(), AdvancementTabObject { object_token: 7 });
    assert!(projection.apply_select(&packet, &tabs));
    assert_eq!(projection.selected_tab_token(), Some(7));
    assert_eq!(projection.selected_tab_identity(), Some(&root));
    assert!(!projection.apply_select(&packet, &tabs));

    tabs.insert(root, AdvancementTabObject { object_token: 8 });
    assert!(projection.apply_select(&packet, &tabs));
    assert_eq!(projection.selected_tab_token(), Some(8));
    assert!(projection.apply_select(&SelectAdvancementsTab { tab: None }, &tabs));
    assert_eq!(projection.selected_tab_token(), None);
}

#[test]
fn c3_tab_list_flattens_fields_independently_and_retains_original_components() {
    let mut projection = TitleTabProjection::default();
    let header = TextComponentNbt::translatable("ferrite.empty_render").unwrap();
    let footer = component("styled footer");
    projection.apply_tab_list(
        TabList {
            header: header.clone(),
            footer: footer.clone(),
        },
        |component| {
            if component == &header {
                String::new()
            } else {
                "footer".to_owned()
            }
        },
    );
    assert!(projection.header.is_none());
    assert_eq!(projection.footer, Some(footer));

    projection.apply_tab_list(
        TabList {
            header: component("next"),
            footer: component(""),
        },
        |value| {
            if value == &component("") {
                String::new()
            } else {
                "next".to_owned()
            }
        },
    );
    assert_eq!(projection.header, Some(component("next")));
    assert!(projection.footer.is_none());
}

#[test]
fn c3_title_publication_targets_directly_reuses_shared_packets_and_keeps_failure_prefix() {
    let clear = publish_clear(&[3, 1, 3], true);
    assert_eq!(
        clear
            .iter()
            .map(|delivery| delivery.recipient)
            .collect::<Vec<_>>(),
        [3, 1, 3]
    );
    assert!(clear.iter().all(|delivery| delivery.packet.reset_times));

    assert_eq!(CanonicalTitleTimes::new(-1, 7, 9), None);
    let times = publish_animation(&[4, 5], CanonicalTitleTimes::new(1, 7, 9).unwrap());
    assert_eq!(
        times[0].packet,
        SetTitlesAnimation {
            fade_in: 1,
            stay: 7,
            fade_out: 9
        }
    );
    assert_eq!(times[1].packet, times[0].packet);

    let publication = publish_resolved(&[9, 8, 7], ResolvedTitleKind::Title, |recipient| {
        if recipient == 8 {
            Err("resolution failed")
        } else {
            Ok(component(&format!("player-{recipient}")))
        }
    });
    assert_eq!(publication.failure, Some("resolution failed"));
    assert_eq!(publication.deliveries.len(), 1);
    assert_eq!(publication.deliveries[0].recipient, 9);
    assert_eq!(
        publication.deliveries[0].packet,
        ResolvedTitlePacket::Title(SetTitleText {
            text: component("player-9"),
        })
    );
}

#[test]
fn c3_advancement_tab_publication_sanitizes_roots_and_sends_identity_changes_only() {
    let root = id("ferrite:root");
    let child = id("ferrite:child");
    let hidden = id("ferrite:hidden");
    let mut publisher = AdvancementTabPublisher::new([
        AdvancementTabDefinition {
            id: root.clone(),
            root: root.clone(),
            has_display: true,
        },
        AdvancementTabDefinition {
            id: child.clone(),
            root: root.clone(),
            has_display: true,
        },
        AdvancementTabDefinition {
            id: hidden.clone(),
            root: hidden.clone(),
            has_display: false,
        },
    ]);
    assert!(publisher.select(Some(&child)).is_none());
    assert!(publisher.select(Some(&hidden)).is_none());
    assert_eq!(
        publisher.select(Some(&root)),
        Some(SelectAdvancementsTab {
            tab: Some(root.clone()),
        })
    );
    assert!(publisher.select(Some(&root)).is_none());
    assert_eq!(
        publisher.select(Some(&child)),
        Some(SelectAdvancementsTab { tab: None })
    );
    assert!(publisher.selected().is_none());
}

#[test]
fn c3_title_tab_order_is_receive_ordered_and_projection_requires_a_level() {
    let mut projection = TitleTabProjection::default();
    projection.apply_animation(SetTitlesAnimation {
        fade_in: 1,
        stay: 1,
        fade_out: 1,
    });
    projection.apply_clear(ClearTitles { reset_times: false });
    projection.apply_title(SetTitleText {
        text: component("later"),
    });
    assert_eq!(projection.title_remaining, 3);
    projection.apply_clear(ClearTitles { reset_times: false });
    assert!(projection.title.is_none());

    assert_eq!(
        PlayEntryProjection::default().apply(PlayClientboundPacket::SetTitleText(SetTitleText {
            text: component("title"),
        })),
        Err(PlayProjectionError::LevelNotInstalled)
    );
}
