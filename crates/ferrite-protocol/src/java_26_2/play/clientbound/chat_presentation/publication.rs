//! Per-recipient chat publication and cache/index evolution.

use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::chat_presentation::packet::{
    BoundChatType, DisguisedChat, FilterMask, MessageSignature, PlayerChat,
    SignedMessageBodyPacked, SystemChat,
};
use crate::java_26_2::play::clientbound::chat_presentation::projection::{
    ChatVisibility, MessageSignatureCache,
};
use crate::java_26_2::value::nbt::TextComponentNbt;

const MAX_PENDING_SIGNATURES: usize = 4_096;
const FALLBACK_PREVIEW_CHARACTERS: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredChat {
    pub sender: u128,
    pub message_index: i32,
    pub signature: Option<MessageSignature>,
    pub body_content: String,
    pub timestamp_ms: i64,
    pub salt: i64,
    pub last_seen: Vec<MessageSignature>,
    pub unsigned_content: Option<TextComponentNbt>,
    pub filter_mask: FilterMask,
    pub decorated: TextComponentNbt,
    pub chat_type: BoundChatType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPublicationConnection {
    pub visibility: ChatVisibility,
    pub filters_message: bool,
    pub next_global_index: i32,
    pub cache: MessageSignatureCache,
    pub pending_signatures: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishedChatPacket {
    Player(PlayerChat),
    Disguised(DisguisedChat),
    System(SystemChat),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatDelivery {
    pub recipient: u64,
    pub packet: PublishedChatPacket,
    pub disconnect_after_send: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerBroadcastResult {
    pub deliveries: Vec<ChatDelivery>,
    pub notify_sender_fully_filtered: bool,
}

pub fn publish_player_chat(
    authored: &AuthoredChat,
    recipients: &mut BTreeMap<u64, ChatPublicationConnection>,
) -> PlayerBroadcastResult {
    let disguised = authored.sender == 0;
    let mut deliveries = Vec::new();
    let mut notify_sender_fully_filtered = false;
    for (recipient, connection) in recipients {
        if connection.visibility != ChatVisibility::Full {
            continue;
        }
        if disguised {
            deliveries.push(ChatDelivery {
                recipient: *recipient,
                packet: PublishedChatPacket::Disguised(DisguisedChat {
                    message: authored.decorated.clone(),
                    chat_type: authored.chat_type.clone(),
                }),
                disconnect_after_send: false,
            });
            continue;
        }
        let filter_mask = if connection.filters_message {
            authored.filter_mask.clone()
        } else {
            FilterMask::Pass
        };
        if connection.filters_message && matches!(authored.filter_mask, FilterMask::FullyFiltered) {
            notify_sender_fully_filtered = true;
        }
        if matches!(filter_mask, FilterMask::FullyFiltered) {
            continue;
        }
        let global_index = connection.next_global_index;
        connection.next_global_index = connection.next_global_index.wrapping_add(1);
        let body = SignedMessageBodyPacked {
            content: authored.body_content.clone(),
            timestamp_ms: authored.timestamp_ms,
            salt: authored.salt,
            last_seen: authored
                .last_seen
                .iter()
                .map(|signature| connection.cache.pack(signature))
                .collect(),
        };
        let packet = PlayerChat {
            global_index,
            sender: authored.sender,
            message_index: authored.message_index,
            signature: authored.signature.clone(),
            body,
            unsigned_content: authored.unsigned_content.clone(),
            filter_mask,
            chat_type: authored.chat_type.clone(),
        };
        let mut disconnect_after_send = false;
        if let Some(signature) = &authored.signature {
            connection.cache.push_batch([signature.clone()]);
            connection.pending_signatures = connection.pending_signatures.saturating_add(1);
            disconnect_after_send = connection.pending_signatures > MAX_PENDING_SIGNATURES;
        }
        deliveries.push(ChatDelivery {
            recipient: *recipient,
            packet: PublishedChatPacket::Player(packet),
            disconnect_after_send,
        });
    }
    PlayerBroadcastResult {
        deliveries,
        notify_sender_fully_filtered,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemRecipient {
    pub visibility: ChatVisibility,
    pub send_succeeds: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemDelivery {
    pub recipient: u64,
    pub packet: SystemChat,
    pub fallback: bool,
}

pub fn publish_system_chat(
    content: &TextComponentNbt,
    flattened: &str,
    overlay: bool,
    recipients: &BTreeMap<u64, SystemRecipient>,
    fallback_component: impl Fn(&str) -> TextComponentNbt,
) -> Vec<SystemDelivery> {
    let mut deliveries = Vec::new();
    for (recipient, state) in recipients {
        let visible = match state.visibility {
            ChatVisibility::Full | ChatVisibility::System => true,
            ChatVisibility::Hidden => overlay,
        };
        if !visible {
            continue;
        }
        if state.send_succeeds {
            deliveries.push(SystemDelivery {
                recipient: *recipient,
                packet: SystemChat {
                    content: content.clone(),
                    overlay,
                },
                fallback: false,
            });
        } else if !overlay {
            let preview: String = flattened
                .chars()
                .take(FALLBACK_PREVIEW_CHARACTERS)
                .collect();
            deliveries.push(SystemDelivery {
                recipient: *recipient,
                packet: SystemChat {
                    content: fallback_component(&preview),
                    overlay: false,
                },
                fallback: true,
            });
        }
    }
    deliveries
}
