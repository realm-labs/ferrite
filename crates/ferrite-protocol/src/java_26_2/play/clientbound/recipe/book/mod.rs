//! Ghost-recipe and recipe-removal delta family.

pub mod codec;
pub mod projection;
pub mod publication;

use crate::java_26_2::play::clientbound::recipe::display::RecipeDisplay;

#[derive(Debug, Clone, PartialEq)]
pub struct PlaceGhostRecipe {
    pub container_id: i32,
    pub display: RecipeDisplay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeBookRemove {
    pub display_ids: Vec<i32>,
}
