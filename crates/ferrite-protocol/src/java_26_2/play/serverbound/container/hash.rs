use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::java_26_2::play::item::{EncodedComponentValue, ItemStack};
use crate::java_26_2::play::serverbound::container::packet::{
    HashedComponentPatch, HashedStack, HashedStackContents,
};
use crate::java_26_2::value::identifier::Identifier;

const COMPONENT_HASH_CACHE_CAPACITY: usize = 256;
const CRC32C_POLYNOMIAL: u32 = 0x82f6_3b78;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ComponentCacheKey {
    component: Identifier,
    encoded_value: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComponentHashCache {
    values: BTreeMap<ComponentCacheKey, i32>,
    insertion_order: VecDeque<ComponentCacheKey>,
}

impl ComponentHashCache {
    #[must_use]
    pub fn hash_stack(&mut self, stack: &ItemStack) -> HashedStack {
        let Some(contents) = stack.contents() else {
            return HashedStack::Empty;
        };
        if contents.count <= 0
            || (contents.item.namespace() == "minecraft" && contents.item.path() == "air")
        {
            return HashedStack::Empty;
        }
        let added = contents
            .components
            .added
            .iter()
            .map(|component| (component.component.clone(), self.hash_component(component)))
            .collect();
        let removed = contents.components.removed.iter().cloned().collect();
        HashedStack::Present(HashedStackContents {
            item: contents.item.clone(),
            count: contents.count,
            components: HashedComponentPatch { added, removed },
        })
    }

    #[must_use]
    pub fn matches(&mut self, expected: &HashedStack, actual: &ItemStack) -> bool {
        self.hash_stack(actual) == *expected
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn hash_component(&mut self, component: &EncodedComponentValue) -> i32 {
        let key = ComponentCacheKey {
            component: component.component.clone(),
            encoded_value: component.encoded_value.clone(),
        };
        if let Some(hash) = self.values.get(&key) {
            return *hash;
        }
        let hash = crc32c(&component.encoded_value) as i32;
        if self.values.len() == COMPONENT_HASH_CACHE_CAPACITY
            && let Some(evicted) = self.insertion_order.pop_front()
        {
            self.values.remove(&evicted);
        }
        self.insertion_order.push_back(key.clone());
        self.values.insert(key, hash);
        hash
    }
}

#[must_use]
pub fn hashed_patch(
    added: BTreeMap<Identifier, i32>,
    removed: BTreeSet<Identifier>,
) -> HashedComponentPatch {
    HashedComponentPatch { added, removed }
}

#[must_use]
pub fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (CRC32C_POLYNOMIAL & (0_u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}
