//! Bubble-column, Frogspawn, and Lily-Pad deterministic boundaries.

pub const FROGSPAWN_STATE_ID: u32 = 32_084;
pub const LILY_PAD_STATE_ID: u32 = 8_920;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubbleFlow {
    Down,
    Up,
}

impl BubbleFlow {
    pub fn state_id(self) -> u32 {
        match self {
            Self::Down => 15_294,
            Self::Up => 15_295,
        }
    }
}

pub fn bubble_occupiable(existing_column: bool, full_source_water_liquid_block: bool) -> bool {
    existing_column || full_source_water_liquid_block
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BubbleUpdate {
    Column(BubbleFlow),
    Water,
    Preserve,
}

pub fn bubble_from_below(
    below_column: Option<BubbleFlow>,
    below_pushes_up: bool,
    below_drags_down: bool,
    old_was_column: bool,
) -> BubbleUpdate {
    if let Some(flow) = below_column {
        BubbleUpdate::Column(flow)
    } else if below_pushes_up {
        BubbleUpdate::Column(BubbleFlow::Up)
    } else if below_drags_down {
        BubbleUpdate::Column(BubbleFlow::Down)
    } else if old_was_column {
        BubbleUpdate::Water
    } else {
        BubbleUpdate::Preserve
    }
}

pub fn bubble_velocity(flow: BubbleFlow, current_y: f64, inside: bool, open_surface: bool) -> f64 {
    match (flow, inside, open_surface) {
        (BubbleFlow::Down, false, true) => (current_y - 0.03).max(-0.9),
        (BubbleFlow::Up, false, true) => (current_y + 0.1).min(1.8),
        (BubbleFlow::Down, true, _) => (current_y - 0.03).max(-0.3),
        (BubbleFlow::Up, true, _) => (current_y + 0.06).min(0.7),
        _ => current_y,
    }
}

pub fn bubble_boat_launch(flow: BubbleFlow, has_player_passenger: bool) -> f64 {
    match flow {
        BubbleFlow::Down => -0.7,
        BubbleFlow::Up if has_player_passenger => 2.7,
        BubbleFlow::Up => 0.6,
    }
}

pub fn bubble_refilled_air(current_air: i32) -> i32 {
    current_air + 4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrogspawnTick {
    DestroyUnsupported,
    Hatch { tadpoles: u8 },
}

pub fn frogspawn_interval(next_int_range: u32, minimum: u32, maximum: u32) -> u32 {
    if maximum <= minimum {
        minimum
    } else {
        minimum + next_int_range.min(maximum - minimum)
    }
}

pub fn frogspawn_due(survives: bool, tadpole_draw_2_to_5: u8) -> FrogspawnTick {
    if survives {
        FrogspawnTick::Hatch {
            tadpoles: tadpole_draw_2_to_5.clamp(2, 5),
        }
    } else {
        FrogspawnTick::DestroyUnsupported
    }
}

pub fn tadpole_horizontal_offset(next_double: f64) -> f64 {
    next_double.clamp(0.2, 0.8)
}

pub fn tadpole_yaw(next_int_1_to_360: u16) -> u16 {
    next_int_1_to_360.clamp(1, 360)
}

pub fn lily_pad_survives(
    below_source_water_or_supported_ice: bool,
    fluid_at_position_empty: bool,
) -> bool {
    below_source_water_or_supported_ice && fluid_at_position_empty
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LilyPadContact {
    pub destroy_with_drops: bool,
    pub breaking_entity_is_boat: bool,
}

pub fn lily_pad_boat_contact(server_side: bool, is_boat: bool) -> Option<LilyPadContact> {
    if server_side && is_boat {
        Some(LilyPadContact {
            destroy_with_drops: true,
            breaking_entity_is_boat: true,
        })
    } else {
        None
    }
}
