use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::item::runtime::progression::advancement::{
    AdvancementDefinition, AdvancementDisplay, AdvancementProgress, AdvancementRequirements,
    AdvancementReward, AdvancementTracker, PERSISTENCE_DATA_FIX_FALLBACK, RewardEvent,
    SavedAdvancementProgress, deliver_reward,
};
use ferrite_gameplay::item::runtime::progression::experience::{ExperienceData, points_for_level};
use ferrite_gameplay::item::runtime::progression::hunger::{
    Difficulty, FoodBranch, FoodData, FoodTickInput,
};
use ferrite_gameplay::item::runtime::random::GameplayRandom;
use ferrite_gameplay::item::runtime::stack::ItemStack;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn id(path: &str) -> ResourceId {
    ResourceId::minecraft(path).unwrap()
}

fn reward(experience: i32) -> AdvancementReward {
    AdvancementReward {
        experience,
        loot_tables: Vec::new(),
        recipes: Vec::new(),
        function: None,
    }
}

fn definition(
    path: &str,
    root: &str,
    requirements: AdvancementRequirements,
) -> AdvancementDefinition {
    AdvancementDefinition {
        key: id(path),
        root: id(root),
        requirements,
        reward: reward(7),
        display: Some(AdvancementDisplay {
            announce_chat: true,
        }),
    }
}

fn food_input(difficulty: Difficulty, hurt: bool, health: f32) -> FoodTickInput {
    FoodTickInput {
        difficulty,
        natural_regeneration: true,
        hurt,
        health,
    }
}

#[test]
fn hunger_eating_clamps_food_and_saturation_to_vanilla_bounds() {
    let mut food = FoodData::new();
    food.food_level = 10;
    food.saturation_level = 1.0;
    food.eat(4, 0.6);
    assert_eq!(food.food_level, 14);
    assert_eq!(food.saturation_level, 5.8);

    food.add_food(100, 100.0);
    assert_eq!((food.food_level, food.saturation_level), (20, 20.0));
    food.add_food(-100, -100.0);
    assert_eq!((food.food_level, food.saturation_level), (0, 0.0));
    assert!(!food.has_enough_food());
    assert!(food.needs_food());
}

#[test]
fn exhaustion_requires_strictly_more_than_four_and_spends_one_quantum() {
    let mut food = FoodData::new();
    food.exhaustion_level = 4.0;
    let exact = food.tick(food_input(Difficulty::Normal, false, 20.0));
    assert!(!exact.spent_exhaustion);
    assert_eq!(food.exhaustion_level, 4.0);

    food.exhaustion_level = 12.5;
    food.saturation_level = 2.0;
    let spent = food.tick(food_input(Difficulty::Normal, false, 20.0));
    assert!(spent.spent_exhaustion);
    assert_eq!(food.exhaustion_level, 8.5);
    assert_eq!(food.saturation_level, 1.0);

    food.saturation_level = 0.0;
    food.exhaustion_level = 4.5;
    food.food_level = 10;
    food.tick(food_input(Difficulty::Peaceful, false, 20.0));
    assert_eq!(food.food_level, 10);
    food.exhaustion_level = 4.5;
    food.tick(food_input(Difficulty::Normal, false, 20.0));
    assert_eq!(food.food_level, 9);
}

#[test]
fn hunger_regeneration_branches_share_timer_and_charge_exact_exhaustion() {
    let mut food = FoodData::new();
    food.tick_timer = 9;
    food.saturation_level = 3.0;
    let saturated = food.tick(food_input(Difficulty::Normal, true, 12.0));
    assert_eq!(saturated.branch, FoodBranch::SaturatedRegeneration);
    assert_eq!(saturated.healed, 0.5);
    assert_eq!(food.exhaustion_level, 3.0);
    assert_eq!(food.tick_timer, 0);

    food.saturation_level = 0.0;
    food.food_level = 18;
    food.tick_timer = 79;
    let slow = food.tick(food_input(Difficulty::Normal, true, 12.0));
    assert_eq!(slow.branch, FoodBranch::SlowRegeneration);
    assert_eq!(slow.healed, 1.0);
    assert_eq!(food.exhaustion_level, 9.0);
    assert_eq!(food.tick_timer, 0);

    food.tick_timer = 50;
    let idle = food.tick(food_input(Difficulty::Normal, false, 20.0));
    assert_eq!(idle.branch, FoodBranch::Idle);
    assert_eq!(food.tick_timer, 0);
}

#[test]
fn starvation_respects_difficulty_health_floors_and_resets_every_eighty_ticks() {
    for (difficulty, health, expected) in [
        (Difficulty::Easy, 10.0, 0.0),
        (Difficulty::Easy, 11.0, 1.0),
        (Difficulty::Normal, 1.0, 0.0),
        (Difficulty::Normal, 2.0, 1.0),
        (Difficulty::Hard, 1.0, 1.0),
    ] {
        let mut food = FoodData {
            food_level: 0,
            saturation_level: 0.0,
            exhaustion_level: 0.0,
            tick_timer: 79,
        };
        let outcome = food.tick(food_input(difficulty, false, health));
        assert_eq!(outcome.branch, FoodBranch::Starvation);
        assert_eq!(outcome.starvation_damage, expected);
        assert_eq!(food.tick_timer, 0);
    }

    let mut food = FoodData::new();
    assert!(!food.cause_exhaustion(3.0, false, false));
    assert!(!food.cause_exhaustion(3.0, true, true));
    assert!(food.cause_exhaustion(50.0, true, false));
    assert_eq!(food.exhaustion_level, 40.0);
}

#[test]
fn experience_level_costs_match_all_piecewise_boundaries() {
    assert_eq!(points_for_level(0), 7);
    assert_eq!(points_for_level(14), 35);
    assert_eq!(points_for_level(15), 37);
    assert_eq!(points_for_level(29), 107);
    assert_eq!(points_for_level(30), 112);
    assert_eq!(points_for_level(31), 121);
    assert_eq!(
        points_for_level(i32::MAX),
        112_i32.wrapping_add(i32::MAX.wrapping_sub(30).wrapping_mul(9))
    );
}

#[test]
fn experience_points_normalize_across_positive_and_negative_level_boundaries() {
    let mut experience = ExperienceData::new(1);
    experience.give_points(8);
    assert_eq!(
        (experience.level, experience.total, experience.score),
        (1, 8, 8)
    );
    assert!((experience.progress - 1.0 / 9.0).abs() < f32::EPSILON);

    experience.give_points(-2);
    assert_eq!(
        (experience.level, experience.total, experience.score),
        (0, 6, 6)
    );
    assert!((experience.progress - 6.0 / 7.0).abs() < f32::EPSILON);

    experience.give_points(-100);
    assert_eq!(
        (experience.level, experience.progress, experience.total),
        (0, 0.0, 0)
    );
    assert_eq!(experience.score, -94);
}

#[test]
fn direct_levels_gate_sound_and_enchantment_and_death_side_effects() {
    let mut experience = ExperienceData::new(1);
    experience.level = 4;
    experience.tick_count = 100;
    assert_eq!(experience.give_levels(1), None);
    experience.level = 4;
    experience.tick_count = 101;
    let sound = experience.give_levels(1).unwrap();
    assert_eq!(sound.pitch, 1.0);
    assert_eq!(sound.volume, 0.125);

    experience.level = 35;
    experience.last_level_up_tick = 0;
    experience.tick_count = 101;
    assert_eq!(experience.give_levels(5).unwrap().volume, 0.75);
    experience.progress = 0.5;
    experience.total = 100;
    experience.on_enchantment_performed(100, 55);
    assert_eq!(
        (
            experience.level,
            experience.progress,
            experience.total,
            experience.enchantment_seed,
        ),
        (0, 0.0, 0, 55)
    );
    experience.load_seed(0, 77);
    assert_eq!(experience.enchantment_seed, 77);
    experience.level = 30;
    assert_eq!(experience.death_reward(false, false), 100);
    assert_eq!(experience.death_reward(true, false), 0);
    assert_eq!(experience.death_reward(false, true), 0);
}

#[test]
fn advancement_progress_uses_and_of_or_groups_and_prunes_unknown_criteria() {
    let requirements = AdvancementRequirements {
        groups: vec![vec!["a".into(), "b".into()], vec!["c".into(), "d".into()]],
    };
    let mut progress = AdvancementProgress::new(requirements);
    assert!(!progress.is_done());
    assert!(progress.grant("a", 1));
    assert!(progress.grant("d", 2));
    assert!(progress.is_done());
    assert!(!progress.grant("unknown", 3));

    progress.update(AdvancementRequirements::all_of(["b".to_owned()]));
    assert_eq!(progress.criteria.keys().cloned().collect::<Vec<_>>(), ["b"]);
    assert!(!progress.is_done());
    assert!(!AdvancementProgress::new(AdvancementRequirements { groups: vec![] }).is_done());
}

#[test]
fn advancement_award_revoke_listeners_and_completion_transition_are_idempotent() {
    let key = id("story/test");
    let root = id("story/root");
    let definition = definition(
        "story/test",
        "story/root",
        AdvancementRequirements::any_of(["a".to_owned(), "b".to_owned()]),
    );
    let mut tracker = AdvancementTracker::load(vec![definition], BTreeMap::new()).tracker;
    assert_eq!(
        tracker.listeners,
        [(key.clone(), "a".into()), (key.clone(), "b".into())]
            .into_iter()
            .collect()
    );

    let award = tracker.award(&key, "a", 11, true);
    assert!(award.changed && award.became_complete && award.announce);
    assert_eq!(award.reward, Some(reward(7)));
    assert!(tracker.listeners.is_empty());
    assert!(tracker.roots_to_update.contains(&root));
    assert!(!tracker.award(&key, "a", 12, true).changed);

    let second_alternative = tracker.award(&key, "b", 13, true);
    assert!(second_alternative.changed);
    assert!(!second_alternative.became_complete);
    tracker.flush(true, |_, _| BTreeSet::new());
    let redundant_revoke = tracker.revoke(&key, "a");
    assert!(redundant_revoke.changed);
    assert!(!redundant_revoke.became_incomplete);
    assert!(tracker.roots_to_update.is_empty());

    let revoke = tracker.revoke(&key, "b");
    assert!(revoke.changed && revoke.became_incomplete);
    assert_eq!(
        tracker.listeners,
        [(key, "a".into()), (id("story/test"), "b".into())]
            .into_iter()
            .collect()
    );
}

#[test]
fn advancement_load_save_flush_and_tab_selection_preserve_protocol_state() {
    let root = definition(
        "story/root",
        "story/root",
        AdvancementRequirements::all_of(["root".to_owned()]),
    );
    let child = definition(
        "story/child",
        "story/root",
        AdvancementRequirements::all_of(["child".to_owned()]),
    );
    let unknown = id("removed");
    let saved = [
        (
            id("story/child"),
            SavedAdvancementProgress {
                completed: [("child".to_owned(), 42)].into_iter().collect(),
            },
        ),
        (
            unknown.clone(),
            SavedAdvancementProgress {
                completed: BTreeMap::new(),
            },
        ),
    ]
    .into_iter()
    .collect();
    let load = AdvancementTracker::load(vec![root, child], saved);
    assert_eq!(load.unknown_saved, [unknown]);
    assert_eq!(
        load.tracker.save()[&id("story/child")].completed["child"],
        42
    );
    let mut tracker = load.tracker;
    let packet = tracker
        .flush(true, |_, _| {
            [id("story/root"), id("story/child")].into_iter().collect()
        })
        .unwrap();
    assert!(packet.reset);
    assert_eq!(packet.added.len(), 2);
    assert_eq!(packet.progress.len(), 1);
    assert!(tracker.flush(true, |_, _| BTreeSet::new()).is_none());

    assert_eq!(tracker.select_tab(Some(&id("story/child"))), None);
    assert_eq!(
        tracker.select_tab(Some(&id("story/root"))),
        Some(Some(id("story/root")))
    );
    assert_eq!(tracker.select_tab(Some(&id("story/child"))), Some(None));
    assert_eq!(tracker.select_tab(Some(&id("story/child"))), None);
}

#[test]
fn advancement_load_reports_unknown_data_and_requests_empty_definition_rewards() {
    let automatic = definition(
        "automatic",
        "automatic",
        AdvancementRequirements { groups: Vec::new() },
    );
    let unknown = id("unknown");
    let saved = [(
        unknown.clone(),
        SavedAdvancementProgress {
            completed: BTreeMap::new(),
        },
    )]
    .into_iter()
    .collect();
    let load = AdvancementTracker::load(vec![automatic], saved);
    assert_eq!(PERSISTENCE_DATA_FIX_FALLBACK, 1343);
    assert_eq!(load.unknown_saved, [unknown]);
    assert_eq!(load.automatic_rewards.len(), 1);
    assert_eq!(load.automatic_rewards[0].advancement, id("automatic"));
    assert_eq!(load.automatic_rewards[0].reward, reward(7));
    assert!(load.tracker.listeners.is_empty());
}

#[derive(Default)]
struct ScriptRandom {
    floats: VecDeque<f32>,
}

impl GameplayRandom for ScriptRandom {
    fn next_int(&mut self, _bound: u32) -> u32 {
        0
    }

    fn next_float(&mut self) -> f32 {
        self.floats.pop_front().unwrap_or(0.5)
    }

    fn next_bool(&mut self) -> bool {
        false
    }
}

#[test]
fn advancement_rewards_preserve_xp_loot_sound_broadcast_recipe_function_order() {
    let reward = AdvancementReward {
        experience: 7,
        loot_tables: vec![id("loot/first"), id("loot/second")],
        recipes: vec![id("recipe")],
        function: Some(id("function")),
    };
    let mut experience = ExperienceData::new(1);
    let mut random = ScriptRandom {
        floats: [0.75, 0.25].into_iter().collect(),
    };
    let delivery = deliver_reward(
        &reward,
        &mut experience,
        &mut random,
        |table| {
            vec![ItemStack::new(
                if table == &id("loot/first") { 1 } else { 2 },
                id("apple"),
                1,
                64,
                0,
            )]
        },
        |stack| stack.identity == 1,
    )
    .unwrap();
    assert_eq!(experience.total, 7);
    assert_eq!(delivery.pickup_pitches, [2.7]);
    assert_eq!(delivery.dropped[0].identity, 2);
    assert_eq!(
        delivery.events,
        [
            RewardEvent::Experience(7),
            RewardEvent::LootTable(id("loot/first")),
            RewardEvent::PickupSound,
            RewardEvent::LootTable(id("loot/second")),
            RewardEvent::InventoryBroadcast,
            RewardEvent::Recipes(vec![id("recipe")]),
            RewardEvent::Function(id("function")),
        ]
    );
}
