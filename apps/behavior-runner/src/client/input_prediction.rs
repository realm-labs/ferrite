//! Observable OS-input, key-mapping, tick/frame, and gameplay-action ordering.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyPolicy {
    Hold,
    Toggle {
        enabled: bool,
        restore_after_screen: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Press,
    Repeat,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputGate {
    pub correct_window: bool,
    pub screen_open: bool,
    pub screen_consumes: bool,
    pub screen_closed_during_handler: bool,
    pub overlay_open: bool,
    pub debug_action: bool,
}

impl Default for InputGate {
    fn default() -> Self {
        Self {
            correct_window: true,
            screen_open: false,
            screen_consumes: false,
            screen_closed_during_handler: false,
            overlay_open: false,
            debug_action: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMappingState {
    policy: KeyPolicy,
    down: bool,
    click_count: u32,
    released_by_screen_when_down: bool,
}

impl KeyMappingState {
    pub const fn new(policy: KeyPolicy) -> Self {
        Self {
            policy,
            down: false,
            click_count: 0,
            released_by_screen_when_down: false,
        }
    }

    pub fn keyboard_event(&mut self, action: InputAction, gate: InputGate) {
        if !gate.correct_window {
            return;
        }
        if gate.screen_open && gate.screen_consumes {
            if gate.screen_closed_during_handler {
                self.set_down(false);
            }
            return;
        }
        if action == InputAction::Release {
            self.set_down(false);
        } else if !gate.screen_open {
            if gate.debug_action {
                self.set_down(false);
            } else {
                self.set_down(true);
                self.click_count = self.click_count.saturating_add(1);
            }
        }
    }

    pub fn mouse_event(&mut self, action: InputAction, gate: InputGate) {
        if !gate.correct_window || (gate.screen_open && gate.screen_consumes) {
            return;
        }
        if !gate.screen_open && !gate.overlay_open {
            let pressed = action == InputAction::Press;
            self.set_down(pressed);
            if pressed {
                self.click_count = self.click_count.saturating_add(1);
            }
        }
    }

    pub fn release_for_focus_or_screen(&mut self) {
        self.click_count = 0;
        if matches!(self.policy, KeyPolicy::Toggle { enabled: true, .. }) && self.down
            || self.released_by_screen_when_down
        {
            self.released_by_screen_when_down = true;
        }
        self.down = false;
    }

    pub fn restore_after_screen_closed(&mut self, keyboard_binding: bool) {
        let restore = matches!(
            self.policy,
            KeyPolicy::Toggle {
                enabled: true,
                restore_after_screen: true
            }
        ) && keyboard_binding
            && self.released_by_screen_when_down;
        self.released_by_screen_when_down = false;
        if restore {
            self.down = true;
        }
    }

    pub fn resample_after_focus(&mut self, physically_down: bool, keyboard_binding: bool) {
        let can_resample =
            keyboard_binding && !matches!(self.policy, KeyPolicy::Toggle { enabled: true, .. });
        if can_resample {
            self.set_down(physically_down);
        }
    }

    pub fn consume_click(&mut self) -> bool {
        if self.click_count == 0 {
            return false;
        }
        self.click_count -= 1;
        true
    }

    pub const fn is_down(&self) -> bool {
        self.down
    }

    pub const fn click_count(&self) -> u32 {
        self.click_count
    }

    fn set_down(&mut self, down: bool) {
        match self.policy {
            KeyPolicy::Toggle { enabled: true, .. } if down => self.down = !self.down,
            KeyPolicy::Toggle { enabled: true, .. } => {}
            KeyPolicy::Hold | KeyPolicy::Toggle { enabled: false, .. } => self.down = down,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseMovement {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseAccumulator {
    position: MouseMovement,
    delta: MouseMovement,
    ignore_first: bool,
}

impl MouseAccumulator {
    pub const fn new(x: f64, y: f64, ignore_first: bool) -> Self {
        Self {
            position: MouseMovement { x, y },
            delta: MouseMovement { x: 0.0, y: 0.0 },
            ignore_first,
        }
    }

    pub fn on_move(&mut self, x: f64, y: f64, window_active: bool) {
        if self.ignore_first {
            self.position = MouseMovement { x, y };
            self.ignore_first = false;
            return;
        }
        if window_active {
            self.delta.x += x - self.position.x;
            self.delta.y += y - self.position.y;
        }
        self.position = MouseMovement { x, y };
    }

    pub fn render_frame(&mut self, window_active: bool) -> MouseMovement {
        let delta = if window_active {
            self.delta
        } else {
            MouseMovement { x: 0.0, y: 0.0 }
        };
        self.delta = MouseMovement { x: 0.0, y: 0.0 };
        delta
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameplayAction {
    ReleaseUse,
    StartAttack,
    StartUse,
    Pick,
    ContinueAttack(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameplayContext {
    pub using_item: bool,
    pub screen_open: bool,
    pub mouse_grabbed: bool,
    pub right_click_delay: u8,
    pub instant_attack: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameplayBindings {
    pub attack: KeyMappingState,
    pub use_item: KeyMappingState,
    pub pick: KeyMappingState,
}

impl Default for GameplayBindings {
    fn default() -> Self {
        Self {
            attack: KeyMappingState::new(KeyPolicy::Hold),
            use_item: KeyMappingState::new(KeyPolicy::Hold),
            pick: KeyMappingState::new(KeyPolicy::Hold),
        }
    }
}

impl GameplayBindings {
    pub fn client_tick(&mut self, context: GameplayContext) -> Vec<GameplayAction> {
        let mut actions = Vec::new();
        if context.using_item {
            if !self.use_item.is_down() {
                actions.push(GameplayAction::ReleaseUse);
            }
            drain(&mut self.attack);
            drain(&mut self.use_item);
            drain(&mut self.pick);
        } else {
            while self.attack.consume_click() {
                actions.push(GameplayAction::StartAttack);
            }
            while self.use_item.consume_click() {
                actions.push(GameplayAction::StartUse);
            }
            while self.pick.consume_click() {
                actions.push(GameplayAction::Pick);
            }
        }
        if self.use_item.is_down() && context.right_click_delay == 0 && !context.using_item {
            actions.push(GameplayAction::StartUse);
        }
        let continue_attack = !context.screen_open
            && !context.instant_attack
            && self.attack.is_down()
            && context.mouse_grabbed;
        actions.push(GameplayAction::ContinueAttack(continue_attack));
        actions
    }
}

fn drain(mapping: &mut KeyMappingState) {
    while mapping.consume_click() {}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientTimeDomains {
    pub client_ticks: u64,
    pub render_frames: u64,
    pub gameplay_cooldown: u32,
    pub last_partial_tick: f32,
}

impl Default for ClientTimeDomains {
    fn default() -> Self {
        Self {
            client_ticks: 0,
            render_frames: 0,
            gameplay_cooldown: 0,
            last_partial_tick: 0.0,
        }
    }
}

impl ClientTimeDomains {
    pub fn tick(&mut self, gameplay_running: bool) {
        self.client_ticks += 1;
        if gameplay_running {
            self.gameplay_cooldown = self.gameplay_cooldown.saturating_add(1);
        }
    }

    pub fn render(&mut self, partial_tick: f32) {
        self.render_frames += 1;
        self.last_partial_tick = partial_tick;
    }
}
