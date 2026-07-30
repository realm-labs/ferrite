#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceRecipe {
    pub container_id: i32,
    pub display_id: i32,
    pub use_maximum_items: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeBookChangeSettings {
    pub book_type: RecipeBookType,
    pub open: bool,
    pub filtering: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeBookSeenRecipe {
    pub display_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

impl RecipeBookType {
    #[must_use]
    pub const fn from_wire(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Crafting),
            1 => Some(Self::Furnace),
            2 => Some(Self::BlastFurnace),
            3 => Some(Self::Smoker),
            _ => None,
        }
    }

    #[must_use]
    pub const fn to_wire(self) -> i32 {
        match self {
            Self::Crafting => 0,
            Self::Furnace => 1,
            Self::BlastFurnace => 2,
            Self::Smoker => 3,
        }
    }
}
