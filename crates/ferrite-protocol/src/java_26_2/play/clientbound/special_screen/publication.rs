use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::java_26_2::play::clientbound::container::publication::{
    ContainerPublicationError, ContainerPublisher, MenuSnapshot,
};
use crate::java_26_2::play::clientbound::packet::{BlockUpdate, PlayClientboundPacket};
use crate::java_26_2::play::clientbound::special_screen::packet::{
    InteractionHand, MountScreenOpen, OpenSignEditor,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MountPublisher {
    containers: ContainerPublisher,
}

impl MountPublisher {
    pub fn open(
        &mut self,
        entity_id: i32,
        inventory_columns: i32,
        menu: MenuSnapshot,
    ) -> Result<Vec<PlayClientboundPacket>, ContainerPublicationError> {
        self.containers.open_specialized(menu, move |container_id| {
            PlayClientboundPacket::MountScreenOpen(MountScreenOpen {
                container_id,
                inventory_columns,
                entity_id,
            })
        })
    }

    pub const fn containers_mut(&mut self) -> &mut ContainerPublisher {
        &mut self.containers
    }
}

#[must_use]
pub fn publish_open_book(
    hand: InteractionHand,
    has_written_content: bool,
    resolution_changed_stack: bool,
    menu_changes: Vec<PlayClientboundPacket>,
) -> Vec<PlayClientboundPacket> {
    if !has_written_content {
        return Vec::new();
    }
    let mut packets = if resolution_changed_stack {
        menu_changes
    } else {
        Vec::new()
    };
    packets.push(PlayClientboundPacket::OpenBook(hand));
    packets
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableSign {
    pub position: BlockPos,
    pub block_state: i32,
    pub waxed: bool,
    pub editor: Option<u128>,
    pub front: [EditableSignLine; 4],
    pub back: [EditableSignLine; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditableSignLine {
    pub text: String,
    pub plain: bool,
}

impl EditableSignLine {
    fn editable(&self) -> bool {
        self.text.is_empty() || self.plain
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignOpenAdmission {
    pub player: u128,
    pub front_text: bool,
    pub command_consumed: bool,
    pub may_build: bool,
}

pub fn publish_open_sign(
    sign: &mut EditableSign,
    admission: SignOpenAdmission,
) -> Result<Vec<PlayClientboundPacket>, SignOpenError> {
    if admission.command_consumed {
        return Err(SignOpenError::CommandConsumed);
    }
    if sign.waxed {
        return Err(SignOpenError::Waxed);
    }
    if sign.editor.is_some_and(|editor| editor != admission.player) {
        return Err(SignOpenError::DifferentEditor);
    }
    if !admission.may_build {
        return Err(SignOpenError::BuildDenied);
    }
    let lines = if admission.front_text {
        &sign.front
    } else {
        &sign.back
    };
    if !lines.iter().all(EditableSignLine::editable) {
        return Err(SignOpenError::NonPlainMessage);
    }
    sign.editor = Some(admission.player);
    Ok(vec![
        PlayClientboundPacket::BlockUpdate(BlockUpdate {
            position: sign.position,
            state: sign.block_state,
        }),
        PlayClientboundPacket::OpenSignEditor(OpenSignEditor {
            position: sign.position,
            front_text: admission.front_text,
        }),
    ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SignOpenError {
    #[error("sign click command consumed the interaction")]
    CommandConsumed,
    #[error("waxed sign cannot open an editor")]
    Waxed,
    #[error("sign has a different active editor")]
    DifferentEditor,
    #[error("player lacks build permission")]
    BuildDenied,
    #[error("selected sign side contains a non-plain message")]
    NonPlainMessage,
}
