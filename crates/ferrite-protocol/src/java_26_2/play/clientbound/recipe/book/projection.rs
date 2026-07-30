use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::clientbound::recipe::book::{PlaceGhostRecipe, RecipeBookRemove};
use crate::java_26_2::play::clientbound::recipe::display::RecipeDisplay;
use crate::java_26_2::play::clientbound::recipe::{RecipeBookAdd, RecipeBookEntry};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecipeBookClientProjection {
    known: BTreeMap<i32, RecipeBookEntry>,
    highlights: BTreeSet<i32>,
    current_container_id: Option<i32>,
    current_screen_updates_recipes: bool,
    ghost: Option<RecipeDisplay>,
    last_removal_order: Vec<i32>,
    collection_refreshes: usize,
    search_refreshes: usize,
    screen_refreshes: usize,
}

impl RecipeBookClientProjection {
    pub fn install_add(&mut self, packet: &RecipeBookAdd) {
        if packet.replace {
            self.known.clear();
            self.highlights.clear();
        }
        for entry in &packet.entries {
            self.known.insert(entry.display_id, entry.clone());
            if entry.highlight {
                self.highlights.insert(entry.display_id);
            }
        }
    }

    pub fn open_menu(&mut self, container_id: i32, screen_updates_recipes: bool) {
        self.current_container_id = Some(container_id);
        self.current_screen_updates_recipes = screen_updates_recipes;
        self.ghost = None;
    }

    pub fn close_menu(&mut self) {
        self.current_container_id = None;
        self.current_screen_updates_recipes = false;
        self.ghost = None;
    }

    pub fn apply(
        &mut self,
        packet: &PlayClientboundPacket,
    ) -> Result<RecipeBookClientAction, RecipeBookProjectionError> {
        match packet {
            PlayClientboundPacket::PlaceGhostRecipe(packet) => Ok(self.apply_ghost(packet)),
            PlayClientboundPacket::RecipeBookRemove(packet) => {
                self.apply_remove(packet);
                Ok(RecipeBookClientAction::Refreshed)
            }
            _ => Err(RecipeBookProjectionError::WrongPacketFamily),
        }
    }

    #[must_use]
    pub fn known(&self) -> &BTreeMap<i32, RecipeBookEntry> {
        &self.known
    }

    #[must_use]
    pub fn highlights(&self) -> &BTreeSet<i32> {
        &self.highlights
    }

    #[must_use]
    pub const fn ghost(&self) -> Option<&RecipeDisplay> {
        self.ghost.as_ref()
    }

    #[must_use]
    pub fn last_removal_order(&self) -> &[i32] {
        &self.last_removal_order
    }

    #[must_use]
    pub const fn collection_refreshes(&self) -> usize {
        self.collection_refreshes
    }

    #[must_use]
    pub const fn search_refreshes(&self) -> usize {
        self.search_refreshes
    }

    #[must_use]
    pub const fn screen_refreshes(&self) -> usize {
        self.screen_refreshes
    }

    fn apply_ghost(&mut self, packet: &PlaceGhostRecipe) -> RecipeBookClientAction {
        if self.current_container_id != Some(packet.container_id)
            || !self.current_screen_updates_recipes
        {
            return RecipeBookClientAction::Ignored;
        }
        self.ghost = Some(packet.display.clone());
        RecipeBookClientAction::GhostReplaced
    }

    fn apply_remove(&mut self, packet: &RecipeBookRemove) {
        self.last_removal_order.clone_from(&packet.display_ids);
        for display_id in &packet.display_ids {
            self.known.remove(display_id);
            self.highlights.remove(display_id);
        }
        self.collection_refreshes = self.collection_refreshes.saturating_add(1);
        self.search_refreshes = self.search_refreshes.saturating_add(1);
        if self.current_screen_updates_recipes {
            self.screen_refreshes = self.screen_refreshes.saturating_add(1);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookClientAction {
    Ignored,
    GhostReplaced,
    Refreshed,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecipeBookProjectionError {
    #[error("packet does not belong to the recipe-book delta family")]
    WrongPacketFamily,
}
