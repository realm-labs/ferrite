#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveTagsPacketKind {
    UpdateTags,
}

impl LiveTagsPacketKind {
    pub const ALL: [Self; 1] = [Self::UpdateTags];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::UpdateTags => 134,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::UpdateTags => "minecraft:update_tags",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveTagReloadStep {
    Tags,
    Recipes,
    RecipeBook,
}

impl LiveTagReloadStep {
    pub const ORDER: [Self; 3] = [Self::Tags, Self::Recipes, Self::RecipeBook];
}
