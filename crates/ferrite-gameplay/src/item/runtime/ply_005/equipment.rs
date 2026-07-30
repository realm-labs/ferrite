//! Nautilus armor, spear combat, and food-on-a-stick equipment joins.

pub const SPEAR_USE_DURATION: u32 = 72_000;
pub const SPEAR_CONTACT_COOLDOWN: u32 = 10;
pub const SPEAR_DAMAGE_SPEED: f32 = 4.6;
pub const SPEAR_KNOCKBACK_SPEED: f32 = 5.1;
pub const SPEAR_BASE_KNOCKBACK: f32 = 0.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NautilusArmorTier {
    Copper,
    Iron,
    Golden,
    Diamond,
    Netherite,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NautilusArmorProfile {
    pub item_id: u32,
    pub armor: f32,
    pub toughness: f32,
    pub knockback_resistance: f32,
    pub damageable: bool,
    pub damage_on_hurt: bool,
}

pub const fn nautilus_armor_profile(tier: NautilusArmorTier) -> NautilusArmorProfile {
    let (item_id, armor, toughness, knockback_resistance) = match tier {
        NautilusArmorTier::Copper => (1_368, 4.0, 0.0, 0.0),
        NautilusArmorTier::Iron => (1_364, 5.0, 0.0, 0.0),
        NautilusArmorTier::Golden => (1_365, 7.0, 0.0, 0.0),
        NautilusArmorTier::Diamond => (1_366, 11.0, 2.0, 0.0),
        NautilusArmorTier::Netherite => (1_367, 19.0, 3.0, 0.1),
    };
    NautilusArmorProfile {
        item_id,
        armor,
        toughness,
        knockback_resistance,
        damageable: false,
        damage_on_hurt: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NautilusEquipInput {
    pub alive: bool,
    pub adult: bool,
    pub tamed: bool,
    pub allowed_by_live_tag: bool,
    pub body_empty: bool,
    pub secondary_use: bool,
    pub server_side: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NautilusEquipOutcome {
    pub persistence_marked_before_admission: bool,
    pub menu_opened: bool,
    pub equipped: bool,
    pub consumed: u8,
    pub guaranteed_drop: bool,
    pub item_used_stat: bool,
}

pub fn equip_nautilus_armor(input: NautilusEquipInput) -> NautilusEquipOutcome {
    let mount_gate = input.alive && input.adult && input.tamed;
    let menu_opened = mount_gate && input.secondary_use;
    let equipped = mount_gate
        && !input.secondary_use
        && input.allowed_by_live_tag
        && input.body_empty
        && input.server_side;
    NautilusEquipOutcome {
        persistence_marked_before_admission: true,
        menu_opened,
        equipped,
        consumed: u8::from(equipped),
        guaranteed_drop: equipped,
        item_used_stat: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShearedNautilusSlot {
    Body,
    Saddle,
    None,
}

pub const fn first_nautilus_shear_slot(
    body_present: bool,
    saddle_present: bool,
    passengers: usize,
    secondary_use: bool,
    prevent_armor_change: bool,
    creative: bool,
) -> ShearedNautilusSlot {
    if passengers != 0 || secondary_use || (prevent_armor_change && !creative) {
        ShearedNautilusSlot::None
    } else if body_present {
        ShearedNautilusSlot::Body
    } else if saddle_present {
        ShearedNautilusSlot::Saddle
    } else {
        ShearedNautilusSlot::None
    }
}

pub const fn zombie_nautilus_sun_protected(body_nonempty: bool) -> bool {
    body_nonempty
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpearTier {
    Wood,
    Stone,
    Copper,
    Iron,
    Gold,
    Diamond,
    Netherite,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpearProfile {
    pub item_id: u32,
    pub durability: u32,
    pub enchantability: u8,
    pub attack_modifier: f32,
    pub stab_ticks: u8,
    pub held_delay: u8,
    pub kinetic_multiplier: f32,
    pub damage_through_tick: u16,
    pub knockback_through_tick: u16,
    pub dismount_through_tick: u16,
    pub dismount_speed: f32,
}

pub const fn spear_profile(tier: SpearTier) -> SpearProfile {
    match tier {
        SpearTier::Wood => spear((1_326, 59, 15, 0.0), (13, 15, 0.7), (300, 200, 100, 14.0)),
        SpearTier::Stone => spear((1_327, 131, 5, 1.0), (15, 14, 0.82), (275, 180, 90, 13.0)),
        SpearTier::Copper => spear((1_328, 190, 13, 1.0), (17, 13, 0.82), (250, 165, 80, 12.0)),
        SpearTier::Iron => spear((1_329, 250, 14, 2.0), (19, 12, 0.95), (225, 135, 50, 11.0)),
        SpearTier::Gold => spear((1_330, 32, 22, 0.0), (19, 14, 0.7), (275, 170, 70, 13.0)),
        SpearTier::Diamond => spear(
            (1_331, 1_561, 10, 3.0),
            (21, 10, 1.075),
            (200, 130, 60, 10.0),
        ),
        SpearTier::Netherite => spear((1_332, 2_031, 15, 4.0), (23, 8, 1.2), (175, 110, 50, 9.0)),
    }
}

const fn spear(
    identity: (u32, u32, u8, f32),
    timing: (u8, u8, f32),
    windows: (u16, u16, u16, f32),
) -> SpearProfile {
    SpearProfile {
        item_id: identity.0,
        durability: identity.1,
        enchantability: identity.2,
        attack_modifier: identity.3,
        stab_ticks: timing.0,
        held_delay: timing.1,
        kinetic_multiplier: timing.2,
        damage_through_tick: windows.0,
        knockback_through_tick: windows.1,
        dismount_through_tick: windows.2,
        dismount_speed: windows.3,
    }
}

pub fn stab_charge_admitted(attack_strength_ticks: u32, attack_strength_delay: f32) -> bool {
    (attack_strength_ticks as f32 + 5.0) / attack_strength_delay >= 1.0
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KineticContact {
    pub damage: bool,
    pub knockback: bool,
    pub dismount: bool,
    pub damage_amount: f32,
}

pub fn kinetic_contact(
    tier: SpearTier,
    elapsed: u16,
    attacker_speed: f32,
    relative_speed: f32,
    attack_damage: f32,
    player_attacker: bool,
) -> KineticContact {
    let profile = spear_profile(tier);
    let factor = if player_attacker { 1.0 } else { 0.2 };
    let damage =
        elapsed <= profile.damage_through_tick && relative_speed >= SPEAR_DAMAGE_SPEED * factor;
    let knockback = elapsed <= profile.knockback_through_tick
        && attacker_speed >= SPEAR_KNOCKBACK_SPEED * factor;
    let dismount = elapsed <= profile.dismount_through_tick
        && attacker_speed >= profile.dismount_speed * factor;
    KineticContact {
        damage,
        knockback,
        dismount,
        damage_amount: if damage {
            attack_damage + (relative_speed * profile.kinetic_multiplier).floor()
        } else {
            0.0
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LungeOutcome {
    pub admitted: bool,
    pub durability_damage: u8,
    pub exhaustion: f32,
    pub horizontal_impulse: f32,
}

pub fn lunge(
    level: u8,
    mounted: bool,
    fall_flying: bool,
    in_water: bool,
    player: bool,
    creative: bool,
    food_level: u8,
) -> LungeOutcome {
    let admitted = (1..=3).contains(&level)
        && !mounted
        && !fall_flying
        && !in_water
        && (!player || creative || food_level >= 7);
    LungeOutcome {
        admitted,
        durability_damage: u8::from(admitted),
        exhaustion: if admitted {
            [0.0, 4.0, 8.0, 12.0][usize::from(level)]
        } else {
            0.0
        },
        horizontal_impulse: if admitted {
            [0.0, 0.458, 0.916, 1.374][usize::from(level)]
        } else {
            0.0
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteeringKind {
    Carrot,
    WarpedFungus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SteeringProfile {
    pub item_id: u32,
    pub maximum_damage: u16,
    pub boost_damage: u16,
}

pub const fn steering_profile(kind: SteeringKind) -> SteeringProfile {
    match kind {
        SteeringKind::Carrot => SteeringProfile {
            item_id: 887,
            maximum_damage: 25,
            boost_damage: 7,
        },
        SteeringKind::WarpedFungus => SteeringProfile {
            item_id: 888,
            maximum_damage: 100,
            boost_damage: 1,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SteeringUseOutcome {
    pub success: bool,
    pub item_used_stat: bool,
    pub boost_total: Option<u16>,
    pub processed_damage: u16,
    pub broken_to_fishing_rod: bool,
    pub preserved_component_patch: bool,
    pub replacement_damage: u16,
}

pub const fn use_steering_stick(
    kind: SteeringKind,
    client_side: bool,
    admission: bool,
    already_boosting: bool,
    next_int_841: u16,
    current_damage: u16,
    processed_damage: u16,
) -> SteeringUseOutcome {
    if client_side {
        return rejected_steering(false);
    }
    if !admission || already_boosting {
        return rejected_steering(true);
    }
    let profile = steering_profile(kind);
    let broken = current_damage.saturating_add(processed_damage) >= profile.maximum_damage;
    SteeringUseOutcome {
        success: true,
        item_used_stat: false,
        boost_total: Some(next_int_841 + 140),
        processed_damage,
        broken_to_fishing_rod: broken,
        preserved_component_patch: broken,
        replacement_damage: 0,
    }
}

pub fn boost_multiplier(elapsed: u16, total: u16) -> f32 {
    1.0 + 1.15 * (std::f32::consts::PI * f32::from(elapsed) / f32::from(total)).sin()
}

const fn rejected_steering(item_used_stat: bool) -> SteeringUseOutcome {
    SteeringUseOutcome {
        success: false,
        item_used_stat,
        boost_total: None,
        processed_damage: 0,
        broken_to_fishing_rod: false,
        preserved_component_patch: false,
        replacement_damage: 0,
    }
}
