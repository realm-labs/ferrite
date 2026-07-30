use ferrite_foundation::direction::Direction;
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::item::runtime::bookshelf::{
    BookshelfUse, CHISELED_BOOKSHELF_SLOTS, ChiseledBookshelf, hit_slot, use_empty_hand,
    use_with_item,
};
use ferrite_gameplay::item::runtime::jukebox::{
    Jukebox, JukeboxItem, JukeboxSong, JukeboxUse, default_song_for_item, use_admission,
};
use ferrite_gameplay::item::runtime::stack::{
    AfterUseStep, ItemStack, UseCooldown, UseRemainder, apply_after_use,
};
use ferrite_gameplay::item::runtime::use_lifecycle::{
    ActiveUse, ConsumableUse, Hand, TerminalUse, TickOutcome, UseComponents, UseDispatch,
    UseProfile, dispatch, seconds_to_ticks,
};

fn id(path: &str) -> ResourceId {
    ResourceId::minecraft(path).unwrap()
}

fn stack(identity: u64, path: &str, count: i32, components: u64) -> ItemStack {
    ItemStack::new(identity, id(path), count, 64, components)
}

fn disc(identity: u64, path: &str) -> JukeboxItem {
    JukeboxItem::from_default_disc(ItemStack::new(identity, id(path), 1, 1, 0))
}

#[test]
fn item_stack_normalization_and_after_use_order_are_locked() {
    assert_eq!(stack(1, "apple", 0, 4), ItemStack::empty());
    assert_eq!(stack(1, "apple", 80, 4).count, 64);

    let before = stack(10, "honey_bottle", 1, 7);
    let returned = ItemStack::empty();
    let remainder = UseRemainder {
        stack: stack(11, "glass_bottle", 1, 0),
    };
    let cooldown = UseCooldown {
        ticks: 20,
        group_fingerprint: Some(99),
    };
    let outcome = apply_after_use(&before, returned, Some(&remainder), Some(cooldown));
    assert_eq!(outcome.hand, remainder.stack);
    assert_eq!(
        outcome.order,
        [AfterUseStep::Remainder, AfterUseStep::Cooldown]
    );
    assert_eq!(outcome.cooldown, Some(cooldown));
    assert!(outcome.installed_returned_object);

    let retained = stack(10, "honey_bottle", 1, 7);
    let unchanged = apply_after_use(&before, retained, Some(&remainder), None);
    assert!(unchanged.extra_remainder.is_none());
    assert!(unchanged.order.is_empty());
    assert!(!unchanged.installed_returned_object);

    let before_two = stack(20, "suspicious_stew", 2, 3);
    let returned_one = stack(20, "suspicious_stew", 1, 3);
    let nonempty = apply_after_use(&before_two, returned_one, Some(&remainder), None);
    assert_eq!(nonempty.hand.item, Some(id("suspicious_stew")));
    assert_eq!(nonempty.extra_remainder, Some(remainder.stack));
}

#[test]
fn component_dispatch_and_active_use_boundaries_are_exact() {
    let consumable = UseComponents {
        consumable: Some(ConsumableUse {
            duration_ticks: 32,
            can_always_eat: false,
        }),
        blocks_attacks: true,
        ..UseComponents::default()
    };
    assert_eq!(
        dispatch(consumable, true),
        UseDispatch::Start { duration_ticks: 32 }
    );
    assert_eq!(
        dispatch(consumable, false),
        UseDispatch::Start {
            duration_ticks: u32::MAX
        }
    );
    assert_eq!(
        dispatch(
            UseComponents {
                consumable: Some(ConsumableUse {
                    duration_ticks: 0,
                    can_always_eat: true,
                }),
                ..UseComponents::default()
            },
            false
        ),
        UseDispatch::InstantConsume
    );
    assert_eq!(seconds_to_ticks(1.6), 32);

    let profile = UseProfile {
        duration_ticks: 1,
        release_driven: false,
        consumable: true,
    };
    let original = stack(30, "apple", 1, 1);
    let mut active = ActiveUse::start(&original, Hand::Main, profile, false).unwrap();
    let changed_components = stack(31, "apple", 1, 2);
    assert_eq!(
        active.tick(&original, &changed_components, profile, true),
        TickOutcome::Updated {
            observed_remaining: 1,
            periodic_consume: false,
            terminal: Some(TerminalUse::Release),
        }
    );
    assert_eq!(active.remaining, 0);

    let mut client = ActiveUse::start(&original, Hand::Off, profile, false).unwrap();
    assert_eq!(
        client.tick(&original, &original, profile, false),
        TickOutcome::Updated {
            observed_remaining: 1,
            periodic_consume: false,
            terminal: None,
        }
    );
    assert_eq!(
        client.tick(&stack(40, "carrot", 1, 0), &original, profile, true),
        TickOutcome::StoppedDifferentItem
    );
    assert!(ActiveUse::start(&ItemStack::empty(), Hand::Main, profile, false).is_none());
    assert!(ActiveUse::start(&original, Hand::Main, profile, true).is_none());
}

#[test]
fn consumable_cadence_and_release_final_update_are_locked() {
    let profile = UseProfile {
        duration_ticks: 32,
        release_driven: false,
        consumable: true,
    };
    let item = stack(50, "apple", 1, 0);
    let mut active = ActiveUse::start(&item, Hand::Main, profile, false).unwrap();
    let mut periodic_remaining = Vec::new();
    while active.remaining > 1 {
        if let TickOutcome::Updated {
            observed_remaining,
            periodic_consume: true,
            ..
        } = active.tick(&item, &item, profile, true)
        {
            periodic_remaining.push(observed_remaining);
        }
    }
    assert_eq!(periodic_remaining, [24, 20, 16, 12, 8, 4]);

    let release_profile = UseProfile {
        duration_ticks: 72_000,
        release_driven: true,
        consumable: false,
    };
    let mut release = ActiveUse::start(&item, Hand::Main, release_profile, false).unwrap();
    release.remaining = 42;
    let outcome = release.release(&item, true, release_profile);
    assert!(outcome.invoked_release);
    assert!(outcome.apply_after_use);
    assert_eq!(outcome.final_update_remaining, Some(42));

    let changed_item = stack(51, "carrot", 1, 0);
    assert!(
        !release
            .release(&changed_item, true, release_profile)
            .invoked_release
    );
}

#[test]
fn bookshelf_hit_sections_and_captured_state_dispatch_are_locked() {
    let expected = [
        ([0.0, 1.0, 0.5], 0),
        ([1.0 / 3.0, 1.0, 0.5], 1),
        ([2.0 / 3.0, 1.0, 0.5], 2),
        ([0.0, 0.5, 0.5], 3),
        ([1.0 / 3.0, 0.5, 0.5], 4),
        ([2.0 / 3.0, 0.5, 0.5], 5),
    ];
    for (relative, slot) in expected {
        assert_eq!(
            hit_slot(Direction::South, Direction::South, relative),
            Some(slot)
        );
    }
    assert_eq!(
        hit_slot(Direction::North, Direction::North, [0.0, 1.0, 0.5]),
        Some(2)
    );
    assert_eq!(
        hit_slot(Direction::East, Direction::East, [0.5, 1.0, 0.0]),
        Some(2)
    );
    assert_eq!(
        hit_slot(Direction::West, Direction::West, [0.5, 1.0, 0.0]),
        Some(0)
    );
    assert_eq!(
        hit_slot(Direction::South, Direction::North, [0.1, 0.9, 0.5]),
        None
    );

    let book = stack(60, "book", 1, 0);
    let nonbook = stack(61, "apple", 1, 0);
    let mut occupied = [false; CHISELED_BOOKSHELF_SLOTS];
    occupied[0] = true;
    assert_eq!(
        use_with_item(
            &nonbook,
            Direction::South,
            Direction::North,
            [0.0, 1.0, 0.0],
            occupied
        ),
        BookshelfUse::TryWithEmptyHand
    );
    assert_eq!(
        use_with_item(
            &book,
            Direction::South,
            Direction::South,
            [0.0, 1.0, 0.0],
            occupied
        ),
        BookshelfUse::TryWithEmptyHand
    );
    assert_eq!(
        use_empty_hand(
            Direction::South,
            Direction::South,
            [0.0, 1.0, 0.0],
            occupied
        ),
        BookshelfUse::Remove { slot: 0 }
    );
}

#[test]
fn bookshelf_item_state_and_last_slot_can_diverge() {
    let mut shelf = ChiseledBookshelf::empty();
    let mut overstack = stack(70, "enchanted_book", 1, 9);
    overstack.count = 12;
    let failed = shelf.set_item(2, overstack.clone(), false);
    assert!(failed.accepted);
    assert!(!failed.state_write_succeeded);
    assert!(failed.unsourced_block_change);
    assert_eq!(shelf.slots()[2].count, 12);
    assert_eq!(shelf.occupied_state(), [false; CHISELED_BOOKSHELF_SLOTS]);
    assert_eq!(shelf.comparator_output(true), 3);

    let applied = shelf.set_item(2, stack(71, "written_book", 1, 2), true);
    assert!(applied.state_changed);
    assert!(shelf.occupied_state()[2]);
    let same_state = shelf.set_item(2, stack(72, "book", 1, 3), true);
    assert!(!same_state.state_changed);
    assert!(same_state.unsourced_block_change);

    let invalid = shelf.set_item(0, stack(73, "apple", 1, 0), true);
    assert!(!invalid.accepted);
    assert!(shelf.can_place_item(0, &stack(74, "knowledge_book", 1, 0)));
    assert!(!shelf.can_place_item(2, &stack(74, "knowledge_book", 1, 0)));
    assert!(ChiseledBookshelf::can_take_to_destination(true));

    let mut raw_slots = std::array::from_fn(|_| ItemStack::empty());
    raw_slots[5] = stack(75, "apple", 1, 0);
    shelf.load_raw(raw_slots, 20);
    assert_eq!(shelf.comparator_output(true), 21);
    assert_eq!(shelf.comparator_output(false), 0);
    assert!(shelf.occupied_state()[2]);
    shelf.clear_content();
    assert!(shelf.slots().iter().all(ItemStack::is_empty));
    assert!(shelf.occupied_state()[2]);
    assert_eq!(shelf.last_interacted_slot(), 20);
}

#[test]
fn bookshelf_raw_removal_and_replacement_rng_budget_are_locked() {
    let mut shelf = ChiseledBookshelf::empty();
    let mut overstack = stack(80, "book", 1, 0);
    overstack.count = 31;
    shelf.set_item(0, overstack, true);
    let one = shelf.remove_item_no_update(0);
    assert_eq!(one.count, 1);
    assert_eq!(shelf.slots()[0].count, 30);
    assert!(shelf.occupied_state()[0]);

    let plan = shelf.replace_and_plan_drops(false, &[0, 20]).unwrap();
    assert_eq!(
        plan.chunks
            .iter()
            .map(|chunk| chunk.count)
            .collect::<Vec<_>>(),
        [10, 20]
    );
    assert_eq!(plan.position_double_draws, 18);
    assert_eq!(plan.bounded_integer_draws, 2);
    assert_eq!(plan.velocity_double_draws, 12);

    let mut suppressed = ChiseledBookshelf::empty();
    suppressed.set_item(0, stack(81, "book", 1, 0), true);
    assert!(
        suppressed
            .replace_and_plan_drops(true, &[])
            .unwrap()
            .chunks
            .is_empty()
    );
    assert!(!suppressed.slots()[0].is_empty());
}

#[test]
fn all_default_jukebox_songs_and_admission_rules_are_locked() {
    let expected = [
        ("music_disc_11", "11", 71.0, 11),
        ("music_disc_13", "13", 178.0, 1),
        ("music_disc_5", "5", 178.0, 15),
        ("music_disc_blocks", "blocks", 345.0, 3),
        ("music_disc_bounce", "bounce", 234.0, 8),
        ("music_disc_cat", "cat", 185.0, 2),
        ("music_disc_chirp", "chirp", 185.0, 4),
        ("music_disc_creator", "creator", 176.0, 12),
        (
            "music_disc_creator_music_box",
            "creator_music_box",
            73.0,
            11,
        ),
        ("music_disc_far", "far", 174.0, 5),
        ("music_disc_lava_chicken", "lava_chicken", 134.0, 9),
        ("music_disc_mall", "mall", 197.0, 6),
        ("music_disc_mellohi", "mellohi", 96.0, 7),
        ("music_disc_otherside", "otherside", 195.0, 14),
        ("music_disc_pigstep", "pigstep", 149.0, 13),
        ("music_disc_precipice", "precipice", 299.0, 13),
        ("music_disc_relic", "relic", 218.0, 14),
        ("music_disc_stal", "stal", 150.0, 8),
        ("music_disc_strad", "strad", 188.0, 9),
        ("music_disc_tears", "tears", 175.0, 10),
        ("music_disc_wait", "wait", 238.0, 12),
        ("music_disc_ward", "ward", 251.0, 10),
    ];
    for (item, song, length, comparator) in expected {
        let profile = default_song_for_item(&id(item)).unwrap();
        assert_eq!(profile.key, id(song));
        assert_eq!(profile.length_seconds, length);
        assert_eq!(profile.comparator_output, comparator);
    }
    assert!(default_song_for_item(&id("apple")).is_none());
    assert_eq!(
        use_admission(false, true, false, true, false),
        JukeboxUse::PredictSuccess
    );
    assert_eq!(
        use_admission(false, true, false, true, true),
        JukeboxUse::ServerSuccess
    );
    assert_eq!(
        use_admission(true, true, true, true, true),
        JukeboxUse::TryWithEmptyHand
    );
    assert_eq!(
        use_admission(false, false, false, true, true),
        JukeboxUse::Pass
    );
}

#[test]
fn jukebox_occupancy_item_song_and_clock_are_independent() {
    let mut jukebox = Jukebox::placed(true);
    assert!(jukebox.has_ticker());
    assert!(jukebox.item().stack.is_empty());
    assert_eq!(jukebox.source_signal(), 0);

    let effects = jukebox.set_item(disc(90, "music_disc_cat"));
    assert!(jukebox.has_record_state());
    assert_eq!(jukebox.comparator_output(), 2);
    assert_eq!(jukebox.source_signal(), 15);
    assert_eq!(effects.play_level_event, Some(id("cat")));
    assert_eq!(effects.unsourced_block_changes, 1);
    assert_eq!(jukebox.tick(3).play_game_events, 1);
    assert_eq!(jukebox.ticks_since_song_started(), 1);

    let invalid = JukeboxItem {
        stack: stack(91, "apple", 1, 0),
        playable: None,
    };
    let stopped = jukebox.set_item(invalid);
    assert!(jukebox.has_record_state());
    assert_eq!(jukebox.comparator_output(), 0);
    assert_eq!(jukebox.source_signal(), 0);
    assert_eq!(stopped.stop_level_events, 1);

    let custom = JukeboxItem {
        stack: stack(92, "apple", 1, 77),
        playable: Some(JukeboxSong {
            key: id("cat"),
            length_seconds: 185.0,
            comparator_output: 2,
        }),
    };
    assert!(!jukebox.can_place_item(&custom));
    let mut empty = Jukebox::empty();
    assert!(empty.can_place_item(&custom));
    empty.set_item(custom);
    assert_eq!(empty.source_signal(), 15);
}

#[test]
fn jukebox_padded_finish_persistence_rollback_and_removal_are_locked() {
    let mut jukebox = Jukebox::empty();
    let cat = disc(100, "music_disc_cat");
    jukebox.set_item(cat.clone());
    let finish = jukebox.active_song().unwrap().padded_finish_tick();
    assert_eq!(finish, 3_720);

    let removed = jukebox.remove_item();
    assert_eq!(removed.1.stop_level_events, 1);
    assert_eq!(jukebox.source_signal(), 0);
    let restarted = jukebox.set_item(removed.0);
    assert_eq!(restarted.play_level_event, Some(id("cat")));
    assert_eq!(jukebox.ticks_since_song_started(), 0);

    let mut loaded = Jukebox::empty();
    let no_play = loaded.load(cat.clone(), None);
    assert_eq!(no_play.play_level_event, None);
    assert_eq!(loaded.source_signal(), 0);
    loaded.load(cat.clone(), Some(finish - 1));
    assert_eq!(loaded.source_signal(), 15);
    assert!(!loaded.has_ticker());
    assert_eq!(loaded.tick(0).stop_level_events, 0);
    assert_eq!(loaded.tick(0).stop_level_events, 1);

    jukebox.set_item(cat.clone());
    jukebox.tick(0);
    let preserved_ticks = jukebox.ticks_since_song_started();
    jukebox.load(cat, Some(finish));
    assert_eq!(jukebox.source_signal(), 15);
    assert_eq!(jukebox.ticks_since_song_started(), preserved_ticks);

    let (ejected, pre_remove) = jukebox.pre_remove(false);
    assert!(ejected.is_some());
    assert!(pre_remove.item_entity_spawned);
    assert_eq!(pre_remove.ejection_float_draws, 2);
    assert_eq!(pre_remove.stop_level_events, 1);
    let final_stop = jukebox.set_removed();
    assert_eq!(final_stop.stop_level_events, 1);
    assert_eq!(final_stop.stop_game_events, 1);

    let mut suppressed = Jukebox::empty();
    suppressed.set_item(disc(101, "music_disc_ward"));
    assert!(suppressed.pre_remove(true).0.is_none());
    assert!(!suppressed.item().stack.is_empty());
    assert_eq!(suppressed.set_removed().stop_level_events, 1);
}
