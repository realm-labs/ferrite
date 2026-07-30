//! Static ingredient profiles and identity-owned material joins in the PLY-005 partition.

pub const ARMADILLO_SCUTE_ITEM_ID: u32 = 917;
pub const BLAZE_ROD_ITEM_ID: u32 = 1_145;
pub const BLAZE_POWDER_ITEM_ID: u32 = 1_153;
pub const BONE_ITEM_ID: u32 = 1_112;
pub const PHANTOM_MEMBRANE_ITEM_ID: u32 = 889;
pub const SHULKER_SHELL_ITEM_ID: u32 = 1_334;
pub const POTTERY_SHERD_FIRST_ID: u32 = 1_477;
pub const SMITHING_TEMPLATE_FIRST_ID: u32 = 1_458;
pub const POTTERY_SHERD_COUNT: usize = 23;
pub const SMITHING_TEMPLATE_COUNT: usize = 19;

pub const POTTERY_PATTERNS: [&str; POTTERY_SHERD_COUNT] = [
    "angler",
    "archer",
    "arms_up",
    "blade",
    "brewer",
    "burn",
    "danger",
    "explorer",
    "flow",
    "friend",
    "guster",
    "heart",
    "heartbreak",
    "howl",
    "miner",
    "mourner",
    "plenty",
    "prize",
    "scrape",
    "sheaf",
    "shelter",
    "skull",
    "snort",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

pub const fn pottery_pattern(item_id: u32) -> Option<&'static str> {
    let index = item_id.wrapping_sub(POTTERY_SHERD_FIRST_ID) as usize;
    if index < POTTERY_PATTERNS.len() {
        Some(POTTERY_PATTERNS[index])
    } else {
        None
    }
}

pub const fn smithing_template_rarity(item_id: u32) -> Option<Rarity> {
    if item_id < SMITHING_TEMPLATE_FIRST_ID
        || item_id >= SMITHING_TEMPLATE_FIRST_ID + SMITHING_TEMPLATE_COUNT as u32
    {
        return None;
    }
    Some(match item_id {
        1_463 | 1_464 | 1_465 | 1_469 => Rarity::Rare,
        1_472 => Rarity::Epic,
        _ => Rarity::Uncommon,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateDuplication {
    pub diamonds: u8,
    pub source_templates: u8,
    pub cores: u8,
    pub output_templates: u8,
    pub copies_source_patch: bool,
}

pub const TEMPLATE_DUPLICATION: TemplateDuplication = TemplateDuplication {
    diamonds: 7,
    source_templates: 1,
    cores: 1,
    output_templates: 2,
    copies_source_patch: false,
};

pub const fn armadillo_shed_timer(next_int_6000: u32) -> u32 {
    next_int_6000 + 6_000
}

pub const fn direct_wolf_armor_repair(maximum_damage: u32) -> u32 {
    maximum_damage / 8
}

pub const fn anvil_wolf_armor_repair(maximum_damage: u32) -> u32 {
    maximum_damage / 4
}

pub const fn bone_tames(next_int_3: u32) -> bool {
    next_int_3 == 0
}

pub const fn blaze_rod_fuel_ticks() -> u32 {
    2_400
}

pub const fn blaze_powder_brewing_uses() -> u8 {
    20
}

pub const fn elytra_repair_per_membrane(maximum_damage: u32) -> u32 {
    maximum_damage / 4
}

pub fn shulker_shell_drop_chance(looting_level: u8) -> f32 {
    if looting_level == 0 {
        0.5
    } else {
        (0.5 + 0.0625 * f32::from(looting_level)).min(1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialKeyKind {
    Normal,
    Ominous,
}

pub fn vault_key_matches(
    configured_kind: TrialKeyKind,
    offered_kind: TrialKeyKind,
    configured_patch: u64,
    offered_patch: u64,
    count: u32,
) -> bool {
    configured_kind == offered_kind && configured_patch == offered_patch && count >= 1
}

pub fn advancement_observes_trial_key(
    predicate_kind: TrialKeyKind,
    offered_kind: TrialKeyKind,
) -> bool {
    predicate_kind == offered_kind
}

pub const fn ancient_city_relic_rarity() -> Rarity {
    Rarity::Uncommon
}

pub const fn ingredient_has_direct_use() -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedRecipe {
    pub primary_count: u8,
    pub other_input_count: u8,
    pub output_count: u8,
    pub copies_input_patch: bool,
}

pub const RECOVERY_COMPASS_RECIPE: FixedRecipe = FixedRecipe {
    primary_count: 8,
    other_input_count: 1,
    output_count: 1,
    copies_input_patch: false,
};

pub const MUSIC_DISC_FIVE_RECIPE: FixedRecipe = FixedRecipe {
    primary_count: 9,
    other_input_count: 0,
    output_count: 1,
    copies_input_patch: false,
};

pub const CONDUIT_RECIPE: FixedRecipe = FixedRecipe {
    primary_count: 8,
    other_input_count: 1,
    output_count: 1,
    copies_input_patch: false,
};

pub const BEACON_RECIPE: FixedRecipe = FixedRecipe {
    primary_count: 5,
    other_input_count: 4,
    output_count: 1,
    copies_input_patch: false,
};

pub const fn wither_nether_star_age(mob_drops: bool, entity_created: bool) -> Option<i32> {
    if mob_drops && entity_created {
        Some(-6_000)
    } else {
        None
    }
}

pub const fn nether_star_world_item_hurt(
    base_admitted: bool,
    damage_resistant_to_explosion: bool,
    source_is_explosion: bool,
) -> bool {
    base_admitted && !(damage_resistant_to_explosion && source_is_explosion)
}
