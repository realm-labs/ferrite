use std::collections::BTreeSet;

use crate::java_26_2::play::clientbound::recipe::RecipeBookSettings;
use crate::java_26_2::play::serverbound::recipe_book::packet::{
    RecipeBookChangeSettings, RecipeBookSeenRecipe, RecipeBookType,
};
use crate::java_26_2::play::serverbound::recipe_book::placement::RecipePlacementIndex;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerRecipeBook {
    pub settings: RecipeBookSettings,
    pub known: BTreeSet<Identifier>,
    pub highlighted: BTreeSet<Identifier>,
}

impl ServerRecipeBook {
    pub fn change_settings(&mut self, packet: RecipeBookChangeSettings) {
        set_setting(&mut self.settings, packet);
    }

    pub fn see_recipe(
        &mut self,
        index: &RecipePlacementIndex,
        packet: RecipeBookSeenRecipe,
    ) -> bool {
        let Some(entry) = index.resolve(packet.display_id) else {
            return false;
        };
        self.highlighted.remove(&entry.parent)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookClientEvent {
    ChangedSetting(RecipeBookChangeSettings),
    RemovedDisplayHighlight(i32),
    SentSettings(RecipeBookChangeSettings),
    SentSeen(RecipeBookSeenRecipe),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipeBookClientState {
    pub settings: RecipeBookSettings,
    pub highlighted_displays: BTreeSet<i32>,
    pub connected: bool,
    pub events: Vec<RecipeBookClientEvent>,
}

impl RecipeBookClientState {
    pub fn change_settings(
        &mut self,
        packet: RecipeBookChangeSettings,
    ) -> Option<RecipeBookChangeSettings> {
        set_setting(&mut self.settings, packet);
        self.events
            .push(RecipeBookClientEvent::ChangedSetting(packet));
        if !self.connected {
            return None;
        }
        self.events
            .push(RecipeBookClientEvent::SentSettings(packet));
        Some(packet)
    }

    pub fn see(&mut self, display_id: i32) -> Option<RecipeBookSeenRecipe> {
        if !self.highlighted_displays.remove(&display_id) {
            return None;
        }
        self.events
            .push(RecipeBookClientEvent::RemovedDisplayHighlight(display_id));
        if !self.connected {
            return None;
        }
        let packet = RecipeBookSeenRecipe { display_id };
        self.events.push(RecipeBookClientEvent::SentSeen(packet));
        Some(packet)
    }
}

fn set_setting(settings: &mut RecipeBookSettings, packet: RecipeBookChangeSettings) {
    match packet.book_type {
        RecipeBookType::Crafting => {
            settings.crafting_open = packet.open;
            settings.crafting_filtering = packet.filtering;
        }
        RecipeBookType::Furnace => {
            settings.furnace_open = packet.open;
            settings.furnace_filtering = packet.filtering;
        }
        RecipeBookType::BlastFurnace => {
            settings.blast_furnace_open = packet.open;
            settings.blast_furnace_filtering = packet.filtering;
        }
        RecipeBookType::Smoker => {
            settings.smoker_open = packet.open;
            settings.smoker_filtering = packet.filtering;
        }
    }
}
