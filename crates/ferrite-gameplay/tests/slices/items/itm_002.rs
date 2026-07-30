use ferrite_foundation::direction::Direction;
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::item::runtime::container_lifecycle::{
    ControlAdmission, LecternControl, MenuSlotSnapshot, RemovalState, RenameControl, anvil_rename,
    close_menu, enchantment_button, lectern_control, loom_selection, set_crafter_slot_state,
    stonecutter_selection, transfer_matching_snapshots,
};
use ferrite_gameplay::item::runtime::container_storage::{
    Barrel, ChestHalf, ChestIdentity, ChestSide, DoubleChest, EnderChestPresentation, LootCaller,
    OpenUser, PendingLoot, PlayerEnderStorage, RandomizableStorage, plan_removal_drops,
};
use ferrite_gameplay::item::runtime::dispenser::{
    DropperDispatch, DynamicDispenseComponents, EXPLICIT_BEHAVIOR_COUNT, LevelEvent,
    OptionalBehaviorState, TntAction, consume_with_remainder, dispatch_dropper, empty_dispenser,
    explicit_behavior, explicit_items, neighbor_trigger, resolve_behavior, wrapper_events,
};
use ferrite_gameplay::item::runtime::hopper::{
    HOPPER_COOLDOWN, Hopper, HopperTickGate, LooseCollectionOutcome, LooseItem,
    destination_preflight_full, loose_item_search_allowed,
};
use ferrite_gameplay::item::runtime::inventory::{
    Inventory, Slot, SlotPolicy, TransferPolicy, move_item_stack_to, select_random_occupied,
    transfer_one,
};
use ferrite_gameplay::item::runtime::menu_click::{ClickMenu, ClickPlayer, ContainerInput};
use ferrite_gameplay::item::runtime::menu_layout::{MenuKind, QuickMoveRouting};
use ferrite_gameplay::item::runtime::menu_sync::{
    ClickPacket, ClickSync, IgnoreReason, MAX_CHANGED_SLOT_HASHES, MenuActor, MenuPacketError,
    MenuSession,
};
use ferrite_gameplay::item::runtime::stack::ItemStack;
use std::collections::BTreeSet;

fn id(path: &str) -> ResourceId {
    ResourceId::minecraft(path).unwrap()
}

fn stack(identity: u64, path: &str, count: i32, maximum: i32, components: u64) -> ItemStack {
    ItemStack::new(identity, id(path), count, maximum, components)
}

fn actor() -> MenuActor {
    MenuActor {
        spectator: false,
        dead_or_dying: false,
    }
}

#[test]
fn common_transfer_uses_merge_then_empty_pass_and_rolls_back_failures() {
    let mut source = stack(1, "apple", 20, 64, 7);
    let mut restricted = Slot::with_stack(stack(2, "apple", 60, 64, 7));
    restricted.policy.may_place = false;
    let empty = Slot::empty();
    let mut slots = [restricted, empty];
    let report = move_item_stack_to(&mut source, &mut slots, 0..2, false);
    assert_eq!(report.moved, 20);
    assert_eq!(report.changed_slots, [0, 1]);
    assert_eq!(slots[0].stack.count, 64);
    assert_eq!(slots[1].stack.count, 16);
    assert!(source.is_empty());

    let mut source_inventory = Inventory::empty(1);
    source_inventory.slots[0].stack = stack(3, "carrot", 2, 64, 0);
    let mut destination = Inventory::empty(1);
    destination.slots[0].policy.may_place = false;
    assert!(!transfer_one(
        &mut source_inventory,
        0,
        &mut destination,
        &[0],
        TransferPolicy::default(),
        4
    ));
    assert_eq!(source_inventory.slots[0].stack.count, 2);
    assert_eq!(source_inventory.changed_calls, 0);
    assert_eq!(destination.changed_calls, 0);
}

#[test]
fn comparator_and_reservoir_selection_preserve_exact_arithmetic_and_draws() {
    let mut inventory = Inventory::empty(9);
    assert_eq!(inventory.comparator_output(99), 0);
    inventory.slots[0].stack = stack(10, "apple", 64, 64, 0);
    assert_eq!(inventory.comparator_output(99), 2);
    for slot in &mut inventory.slots {
        slot.stack = stack(11, "apple", 64, 64, 0);
    }
    assert_eq!(inventory.comparator_output(99), 15);
    assert_eq!(
        select_random_occupied(&inventory.slots[0..3], &[0, 1, 0]).unwrap(),
        Some(2)
    );
    assert!(select_random_occupied(&inventory.slots[0..2], &[0]).is_err());
}

#[test]
fn all_twenty_five_menu_layouts_are_closed_and_simple_routes_are_exact() {
    assert_eq!(MenuKind::ALL.len(), 25);
    assert_eq!(
        MenuKind::ALL
            .map(MenuKind::registry_path)
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len(),
        25
    );
    let six = MenuKind::Generic9x6.profile();
    assert_eq!(six.machine, 0..54);
    assert_eq!(six.player_main, 54..81);
    assert_eq!(six.hotbar, 81..90);
    assert_eq!(six.simple_quick_move_target(0).unwrap().range, 54..90);
    assert!(six.simple_quick_move_target(0).unwrap().reverse);
    assert_eq!(
        MenuKind::Hopper
            .profile()
            .simple_quick_move_target(40)
            .unwrap()
            .range,
        0..5
    );
    assert_eq!(
        MenuKind::BrewingStand.profile().routing,
        QuickMoveRouting::Brewing
    );
    assert_eq!(MenuKind::Lectern.profile().total_slots, 1);
}

#[test]
fn pickup_quick_move_swap_clone_and_throw_follow_slot_policies() {
    let layout = MenuKind::Generic3x3.profile();
    let mut slots = (0..layout.total_slots)
        .map(|_| Slot::empty())
        .collect::<Vec<_>>();
    slots[0].stack = stack(20, "apple", 9, 64, 1);
    let mut menu = ClickMenu::new(slots, 1_000);
    let mut player = ClickPlayer::empty();

    menu.clicked(0, 1, ContainerInput::Pickup, &mut player, &layout)
        .unwrap();
    assert_eq!(menu.carried.count, 5);
    assert_eq!(menu.slots[0].stack.count, 4);
    menu.clicked(1, 1, ContainerInput::Pickup, &mut player, &layout)
        .unwrap();
    assert_eq!(menu.slots[1].stack.count, 1);
    assert_eq!(menu.carried.count, 4);

    menu.carried = ItemStack::empty();
    menu.clicked(0, 0, ContainerInput::QuickMove, &mut player, &layout)
        .unwrap();
    assert!(menu.slots[0].stack.is_empty());
    assert_eq!(menu.slots[44].stack.count, 4);

    player.inventory[2] = stack(21, "carrot", 70, 99, 0);
    menu.slots[2].policy.maximum = 16;
    menu.clicked(2, 2, ContainerInput::Swap, &mut player, &layout)
        .unwrap();
    assert_eq!(menu.slots[2].stack.count, 16);
    assert_eq!(player.inventory[2].count, 54);

    menu.carried = ItemStack::empty();
    player.infinite_materials = true;
    menu.slots[3].stack = stack(22, "diamond", 1, 64, 0);
    menu.slots[3].policy.may_pickup = false;
    menu.clicked(3, 0, ContainerInput::Clone, &mut player, &layout)
        .unwrap();
    assert_eq!(menu.carried.count, 64);

    menu.carried = ItemStack::empty();
    menu.clicked(3, 0, ContainerInput::Throw, &mut player, &layout)
        .unwrap();
    assert!(menu.dropped.is_empty());
    menu.slots[3].policy.may_pickup = true;
    menu.clicked(3, 0, ContainerInput::Throw, &mut player, &layout)
        .unwrap();
    assert_eq!(menu.dropped.len(), 1);
}

#[test]
fn pickup_all_and_quick_craft_keep_two_pass_and_phase_boundaries() {
    let layout = MenuKind::Generic3x3.profile();
    let mut slots = (0..layout.total_slots)
        .map(|_| Slot::empty())
        .collect::<Vec<_>>();
    slots[1].stack = stack(30, "apple", 64, 64, 0);
    slots[2].stack = stack(31, "apple", 2, 64, 0);
    let mut menu = ClickMenu::new(slots, 2_000);
    let mut player = ClickPlayer::empty();
    menu.carried = stack(32, "apple", 1, 64, 0);
    menu.clicked(0, 0, ContainerInput::PickupAll, &mut player, &layout)
        .unwrap();
    assert_eq!(menu.carried.count, 64);
    assert_eq!(menu.slots[2].stack.count, 0);
    assert_eq!(menu.slots[1].stack.count, 3);

    menu.carried = stack(33, "carrot", 8, 64, 0);
    menu.clicked(-999, 0, ContainerInput::QuickCraft, &mut player, &layout)
        .unwrap();
    menu.clicked(3, 1, ContainerInput::QuickCraft, &mut player, &layout)
        .unwrap();
    menu.clicked(4, 1, ContainerInput::QuickCraft, &mut player, &layout)
        .unwrap();
    menu.clicked(-999, 2, ContainerInput::QuickCraft, &mut player, &layout)
        .unwrap();
    assert_eq!(menu.slots[3].stack.count, 4);
    assert_eq!(menu.slots[4].stack.count, 4);
    assert!(menu.carried.is_empty());

    menu.carried = stack(34, "carrot", 2, 64, 0);
    menu.clicked(-999, 0, ContainerInput::QuickCraft, &mut player, &layout)
        .unwrap();
    menu.clicked(5, 0, ContainerInput::Pickup, &mut player, &layout)
        .unwrap();
    assert_eq!(menu.quick_craft.status, 0);
    assert!(menu.slots[5].stack.is_empty());
}

#[test]
fn server_replays_stale_clicks_then_selects_delta_or_full_sync() {
    let layout = MenuKind::Generic3x3.profile();
    let mut slots = (0..layout.total_slots)
        .map(|_| Slot::empty())
        .collect::<Vec<_>>();
    slots[0].stack = stack(40, "apple", 1, 64, 0);
    let menu = ClickMenu::new(slots, 3_000);
    let mut session = MenuSession::new(7, menu, layout);
    let mut player = ClickPlayer::empty();
    let packet = ClickPacket {
        container_id: 7,
        state_id: 0,
        slot: 0,
        button: 0,
        input: ContainerInput::Pickup,
        changed_slot_hashes: vec![(0, 44)],
        carried_hash: 45,
    };
    let first = session
        .handle_click(packet.clone(), actor(), &mut player)
        .unwrap();
    assert_eq!(
        first,
        ClickSync::Deltas {
            slot_deltas: vec![ferrite_gameplay::item::runtime::menu_sync::SlotDelta {
                index: 0,
                state_id: 1,
            }],
            carried_changed: true,
        }
    );
    assert_eq!(session.remote_slot_hashes.get(&0), Some(&44));

    let stale = session.handle_click(packet, actor(), &mut player).unwrap();
    assert_eq!(
        stale,
        ClickSync::Full {
            state_id: 2,
            click_executed: true,
        }
    );
    assert_eq!(session.menu.slots[0].stack.count, 1);

    let wrong = ClickPacket {
        container_id: 8,
        state_id: 2,
        slot: 0,
        button: 0,
        input: ContainerInput::Pickup,
        changed_slot_hashes: vec![],
        carried_hash: 0,
    };
    assert_eq!(
        session.handle_click(wrong, actor(), &mut player).unwrap(),
        ClickSync::Ignored(IgnoreReason::WrongContainer)
    );
}

#[test]
fn menu_packet_limits_spectator_full_sync_and_state_wrap_are_locked() {
    let layout = MenuKind::Lectern.profile();
    let menu = ClickMenu::new(vec![Slot::empty()], 4_000);
    let mut session = MenuSession::new(3, menu, layout);
    session.state_id = 32_767;
    let packet = ClickPacket {
        container_id: 3,
        state_id: 32_767,
        slot: 0,
        button: 0,
        input: ContainerInput::Pickup,
        changed_slot_hashes: vec![],
        carried_hash: 0,
    };
    assert_eq!(
        session
            .handle_click(
                packet.clone(),
                MenuActor {
                    spectator: true,
                    dead_or_dying: false,
                },
                &mut ClickPlayer::empty()
            )
            .unwrap(),
        ClickSync::Full {
            state_id: 0,
            click_executed: false,
        }
    );
    let mut oversized = packet;
    oversized.changed_slot_hashes = (0..=MAX_CHANGED_SLOT_HASHES)
        .map(|index| (index as i32, 0))
        .collect();
    assert!(matches!(
        session.handle_click(oversized, actor(), &mut ClickPlayer::empty()),
        Err(MenuPacketError::TooManyChangedSlots { .. })
    ));
    assert!(session.menu.is_valid_slot_index(-2));
}

#[test]
fn close_disposition_snapshot_transfer_and_controls_are_exact() {
    let mut inventory = Inventory::empty(2);
    inventory.slots[0].stack = stack(50, "apple", 60, 64, 1);
    let mut cursor = stack(51, "apple", 10, 64, 1);
    let mut transient = [stack(52, "carrot", 2, 64, 0)];
    let closed = close_menu(
        &mut cursor,
        &mut transient,
        &mut inventory,
        RemovalState::Active,
    );
    assert!(closed.cursor_cleared);
    assert_eq!(inventory.slots[0].stack.count, 64);
    assert_eq!(inventory.slots[1].stack.count, 6);
    assert_eq!(closed.dropped[0].item, Some(id("carrot")));

    let source = vec![MenuSlotSnapshot {
        backing_container: 10,
        backing_slot: 4,
        local: stack(53, "diamond", 1, 64, 0),
        remote_hash: 99,
    }];
    let mut target = vec![MenuSlotSnapshot {
        backing_container: 10,
        backing_slot: 4,
        local: ItemStack::empty(),
        remote_hash: 0,
    }];
    assert_eq!(transfer_matching_snapshots(&source, &mut target), 1);
    assert_eq!(target[0].remote_hash, 99);

    assert!(
        ControlAdmission {
            container_matches: true,
            spectator: false,
            still_valid: false,
        }
        .crafter_slot_state()
    );
    assert!(
        !ControlAdmission {
            container_matches: true,
            spectator: false,
            still_valid: false,
        }
        .generic_button()
    );
    assert_eq!(lectern_control(100), Some(LecternControl::SetPage(0)));
    assert!(enchantment_button(2, true, 10, 3, 10, false));
    assert!(!enchantment_button(3, true, 10, 4, 10, false));
    assert!(stonecutter_selection(Some(1), 99, 3).acknowledged);
    assert_eq!(stonecutter_selection(Some(1), 99, 3).selected, None);
    assert!(!loom_selection(4, 4).acknowledged);
    assert_eq!(anvil_rename(None, ""), RenameControl::RemoveCustomName);
    assert_eq!(
        anvil_rename(None, &"x".repeat(51)),
        RenameControl::RejectedTooLong
    );

    let slots = std::array::from_fn(|_| ItemStack::empty());
    let mut disabled = [false; 9];
    assert!(set_crafter_slot_state(&slots, &mut disabled, 8, false));
    assert!(disabled[8]);
}

#[test]
fn randomizable_storage_barrel_and_removal_keep_context_and_rng_boundaries() {
    let mut storage = RandomizableStorage::<27>::empty();
    storage.pending_loot = Some(PendingLoot {
        table_fingerprint: 1,
        seed: 2,
    });
    storage.player_open(70, 1.5);
    assert_eq!(
        storage.materialized_by,
        Some(LootCaller::Player {
            player_fingerprint: 70,
            luck_bits: 1.5_f32.to_bits(),
        })
    );
    storage.pending_loot = Some(PendingLoot {
        table_fingerprint: 3,
        seed: 4,
    });
    storage.get(0);
    assert_eq!(storage.materialized_by, Some(LootCaller::NullPlayer));

    let user = OpenUser {
        spectator: false,
        interaction_range: 5.0,
        reports_open: true,
    };
    let mut barrel = Barrel::empty(Direction::North);
    assert!(barrel.start_open(false, user).opened_boundary);
    assert!(barrel.open_state);
    assert!(barrel.stop_open(false, user).closed_boundary);
    assert!(!barrel.open_state);
    let recount = barrel.recount(&[user]);
    assert!(recount.opened_boundary);
    assert!(recount.schedule_recount);
    assert!(barrel.openers.still_valid(80.9, 5.0));
    assert!(!barrel.openers.still_valid(81.0, 5.0));

    barrel.storage.inventory.slots[0].stack = stack(71, "apple", 31, 64, 0);
    let plan = plan_removal_drops(&mut barrel.storage.inventory, false, &[0, 20]).unwrap();
    assert_eq!(plan.position_double_draws, 81);
    assert_eq!(
        plan.chunks
            .iter()
            .map(|chunk| chunk.count)
            .collect::<Vec<_>>(),
        [10, 21]
    );
    assert_eq!(plan.velocity_double_draws, 12);
}

#[test]
fn double_chest_is_right_first_obstruction_aware_and_independently_powered() {
    let mut right = ChestHalf::empty(ChestIdentity::Trapped, ChestSide::Right, Direction::South);
    let mut left = ChestHalf::empty(ChestIdentity::Trapped, ChestSide::Left, Direction::South);
    right.storage.inventory.slots[0].stack = stack(80, "diamond", 1, 64, 0);
    left.storage.inventory.slots[0].stack = stack(81, "emerald", 1, 64, 0);
    right.openers.count = 20;
    let mut pair = DoubleChest::combine(right, left).unwrap();
    assert_eq!(pair.get(0, false).unwrap().item, Some(id("diamond")));
    assert_eq!(pair.get(27, false).unwrap().item, Some(id("emerald")));
    assert_eq!(pair.right.weak_signal(), 15);
    assert_eq!(pair.right.direct_signal(Direction::Up), 15);
    assert_eq!(pair.right.direct_signal(Direction::North), 0);

    pair.left.blocked = true;
    assert!(pair.get(0, false).is_none());
    assert_eq!(pair.comparator_output(), 0);
    assert_eq!(pair.get(0, true).unwrap().item, Some(id("diamond")));
    let wrong = ChestHalf::empty(ChestIdentity::Ordinary, ChestSide::Right, Direction::South);
    assert!(DoubleChest::combine(wrong, pair.left.clone()).is_err());
}

#[test]
fn ender_storage_is_player_owned_and_lid_events_are_presentation_only() {
    let mut storage = PlayerEnderStorage::empty();
    storage.load_slots(&[
        (2, stack(90, "apple", 1, 64, 0)),
        (2, stack(91, "diamond", 1, 64, 0)),
        (27, stack(92, "emerald", 1, 64, 0)),
    ]);
    assert_eq!(
        storage.storage.inventory.slots[2].stack.item,
        Some(id("diamond"))
    );
    assert_eq!(storage.saved_slots().len(), 1);
    storage.active_block_entity = Some(500);

    let mut presentation = EnderChestPresentation::new(500);
    assert!(presentation.apply_block_event(1, 1));
    for _ in 0..10 {
        presentation.animate_lid();
    }
    assert_eq!(presentation.lid_current, 1.0);
    assert_eq!(presentation.eased_lid(1.0), 1.0);
    assert!(!presentation.apply_block_event(2, 0));
    assert_eq!(storage.active_block_entity, Some(presentation.identity));
}

#[test]
fn hopper_push_pull_cooldown_rollback_and_loose_partial_absorption_are_locked() {
    let mut source = Hopper::new(Direction::East);
    let mut destination = Hopper::new(Direction::Down);
    source.inventory.slots[0].stack = stack(100, "apple", 2, 64, 0);
    source.last_ticked_game_time = 20;
    destination.last_ticked_game_time = 20;
    let mut identity = 1_000;
    assert!(source.push_to_hopper(&mut destination, &mut identity));
    assert_eq!(source.inventory.slots[0].stack.count, 1);
    assert_eq!(destination.inventory.slots[0].stack.count, 1);
    assert_eq!(destination.transfer_cooldown, 7);

    let mut above = Inventory::empty(1);
    above.slots[0].stack = stack(101, "carrot", 1, 64, 0);
    assert!(source.pull_from_inventory(&mut above, &[0], true, &mut identity));
    assert!(source.finish_transaction(true, true));
    assert_eq!(source.transfer_cooldown, HOPPER_COOLDOWN);
    assert_eq!(source.begin_tick(21), HopperTickGate::CoolingDown);

    let mut full = Inventory::empty(1);
    full.slots[0].stack = stack(102, "stone", 64, 64, 0);
    assert!(destination_preflight_full(&full, &[0]));
    let before = source.inventory.slots[0].stack.clone();
    assert!(!source.push_to_inventory(&mut full, &[0], true, &mut identity));
    assert_eq!(source.inventory.slots[0].stack, before);

    let mut collector = Hopper::new(Direction::Down);
    for slot in &mut collector.inventory.slots[0..4] {
        slot.stack = stack(103, "stone", 64, 64, 0);
    }
    collector.inventory.slots[4].stack = stack(104, "apple", 63, 64, 0);
    let mut entity = LooseItem {
        stack: stack(105, "apple", 2, 64, 0),
        discarded: false,
    };
    assert_eq!(
        collector.collect_loose_item(&mut entity),
        LooseCollectionOutcome::PartiallyAbsorbed { moved: 1 }
    );
    assert_eq!(collector.transfer_cooldown, -1);
    assert!(loose_item_search_allowed(false, true, true));
    assert!(!loose_item_search_allowed(false, true, false));
}

#[test]
fn dispenser_catalog_resolution_wrapper_order_and_sticky_failures_are_locked() {
    let items = explicit_items().collect::<Vec<_>>();
    assert_eq!(EXPLICIT_BEHAVIOR_COUNT, 80);
    assert_eq!(items.len(), 80);
    assert_eq!(items.iter().copied().collect::<BTreeSet<_>>().len(), 80);
    for item in items {
        assert!(explicit_behavior(&id(item)).is_some(), "{item}");
    }
    let dynamic = DynamicDispenseComponents {
        feature_enabled: true,
        equippable: true,
        sulfur_swallowable: true,
        spawn_egg_with_entity_data: true,
    };
    assert_eq!(
        resolve_behavior(&id("tnt"), dynamic),
        ferrite_gameplay::item::runtime::dispenser::DispenseBehavior::Tnt
    );
    assert_eq!(
        resolve_behavior(
            &id("diamond_helmet"),
            DynamicDispenseComponents {
                feature_enabled: false,
                ..dynamic
            }
        ),
        ferrite_gameplay::item::runtime::dispenser::DispenseBehavior::DefaultEjection
    );
    assert_eq!(
        wrapper_events(
            ferrite_gameplay::item::runtime::dispenser::DispenseBehavior::FilledBucket,
            false,
            Direction::East,
            1
        ),
        [
            LevelEvent::Dispense,
            LevelEvent::Animate(Direction::East),
            LevelEvent::Fail,
            LevelEvent::Animate(Direction::East),
        ]
    );

    let mut sticky = OptionalBehaviorState::default();
    assert!(!sticky.brush(false));
    assert!(!sticky.brush(true));
    assert!(!sticky.tnt(TntAction::GameRuleDisabled));
    assert!(!sticky.tnt(TntAction::SulfurCubeAccepted));
    assert!(sticky.tnt(TntAction::OrdinaryPrime));
}

#[test]
fn dispenser_remainders_dropper_targets_and_retained_trigger_are_locked() {
    let mut source = empty_dispenser();
    let selected = stack(110, "honey_bottle", 2, 64, 0);
    for slot in &mut source.slots {
        slot.stack = stack(111, "stone", 64, 64, 0);
    }
    let remainder = consume_with_remainder(
        selected,
        stack(112, "glass_bottle", 1, 64, 0),
        &mut source,
        Direction::North,
    );
    assert_eq!(remainder.selected_stack.count, 1);
    assert!(remainder.ejected_remainder.is_some());
    assert_eq!(remainder.extra_events.len(), 2);

    let mut dropper = empty_dispenser();
    dropper.slots[2].stack = stack(113, "apple", 1, 64, 0);
    dropper.slots[7].stack = stack(114, "carrot", 1, 64, 0);
    let ejected = dispatch_dropper(&mut dropper, &[0, 0], None, Direction::West, 2_000).unwrap();
    assert!(matches!(
        ejected,
        DropperDispatch::Ejected {
            selected_slot: 7,
            random_double_draws: 7,
            ..
        }
    ));

    dropper.slots[2].stack = stack(115, "apple", 1, 64, 0);
    let mut target = Inventory::empty(1);
    target.slots[0].policy = SlotPolicy {
        may_place: false,
        ..SlotPolicy::default()
    };
    let target_result = dispatch_dropper(
        &mut dropper,
        &[0],
        Some(&mut target),
        Direction::South,
        2_001,
    )
    .unwrap();
    assert_eq!(
        target_result,
        DropperDispatch::Target {
            selected_slot: 2,
            inserted: false,
        }
    );
    assert_eq!(dropper.slots[2].stack.count, 1);

    assert_eq!(neighbor_trigger(true, false).schedule_after, Some(4));
    assert_eq!(neighbor_trigger(true, false).offered_triggered, Some(true));
    assert_eq!(neighbor_trigger(false, true).offered_triggered, Some(false));
}
