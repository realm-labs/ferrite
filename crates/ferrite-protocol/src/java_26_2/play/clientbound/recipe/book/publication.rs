use std::collections::BTreeSet;

use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::clientbound::recipe::book::{PlaceGhostRecipe, RecipeBookRemove};
use crate::java_26_2::play::clientbound::recipe::display::RecipeDisplay;
use crate::java_26_2::play::item::ItemStack;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeDisplaySource {
    pub parent: Identifier,
    pub display: RecipeDisplay,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexedRecipeDisplay {
    pub display_id: i32,
    pub parent: Identifier,
    pub display: RecipeDisplay,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecipeDisplayIndex {
    entries: Vec<IndexedRecipeDisplay>,
}

impl RecipeDisplayIndex {
    pub fn rebuild(
        sources: impl IntoIterator<Item = RecipeDisplaySource>,
    ) -> Result<Self, RecipeDisplayIndexError> {
        let mut entries = Vec::new();
        for source in sources {
            if !source.enabled {
                continue;
            }
            let display_id = i32::try_from(entries.len())
                .map_err(|_| RecipeDisplayIndexError::TooManyDisplays)?;
            entries.push(IndexedRecipeDisplay {
                display_id,
                parent: source.parent,
                display: source.display,
            });
        }
        Ok(Self { entries })
    }

    #[must_use]
    pub fn resolve(&self, display_id: i32) -> Option<&IndexedRecipeDisplay> {
        usize::try_from(display_id)
            .ok()
            .and_then(|index| self.entries.get(index))
    }

    #[must_use]
    pub fn display_ids_for_parent(&self, parent: &Identifier) -> Vec<i32> {
        self.entries
            .iter()
            .filter(|entry| &entry.parent == parent)
            .map(|entry| entry.display_id)
            .collect()
    }

    #[must_use]
    pub fn entries(&self) -> &[IndexedRecipeDisplay] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeBookPublisher {
    index: RecipeDisplayIndex,
    known: BTreeSet<Identifier>,
    highlighted: BTreeSet<Identifier>,
}

impl RecipeBookPublisher {
    #[must_use]
    pub fn new(index: RecipeDisplayIndex) -> Self {
        Self {
            index,
            known: BTreeSet::new(),
            highlighted: BTreeSet::new(),
        }
    }

    pub fn mark_known(&mut self, parent: Identifier, highlighted: bool) {
        if highlighted {
            self.highlighted.insert(parent.clone());
        }
        self.known.insert(parent);
    }

    #[must_use]
    pub fn remove_recipes(&mut self, parents: &[Identifier]) -> RecipeRemovalPublication {
        let mut display_ids = Vec::new();
        for parent in parents {
            if !self.known.remove(parent) {
                continue;
            }
            self.highlighted.remove(parent);
            display_ids.extend(self.index.display_ids_for_parent(parent));
        }
        let removed_display_count = display_ids.len();
        let packet = (!display_ids.is_empty()).then_some(PlayClientboundPacket::RecipeBookRemove(
            RecipeBookRemove { display_ids },
        ));
        RecipeRemovalPublication {
            removed_display_count,
            packet,
        }
    }

    #[must_use]
    pub fn known(&self) -> &BTreeSet<Identifier> {
        &self.known
    }

    #[must_use]
    pub fn highlighted(&self) -> &BTreeSet<Identifier> {
        &self.highlighted
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeRemovalPublication {
    pub removed_display_count: usize,
    pub packet: Option<PlayClientboundPacket>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostPublicationStep {
    ReturnedInputs,
    ClearedGrid,
    SentGhost,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GhostPlacementPublication {
    pub returned_inputs: Vec<ItemStack>,
    pub packet: PlayClientboundPacket,
    pub steps: Vec<GhostPublicationStep>,
}

#[must_use]
pub fn publish_failed_placement(
    container_id: i32,
    display: &RecipeDisplay,
    crafting_inputs: &mut [ItemStack],
) -> GhostPlacementPublication {
    let returned_inputs = crafting_inputs
        .iter()
        .filter(|stack| !stack.is_empty())
        .cloned()
        .collect();
    crafting_inputs.fill(ItemStack::Empty);
    GhostPlacementPublication {
        returned_inputs,
        packet: PlayClientboundPacket::PlaceGhostRecipe(Box::new(PlaceGhostRecipe {
            container_id,
            display: display.clone(),
        })),
        steps: vec![
            GhostPublicationStep::ReturnedInputs,
            GhostPublicationStep::ClearedGrid,
            GhostPublicationStep::SentGhost,
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeDisplayIndexError {
    TooManyDisplays,
}
