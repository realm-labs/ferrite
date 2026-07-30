//! Arrow ammunition selection, potion payloads, and egg hatch transactions.

use crate::item::runtime::ply_005::consumables::{PotionContents, PotionEffect};
use crate::item::runtime::stack::ItemStack;

pub const SPECTRAL_GLOW_DURATION: u32 = 200;
pub const TIPPED_DURATION_SCALE: f32 = 0.125;
pub const ARROW_POTION_CONVERSION_AGE: u32 = 600;
pub const EGG_ENTITY_ID: u32 = 39;
pub const EGG_MAXIMUM_STACK: u8 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmmunitionKind {
    Arrow,
    TippedArrow,
    SpectralArrow,
    FireworkRocket,
    Other,
}

pub const fn supported_ammunition(kind: AmmunitionKind, crossbow_held: bool) -> bool {
    matches!(
        kind,
        AmmunitionKind::Arrow | AmmunitionKind::TippedArrow | AmmunitionKind::SpectralArrow
    ) || (crossbow_held && matches!(kind, AmmunitionKind::FireworkRocket))
}

pub fn select_player_ammunition(
    offhand: Option<AmmunitionKind>,
    main_hand: Option<AmmunitionKind>,
    inventory: &[AmmunitionKind],
    crossbow_held: bool,
    infinite_materials: bool,
) -> Option<AmmunitionKind> {
    offhand
        .filter(|kind| supported_ammunition(*kind, crossbow_held))
        .or_else(|| main_hand.filter(|kind| supported_ammunition(*kind, crossbow_held)))
        .or_else(|| {
            inventory
                .iter()
                .copied()
                .find(|kind| supported_ammunition(*kind, false))
        })
        .or(infinite_materials.then_some(AmmunitionKind::Arrow))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmmunitionUse {
    pub projectile_stack: ItemStack,
    pub intangible_projectile: bool,
    pub source_consumed: u32,
}

pub fn take_ammunition(
    source: &mut ItemStack,
    requested_cost: u32,
    extra_projectile: bool,
    infinite_materials: bool,
    ordinary_arrow: bool,
    projectile_identity: u64,
) -> AmmunitionUse {
    let infinity_applies = infinite_materials || (ordinary_arrow && requested_cost == 0);
    if extra_projectile || infinity_applies || requested_cost == 0 {
        let mut projectile_stack = source.copy_with_identity(projectile_identity);
        projectile_stack.count = i32::from(!source.is_empty());
        return AmmunitionUse {
            projectile_stack,
            intangible_projectile: true,
            source_consumed: 0,
        };
    }
    let requested = i32::try_from(requested_cost).unwrap_or(i32::MAX);
    if source.count < requested {
        return AmmunitionUse {
            projectile_stack: ItemStack::empty(),
            intangible_projectile: false,
            source_consumed: 0,
        };
    }
    let projectile_stack = source.split(requested, projectile_identity);
    AmmunitionUse {
        source_consumed: if projectile_stack.is_empty() {
            0
        } else {
            requested_cost
        },
        projectile_stack,
        intangible_projectile: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupMode {
    Allowed,
    CreativeOnly,
    Disallowed,
}

pub const fn pickup_after_owner(current: PickupMode, owner: ProjectileOwner) -> PickupMode {
    match owner {
        ProjectileOwner::Player if matches!(current, PickupMode::Disallowed) => PickupMode::Allowed,
        ProjectileOwner::OminousSpawner => PickupMode::Disallowed,
        ProjectileOwner::Player | ProjectileOwner::Other => current,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileOwner {
    Player,
    OminousSpawner,
    Other,
}

pub const fn can_pick_up(
    mode: PickupMode,
    in_ground_or_no_physics: bool,
    shake_time: u32,
    infinite_materials: bool,
) -> bool {
    in_ground_or_no_physics
        && shake_time == 0
        && match mode {
            PickupMode::Allowed => true,
            PickupMode::CreativeOnly => infinite_materials,
            PickupMode::Disallowed => false,
        }
}

pub fn arrow_hit_effects(contents: &PotionContents) -> Vec<PotionEffect> {
    let scale = f32::from_bits(contents.duration_scale_bits);
    contents
        .base_effects
        .iter()
        .chain(&contents.custom_effects)
        .cloned()
        .map(|mut effect| {
            if effect.duration > 0 && !effect.instantaneous {
                effect.duration = ((effect.duration as f32 * scale).floor() as i32).max(1);
            }
            effect
        })
        .collect()
}

pub const fn converts_to_plain_arrow(contents_empty: bool, grounded_age: u32) -> bool {
    !contents_empty && grounded_age >= ARROW_POTION_CONVERSION_AGE
}

pub const fn arrow_color(contents_empty: bool, explicit_or_computed: Option<i32>) -> i32 {
    if contents_empty {
        -1
    } else if let Some(color) = explicit_or_computed {
        color
    } else {
        -13_083_194
    }
}

pub fn imbued_tipped_contents(center: Option<PotionContents>) -> PotionContents {
    let mut contents = center.unwrap_or(PotionContents {
        base_effects: Vec::new(),
        custom_effects: Vec::new(),
        duration_scale_bits: TIPPED_DURATION_SCALE.to_bits(),
    });
    contents.duration_scale_bits = TIPPED_DURATION_SCALE.to_bits();
    contents
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EggHatchCount {
    None,
    One,
    Four,
}

pub const fn egg_hatch(first_next_int_8: u32, second_next_int_32: Option<u32>) -> EggHatchCount {
    if first_next_int_8 != 0 {
        EggHatchCount::None
    } else if matches!(second_next_int_32, Some(0)) {
        EggHatchCount::Four
    } else {
        EggHatchCount::One
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EggHitOutcome {
    pub requested_chicks: u8,
    pub emitted_entity_event_three: bool,
    pub discarded_projectile: bool,
}

pub const fn egg_hit(first_next_int_8: u32, second_next_int_32: Option<u32>) -> EggHitOutcome {
    let requested_chicks = match egg_hatch(first_next_int_8, second_next_int_32) {
        EggHatchCount::None => 0,
        EggHatchCount::One => 1,
        EggHatchCount::Four => 4,
    };
    EggHitOutcome {
        requested_chicks,
        emitted_entity_event_three: true,
        discarded_projectile: true,
    }
}

pub const fn laying_tick(
    alive: bool,
    adult: bool,
    jockey: bool,
    current_timer: u32,
) -> Option<u32> {
    if !alive || !adult || jockey {
        None
    } else {
        Some(current_timer.saturating_sub(1))
    }
}

pub const fn reset_laying_timer(next_int_6000: u32) -> u32 {
    next_int_6000 + 6_000
}
