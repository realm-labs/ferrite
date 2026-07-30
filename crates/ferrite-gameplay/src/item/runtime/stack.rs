//! Shared item-stack identity and after-use component processing.

use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub identity: u64,
    pub item: Option<ResourceId>,
    pub count: i32,
    pub maximum: i32,
    pub component_fingerprint: u64,
}

impl ItemStack {
    pub const fn empty() -> Self {
        Self {
            identity: 0,
            item: None,
            count: 0,
            maximum: 64,
            component_fingerprint: 0,
        }
    }

    pub fn new(
        identity: u64,
        item: ResourceId,
        count: i32,
        maximum: i32,
        component_fingerprint: u64,
    ) -> Self {
        Self {
            identity,
            item: Some(item),
            count,
            maximum,
            component_fingerprint,
        }
        .normalized()
    }

    pub fn normalized(mut self) -> Self {
        if self.item.is_none() || self.count <= 0 {
            return Self::empty();
        }
        self.count = self.count.min(self.maximum);
        self
    }

    pub const fn is_empty(&self) -> bool {
        self.item.is_none() || self.count <= 0
    }

    pub fn same_item(&self, other: &Self) -> bool {
        !self.is_empty() && self.item == other.item
    }

    pub fn equal_stack(&self, other: &Self) -> bool {
        self.item == other.item
            && self.count == other.count
            && self.component_fingerprint == other.component_fingerprint
    }

    pub fn compatible_with(&self, other: &Self) -> bool {
        self.same_item(other) && self.component_fingerprint == other.component_fingerprint
    }

    pub fn copy_with_identity(&self, identity: u64) -> Self {
        let mut copy = self.clone();
        copy.identity = identity;
        copy
    }

    pub fn split(&mut self, requested: i32, identity: u64) -> Self {
        if self.is_empty() || requested <= 0 {
            return Self::empty();
        }
        let removed = requested.min(self.count);
        self.count -= removed;
        let mut split = self.copy_with_identity(identity);
        split.count = removed;
        if self.count <= 0 {
            *self = Self::empty();
        }
        split
    }

    pub fn grow(&mut self, amount: i32) {
        self.count = self.count.saturating_add(amount);
        if self.count <= 0 {
            *self = Self::empty();
        }
    }

    pub fn shrink(&mut self, amount: i32) {
        self.grow(amount.saturating_neg());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseRemainder {
    pub stack: ItemStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseCooldown {
    pub ticks: u32,
    pub group_fingerprint: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterUseStep {
    Remainder,
    Cooldown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AfterUseOutcome {
    pub hand: ItemStack,
    pub extra_remainder: Option<ItemStack>,
    pub cooldown: Option<UseCooldown>,
    pub installed_returned_object: bool,
    pub order: Vec<AfterUseStep>,
}

pub fn apply_after_use(
    before: &ItemStack,
    returned: ItemStack,
    remainder: Option<&UseRemainder>,
    cooldown: Option<UseCooldown>,
) -> AfterUseOutcome {
    let installed_returned_object = returned.identity != before.identity;
    let count_fell = returned.count < before.count;
    let mut hand = returned;
    let mut extra_remainder = None;
    let mut order = Vec::with_capacity(2);

    if count_fell && let Some(remainder) = remainder {
        order.push(AfterUseStep::Remainder);
        if hand.is_empty() {
            hand = remainder.stack.clone();
        } else {
            extra_remainder = Some(remainder.stack.clone());
        }
    }
    if cooldown.is_some() {
        order.push(AfterUseStep::Cooldown);
    }

    AfterUseOutcome {
        hand,
        extra_remainder,
        cooldown,
        installed_returned_object,
        order,
    }
}
