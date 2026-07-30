//! Happy Ghast harness equipment, shearing, passengers, and ridden input.

use crate::item::runtime::stack::ItemStack;

pub const HAPPY_GHAST_ENTITY_ID: u32 = 58;
pub const MAX_DIRECT_PASSENGERS: usize = 4;
pub const LOAD_TIMEOUT_GRACE_TICKS: u32 = 60;
pub const STILL_TIMEOUT_MAXIMUM: i32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarnessAdmission {
    pub server_side: bool,
    pub target_alive: bool,
    pub target_adult: bool,
    pub allowed_by_live_tag: bool,
    pub body_slot_empty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipResult {
    Pass,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipOutcome {
    pub result: EquipResult,
    pub equipped: ItemStack,
    pub consumed: u8,
    pub guaranteed_drop: bool,
    pub persistence_required: bool,
    pub equip_sound_seed_requested: bool,
    pub equip_event: bool,
    pub item_used_stat: bool,
}

pub fn equip_harness(
    held: &mut ItemStack,
    admission: HarnessAdmission,
    dispenser: bool,
) -> EquipOutcome {
    if !admission.target_alive
        || !admission.target_adult
        || !admission.allowed_by_live_tag
        || !admission.body_slot_empty
    {
        return EquipOutcome {
            result: EquipResult::Pass,
            equipped: ItemStack::empty(),
            consumed: 0,
            guaranteed_drop: false,
            persistence_required: false,
            equip_sound_seed_requested: false,
            equip_event: false,
            item_used_stat: false,
        };
    }
    let equipped = if admission.server_side {
        held.split(1, held.identity)
    } else {
        ItemStack::empty()
    };
    EquipOutcome {
        result: EquipResult::Success,
        equipped,
        consumed: u8::from(admission.server_side),
        guaranteed_drop: admission.server_side,
        persistence_required: admission.server_side && dispenser,
        equip_sound_seed_requested: admission.server_side,
        equip_event: admission.server_side,
        item_used_stat: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispenserCandidate {
    pub living: bool,
    pub alive: bool,
    pub spectator: bool,
    pub allowed_by_live_tag: bool,
    pub slot_admitting: bool,
    pub body_slot_empty: bool,
}

pub fn first_dispenser_candidate(candidates: &[DispenserCandidate]) -> Option<usize> {
    candidates.iter().position(|candidate| {
        candidate.living
            && candidate.alive
            && !candidate.spectator
            && candidate.allowed_by_live_tag
            && candidate.slot_admitting
            && candidate.body_slot_empty
    })
}

pub const fn valid_body_equipment(
    stack_present: bool,
    adult: bool,
    alive: bool,
    allowed_by_live_tag: bool,
) -> bool {
    stack_present && adult && alive && allowed_by_live_tag
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShearInput {
    pub leashed: bool,
    pub secondary_use: bool,
    pub passengers: usize,
    pub body_harness: bool,
    pub prevent_armor_change: bool,
    pub creative: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShearStep {
    CutLeash,
    DamageShears,
    ClearBody,
    UnequipEvent,
    ShearEvent,
    SpawnEquipment,
    PlayerShearedCriterion,
    UnequipSound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShearOutcome {
    pub consumed: bool,
    pub recovered_harness: bool,
    pub steps: Vec<ShearStep>,
}

pub fn shear_happy_ghast(input: ShearInput) -> ShearOutcome {
    if input.leashed {
        return ShearOutcome {
            consumed: true,
            recovered_harness: false,
            steps: vec![ShearStep::CutLeash, ShearStep::DamageShears],
        };
    }
    if input.secondary_use
        || input.passengers != 0
        || !input.body_harness
        || (input.prevent_armor_change && !input.creative)
    {
        return ShearOutcome {
            consumed: false,
            recovered_harness: false,
            steps: Vec::new(),
        };
    }
    ShearOutcome {
        consumed: true,
        recovered_harness: true,
        steps: vec![
            ShearStep::DamageShears,
            ShearStep::ClearBody,
            ShearStep::UnequipEvent,
            ShearStep::ShearEvent,
            ShearStep::SpawnEquipment,
            ShearStep::PlayerShearedCriterion,
            ShearStep::UnequipSound,
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemptationSet {
    FoodOnly,
    FoodAndHarnesses,
}

pub const fn temptation_set(baby: bool, valid_harness: bool) -> TemptationSet {
    if baby || valid_harness {
        TemptationSet::FoodOnly
    } else {
        TemptationSet::FoodAndHarnesses
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HappyGhastInteraction {
    Generic,
    Equip,
    Mount,
}

pub const fn interact_happy_ghast(
    baby: bool,
    equip_consumed: bool,
    valid_harness: bool,
    secondary_use: bool,
) -> HappyGhastInteraction {
    if baby {
        HappyGhastInteraction::Generic
    } else if equip_consumed {
        HappyGhastInteraction::Equip
    } else if valid_harness && !secondary_use {
        HappyGhastInteraction::Mount
    } else {
        HappyGhastInteraction::Generic
    }
}

pub const fn can_add_passenger(current: usize) -> bool {
    current < MAX_DIRECT_PASSENGERS
}

pub const fn has_player_controller(
    valid_harness: bool,
    still_timeout: i32,
    first_passenger_is_player: bool,
) -> bool {
    valid_harness && still_timeout <= 0 && first_passenger_is_player
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiddenInput {
    pub strafe: f32,
    pub forward: f32,
    pub jumping: bool,
    pub pitch_degrees: f32,
    pub flying_speed: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub fn ridden_input(input: RiddenInput) -> MovementVector {
    let mut forward = 0.0_f32;
    let mut up = 0.0_f32;
    if input.forward != 0.0 {
        let radians = input.pitch_degrees * (std::f32::consts::PI / 180.0);
        let mut forward_look = radians.cos();
        let mut up_look = -radians.sin();
        if input.forward < 0.0 {
            forward_look *= -0.5;
            up_look *= -0.5;
        }
        forward = forward_look;
        up = up_look;
    }
    if input.jumping {
        up += 0.5;
    }
    let scale = f64::from(3.9_f32) * input.flying_speed;
    MovementVector {
        x: f64::from(input.strafe) * scale,
        y: f64::from(up) * scale,
        z: f64::from(forward) * scale,
    }
}

pub const fn travel_speed(flying_speed: f32) -> f32 {
    flying_speed * 5.0 / 3.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiddenRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub body_yaw: f32,
    pub head_yaw: f32,
    pub previous_yaw: f32,
}

pub fn ridden_rotation(current_yaw: f32, player_yaw: f32, player_pitch: f32) -> RiddenRotation {
    let difference = wrap_degrees(player_yaw - current_yaw);
    let yaw = current_yaw + difference * 0.08;
    RiddenRotation {
        yaw,
        pitch: player_pitch * 0.5,
        body_yaw: yaw,
        head_yaw: yaw,
        previous_yaw: yaw,
    }
}

fn wrap_degrees(value: f32) -> f32 {
    let mut wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped -= 360.0;
    }
    if wrapped < -180.0 {
        wrapped += 360.0;
    }
    wrapped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StillTimeout {
    pub tick_count: u32,
    pub remaining: i32,
}

impl StillTimeout {
    pub fn tick(&mut self, player_above: bool) {
        self.tick_count = self.tick_count.saturating_add(1);
        if self.remaining > 0 && self.tick_count > LOAD_TIMEOUT_GRACE_TICKS {
            self.remaining -= 1;
        }
        if player_above {
            self.remaining = STILL_TIMEOUT_MAXIMUM;
        }
    }

    pub fn passenger_added(&mut self, player_above: bool) {
        if !player_above {
            self.remaining = 0;
        } else if self.remaining > STILL_TIMEOUT_MAXIMUM {
            self.remaining = STILL_TIMEOUT_MAXIMUM;
        }
    }

    pub fn passenger_removed(&mut self) {
        self.remaining = STILL_TIMEOUT_MAXIMUM;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarnessRecipe {
    pub target_color: &'static str,
    pub base_leather: u8,
    pub base_glass: u8,
    pub base_wool: u8,
    pub recolor_source_count: u8,
    pub excludes_same_color: bool,
    pub copies_source_components: bool,
}

pub const fn harness_recipe(target_color: &'static str) -> HarnessRecipe {
    HarnessRecipe {
        target_color,
        base_leather: 3,
        base_glass: 2,
        base_wool: 1,
        recolor_source_count: 15,
        excludes_same_color: true,
        copies_source_components: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarnessComponentProfile {
    pub body_slot: bool,
    pub dispensable: bool,
    pub swappable: bool,
    pub damage_on_hurt: bool,
    pub equip_on_interact: bool,
    pub can_be_sheared: bool,
    pub has_durability: bool,
    pub has_defense: bool,
}

pub const HARNESS_COMPONENTS: HarnessComponentProfile = HarnessComponentProfile {
    body_slot: true,
    dispensable: true,
    swappable: true,
    damage_on_hurt: true,
    equip_on_interact: true,
    can_be_sheared: true,
    has_durability: false,
    has_defense: false,
};
