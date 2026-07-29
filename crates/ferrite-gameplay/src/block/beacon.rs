//! Incremental beacon beam publication, pyramid effects, and menu validation.

pub const BLOCK_STATE_ID: u32 = 9_980;
pub const BLOCK_ENTITY_PROTOCOL_ID: u32 = 15;
pub const SCAN_CELLS_PER_TICK: usize = 10;
pub const REFRESH_INTERVAL: u64 = 80;
pub const FINAL_SECTION_HEIGHT: u32 = 2_048;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeamSection {
    pub color: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamCell {
    Color(u32),
    Transparent,
    Dampening(u8),
    Bedrock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconScan {
    pub cursor_y: i32,
    pub working: Vec<BeamSection>,
    pub published: Vec<BeamSection>,
    pub level: u8,
}

impl BeaconScan {
    pub fn new(min_y: i32) -> Self {
        Self {
            cursor_y: min_y - 1,
            working: Vec::new(),
            published: Vec::new(),
            level: 0,
        }
    }

    pub fn begin(&mut self, beacon_y: i32) {
        self.cursor_y = beacon_y;
        self.working.clear();
        self.working.push(BeamSection {
            color: 0xffff_ffff,
            height: 1,
        });
    }

    pub fn advance<I>(&mut self, cells: I, maximum_y: i32) -> ScanProgress
    where
        I: IntoIterator<Item = BeamCell>,
    {
        let mut visited = 0;
        let mut blocked = false;
        for cell in cells.into_iter().take(SCAN_CELLS_PER_TICK) {
            if self.cursor_y >= maximum_y {
                break;
            }
            self.cursor_y += 1;
            visited += 1;
            match cell {
                BeamCell::Color(color) => append_color(&mut self.working, color),
                BeamCell::Transparent | BeamCell::Bedrock => extend_last(&mut self.working),
                BeamCell::Dampening(value) if value < 15 => extend_last(&mut self.working),
                BeamCell::Dampening(_) => {
                    self.working.clear();
                    self.cursor_y = maximum_y;
                    blocked = true;
                    break;
                }
            }
        }

        let completed = self.cursor_y >= maximum_y;
        if completed {
            self.published = self.working.clone();
        }
        ScanProgress {
            visited,
            completed,
            blocked,
        }
    }
}

fn extend_last(sections: &mut [BeamSection]) {
    if let Some(last) = sections.last_mut() {
        last.height += 1;
    }
}

fn append_color(sections: &mut Vec<BeamSection>, color: u32) {
    if sections.len() <= 1 {
        sections.push(BeamSection { color, height: 1 });
        return;
    }
    if let Some(last) = sections.last_mut() {
        if last.color == color {
            last.height += 1;
            return;
        }
        let averaged = average_argb(last.color, color);
        sections.push(BeamSection {
            color: averaged,
            height: 1,
        });
    }
}

pub const fn average_argb(left: u32, right: u32) -> u32 {
    let a = (((left >> 24) & 0xff) + ((right >> 24) & 0xff)) / 2;
    let r = (((left >> 16) & 0xff) + ((right >> 16) & 0xff)) / 2;
    let g = (((left >> 8) & 0xff) + ((right >> 8) & 0xff)) / 2;
    let b = ((left & 0xff) + (right & 0xff)) / 2;
    (a << 24) | (r << 16) | (g << 8) | b
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanProgress {
    pub visited: usize,
    pub completed: bool,
    pub blocked: bool,
}

pub const fn base_cell_count(level: u8) -> Option<u16> {
    match level {
        1 => Some(9),
        2 => Some(25),
        3 => Some(49),
        4 => Some(81),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectApplication {
    pub radius: u16,
    pub duration: u16,
    pub primary_amplifier: u8,
    pub apply_secondary: bool,
}

pub const fn effect_application(
    level: u8,
    has_primary: bool,
    secondary_equals_primary: bool,
    has_distinct_secondary: bool,
) -> Option<EffectApplication> {
    if level == 0 || !has_primary {
        return None;
    }
    let radius = 10 + 10 * level as u16;
    let duration = 180 + 40 * level as u16;
    Some(EffectApplication {
        radius,
        duration,
        primary_amplifier: if level == 4 && secondary_equals_primary {
            1
        } else {
            0
        },
        apply_secondary: level == 4 && has_distinct_secondary,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaconSelection {
    Rejected,
    Accepted,
    NullSecondaryFault,
}

pub const fn validate_selection(
    level: u8,
    primary_present: bool,
    primary_required_level: u8,
    secondary_present: bool,
    secondary_is_regeneration: bool,
    secondary_equals_primary: bool,
) -> BeaconSelection {
    if !primary_present {
        return if level < 4 && secondary_present {
            BeaconSelection::NullSecondaryFault
        } else {
            BeaconSelection::Rejected
        };
    }
    if primary_required_level > level {
        return BeaconSelection::Rejected;
    }
    if !secondary_present {
        return BeaconSelection::Accepted;
    }
    if level == 4 && (secondary_is_regeneration || secondary_equals_primary) {
        BeaconSelection::Accepted
    } else {
        BeaconSelection::Rejected
    }
}

pub const fn refresh_uses_previous_publication(game_time: u64) -> bool {
    game_time.is_multiple_of(REFRESH_INTERVAL)
}
