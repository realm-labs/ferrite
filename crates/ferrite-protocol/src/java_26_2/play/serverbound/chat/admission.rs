use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::completion::packet::{
    CommandSuggestions, SuggestionEntry,
};
use crate::java_26_2::play::serverbound::chat::last_seen::{LastSeenError, LastSeenValidator};
use crate::java_26_2::play::serverbound::chat::packet::{
    ChatCommandSigned, ChatMessage, CommandSuggestion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatAdmission {
    Scheduled,
    DisconnectIllegalCharacters,
    DisconnectValidation(LastSeenError),
    DisabledByOptions,
}

pub fn admit_chat(
    validator: &mut LastSeenValidator,
    packet: &ChatMessage,
    visibility: ChatVisibility,
) -> ChatAdmission {
    if let Err(error) = validator.apply_update(packet.last_seen) {
        return ChatAdmission::DisconnectValidation(error);
    }
    if has_illegal_chat_character(&packet.message) {
        return ChatAdmission::DisconnectIllegalCharacters;
    }
    if visibility == ChatVisibility::Hidden {
        return ChatAdmission::DisabledByOptions;
    }
    ChatAdmission::Scheduled
}

pub fn admit_signed_command(
    validator: &mut LastSeenValidator,
    packet: &ChatCommandSigned,
) -> ChatAdmission {
    if let Err(error) = validator.apply_update(packet.last_seen) {
        return ChatAdmission::DisconnectValidation(error);
    }
    if has_illegal_chat_character(&packet.command) {
        ChatAdmission::DisconnectIllegalCharacters
    } else {
        ChatAdmission::Scheduled
    }
}

#[must_use]
pub fn admit_unsigned_command(command: &str) -> ChatAdmission {
    if has_illegal_chat_character(command) {
        ChatAdmission::DisconnectIllegalCharacters
    } else {
        ChatAdmission::Scheduled
    }
}

#[must_use]
pub fn has_illegal_chat_character(value: &str) -> bool {
    value.chars().any(|character| {
        character == '\u{00a7}' || character < '\u{0020}' || character == '\u{007f}'
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickThrottler {
    counter: i32,
    threshold_seconds: i32,
}

impl TickThrottler {
    #[must_use]
    pub const fn new(threshold_seconds: i32) -> Self {
        Self {
            counter: 0,
            threshold_seconds,
        }
    }

    #[must_use]
    pub const fn counter(self) -> i32 {
        self.counter
    }

    pub fn charge(&mut self, exempt: bool) -> bool {
        self.counter = self.counter.wrapping_add(20);
        let threshold = self.threshold_seconds.wrapping_mul(20);
        !exempt && threshold > 0 && self.counter >= threshold
    }

    pub fn tick(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionRequest {
    pub transaction_id: i32,
    pub parsed_input: String,
}

#[must_use]
pub fn normalize_suggestion(packet: &CommandSuggestion) -> SuggestionRequest {
    SuggestionRequest {
        transaction_id: packet.transaction_id,
        parsed_input: packet
            .input
            .strip_prefix('/')
            .unwrap_or(&packet.input)
            .to_owned(),
    }
}

pub fn truncate_suggestions<T>(suggestions: &mut Vec<T>) {
    suggestions.truncate(1_000);
}

#[must_use]
pub fn complete_suggestions(
    packet: &CommandSuggestion,
    start: i32,
    length: i32,
    mut entries: Vec<SuggestionEntry>,
) -> CommandSuggestions {
    truncate_suggestions(&mut entries);
    CommandSuggestions {
        transaction: packet.transaction_id,
        start,
        length,
        entries,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatFilterPolicy {
    Pass,
    FullyFiltered,
    PartiallyFiltered(Vec<i64>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedPlayerChat {
    pub sender: u128,
    pub signed_content: String,
    pub decorated_content: Option<String>,
    pub filter_policy: ChatFilterPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatFutureChain {
    connected: bool,
    next_token: u64,
    drain_token: u64,
    completed: BTreeMap<u64, NormalizedPlayerChat>,
}

impl Default for ChatFutureChain {
    fn default() -> Self {
        Self {
            connected: true,
            next_token: 0,
            drain_token: 0,
            completed: BTreeMap::new(),
        }
    }
}

impl ChatFutureChain {
    pub fn append(&mut self) -> Option<u64> {
        if !self.connected {
            return None;
        }
        let token = self.next_token;
        self.next_token = self.next_token.wrapping_add(1);
        Some(token)
    }

    pub fn complete(
        &mut self,
        token: u64,
        event: NormalizedPlayerChat,
    ) -> Vec<NormalizedPlayerChat> {
        if !self.connected || token < self.drain_token || token >= self.next_token {
            return Vec::new();
        }
        self.completed.insert(token, event);
        let mut ready = Vec::new();
        while let Some(event) = self.completed.remove(&self.drain_token) {
            ready.push(event);
            self.drain_token = self.drain_token.wrapping_add(1);
        }
        ready
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
        self.completed.clear();
    }
}
