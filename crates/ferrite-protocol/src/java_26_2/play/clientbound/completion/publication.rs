use crate::java_26_2::play::clientbound::completion::packet::{
    CommandSuggestions, SuggestionEntry,
};
use crate::java_26_2::play::clientbound::completion::projection::CompletionRequest;

const MAX_PUBLISHED_SUGGESTIONS: usize = 1_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCompletionRequest<T> {
    pub transaction: i32,
    pub parsed: T,
}

pub fn parse_completion_request<T>(
    request: &CompletionRequest,
    parse: impl FnOnce(&str) -> T,
) -> ParsedCompletionRequest<T> {
    let command = request.input.strip_prefix('/').unwrap_or(&request.input);
    ParsedCompletionRequest {
        transaction: request.transaction,
        parsed: parse(command),
    }
}

#[must_use]
pub fn publish_completion<T>(
    request: &ParsedCompletionRequest<T>,
    start: i32,
    length: i32,
    entries: impl IntoIterator<Item = SuggestionEntry>,
) -> CommandSuggestions {
    CommandSuggestions {
        transaction: request.transaction,
        start,
        length,
        entries: entries
            .into_iter()
            .take(MAX_PUBLISHED_SUGGESTIONS)
            .collect(),
    }
}
