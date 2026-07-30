use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_gameplay::player::interaction::attack::{
    AttackTargetKind, ClientAttackContext, ClientAttackEffect, ServerAttackContext,
    ServerAttackOutcome, admit_server_attack, plan_client_attack,
};
use ferrite_gameplay::player::interaction::targeting::{
    PickCandidate, PickRanges, select_pick, select_with_attack_range,
};
use ferrite_gameplay::player::interaction::use_action::{
    ClientUseContext, ClientUseStop, HandUseInput, PredictionSequence, ServerAirUseContext,
    ServerAirUseResult, ServerBlockAdmission, ServerBlockAdmissionResult, ServerEntityAdmission,
    ServerUseEffect, UseEffect, converge_server_air_use, plan_client_use, plan_server_air_use,
    plan_server_block_use, plan_server_entity_use,
};
use ferrite_gameplay::player::interaction::{
    BlockHit, EntityHit, Hand, HitTarget, InteractionResult, ItemContext, StackMutation,
    StackState, SwingSource,
};
use ferrite_gameplay::player::state::Vec3;

fn stack(object_id: u64) -> StackState {
    StackState {
        object_id,
        item_id: 10,
        count: 1,
        damage: 0,
        use_duration: 0,
        feature_enabled: true,
        on_cooldown: false,
    }
}

fn success(swing: SwingSource, item: ItemContext) -> InteractionResult {
    InteractionResult::Success { swing, item }
}

fn block_hit() -> BlockHit {
    BlockHit {
        position: BlockPos::new(0, 64, 0),
        location: Vec3::new(0.5, 64.5, 0.5),
        face: Direction::Up,
    }
}

fn entity_hit() -> EntityHit {
    EntityHit {
        entity_id: 42,
        location: Vec3::new(3.0, 65.0, 0.0),
        relative_location: Vec3::new(0.0, 1.0, 0.0),
    }
}

#[test]
fn client_pick_keeps_block_ties_and_strictly_filters_each_target_range() {
    let block = PickCandidate {
        target: HitTarget::Block(block_hit()),
        distance_squared: 9.0,
    };
    let tied_entity = PickCandidate {
        target: HitTarget::Entity(entity_hit()),
        distance_squared: 9.0,
    };
    assert!(matches!(
        select_pick(
            Vec3::ZERO,
            Some(block),
            &[tied_entity],
            PickRanges {
                block: 4.5,
                entity: 3.0,
            },
        ),
        HitTarget::Block(_)
    ));
    assert!(matches!(
        select_pick(
            Vec3::ZERO,
            None,
            &[tied_entity],
            PickRanges {
                block: 4.5,
                entity: 3.0,
            },
        ),
        HitTarget::Miss { .. }
    ));

    let out_of_block_range = PickCandidate {
        target: HitTarget::Block(block_hit()),
        distance_squared: 4.5_f64.powi(2),
    };
    assert_eq!(
        select_with_attack_range(
            Some(out_of_block_range),
            HitTarget::Entity(entity_hit()),
            4.5,
        ),
        HitTarget::Entity(entity_hit())
    );
}

fn attack_context(hit: HitTarget) -> ClientAttackContext {
    ClientAttackContext {
        miss_delay_remaining: 0,
        hit: Some(hit),
        hands_busy: false,
        item_feature_enabled: true,
        cannot_attack_with_item: false,
        spectator: false,
        piercing_weapon: false,
        custom_range_present: false,
        custom_range_admits_hit: true,
        block_is_air: false,
        block_became_air_during_start: false,
        game_mode_uses_miss_time: true,
    }
}

#[test]
fn attack_paths_preserve_spectator_piercing_custom_miss_and_instant_block_branches() {
    let custom_reject = plan_client_attack(ClientAttackContext {
        custom_range_present: true,
        custom_range_admits_hit: false,
        ..attack_context(HitTarget::Entity(entity_hit()))
    });
    assert_eq!(
        custom_reject.effects,
        vec![ClientAttackEffect::Swing(Hand::Main)]
    );

    let spectator = plan_client_attack(ClientAttackContext {
        spectator: true,
        ..attack_context(HitTarget::Entity(entity_hit()))
    });
    assert_eq!(
        spectator.effects,
        vec![ClientAttackEffect::Spectate(entity_hit().entity_id)]
    );

    let piercing = plan_client_attack(ClientAttackContext {
        piercing_weapon: true,
        ..attack_context(HitTarget::Entity(entity_hit()))
    });
    assert_eq!(
        piercing.effects,
        vec![
            ClientAttackEffect::PiercingAttack,
            ClientAttackEffect::Swing(Hand::Main)
        ]
    );

    let miss = plan_client_attack(attack_context(HitTarget::Miss {
        location: Vec3::ZERO,
    }));
    assert_eq!(
        miss.effects,
        vec![
            ClientAttackEffect::InstallMissDelay(10),
            ClientAttackEffect::ResetAttackStrength,
            ClientAttackEffect::Swing(Hand::Main)
        ]
    );

    let instant = plan_client_attack(ClientAttackContext {
        block_became_air_during_start: true,
        ..attack_context(HitTarget::Block(block_hit()))
    });
    assert!(instant.instant_block_attack);
    assert_eq!(
        instant.effects,
        vec![
            ClientAttackEffect::StartBlockBreak,
            ClientAttackEffect::InstantBlockAttack,
            ClientAttackEffect::Swing(Hand::Main)
        ]
    );
}

#[test]
fn server_attack_uses_plus_three_strict_range_and_disconnects_invalid_targets() {
    let base = ServerAttackContext {
        target_current: true,
        inside_world_border: true,
        distance_to_bounds_squared: 64.0,
        attack_range: 5.0,
        target_kind: AttackTargetKind::Ordinary,
        item_feature_enabled: true,
        cannot_attack_with_item: false,
    };
    assert_eq!(admit_server_attack(base), ServerAttackOutcome::Ignored);
    assert_eq!(
        admit_server_attack(ServerAttackContext {
            distance_to_bounds_squared: 63.999_999,
            ..base
        }),
        ServerAttackOutcome::Attack
    );
    assert_eq!(
        admit_server_attack(ServerAttackContext {
            distance_to_bounds_squared: 1.0,
            target_kind: AttackTargetKind::ExperienceOrb,
            ..base
        }),
        ServerAttackOutcome::DisconnectInvalidTarget
    );
}

#[test]
fn entity_pass_falls_into_same_hand_air_then_offhand_and_disabled_stack_aborts_all() {
    let main = stack(1);
    let off = stack(2);
    let plan = plan_client_use(ClientUseContext {
        destroying: false,
        hands_busy: false,
        spectator: false,
        infinite_materials: false,
        secondary_use: false,
        target_inside_border: true,
        entity_in_strict_range: true,
        target: HitTarget::Entity(entity_hit()),
        hands: [
            HandUseInput {
                stack: main,
                entity_result: InteractionResult::Fail,
                air_result: InteractionResult::Pass,
                ..HandUseInput::default()
            },
            HandUseInput {
                stack: off,
                entity_result: InteractionResult::Pass,
                air_result: success(SwingSource::Client, ItemContext::None),
                ..HandUseInput::default()
            },
        ],
    });
    assert_eq!(plan.stop, ClientUseStop::Success);
    assert!(plan.effects.contains(&UseEffect::SendUseInAir(Hand::Main)));
    assert!(plan.effects.contains(&UseEffect::SendUseInAir(Hand::Off)));
    assert_eq!(plan.effects.last(), Some(&UseEffect::Swing(Hand::Off)));

    let disabled = plan_client_use(ClientUseContext {
        hands: [
            HandUseInput {
                stack: StackState {
                    feature_enabled: false,
                    ..main
                },
                ..HandUseInput::default()
            },
            HandUseInput::default(),
        ],
        ..ClientUseContext {
            destroying: false,
            hands_busy: false,
            spectator: false,
            infinite_materials: false,
            secondary_use: false,
            target_inside_border: true,
            entity_in_strict_range: true,
            target: HitTarget::Entity(entity_hit()),
            hands: [HandUseInput::default(), HandUseInput::default()],
        }
    });
    assert_eq!(disabled.stop, ClientUseStop::FeatureDisabled);
    assert_eq!(disabled.effects, vec![UseEffect::SetRightClickDelay(4)]);
}

#[test]
fn infinite_material_entity_restore_requires_same_stack_object_and_lower_count() {
    let held = StackState {
        count: 4,
        ..stack(1)
    };
    let common = ClientUseContext {
        destroying: false,
        hands_busy: false,
        spectator: false,
        infinite_materials: true,
        secondary_use: false,
        target_inside_border: true,
        entity_in_strict_range: true,
        target: HitTarget::Entity(entity_hit()),
        hands: [HandUseInput::default(), HandUseInput::default()],
    };
    let restored = plan_client_use(ClientUseContext {
        hands: [
            HandUseInput {
                stack: held,
                entity_stack_after: StackState { count: 3, ..held },
                entity_result: success(SwingSource::None, ItemContext::None),
                ..HandUseInput::default()
            },
            HandUseInput::default(),
        ],
        ..common
    });
    assert!(restored.effects.contains(&UseEffect::MutateStack {
        hand: Hand::Main,
        mutation: StackMutation::RestoreCount(4),
    }));

    let replaced = plan_client_use(ClientUseContext {
        hands: [
            HandUseInput {
                stack: held,
                entity_stack_after: StackState {
                    object_id: 9,
                    count: 3,
                    ..held
                },
                entity_result: success(SwingSource::None, ItemContext::None),
                ..HandUseInput::default()
            },
            HandUseInput::default(),
        ],
        ..common
    });
    assert!(!replaced.effects.iter().any(|effect| {
        matches!(
            effect,
            UseEffect::MutateStack {
                mutation: StackMutation::RestoreCount(_),
                ..
            }
        )
    }));
}

#[test]
fn block_try_empty_hand_marker_and_fail_terminal_keep_callback_order() {
    let main = stack(1);
    let transformed = StackState {
        object_id: 3,
        item_id: 11,
        count: 1,
        ..main
    };
    let plan = plan_client_use(ClientUseContext {
        destroying: false,
        hands_busy: false,
        spectator: false,
        infinite_materials: false,
        secondary_use: false,
        target_inside_border: true,
        entity_in_strict_range: false,
        target: HitTarget::Block(block_hit()),
        hands: [
            HandUseInput {
                stack: main,
                block_result: InteractionResult::TryEmptyHandInteraction,
                empty_hand_result: InteractionResult::Pass,
                use_on_result: success(
                    SwingSource::Client,
                    ItemContext::ItemUsed {
                        transformed: Some(transformed),
                    },
                ),
                ..HandUseInput::default()
            },
            HandUseInput::default(),
        ],
    });
    let block = block_hit().position;
    assert_eq!(plan.stop, ClientUseStop::Success);
    assert!(plan.effects.windows(3).any(|window| {
        window
            == [
                UseEffect::BlockItemCallback {
                    position: block,
                    hand: Hand::Main,
                },
                UseEffect::EmptyHandCallback { position: block },
                UseEffect::UseOnCallback {
                    position: block,
                    hand: Hand::Main,
                },
            ]
    }));
    assert!(plan.effects.contains(&UseEffect::MutateStack {
        hand: Hand::Main,
        mutation: StackMutation::Replace(transformed),
    }));

    let failed = plan_client_use(ClientUseContext {
        hands: [
            HandUseInput {
                stack: main,
                use_on_result: InteractionResult::Fail,
                ..HandUseInput::default()
            },
            HandUseInput {
                stack: stack(2),
                air_result: success(SwingSource::Client, ItemContext::None),
                ..HandUseInput::default()
            },
        ],
        ..ClientUseContext {
            destroying: false,
            hands_busy: false,
            spectator: false,
            infinite_materials: false,
            secondary_use: true,
            target_inside_border: true,
            entity_in_strict_range: false,
            target: HitTarget::Block(block_hit()),
            hands: [HandUseInput::default(), HandUseInput::default()],
        }
    });
    assert_eq!(failed.stop, ClientUseStop::BlockFail);
    assert!(!failed.effects.contains(&UseEffect::SendUseInAir(Hand::Off)));
}

fn server_block_admission() -> ServerBlockAdmission {
    ServerBlockAdmission {
        sequence: 7,
        client_loaded: true,
        hand_feature_enabled: true,
        eye: Vec3::new(0.5, 65.5, 5.999_999),
        target: block_hit().position,
        interaction_range: 4.5,
        offset_x: 0.5,
        offset_y: 0.5,
        offset_z: 0.5,
        within_build_height: true,
        spawn_protected: false,
        teleport_pending: false,
        may_interact: true,
    }
}

#[test]
fn prediction_and_server_block_entity_admission_preserve_ack_and_swing_order() {
    let mut sequences = PredictionSequence::default();
    assert_eq!(sequences.acknowledgement(), -1);
    assert!(sequences.register(3));
    assert!(sequences.register(9));
    assert!(sequences.register(4));
    assert_eq!(sequences.acknowledgement(), 9);

    let result = success(
        SwingSource::Server,
        ItemContext::ItemUsed { transformed: None },
    );
    let block_plan = plan_server_block_use(server_block_admission(), Hand::Main, result);
    assert_eq!(block_plan.admission, ServerBlockAdmissionResult::Invoke);
    assert_eq!(
        block_plan.effects,
        vec![
            ServerUseEffect::AcknowledgeSequence(7),
            ServerUseEffect::InvokeBlockTransaction,
            ServerUseEffect::TriggerBlockCriterion,
            ServerUseEffect::ServerSwing(Hand::Main),
            ServerUseEffect::UpdateTargetBlock(block_hit().position),
            ServerUseEffect::UpdateHitFaceNeighbor,
        ]
    );

    let entity_effects = plan_server_entity_use(
        ServerEntityAdmission {
            client_loaded: true,
            target_current: true,
            inside_world_border: true,
            distance_to_bounds_squared: 63.999,
            interaction_range: 5.0,
            hand_feature_enabled: true,
        },
        entity_hit().entity_id,
        Hand::Off,
        result,
    );
    assert_eq!(
        entity_effects,
        vec![
            ServerUseEffect::InstallSecondaryAction,
            ServerUseEffect::InvokeEntityTransaction {
                entity_id: entity_hit().entity_id,
                hand: Hand::Off,
            },
            ServerUseEffect::TriggerEntityCriterion,
            ServerUseEffect::ServerSwing(Hand::Off),
        ]
    );
}

#[test]
fn server_air_stack_identity_duration_and_transformation_control_resync() {
    let before = stack(1);
    assert_eq!(
        converge_server_air_use(before, before, InteractionResult::Pass, false, false),
        ServerAirUseResult {
            mutation: StackMutation::Retain,
            resync_inventory: false,
        }
    );
    let transformed = StackState {
        object_id: 2,
        item_id: 12,
        count: 1,
        damage: 4,
        ..before
    };
    let effects = plan_server_air_use(ServerAirUseContext {
        sequence: 12,
        client_loaded: true,
        hand: Hand::Main,
        hand_feature_enabled: true,
        before,
        current: transformed,
        result: success(
            SwingSource::None,
            ItemContext::ItemUsed {
                transformed: Some(transformed),
            },
        ),
        began_using: false,
        actively_using: false,
    });
    assert_eq!(
        effects,
        vec![
            ServerUseEffect::AcknowledgeSequence(12),
            ServerUseEffect::ApplyPacketRotation,
            ServerUseEffect::InvokeAirTransaction(Hand::Main),
            ServerUseEffect::MutateStack(StackMutation::Replace(transformed)),
            ServerUseEffect::ResyncInventory,
        ]
    );
}
