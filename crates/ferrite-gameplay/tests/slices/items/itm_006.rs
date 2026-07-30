use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::item::runtime::enchantment::{
    ActiveEnchantment, EnchantOfferRequest, EnchantedItem, EnchantingPlayer, EnchantmentCost,
    EnchantmentDefinition, EquipmentSlot, EquippedEnchantments, PostAttackTarget,
    commit_enchanting_offer, compose_item_int, compose_item_value, enchanting_cost,
    enchantments_compatible, equipment_immunity, offer_costs, select_enchantments, visit_equipment,
    visit_post_attack,
};
use ferrite_gameplay::item::runtime::inventory::Inventory;
use ferrite_gameplay::item::runtime::loot::context::{
    LOOT_CONTEXT_SET_COUNT, LootContext, LootContextError, LootContextSet, LootParameter, LootValue,
};
use ferrite_gameplay::item::runtime::loot::evaluator::{
    LootDispatch, LootEvaluationError, LootEvaluator, LootWarning, consume_generated_once,
    split_normal_stacks,
};
use ferrite_gameplay::item::runtime::loot::fill::fill_container;
use ferrite_gameplay::item::runtime::loot::model::{
    ExpandedLootEntry, LootCondition, LootDataKind, LootEntry, LootFunction, LootNumberProvider,
    LootOutput, LootPool, LootRandomOwner, LootTable, resolve_random_owner,
};
use ferrite_gameplay::item::runtime::random::GameplayRandom;
use ferrite_gameplay::item::runtime::stack::ItemStack;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn id(path: &str) -> ResourceId {
    ResourceId::minecraft(path).unwrap()
}

fn stack(identity: u64, path: &str, count: i32, maximum: i32) -> ItemStack {
    let mut stack = ItemStack::new(identity, id(path), count.min(maximum), maximum, 0);
    stack.count = count;
    stack
}

#[derive(Default)]
struct ScriptRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    booleans: VecDeque<bool>,
    bounds: Vec<u32>,
    float_calls: usize,
    boolean_calls: usize,
}

impl ScriptRandom {
    fn with_integers(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            integers: values.into_iter().collect(),
            ..Self::default()
        }
    }
}

impl GameplayRandom for ScriptRandom {
    fn next_int(&mut self, bound: u32) -> u32 {
        self.bounds.push(bound);
        self.integers.pop_front().unwrap_or(0)
    }

    fn next_float(&mut self) -> f32 {
        self.float_calls += 1;
        self.floats.pop_front().unwrap_or(0.5)
    }

    fn next_bool(&mut self) -> bool {
        self.boolean_calls += 1;
        self.booleans.pop_front().unwrap_or(false)
    }
}

fn active(path: &str, level: u8, slots: &[EquipmentSlot], effects: &[&str]) -> ActiveEnchantment {
    ActiveEnchantment {
        key: id(path),
        level,
        matching_slots: slots.iter().copied().collect(),
        effects: effects.iter().map(|effect| id(effect)).collect(),
    }
}

fn item(path: &str, enchantable: Option<u32>, ordinary_book: bool) -> EnchantedItem {
    EnchantedItem {
        stack: stack(1, path, 1, 1),
        enchantable,
        ordinary_book,
        active: Vec::new(),
        stored: Vec::new(),
    }
}

fn definition(
    path: &str,
    weight: u32,
    exclusive: &[&str],
    effects: &[&str],
) -> EnchantmentDefinition {
    EnchantmentDefinition {
        key: id(path),
        weight,
        minimum_level: 1,
        maximum_level: 3,
        minimum_cost: EnchantmentCost {
            base: 1,
            per_level_above_first: 10,
        },
        maximum_cost: EnchantmentCost {
            base: 100,
            per_level_above_first: 10,
        },
        primary_items: [id("diamond_sword")].into_iter().collect(),
        exclusive_with: exclusive.iter().map(|path| id(path)).collect(),
        matching_slots: [EquipmentSlot::MainHand].into_iter().collect(),
        effects: effects.iter().map(|effect| id(effect)).collect(),
    }
}

#[test]
fn enchantment_hooks_preserve_entry_effect_equipment_and_post_attack_order() {
    let component = id("damage");
    let mut sword = item("diamond_sword", Some(10), false);
    sword.active = vec![
        active(
            "first",
            1,
            &[EquipmentSlot::MainHand],
            &["damage", "damage"],
        ),
        active("second", 2, &[EquipmentSlot::MainHand], &["damage"]),
    ];
    sword.stored = vec![active(
        "stored_only",
        1,
        &[EquipmentSlot::MainHand],
        &["damage"],
    )];
    let mut effect_order = Vec::new();
    let value = compose_item_value(&sword, &component, 2.0, |entry, index, value| {
        effect_order.push((entry.key.clone(), index));
        value * 2.0 + f32::from(entry.level)
    });
    assert_eq!(value, 24.0);
    assert_eq!(
        effect_order,
        [(id("first"), 0), (id("first"), 1), (id("second"), 0)]
    );
    assert_eq!(
        compose_item_int(&sword, &component, 1, true, |_, _, _| -0.75),
        0
    );

    let equipment = vec![
        EquippedEnchantments {
            slot: EquipmentSlot::Head,
            item: EnchantedItem {
                active: vec![active("head", 1, &[EquipmentSlot::Head], &[])],
                ..item("diamond_helmet", None, false)
            },
        },
        EquippedEnchantments {
            slot: EquipmentSlot::MainHand,
            item: sword.clone(),
        },
    ];
    let mut visits = Vec::new();
    visit_equipment(&equipment, |slot, entry| {
        visits.push((slot, entry.key.clone()))
    });
    assert_eq!(visits[0], (EquipmentSlot::MainHand, id("first")));
    assert_eq!(visits.last(), Some(&(EquipmentSlot::Head, id("head"))));

    let mut immunity_calls = 0;
    assert!(equipment_immunity(&equipment, |_, _| {
        immunity_calls += 1;
        immunity_calls == 1
    }));
    assert_eq!(immunity_calls, 3);

    let mut post_attack = Vec::new();
    visit_post_attack(&equipment, Some(&sword), |target, slot, entry| {
        post_attack.push((target, slot, entry.key.clone()));
    });
    assert!(
        post_attack[..3]
            .iter()
            .all(|visit| visit.0 == PostAttackTarget::Victim)
    );
    assert!(
        post_attack[3..]
            .iter()
            .all(|visit| visit.0 == PostAttackTarget::Attacker)
    );
}

#[test]
fn compatibility_is_distinct_and_checks_both_exclusive_sets() {
    let ordinary = definition("ordinary", 1, &[], &[]);
    let excludes_other = definition("exclusive", 1, &["ordinary"], &[]);
    assert!(!enchantments_compatible(&ordinary, &ordinary));
    assert!(!enchantments_compatible(&ordinary, &excludes_other));
    assert!(!enchantments_compatible(&excludes_other, &ordinary));
    assert!(enchantments_compatible(
        &ordinary,
        &definition("compatible", 1, &[], &[])
    ));
}

#[test]
fn offer_cost_uses_two_draws_per_slot_and_caps_only_above_fifteen() {
    let mut random = ScriptRandom::with_integers([7, 0, 7, 0, 7, 0]);
    assert_eq!(offer_costs(&mut random, 0, Some(8)).unwrap(), [2, 6, 8]);
    assert_eq!(random.bounds, [8, 1, 8, 1, 8, 1]);

    let mut capped = ScriptRandom::with_integers([0, 15]);
    assert_eq!(enchanting_cost(&mut capped, 2, 16, Some(8)).unwrap(), 30);
    assert_eq!(capped.bounds, [8, 16]);

    let mut absent = ScriptRandom::default();
    assert_eq!(enchanting_cost(&mut absent, 0, 15, None).unwrap(), 0);
    assert!(absent.bounds.is_empty());
    assert!(enchanting_cost(&mut absent, 0, -1, Some(8)).is_err());
}

#[test]
fn selection_uses_registry_order_weighting_cumulative_compatibility_and_loop_draw() {
    let definitions = [
        definition("a", 1, &[], &[]),
        definition("b", 1, &[], &[]),
        definition("c", 1, &["a"], &[]),
    ];
    let sword = item("diamond_sword", Some(4), false);
    let mut random = ScriptRandom::with_integers([0, 0, 0, 0, 0, 49]);
    let selected = select_enchantments(&mut random, &sword, 10, &definitions).unwrap();
    assert_eq!(
        selected.iter().map(|entry| &entry.key).collect::<Vec<_>>(),
        [&id("a"), &id("b")]
    );
    assert_eq!(random.bounds, [2, 2, 3, 50, 1, 50]);
    assert_eq!(random.float_calls, 2);

    let mut book = item("book", Some(4), true);
    let mut book_random = ScriptRandom::with_integers([0, 0, 0, 0, 0, 49, 1]);
    let book_selected = select_enchantments(&mut book_random, &book, 10, &definitions).unwrap();
    assert_eq!(book_selected.len(), 1);
    assert_eq!(book_random.bounds.last(), Some(&2));
    book.stored = vec![active("stored", 1, &[EquipmentSlot::MainHand], &[])];
    assert!(book.active.is_empty());
}

#[test]
fn menu_commit_spends_slot_levels_transmutes_books_and_refreshes_seed() {
    let definitions = [definition("sharpness", 1, &[], &["damage"])];
    let mut book = item("book", Some(4), true);
    let mut player = EnchantingPlayer {
        experience_levels: 30,
        enchantment_seed: 7,
        infinite_materials: false,
    };
    let mut lapis = 3;
    let mut random = ScriptRandom::with_integers([0, 0, 0, 49]);
    let commit = commit_enchanting_offer(
        &mut book,
        &mut player,
        &mut lapis,
        EnchantOfferRequest {
            slot: 1,
            displayed_cost: 20,
            refreshed_seed: 99,
        },
        &definitions,
        &mut random,
    )
    .unwrap();
    assert!(commit.committed && commit.transmuted_book);
    assert_eq!((commit.levels_spent, commit.lapis_spent), (2, 2));
    assert_eq!(
        (player.experience_levels, player.enchantment_seed, lapis),
        (28, 99, 1)
    );
    assert_eq!(book.stack.item.as_ref(), Some(&id("enchanted_book")));
    assert!(book.active.is_empty());
    assert_eq!(book.stored[0].key, id("sharpness"));

    let mut creative = item("diamond_sword", Some(4), false);
    let mut creative_player = EnchantingPlayer {
        experience_levels: 0,
        enchantment_seed: 1,
        infinite_materials: true,
    };
    let mut no_lapis = 0;
    let mut creative_random = ScriptRandom::with_integers([0, 0, 0, 49]);
    assert!(
        commit_enchanting_offer(
            &mut creative,
            &mut creative_player,
            &mut no_lapis,
            EnchantOfferRequest {
                slot: 2,
                displayed_cost: 30,
                refreshed_seed: 2,
            },
            &definitions,
            &mut creative_random,
        )
        .unwrap()
        .committed
    );
    assert_eq!((creative_player.experience_levels, no_lapis), (0, 0));
}

#[test]
fn loot_context_catalog_is_closed_and_required_or_disallowed_values_fail() {
    assert_eq!(LootContextSet::ALL.len(), LOOT_CONTEXT_SET_COUNT);
    assert_eq!(
        LootContextSet::ALL
            .iter()
            .map(|set| set.id())
            .collect::<BTreeSet<_>>()
            .len(),
        LOOT_CONTEXT_SET_COUNT
    );
    assert_eq!(LootDataKind::ALL.len(), 3);

    let missing = LootContext::create(LootContextSet::Chest, BTreeMap::new(), BTreeMap::new(), 0.0);
    assert_eq!(
        missing.unwrap_err(),
        LootContextError::MissingRequired(LootParameter::Origin)
    );
    let disallowed = LootContext::create(
        LootContextSet::Empty,
        [(
            LootParameter::Tool,
            LootValue::Stack(stack(2, "stick", 1, 64)),
        )]
        .into_iter()
        .collect(),
        BTreeMap::new(),
        0.0,
    );
    assert_eq!(
        disallowed.unwrap_err(),
        LootContextError::Disallowed(LootParameter::Tool)
    );
}

#[test]
fn loot_random_owner_obeys_source_seed_sequence_then_level_precedence() {
    assert_eq!(
        resolve_random_owner(true, 42, Some(&id("chests"))),
        LootRandomOwner::ExplicitSource
    );
    assert_eq!(
        resolve_random_owner(false, 42, Some(&id("chests"))),
        LootRandomOwner::ExplicitSeed(42)
    );
    assert_eq!(
        resolve_random_owner(false, 0, Some(&id("chests"))),
        LootRandomOwner::TableSequence(id("chests"))
    );
    assert_eq!(resolve_random_owner(false, 0, None), LootRandomOwner::Level);
}

#[derive(Default)]
struct TestDispatch {
    calls: Vec<String>,
    weights: BTreeMap<u64, VecDeque<i32>>,
    outputs: BTreeMap<u64, Vec<LootOutput>>,
    disabled: BTreeSet<ResourceId>,
}

impl LootDispatch for TestDispatch {
    fn test_condition(
        &mut self,
        condition: &LootCondition,
        _context: &LootContext,
        _random: &mut dyn GameplayRandom,
    ) -> Result<bool, LootEvaluationError> {
        self.calls
            .push(format!("condition:{}", condition.type_id.path()));
        Ok(condition.payload.first().copied().unwrap_or(0) != 0)
    }

    fn number_int(
        &mut self,
        provider: &LootNumberProvider,
        _context: &LootContext,
        _random: &mut dyn GameplayRandom,
    ) -> Result<i32, LootEvaluationError> {
        self.calls.push(format!("int:{}", provider.type_id.path()));
        Ok(i32::from(provider.payload.first().copied().unwrap_or(0)))
    }

    fn number_float(
        &mut self,
        provider: &LootNumberProvider,
        _context: &LootContext,
        _random: &mut dyn GameplayRandom,
    ) -> Result<f32, LootEvaluationError> {
        self.calls
            .push(format!("float:{}", provider.type_id.path()));
        Ok(f32::from(provider.payload.first().copied().unwrap_or(0)))
    }

    fn expand_entry(
        &mut self,
        entry: &LootEntry,
        _context: &LootContext,
        _random: &mut dyn GameplayRandom,
        output: &mut Vec<ExpandedLootEntry>,
    ) -> Result<(), LootEvaluationError> {
        let handle = u64::from(entry.payload.first().copied().unwrap_or(0));
        self.calls.push(format!("expand:{handle}"));
        output.push(ExpandedLootEntry { handle });
        Ok(())
    }

    fn entry_weight(
        &mut self,
        entry: ExpandedLootEntry,
        _luck: f32,
    ) -> Result<i32, LootEvaluationError> {
        self.calls.push(format!("weight:{}", entry.handle));
        Ok(self
            .weights
            .get_mut(&entry.handle)
            .and_then(VecDeque::pop_front)
            .unwrap_or(1))
    }

    fn create_outputs(
        &mut self,
        entry: ExpandedLootEntry,
        _context: &LootContext,
        _random: &mut dyn GameplayRandom,
        output: &mut Vec<LootOutput>,
    ) -> Result<(), LootEvaluationError> {
        self.calls.push(format!("create:{}", entry.handle));
        output.extend(self.outputs.get(&entry.handle).cloned().unwrap_or_default());
        Ok(())
    }

    fn apply_function(
        &mut self,
        function: &LootFunction,
        _context: &LootContext,
        _random: &mut dyn GameplayRandom,
        mut stack: ItemStack,
    ) -> Result<ItemStack, LootEvaluationError> {
        self.calls
            .push(format!("function:{}", function.type_id.path()));
        stack.grow(i32::from(function.payload.first().copied().unwrap_or(0)));
        Ok(stack)
    }

    fn item_enabled(&self, stack: &ItemStack) -> bool {
        stack
            .item
            .as_ref()
            .is_some_and(|item| !self.disabled.contains(item))
    }
}

fn descriptor(path: &str, payload: u8) -> (ResourceId, Vec<u8>) {
    (id(path), vec![payload])
}

fn number(path: &str, value: u8) -> LootNumberProvider {
    let (type_id, payload) = descriptor(path, value);
    LootNumberProvider { type_id, payload }
}

fn function(path: &str, add: u8) -> LootFunction {
    let (type_id, payload) = descriptor(path, add);
    LootFunction { type_id, payload }
}

fn entry(handle: u8) -> LootEntry {
    LootEntry {
        type_id: id("entry"),
        payload: vec![handle],
    }
}

fn pool(entries: Vec<LootEntry>) -> LootPool {
    LootPool {
        entries,
        conditions: Vec::new(),
        functions: Vec::new(),
        rolls: number("rolls", 1),
        bonus_rolls: number("bonus", 0),
    }
}

fn empty_context() -> LootContext {
    LootContext::create(LootContextSet::Empty, BTreeMap::new(), BTreeMap::new(), 0.0).unwrap()
}

#[test]
fn loot_conditions_short_circuit_and_single_candidate_elides_weighted_draw() {
    let mut accepted_pool = pool(vec![entry(1)]);
    accepted_pool.conditions = vec![
        LootCondition {
            type_id: id("first"),
            payload: vec![1],
        },
        LootCondition {
            type_id: id("second"),
            payload: vec![0],
        },
        LootCondition {
            type_id: id("never"),
            payload: vec![1],
        },
    ];
    let key = id("short_circuit");
    let table = LootTable {
        key: key.clone(),
        parameter_set: LootContextSet::Empty,
        random_sequence: None,
        pools: vec![accepted_pool],
        functions: Vec::new(),
    };
    let tables = [(key.clone(), table)].into_iter().collect();
    let mut dispatch = TestDispatch::default();
    let mut random = ScriptRandom::default();
    let result = LootEvaluator::new(&tables, &empty_context(), &mut dispatch, &mut random)
        .evaluate(&key)
        .unwrap();
    assert!(result.stacks.is_empty());
    assert_eq!(dispatch.calls, ["condition:first", "condition:second"]);
    assert!(random.bounds.is_empty());

    let mut one_pool = pool(vec![entry(1)]);
    one_pool.functions.push(function("pool_add", 1));
    let one_key = id("one");
    let one_table = LootTable {
        key: one_key.clone(),
        parameter_set: LootContextSet::Empty,
        random_sequence: None,
        pools: vec![one_pool],
        functions: vec![function("table_add", 10)],
    };
    let one_tables = [(one_key.clone(), one_table)].into_iter().collect();
    let mut one_dispatch = TestDispatch::default();
    one_dispatch
        .outputs
        .insert(1, vec![LootOutput::Stack(stack(3, "apple", 1, 64))]);
    let mut one_random = ScriptRandom::default();
    let result = LootEvaluator::new(
        &one_tables,
        &empty_context(),
        &mut one_dispatch,
        &mut one_random,
    )
    .evaluate(&one_key)
    .unwrap();
    assert_eq!(result.stacks[0].count, 12);
    assert!(one_random.bounds.is_empty());
    let function_calls = one_dispatch
        .calls
        .iter()
        .filter(|call| call.starts_with("function:"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(function_calls, ["function:pool_add", "function:table_add"]);
}

#[test]
fn weighted_loot_requeries_dynamic_weights_in_retained_order() {
    let key = id("weighted");
    let table = LootTable {
        key: key.clone(),
        parameter_set: LootContextSet::Empty,
        random_sequence: None,
        pools: vec![pool(vec![entry(1), entry(2)])],
        functions: Vec::new(),
    };
    let tables = [(key.clone(), table)].into_iter().collect();
    let mut dispatch = TestDispatch::default();
    dispatch.weights.insert(1, [5, 1].into_iter().collect());
    dispatch.weights.insert(2, [5, 5].into_iter().collect());
    dispatch
        .outputs
        .insert(2, vec![LootOutput::Stack(stack(4, "diamond", 1, 64))]);
    let mut random = ScriptRandom::with_integers([4]);
    let result = LootEvaluator::new(&tables, &empty_context(), &mut dispatch, &mut random)
        .evaluate(&key)
        .unwrap();
    assert_eq!(result.stacks[0].item.as_ref(), Some(&id("diamond")));
    let weight_calls = dispatch
        .calls
        .iter()
        .filter(|call| call.starts_with("weight:"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        weight_calls,
        ["weight:1", "weight:2", "weight:1", "weight:2"]
    );
    assert_eq!(random.bounds, [10]);
}

#[test]
fn recursive_tables_warn_and_stack_splitter_uses_the_maximum_boundary() {
    let key = id("recursive");
    let table = LootTable {
        key: key.clone(),
        parameter_set: LootContextSet::Empty,
        random_sequence: None,
        pools: vec![pool(vec![entry(1)])],
        functions: Vec::new(),
    };
    let tables = [(key.clone(), table)].into_iter().collect();
    let mut dispatch = TestDispatch::default();
    dispatch
        .outputs
        .insert(1, vec![LootOutput::Table(key.clone())]);
    let mut random = ScriptRandom::default();
    let result = LootEvaluator::new(&tables, &empty_context(), &mut dispatch, &mut random)
        .evaluate(&key)
        .unwrap();
    assert!(result.stacks.is_empty());
    assert_eq!(result.warnings, [LootWarning::RecursiveTable(key)]);

    let disabled = id("disabled");
    let split = split_normal_stacks(
        vec![
            stack(5, "apple", 63, 64),
            stack(6, "stone", 64, 64),
            stack(7, "diamond", 65, 64),
            stack(8, "disabled", 1, 64),
        ],
        |stack| stack.item.as_ref() != Some(&disabled),
    );
    assert_eq!(
        split.iter().map(|stack| stack.count).collect::<Vec<_>>(),
        [63, 64, 64, 1]
    );
}

#[test]
fn container_fill_generates_once_shuffles_empty_slots_and_splits_without_overwrite() {
    let mut inventory = Inventory::empty(3);
    inventory.slots[1].stack = stack(9, "barrier", 1, 64);
    let mut random = ScriptRandom::with_integers([0, 0, 0, 0]);
    let fill =
        fill_container(&mut inventory, vec![stack(10, "apple", 4, 64)], &mut random).unwrap();
    assert_eq!(inventory.slots[1].stack.item.as_ref(), Some(&id("barrier")));
    assert_eq!(
        inventory
            .slots
            .iter()
            .filter(|slot| slot.stack.item.as_ref() == Some(&id("apple")))
            .map(|slot| slot.stack.count)
            .sum::<i32>(),
        4
    );
    assert_eq!(fill.written_slots.len(), 2);
    assert!(fill.overfill.is_empty());
    assert_eq!(random.bounds, [2, 1, 2, 2]);
    assert_eq!(random.boolean_calls, 1);

    let attempts = std::cell::Cell::new(0);
    let consumed = consume_generated_once(
        vec![stack(11, "apple", 1, 64), stack(12, "stone", 1, 64)],
        |stack| {
            attempts.set(attempts.get() + 1);
            stack.item.as_ref() == Some(&id("apple"))
        },
    );
    assert_eq!(attempts.get(), 2);
    assert_eq!((consumed.accepted.len(), consumed.refused.len()), (1, 1));
}
