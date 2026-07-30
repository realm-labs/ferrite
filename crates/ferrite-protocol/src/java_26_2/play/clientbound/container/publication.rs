//! Canonical server publication order and remote-state tracking for ordinary menus.

use thiserror::Error;

use crate::java_26_2::play::clientbound::container::packet::{
    ContainerClose, ContainerSetContent, ContainerSetData, ContainerSetSlot, OpenScreen,
    SetCursorItem,
};
use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::item::ItemStack;
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

const MAX_STATE_ID: i32 = 32_767;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSnapshot {
    pub menu_type: Identifier,
    pub title: TextComponentNbt,
    pub slots: Vec<ItemStack>,
    pub carried: ItemStack,
    pub data: Vec<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublishedMenu {
    container_id: i32,
    state_id: i32,
    remote: MenuSnapshot,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerPublisher {
    counter: i32,
    current: Option<PublishedMenu>,
}

impl ContainerPublisher {
    pub fn open(
        &mut self,
        snapshot: MenuSnapshot,
    ) -> Result<Vec<PlayClientboundPacket>, ContainerPublicationError> {
        validate_snapshot(&snapshot)?;
        let mut packets = self.close();
        self.counter = self.counter % 100 + 1;
        let container_id = self.counter;
        packets.push(PlayClientboundPacket::OpenScreen(OpenScreen {
            container_id,
            menu_type: snapshot.menu_type.clone(),
            title: snapshot.title.clone(),
        }));
        let mut menu = PublishedMenu {
            container_id,
            state_id: 0,
            remote: snapshot,
        };
        append_full_state(&mut packets, &mut menu);
        self.current = Some(menu);
        Ok(packets)
    }

    #[must_use]
    pub fn close(&mut self) -> Vec<PlayClientboundPacket> {
        self.current.take().map_or_else(Vec::new, |menu| {
            vec![PlayClientboundPacket::ContainerClose(ContainerClose {
                container_id: menu.container_id,
            })]
        })
    }

    pub fn broadcast_full(
        &mut self,
        authoritative: MenuSnapshot,
    ) -> Result<Vec<PlayClientboundPacket>, ContainerPublicationError> {
        validate_snapshot(&authoritative)?;
        let menu = self
            .current
            .as_mut()
            .ok_or(ContainerPublicationError::NoCurrentMenu)?;
        require_shape(menu, &authoritative)?;
        menu.remote = authoritative;
        let mut packets = Vec::new();
        append_full_state(&mut packets, menu);
        Ok(packets)
    }

    pub fn broadcast_changes(
        &mut self,
        authoritative: &MenuSnapshot,
    ) -> Result<Vec<PlayClientboundPacket>, ContainerPublicationError> {
        validate_snapshot(authoritative)?;
        let menu = self
            .current
            .as_mut()
            .ok_or(ContainerPublicationError::NoCurrentMenu)?;
        require_shape(menu, authoritative)?;
        let mut packets = Vec::new();
        for (index, item) in authoritative.slots.iter().enumerate() {
            if menu.remote.slots[index] == *item {
                continue;
            }
            menu.state_id = increment_state(menu.state_id);
            packets.push(PlayClientboundPacket::ContainerSetSlot(ContainerSetSlot {
                container_id: menu.container_id,
                state_id: menu.state_id,
                slot: i16::try_from(index)
                    .map_err(|_| ContainerPublicationError::SlotIndex { index })?,
                item: item.clone(),
            }));
            menu.remote.slots[index] = item.clone();
        }
        if menu.remote.carried != authoritative.carried {
            packets.push(PlayClientboundPacket::SetCursorItem(SetCursorItem {
                item: authoritative.carried.clone(),
            }));
            menu.remote.carried = authoritative.carried.clone();
        }
        for (index, value) in authoritative.data.iter().copied().enumerate() {
            if menu.remote.data[index] == value {
                continue;
            }
            packets.push(PlayClientboundPacket::ContainerSetData(ContainerSetData {
                container_id: menu.container_id,
                property_id: i16::try_from(index)
                    .map_err(|_| ContainerPublicationError::DataIndex { index })?,
                value,
            }));
            menu.remote.data[index] = value;
        }
        Ok(packets)
    }

    #[must_use]
    pub fn current_container_id(&self) -> Option<i32> {
        self.current.as_ref().map(|menu| menu.container_id)
    }

    #[must_use]
    pub fn current_state_id(&self) -> Option<i32> {
        self.current.as_ref().map(|menu| menu.state_id)
    }
}

fn append_full_state(packets: &mut Vec<PlayClientboundPacket>, menu: &mut PublishedMenu) {
    menu.state_id = increment_state(menu.state_id);
    packets.push(PlayClientboundPacket::ContainerSetContent(
        ContainerSetContent {
            container_id: menu.container_id,
            state_id: menu.state_id,
            slots: menu.remote.slots.clone(),
            carried: menu.remote.carried.clone(),
        },
    ));
    packets.extend(
        menu.remote
            .data
            .iter()
            .copied()
            .enumerate()
            .map(|(index, value)| {
                PlayClientboundPacket::ContainerSetData(ContainerSetData {
                    container_id: menu.container_id,
                    property_id: i16::try_from(index)
                        .expect("validated menu data index fits signed short"),
                    value,
                })
            }),
    );
}

fn validate_snapshot(snapshot: &MenuSnapshot) -> Result<(), ContainerPublicationError> {
    if snapshot.slots.len() > i16::MAX as usize + 1 {
        return Err(ContainerPublicationError::TooManySlots {
            slots: snapshot.slots.len(),
        });
    }
    if snapshot.data.len() > i16::MAX as usize + 1 {
        return Err(ContainerPublicationError::TooManyDataSlots {
            data: snapshot.data.len(),
        });
    }
    Ok(())
}

fn require_shape(
    menu: &PublishedMenu,
    authoritative: &MenuSnapshot,
) -> Result<(), ContainerPublicationError> {
    if menu.remote.menu_type != authoritative.menu_type {
        return Err(ContainerPublicationError::MenuTypeChanged);
    }
    if menu.remote.title != authoritative.title {
        return Err(ContainerPublicationError::TitleChanged);
    }
    if menu.remote.slots.len() != authoritative.slots.len() {
        return Err(ContainerPublicationError::SlotShapeChanged {
            expected: menu.remote.slots.len(),
            actual: authoritative.slots.len(),
        });
    }
    if menu.remote.data.len() != authoritative.data.len() {
        return Err(ContainerPublicationError::DataShapeChanged {
            expected: menu.remote.data.len(),
            actual: authoritative.data.len(),
        });
    }
    Ok(())
}

const fn increment_state(state_id: i32) -> i32 {
    state_id.wrapping_add(1) & MAX_STATE_ID
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContainerPublicationError {
    #[error("ordinary-menu publication has no current menu")]
    NoCurrentMenu,
    #[error("menu has {slots} slots, beyond signed-short delta indices")]
    TooManySlots { slots: usize },
    #[error("menu has {data} data slots, beyond signed-short property indices")]
    TooManyDataSlots { data: usize },
    #[error("menu type cannot change while broadcasting an existing menu")]
    MenuTypeChanged,
    #[error("menu title cannot change while broadcasting an existing menu")]
    TitleChanged,
    #[error("menu slot shape changed from {expected} to {actual}")]
    SlotShapeChanged { expected: usize, actual: usize },
    #[error("menu data shape changed from {expected} to {actual}")]
    DataShapeChanged { expected: usize, actual: usize },
    #[error("menu slot index {index} does not fit a signed short")]
    SlotIndex { index: usize },
    #[error("menu data index {index} does not fit a signed short")]
    DataIndex { index: usize },
}
