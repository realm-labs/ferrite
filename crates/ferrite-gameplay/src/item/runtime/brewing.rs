//! Brewing fuel, cancellation, ordered mixes, and three-bottle commit.

use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;

pub const BREW_DURATION: u32 = 400;
pub const BREW_FUEL_USES: u8 = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionStack {
    pub stack: ItemStack,
    pub potion_fingerprint: Option<u64>,
}

impl PotionStack {
    pub fn empty() -> Self {
        Self {
            stack: ItemStack::empty(),
            potion_fingerprint: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MixKind {
    Container {
        from_item: ResourceId,
        to_item: ResourceId,
    },
    Potion {
        from_potion: u64,
        to_potion: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixEdge {
    pub ingredient: ResourceId,
    pub kind: MixKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewingStand {
    pub bottles: [PotionStack; 3],
    pub ingredient: ItemStack,
    pub fuel: ItemStack,
    pub brew_time: u32,
    pub fuel_uses: u8,
    pub remembered_ingredient: Option<ResourceId>,
    pub dropped_remainders: Vec<ItemStack>,
}

impl BrewingStand {
    pub fn empty() -> Self {
        Self {
            bottles: std::array::from_fn(|_| PotionStack::empty()),
            ingredient: ItemStack::empty(),
            fuel: ItemStack::empty(),
            brew_time: 0,
            fuel_uses: 0,
            remembered_ingredient: None,
            dropped_remainders: Vec::new(),
        }
    }

    pub fn tick(
        &mut self,
        fuel_item: &ResourceId,
        mixes: &[MixEdge],
        ingredient_remainder: Option<&ItemStack>,
    ) -> BrewTickOutcome {
        let mut fuel_refilled = false;
        if self.fuel_uses == 0 && self.fuel.item.as_ref() == Some(fuel_item) {
            self.fuel.shrink(1);
            self.fuel_uses = BREW_FUEL_USES;
            fuel_refilled = true;
        }
        let brewable = self.is_brewable(mixes);
        let mut started = false;
        let mut completed = false;
        let mut cancelled = false;
        if self.brew_time > 0 {
            self.brew_time -= 1;
            if self.brew_time == 0 {
                if brewable {
                    self.do_brew(mixes, ingredient_remainder);
                    completed = true;
                }
            } else if !brewable
                || self.ingredient.item.as_ref() != self.remembered_ingredient.as_ref()
            {
                self.brew_time = 0;
                cancelled = true;
            }
        } else if brewable && self.fuel_uses > 0 {
            self.fuel_uses -= 1;
            self.brew_time = BREW_DURATION;
            self.remembered_ingredient = self.ingredient.item.clone();
            started = true;
        }
        BrewTickOutcome {
            fuel_refilled,
            started,
            completed,
            cancelled,
        }
    }

    pub fn bottle_presence(&self) -> [bool; 3] {
        std::array::from_fn(|index| !self.bottles[index].stack.is_empty())
    }

    fn is_brewable(&self, mixes: &[MixEdge]) -> bool {
        let Some(ingredient) = self.ingredient.item.as_ref() else {
            return false;
        };
        self.bottles.iter().any(|bottle| {
            !bottle.stack.is_empty()
                && mixes
                    .iter()
                    .any(|edge| &edge.ingredient == ingredient && edge_matches(edge, bottle))
        })
    }

    fn do_brew(&mut self, mixes: &[MixEdge], ingredient_remainder: Option<&ItemStack>) {
        let Some(ingredient) = self.ingredient.item.as_ref() else {
            return;
        };
        for bottle in &mut self.bottles {
            let Some(edge) = mixes
                .iter()
                .find(|edge| &edge.ingredient == ingredient && edge_matches(edge, bottle))
            else {
                continue;
            };
            apply_edge(edge, bottle);
        }
        self.ingredient.shrink(1);
        if let Some(remainder) = ingredient_remainder {
            if self.ingredient.is_empty() {
                self.ingredient = remainder.clone();
            } else {
                self.dropped_remainders.push(remainder.clone());
            }
        }
    }
}

fn edge_matches(edge: &MixEdge, bottle: &PotionStack) -> bool {
    match &edge.kind {
        MixKind::Container { from_item, .. } => bottle.stack.item.as_ref() == Some(from_item),
        MixKind::Potion { from_potion, .. } => bottle.potion_fingerprint == Some(*from_potion),
    }
}

fn apply_edge(edge: &MixEdge, bottle: &mut PotionStack) {
    match &edge.kind {
        MixKind::Container { to_item, .. } => {
            bottle.stack.item = Some(to_item.clone());
        }
        MixKind::Potion { to_potion, .. } => {
            bottle.potion_fingerprint = Some(*to_potion);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrewTickOutcome {
    pub fuel_refilled: bool,
    pub started: bool,
    pub completed: bool,
    pub cancelled: bool,
}
