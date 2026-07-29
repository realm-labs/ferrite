//! Lectern content/state divergence, page control, pulse, and signals.

pub const BLOCK_ENTITY_PROTOCOL_ID: u32 = 27;
pub const PULSE_DELAY: u64 = 2;
pub const PAGE_SOUND_EVENT: u16 = 1043;
pub const BLOCK_UPDATE_FLAGS: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LecternState {
    pub has_book: bool,
    pub powered: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LecternContent {
    pub has_stack: bool,
    pub page_count: i32,
    pub page: i32,
    pub dirty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertResult {
    Rejected,
    Inserted {
        update_flags: u16,
        schedule_delay: Option<u64>,
    },
}

impl LecternContent {
    pub fn insert(&mut self, state: LecternState, page_count: i32) -> InsertResult {
        if state.has_book {
            return InsertResult::Rejected;
        }
        self.has_stack = true;
        self.page_count = page_count.max(0);
        self.page = 0;
        self.dirty = true;
        InsertResult::Inserted {
            update_flags: BLOCK_UPDATE_FLAGS,
            schedule_delay: None,
        }
    }

    pub fn set_page(&mut self, requested: i32, tick_already_scheduled: bool) -> PageResult {
        let page = if self.page_count == 0 {
            -1
        } else {
            requested.clamp(0, self.page_count - 1)
        };
        if page == self.page {
            return PageResult::Unchanged;
        }
        self.page = page;
        self.dirty = true;
        PageResult::Changed {
            page,
            schedule_delay: (!tick_already_scheduled).then_some(PULSE_DELAY),
            level_event: PAGE_SOUND_EVENT,
        }
    }

    pub fn load(has_stack: bool, page_count: i32, stored_page: i32) -> Self {
        let page = if page_count == 0 {
            -1
        } else {
            stored_page.clamp(0, page_count - 1)
        };
        Self {
            has_stack,
            page_count,
            page,
            dirty: false,
        }
    }

    pub fn analog_output(&self, state: LecternState) -> u8 {
        if !state.has_book {
            return 0;
        }
        if self.page_count <= 0 {
            return 14;
        }
        let progress = self.page as f32 / (self.page_count - 1).max(1) as f32;
        (progress * 14.0).floor() as u8 + u8::from(self.has_stack)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageResult {
    Unchanged,
    Changed {
        page: i32,
        schedule_delay: Option<u64>,
        level_event: u16,
    },
}

pub const fn weak_signal(state: LecternState) -> u8 {
    if state.powered { 15 } else { 0 }
}

pub const fn direct_signal(state: LecternState, queried_down: bool) -> u8 {
    if state.powered && queried_down { 15 } else { 0 }
}

pub const fn due_state(mut state: LecternState) -> LecternState {
    state.powered = false;
    state
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemovalOutcome {
    pub eject_book: bool,
    pub clear_state_book: bool,
    pub clear_content: bool,
}

pub const fn removal_outcome(captured_state: LecternState) -> RemovalOutcome {
    RemovalOutcome {
        eject_book: captured_state.has_book,
        clear_state_book: captured_state.has_book,
        clear_content: false,
    }
}
