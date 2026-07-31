use ferrite_foundation::coordinate::BlockPos;

use crate::java_26_2::play::serverbound::sign_update::packet::SignUpdate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredLine {
    pub raw: String,
    pub filtered_or_empty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSignSubmission {
    pub position: BlockPos,
    pub front_text: bool,
    pub stripped_lines: [String; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignEditorProjection {
    position: BlockPos,
    front_text: bool,
    lines: [String; 4],
    removed: bool,
}

impl SignEditorProjection {
    #[must_use]
    pub const fn new(position: BlockPos, front_text: bool, lines: [String; 4]) -> Self {
        Self {
            position,
            front_text,
            lines,
            removed: false,
        }
    }

    pub fn lines_mut(&mut self) -> &mut [String; 4] {
        &mut self.lines
    }

    pub fn removed(&mut self, connection_exists: bool) -> Option<SignUpdate> {
        if self.removed {
            return None;
        }
        self.removed = true;
        connection_exists.then(|| SignUpdate {
            position: self.position,
            front_text: self.front_text,
            lines: self.lines.clone(),
        })
    }
}

impl PendingSignSubmission {
    #[must_use]
    pub fn from_packet(packet: SignUpdate) -> Self {
        Self {
            position: packet.position,
            front_text: packet.front_text,
            stripped_lines: packet.lines.map(|line| strip_legacy_formatting(&line)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignLine {
    pub literal: String,
    pub filtered_literal: Option<String>,
    pub style: String,
}

impl SignLine {
    #[must_use]
    pub fn empty(style: impl Into<String>) -> Self {
        Self {
            literal: String::new(),
            filtered_literal: None,
            style: style.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignText {
    pub lines: [SignLine; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignEntity {
    pub waxed: bool,
    pub has_level: bool,
    pub allowed_editor: Option<u128>,
    pub front: SignText,
    pub back: SignText,
    pub changed_calls: u32,
    pub block_update_flags: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignCompletionState {
    Unloaded,
    MissingBlockEntity,
    OtherBlockEntity,
    Sign(Box<SignEntity>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignUpdateOutcome {
    IgnoredUnloaded,
    IgnoredMissingSign,
    RejectedAuthorization,
    Applied,
}

pub fn complete_sign_update(
    state: &mut SignCompletionState,
    submission: &PendingSignSubmission,
    filtered: [FilteredLine; 4],
    sender: u128,
    player_uses_filtering: bool,
) -> SignUpdateOutcome {
    let sign = match state {
        SignCompletionState::Unloaded => return SignUpdateOutcome::IgnoredUnloaded,
        SignCompletionState::MissingBlockEntity | SignCompletionState::OtherBlockEntity => {
            return SignUpdateOutcome::IgnoredMissingSign;
        }
        SignCompletionState::Sign(sign) => sign,
    };
    if sign.waxed || !sign.has_level || sign.allowed_editor != Some(sender) {
        return SignUpdateOutcome::RejectedAuthorization;
    }

    let selected = if submission.front_text {
        &mut sign.front
    } else {
        &mut sign.back
    };
    for (current, filtered) in selected.lines.iter_mut().zip(filtered) {
        *current = SignLine {
            literal: if player_uses_filtering {
                filtered.filtered_or_empty.clone()
            } else {
                filtered.raw
            },
            filtered_literal: (!player_uses_filtering).then_some(filtered.filtered_or_empty),
            style: current.style.clone(),
        };
    }
    sign.changed_calls = sign.changed_calls.wrapping_add(1);
    sign.block_update_flags.push(3);
    sign.allowed_editor = None;
    sign.block_update_flags.push(3);
    SignUpdateOutcome::Applied
}

pub fn tick_editor_authorization(sign: &mut SignEntity, editor_present_and_in_padded_range: bool) {
    if sign.allowed_editor.is_some() && !editor_present_and_in_padded_range {
        sign.allowed_editor = None;
    }
}

#[must_use]
pub fn strip_legacy_formatting(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character == '\u{00a7}'
            && let Some(next) = characters.next()
        {
            if is_legacy_format_code(next) {
                continue;
            }
            output.push(character);
            output.push(next);
            continue;
        }
        output.push(character);
    }
    output
}

fn is_legacy_format_code(character: char) -> bool {
    matches!(
        character.to_ascii_lowercase(),
        '0'..='9' | 'a'..='f' | 'k'..='o' | 'r'
    )
}
