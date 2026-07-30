//! Server-side predicted-click replay and delta/full synchronization.

use crate::item::runtime::menu_click::{ClickError, ClickMenu, ClickPlayer, ContainerInput};
use crate::item::runtime::menu_layout::MenuLayout;
use crate::item::runtime::stack::ItemStack;
use std::collections::BTreeMap;

pub const MAX_CHANGED_SLOT_HASHES: usize = 128;
pub const MAX_STATE_ID: u16 = 32_767;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickPacket {
    pub container_id: u8,
    pub state_id: u16,
    pub slot: i32,
    pub button: i32,
    pub input: ContainerInput,
    pub changed_slot_hashes: Vec<(i32, u64)>,
    pub carried_hash: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuActor {
    pub spectator: bool,
    pub dead_or_dying: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSession {
    pub container_id: u8,
    pub state_id: u16,
    pub menu: ClickMenu,
    pub layout: MenuLayout,
    pub still_valid: bool,
    pub remote_slot_hashes: BTreeMap<i32, u64>,
    pub remote_carried_hash: u64,
    local_slots: Vec<ItemStack>,
    local_carried: ItemStack,
}

impl MenuSession {
    pub fn new(container_id: u8, menu: ClickMenu, layout: MenuLayout) -> Self {
        let local_slots = menu.slots.iter().map(|slot| slot.stack.clone()).collect();
        let local_carried = menu.carried.clone();
        Self {
            container_id,
            state_id: 0,
            menu,
            layout,
            still_valid: true,
            remote_slot_hashes: BTreeMap::new(),
            remote_carried_hash: 0,
            local_slots,
            local_carried,
        }
    }

    pub fn handle_click(
        &mut self,
        packet: ClickPacket,
        actor: MenuActor,
        player: &mut ClickPlayer,
    ) -> Result<ClickSync, MenuPacketError> {
        if packet.changed_slot_hashes.len() > MAX_CHANGED_SLOT_HASHES {
            return Err(MenuPacketError::TooManyChangedSlots {
                actual: packet.changed_slot_hashes.len(),
            });
        }
        if packet.container_id != self.container_id {
            return Ok(ClickSync::Ignored(IgnoreReason::WrongContainer));
        }
        if actor.spectator || actor.dead_or_dying {
            return Ok(self.full_snapshot(false));
        }
        if !self.still_valid {
            return Ok(ClickSync::Ignored(IgnoreReason::InvalidMenu));
        }
        if !self.menu.is_valid_slot_index(packet.slot) {
            return Ok(ClickSync::Ignored(IgnoreReason::RejectedSlot));
        }

        let stale = packet.state_id != self.state_id;
        self.menu
            .clicked(
                packet.slot,
                packet.button,
                packet.input,
                player,
                &self.layout,
            )
            .map_err(MenuPacketError::Click)?;
        for (index, hash) in packet.changed_slot_hashes {
            self.remote_slot_hashes.insert(index, hash);
        }
        self.remote_carried_hash = packet.carried_hash;

        if stale {
            Ok(self.full_snapshot(true))
        } else {
            Ok(self.broadcast_changes())
        }
    }

    fn full_snapshot(&mut self, click_executed: bool) -> ClickSync {
        self.increment_state_id();
        self.refresh_local();
        ClickSync::Full {
            state_id: self.state_id,
            click_executed,
        }
    }

    fn broadcast_changes(&mut self) -> ClickSync {
        let mut slot_deltas = Vec::new();
        for index in 0..self.menu.slots.len() {
            let current = self.menu.slots[index].stack.clone();
            if self.local_slots.get(index) == Some(&current) {
                continue;
            }
            self.increment_state_id();
            slot_deltas.push(SlotDelta {
                index,
                state_id: self.state_id,
            });
            self.local_slots[index] = current;
        }
        let carried_changed = self.local_carried != self.menu.carried;
        if carried_changed {
            self.local_carried = self.menu.carried.clone();
        }
        ClickSync::Deltas {
            slot_deltas,
            carried_changed,
        }
    }

    fn increment_state_id(&mut self) {
        self.state_id = self.state_id.wrapping_add(1) & MAX_STATE_ID;
    }

    fn refresh_local(&mut self) {
        self.local_slots = self
            .menu
            .slots
            .iter()
            .map(|slot| slot.stack.clone())
            .collect();
        self.local_carried = self.menu.carried.clone();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickSync {
    Ignored(IgnoreReason),
    Full {
        state_id: u16,
        click_executed: bool,
    },
    Deltas {
        slot_deltas: Vec<SlotDelta>,
        carried_changed: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreReason {
    WrongContainer,
    InvalidMenu,
    RejectedSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotDelta {
    pub index: usize,
    pub state_id: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuPacketError {
    TooManyChangedSlots { actual: usize },
    Click(ClickError),
}
