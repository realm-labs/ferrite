use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::inventory_progression::packet::{
    Advancement, AdvancementHolder, AdvancementProgress, UpdateAdvancements,
};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq)]
struct AdvancementRecord {
    holder: AdvancementHolder,
    progress: AdvancementProgress,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancementPublisher {
    capacity: usize,
    records: BTreeMap<Identifier, AdvancementRecord>,
    visible: BTreeSet<Identifier>,
    dirty_progress: BTreeSet<Identifier>,
    dirty_visibility: bool,
    first_packet: bool,
}

impl AdvancementPublisher {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            records: BTreeMap::new(),
            visible: BTreeSet::new(),
            dirty_progress: BTreeSet::new(),
            dirty_visibility: true,
            first_packet: true,
        }
    }

    pub fn insert(
        &mut self,
        holder: AdvancementHolder,
        progress: AdvancementProgress,
    ) -> Result<(), AdvancementPublicationError> {
        if !self.records.contains_key(&holder.id) && self.records.len() == self.capacity {
            return Err(AdvancementPublicationError::Capacity {
                capacity: self.capacity,
            });
        }
        let id = holder.id.clone();
        self.records
            .insert(id.clone(), AdvancementRecord { holder, progress });
        self.dirty_progress.insert(id);
        self.dirty_visibility = true;
        Ok(())
    }

    pub fn update_progress(
        &mut self,
        id: &Identifier,
        progress: AdvancementProgress,
    ) -> Result<(), AdvancementPublicationError> {
        let record = self
            .records
            .get_mut(id)
            .ok_or_else(|| AdvancementPublicationError::UnknownAdvancement { id: id.clone() })?;
        record.progress = progress;
        self.dirty_progress.insert(id.clone());
        self.dirty_visibility = true;
        Ok(())
    }

    pub fn flush(
        &mut self,
        show_advancements: bool,
    ) -> Result<Option<UpdateAdvancements>, AdvancementPublicationError> {
        let mut added = Vec::new();
        let mut removed = BTreeSet::new();
        if self.dirty_visibility {
            let next_visible = self.evaluate_visibility()?;
            for id in next_visible.difference(&self.visible) {
                let record = self
                    .records
                    .get(id)
                    .expect("visible advancement has an authoritative record");
                added.push(record.holder.clone());
                self.dirty_progress.insert(id.clone());
            }
            removed.extend(self.visible.difference(&next_visible).cloned());
            self.visible = next_visible;
        }
        let mut progress = BTreeMap::new();
        for id in &self.dirty_progress {
            if self.visible.contains(id) {
                progress.insert(
                    id.clone(),
                    self.records
                        .get(id)
                        .expect("dirty advancement has an authoritative record")
                        .progress
                        .clone(),
                );
            }
        }
        self.dirty_progress.clear();
        self.dirty_visibility = false;

        let reset = self.first_packet;
        self.first_packet = false;
        if added.is_empty() && removed.is_empty() && progress.is_empty() {
            Ok(None)
        } else {
            Ok(Some(UpdateAdvancements {
                reset,
                added,
                removed,
                progress,
                show_advancements,
            }))
        }
    }

    #[must_use]
    pub fn is_visible(&self, id: &Identifier) -> bool {
        self.visible.contains(id)
    }

    fn evaluate_visibility(&self) -> Result<BTreeSet<Identifier>, AdvancementPublicationError> {
        let children = self.children();
        let mut subtree_complete = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        for id in self.records.keys() {
            self.compute_subtree_complete(id, &children, &mut subtree_complete, &mut visiting)?;
        }
        Ok(self
            .records
            .keys()
            .filter(|id| self.visible_by_rule(id, &subtree_complete))
            .cloned()
            .collect())
    }

    fn children(&self) -> BTreeMap<Identifier, Vec<Identifier>> {
        let mut children: BTreeMap<Identifier, Vec<Identifier>> = BTreeMap::new();
        for (id, record) in &self.records {
            if let Some(parent) = &record.holder.advancement.parent
                && self.records.contains_key(parent)
            {
                children.entry(parent.clone()).or_default().push(id.clone());
            }
        }
        children
    }

    fn compute_subtree_complete(
        &self,
        id: &Identifier,
        children: &BTreeMap<Identifier, Vec<Identifier>>,
        memo: &mut BTreeMap<Identifier, bool>,
        visiting: &mut BTreeSet<Identifier>,
    ) -> Result<bool, AdvancementPublicationError> {
        if let Some(value) = memo.get(id) {
            return Ok(*value);
        }
        if !visiting.insert(id.clone()) {
            return Err(AdvancementPublicationError::ParentCycle { id: id.clone() });
        }
        let record = self
            .records
            .get(id)
            .expect("visibility traversal starts from known records");
        let mut complete = progress_complete(&record.progress, &record.holder.advancement);
        if let Some(descendants) = children.get(id) {
            for child in descendants {
                complete |= self.compute_subtree_complete(child, children, memo, visiting)?;
            }
        }
        visiting.remove(id);
        memo.insert(id.clone(), complete);
        Ok(complete)
    }

    fn visible_by_rule(
        &self,
        id: &Identifier,
        subtree_complete: &BTreeMap<Identifier, bool>,
    ) -> bool {
        if subtree_complete.get(id).copied().unwrap_or(false) {
            return true;
        }
        let mut current = Some(id);
        for _ in 0..3 {
            let Some(candidate) = current else {
                break;
            };
            let Some(record) = self.records.get(candidate) else {
                break;
            };
            match visibility_rule(&record.holder.advancement, &record.progress) {
                VisibilityRule::Show => return true,
                VisibilityRule::Hide => return false,
                VisibilityRule::NoChange => {
                    current = record.holder.advancement.parent.as_ref();
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisibilityRule {
    Show,
    Hide,
    NoChange,
}

fn visibility_rule(advancement: &Advancement, progress: &AdvancementProgress) -> VisibilityRule {
    let Some(display) = &advancement.display else {
        return VisibilityRule::Hide;
    };
    if progress_complete(progress, advancement) {
        VisibilityRule::Show
    } else if display.hidden {
        VisibilityRule::Hide
    } else {
        VisibilityRule::NoChange
    }
}

fn progress_complete(progress: &AdvancementProgress, advancement: &Advancement) -> bool {
    !advancement.requirements.is_empty()
        && advancement.requirements.iter().all(|group| {
            group.iter().any(|criterion| {
                progress
                    .criteria
                    .get(criterion)
                    .is_some_and(Option::is_some)
            })
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AdvancementPublicationError {
    #[error("advancement publisher reached its {capacity}-record bound")]
    Capacity { capacity: usize },
    #[error("advancement {id} is absent from the publisher")]
    UnknownAdvancement { id: Identifier },
    #[error("advancement parent graph contains a cycle at {id}")]
    ParentCycle { id: Identifier },
}
