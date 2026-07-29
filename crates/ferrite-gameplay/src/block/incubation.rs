//! Sniffer-Egg interval, crack, and hatch ordering.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnifferEgg {
    pub hatch: u8,
}

impl SnifferEgg {
    pub fn new(hatch: u8) -> Option<Self> {
        if hatch <= 2 {
            Some(Self { hatch })
        } else {
            None
        }
    }

    pub fn state_id(self) -> u32 {
        15_102 + self.hatch as u32
    }

    pub fn scheduled_delay(self, boosted: bool, next_int_300: u16) -> u32 {
        let _ = self;
        let base = if boosted { 4_000 } else { 8_000 };
        base + next_int_300.min(299) as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnifferEggDue {
    Crack {
        next: SnifferEgg,
        pitch: f32,
        flags: u16,
    },
    Hatch {
        pitch: f32,
        destroy_with_drops: bool,
        create_baby: bool,
    },
}

pub fn sniffer_egg_due(state: SnifferEgg, next_float: f32) -> SnifferEggDue {
    let pitch = 0.9 + next_float * 0.2;
    if state.hatch < 2 {
        SnifferEggDue::Crack {
            next: SnifferEgg {
                hatch: state.hatch + 1,
            },
            pitch,
            flags: 2,
        }
    } else {
        SnifferEggDue::Hatch {
            pitch,
            destroy_with_drops: false,
            create_baby: true,
        }
    }
}

pub fn sniffer_yaw(next_float: f32) -> f32 {
    let degrees = next_float * 360.0;
    if degrees >= 180.0 {
        degrees - 360.0
    } else {
        degrees
    }
}
