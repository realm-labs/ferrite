use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::container::packet::{
    ContainerSetContent, ContainerSetData, ContainerSetSlot, SetCursorItem,
};
use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::item::ItemStack;
use crate::java_26_2::play::serverbound::container::hash::ComponentHashCache;
use crate::java_26_2::play::serverbound::container::packet::{
    ContainerClick, ContainerInput, HashedStack,
};

const MAX_STATE_ID: i32 = 32_767;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerActor {
    pub spectator: bool,
    pub dead_or_dying: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerAuthoritativeState {
    pub slots: Vec<ItemStack>,
    pub carried: ItemStack,
    pub data: Vec<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteStack {
    exact: Option<ItemStack>,
    predicted: Option<HashedStack>,
}

impl RemoteStack {
    fn exact(stack: ItemStack) -> Self {
        Self {
            exact: Some(stack),
            predicted: None,
        }
    }

    fn install_prediction(&mut self, predicted: HashedStack) {
        self.exact = None;
        self.predicted = Some(predicted);
    }

    fn agrees(&mut self, actual: &ItemStack, hashes: &mut ComponentHashCache) -> bool {
        let agrees = self
            .exact
            .as_ref()
            .is_some_and(|expected| stacks_equal(expected, actual))
            || self
                .predicted
                .as_ref()
                .is_some_and(|expected| hashes.matches(expected, actual));
        if agrees {
            self.install_exact(actual.clone());
        }
        agrees
    }

    fn install_exact(&mut self, stack: ItemStack) {
        self.exact = Some(stack);
        self.predicted = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerMenuTransaction {
    pub container_id: i32,
    pub state_id: i32,
    pub still_valid: bool,
    pub authoritative: ContainerAuthoritativeState,
    remote_slots: Vec<RemoteStack>,
    remote_carried: RemoteStack,
    remote_data: Vec<i16>,
    hashes: ComponentHashCache,
    pub idle_resets: u64,
    pub suppression_windows: u64,
}

impl ContainerMenuTransaction {
    pub fn new(
        container_id: i32,
        state_id: i32,
        authoritative: ContainerAuthoritativeState,
    ) -> Result<Self, ContainerTransactionError> {
        validate_shape(&authoritative)?;
        let remote_slots = authoritative
            .slots
            .iter()
            .cloned()
            .map(RemoteStack::exact)
            .collect();
        Ok(Self {
            container_id,
            state_id,
            still_valid: true,
            remote_slots,
            remote_carried: RemoteStack::exact(authoritative.carried.clone()),
            remote_data: authoritative.data.clone(),
            authoritative,
            hashes: ComponentHashCache::default(),
            idle_resets: 0,
            suppression_windows: 0,
        })
    }

    pub fn handle_click(
        &mut self,
        packet: ContainerClick,
        actor: ContainerActor,
        execute: impl FnOnce(
            &mut [ItemStack],
            &mut ItemStack,
            &mut [i16],
            ContainerClickCommand,
        ) -> Result<(), ContainerExecutionError>,
    ) -> Result<ContainerClickOutcome, ContainerTransactionError> {
        self.idle_resets = self.idle_resets.wrapping_add(1);
        if packet.container_id != self.container_id {
            return Ok(ContainerClickOutcome::Ignored(
                ContainerClickIgnore::WrongContainer,
            ));
        }
        if actor.spectator || actor.dead_or_dying {
            return Ok(ContainerClickOutcome::Converged {
                click_executed: false,
                stale_state: false,
                ignored_changed_slots: 0,
                packets: self.broadcast_full(),
            });
        }
        if !self.still_valid {
            return Ok(ContainerClickOutcome::Ignored(
                ContainerClickIgnore::InvalidMenu,
            ));
        }
        let slot = i32::from(packet.slot);
        if !valid_outer_slot(slot, self.authoritative.slots.len()) {
            return Ok(ContainerClickOutcome::Ignored(
                ContainerClickIgnore::RejectedSlot,
            ));
        }

        let stale_state = packet.state_id != self.state_id;
        self.suppression_windows = self.suppression_windows.wrapping_add(1);
        execute(
            &mut self.authoritative.slots,
            &mut self.authoritative.carried,
            &mut self.authoritative.data,
            ContainerClickCommand {
                slot,
                button: i32::from(packet.button),
                input: packet.input,
            },
        )?;

        let mut ignored_changed_slots = 0;
        for (slot, hash) in packet.changed_slots {
            let Ok(index) = usize::try_from(slot) else {
                ignored_changed_slots += 1;
                continue;
            };
            let Some(remote) = self.remote_slots.get_mut(index) else {
                ignored_changed_slots += 1;
                continue;
            };
            remote.install_prediction(hash);
        }
        self.remote_carried.install_prediction(packet.carried);

        let packets = if stale_state {
            self.broadcast_full()
        } else {
            self.broadcast_changes()
        };
        Ok(ContainerClickOutcome::Converged {
            click_executed: true,
            stale_state,
            ignored_changed_slots,
            packets,
        })
    }

    pub fn broadcast_changes(&mut self) -> Vec<PlayClientboundPacket> {
        let mut packets = Vec::new();
        for index in 0..self.authoritative.slots.len() {
            let item = &self.authoritative.slots[index];
            if self.remote_slots[index].agrees(item, &mut self.hashes) {
                continue;
            }
            self.state_id = increment_state(self.state_id);
            packets.push(PlayClientboundPacket::ContainerSetSlot(ContainerSetSlot {
                container_id: self.container_id,
                state_id: self.state_id,
                slot: i16::try_from(index)
                    .expect("validated transaction slot index fits signed short"),
                item: item.clone(),
            }));
            self.remote_slots[index].install_exact(item.clone());
        }
        if !self
            .remote_carried
            .agrees(&self.authoritative.carried, &mut self.hashes)
        {
            packets.push(PlayClientboundPacket::SetCursorItem(SetCursorItem {
                item: self.authoritative.carried.clone(),
            }));
            self.remote_carried
                .install_exact(self.authoritative.carried.clone());
        }
        for (index, value) in self.authoritative.data.iter().copied().enumerate() {
            if self.remote_data[index] == value {
                continue;
            }
            packets.push(PlayClientboundPacket::ContainerSetData(ContainerSetData {
                container_id: self.container_id,
                property_id: i16::try_from(index)
                    .expect("validated transaction data index fits signed short"),
                value,
            }));
            self.remote_data[index] = value;
        }
        packets
    }

    fn broadcast_full(&mut self) -> Vec<PlayClientboundPacket> {
        self.state_id = increment_state(self.state_id);
        self.remote_slots = self
            .authoritative
            .slots
            .iter()
            .cloned()
            .map(RemoteStack::exact)
            .collect();
        self.remote_carried = RemoteStack::exact(self.authoritative.carried.clone());
        self.remote_data.clone_from(&self.authoritative.data);
        let mut packets = vec![PlayClientboundPacket::ContainerSetContent(
            ContainerSetContent {
                container_id: self.container_id,
                state_id: self.state_id,
                slots: self.authoritative.slots.clone(),
                carried: self.authoritative.carried.clone(),
            },
        )];
        packets.extend(self.authoritative.data.iter().copied().enumerate().map(
            |(index, value)| {
                PlayClientboundPacket::ContainerSetData(ContainerSetData {
                    container_id: self.container_id,
                    property_id: i16::try_from(index)
                        .expect("validated transaction data index fits signed short"),
                    value,
                })
            },
        ));
        packets
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerClickCommand {
    pub slot: i32,
    pub button: i32,
    pub input: ContainerInput,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerClickOutcome {
    Ignored(ContainerClickIgnore),
    Converged {
        click_executed: bool,
        stale_state: bool,
        ignored_changed_slots: usize,
        packets: Vec<PlayClientboundPacket>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerClickIgnore {
    WrongContainer,
    InvalidMenu,
    RejectedSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerClientMenu {
    pub container_id: i32,
    pub state_id: i32,
    pub slots: Vec<ItemStack>,
    pub carried: ItemStack,
    hashes: ComponentHashCache,
    pub predictions: u64,
}

impl ContainerClientMenu {
    #[must_use]
    pub fn new(
        container_id: i32,
        state_id: i32,
        slots: Vec<ItemStack>,
        carried: ItemStack,
    ) -> Self {
        Self {
            container_id,
            state_id,
            slots,
            carried,
            hashes: ComponentHashCache::default(),
            predictions: 0,
        }
    }

    pub fn predict_click(
        &mut self,
        supplied_container_id: i32,
        slot: i32,
        button: i32,
        input: ContainerInput,
        execute: impl FnOnce(&mut [ItemStack], &mut ItemStack),
    ) -> Result<ContainerClientClick, ContainerClientClickError> {
        if supplied_container_id != self.container_id {
            return Ok(ContainerClientClick::IgnoredWrongContainer);
        }
        let slot = i16::try_from(slot)
            .map_err(|_| ContainerClientClickError::SlotWidth { value: slot })?;
        let button = i8::try_from(button)
            .map_err(|_| ContainerClientClickError::ButtonWidth { value: button })?;
        let before = self.slots.clone();
        execute(&mut self.slots, &mut self.carried);
        self.predictions = self.predictions.wrapping_add(1);
        let changed_slots = before
            .iter()
            .zip(&self.slots)
            .enumerate()
            .filter(|(_, (before, after))| !stacks_equal(before, after))
            .map(|(index, (_, after))| {
                (
                    i16::try_from(index)
                        .expect("client menu emitted a slot index outside signed-short width"),
                    self.hashes.hash_stack(after),
                )
            })
            .collect();
        Ok(ContainerClientClick::PredictedAndSend(ContainerClick {
            container_id: self.container_id,
            state_id: self.state_id,
            slot,
            button,
            input,
            changed_slots,
            carried: self.hashes.hash_stack(&self.carried),
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerClientClick {
    IgnoredWrongContainer,
    PredictedAndSend(ContainerClick),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContainerClientClickError {
    #[error("client click slot {value} does not fit a signed short")]
    SlotWidth { value: i32 },
    #[error("client click button {value} does not fit a signed byte")]
    ButtonWidth { value: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("authoritative menu click failed: {reason}")]
pub struct ContainerExecutionError {
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContainerTransactionError {
    #[error("container transaction has {slots} slots, beyond signed-short indexing")]
    TooManySlots { slots: usize },
    #[error("container transaction has {data} data values, beyond signed-short indexing")]
    TooManyDataValues { data: usize },
    #[error(transparent)]
    Execution(#[from] ContainerExecutionError),
}

fn validate_shape(state: &ContainerAuthoritativeState) -> Result<(), ContainerTransactionError> {
    if state.slots.len() > i16::MAX as usize + 1 {
        return Err(ContainerTransactionError::TooManySlots {
            slots: state.slots.len(),
        });
    }
    if state.data.len() > i16::MAX as usize + 1 {
        return Err(ContainerTransactionError::TooManyDataValues {
            data: state.data.len(),
        });
    }
    Ok(())
}

fn valid_outer_slot(slot: i32, slots: usize) -> bool {
    slot == -1 || slot == -999 || slot < slots as i32
}

fn stacks_equal(left: &ItemStack, right: &ItemStack) -> bool {
    let (Some(left), Some(right)) = (left.contents(), right.contents()) else {
        return left.is_empty() && right.is_empty();
    };
    if left.item != right.item || left.count != right.count {
        return false;
    }
    let left_added = left
        .components
        .added
        .iter()
        .map(|value| (&value.component, &value.encoded_value))
        .collect::<BTreeMap<_, _>>();
    let right_added = right
        .components
        .added
        .iter()
        .map(|value| (&value.component, &value.encoded_value))
        .collect::<BTreeMap<_, _>>();
    let left_removed = left.components.removed.iter().collect::<BTreeSet<_>>();
    let right_removed = right.components.removed.iter().collect::<BTreeSet<_>>();
    left_added == right_added && left_removed == right_removed
}

const fn increment_state(state_id: i32) -> i32 {
    state_id.wrapping_add(1) & MAX_STATE_ID
}
