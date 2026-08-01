use std::collections::BTreeMap;

use thiserror::Error;

use crate::java_26_2::value::identifier::Identifier;

pub const COMMAND_ARGUMENT_TYPE: &str = "minecraft:command_argument_type";
pub const ATTRIBUTE: &str = "minecraft:attribute";
pub const BIOME: &str = "minecraft:worldgen/biome";
pub const BLOCK: &str = "minecraft:block";
pub const CAT_SOUND_VARIANT: &str = "minecraft:cat_sound_variant";
pub const CAT_VARIANT: &str = "minecraft:cat_variant";
pub const CHAT_TYPE: &str = "minecraft:chat_type";
pub const CHICKEN_SOUND_VARIANT: &str = "minecraft:chicken_sound_variant";
pub const CHICKEN_VARIANT: &str = "minecraft:chicken_variant";
pub const COW_SOUND_VARIANT: &str = "minecraft:cow_sound_variant";
pub const COW_VARIANT: &str = "minecraft:cow_variant";
pub const CUSTOM_STAT: &str = "minecraft:custom_stat";
pub const DATA_COMPONENT_TYPE: &str = "minecraft:data_component_type";
pub const DAMAGE_TYPE: &str = "minecraft:damage_type";
pub const DIMENSION_TYPE: &str = "minecraft:dimension_type";
pub const ENTITY_TYPE: &str = "minecraft:entity_type";
pub const ITEM: &str = "minecraft:item";
pub const MAP_DECORATION_TYPE: &str = "minecraft:map_decoration_type";
pub const MENU: &str = "minecraft:menu";
pub const MOB_EFFECT: &str = "minecraft:mob_effect";
pub const NUMBER_FORMAT_TYPE: &str = "minecraft:number_format_type";
pub const PAINTING_VARIANT: &str = "minecraft:painting_variant";
pub const PIG_SOUND_VARIANT: &str = "minecraft:pig_sound_variant";
pub const PIG_VARIANT: &str = "minecraft:pig_variant";
pub const RECIPE_BOOK_CATEGORY: &str = "minecraft:recipe_book_category";
pub const RECIPE_DISPLAY: &str = "minecraft:recipe_display";
pub const SLOT_DISPLAY: &str = "minecraft:slot_display";
pub const SOUND_EVENT: &str = "minecraft:sound_event";
pub const STAT_TYPE: &str = "minecraft:stat_type";
pub const TRIM_PATTERN: &str = "minecraft:trim_pattern";
pub const VILLAGER_PROFESSION: &str = "minecraft:villager_profession";
pub const VILLAGER_TYPE: &str = "minecraft:villager_type";
pub const WOLF_SOUND_VARIANT: &str = "minecraft:wolf_sound_variant";
pub const WOLF_VARIANT: &str = "minecraft:wolf_variant";
pub const WORLD_CLOCK: &str = "minecraft:world_clock";
pub const ZOMBIE_NAUTILUS_VARIANT: &str = "minecraft:zombie_nautilus_variant";
pub const FROG_VARIANT: &str = "minecraft:frog_variant";

/// Connection-local numeric registry projections reconstructed during configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayRegistries {
    entries: BTreeMap<Identifier, Vec<Identifier>>,
}

impl PlayRegistries {
    #[must_use]
    pub fn new(entries: BTreeMap<Identifier, Vec<Identifier>>) -> Self {
        Self { entries }
    }

    pub fn insert(&mut self, registry: Identifier, values: Vec<Identifier>) {
        self.entries.insert(registry, values);
    }

    pub fn resolve(
        &self,
        registry: &'static str,
        raw_id: i32,
    ) -> Result<Identifier, PlayRegistryError> {
        let index = usize::try_from(raw_id)
            .map_err(|_| PlayRegistryError::UnknownRawId { registry, raw_id })?;
        self.table(registry)?
            .get(index)
            .cloned()
            .ok_or(PlayRegistryError::UnknownRawId { registry, raw_id })
    }

    pub fn raw_id(
        &self,
        registry: &'static str,
        identity: &Identifier,
    ) -> Result<i32, PlayRegistryError> {
        let index = self
            .table(registry)?
            .iter()
            .position(|candidate| candidate == identity)
            .ok_or_else(|| PlayRegistryError::UnknownIdentity {
                registry,
                identity: identity.clone(),
            })?;
        i32::try_from(index).map_err(|_| PlayRegistryError::RegistryTooLarge {
            registry,
            entries: index.saturating_add(1),
        })
    }

    fn table(&self, registry: &'static str) -> Result<&[Identifier], PlayRegistryError> {
        let key = Identifier::parse(registry).expect("locked registry identity is valid");
        self.entries
            .get(&key)
            .map(Vec::as_slice)
            .ok_or(PlayRegistryError::MissingRegistry { registry })
    }

    pub fn len(&self, registry: &'static str) -> Result<usize, PlayRegistryError> {
        Ok(self.table(registry)?.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlayRegistryError {
    #[error("play registry snapshot is missing {registry}")]
    MissingRegistry { registry: &'static str },
    #[error("raw ID {raw_id} is absent from play registry {registry}")]
    UnknownRawId { registry: &'static str, raw_id: i32 },
    #[error("identity {identity} is absent from play registry {registry}")]
    UnknownIdentity {
        registry: &'static str,
        identity: Identifier,
    },
    #[error("play registry {registry} has {entries} entries, beyond signed VarInt indexing")]
    RegistryTooLarge {
        registry: &'static str,
        entries: usize,
    },
}
