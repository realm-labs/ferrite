use std::collections::{BTreeMap, BTreeSet};

use crate::java_26_2::play::clientbound::merchant::packet::{ItemCost, MerchantOffer};
use crate::java_26_2::play::item::{
    DataComponentPatch, EncodedComponentValue, ItemStack, StackContents,
};
use crate::java_26_2::play::serverbound::merchant::packet::SelectTrade;
use crate::java_26_2::value::identifier::Identifier;

const PLAYER_INVENTORY_SLOT_COUNT: usize = 36;
const DEFAULT_MAXIMUM_STACK: i32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerchantSelectionStep {
    StoredHint(i32),
    RecomputedResult,
    ReturnedPayment { slot: usize, moved: i32 },
    FilledPayment { slot: usize, moved: i32 },
    SentRequest(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerchantSelectionTrace {
    pub steps: Vec<MerchantSelectionStep>,
    pub direct_response: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantMenuTransaction {
    pub still_valid: bool,
    pub offers: Vec<MerchantOffer>,
    pub selection_hint: i32,
    pub payment_a: ItemStack,
    pub payment_b: ItemStack,
    pub result: ItemStack,
    pub future_experience: i32,
    pub active_offer: Option<usize>,
    pub player_inventory: Vec<ItemStack>,
    pub merchant_notifications: Vec<ItemStack>,
    maximum_stack_sizes: BTreeMap<Identifier, i32>,
}

impl MerchantMenuTransaction {
    #[must_use]
    pub fn new(still_valid: bool, offers: Vec<MerchantOffer>) -> Self {
        Self {
            still_valid,
            offers,
            selection_hint: 0,
            payment_a: ItemStack::Empty,
            payment_b: ItemStack::Empty,
            result: ItemStack::Empty,
            future_experience: 0,
            active_offer: None,
            player_inventory: vec![ItemStack::Empty; PLAYER_INVENTORY_SLOT_COUNT],
            merchant_notifications: Vec::new(),
            maximum_stack_sizes: BTreeMap::new(),
        }
    }

    pub fn set_maximum_stack_size(&mut self, item: Identifier, maximum: i32) {
        self.maximum_stack_sizes.insert(item, maximum.max(1));
    }

    #[must_use]
    pub fn maximum_stack_size(&self, item: &Identifier) -> i32 {
        self.maximum_stack_sizes
            .get(item)
            .copied()
            .unwrap_or(DEFAULT_MAXIMUM_STACK)
            .max(1)
    }

    pub fn apply_selection(&mut self, selection_hint: i32) -> MerchantSelectionTrace {
        let mut steps = vec![MerchantSelectionStep::StoredHint(selection_hint)];
        self.selection_hint = selection_hint;
        self.recompute_result();
        steps.push(MerchantSelectionStep::RecomputedResult);
        if let Ok(index) = usize::try_from(selection_hint)
            && index < self.offers.len()
        {
            self.try_move_items(index, &mut steps);
        }
        MerchantSelectionTrace {
            steps,
            direct_response: false,
        }
    }

    pub fn recompute_result(&mut self) {
        let (first, second) = if self.payment_a.is_empty() {
            (self.payment_b.clone(), ItemStack::Empty)
        } else {
            (self.payment_a.clone(), self.payment_b.clone())
        };
        if first.is_empty() {
            self.active_offer = None;
            self.result = ItemStack::Empty;
            self.future_experience = 0;
            return;
        }
        if self.offers.is_empty() {
            self.active_offer = None;
            self.merchant_notifications.push(self.result.clone());
            return;
        }

        let matched = self
            .find_offer(&first, &second)
            .filter(|index| !self.offers[*index].is_out_of_stock())
            .or_else(|| {
                self.find_offer(&second, &first)
                    .filter(|index| !self.offers[*index].is_out_of_stock())
            });
        if let Some(index) = matched {
            let offer = &self.offers[index];
            self.active_offer = Some(index);
            self.result = offer.assemble();
            self.future_experience = offer.experience;
        } else {
            self.active_offer = None;
            self.result = ItemStack::Empty;
            self.future_experience = 0;
        }
        self.merchant_notifications.push(self.result.clone());
    }

    fn find_offer(&self, payment_a: &ItemStack, payment_b: &ItemStack) -> Option<usize> {
        let forced = self.selection_hint > 0
            && usize::try_from(self.selection_hint)
                .ok()
                .is_some_and(|hint| hint < self.offers.len());
        if forced {
            let index = usize::try_from(self.selection_hint).ok()?;
            return self
                .offer_satisfied(index, payment_a, payment_b)
                .then_some(index);
        }
        self.offers.iter().enumerate().find_map(|(index, _)| {
            self.offer_satisfied(index, payment_a, payment_b)
                .then_some(index)
        })
    }

    fn offer_satisfied(&self, index: usize, payment_a: &ItemStack, payment_b: &ItemStack) -> bool {
        let offer = &self.offers[index];
        offer.satisfied_by(
            payment_a,
            payment_b,
            self.maximum_stack_size(&offer.cost_a.item),
        )
    }

    fn try_move_items(&mut self, index: usize, steps: &mut Vec<MerchantSelectionStep>) {
        if !self.return_payment(0, steps) {
            return;
        }
        if !self.return_payment(1, steps) {
            return;
        }
        if !self.payment_a.is_empty() || !self.payment_b.is_empty() {
            return;
        }
        let offer = self.offers[index].clone();
        self.fill_payment(0, &offer.cost_a, steps);
        if let Some(cost_b) = &offer.cost_b {
            self.fill_payment(1, cost_b, steps);
        }
    }

    fn return_payment(&mut self, slot: usize, steps: &mut Vec<MerchantSelectionStep>) -> bool {
        let mut payment = if slot == 0 {
            std::mem::take(&mut self.payment_a)
        } else {
            std::mem::take(&mut self.payment_b)
        };
        if payment.is_empty() {
            self.install_payment(slot, payment);
            return true;
        }
        let before = payment.count();
        move_to_inventory_reverse(
            &mut payment,
            &mut self.player_inventory,
            &self.maximum_stack_sizes,
        );
        let moved = before.saturating_sub(payment.count());
        self.install_payment(slot, payment);
        if moved > 0 {
            self.recompute_result();
            steps.push(MerchantSelectionStep::RecomputedResult);
        }
        steps.push(MerchantSelectionStep::ReturnedPayment { slot, moved });
        moved > 0
    }

    fn fill_payment(
        &mut self,
        slot: usize,
        cost: &ItemCost,
        steps: &mut Vec<MerchantSelectionStep>,
    ) {
        let mut target = if slot == 0 {
            std::mem::take(&mut self.payment_a)
        } else {
            std::mem::take(&mut self.payment_b)
        };
        let maximum = self.maximum_stack_size(&cost.item);
        let mut moved_total: i32 = 0;
        for index in 0..self.player_inventory.len() {
            let moved = {
                let source = &mut self.player_inventory[index];
                if !cost.matches(source)
                    || (!target.is_empty() && !same_item_and_components(&target, source))
                {
                    continue;
                }
                let capacity = maximum.saturating_sub(target.count()).max(0);
                let moved = capacity.min(source.count()).max(0);
                if moved == 0 {
                    continue;
                }
                if target.is_empty() {
                    target = copy_with_count(source, moved);
                } else if let ItemStack::Present(contents) = &mut target {
                    contents.count = contents.count.saturating_add(moved);
                }
                shrink(source, moved);
                moved
            };
            moved_total = moved_total.saturating_add(moved);
            self.install_payment(slot, target.clone());
            self.recompute_result();
            steps.push(MerchantSelectionStep::FilledPayment { slot, moved });
            steps.push(MerchantSelectionStep::RecomputedResult);
            if target.count() >= maximum {
                break;
            }
        }
        self.install_payment(slot, target);
        if moved_total == 0 {
            steps.push(MerchantSelectionStep::FilledPayment { slot, moved: 0 });
        }
    }

    fn install_payment(&mut self, slot: usize, stack: ItemStack) {
        if slot == 0 {
            self.payment_a = stack;
        } else {
            self.payment_b = stack;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerchantSelectionOutcome {
    IgnoredWrongMenu,
    IgnoredInvalidMenu,
    Applied,
}

pub fn handle_select_trade(
    current_menu: Option<&mut MerchantMenuTransaction>,
    packet: SelectTrade,
) -> (MerchantSelectionOutcome, Option<MerchantSelectionTrace>) {
    let Some(menu) = current_menu else {
        return (MerchantSelectionOutcome::IgnoredWrongMenu, None);
    };
    if !menu.still_valid {
        return (MerchantSelectionOutcome::IgnoredInvalidMenu, None);
    }
    (
        MerchantSelectionOutcome::Applied,
        Some(menu.apply_selection(packet.selection_hint)),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerchantClientSelection {
    pub packet: SelectTrade,
    pub trace: MerchantSelectionTrace,
}

pub fn predict_trade_selection(
    menu: &mut MerchantMenuTransaction,
    visible_index: i32,
    scroll_offset: i32,
) -> MerchantClientSelection {
    let selection_hint = visible_index.wrapping_add(scroll_offset);
    let mut trace = menu.apply_selection(selection_hint);
    trace
        .steps
        .push(MerchantSelectionStep::SentRequest(selection_hint));
    MerchantClientSelection {
        packet: SelectTrade { selection_hint },
        trace,
    }
}

fn move_to_inventory_reverse(
    source: &mut ItemStack,
    inventory: &mut [ItemStack],
    maximum_stack_sizes: &BTreeMap<Identifier, i32>,
) {
    let maximum = source
        .contents()
        .and_then(|contents| maximum_stack_sizes.get(&contents.item))
        .copied()
        .unwrap_or(DEFAULT_MAXIMUM_STACK)
        .max(1);
    for target in inventory.iter_mut().rev() {
        if source.is_empty() {
            return;
        }
        if target.is_empty() || !same_item_and_components(source, target) {
            continue;
        }
        let moved = maximum
            .saturating_sub(target.count())
            .max(0)
            .min(source.count());
        grow(target, moved);
        shrink(source, moved);
    }
    for target in inventory.iter_mut().rev() {
        if source.is_empty() {
            return;
        }
        if !target.is_empty() {
            continue;
        }
        let moved = maximum.min(source.count()).max(0);
        *target = copy_with_count(source, moved);
        shrink(source, moved);
    }
}

fn same_item_and_components(left: &ItemStack, right: &ItemStack) -> bool {
    match (left.contents(), right.contents()) {
        (Some(left), Some(right)) => {
            left.item == right.item
                && normalized_patch(&left.components) == normalized_patch(&right.components)
        }
        (None, None) => true,
        _ => false,
    }
}

fn normalized_patch(
    patch: &DataComponentPatch,
) -> (BTreeMap<Identifier, Vec<u8>>, BTreeSet<Identifier>) {
    let added = patch
        .added
        .iter()
        .map(
            |EncodedComponentValue {
                 component,
                 encoded_value,
             }| (component.clone(), encoded_value.clone()),
        )
        .collect();
    let removed = patch.removed.iter().cloned().collect();
    (added, removed)
}

fn copy_with_count(source: &ItemStack, count: i32) -> ItemStack {
    let Some(contents) = source.contents() else {
        return ItemStack::Empty;
    };
    ItemStack::Present(StackContents {
        item: contents.item.clone(),
        count,
        components: contents.components.clone(),
    })
}

fn grow(stack: &mut ItemStack, amount: i32) {
    if amount <= 0 {
        return;
    }
    if let ItemStack::Present(contents) = stack {
        contents.count = contents.count.saturating_add(amount);
    }
}

fn shrink(stack: &mut ItemStack, amount: i32) {
    if amount <= 0 {
        return;
    }
    let ItemStack::Present(contents) = stack else {
        return;
    };
    contents.count = contents.count.saturating_sub(amount);
    if contents.count <= 0 {
        *stack = ItemStack::Empty;
    }
}
