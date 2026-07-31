use std::collections::BTreeSet;

use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    CommonSpawnInfo, GameMode, PlayClientboundPacket, PlayLogin,
};
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::scoreboard::packet::{
    CollisionRule, DisplaySlot, NameTagVisibility, NumberFormat, ObjectiveParameters,
    ObjectiveRenderType, ResetScore, ScoreboardPacket, SetDisplayObjective, SetObjective,
    SetPlayerTeam, SetScore, TeamColor, TeamParameters,
};
use ferrite_protocol::java_26_2::play::clientbound::scoreboard::projection::{
    NumberFormatSource, ScoreboardProjection, ScoreboardProjectionError,
};
use ferrite_protocol::java_26_2::play::clientbound::scoreboard::publication::{
    AuthoritativeTeam, ServerScoreboardPublisher,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::{NUMBER_FORMAT_TYPE, PlayRegistries};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn component(value: &str) -> TextComponentNbt {
    TextComponentNbt::literal(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(NUMBER_FORMAT_TYPE),
        vec![
            id("minecraft:blank"),
            id("minecraft:styled"),
            id("minecraft:fixed"),
        ],
    );
    registries
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn objective_parameters(name: &str) -> ObjectiveParameters {
    ObjectiveParameters {
        display_name: component(name),
        render_type: ObjectiveRenderType::Integer,
        number_format: None,
    }
}

fn team_parameters(name: &str, color: Option<TeamColor>) -> TeamParameters {
    TeamParameters {
        display_name: component(name),
        member_prefix: component("["),
        member_suffix: component("]"),
        visibility: NameTagVisibility::Always,
        collision_rule: CollisionRule::Always,
        color,
        allow_friendly_fire: true,
        see_friendly_invisibles: false,
    }
}

fn add_objective(name: &str) -> SetObjective {
    SetObjective {
        objective_name: name.to_owned(),
        method: 0,
        parameters: Some(objective_parameters(name)),
    }
}

fn score(owner: &str, objective: &str, value: i32) -> SetScore {
    SetScore {
        owner: owner.to_owned(),
        objective_name: objective.to_owned(),
        score: value,
        display: None,
        number_format: None,
    }
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

#[test]
fn c3_gold_clientbound_scoreboard_locks_all_five_empty_bodies() {
    let registries = registries();
    let cases = [
        (
            PlayClientboundPacket::ResetScore(ResetScore {
                owner: String::new(),
                objective_name: None,
            }),
            vec![79, 0, 0],
        ),
        (
            PlayClientboundPacket::SetDisplayObjective(SetDisplayObjective {
                slot: DisplaySlot::List,
                objective_name: None,
            }),
            vec![98, 0, 0],
        ),
        (
            PlayClientboundPacket::SetObjective(SetObjective {
                objective_name: String::new(),
                method: 1,
                parameters: None,
            }),
            vec![106, 0, 1],
        ),
        (
            PlayClientboundPacket::SetPlayerTeam(SetPlayerTeam {
                team_name: String::new(),
                method: 1,
                parameters: None,
                players: Vec::new(),
            }),
            vec![109, 0, 1],
        ),
        (
            PlayClientboundPacket::SetScore(SetScore {
                owner: String::new(),
                objective_name: String::new(),
                score: 0,
                display: None,
                number_format: None,
            }),
            vec![110, 0, 0, 0, 0, 0],
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
fn c3_scoreboard_codecs_roundtrip_methods_components_formats_and_signed_values() {
    let registries = registries();
    let formats = [
        NumberFormat::Blank,
        NumberFormat::Styled(component("style").network_nbt().clone()),
        NumberFormat::Fixed(component("fixed")),
    ];
    for format in formats {
        let packet = PlayClientboundPacket::SetScore(SetScore {
            owner: "owner".into(),
            objective_name: "objective".into(),
            score: i32::MIN,
            display: Some(component("shown")),
            number_format: Some(format),
        });
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }

    for method in [0, 1, 2, 3, 4, -1] {
        let packet = PlayClientboundPacket::SetPlayerTeam(SetPlayerTeam {
            team_name: "team".into(),
            method,
            parameters: matches!(method, 0 | 2)
                .then(|| team_parameters("team", Some(TeamColor::White))),
            players: matches!(method, 0 | 3 | 4)
                .then(|| vec!["one".into(), "two".into()])
                .unwrap_or_default(),
        });
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(
            decode_packet(&encoded, context(&registries)).unwrap(),
            packet
        );
    }
}

#[test]
fn c3_scoreboard_fallback_ids_and_unknown_methods_are_complete_no_field_forms() {
    assert_eq!(DisplaySlot::from_fallback_id(-1), DisplaySlot::List);
    assert_eq!(DisplaySlot::from_fallback_id(19), DisplaySlot::List);
    assert_eq!(
        DisplaySlot::from_fallback_id(18),
        DisplaySlot::SidebarTeam(TeamColor::White)
    );
    assert_eq!(TeamColor::from_fallback_id(-1), TeamColor::Black);
    assert_eq!(TeamColor::from_fallback_id(16), TeamColor::Black);
    assert_eq!(
        NameTagVisibility::from_fallback_id(4),
        NameTagVisibility::Always
    );
    assert_eq!(CollisionRule::from_fallback_id(-1), CollisionRule::Always);

    let registries = registries();
    let unknown_objective = [106, 1, b'x', 127];
    assert_eq!(
        decode_packet(&unknown_objective, context(&registries)).unwrap(),
        PlayClientboundPacket::SetObjective(SetObjective {
            objective_name: "x".into(),
            method: 127,
            parameters: None,
        })
    );
    let unknown_team = [109, 1, b't', 0xff];
    assert_eq!(
        decode_packet(&unknown_team, context(&registries)).unwrap(),
        PlayClientboundPacket::SetPlayerTeam(SetPlayerTeam {
            team_name: "t".into(),
            method: -1,
            parameters: None,
            players: Vec::new(),
        })
    );

    let team = PlayClientboundPacket::SetPlayerTeam(SetPlayerTeam {
        team_name: "t".into(),
        method: 2,
        parameters: Some(team_parameters("t", Some(TeamColor::Black))),
        players: Vec::new(),
    });
    let mut noncanonical = encode_packet(&team, &registries).unwrap();
    let length = noncanonical.len();
    noncanonical[length - 3] = 2;
    noncanonical[length - 1] = 0xff;
    let decoded = decode_packet(&noncanonical, context(&registries)).unwrap();
    let normalized = encode_packet(&decoded, &registries).unwrap();
    assert_eq!(normalized[normalized.len() - 3], 1);
    assert_eq!(normalized[normalized.len() - 1], 3);
}

#[test]
fn c3_scoreboard_codecs_fail_strict_registry_render_shape_count_and_framing() {
    let registries = registries();
    let mut objective = encode_packet(
        &PlayClientboundPacket::SetObjective(add_objective("x")),
        &registries,
    )
    .unwrap();
    let render_index = objective.len() - 2;
    objective[render_index] = 2;
    assert!(decode_packet(&objective, context(&registries)).is_err());

    let mut formatted = encode_packet(
        &PlayClientboundPacket::SetScore(SetScore {
            number_format: Some(NumberFormat::Blank),
            ..score("o", "x", 1)
        }),
        &registries,
    )
    .unwrap();
    *formatted.last_mut().unwrap() = 3;
    assert!(decode_packet(&formatted, context(&registries)).is_err());

    assert!(
        decode_packet(
            &[109, 0, 3, 0xff, 0xff, 0xff, 0xff, 0x0f],
            context(&registries)
        )
        .is_err()
    );
    assert!(decode_packet(&[79], context(&registries)).is_err());
    assert!(decode_packet(&[98, 0], context(&registries)).is_err());
    assert!(decode_packet(&[110, 0, 0, 0, 0, 0, 0], context(&registries)).is_err());
}

#[test]
fn c3_objective_score_reset_and_display_transitions_use_handler_time_lookup() {
    let mut projection = ScoreboardProjection::default();
    assert!(projection.apply_score(score("owner", "missing", 1)).warned);
    projection.apply_objective(add_objective("a")).unwrap();
    assert!(matches!(
        projection.apply_objective(add_objective("a")),
        Err(ScoreboardProjectionError::DuplicateObjective { .. })
    ));
    projection.apply_score(score("owner", "a", -7));
    projection.apply_display(SetDisplayObjective {
        slot: DisplaySlot::Sidebar,
        objective_name: Some("a".into()),
    });
    assert_eq!(projection.display_slots()[&DisplaySlot::Sidebar], "a");
    projection.apply_display(SetDisplayObjective {
        slot: DisplaySlot::Sidebar,
        objective_name: Some("missing".into()),
    });
    assert!(
        !projection
            .display_slots()
            .contains_key(&DisplaySlot::Sidebar)
    );
    assert!(
        projection
            .apply_reset(ResetScore {
                owner: "owner".into(),
                objective_name: Some("missing".into()),
            })
            .unwrap()
            .warned
    );
    projection
        .apply_objective(SetObjective {
            objective_name: "a".into(),
            method: 1,
            parameters: None,
        })
        .unwrap();
    assert!(projection.scores().is_empty());
}

#[test]
fn c3_team_membership_is_unique_and_duplicate_removal_partially_applies_then_faults() {
    let mut projection = ScoreboardProjection::default();
    for (team, members) in [("a", vec!["x", "y"]), ("b", vec!["x"])] {
        projection
            .apply_team(SetPlayerTeam {
                team_name: team.into(),
                method: 0,
                parameters: Some(team_parameters(team, None)),
                players: members.into_iter().map(str::to_owned).collect(),
            })
            .unwrap();
    }
    assert_eq!(projection.member_team("x"), Some("b"));
    assert!(!projection.teams()["a"].members.contains("x"));
    assert!(matches!(
        projection.apply_team(SetPlayerTeam {
            team_name: "b".into(),
            method: 4,
            parameters: None,
            players: vec!["x".into(), "x".into()],
        }),
        Err(ScoreboardProjectionError::InvalidTeamMemberRemoval { .. })
    ));
    assert_eq!(projection.member_team("x"), None);
    assert!(!projection.teams()["b"].members.contains("x"));
}

#[test]
fn c3_sidebar_selects_team_slot_filters_sorts_caps_and_resolves_formats() {
    let mut projection = ScoreboardProjection::default();
    let mut ordinary = objective_parameters("ordinary");
    ordinary.number_format = Some(NumberFormat::Blank);
    projection
        .apply_objective(SetObjective {
            objective_name: "ordinary".into(),
            method: 0,
            parameters: Some(ordinary),
        })
        .unwrap();
    projection.apply_objective(add_objective("red")).unwrap();
    projection
        .apply_team(SetPlayerTeam {
            team_name: "team".into(),
            method: 0,
            parameters: Some(team_parameters("team", Some(TeamColor::Red))),
            players: vec!["local".into()],
        })
        .unwrap();
    projection.apply_display(SetDisplayObjective {
        slot: DisplaySlot::Sidebar,
        objective_name: Some("ordinary".into()),
    });
    projection.apply_display(SetDisplayObjective {
        slot: DisplaySlot::SidebarTeam(TeamColor::Red),
        objective_name: Some("red".into()),
    });
    for index in 0..18 {
        let mut entry = score(&format!("owner{index:02}"), "red", index);
        if index == 17 {
            entry.display = Some(component("winner"));
            entry.number_format = Some(NumberFormat::Fixed(component("fixed")));
        }
        projection.apply_score(entry);
    }
    projection.apply_score(score("#hidden", "red", i32::MAX));
    let entries = projection.sidebar_entries("local");
    assert_eq!(entries.len(), 15);
    assert_eq!(entries[0].owner, "owner17");
    assert_eq!(entries[0].display, Some(component("winner")));
    assert_eq!(entries[0].format_source, NumberFormatSource::Entry);
    assert!(entries.iter().all(|entry| !entry.owner.starts_with('#')));

    let ordinary = projection.sidebar_entries("not-on-team");
    assert!(ordinary.is_empty());
}

#[test]
fn c3_player_list_and_below_name_use_distinct_defaults_and_hearts_bypass_formats() {
    let mut projection = ScoreboardProjection::default();
    let mut parameters = objective_parameters("health");
    parameters.render_type = ObjectiveRenderType::Hearts;
    parameters.number_format = Some(NumberFormat::Blank);
    projection
        .apply_objective(SetObjective {
            objective_name: "health".into(),
            method: 0,
            parameters: Some(parameters),
        })
        .unwrap();
    let mut entry = score("player", "health", -3);
    entry.number_format = Some(NumberFormat::Fixed(component("ignored")));
    projection.apply_score(entry);
    projection.apply_display(SetDisplayObjective {
        slot: DisplaySlot::List,
        objective_name: Some("health".into()),
    });
    projection.apply_display(SetDisplayObjective {
        slot: DisplaySlot::BelowName,
        objective_name: Some("health".into()),
    });
    let list = projection.player_list_entries(&["missing".into(), "player".into()]);
    assert_eq!(list.len(), 1);
    assert!(list[0].hearts);
    assert_eq!(list[0].number_format, None);
    assert_eq!(
        list[0].format_source,
        NumberFormatSource::YellowDecimalDefault
    );
    assert!(projection.below_name_entry("player", false).is_none());
    let below = projection.below_name_entry("player", true).unwrap();
    assert!(below.score.hearts);
    assert_eq!(
        below.score.format_source,
        NumberFormatSource::UnstyledDecimalDefault
    );
    assert_eq!(below.objective_display_name, component("health"));
}

#[test]
fn c3_objective_tracking_batches_add_slots_scores_per_recipient_and_stops_on_final_slot() {
    let mut publisher = ServerScoreboardPublisher::new(vec![1, 2]);
    publisher.define_objective("a".into(), objective_parameters("a"));
    assert!(publisher.set_score(score("owner", "a", 9)).is_empty());
    let deliveries = publisher.set_display(DisplaySlot::Sidebar, Some("a".into()));
    assert!(publisher.is_tracked("a"));
    assert_eq!(deliveries.len(), 6);
    assert!(matches!(
        deliveries[0].packet,
        ScoreboardPacket::SetObjective(_)
    ));
    assert!(matches!(
        deliveries[1].packet,
        ScoreboardPacket::SetDisplayObjective(_)
    ));
    assert!(matches!(
        deliveries[2].packet,
        ScoreboardPacket::SetScore(_)
    ));
    assert_eq!(deliveries[0].recipient, 1);
    assert_eq!(deliveries[3].recipient, 2);

    let second = publisher.set_display(DisplaySlot::BelowName, Some("a".into()));
    assert_eq!(second.len(), 2);
    let clear_one = publisher.set_display(DisplaySlot::Sidebar, None);
    assert_eq!(clear_one.len(), 2);
    assert!(publisher.is_tracked("a"));
    let clear_final = publisher.set_display(DisplaySlot::BelowName, None);
    assert_eq!(clear_final.len(), 2);
    assert!(matches!(
        clear_final[0].packet,
        ScoreboardPacket::SetObjective(SetObjective { method: 1, .. })
    ));
    assert!(!publisher.is_tracked("a"));
}

#[test]
fn c3_team_publication_is_global_single_member_and_remakes_waypoints() {
    let mut publisher = ServerScoreboardPublisher::new(vec![7, 8]);
    publisher.define_team(
        "team".into(),
        AuthoritativeTeam {
            parameters: team_parameters("team", Some(TeamColor::Blue)),
            members: BTreeSet::from(["a".into(), "b".into()]),
        },
    );
    let add = publisher.publish_team_add("team");
    assert_eq!(add.deliveries.len(), 2);
    assert_eq!(add.waypoint_remakes, ["a", "b"]);
    let member = publisher.publish_member_change("team", "a".into(), false);
    assert_eq!(member.deliveries.len(), 2);
    assert_eq!(member.waypoint_remakes, ["a"]);
    assert!(matches!(
        member.deliveries[0].packet,
        ScoreboardPacket::SetPlayerTeam(SetPlayerTeam { method: 4, ref players, .. }) if players == &["a"]
    ));
}

#[test]
fn c3_joining_projection_orders_teams_then_first_displayed_objective_batches() {
    let mut publisher = ServerScoreboardPublisher::new(Vec::new());
    publisher.define_team(
        "a-team".into(),
        AuthoritativeTeam {
            parameters: team_parameters("a-team", None),
            members: BTreeSet::new(),
        },
    );
    publisher.define_objective("objective".into(), objective_parameters("objective"));
    publisher.set_display(DisplaySlot::Sidebar, Some("objective".into()));
    publisher.set_display(DisplaySlot::BelowName, Some("objective".into()));
    let packets = publisher.joining_packets();
    assert!(matches!(packets[0], ScoreboardPacket::SetPlayerTeam(_)));
    assert!(matches!(packets[1], ScoreboardPacket::SetObjective(_)));
    assert!(matches!(
        packets[2],
        ScoreboardPacket::SetDisplayObjective(_)
    ));
    assert!(matches!(
        packets[3],
        ScoreboardPacket::SetDisplayObjective(_)
    ));
}

#[test]
fn c3_scoreboard_projection_requires_an_installed_play_level() {
    assert_eq!(
        PlayEntryProjection::default()
            .apply(PlayClientboundPacket::SetObjective(add_objective("a"))),
        Err(PlayProjectionError::LevelNotInstalled)
    );
    let mut projection = PlayEntryProjection::default();
    projection.apply(login()).unwrap();
    projection
        .apply(PlayClientboundPacket::SetObjective(add_objective("a")))
        .unwrap();
    projection
        .apply(PlayClientboundPacket::SetScore(score("owner", "a", 3)))
        .unwrap();
    assert_eq!(
        projection.scoreboard().scores()[&("owner".into(), "a".into())].value,
        3
    );
}
