//! Connection-local chat cache, validation, delay, presentation, and deletion state.

use std::collections::{BTreeMap, VecDeque};

use thiserror::Error;

use crate::java_26_2::play::clientbound::chat_presentation::packet::{
    DeleteChat, DisguisedChat, FilterMask, MessageSignature, PackedMessageSignature, PlayerChat,
    SystemChat,
};

const SIGNATURE_CACHE_CAPACITY: usize = 128;
const LAST_SEEN_CAPACITY: usize = 20;
const CHAT_ACK_THRESHOLD: usize = 64;
const SECURE_WINDOW_MILLIS: i64 = 7 * 60 * 1_000;
const DELETE_DELAY_TICKS: u64 = 60;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageSignatureCache {
    entries: Vec<MessageSignature>,
}

impl MessageSignatureCache {
    pub fn unpack(
        &self,
        packed: &PackedMessageSignature,
    ) -> Result<Option<MessageSignature>, ChatProjectionError> {
        match packed {
            PackedMessageSignature::Full(signature) => Ok(Some(signature.clone())),
            PackedMessageSignature::CacheIndex(index) => {
                let index = usize::try_from(*index)
                    .map_err(|_| ChatProjectionError::InvalidCacheIndex { index: *index })?;
                if index >= SIGNATURE_CACHE_CAPACITY {
                    return Err(ChatProjectionError::InvalidCacheIndex {
                        index: i32::try_from(index).unwrap_or(i32::MAX),
                    });
                }
                Ok(self.entries.get(index).cloned())
            }
        }
    }

    pub fn push_batch(&mut self, signatures: impl IntoIterator<Item = MessageSignature>) {
        let mut queued = Vec::new();
        for signature in signatures {
            if !queued.contains(&signature) {
                queued.push(signature);
            }
        }
        if queued.is_empty() {
            return;
        }
        self.entries.retain(|entry| !queued.contains(entry));
        queued.reverse();
        queued.append(&mut self.entries);
        queued.truncate(SIGNATURE_CACHE_CAPACITY);
        self.entries = queued;
    }

    #[must_use]
    pub fn pack(&self, signature: &MessageSignature) -> PackedMessageSignature {
        self.entries
            .iter()
            .position(|candidate| candidate == signature)
            .and_then(|index| i32::try_from(index).ok())
            .map_or_else(
                || PackedMessageSignature::Full(signature.clone()),
                PackedMessageSignature::CacheIndex,
            )
    }

    #[must_use]
    pub fn entries(&self) -> &[MessageSignature] {
        &self.entries
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LastSeenTracker {
    entries: VecDeque<Option<MessageSignature>>,
    last_signature: Option<MessageSignature>,
    offset: usize,
}

impl LastSeenTracker {
    pub fn record(&mut self, signature: MessageSignature, shown: bool) {
        if self.last_signature.as_ref() == Some(&signature) {
            return;
        }
        self.last_signature = Some(signature.clone());
        self.entries.push_back(shown.then_some(signature));
        self.offset = self.offset.saturating_add(1);
        if self.entries.len() > LAST_SEEN_CAPACITY {
            self.entries.pop_front();
        }
    }

    pub fn clear_first(&mut self, signature: &MessageSignature) -> bool {
        let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.as_ref() == Some(signature))
        else {
            return false;
        };
        *entry = None;
        true
    }

    #[must_use]
    pub const fn acknowledgement_required(&self) -> bool {
        self.offset > CHAT_ACK_THRESHOLD
    }

    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    #[must_use]
    pub fn entries(&self) -> &VecDeque<Option<MessageSignature>> {
        &self.entries
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatTrust {
    Secure,
    Modified,
    NotSecure,
    ValidationError,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayKind {
    Player,
    ValidationError,
    Disguised,
    System,
    Overlay,
    DeletedMarker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayedChat {
    pub kind: DisplayKind,
    pub content: String,
    pub signature: Option<MessageSignature>,
    pub source: Option<u128>,
    pub trust: ChatTrust,
    pub added_tick: u64,
    pub used_unsigned_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenderChatState {
    pub profile_name: String,
    pub session_id: Option<u128>,
    pub session_expired: bool,
    pub validator_poisoned: bool,
    pub blocked: bool,
    pub friend: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationEvidence {
    pub profile_key_valid: bool,
    pub signature_valid: bool,
    pub chain_valid: bool,
    pub decorated_contains_signed: bool,
    pub unsigned_uses_default_font: bool,
}

impl Default for ValidationEvidence {
    fn default() -> Self {
        Self {
            profile_key_valid: true,
            signature_valid: true,
            chain_valid: true,
            decorated_contains_signed: true,
            unsigned_uses_default_font: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatPresentationPolicy {
    pub now_ms: i64,
    pub gui_tick: u64,
    pub local_profile: u128,
    pub integrated_server: bool,
    pub enforces_secure_chat: bool,
    pub secure_only: bool,
    pub visibility: ChatVisibility,
    pub friends_only: bool,
    pub local_receiver_allowed: bool,
    pub delay_seconds: f64,
    pub paused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatPresentationAction {
    Displayed,
    Queued,
    Suppressed,
    Deleted,
    DeleteScheduled,
    Noop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatClientOutcome {
    pub action: ChatPresentationAction,
    pub acknowledgement_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum QueuedKind {
    Normal {
        content: String,
        source: u128,
        trust: ChatTrust,
        used_unsigned_content: bool,
        allowed: bool,
    },
    ValidationError {
        source: Option<u128>,
        allowed: bool,
    },
    Disguised {
        allowed: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueuedChat {
    queue_key: Option<MessageSignature>,
    processed_signature: Option<MessageSignature>,
    kind: QueuedKind,
    due_ms: i64,
    gui_tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingDeletion {
    signature: MessageSignature,
    due_tick: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatClientProjection {
    pub next_global_index: i32,
    pub cache: MessageSignatureCache,
    pub last_seen: LastSeenTracker,
    pub senders: BTreeMap<u128, SenderChatState>,
    pub displayed: Vec<DisplayedChat>,
    queue: VecDeque<QueuedChat>,
    pending_deletions: Vec<PendingDeletion>,
    previous_message_ms: i64,
    discovered_names: BTreeMap<String, u128>,
    social: BTreeMap<u128, (bool, bool)>,
}

impl ChatClientProjection {
    pub fn install_sender(&mut self, id: u128, state: SenderChatState) {
        self.discovered_names.insert(state.profile_name.clone(), id);
        self.social.insert(id, (state.blocked, state.friend));
        self.senders.insert(id, state);
    }

    pub fn remove_sender(&mut self, id: u128) {
        self.senders.remove(&id);
    }

    pub fn apply_player(
        &mut self,
        packet: &PlayerChat,
        policy: &ChatPresentationPolicy,
        evidence: ValidationEvidence,
    ) -> Result<ChatClientOutcome, ChatProjectionError> {
        let expected = self.next_global_index;
        self.next_global_index = self.next_global_index.wrapping_add(1);
        if packet.global_index != expected {
            return Err(ChatProjectionError::BadGlobalIndex {
                expected,
                actual: packet.global_index,
            });
        }

        let mut unpacked = Vec::with_capacity(packet.body.last_seen.len());
        for packed in &packet.body.last_seen {
            unpacked.push(
                self.cache
                    .unpack(packed)?
                    .ok_or(ChatProjectionError::UnresolvedPackedSignature)?,
            );
        }
        let mut pushed = unpacked;
        pushed.extend(packet.signature.iter().cloned());
        self.cache.push_batch(pushed);

        let sender = self.senders.get_mut(&packet.sender);
        let validation_error = match sender {
            None => true,
            Some(sender) if sender.session_id.is_some() => {
                let rejected = sender.validator_poisoned
                    || sender.session_expired
                    || packet.signature.is_none()
                    || !evidence.profile_key_valid
                    || !evidence.signature_valid
                    || !evidence.chain_valid;
                if rejected {
                    sender.validator_poisoned = true;
                }
                rejected
            }
            Some(_) => policy.enforces_secure_chat,
        };
        if validation_error {
            let allowed = self.error_allowed(packet.sender, policy);
            return Ok(self.enqueue_or_execute(
                QueuedChat {
                    queue_key: None,
                    processed_signature: packet.signature.clone(),
                    kind: QueuedKind::ValidationError {
                        source: self
                            .senders
                            .contains_key(&packet.sender)
                            .then_some(packet.sender),
                        allowed,
                    },
                    due_ms: 0,
                    gui_tick: policy.gui_tick,
                },
                policy,
            ));
        }

        let effective_signature = if self
            .senders
            .get(&packet.sender)
            .is_some_and(|sender| sender.session_id.is_none())
        {
            None
        } else {
            packet.signature.clone()
        };
        let trust = trust(packet, policy, evidence, effective_signature.is_some());
        let mut allowed = policy.visibility == ChatVisibility::Full
            && policy.local_receiver_allowed
            && self.sender_allowed(packet.sender, policy)
            && !matches!(packet.filter_mask, FilterMask::FullyFiltered);
        if policy.secure_only && trust == ChatTrust::NotSecure {
            allowed = false;
        }
        let content = match &packet.filter_mask {
            FilterMask::PartiallyFiltered(words) => {
                apply_partial_mask(&packet.body.content, words)?
            }
            FilterMask::Pass | FilterMask::FullyFiltered => packet.body.content.clone(),
        };
        let used_unsigned_content = packet.unsigned_content.is_some()
            && !policy.secure_only
            && matches!(packet.filter_mask, FilterMask::Pass);
        Ok(self.enqueue_or_execute(
            QueuedChat {
                queue_key: effective_signature.clone(),
                processed_signature: effective_signature,
                kind: QueuedKind::Normal {
                    content,
                    source: packet.sender,
                    trust,
                    used_unsigned_content,
                    allowed,
                },
                due_ms: 0,
                gui_tick: policy.gui_tick,
            },
            policy,
        ))
    }

    pub fn apply_disguised(
        &mut self,
        _packet: &DisguisedChat,
        policy: &ChatPresentationPolicy,
    ) -> ChatClientOutcome {
        self.enqueue_or_execute(
            QueuedChat {
                queue_key: None,
                processed_signature: None,
                kind: QueuedKind::Disguised {
                    allowed: policy.visibility == ChatVisibility::Full,
                },
                due_ms: 0,
                gui_tick: policy.gui_tick,
            },
            policy,
        )
    }

    pub fn apply_system(
        &mut self,
        _packet: &SystemChat,
        plain_text: &str,
        policy: &ChatPresentationPolicy,
    ) -> ChatClientOutcome {
        let allowed = if _packet.overlay {
            true
        } else {
            matches!(
                policy.visibility,
                ChatVisibility::Full | ChatVisibility::System
            ) && self.system_sender_allowed(plain_text, policy)
        };
        if !allowed {
            return outcome(ChatPresentationAction::Suppressed, &self.last_seen);
        }
        self.displayed.push(DisplayedChat {
            kind: if _packet.overlay {
                DisplayKind::Overlay
            } else {
                DisplayKind::System
            },
            content: plain_text.to_owned(),
            signature: None,
            source: None,
            trust: ChatTrust::System,
            added_tick: policy.gui_tick,
            used_unsigned_content: false,
        });
        outcome(ChatPresentationAction::Displayed, &self.last_seen)
    }

    pub fn apply_delete(
        &mut self,
        packet: &DeleteChat,
        gui_tick: u64,
    ) -> Result<ChatClientOutcome, ChatProjectionError> {
        let signature = self
            .cache
            .unpack(&packet.signature)?
            .ok_or(ChatProjectionError::UnresolvedPackedSignature)?;
        self.last_seen.clear_first(&signature);
        let before = self.queue.len();
        self.queue
            .retain(|queued| queued.queue_key.as_ref() != Some(&signature));
        if self.queue.len() != before {
            return Ok(outcome(ChatPresentationAction::Deleted, &self.last_seen));
        }
        let Some(line) = self
            .displayed
            .iter_mut()
            .find(|line| line.signature.as_ref() == Some(&signature))
        else {
            return Ok(outcome(ChatPresentationAction::Noop, &self.last_seen));
        };
        if gui_tick.saturating_sub(line.added_tick) >= DELETE_DELAY_TICKS {
            replace_deleted(line);
            Ok(outcome(ChatPresentationAction::Deleted, &self.last_seen))
        } else {
            self.pending_deletions.push(PendingDeletion {
                signature,
                due_tick: line.added_tick.saturating_add(DELETE_DELAY_TICKS),
            });
            Ok(outcome(
                ChatPresentationAction::DeleteScheduled,
                &self.last_seen,
            ))
        }
    }

    pub fn tick(&mut self, now_ms: i64, gui_tick: u64, paused: bool) -> Vec<ChatClientOutcome> {
        if paused {
            self.previous_message_ms = self.previous_message_ms.saturating_add(50);
            for queued in &mut self.queue {
                queued.due_ms = queued.due_ms.saturating_add(50);
            }
        }
        let mut outcomes = Vec::new();
        while self
            .queue
            .front()
            .is_some_and(|queued| queued.due_ms <= now_ms)
        {
            let queued = self.queue.pop_front().expect("front was present");
            let shown = queued_is_allowed(&queued);
            outcomes.push(self.execute(queued, now_ms));
            if shown {
                break;
            }
        }
        let mut remaining = Vec::new();
        for deletion in self.pending_deletions.drain(..) {
            if deletion.due_tick <= gui_tick {
                if let Some(line) = self
                    .displayed
                    .iter_mut()
                    .find(|line| line.signature.as_ref() == Some(&deletion.signature))
                {
                    replace_deleted(line);
                    outcomes.push(outcome(ChatPresentationAction::Deleted, &self.last_seen));
                }
            } else {
                remaining.push(deletion);
            }
        }
        self.pending_deletions = remaining;
        outcomes
    }

    pub fn set_delay_seconds(
        &mut self,
        seconds: f64,
        now_ms: i64,
        gui_tick: u64,
        paused: bool,
    ) -> Vec<ChatClientOutcome> {
        if seconds != 0.0 || paused {
            return Vec::new();
        }
        let mut outcomes = Vec::new();
        while let Some(mut queued) = self.queue.pop_front() {
            queued.gui_tick = gui_tick;
            outcomes.push(self.execute(queued, now_ms));
        }
        self.previous_message_ms = 0;
        outcomes
    }

    #[must_use]
    pub fn queued_len(&self) -> usize {
        self.queue.len()
    }

    fn enqueue_or_execute(
        &mut self,
        mut queued: QueuedChat,
        policy: &ChatPresentationPolicy,
    ) -> ChatClientOutcome {
        let delay_ms = delay_millis(policy.delay_seconds);
        let due_ms = self.previous_message_ms.saturating_add(delay_ms);
        if delay_ms > 0 && policy.now_ms < due_ms {
            queued.due_ms = due_ms;
            self.queue.push_back(queued);
            outcome(ChatPresentationAction::Queued, &self.last_seen)
        } else {
            self.execute(queued, policy.now_ms)
        }
    }

    fn execute(&mut self, queued: QueuedChat, now_ms: i64) -> ChatClientOutcome {
        let (display, processed_signature, signature_was_shown) = match queued.kind {
            QueuedKind::Normal {
                content,
                source,
                trust,
                used_unsigned_content,
                allowed,
            } => (
                allowed.then_some(DisplayedChat {
                    kind: DisplayKind::Player,
                    content,
                    signature: queued.queue_key.clone(),
                    source: Some(source),
                    trust,
                    added_tick: queued.gui_tick,
                    used_unsigned_content,
                }),
                queued.processed_signature,
                allowed,
            ),
            QueuedKind::ValidationError { source, allowed } => (
                allowed.then_some(DisplayedChat {
                    kind: DisplayKind::ValidationError,
                    content: "multiplayer.disconnect.unsigned_chat".to_owned(),
                    signature: None,
                    source,
                    trust: ChatTrust::ValidationError,
                    added_tick: queued.gui_tick,
                    used_unsigned_content: false,
                }),
                queued.processed_signature,
                false,
            ),
            QueuedKind::Disguised { allowed } => (
                allowed.then_some(DisplayedChat {
                    kind: DisplayKind::Disguised,
                    content: "disguised".to_owned(),
                    signature: None,
                    source: None,
                    trust: ChatTrust::System,
                    added_tick: queued.gui_tick,
                    used_unsigned_content: false,
                }),
                None,
                false,
            ),
        };
        let shown = display.is_some();
        if let Some(display) = display {
            self.displayed.push(display);
            self.previous_message_ms = now_ms;
        }
        if let Some(signature) = processed_signature {
            self.last_seen.record(signature, signature_was_shown);
        }
        outcome(
            if shown {
                ChatPresentationAction::Displayed
            } else {
                ChatPresentationAction::Suppressed
            },
            &self.last_seen,
        )
    }

    fn sender_allowed(&self, sender: u128, policy: &ChatPresentationPolicy) -> bool {
        self.social
            .get(&sender)
            .is_some_and(|(blocked, friend)| !blocked && (!policy.friends_only || *friend))
    }

    fn error_allowed(&self, sender: u128, policy: &ChatPresentationPolicy) -> bool {
        policy.visibility == ChatVisibility::Full
            && policy.local_receiver_allowed
            && self
                .social
                .get(&sender)
                .is_none_or(|(blocked, friend)| !blocked && (!policy.friends_only || *friend))
    }

    fn system_sender_allowed(&self, text: &str, policy: &ChatPresentationPolicy) -> bool {
        let Some(name) = guessed_sender(text) else {
            return true;
        };
        self.discovered_names
            .get(name)
            .and_then(|id| self.social.get(id))
            .is_none_or(|(blocked, friend)| !blocked && (!policy.friends_only || *friend))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChatProjectionError {
    #[error("chat global index expected {expected}, received {actual}")]
    BadGlobalIndex { expected: i32, actual: i32 },
    #[error("packed signature cache index {index} is outside 0..128")]
    InvalidCacheIndex { index: i32 },
    #[error("packed signature references an empty cache slot")]
    UnresolvedPackedSignature,
    #[error("partial filter bit {position} is outside message length {length}")]
    FilterPositionOutOfRange { position: usize, length: usize },
}

fn trust(
    packet: &PlayerChat,
    policy: &ChatPresentationPolicy,
    evidence: ValidationEvidence,
    has_signature: bool,
) -> ChatTrust {
    if policy.integrated_server && packet.sender == policy.local_profile {
        return ChatTrust::Secure;
    }
    let age = policy.now_ms.saturating_sub(packet.body.timestamp_ms);
    if has_signature && age <= SECURE_WINDOW_MILLIS {
        if !evidence.decorated_contains_signed
            || (packet.unsigned_content.is_some() && !evidence.unsigned_uses_default_font)
        {
            ChatTrust::Modified
        } else {
            ChatTrust::Secure
        }
    } else {
        ChatTrust::NotSecure
    }
}

fn apply_partial_mask(content: &str, words: &[i64]) -> Result<String, ChatProjectionError> {
    let mut characters: Vec<char> = content.chars().collect();
    for (word_index, word) in words.iter().enumerate() {
        let mut bits = *word as u64;
        while bits != 0 {
            let bit = bits.trailing_zeros() as usize;
            let position = word_index.saturating_mul(64).saturating_add(bit);
            let length = characters.len();
            let character = characters
                .get_mut(position)
                .ok_or(ChatProjectionError::FilterPositionOutOfRange { position, length })?;
            *character = '#';
            bits &= bits - 1;
        }
    }
    Ok(characters.into_iter().collect())
}

fn guessed_sender(text: &str) -> Option<&str> {
    let start = text.find('<')?.saturating_add(1);
    let end = text[start..].find('>')?.saturating_add(start);
    Some(&text[start..end])
}

fn delay_millis(seconds: f64) -> i64 {
    let millis = seconds * 1_000.0;
    if millis.is_nan() {
        0
    } else if millis >= i64::MAX as f64 {
        i64::MAX
    } else if millis <= i64::MIN as f64 {
        i64::MIN
    } else {
        millis as i64
    }
}

fn queued_is_allowed(queued: &QueuedChat) -> bool {
    match queued.kind {
        QueuedKind::Normal { allowed, .. }
        | QueuedKind::ValidationError { allowed, .. }
        | QueuedKind::Disguised { allowed } => allowed,
    }
}

fn replace_deleted(line: &mut DisplayedChat) {
    line.kind = DisplayKind::DeletedMarker;
    line.content = "chat.deleted_marker".to_owned();
    line.signature = None;
    line.source = None;
    line.trust = ChatTrust::System;
    line.used_unsigned_content = false;
}

fn outcome(action: ChatPresentationAction, tracker: &LastSeenTracker) -> ChatClientOutcome {
    ChatClientOutcome {
        action,
        acknowledgement_required: tracker.acknowledgement_required(),
    }
}
