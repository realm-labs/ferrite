//! Redstone-driven test-block state, edits, latches, and scan precedence.

pub const BLOCK_ENTITY_PROTOCOL_ID: u32 = 45;
pub const MESSAGE_UI_MAX_UTF16: usize = 128;
pub const MESSAGE_PACKET_MAX_UTF16: usize = 32_767;
pub const EDIT_WRITE_FLAGS: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

impl TestBlockMode {
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::Start => 0,
            Self::Log => 1,
            Self::Fail => 2,
            Self::Accept => 3,
        }
    }

    pub const fn from_wire_id(value: i32) -> Self {
        match value {
            1 => Self::Log,
            2 => Self::Fail,
            3 => Self::Accept,
            _ => Self::Start,
        }
    }

    pub const fn state_id(self) -> u32 {
        21_738 + self.wire_id() as u32
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestBlockEntity {
    pub mode: TestBlockMode,
    pub message: String,
    pub powered: bool,
    pub triggered: bool,
}

impl TestBlockEntity {
    pub fn new(mode: TestBlockMode) -> Self {
        Self {
            mode,
            message: String::new(),
            powered: false,
            triggered: false,
        }
    }

    pub fn neighbor_signal(&mut self, live_signal: bool) -> bool {
        if matches!(self.mode, TestBlockMode::Start) || self.powered == live_signal {
            return false;
        }
        self.powered = live_signal;
        if live_signal {
            self.trigger();
            true
        } else {
            false
        }
    }

    pub fn trigger(&mut self) {
        if matches!(self.mode, TestBlockMode::Start) {
            self.powered = true;
        } else {
            self.triggered = true;
        }
    }

    pub fn reset(&mut self) -> bool {
        self.triggered = false;
        if matches!(self.mode, TestBlockMode::Start) {
            self.powered = false;
            true
        } else {
            false
        }
    }

    pub fn ordinary_signal(&self, live_mode: TestBlockMode) -> u8 {
        if matches!(live_mode, TestBlockMode::Start) && self.powered {
            15
        } else {
            0
        }
    }

    pub fn load(mode: Option<TestBlockMode>, message: Option<&str>, powered: Option<bool>) -> Self {
        Self {
            mode: mode.unwrap_or(TestBlockMode::Fail),
            message: message.unwrap_or_default().to_owned(),
            powered: powered.unwrap_or(false),
            triggered: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditEffect {
    SetEntityMode,
    StateWrite { flags: u16 },
    SetMessage,
    Dirty,
    DirectBlockUpdate,
}

pub const EDIT_EFFECTS: [EditEffect; 5] = [
    EditEffect::SetEntityMode,
    EditEffect::StateWrite {
        flags: EDIT_WRITE_FLAGS,
    },
    EditEffect::SetMessage,
    EditEffect::Dirty,
    EditEffect::DirectBlockUpdate,
];

pub fn edit(
    entity: &mut TestBlockEntity,
    mode: TestBlockMode,
    message: impl Into<String>,
) -> [EditEffect; 5] {
    entity.mode = mode;
    entity.message = message.into();
    EDIT_EFFECTS
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanResult {
    MissingAccept,
    Running { reset_logs: usize },
    Success,
    Failure(String),
}

pub fn scan_outcomes(
    accepts: &[TestBlockEntity],
    failures: &[TestBlockEntity],
    logs: &[TestBlockEntity],
) -> ScanResult {
    if accepts.is_empty() {
        return ScanResult::MissingAccept;
    }
    if accepts.iter().any(|entity| entity.triggered) {
        return ScanResult::Success;
    }
    if let Some(entity) = failures.iter().find(|entity| entity.triggered) {
        return ScanResult::Failure(entity.message.clone());
    }
    ScanResult::Running {
        reset_logs: logs.iter().filter(|entity| entity.triggered).count(),
    }
}

pub const fn start_cardinality_valid(count: usize) -> bool {
    count == 1
}

pub fn truncate_ui_message(value: &str) -> String {
    let mut units = value
        .encode_utf16()
        .take(MESSAGE_UI_MAX_UTF16)
        .collect::<Vec<_>>();
    if units
        .last()
        .is_some_and(|unit| (0xd800..=0xdbff).contains(unit))
    {
        units.pop();
    }
    String::from_utf16_lossy(&units)
}
