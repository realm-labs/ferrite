//! Deterministic block-owned entity-contact and cauldron dispatch rules.

use crate::player::state::Vec3;
use ferrite_foundation::direction::Axis;

pub fn slime_step_velocity(velocity: Vec3, stepping_carefully: bool) -> Vec3 {
    let vertical_speed = velocity.y.abs();
    if stepping_carefully || vertical_speed >= 0.1 {
        return velocity;
    }
    let multiplier = 0.4 + vertical_speed * 0.2;
    Vec3::new(velocity.x * multiplier, velocity.y, velocity.z * multiplier)
}

pub fn moving_slime_velocity(velocity: Vec3, axis: Axis, direction_step: i32) -> Vec3 {
    let step = f64::from(direction_step);
    match axis {
        Axis::X => Vec3::new(step, velocity.y, velocity.z),
        Axis::Y => Vec3::new(velocity.x, step, velocity.z),
        Axis::Z => Vec3::new(velocity.x, velocity.y, step),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HoneySlide {
    Rejected,
    Accepted { velocity: Vec3 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoneySlideInput {
    pub velocity: Vec3,
    pub on_ground: bool,
    pub entity_y: f64,
    pub block_y: i32,
    pub entity_width: f64,
    pub center_offset_x: f64,
    pub center_offset_z: f64,
}

pub fn honey_slide(input: HoneySlideInput) -> HoneySlide {
    const DRAG: f64 = 0.980_000_019_073_486_3;
    const EPSILON: f64 = 1.0e-7;
    let old_y = input.velocity.y / DRAG + 0.08;
    let beside = input.center_offset_x.abs() + EPSILON > 0.4375 + input.entity_width / 2.0
        || input.center_offset_z.abs() + EPSILON > 0.4375 + input.entity_width / 2.0;
    if input.on_ground
        || input.entity_y > f64::from(input.block_y) + 0.9375 - EPSILON
        || old_y >= -0.08
        || !beside
    {
        return HoneySlide::Rejected;
    }
    let horizontal = if old_y < -0.13 { -0.05 / old_y } else { 1.0 };
    HoneySlide::Accepted {
        velocity: Vec3::new(
            input.velocity.x * horizontal,
            (-0.05 - 0.08) * DRAG,
            input.velocity.z * horizontal,
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StickyBlock {
    Slime,
    Honey,
    Ordinary,
}

pub const fn blocks_stick(left: StickyBlock, right: StickyBlock) -> bool {
    if matches!(
        (left, right),
        (StickyBlock::Slime, StickyBlock::Honey) | (StickyBlock::Honey, StickyBlock::Slime)
    ) {
        return false;
    }
    matches!(left, StickyBlock::Slime | StickyBlock::Honey)
        || matches!(right, StickyBlock::Slime | StickyBlock::Honey)
}

pub const fn magma_hurts(is_living: bool, stepping_carefully: bool) -> bool {
    is_living && !stepping_carefully
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronItem {
    EmptyBucket,
    LavaBucket,
    WaterBucket,
    PowderSnowBucket,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CauldronState {
    Lava,
    Empty,
    WaterLevelThree,
    PowderSnowLevelThree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionResult {
    Success,
    Consume,
    TryWithEmptyHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CauldronInteraction {
    pub result: InteractionResult,
    pub replacement: Option<CauldronState>,
    pub mutates_inventory: bool,
}

pub const fn lava_cauldron_interaction(
    item: CauldronItem,
    water_above: bool,
    server_side: bool,
) -> CauldronInteraction {
    let target = match item {
        CauldronItem::EmptyBucket => Some(CauldronState::Empty),
        CauldronItem::LavaBucket if water_above => {
            return CauldronInteraction {
                result: InteractionResult::Consume,
                replacement: None,
                mutates_inventory: false,
            };
        }
        CauldronItem::LavaBucket => Some(CauldronState::Lava),
        CauldronItem::WaterBucket => Some(CauldronState::WaterLevelThree),
        CauldronItem::PowderSnowBucket if water_above => {
            return CauldronInteraction {
                result: InteractionResult::Consume,
                replacement: None,
                mutates_inventory: false,
            };
        }
        CauldronItem::PowderSnowBucket => Some(CauldronState::PowderSnowLevelThree),
        CauldronItem::Other => {
            return CauldronInteraction {
                result: InteractionResult::TryWithEmptyHand,
                replacement: None,
                mutates_inventory: false,
            };
        }
    };
    CauldronInteraction {
        result: InteractionResult::Success,
        replacement: if server_side { target } else { None },
        mutates_inventory: server_side,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsideEffect {
    ClearFreeze,
    LavaIgnite,
    LavaHurt,
}

pub const LAVA_CAULDRON_EFFECTS: [InsideEffect; 3] = [
    InsideEffect::ClearFreeze,
    InsideEffect::LavaIgnite,
    InsideEffect::LavaHurt,
];
