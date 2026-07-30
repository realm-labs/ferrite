//! Advancement requirement, listener, reward, visibility, persistence, and flush state.

use crate::item::runtime::progression::experience::ExperienceData;
use crate::item::runtime::random::{GameplayRandom, GameplayRandomError, checked_float};
use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;
use std::collections::{BTreeMap, BTreeSet};

pub const PERSISTENCE_DATA_FIX_FALLBACK: i32 = 1343;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementRequirements {
    pub groups: Vec<Vec<String>>,
}

impl AdvancementRequirements {
    pub fn all_of(criteria: impl IntoIterator<Item = String>) -> Self {
        Self {
            groups: criteria
                .into_iter()
                .map(|criterion| vec![criterion])
                .collect(),
        }
    }

    pub fn any_of(criteria: impl IntoIterator<Item = String>) -> Self {
        Self {
            groups: vec![criteria.into_iter().collect()],
        }
    }

    pub fn names(&self) -> BTreeSet<String> {
        self.groups.iter().flatten().cloned().collect()
    }

    pub fn test(&self, completed: impl Fn(&str) -> bool) -> bool {
        !self.groups.is_empty()
            && self
                .groups
                .iter()
                .all(|group| group.iter().any(|criterion| completed(criterion)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionProgress {
    pub obtained_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementProgress {
    pub criteria: BTreeMap<String, CriterionProgress>,
    pub requirements: AdvancementRequirements,
}

impl AdvancementProgress {
    pub fn new(requirements: AdvancementRequirements) -> Self {
        let mut progress = Self {
            criteria: BTreeMap::new(),
            requirements: AdvancementRequirements { groups: Vec::new() },
        };
        progress.update(requirements);
        progress
    }

    pub fn update(&mut self, requirements: AdvancementRequirements) {
        let names = requirements.names();
        self.criteria.retain(|name, _| names.contains(name));
        for name in names {
            self.criteria
                .entry(name)
                .or_insert(CriterionProgress { obtained_at: None });
        }
        self.requirements = requirements;
    }

    pub fn is_done(&self) -> bool {
        self.requirements.test(|criterion| {
            self.criteria
                .get(criterion)
                .is_some_and(|progress| progress.obtained_at.is_some())
        })
    }

    pub fn has_progress(&self) -> bool {
        self.criteria
            .values()
            .any(|progress| progress.obtained_at.is_some())
    }

    pub fn grant(&mut self, criterion: &str, obtained_at: i64) -> bool {
        let Some(progress) = self.criteria.get_mut(criterion) else {
            return false;
        };
        if progress.obtained_at.is_some() {
            return false;
        }
        progress.obtained_at = Some(obtained_at);
        true
    }

    pub fn revoke(&mut self, criterion: &str) -> bool {
        let Some(progress) = self.criteria.get_mut(criterion) else {
            return false;
        };
        if progress.obtained_at.take().is_none() {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementReward {
    pub experience: i32,
    pub loot_tables: Vec<ResourceId>,
    pub recipes: Vec<ResourceId>,
    pub function: Option<ResourceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvancementDisplay {
    pub announce_chat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementDefinition {
    pub key: ResourceId,
    pub root: ResourceId,
    pub requirements: AdvancementRequirements,
    pub reward: AdvancementReward,
    pub display: Option<AdvancementDisplay>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedAdvancementProgress {
    pub completed: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementTracker {
    definitions: Vec<AdvancementDefinition>,
    pub progress: BTreeMap<ResourceId, AdvancementProgress>,
    pub listeners: BTreeSet<(ResourceId, String)>,
    pub visible: BTreeSet<ResourceId>,
    pub progress_changed: BTreeSet<ResourceId>,
    pub roots_to_update: BTreeSet<ResourceId>,
    pub selected_tab: Option<ResourceId>,
    first_flush: bool,
}

impl AdvancementTracker {
    pub fn load(
        definitions: Vec<AdvancementDefinition>,
        saved: BTreeMap<ResourceId, SavedAdvancementProgress>,
    ) -> AdvancementLoad {
        let mut tracker = Self {
            definitions,
            progress: BTreeMap::new(),
            listeners: BTreeSet::new(),
            visible: BTreeSet::new(),
            progress_changed: BTreeSet::new(),
            roots_to_update: BTreeSet::new(),
            selected_tab: None,
            first_flush: true,
        };
        let mut unknown_saved = Vec::new();
        for (key, saved_progress) in saved {
            let Some(definition) = tracker.definition(&key).cloned() else {
                unknown_saved.push(key);
                continue;
            };
            let mut progress = AdvancementProgress::new(definition.requirements.clone());
            for (criterion, instant) in saved_progress.completed {
                progress.grant(&criterion, instant);
            }
            tracker.progress.insert(key.clone(), progress);
            tracker.progress_changed.insert(key);
            tracker.roots_to_update.insert(definition.root);
        }

        let mut automatic_rewards = Vec::new();
        for definition in tracker.definitions.clone() {
            tracker.ensure_progress(&definition.key);
            if definition.requirements.names().is_empty() {
                automatic_rewards.push(RewardRequest {
                    advancement: definition.key,
                    reward: definition.reward,
                });
            }
        }
        tracker.rebuild_listeners();
        AdvancementLoad {
            tracker,
            unknown_saved,
            automatic_rewards,
        }
    }

    pub fn award(
        &mut self,
        key: &ResourceId,
        criterion: &str,
        obtained_at: i64,
        show_advancement_messages: bool,
    ) -> ProgressMutation {
        let Some(definition) = self.definition(key).cloned() else {
            return ProgressMutation::unchanged();
        };
        self.ensure_progress(key);
        let progress = self.progress.get_mut(key).expect("progress was ensured");
        let was_done = progress.is_done();
        if !progress.grant(criterion, obtained_at) {
            return ProgressMutation::unchanged();
        }
        let is_done = progress.is_done();
        self.unregister_completed_listeners(key, is_done);
        self.progress_changed.insert(key.clone());
        if !was_done && is_done {
            self.roots_to_update.insert(definition.root);
        }
        ProgressMutation {
            changed: true,
            became_complete: !was_done && is_done,
            became_incomplete: false,
            reward: (!was_done && is_done).then_some(definition.reward),
            announce: !was_done
                && is_done
                && show_advancement_messages
                && definition
                    .display
                    .is_some_and(|display| display.announce_chat),
        }
    }

    pub fn revoke(&mut self, key: &ResourceId, criterion: &str) -> ProgressMutation {
        let Some(definition) = self.definition(key).cloned() else {
            return ProgressMutation::unchanged();
        };
        self.ensure_progress(key);
        let progress = self.progress.get_mut(key).expect("progress was ensured");
        let was_done = progress.is_done();
        if !progress.revoke(criterion) {
            return ProgressMutation::unchanged();
        }
        let is_done = progress.is_done();
        self.register_incomplete_listeners(key);
        self.progress_changed.insert(key.clone());
        if was_done && !is_done {
            self.roots_to_update.insert(definition.root);
        }
        ProgressMutation {
            changed: true,
            became_complete: false,
            became_incomplete: was_done && !is_done,
            reward: None,
            announce: false,
        }
    }

    pub fn flush(
        &mut self,
        show_advancements: bool,
        mut visible_for_root: impl FnMut(
            &ResourceId,
            &BTreeMap<ResourceId, AdvancementProgress>,
        ) -> BTreeSet<ResourceId>,
    ) -> Option<AdvancementPacket> {
        if !self.first_flush && self.roots_to_update.is_empty() && self.progress_changed.is_empty()
        {
            return None;
        }
        let mut added = BTreeSet::new();
        let mut removed = BTreeSet::new();
        for root in std::mem::take(&mut self.roots_to_update) {
            let root_members = self
                .definitions
                .iter()
                .filter(|definition| definition.root == root)
                .map(|definition| definition.key.clone())
                .collect::<BTreeSet<_>>();
            let desired = visible_for_root(&root, &self.progress);
            for key in root_members {
                match (self.visible.contains(&key), desired.contains(&key)) {
                    (false, true) => {
                        self.visible.insert(key.clone());
                        added.insert(key);
                    }
                    (true, false) => {
                        self.visible.remove(&key);
                        removed.insert(key);
                    }
                    _ => {}
                }
            }
        }
        let mut progress = BTreeMap::new();
        for key in std::mem::take(&mut self.progress_changed) {
            if self.visible.contains(&key)
                && let Some(value) = self.progress.get(&key)
            {
                progress.insert(key, value.clone());
            }
        }
        let reset = self.first_flush;
        self.first_flush = false;
        if added.is_empty() && removed.is_empty() && progress.is_empty() {
            None
        } else {
            Some(AdvancementPacket {
                reset,
                added,
                removed,
                progress,
                show_advancements,
            })
        }
    }

    pub fn select_tab(&mut self, requested: Option<&ResourceId>) -> Option<Option<ResourceId>> {
        let selected = requested.and_then(|key| {
            self.definition(key)
                .filter(|definition| {
                    definition.key == definition.root && definition.display.is_some()
                })
                .map(|definition| definition.key.clone())
        });
        if self.selected_tab == selected {
            None
        } else {
            self.selected_tab = selected.clone();
            Some(selected)
        }
    }

    pub fn save(&self) -> BTreeMap<ResourceId, SavedAdvancementProgress> {
        self.progress
            .iter()
            .filter(|(_, progress)| progress.has_progress())
            .map(|(key, progress)| {
                let completed = progress
                    .criteria
                    .iter()
                    .filter_map(|(criterion, progress)| {
                        progress
                            .obtained_at
                            .map(|instant| (criterion.clone(), instant))
                    })
                    .collect();
                (key.clone(), SavedAdvancementProgress { completed })
            })
            .collect()
    }

    fn definition(&self, key: &ResourceId) -> Option<&AdvancementDefinition> {
        self.definitions
            .iter()
            .find(|definition| &definition.key == key)
    }

    fn ensure_progress(&mut self, key: &ResourceId) {
        if self.progress.contains_key(key) {
            return;
        }
        if let Some(definition) = self.definition(key) {
            self.progress.insert(
                key.clone(),
                AdvancementProgress::new(definition.requirements.clone()),
            );
        }
    }

    fn rebuild_listeners(&mut self) {
        self.listeners.clear();
        for definition in self.definitions.clone() {
            self.register_incomplete_listeners(&definition.key);
        }
    }

    fn register_incomplete_listeners(&mut self, key: &ResourceId) {
        let Some(progress) = self.progress.get(key) else {
            return;
        };
        if progress.is_done() {
            return;
        }
        for (criterion, state) in &progress.criteria {
            if state.obtained_at.is_none() {
                self.listeners.insert((key.clone(), criterion.clone()));
            }
        }
    }

    fn unregister_completed_listeners(&mut self, key: &ResourceId, advancement_done: bool) {
        let Some(progress) = self.progress.get(key) else {
            return;
        };
        self.listeners.retain(|(listener_key, criterion)| {
            listener_key != key
                || (!advancement_done
                    && progress
                        .criteria
                        .get(criterion)
                        .is_some_and(|state| state.obtained_at.is_none()))
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementLoad {
    pub tracker: AdvancementTracker,
    pub unknown_saved: Vec<ResourceId>,
    pub automatic_rewards: Vec<RewardRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewardRequest {
    pub advancement: ResourceId,
    pub reward: AdvancementReward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressMutation {
    pub changed: bool,
    pub became_complete: bool,
    pub became_incomplete: bool,
    pub reward: Option<AdvancementReward>,
    pub announce: bool,
}

impl ProgressMutation {
    fn unchanged() -> Self {
        Self {
            changed: false,
            became_complete: false,
            became_incomplete: false,
            reward: None,
            announce: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementPacket {
    pub reset: bool,
    pub added: BTreeSet<ResourceId>,
    pub removed: BTreeSet<ResourceId>,
    pub progress: BTreeMap<ResourceId, AdvancementProgress>,
    pub show_advancements: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RewardDelivery {
    pub events: Vec<RewardEvent>,
    pub dropped: Vec<ItemStack>,
    pub pickup_pitches: Vec<f32>,
    pub broadcast_inventory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewardEvent {
    Experience(i32),
    LootTable(ResourceId),
    PickupSound,
    InventoryBroadcast,
    Recipes(Vec<ResourceId>),
    Function(ResourceId),
}

pub fn deliver_reward(
    reward: &AdvancementReward,
    experience: &mut ExperienceData,
    random: &mut dyn GameplayRandom,
    mut generate_loot: impl FnMut(&ResourceId) -> Vec<ItemStack>,
    mut insert: impl FnMut(&mut ItemStack) -> bool,
) -> Result<RewardDelivery, GameplayRandomError> {
    let mut events = vec![RewardEvent::Experience(reward.experience)];
    experience.give_points(reward.experience);
    let mut dropped = Vec::new();
    let mut pickup_pitches = Vec::new();
    let mut broadcast_inventory = false;
    for table in &reward.loot_tables {
        events.push(RewardEvent::LootTable(table.clone()));
        for mut stack in generate_loot(table) {
            if insert(&mut stack) {
                let pitch = ((checked_float(random)? - checked_float(random)?) * 0.7 + 1.0) * 2.0;
                pickup_pitches.push(pitch);
                events.push(RewardEvent::PickupSound);
                broadcast_inventory = true;
            } else {
                dropped.push(stack);
            }
        }
    }
    if broadcast_inventory {
        events.push(RewardEvent::InventoryBroadcast);
    }
    if !reward.recipes.is_empty() {
        events.push(RewardEvent::Recipes(reward.recipes.clone()));
    }
    if let Some(function) = &reward.function {
        events.push(RewardEvent::Function(function.clone()));
    }
    Ok(RewardDelivery {
        events,
        dropped,
        pickup_pitches,
        broadcast_inventory,
    })
}
