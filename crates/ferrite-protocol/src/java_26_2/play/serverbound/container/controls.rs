use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::item::ItemStack;
use crate::java_26_2::play::serverbound::container::packet::{
    ContainerButtonClick, ContainerClose, ContainerSlotStateChanged, HashedStack, SetCarriedItem,
};
use crate::java_26_2::play::serverbound::container::transaction::{
    ContainerExecutionError, ContainerMenuTransaction, ContainerTransactionError,
};

pub fn handle_button(
    current: Option<&mut ContainerMenuTransaction>,
    packet: ContainerButtonClick,
    spectator: bool,
    execute: impl FnOnce(
        i32,
        &mut [ItemStack],
        &mut ItemStack,
        &mut [i16],
    ) -> Result<bool, ContainerExecutionError>,
) -> Result<ContainerButtonOutcome, ContainerTransactionError> {
    let Some(menu) = current else {
        return Ok(ContainerButtonOutcome::IgnoredWrongContainer);
    };
    menu.idle_resets = menu.idle_resets.wrapping_add(1);
    if packet.container_id != menu.container_id {
        return Ok(ContainerButtonOutcome::IgnoredWrongContainer);
    }
    if spectator {
        return Ok(ContainerButtonOutcome::IgnoredSpectator);
    }
    if !menu.still_valid {
        return Ok(ContainerButtonOutcome::IgnoredInvalidMenu);
    }
    if !execute(
        packet.button_id,
        &mut menu.authoritative.slots,
        &mut menu.authoritative.carried,
        &mut menu.authoritative.data,
    )? {
        return Ok(ContainerButtonOutcome::RejectedByMenu);
    }
    Ok(ContainerButtonOutcome::Applied {
        packets: menu.broadcast_changes(),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerButtonOutcome {
    IgnoredWrongContainer,
    IgnoredSpectator,
    IgnoredInvalidMenu,
    RejectedByMenu,
    Applied { packets: Vec<PlayClientboundPacket> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrafterMenuState {
    pub container_id: i32,
    pub real_block_entity: bool,
    pub slots: [ItemStack; 9],
    pub disabled: [bool; 9],
    pub dirty_writes: u64,
}

pub fn handle_crafter_slot_state(
    current: Option<&mut CrafterMenuState>,
    spectator: bool,
    packet: ContainerSlotStateChanged,
) -> CrafterSlotStateOutcome {
    if spectator {
        return CrafterSlotStateOutcome::IgnoredSpectator;
    }
    let Some(crafter) = current else {
        return CrafterSlotStateOutcome::IgnoredWrongMenu;
    };
    if packet.container_id != crafter.container_id {
        return CrafterSlotStateOutcome::IgnoredWrongContainer;
    }
    if !crafter.real_block_entity {
        return CrafterSlotStateOutcome::IgnoredWrongBacking;
    }
    let Ok(index) = usize::try_from(packet.slot_id) else {
        return CrafterSlotStateOutcome::IgnoredSlot;
    };
    let Some(stack) = crafter.slots.get(index) else {
        return CrafterSlotStateOutcome::IgnoredSlot;
    };
    if !stack.is_empty() {
        return CrafterSlotStateOutcome::IgnoredNonempty;
    }
    let disabled = !packet.new_state;
    if crafter.disabled[index] == disabled {
        return CrafterSlotStateOutcome::Unchanged;
    }
    crafter.disabled[index] = disabled;
    crafter.dirty_writes = crafter.dirty_writes.wrapping_add(1);
    CrafterSlotStateOutcome::Applied {
        slot: index,
        stored_value: i32::from(disabled),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrafterSlotStateOutcome {
    IgnoredSpectator,
    IgnoredWrongMenu,
    IgnoredWrongContainer,
    IgnoredWrongBacking,
    IgnoredSlot,
    IgnoredNonempty,
    Unchanged,
    Applied { slot: usize, stored_value: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferableRemoteSlot {
    pub backing_container: u64,
    pub backing_slot: usize,
    pub exact: Option<ItemStack>,
    pub predicted: Option<HashedStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseableMenu {
    pub container_id: i32,
    pub remote_slots: Vec<TransferableRemoteSlot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerCloseSession {
    pub current: Option<CloseableMenu>,
    pub inventory_remote_slots: Vec<TransferableRemoteSlot>,
    pub removals: u64,
}

impl ContainerCloseSession {
    #[must_use]
    pub fn handle_close(&mut self, _packet: ContainerClose) -> ContainerCloseOutcome {
        let Some(closing) = self.current.take() else {
            return ContainerCloseOutcome::InventoryMenuSelected {
                closed_container_id: None,
                transferred_slots: 0,
                response_packets: 0,
            };
        };
        let mut transferred_slots = 0;
        for target in &mut self.inventory_remote_slots {
            let Some(source) = closing.remote_slots.iter().find(|source| {
                source.backing_container == target.backing_container
                    && source.backing_slot == target.backing_slot
            }) else {
                continue;
            };
            target.exact.clone_from(&source.exact);
            target.predicted.clone_from(&source.predicted);
            transferred_slots += 1;
        }
        self.removals = self.removals.wrapping_add(1);
        ContainerCloseOutcome::InventoryMenuSelected {
            closed_container_id: Some(closing.container_id),
            transferred_slots,
            response_packets: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerCloseOutcome {
    InventoryMenuSelected {
        closed_container_id: Option<i32>,
        transferred_slots: usize,
        response_packets: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarriedSelectionState {
    pub selected: usize,
    pub active_main_hand_use: bool,
    pub idle_resets: u64,
    pub stopped_main_hand_use: u64,
    pub equipment_dirty: bool,
}

impl CarriedSelectionState {
    #[must_use]
    pub const fn new(selected: usize) -> Self {
        Self {
            selected,
            active_main_hand_use: false,
            idle_resets: 0,
            stopped_main_hand_use: 0,
            equipment_dirty: false,
        }
    }

    pub fn handle_set_carried(&mut self, packet: SetCarriedItem) -> CarriedSelectionOutcome {
        let Ok(selected) = usize::try_from(packet.slot) else {
            return CarriedSelectionOutcome::IgnoredInvalidSlot;
        };
        if selected > 8 {
            return CarriedSelectionOutcome::IgnoredInvalidSlot;
        }
        self.idle_resets = self.idle_resets.wrapping_add(1);
        if selected == self.selected {
            return CarriedSelectionOutcome::AcceptedUnchanged;
        }
        if self.active_main_hand_use {
            self.active_main_hand_use = false;
            self.stopped_main_hand_use = self.stopped_main_hand_use.wrapping_add(1);
        }
        let previous = self.selected;
        self.selected = selected;
        self.equipment_dirty = true;
        CarriedSelectionOutcome::Changed { previous, selected }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarriedSelectionOutcome {
    IgnoredInvalidSlot,
    AcceptedUnchanged,
    Changed { previous: usize, selected: usize },
}
