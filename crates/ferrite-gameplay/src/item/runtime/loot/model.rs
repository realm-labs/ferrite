//! Codec-neutral loot descriptors consumed by registry-backed dispatch.

use crate::item::runtime::loot::context::LootContextSet;
use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootDataKind {
    Predicate,
    Modifier,
    Table,
}

impl LootDataKind {
    pub const ALL: [Self; 3] = [Self::Predicate, Self::Modifier, Self::Table];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootCondition {
    pub type_id: ResourceId,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootFunction {
    pub type_id: ResourceId,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootNumberProvider {
    pub type_id: ResourceId,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootEntry {
    pub type_id: ResourceId,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootPool {
    pub entries: Vec<LootEntry>,
    pub conditions: Vec<LootCondition>,
    pub functions: Vec<LootFunction>,
    pub rolls: LootNumberProvider,
    pub bonus_rolls: LootNumberProvider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootTable {
    pub key: ResourceId,
    pub parameter_set: LootContextSet,
    pub random_sequence: Option<ResourceId>,
    pub pools: Vec<LootPool>,
    pub functions: Vec<LootFunction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpandedLootEntry {
    pub handle: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootOutput {
    Stack(ItemStack),
    Table(ResourceId),
    DynamicDrop(ResourceId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LootRandomOwner {
    ExplicitSource,
    ExplicitSeed(u64),
    TableSequence(ResourceId),
    Level,
}

pub fn resolve_random_owner(
    explicit_source: bool,
    optional_seed: u64,
    table_sequence: Option<&ResourceId>,
) -> LootRandomOwner {
    if explicit_source {
        LootRandomOwner::ExplicitSource
    } else if optional_seed != 0 {
        LootRandomOwner::ExplicitSeed(optional_seed)
    } else if let Some(sequence) = table_sequence {
        LootRandomOwner::TableSequence(sequence.clone())
    } else {
        LootRandomOwner::Level
    }
}
