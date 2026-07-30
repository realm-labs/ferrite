use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeldBlockHit {
    pub position: BlockPos,
    pub face: Direction,
    pub is_air: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeldAttackContext {
    pub click_succeeded_this_tick: bool,
    pub miss_delay_positive: bool,
    pub using_item: bool,
    pub piercing_weapon: bool,
    pub screen_open: bool,
    pub attack_held: bool,
    pub mouse_captured: bool,
    pub hit: Option<HeldBlockHit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeldAttackDecision {
    SuppressedByClick,
    EarlyReturn,
    Continue(HeldBlockHit),
    Stop,
    AirWithoutStop,
}

#[must_use]
pub const fn decide_held_attack(context: HeldAttackContext) -> HeldAttackDecision {
    if context.click_succeeded_this_tick {
        return HeldAttackDecision::SuppressedByClick;
    }
    if context.miss_delay_positive || context.using_item || context.piercing_weapon {
        return HeldAttackDecision::EarlyReturn;
    }
    if context.screen_open || !context.attack_held || !context.mouse_captured {
        return HeldAttackDecision::Stop;
    }
    match context.hit {
        Some(hit) if !hit.is_air => HeldAttackDecision::Continue(hit),
        Some(_) => HeldAttackDecision::AirWithoutStop,
        None => HeldAttackDecision::Stop,
    }
}
