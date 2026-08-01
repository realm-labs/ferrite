use std::cell::Cell;

use behavior_runner::client::feedback::{
    CommandBlockFeedback, FeedbackDelivery, FeedbackDestination, FeedbackRouter, FeedbackRules,
    FeedbackSource, FeedbackTrace, PlayerAudience, TextColor, TextComponent, TextStyle,
};

fn literal(text: &str) -> TextComponent {
    TextComponent::literal(text)
}

fn players() -> Vec<PlayerAudience> {
    vec![
        PlayerAudience {
            id: 1,
            display_name: literal("source"),
            operator: true,
        },
        PlayerAudience {
            id: 2,
            display_name: literal("first-op"),
            operator: true,
        },
        PlayerAudience {
            id: 3,
            display_name: literal("ordinary"),
            operator: false,
        },
        PlayerAudience {
            id: 4,
            display_name: literal("second-op"),
            operator: true,
        },
    ]
}

fn router(rules: FeedbackRules) -> FeedbackRouter {
    FeedbackRouter {
        rules,
        players: players(),
        timestamp: "12:34:56".to_owned(),
        trace: FeedbackTrace::default(),
    }
}

fn all_rules(value: bool) -> FeedbackRules {
    FeedbackRules {
        command_block_output: value,
        send_command_feedback: value,
        log_admin_commands: value,
    }
}

fn admin(source: &str, message: &str) -> TextComponent {
    TextComponent::translatable("chat.type.admin", vec![literal(source), literal(message)])
        .styled(TextColor::Gray, true)
}

#[test]
fn success_is_lazy_once_and_routes_direct_before_ordered_op_and_log_copies() {
    let evaluations = Cell::new(0);
    let mut source = FeedbackSource::Player {
        id: 1,
        display_name: literal("source"),
    };
    let mut route = router(all_rules(true));
    route.send_success(&mut source, false, true, || {
        evaluations.set(evaluations.get() + 1);
        literal("done")
    });

    assert_eq!(evaluations.get(), 1);
    assert_eq!(
        route.trace.deliveries,
        vec![
            FeedbackDelivery {
                destination: FeedbackDestination::Player(1),
                component: literal("done"),
            },
            FeedbackDelivery {
                destination: FeedbackDestination::Player(2),
                component: admin("source", "done"),
            },
            FeedbackDelivery {
                destination: FeedbackDestination::Player(4),
                component: admin("source", "done"),
            },
            FeedbackDelivery {
                destination: FeedbackDestination::ServerLog,
                component: admin("source", "done"),
            },
        ]
    );
}

#[test]
fn silent_and_inert_sources_do_not_evaluate_success_and_failure_is_direct_red_only() {
    let evaluations = Cell::new(0);
    let mut silent_player = FeedbackSource::Player {
        id: 1,
        display_name: literal("source"),
    };
    let mut route = router(all_rules(true));
    route.send_success(&mut silent_player, true, true, || {
        evaluations.set(evaluations.get() + 1);
        literal("hidden")
    });
    route.send_failure(&mut silent_player, true, literal("hidden failure"));
    let mut null = FeedbackSource::Null;
    route.send_success(&mut null, false, true, || {
        evaluations.set(evaluations.get() + 1);
        literal("also hidden")
    });

    assert_eq!(evaluations.get(), 0);
    assert!(route.trace.deliveries.is_empty());

    route.send_failure(&mut silent_player, false, literal("bad"));
    assert_eq!(
        route.trace.deliveries,
        vec![FeedbackDelivery {
            destination: FeedbackDestination::Player(1),
            component: TextComponent::Styled {
                content: Box::new(TextComponent::Sequence(vec![literal(""), literal("bad")])),
                style: TextStyle {
                    color: Some(TextColor::Red),
                    italic: false,
                },
            },
        }]
    );
}

#[test]
fn three_rules_and_dedicated_properties_gate_independent_routes() {
    for command_block_output in [false, true] {
        for send_command_feedback in [false, true] {
            for log_admin_commands in [false, true] {
                let rules = FeedbackRules {
                    command_block_output,
                    send_command_feedback,
                    log_admin_commands,
                };
                let mut route = router(rules);
                let mut source = FeedbackSource::CommandBlock {
                    display_name: literal("@"),
                    feedback: CommandBlockFeedback::default(),
                };
                route.send_success(&mut source, false, true, || literal("result"));

                let direct = matches!(
                    &source,
                    FeedbackSource::CommandBlock { feedback, .. }
                        if feedback.last_output.is_some()
                );
                let op_count = route
                    .trace
                    .deliveries
                    .iter()
                    .filter(|delivery| {
                        matches!(delivery.destination, FeedbackDestination::Player(_))
                    })
                    .count();
                let logged = route
                    .trace
                    .deliveries
                    .iter()
                    .any(|delivery| delivery.destination == FeedbackDestination::ServerLog);
                assert_eq!(direct, send_command_feedback);
                assert_eq!(
                    op_count,
                    usize::from(command_block_output && send_command_feedback) * 3
                );
                assert_eq!(logged, command_block_output && log_admin_commands);
            }
        }
    }

    for (source, should_broadcast) in [("console", false), ("rcon", true)] {
        let mut route = router(all_rules(true));
        let mut source = match source {
            "console" => FeedbackSource::Server {
                display_name: literal("Server"),
                inform_admins: should_broadcast,
            },
            _ => FeedbackSource::Rcon {
                display_name: literal("Rcon"),
                inform_admins: should_broadcast,
                buffer: String::new(),
            },
        };
        route.send_success(&mut source, false, true, || literal("ok"));
        assert_eq!(
            route
                .trace
                .deliveries
                .iter()
                .filter(|delivery| {
                    matches!(delivery.destination, FeedbackDestination::Player(_))
                })
                .count(),
            usize::from(should_broadcast) * 3
        );
    }
}

#[test]
fn console_and_rcon_preserve_their_distinct_direct_and_admin_channels() {
    let mut route = router(all_rules(true));
    let mut console = FeedbackSource::Server {
        display_name: literal("Server"),
        inform_admins: true,
    };
    route.send_success(&mut console, false, true, || literal("console result"));
    assert_eq!(
        route.trace.deliveries[0].destination,
        FeedbackDestination::ServerLog
    );
    assert_eq!(route.trace.deliveries.len(), 4);

    let mut route = router(all_rules(true));
    let mut rcon = FeedbackSource::Rcon {
        display_name: literal("Rcon"),
        inform_admins: true,
        buffer: String::new(),
    };
    route.send_success(&mut rcon, false, true, || literal("first"));
    route.send_failure(&mut rcon, false, literal("second"));
    let FeedbackSource::Rcon { buffer, .. } = rcon else {
        unreachable!();
    };
    assert_eq!(buffer, "firstsecond");
    assert_eq!(route.trace.deliveries.len(), 4);
}

#[test]
fn command_block_output_is_timestamped_updated_and_rejected_after_close() {
    let mut route = router(all_rules(true));
    let mut source = FeedbackSource::CommandBlock {
        display_name: literal("@"),
        feedback: CommandBlockFeedback::default(),
    };
    route.send_success(&mut source, false, false, || literal("first"));
    route.send_failure(&mut source, false, literal("failure"));
    let FeedbackSource::CommandBlock { feedback, .. } = &mut source else {
        unreachable!();
    };
    assert_eq!(feedback.update_count, 2);
    assert_eq!(
        feedback.last_output,
        Some(TextComponent::Sequence(vec![
            literal("[12:34:56] "),
            TextComponent::Sequence(vec![literal(""), literal("failure")])
                .styled(TextColor::Red, false),
        ]))
    );
    feedback.close();
    route.send_failure(&mut source, false, literal("late"));
    let FeedbackSource::CommandBlock { feedback, .. } = source else {
        unreachable!();
    };
    assert_eq!(feedback.update_count, 2);
}

#[test]
fn complete_source_rule_silence_broadcast_matrix_matches_all_route_gates() {
    const CASES: usize = 9;
    let mut checked = 0;
    for case in 0..CASES {
        for command_block_output in [false, true] {
            for send_command_feedback in [false, true] {
                for log_admin_commands in [false, true] {
                    for silent in [false, true] {
                        for broadcast in [false, true] {
                            let rules = FeedbackRules {
                                command_block_output,
                                send_command_feedback,
                                log_admin_commands,
                            };
                            let mut route = router(rules);
                            let mut source = matrix_source(case);
                            let evaluations = Cell::new(0);
                            route.send_success(&mut source, silent, broadcast, || {
                                evaluations.set(evaluations.get() + 1);
                                literal("matrix")
                            });

                            let accepts_direct = match case {
                                0 | 5 => send_command_feedback,
                                1..=4 => true,
                                _ => false,
                            } && !silent;
                            let informs = match case {
                                0 | 2 | 4 => true,
                                5 => command_block_output,
                                _ => false,
                            } && !silent
                                && broadcast;
                            assert_eq!(evaluations.get(), usize::from(accepts_direct || informs));

                            let direct_trace = usize::from(accepts_direct && matches!(case, 0..=2));
                            let operator_count = if case == 0 { 2 } else { 3 };
                            let operator_trace =
                                usize::from(informs && send_command_feedback) * operator_count;
                            let admin_log = usize::from(
                                informs && !matches!(case, 1 | 2) && log_admin_commands,
                            );
                            assert_eq!(
                                route.trace.deliveries.len(),
                                direct_trace + operator_trace + admin_log,
                                "route count diverged for case {case} and rules {rules:?}"
                            );
                            checked += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(checked, 288);
}

fn matrix_source(case: usize) -> FeedbackSource {
    match case {
        0 => FeedbackSource::Player {
            id: 1,
            display_name: literal("source"),
        },
        1 | 2 => FeedbackSource::Server {
            display_name: literal("Server"),
            inform_admins: case == 2,
        },
        3 | 4 => FeedbackSource::Rcon {
            display_name: literal("Rcon"),
            inform_admins: case == 4,
            buffer: String::new(),
        },
        5 => FeedbackSource::CommandBlock {
            display_name: literal("@"),
            feedback: CommandBlockFeedback::default(),
        },
        6 => {
            let mut feedback = CommandBlockFeedback::default();
            feedback.track_output = false;
            FeedbackSource::CommandBlock {
                display_name: literal("@"),
                feedback,
            }
        }
        7 => {
            let mut feedback = CommandBlockFeedback::default();
            feedback.close();
            FeedbackSource::CommandBlock {
                display_name: literal("@"),
                feedback,
            }
        }
        _ => FeedbackSource::Null,
    }
}

#[test]
fn gamemode_feedback_splits_self_target_and_source_and_skips_no_change() {
    let mut route = router(all_rules(true));
    let mut source = FeedbackSource::Player {
        id: 1,
        display_name: literal("source"),
    };
    assert!(route.route_gamemode_change(&mut source, false, 1, literal("creative"), true));
    assert!(route.route_gamemode_change(&mut source, false, 3, literal("survival"), true));
    let count = route.trace.deliveries.len();
    assert!(!route.route_gamemode_change(&mut source, false, 3, literal("adventure"), false));
    assert_eq!(route.trace.deliveries.len(), count);
    assert!(route.trace.deliveries.iter().any(|delivery| {
        delivery.destination == FeedbackDestination::Player(3)
            && delivery.component
                == TextComponent::translatable("gameMode.changed", vec![literal("survival")])
    }));

    route.rules.send_command_feedback = false;
    let before = route.trace.deliveries.len();
    assert!(route.route_gamemode_change(&mut source, true, 3, literal("creative"), true));
    assert_eq!(route.trace.deliveries.len(), before);
}

#[test]
fn placement_snapshots_feedback_only_without_block_entity_data() {
    for send_command_feedback in [false, true] {
        for block_automatic in [false, true] {
            let mut route = router(FeedbackRules {
                command_block_output: true,
                send_command_feedback,
                log_admin_commands: true,
            });
            let mut feedback = CommandBlockFeedback::default();
            feedback.track_output = !send_command_feedback;
            feedback.automatic = !block_automatic;
            route.apply_command_block_placement(&mut feedback, block_automatic, false, true);
            assert_eq!(feedback.track_output, send_command_feedback);
            assert_eq!(feedback.automatic, block_automatic);
            assert!(feedback.powered);
            assert_eq!(feedback.power_update_count, 1);

            let preserved = feedback.clone();
            route.rules.send_command_feedback = !send_command_feedback;
            route.apply_command_block_placement(&mut feedback, !block_automatic, true, false);
            assert_eq!(feedback.track_output, preserved.track_output);
            assert_eq!(feedback.automatic, preserved.automatic);
            assert!(!feedback.powered);
            assert_eq!(feedback.power_update_count, 2);
        }
    }
}
