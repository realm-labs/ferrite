use crate::java_26_2::play::item::{EncodedComponentValue, ItemStack};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemCost {
    pub item: Identifier,
    pub count: i32,
    /// Exact component expectations retain wire order and duplicates.
    pub components: Vec<EncodedComponentValue>,
}

impl ItemCost {
    #[must_use]
    pub fn matches(&self, candidate: &ItemStack) -> bool {
        let Some(contents) = candidate.contents() else {
            return false;
        };
        contents.item == self.item
            && self.components.iter().all(|expected| {
                contents.components.added.iter().any(|actual| {
                    actual.component == expected.component
                        && actual.encoded_value == expected.encoded_value
                })
            })
    }

    #[must_use]
    pub fn accepts_count(&self, candidate: &ItemStack, required: i32) -> bool {
        self.matches(candidate) && candidate.count() >= required
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantOffer {
    pub cost_a: ItemCost,
    pub result: ItemStack,
    pub cost_b: Option<ItemCost>,
    pub uses: i32,
    pub max_uses: i32,
    pub experience: i32,
    pub special_price_difference: i32,
    pub price_multiplier: f32,
    pub demand: i32,
    /// Network-created offers always reward experience; this value is not carried on the wire.
    pub reward_experience: bool,
}

impl MerchantOffer {
    #[must_use]
    pub const fn is_out_of_stock(&self) -> bool {
        self.uses >= self.max_uses
    }

    #[must_use]
    pub fn modified_cost_a_count(&self, maximum_stack_size: i32) -> i32 {
        let product = self.cost_a.count.wrapping_mul(self.demand);
        let scaled = (product as f32) * self.price_multiplier;
        let demand_delta = java_floor(scaled).max(0);
        let modified = self
            .cost_a
            .count
            .wrapping_add(demand_delta)
            .wrapping_add(self.special_price_difference);
        let maximum = maximum_stack_size.max(1);
        modified.clamp(1, maximum)
    }

    #[must_use]
    pub fn satisfied_by(
        &self,
        payment_a: &ItemStack,
        payment_b: &ItemStack,
        maximum_stack_size: i32,
    ) -> bool {
        if !self
            .cost_a
            .accepts_count(payment_a, self.modified_cost_a_count(maximum_stack_size))
        {
            return false;
        }
        self.cost_b.as_ref().map_or_else(
            || payment_b.is_empty(),
            |cost| cost.accepts_count(payment_b, cost.count),
        )
    }

    #[must_use]
    pub fn assemble(&self) -> ItemStack {
        self.result.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantOffers {
    pub container_id: i32,
    pub offers: Vec<MerchantOffer>,
    pub villager_level: i32,
    pub villager_experience: i32,
    pub show_progress: bool,
    pub can_restock: bool,
}

fn java_floor(value: f32) -> i32 {
    let truncated = value as i32;
    if value < truncated as f32 {
        truncated.wrapping_sub(1)
    } else {
        truncated
    }
}
