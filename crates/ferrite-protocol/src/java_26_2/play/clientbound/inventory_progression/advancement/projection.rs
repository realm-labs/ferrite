use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::inventory_progression::packet::{
    Advancement, AdvancementHolder, AdvancementProgress, UpdateAdvancements,
};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq)]
struct AdvancementNode {
    holder: AdvancementHolder,
    parent: Option<u64>,
    children: BTreeSet<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedAdvancementProgress {
    pub criteria: BTreeMap<String, Option<i64>>,
    pub requirements: Vec<Vec<String>>,
    pub complete: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementProjectionAction {
    pub removed_unknown: Vec<Identifier>,
    pub unresolved_added: Vec<Identifier>,
    pub unknown_progress: Vec<Identifier>,
    pub listener_notifications: Vec<Identifier>,
    pub telemetry: Vec<Identifier>,
    pub toasts: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancementClientProjection {
    maximum_nodes: usize,
    maximum_progress: usize,
    next_node: u64,
    nodes: BTreeMap<u64, AdvancementNode>,
    lookup: BTreeMap<Identifier, u64>,
    roots: BTreeSet<u64>,
    progress: BTreeMap<Identifier, NormalizedAdvancementProgress>,
    selected_tab: Option<Identifier>,
}

impl AdvancementClientProjection {
    #[must_use]
    pub fn new(maximum_nodes: usize, maximum_progress: usize) -> Self {
        Self {
            maximum_nodes,
            maximum_progress,
            next_node: 0,
            nodes: BTreeMap::new(),
            lookup: BTreeMap::new(),
            roots: BTreeSet::new(),
            progress: BTreeMap::new(),
            selected_tab: None,
        }
    }

    pub fn apply(
        &mut self,
        packet: &UpdateAdvancements,
        level_exists: bool,
    ) -> Result<AdvancementProjectionAction, AdvancementProjectionError> {
        let mut action = AdvancementProjectionAction::default();
        if packet.reset {
            self.nodes.clear();
            self.lookup.clear();
            self.roots.clear();
            self.progress.clear();
        }
        for removed in &packet.removed {
            if let Some(node) = self.lookup.get(removed).copied() {
                self.remove_node(node);
            } else {
                action.removed_unknown.push(removed.clone());
            }
        }
        self.add_all(&packet.added, &mut action)?;
        for (id, progress) in &packet.progress {
            let Some(node) = self.lookup.get(id).copied() else {
                action.unknown_progress.push(id.clone());
                continue;
            };
            let advancement = &self
                .nodes
                .get(&node)
                .expect("lookup points to a live advancement node")
                .holder
                .advancement;
            let normalized = normalize_progress(progress, advancement);
            let complete = normalized.complete;
            if !self.progress.contains_key(id) && self.progress.len() == self.maximum_progress {
                return Err(AdvancementProjectionError::ProgressCapacity {
                    capacity: self.maximum_progress,
                });
            }
            self.progress.insert(id.clone(), normalized);
            action.listener_notifications.push(id.clone());
            if !packet.reset && complete {
                if level_exists {
                    action.telemetry.push(id.clone());
                }
                if packet.show_advancements
                    && advancement
                        .display
                        .as_ref()
                        .is_some_and(|display| display.show_toast)
                {
                    action.toasts.push(id.clone());
                }
            }
        }
        Ok(action)
    }

    pub fn select_tab(&mut self, selected_tab: Option<Identifier>) {
        self.selected_tab = selected_tab;
    }

    #[must_use]
    pub fn selected_tab(&self) -> Option<&Identifier> {
        self.selected_tab.as_ref()
    }

    #[must_use]
    pub fn contains(&self, id: &Identifier) -> bool {
        self.lookup.contains_key(id)
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    #[must_use]
    pub fn progress(&self, id: &Identifier) -> Option<&NormalizedAdvancementProgress> {
        self.progress.get(id)
    }

    fn add_all(
        &mut self,
        added: &[AdvancementHolder],
        action: &mut AdvancementProjectionAction,
    ) -> Result<(), AdvancementProjectionError> {
        let mut pending: Vec<&AdvancementHolder> = added.iter().collect();
        loop {
            let before = pending.len();
            let mut deferred = Vec::new();
            for holder in pending {
                let parent = holder
                    .advancement
                    .parent
                    .as_ref()
                    .and_then(|parent| self.lookup.get(parent).copied());
                if holder.advancement.parent.is_some() && parent.is_none() {
                    deferred.push(holder);
                } else {
                    self.insert_node(holder.clone(), parent)?;
                }
            }
            pending = deferred;
            if pending.is_empty() || pending.len() == before {
                break;
            }
        }
        action
            .unresolved_added
            .extend(pending.into_iter().map(|holder| holder.id.clone()));
        Ok(())
    }

    fn insert_node(
        &mut self,
        holder: AdvancementHolder,
        parent: Option<u64>,
    ) -> Result<(), AdvancementProjectionError> {
        if self.nodes.len() == self.maximum_nodes {
            return Err(AdvancementProjectionError::NodeCapacity {
                capacity: self.maximum_nodes,
            });
        }
        let node = self.next_node;
        self.next_node = self.next_node.wrapping_add(1);
        self.nodes.insert(
            node,
            AdvancementNode {
                holder: holder.clone(),
                parent,
                children: BTreeSet::new(),
            },
        );
        self.lookup.insert(holder.id, node);
        if let Some(parent) = parent {
            self.nodes
                .get_mut(&parent)
                .expect("resolved parent remains present")
                .children
                .insert(node);
        } else {
            self.roots.insert(node);
        }
        Ok(())
    }

    fn remove_node(&mut self, node: u64) {
        let children = self
            .nodes
            .get(&node)
            .map(|node| node.children.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        for child in children {
            self.remove_node(child);
        }
        let Some(removed) = self.nodes.remove(&node) else {
            return;
        };
        if let Some(parent) = removed.parent {
            if let Some(parent) = self.nodes.get_mut(&parent) {
                parent.children.remove(&node);
            }
        } else {
            self.roots.remove(&node);
        }
        if self.lookup.get(&removed.holder.id) == Some(&node) {
            self.lookup.remove(&removed.holder.id);
        }
    }
}

fn normalize_progress(
    progress: &AdvancementProgress,
    advancement: &Advancement,
) -> NormalizedAdvancementProgress {
    let required: BTreeSet<&str> = advancement
        .requirements
        .iter()
        .flatten()
        .map(String::as_str)
        .collect();
    let mut criteria = progress.criteria.clone();
    criteria.retain(|name, _| required.contains(name.as_str()));
    for name in required {
        criteria.entry(name.to_owned()).or_insert(None);
    }
    let complete = !advancement.requirements.is_empty()
        && advancement.requirements.iter().all(|group| {
            group
                .iter()
                .any(|name| criteria.get(name).is_some_and(Option::is_some))
        });
    NormalizedAdvancementProgress {
        criteria,
        requirements: advancement.requirements.clone(),
        complete,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AdvancementProjectionError {
    #[error("advancement tree reached its {capacity}-node bound")]
    NodeCapacity { capacity: usize },
    #[error("advancement progress reached its {capacity}-entry bound")]
    ProgressCapacity { capacity: usize },
}
