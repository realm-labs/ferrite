//! Knowledge-book consume-before-validation and ordered recipe unlocking.

use ferrite_foundation::resource::ResourceId;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeDisposition {
    Unlockable,
    AlreadyKnown,
    Special,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeResolution {
    pub key: ResourceId,
    pub disposition: RecipeDisposition,
    pub displays: Vec<ResourceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeBookResult {
    Fail,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeBookOutcome {
    pub result: KnowledgeBookResult,
    pub consumed: u8,
    pub queried: Vec<ResourceId>,
    pub first_missing: Option<ResourceId>,
    pub newly_known: Vec<ResourceId>,
    pub display_packet: Vec<ResourceId>,
    pub item_used_stat: bool,
}

pub fn use_knowledge_book(
    recipes: &[ResourceId],
    infinite_materials: bool,
    client_side: bool,
    mut resolve: impl FnMut(&ResourceId) -> RecipeResolution,
) -> KnowledgeBookOutcome {
    let consumed = u8::from(!infinite_materials);
    if recipes.is_empty() {
        return failed(consumed, Vec::new(), None);
    }
    if client_side {
        return KnowledgeBookOutcome {
            result: KnowledgeBookResult::Success,
            consumed,
            queried: Vec::new(),
            first_missing: None,
            newly_known: Vec::new(),
            display_packet: Vec::new(),
            item_used_stat: false,
        };
    }

    let mut queried = Vec::with_capacity(recipes.len());
    let mut resolved = Vec::with_capacity(recipes.len());
    for key in recipes {
        queried.push(key.clone());
        let recipe = resolve(key);
        if recipe.disposition == RecipeDisposition::Missing {
            return failed(consumed, queried, Some(key.clone()));
        }
        resolved.push(recipe);
    }

    let mut newly_known = Vec::new();
    let mut display_packet = Vec::new();
    let mut awarded = BTreeSet::new();
    for recipe in resolved {
        if recipe.disposition == RecipeDisposition::Unlockable && awarded.insert(recipe.key.clone())
        {
            newly_known.push(recipe.key);
            display_packet.extend(recipe.displays);
        }
    }
    KnowledgeBookOutcome {
        result: KnowledgeBookResult::Success,
        consumed,
        queried,
        first_missing: None,
        newly_known,
        display_packet,
        item_used_stat: true,
    }
}

fn failed(
    consumed: u8,
    queried: Vec<ResourceId>,
    first_missing: Option<ResourceId>,
) -> KnowledgeBookOutcome {
    KnowledgeBookOutcome {
        result: KnowledgeBookResult::Fail,
        consumed,
        queried,
        first_missing,
        newly_known: Vec::new(),
        display_packet: Vec::new(),
        item_used_stat: false,
    }
}
