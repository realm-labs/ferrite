use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionEntry {
    pub text: String,
    pub tooltip: Option<TextComponentNbt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSuggestions {
    pub transaction: i32,
    pub start: i32,
    pub length: i32,
    pub entries: Vec<SuggestionEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomCompletionAction {
    Add,
    Remove,
    Set,
}

impl CustomCompletionAction {
    #[must_use]
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::Add => 0,
            Self::Remove => 1,
            Self::Set => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomChatCompletions {
    pub action: CustomCompletionAction,
    pub entries: Vec<String>,
}
