//! Sign side selection, editor lease, applicators, clicks, and text commits.

pub const WOOD_TYPES: usize = 12;
pub const BLOCK_COUNT: usize = 48;
pub const STATE_COUNT: usize = 1_344;
pub const ORDINARY_LINE_WIDTH: u16 = 90;
pub const HANGING_LINE_WIDTH: u16 = 60;
pub const EDITOR_EXTRA_RANGE: f64 = 4.0;
pub const WAX_LEVEL_EVENT: u16 = 3003;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignSide {
    Front,
    Back,
}

pub fn selected_side(sign_yaw: f32, player_angle: f32) -> SignSide {
    let difference = (player_angle - sign_yaw + 180.0).rem_euclid(360.0) - 180.0;
    if difference.abs() <= 90.0 {
        SignSide::Front
    } else {
        SignSide::Back
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickAction {
    RunCommand(String),
    ShowDialog(String),
    Custom(String),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignLine {
    pub text: String,
    pub editable_literal: bool,
    pub click: Option<ClickAction>,
}

impl Default for SignLine {
    fn default() -> Self {
        Self {
            text: String::new(),
            editable_literal: true,
            click: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignText {
    pub lines: [SignLine; 4],
    pub color: u8,
    pub glowing: bool,
}

impl Default for SignText {
    fn default() -> Self {
        Self {
            lines: std::array::from_fn(|_| SignLine::default()),
            color: 0,
            glowing: false,
        }
    }
}

impl SignText {
    pub fn has_visible_text(&self) -> bool {
        self.lines.iter().any(|line| !line.text.is_empty())
    }

    pub fn editable(&self) -> bool {
        self.lines.iter().all(|line| line.editable_literal)
    }

    pub fn click_actions(&self) -> Vec<&ClickAction> {
        self.lines
            .iter()
            .filter_map(|line| line.click.as_ref())
            .filter(|action| !matches!(action, ClickAction::Unsupported))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicator {
    Dye(u8),
    GlowInk,
    Ink,
    Honeycomb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicatorResult {
    Blocked,
    NoChange,
    Changed { level_event: Option<u16> },
}

pub fn apply(text: &mut SignText, waxed: &mut bool, applicator: Applicator) -> ApplicatorResult {
    if *waxed {
        return ApplicatorResult::Blocked;
    }
    match applicator {
        Applicator::Dye(color) if text.has_visible_text() && text.color != color => {
            text.color = color;
            ApplicatorResult::Changed { level_event: None }
        }
        Applicator::GlowInk if text.has_visible_text() && !text.glowing => {
            text.glowing = true;
            ApplicatorResult::Changed { level_event: None }
        }
        Applicator::Ink if text.has_visible_text() && text.glowing => {
            text.glowing = false;
            ApplicatorResult::Changed { level_event: None }
        }
        Applicator::Honeycomb if !*waxed => {
            *waxed = true;
            ApplicatorResult::Changed {
                level_event: Some(WAX_LEVEL_EVENT),
            }
        }
        _ => ApplicatorResult::NoChange,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorLease {
    pub editor: Option<u128>,
}

impl EditorLease {
    pub fn can_mutate(self, player: u128) -> bool {
        self.editor.is_none() || self.editor == Some(player)
    }

    pub fn open(&mut self, player: u128) {
        self.editor = Some(player);
    }

    pub fn tick(&mut self, player_resolves: bool, in_range: bool) {
        if !player_resolves || !in_range {
            self.editor = None;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyHandResult {
    Pass,
    ClickHandled,
    WaxedFailure,
    OpenEditor,
}

pub fn empty_hand(
    text: &SignText,
    waxed: bool,
    lease_admits: bool,
    may_build: bool,
) -> EmptyHandResult {
    let clicked = !text.click_actions().is_empty();
    if waxed {
        EmptyHandResult::WaxedFailure
    } else if clicked {
        EmptyHandResult::ClickHandled
    } else if lease_admits && may_build && text.editable() {
        EmptyHandResult::OpenEditor
    } else {
        EmptyHandResult::Pass
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditResult {
    Rejected,
    Accepted { update_requests: u8 },
}

pub fn commit_edit(
    text: &mut SignText,
    waxed: bool,
    authorized: bool,
    filtered: [&str; 4],
) -> EditResult {
    if waxed || !authorized {
        return EditResult::Rejected;
    }
    let mut changed = false;
    for (line, replacement) in text.lines.iter_mut().zip(filtered) {
        if line.text != replacement {
            line.text = replacement.to_owned();
            changed = true;
        }
    }
    EditResult::Accepted {
        update_requests: 1 + u8::from(changed),
    }
}

pub const fn hanging_chain_precedence(
    waxed_visible_run_command: bool,
    held_hanging_sign: bool,
    face_admits_chain: bool,
) -> bool {
    held_hanging_sign && face_admits_chain && !waxed_visible_run_command
}

pub const fn renderer_light(glowing: bool, ordinary_light: u32) -> u32 {
    if glowing { 15_728_880 } else { ordinary_light }
}

pub const fn outline_visible(color_is_black: bool, scoped: bool, distance_squared: f64) -> bool {
    color_is_black || scoped || distance_squared < 256.0
}
