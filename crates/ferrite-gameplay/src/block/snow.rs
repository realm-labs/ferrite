//! Snow-layer and Powder-Snow geometry, support, contact, and freezing rules.

pub const SNOW_BLOCK_STATE_ID: u32 = 6_928;
pub const POWDER_SNOW_STATE_ID: u32 = 27_162;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnowLayer {
    pub layers: u8,
}

impl SnowLayer {
    pub fn new(layers: u8) -> Option<Self> {
        if (1..=8).contains(&layers) {
            Some(Self { layers })
        } else {
            None
        }
    }

    pub fn state_id(self) -> u32 {
        6_918 + self.layers as u32
    }

    pub fn outline_height_sixteenths(self) -> u8 {
        self.layers * 2
    }

    pub fn collision_height_sixteenths(self) -> u8 {
        self.layers * 2 - 2
    }

    pub fn land_pathfindable(self) -> bool {
        self.layers < 5
    }

    pub fn can_stack(self, held_snow: bool, replacing_clicked: bool, clicked_up: bool) -> bool {
        held_snow && self.layers < 8 && (!replacing_clicked || clicked_up)
    }

    pub fn stacked(self) -> Self {
        Self {
            layers: if self.layers < 8 { self.layers + 1 } else { 8 },
        }
    }
}

pub fn snow_layer_survives(
    below_cannot_support: bool,
    below_override_support: bool,
    below_full_up_face: bool,
    below_exact_eight_layer_snow: bool,
) -> bool {
    !below_cannot_support
        && (below_override_support || below_full_up_face || below_exact_eight_layer_snow)
}

pub fn snow_layer_melts(block_light: u8) -> bool {
    block_light > 11
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowderCollision {
    Empty,
    FallPlatform,
    FullCube,
}

pub fn powder_snow_collision(
    has_entity: bool,
    fall_distance: f32,
    falling_block: bool,
    can_walk: bool,
    above_full_shape: bool,
    descending: bool,
) -> PowderCollision {
    if !has_entity {
        PowderCollision::Empty
    } else if fall_distance > 2.5 {
        PowderCollision::FallPlatform
    } else if (falling_block || can_walk) && above_full_shape && !descending {
        PowderCollision::FullCube
    } else {
        PowderCollision::Empty
    }
}

pub fn powder_snow_walkable(type_in_tag: bool, leather_boots: bool) -> bool {
    type_in_tag || leather_boots
}

pub fn frozen_ticks(current: u16, in_powder_snow_and_can_freeze: bool) -> u16 {
    if in_powder_snow_and_can_freeze {
        current.saturating_add(1).min(140)
    } else {
        current.saturating_sub(2)
    }
}

pub fn freeze_damage_due(ticks_frozen: u16, tick_count: u32) -> bool {
    ticks_frozen >= 140 && tick_count.is_multiple_of(40)
}

pub fn freeze_damage(extra_hurt_type: bool) -> u8 {
    if extra_hurt_type { 5 } else { 1 }
}

pub fn powder_snow_fall_sound(fall_distance: f32) -> Option<bool> {
    if fall_distance < 4.0 {
        None
    } else {
        Some(fall_distance >= 7.0)
    }
}
