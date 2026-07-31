use std::collections::BTreeSet;

use thiserror::Error;

use crate::java_26_2::play::clientbound::completion::packet::{
    CommandSuggestions, CustomChatCompletions, CustomCompletionAction, SuggestionEntry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuggestionRange {
    pub start: i32,
    pub end: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionResult {
    pub range: SuggestionRange,
    pub entries: Vec<SuggestionEntry>,
}

impl CompletionResult {
    pub fn validate_for_input(&self, input: &str) -> Result<(), CompletionUiError> {
        let input_length = i32::try_from(input.encode_utf16().count()).unwrap_or(i32::MAX);
        if self.range.start < 0
            || self.range.end < self.range.start
            || self.range.end > input_length
        {
            return Err(CompletionUiError::InvalidRange {
                start: self.range.start,
                end: self.range.end,
                input_length,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionRequest {
    pub transaction: i32,
    pub input: String,
    pub canceled_previous: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandCompletionOutcome {
    Completed(CompletionResult),
    IgnoredStale(CompletionResult),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CompletionProjectionError {
    #[error("command suggestion transaction -1 matched an absent pending future")]
    MissingPendingFuture,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CompletionUiError {
    #[error("suggestion range [{start}, {end}) is invalid for input length {input_length}")]
    InvalidRange {
        start: i32,
        end: i32,
        input_length: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionClientProjection {
    transaction_counter: i32,
    pending_transaction: i32,
    pending_future: bool,
    custom: BTreeSet<String>,
}

impl Default for CompletionClientProjection {
    fn default() -> Self {
        Self {
            transaction_counter: -1,
            pending_transaction: -1,
            pending_future: false,
            custom: BTreeSet::new(),
        }
    }
}

impl CompletionClientProjection {
    #[must_use]
    pub fn with_transaction_counter(transaction_counter: i32) -> Self {
        Self {
            transaction_counter,
            ..Self::default()
        }
    }

    pub fn begin_request(&mut self, input: impl Into<String>) -> CompletionRequest {
        let canceled_previous = self.pending_future;
        self.transaction_counter = self.transaction_counter.wrapping_add(1);
        self.pending_transaction = self.transaction_counter;
        self.pending_future = true;
        CompletionRequest {
            transaction: self.pending_transaction,
            input: input.into(),
            canceled_previous,
        }
    }

    pub fn apply_command(
        &mut self,
        packet: &CommandSuggestions,
    ) -> Result<CommandCompletionOutcome, CompletionProjectionError> {
        let converted = CompletionResult {
            range: SuggestionRange {
                start: packet.start,
                end: packet.start.wrapping_add(packet.length),
            },
            entries: packet.entries.clone(),
        };
        if packet.transaction != self.pending_transaction {
            return Ok(CommandCompletionOutcome::IgnoredStale(converted));
        }
        if !self.pending_future {
            return Err(CompletionProjectionError::MissingPendingFuture);
        }
        self.pending_future = false;
        self.pending_transaction = -1;
        Ok(CommandCompletionOutcome::Completed(converted))
    }

    pub fn apply_custom(&mut self, packet: &CustomChatCompletions) {
        if packet.action == CustomCompletionAction::Set {
            self.custom.clear();
        }
        for entry in &packet.entries {
            match packet.action {
                CustomCompletionAction::Add | CustomCompletionAction::Set => {
                    self.custom.insert(entry.clone());
                }
                CustomCompletionAction::Remove => {
                    self.custom.remove(entry);
                }
            }
        }
    }

    #[must_use]
    pub fn chat_candidates<'a>(
        &self,
        online_names: impl IntoIterator<Item = &'a str>,
    ) -> BTreeSet<String> {
        online_names
            .into_iter()
            .map(str::to_owned)
            .chain(self.custom.iter().cloned())
            .collect()
    }

    #[must_use]
    pub const fn pending_transaction(&self) -> i32 {
        self.pending_transaction
    }

    #[must_use]
    pub const fn has_pending_future(&self) -> bool {
        self.pending_future
    }

    #[must_use]
    pub fn custom_entries(&self) -> &BTreeSet<String> {
        &self.custom
    }
}
