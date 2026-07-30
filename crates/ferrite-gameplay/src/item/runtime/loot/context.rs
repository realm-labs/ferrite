//! Exact 26.2 loot-context parameter-set schemas.

use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;
use std::collections::BTreeMap;

pub const LOOT_CONTEXT_SET_COUNT: usize = 26;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LootParameter {
    ThisEntity,
    LastDamagePlayer,
    DamageSource,
    AttackingEntity,
    DirectAttackingEntity,
    Origin,
    BlockState,
    BlockEntity,
    Tool,
    ExplosionRadius,
    AdditionalCostComponentAllowed,
    TargetEntity,
    InteractingEntity,
    EnchantmentLevel,
    EnchantmentActive,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LootValue {
    Identity(u64),
    Integer(i64),
    Float(f32),
    Boolean(bool),
    Position([f64; 3]),
    Stack(ItemStack),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LootContextSet {
    Empty,
    Chest,
    Command,
    Selector,
    VillagerTrade,
    Fishing,
    Entity,
    Equipment,
    Archaeology,
    Gift,
    PiglinBarter,
    Vault,
    AdvancementReward,
    AdvancementEntity,
    AdvancementLocation,
    BlockUse,
    Generic,
    Block,
    Shearing,
    EntityInteract,
    BlockInteract,
    EnchantedDamage,
    EnchantedItem,
    EnchantedLocation,
    EnchantedEntity,
    HitBlock,
}

impl LootContextSet {
    pub const ALL: [Self; LOOT_CONTEXT_SET_COUNT] = [
        Self::Empty,
        Self::Chest,
        Self::Command,
        Self::Selector,
        Self::VillagerTrade,
        Self::Fishing,
        Self::Entity,
        Self::Equipment,
        Self::Archaeology,
        Self::Gift,
        Self::PiglinBarter,
        Self::Vault,
        Self::AdvancementReward,
        Self::AdvancementEntity,
        Self::AdvancementLocation,
        Self::BlockUse,
        Self::Generic,
        Self::Block,
        Self::Shearing,
        Self::EntityInteract,
        Self::BlockInteract,
        Self::EnchantedDamage,
        Self::EnchantedItem,
        Self::EnchantedLocation,
        Self::EnchantedEntity,
        Self::HitBlock,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Chest => "chest",
            Self::Command => "command",
            Self::Selector => "selector",
            Self::VillagerTrade => "villager_trade",
            Self::Fishing => "fishing",
            Self::Entity => "entity",
            Self::Equipment => "equipment",
            Self::Archaeology => "archaeology",
            Self::Gift => "gift",
            Self::PiglinBarter => "barter",
            Self::Vault => "vault",
            Self::AdvancementReward => "advancement_reward",
            Self::AdvancementEntity => "advancement_entity",
            Self::AdvancementLocation => "advancement_location",
            Self::BlockUse => "block_use",
            Self::Generic => "generic",
            Self::Block => "block",
            Self::Shearing => "shearing",
            Self::EntityInteract => "entity_interact",
            Self::BlockInteract => "block_interact",
            Self::EnchantedDamage => "enchanted_damage",
            Self::EnchantedItem => "enchanted_item",
            Self::EnchantedLocation => "enchanted_location",
            Self::EnchantedEntity => "enchanted_entity",
            Self::HitBlock => "hit_block",
        }
    }

    pub const fn required(self) -> &'static [LootParameter] {
        use LootParameter as P;
        match self {
            Self::Empty => &[],
            Self::Chest | Self::Command => &[P::Origin],
            Self::Selector => &[P::Origin, P::ThisEntity],
            Self::VillagerTrade => &[P::Origin, P::ThisEntity, P::AdditionalCostComponentAllowed],
            Self::Fishing => &[P::Origin, P::Tool],
            Self::Entity => &[P::ThisEntity, P::Origin, P::DamageSource],
            Self::Equipment | Self::Gift | Self::AdvancementReward | Self::AdvancementEntity => {
                &[P::Origin, P::ThisEntity]
            }
            Self::Archaeology | Self::Shearing => &[P::Origin, P::ThisEntity, P::Tool],
            Self::PiglinBarter => &[P::ThisEntity],
            Self::Vault => &[P::Origin],
            Self::AdvancementLocation => &[P::ThisEntity, P::Origin, P::Tool, P::BlockState],
            Self::BlockUse => &[P::ThisEntity, P::Origin, P::BlockState],
            Self::Generic => &[
                P::ThisEntity,
                P::LastDamagePlayer,
                P::DamageSource,
                P::AttackingEntity,
                P::DirectAttackingEntity,
                P::Origin,
                P::BlockState,
                P::BlockEntity,
                P::Tool,
                P::ExplosionRadius,
                P::AdditionalCostComponentAllowed,
            ],
            Self::Block => &[P::BlockState, P::Origin, P::Tool],
            Self::EntityInteract => &[P::TargetEntity, P::Tool],
            Self::BlockInteract => &[P::BlockState],
            Self::EnchantedDamage => &[
                P::ThisEntity,
                P::EnchantmentLevel,
                P::Origin,
                P::DamageSource,
            ],
            Self::EnchantedItem => &[P::Tool, P::EnchantmentLevel],
            Self::EnchantedLocation => &[
                P::ThisEntity,
                P::EnchantmentLevel,
                P::Origin,
                P::EnchantmentActive,
            ],
            Self::EnchantedEntity => &[P::ThisEntity, P::EnchantmentLevel, P::Origin],
            Self::HitBlock => &[P::ThisEntity, P::EnchantmentLevel, P::Origin, P::BlockState],
        }
    }

    pub const fn optional(self) -> &'static [LootParameter] {
        use LootParameter as P;
        match self {
            Self::Chest | Self::Command | Self::Fishing => &[P::ThisEntity],
            Self::Entity => &[
                P::AttackingEntity,
                P::DirectAttackingEntity,
                P::LastDamagePlayer,
            ],
            Self::Vault => &[P::ThisEntity, P::Tool],
            Self::Block => &[P::ThisEntity, P::BlockEntity, P::ExplosionRadius],
            Self::EntityInteract => &[P::InteractingEntity],
            Self::BlockInteract => &[P::BlockEntity, P::InteractingEntity, P::Tool],
            Self::EnchantedDamage => &[P::DirectAttackingEntity, P::AttackingEntity],
            _ => &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LootContext {
    pub parameter_set: LootContextSet,
    pub values: BTreeMap<LootParameter, LootValue>,
    pub dynamic_drops: BTreeMap<ResourceId, Vec<ItemStack>>,
    pub luck: f32,
}

impl LootContext {
    pub fn create(
        parameter_set: LootContextSet,
        values: BTreeMap<LootParameter, LootValue>,
        dynamic_drops: BTreeMap<ResourceId, Vec<ItemStack>>,
        luck: f32,
    ) -> Result<Self, LootContextError> {
        for required in parameter_set.required() {
            if !values.contains_key(required) {
                return Err(LootContextError::MissingRequired(*required));
            }
        }
        for provided in values.keys() {
            if !parameter_set.required().contains(provided)
                && !parameter_set.optional().contains(provided)
            {
                return Err(LootContextError::Disallowed(*provided));
            }
        }
        Ok(Self {
            parameter_set,
            values,
            dynamic_drops,
            luck,
        })
    }

    pub fn value(&self, parameter: LootParameter) -> Option<&LootValue> {
        self.values.get(&parameter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootContextError {
    MissingRequired(LootParameter),
    Disallowed(LootParameter),
}
