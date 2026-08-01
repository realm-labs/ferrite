//! Java 26.2 container-screen gesture and dialog-control semantics.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuInput {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticClick {
    pub slot: i32,
    pub button: i32,
    pub input: MenuInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotGesture {
    pub index: i32,
    pub item: Option<String>,
    pub count: i32,
    pub maximum: i32,
    pub active: bool,
    pub may_pickup: bool,
    pub may_place: bool,
    pub same_container: bool,
}

impl SlotGesture {
    fn compatible_with_carried(&self, carried_item: Option<&str>) -> bool {
        self.item.is_none() || self.item.as_deref() == carried_item
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotBox {
    pub slot: SlotGesture,
    pub x: i32,
    pub y: i32,
}

pub fn hovered_slot(slots: &[SlotBox], pointer_x: f64, pointer_y: f64) -> Option<&SlotGesture> {
    slots
        .iter()
        .find(|slot| {
            let x = f64::from(slot.x);
            let y = f64::from(slot.y);
            slot.slot.active
                && pointer_x >= x - 1.0
                && pointer_x < x + 17.0
                && pointer_y >= y - 1.0
                && pointer_y < y + 17.0
        })
        .map(|slot| &slot.slot)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PointerModifiers {
    pub shift: bool,
    pub control: bool,
    pub creative: bool,
    pub pick_button: bool,
    pub offhand_button: bool,
    pub hotbar_button: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerPress {
    pub now_ms: u64,
    pub button: i32,
    pub outside: bool,
    pub carried_count: i32,
    pub modifiers: PointerModifiers,
    pub same_screen_interval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuGestureState {
    has_last_click: bool,
    last_slot: Option<i32>,
    last_button: i32,
    last_click_ms: u64,
    double_click: bool,
    skip_next_release: bool,
    quick_active: bool,
    quick_button: i32,
    quick_type: u8,
    quick_slots: Vec<QuickSlot>,
    pub quick_remainder: i32,
}

impl Default for MenuGestureState {
    fn default() -> Self {
        Self {
            has_last_click: false,
            last_slot: None,
            last_button: -1,
            last_click_ms: 0,
            double_click: false,
            skip_next_release: false,
            quick_active: false,
            quick_button: -1,
            quick_type: 0,
            quick_slots: Vec::new(),
            quick_remainder: 0,
        }
    }
}

impl MenuGestureState {
    pub fn press(&mut self, slot: Option<&SlotGesture>, press: PointerPress) -> Vec<SemanticClick> {
        let PointerPress {
            now_ms,
            button,
            outside,
            carried_count,
            modifiers,
            same_screen_interval,
        } = press;
        let slot_id = slot.map(|slot| slot.index);
        self.double_click = self.has_last_click
            && same_screen_interval
            && now_ms.saturating_sub(self.last_click_ms) < 250
            && self.last_button == button
            && self.last_slot == slot_id;
        self.last_click_ms = now_ms;
        self.last_button = button;
        self.last_slot = slot_id;
        self.has_last_click = true;

        let primary = matches!(button, 0 | 1) || (modifiers.pick_button && modifiers.creative);
        if primary {
            if carried_count == 0 && !self.quick_active {
                let click = if modifiers.pick_button && modifiers.creative && slot_id.is_some() {
                    SemanticClick {
                        slot: slot_id.unwrap_or(-999),
                        button: 2,
                        input: MenuInput::Clone,
                    }
                } else if modifiers.shift && slot_id.is_some() {
                    SemanticClick {
                        slot: slot_id.unwrap_or(-1),
                        button,
                        input: MenuInput::QuickMove,
                    }
                } else if outside {
                    SemanticClick {
                        slot: -999,
                        button,
                        input: MenuInput::Throw,
                    }
                } else {
                    SemanticClick {
                        slot: slot_id.unwrap_or(-1),
                        button,
                        input: MenuInput::Pickup,
                    }
                };
                self.skip_next_release = true;
                return vec![click];
            }
            if carried_count > 0 {
                self.quick_active = true;
                self.quick_button = button;
                self.quick_type = if modifiers.pick_button && modifiers.creative {
                    2
                } else if button == 1 {
                    1
                } else {
                    0
                };
                self.quick_slots.clear();
                self.quick_remainder = carried_count;
            }
            return Vec::new();
        }

        if carried_count == 0
            && let Some(slot) = slot.filter(|slot| slot.active)
        {
            if modifiers.offhand_button {
                return vec![SemanticClick {
                    slot: slot.index,
                    button: 40,
                    input: MenuInput::Swap,
                }];
            }
            if let Some(hotbar) = modifiers.hotbar_button.filter(|value| *value <= 8) {
                return vec![SemanticClick {
                    slot: slot.index,
                    button: i32::from(hotbar),
                    input: MenuInput::Swap,
                }];
            }
        }
        Vec::new()
    }

    pub fn drag(&mut self, slot: &SlotGesture, carried_item: Option<&str>, carried_count: i32) {
        if !self.quick_active
            || carried_count == 0
            || self
                .quick_slots
                .iter()
                .any(|candidate| candidate.index == slot.index)
            || !slot.active
            || !slot.may_place
            || !slot.compatible_with_carried(carried_item)
            || (self.quick_type != 2 && carried_count <= self.quick_slots.len() as i32)
        {
            return;
        }
        self.quick_slots.push(QuickSlot {
            index: slot.index,
            existing: slot.count,
            maximum: slot.maximum,
        });
        let per_slot = match self.quick_type {
            0 => carried_count / self.quick_slots.len() as i32,
            1 => 1,
            2 => i32::MAX,
            _ => 0,
        };
        self.quick_remainder = self
            .quick_slots
            .iter()
            .fold(carried_count, |remaining, slot| {
                let target = slot.existing.saturating_add(per_slot).min(slot.maximum);
                remaining.saturating_sub(target.saturating_sub(slot.existing))
            });
    }

    pub fn release(
        &mut self,
        button: i32,
        slot: Option<&SlotGesture>,
        carried_count: i32,
        modifiers: PointerModifiers,
        matching_slots: &[SlotGesture],
    ) -> Vec<SemanticClick> {
        if self.double_click && button == 0 {
            self.double_click = false;
            self.clear_quick();
            let Some(slot) = slot.filter(|slot| slot.may_pickup && slot.item.is_some()) else {
                return Vec::new();
            };
            if modifiers.shift {
                return matching_slots
                    .iter()
                    .filter(|candidate| {
                        candidate.same_container
                            && candidate.may_pickup
                            && candidate.item == slot.item
                    })
                    .map(|candidate| SemanticClick {
                        slot: candidate.index,
                        button,
                        input: MenuInput::QuickMove,
                    })
                    .collect();
            }
            return vec![SemanticClick {
                slot: slot.index,
                button,
                input: MenuInput::PickupAll,
            }];
        }
        if self.quick_active && button != self.quick_button {
            self.clear_quick();
            return Vec::new();
        }
        if self.skip_next_release {
            self.skip_next_release = false;
            return Vec::new();
        }
        if self.quick_active && !self.quick_slots.is_empty() {
            let mut clicks = Vec::with_capacity(self.quick_slots.len() + 2);
            clicks.push(quick_click(-999, 0, self.quick_type));
            clicks.extend(
                self.quick_slots
                    .iter()
                    .map(|slot| quick_click(slot.index, 1, self.quick_type)),
            );
            clicks.push(quick_click(-999, 2, self.quick_type));
            self.clear_quick();
            return clicks;
        }
        self.clear_quick();
        if carried_count == 0 {
            return Vec::new();
        }
        let input = if modifiers.pick_button && modifiers.creative {
            MenuInput::Clone
        } else if modifiers.shift && slot.is_some() {
            MenuInput::QuickMove
        } else {
            MenuInput::Pickup
        };
        vec![SemanticClick {
            slot: slot.map_or(-999, |slot| slot.index),
            button,
            input,
        }]
    }

    pub fn close(&mut self) {
        *self = Self::default();
    }

    pub fn keyboard(&self, slot: Option<&SlotGesture>, input: KeyboardInput) -> KeyboardGesture {
        let KeyboardInput {
            inventory_key,
            pick_key,
            drop_key,
            control,
            carried_empty,
            offhand_key,
            hotbar_key,
        } = input;
        if inventory_key {
            return KeyboardGesture {
                close: true,
                clicks: Vec::new(),
            };
        }
        let mut clicks = Vec::new();
        if let Some(slot) = slot.filter(|slot| slot.item.is_some()) {
            if pick_key {
                clicks.push(SemanticClick {
                    slot: slot.index,
                    button: 0,
                    input: MenuInput::Clone,
                });
            } else if drop_key {
                clicks.push(SemanticClick {
                    slot: slot.index,
                    button: i32::from(control),
                    input: MenuInput::Throw,
                });
            }
        }
        if carried_empty {
            let Some(slot) = slot.filter(|slot| slot.active) else {
                return KeyboardGesture {
                    close: false,
                    clicks,
                };
            };
            if offhand_key {
                clicks.push(SemanticClick {
                    slot: slot.index,
                    button: 40,
                    input: MenuInput::Swap,
                });
            } else if let Some(hotbar) = hotbar_key.filter(|value| *value <= 8) {
                clicks.push(SemanticClick {
                    slot: slot.index,
                    button: i32::from(hotbar),
                    input: MenuInput::Swap,
                });
            }
        }
        KeyboardGesture {
            close: false,
            clicks,
        }
    }

    fn clear_quick(&mut self) {
        self.quick_active = false;
        self.quick_slots.clear();
        self.quick_remainder = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct QuickSlot {
    index: i32,
    existing: i32,
    maximum: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardGesture {
    pub close: bool,
    pub clicks: Vec<SemanticClick>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyboardInput {
    pub inventory_key: bool,
    pub pick_key: bool,
    pub drop_key: bool,
    pub control: bool,
    pub carried_empty: bool,
    pub offhand_key: bool,
    pub hotbar_key: Option<u8>,
}

fn quick_click(slot: i32, phase: i32, kind: u8) -> SemanticClick {
    SemanticClick {
        slot,
        button: phase | (i32::from(kind) << 2),
        input: MenuInput::QuickCraft,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubmittedTag {
    Byte(i8),
    Int(i32),
    Float(f32),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BooleanControl {
    pub initial: bool,
    pub on_true: String,
    pub on_false: String,
}

impl Default for BooleanControl {
    fn default() -> Self {
        Self {
            initial: false,
            on_true: "true".to_owned(),
            on_false: "false".to_owned(),
        }
    }
}

impl BooleanControl {
    pub fn submit(&self, selected: bool) -> (SubmittedTag, String) {
        (
            SubmittedTag::Byte(i8::from(selected)),
            if selected {
                &self.on_true
            } else {
                &self.on_false
            }
            .clone(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberRangeControl {
    pub start: f32,
    pub end: f32,
    pub initial: Option<f32>,
    pub step: Option<f32>,
    pub width: u16,
}

impl NumberRangeControl {
    pub fn new(
        start: f32,
        end: f32,
        initial: Option<f32>,
        step: Option<f32>,
    ) -> Result<Self, ControlError> {
        let low = start.min(end);
        let high = start.max(end);
        if initial.is_some_and(|value| value < low || value > high) {
            return Err(ControlError::InitialOutOfRange);
        }
        if step.is_some_and(|value| !value.is_finite() || value <= 0.0) {
            return Err(ControlError::InvalidStep);
        }
        Ok(Self {
            start,
            end,
            initial,
            step,
            width: 200,
        })
    }

    pub fn slider_position(self, value: f32) -> f32 {
        if self.start == self.end {
            0.5
        } else {
            (value - self.start) / (self.end - self.start)
        }
    }

    pub fn normalize(self, value: f32) -> f32 {
        let low = self.start.min(self.end);
        let high = self.start.max(self.end);
        let Some(step) = self.step else {
            return value.clamp(low, high);
        };
        let base = self.initial.unwrap_or((self.start + self.end) / 2.0);
        let mut rounded = base + ((value - base) / step).round() * step;
        if rounded > high {
            rounded -= step;
        }
        if rounded < low {
            rounded += step;
        }
        rounded.clamp(low, high)
    }

    pub fn submit(self, value: f32) -> (SubmittedTag, String) {
        let value = self.normalize(value);
        let integer = value as i32;
        if integer as f32 == value {
            (SubmittedTag::Int(integer), integer.to_string())
        } else {
            (SubmittedTag::Float(value), value.to_string())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleOptionEntry {
    pub id: String,
    pub display: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleOptionControl {
    pub entries: Vec<SingleOptionEntry>,
    pub selected: usize,
}

impl SingleOptionControl {
    pub fn new(entries: Vec<SingleOptionEntry>, initial: &[usize]) -> Result<Self, ControlError> {
        if entries.is_empty() {
            return Err(ControlError::EmptyOptions);
        }
        if initial.len() > 1 || initial.first().is_some_and(|index| *index >= entries.len()) {
            return Err(ControlError::InvalidInitialOption);
        }
        Ok(Self {
            entries,
            selected: initial.first().copied().unwrap_or(0),
        })
    }

    pub fn cycle(&mut self) {
        self.selected = (self.selected + 1) % self.entries.len();
    }

    pub fn display(&self) -> &str {
        self.entries[self.selected]
            .display
            .as_deref()
            .unwrap_or(&self.entries[self.selected].id)
    }

    pub fn submit(&self) -> (SubmittedTag, String) {
        let id = self.entries[self.selected].id.clone();
        (SubmittedTag::String(id.clone()), id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextControl {
    pub width: u16,
    pub label_visible: bool,
    pub initial: String,
    pub max_length: usize,
    pub max_lines: Option<u16>,
    pub height: u16,
}

impl Default for TextControl {
    fn default() -> Self {
        Self {
            width: 200,
            label_visible: true,
            initial: String::new(),
            max_length: 32,
            max_lines: None,
            height: 20,
        }
    }
}

impl TextControl {
    pub fn single_line(initial: String, max_length: usize) -> Result<Self, ControlError> {
        Self::build(initial, max_length, None, Some(20))
    }

    pub fn multiline(
        initial: String,
        max_length: usize,
        max_lines: Option<u16>,
        height: Option<u16>,
    ) -> Result<Self, ControlError> {
        let lines = max_lines.unwrap_or(4);
        if lines == 0 {
            return Err(ControlError::InvalidLineCount);
        }
        let default_height = (9_u32 * u32::from(lines) + 8).min(512) as u16;
        Self::build(
            initial,
            max_length,
            Some(lines),
            Some(height.unwrap_or(default_height)),
        )
    }

    fn build(
        initial: String,
        max_length: usize,
        max_lines: Option<u16>,
        height: Option<u16>,
    ) -> Result<Self, ControlError> {
        if max_length == 0 || initial.chars().count() > max_length {
            return Err(ControlError::InvalidTextLength);
        }
        let height = height.expect("constructors always select a height");
        if !(1..=512).contains(&height) {
            return Err(ControlError::InvalidHeight);
        }
        Ok(Self {
            width: 200,
            label_visible: true,
            initial,
            max_length,
            max_lines,
            height,
        })
    }

    pub fn submit(
        &self,
        value: &str,
        template: &str,
    ) -> Result<(SubmittedTag, String), ControlError> {
        if value.chars().count() > self.max_length {
            return Err(ControlError::InvalidTextLength);
        }
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        Ok((
            SubmittedTag::String(value.to_owned()),
            template.replace("%s", &escaped),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlError {
    InitialOutOfRange,
    InvalidStep,
    EmptyOptions,
    InvalidInitialOption,
    InvalidTextLength,
    InvalidLineCount,
    InvalidHeight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlKind {
    Boolean,
    NumberRange,
    SingleOption,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlDispatch {
    Registered(ControlKind),
    Ignored {
        logged: bool,
        widget_added: bool,
        getter_added: bool,
    },
}

pub fn dispatch_control(identifier: &str) -> ControlDispatch {
    let kind = match identifier {
        "boolean" => ControlKind::Boolean,
        "number_range" => ControlKind::NumberRange,
        "single_option" => ControlKind::SingleOption,
        "text" => ControlKind::Text,
        _ => {
            return ControlDispatch::Ignored {
                logged: true,
                widget_added: false,
                getter_added: false,
            };
        }
    };
    ControlDispatch::Registered(kind)
}
