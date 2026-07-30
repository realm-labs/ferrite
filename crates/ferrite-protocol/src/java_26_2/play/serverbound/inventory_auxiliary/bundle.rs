use crate::java_26_2::play::serverbound::inventory_auxiliary::packet::BundleItemSelected;

#[derive(Debug, Clone)]
pub struct BundleContentsProjection {
    entries: Vec<u64>,
    selected: i32,
}

impl PartialEq for BundleContentsProjection {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl Eq for BundleContentsProjection {}

impl BundleContentsProjection {
    #[must_use]
    pub fn from_component_stream(entries: Vec<u64>) -> Self {
        Self {
            entries,
            selected: -1,
        }
    }

    #[must_use]
    pub fn entries(&self) -> &[u64] {
        &self.entries
    }

    #[must_use]
    pub const fn selected(&self) -> i32 {
        self.selected
    }

    pub fn toggle_selected(&mut self, requested: i32) {
        let selected = usize::try_from(requested)
            .ok()
            .filter(|index| *index < self.entries.len())
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1);
        self.selected = if selected == self.selected {
            -1
        } else {
            selected
        };
    }

    pub fn remove_selected_or_first(&mut self) -> Option<u64> {
        if self.entries.is_empty() {
            self.selected = -1;
            return None;
        }
        let index = usize::try_from(self.selected)
            .ok()
            .filter(|index| *index < self.entries.len())
            .unwrap_or(0);
        self.selected = -1;
        Some(self.entries.remove(index))
    }

    #[must_use]
    pub fn reconstructed(&self) -> Self {
        Self::from_component_stream(self.entries.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BundleMenuStack {
    pub contents: Option<BundleContentsProjection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleSelectionOutcome {
    IgnoredInvalidSlot,
    IgnoredMissingComponent,
    Applied { selected: i32 },
}

pub fn handle_bundle_selection(
    current_menu: &mut [BundleMenuStack],
    packet: BundleItemSelected,
) -> BundleSelectionOutcome {
    let Some(stack) = usize::try_from(packet.slot)
        .ok()
        .and_then(|slot| current_menu.get_mut(slot))
    else {
        return BundleSelectionOutcome::IgnoredInvalidSlot;
    };
    let Some(contents) = stack.contents.as_mut() else {
        return BundleSelectionOutcome::IgnoredMissingComponent;
    };
    contents.toggle_selected(packet.selected);
    BundleSelectionOutcome::Applied {
        selected: contents.selected(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleClientEvent {
    Mutated { slot: i32, selected: i32 },
    Sent(BundleItemSelected),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BundleClientMenu {
    pub slots: Vec<BundleMenuStack>,
    pub events: Vec<BundleClientEvent>,
}

impl BundleClientMenu {
    pub fn toggle(&mut self, slot: i32, selected: i32) -> Option<BundleItemSelected> {
        let stack = usize::try_from(slot)
            .ok()
            .and_then(|slot| self.slots.get_mut(slot))?;
        let contents = stack.contents.as_mut()?;
        if selected < -1 {
            return None;
        }
        contents.toggle_selected(selected);
        let packet = BundleItemSelected {
            slot,
            selected: contents.selected(),
        };
        self.events.push(BundleClientEvent::Mutated {
            slot,
            selected: contents.selected(),
        });
        self.events.push(BundleClientEvent::Sent(packet));
        Some(packet)
    }

    pub fn clear(&mut self, slot: i32) -> Option<BundleItemSelected> {
        let stack = usize::try_from(slot)
            .ok()
            .and_then(|slot| self.slots.get_mut(slot))?;
        let contents = stack.contents.as_mut()?;
        contents.selected = -1;
        let packet = BundleItemSelected { slot, selected: -1 };
        self.events
            .push(BundleClientEvent::Mutated { slot, selected: -1 });
        self.events.push(BundleClientEvent::Sent(packet));
        Some(packet)
    }

    pub fn scroll(&mut self, slot: i32, direction: i32) -> Option<BundleItemSelected> {
        let stack = usize::try_from(slot)
            .ok()
            .and_then(|slot| self.slots.get(slot))?;
        let contents = stack.contents.as_ref()?;
        let visible = displayed_item_count(contents.entries.len());
        if visible == 0 || direction == 0 {
            return None;
        }
        let visible = i32::try_from(visible).ok()?;
        let current = contents.selected;
        let selected = if (0..visible).contains(&current) {
            (current + direction.signum()).rem_euclid(visible)
        } else if direction.is_positive() {
            0
        } else {
            visible - 1
        };
        self.toggle(slot, selected)
    }
}

#[must_use]
pub const fn displayed_item_count(content_size: usize) -> usize {
    if content_size <= 12 {
        content_size
    } else {
        8 + (content_size - 1) % 4
    }
}
