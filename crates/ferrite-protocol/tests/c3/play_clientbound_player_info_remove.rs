use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    CommonSpawnInfo, GameMode, PlayClientboundPacket, PlayLogin,
};
use ferrite_protocol::java_26_2::play::clientbound::player_info::{
    AddedProfile, PlayerInfoActions, PlayerInfoEntry, PlayerInfoError, PlayerInfoUpdate,
};
use ferrite_protocol::java_26_2::play::clientbound::player_info_remove::PlayerInfoRemove;
use ferrite_protocol::java_26_2::play::clientbound::player_info_remove::projection::{
    PlayerInfoRemovalEffect, PlayerInfoRemovalProjection, ProjectedPlayerInfo, SocialRelationship,
};
use ferrite_protocol::java_26_2::play::clientbound::player_info_remove::publication::{
    PlayerDepartureStep, publish_departure, respawn_replacement_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::wire::error::WireError;

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

fn login() -> PlayClientboundPacket {
    PlayClientboundPacket::Login(PlayLogin {
        player_entity_id: 1,
        hardcore: false,
        levels: BTreeSet::from([id("minecraft:overworld")]),
        max_players: 20,
        chunk_radius: 2,
        simulation_distance: 2,
        reduced_debug_info: false,
        show_death_screen: true,
        limited_crafting: false,
        spawn: CommonSpawnInfo {
            dimension_type: id("minecraft:overworld"),
            dimension: id("minecraft:overworld"),
            obfuscated_seed: 0,
            game_mode: GameMode::Survival,
            previous_game_mode: None,
            is_debug: false,
            is_flat: false,
            last_death: None,
            portal_cooldown: 0,
            sea_level: 63,
        },
        online_mode: false,
        enforces_secure_chat: false,
    })
}

fn add_player(profile_id: u128, name: &str) -> PlayClientboundPacket {
    PlayClientboundPacket::PlayerInfoUpdate(PlayerInfoUpdate {
        actions: PlayerInfoActions::from_bits(PlayerInfoActions::ADD_PLAYER),
        entries: vec![PlayerInfoEntry {
            profile_id,
            added_profile: Some(AddedProfile {
                name: name.to_owned(),
                properties: Vec::new(),
            }),
            chat_session: None,
            game_mode: None,
            listed: None,
            latency_millis: None,
            display_name: None,
            list_order: None,
            show_hat: None,
        }],
    })
}

fn info(name: &str, listed: bool, session: Option<u128>, object_token: u64) -> ProjectedPlayerInfo {
    ProjectedPlayerInfo {
        profile_name: name.to_owned(),
        listed,
        chat_session: session,
        object_token,
    }
}

fn assert_roundtrip(profile_ids: Vec<u128>) {
    let registries = PlayRegistries::default();
    let packet =
        PlayClientboundPacket::PlayerInfoRemove(Box::new(PlayerInfoRemove { profile_ids }));
    let body = encode_packet(&packet, &registries).unwrap();
    assert_eq!(decode_packet(&body, context(&registries)).unwrap(), packet);
}

#[test]
fn c3_gold_player_info_remove_locks_the_canonical_singleton_body() {
    let registries = PlayRegistries::default();
    let body = encode_packet(
        &PlayClientboundPacket::PlayerInfoRemove(Box::new(PlayerInfoRemove {
            profile_ids: vec![0],
        })),
        &registries,
    )
    .unwrap();
    assert_eq!(body, [vec![0x45, 1], vec![0; 16]].concat());
}

#[test]
fn c3_player_info_remove_codec_preserves_empty_duplicate_order_and_uuid_bits() {
    assert_roundtrip(Vec::new());
    assert_roundtrip(vec![0, u128::MAX, 7, 7, 1]);
}

#[test]
fn c3_player_info_remove_codec_faults_negative_impossible_truncated_and_residual_forms() {
    let registries = PlayRegistries::default();
    assert_eq!(
        decode_packet(&[0x45, 0xff, 0xff, 0xff, 0xff, 0x0f], context(&registries),),
        Err(PlayClientboundCodecError::PlayerInfo(
            PlayerInfoError::Wire(WireError::NegativeLength {
                field: "removed player profiles",
                value: -1,
            })
        ))
    );
    let impossible = [vec![0x45, 2], vec![0; 16]].concat();
    assert!(decode_packet(&impossible, context(&registries)).is_err());

    let mut truncated = vec![0x45, 1];
    truncated.extend([0; 15]);
    assert!(decode_packet(&truncated, context(&registries)).is_err());
    let trailing = [vec![0x45, 1], vec![0; 17]].concat();
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

#[test]
fn c3_player_info_remove_notifies_every_uuid_and_preserves_persistent_social_state() {
    let mut projection = PlayerInfoRemovalProjection::default();
    projection.install(1, info("alice", true, Some(11), 101));
    projection.install(2, info("bob", false, Some(22), 202));
    projection.set_relationship(
        1,
        SocialRelationship {
            hidden: true,
            blocked: true,
            friend: false,
        },
    );
    assert_eq!(
        projection
            .listed_objects()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [101]
    );

    let effects = projection.apply(&PlayerInfoRemove {
        profile_ids: vec![9, 1, 1, 2],
    });
    assert_eq!(
        effects,
        [
            PlayerInfoRemovalEffect {
                profile_id: 9,
                removed_object_token: None,
            },
            PlayerInfoRemovalEffect {
                profile_id: 1,
                removed_object_token: Some(101),
            },
            PlayerInfoRemovalEffect {
                profile_id: 1,
                removed_object_token: None,
            },
            PlayerInfoRemovalEffect {
                profile_id: 2,
                removed_object_token: Some(202),
            },
        ]
    );
    assert_eq!(projection.social_removals(), [9, 1, 1, 2]);
    assert!(projection.entries().is_empty());
    assert!(projection.listed_objects().is_empty());
    assert!(projection.online_names().is_empty());
    assert_eq!(projection.chat_session(1), None);
    assert_eq!(projection.discovered_names().get("alice"), Some(&1));
    assert!(projection.relationships().get(&1).unwrap().blocked);
}

#[test]
fn c3_player_info_remove_receive_order_deletes_newer_reinitialization_and_later_add_recreates() {
    let mut projection = PlayerInfoRemovalProjection::default();
    projection.install(7, info("old", true, Some(1), 1));
    projection.install(7, info("new", true, Some(2), 2));
    assert_eq!(
        projection
            .listed_objects()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [2]
    );
    let effects = projection.apply(&PlayerInfoRemove {
        profile_ids: vec![7],
    });
    assert_eq!(effects[0].removed_object_token, Some(2));
    assert!(projection.entries().is_empty());

    projection.install(7, info("latest", false, Some(3), 3));
    assert_eq!(projection.entries().get(&7).unwrap().profile_name, "latest");
    assert_eq!(projection.chat_session(7), Some(3));
    assert!(projection.online_names().contains("latest"));
}

#[test]
fn c3_player_departure_publication_is_global_singleton_and_follows_tracker_teardown() {
    let publication = publish_departure(1, &[2, 3]);
    assert_eq!(
        &publication.steps[..5],
        [
            PlayerDepartureStep::SavePlayer(1),
            PlayerDepartureStep::RemoveEntity(1),
            PlayerDepartureStep::RemoveServerMembership(1),
            PlayerDepartureStep::DisconnectPresentationServices(1),
            PlayerDepartureStep::PublishTrackerRemoval(1),
        ]
    );
    assert_eq!(
        &publication.steps[5..],
        [
            PlayerDepartureStep::PublishPlayerInfoRemoval {
                recipient: 2,
                departed: 1,
            },
            PlayerDepartureStep::PublishPlayerInfoRemoval {
                recipient: 3,
                departed: 1,
            },
        ]
    );
    assert_eq!(
        publication
            .deliveries
            .iter()
            .map(|delivery| (delivery.recipient, delivery.packet.profile_ids.as_slice()))
            .collect::<Vec<_>>(),
        [(2, &[1][..]), (3, &[1][..])]
    );
    assert_eq!(respawn_replacement_packet(), None);
}

#[test]
fn c3_player_info_remove_requires_an_installed_play_level() {
    assert_eq!(
        PlayEntryProjection::default().apply(PlayClientboundPacket::PlayerInfoRemove(Box::new(
            PlayerInfoRemove {
                profile_ids: Vec::new(),
            },
        ))),
        Err(PlayProjectionError::LevelNotInstalled)
    );

    let mut projection = PlayEntryProjection::default();
    projection.apply(login()).unwrap();
    projection.apply(add_player(9, "first")).unwrap();
    assert_eq!(projection.players().get(&9).unwrap().profile.name, "first");
    projection
        .apply(PlayClientboundPacket::PlayerInfoRemove(Box::new(
            PlayerInfoRemove {
                profile_ids: vec![9],
            },
        )))
        .unwrap();
    assert!(projection.players().is_empty());
    projection.apply(add_player(9, "second")).unwrap();
    assert_eq!(projection.players().get(&9).unwrap().profile.name, "second");
}
