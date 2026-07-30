//! Exact material-source loot arithmetic and source-block profiles.

use crate::item::runtime::sim_004::catalog::MaterialItem;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolTier {
    None,
    Stone,
    Iron,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OreVariant {
    pub block_id: u32,
    pub item_id: u32,
    pub first_state_id: u32,
    pub last_state_id: u32,
    pub hardness: f32,
    pub resistance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OreProfile {
    pub material: MaterialItem,
    pub variants: &'static [OreVariant],
    pub minimum_tier: ToolTier,
    pub base_count_minimum: u8,
    pub base_count_maximum: u8,
    pub experience_minimum: u8,
    pub experience_maximum: u8,
    pub furnace_experience: f32,
    pub configured_size: u8,
}

const DIAMOND_ORES: [OreVariant; 2] = [
    ore(203, 105, 5_307, 5_307, 3.0, 3.0),
    ore(204, 106, 5_308, 5_308, 4.5, 3.0),
];
const EMERALD_ORES: [OreVariant; 2] = [
    ore(398, 101, 9_573, 9_573, 3.0, 3.0),
    ore(399, 102, 9_574, 9_574, 4.5, 3.0),
];
const LAPIS_ORES: [OreVariant; 2] = [
    ore(102, 103, 563, 563, 3.0, 3.0),
    ore(103, 104, 564, 564, 4.5, 3.0),
];
const QUARTZ_ORES: [OreVariant; 1] = [ore(476, 108, 11_312, 11_312, 3.0, 3.0)];
const REDSTONE_ORES: [OreVariant; 2] = [
    ore(271, 99, 6_881, 6_882, 3.0, 3.0),
    ore(272, 100, 6_883, 6_884, 4.5, 3.0),
];

const fn ore(
    block_id: u32,
    item_id: u32,
    first_state_id: u32,
    last_state_id: u32,
    hardness: f32,
    resistance: f32,
) -> OreVariant {
    OreVariant {
        block_id,
        item_id,
        first_state_id,
        last_state_id,
        hardness,
        resistance,
    }
}

pub const fn ore_profile(material: MaterialItem) -> Option<OreProfile> {
    match material {
        MaterialItem::Diamond => Some(OreProfile {
            material,
            variants: &DIAMOND_ORES,
            minimum_tier: ToolTier::Iron,
            base_count_minimum: 1,
            base_count_maximum: 1,
            experience_minimum: 3,
            experience_maximum: 7,
            furnace_experience: 1.0,
            configured_size: 4,
        }),
        MaterialItem::Emerald => Some(OreProfile {
            material,
            variants: &EMERALD_ORES,
            minimum_tier: ToolTier::Iron,
            base_count_minimum: 1,
            base_count_maximum: 1,
            experience_minimum: 3,
            experience_maximum: 7,
            furnace_experience: 1.0,
            configured_size: 3,
        }),
        MaterialItem::LapisLazuli => Some(OreProfile {
            material,
            variants: &LAPIS_ORES,
            minimum_tier: ToolTier::Stone,
            base_count_minimum: 4,
            base_count_maximum: 9,
            experience_minimum: 2,
            experience_maximum: 5,
            furnace_experience: 0.2,
            configured_size: 7,
        }),
        MaterialItem::Quartz => Some(OreProfile {
            material,
            variants: &QUARTZ_ORES,
            minimum_tier: ToolTier::None,
            base_count_minimum: 1,
            base_count_maximum: 1,
            experience_minimum: 2,
            experience_maximum: 5,
            furnace_experience: 0.2,
            configured_size: 14,
        }),
        MaterialItem::Redstone => Some(OreProfile {
            material,
            variants: &REDSTONE_ORES,
            minimum_tier: ToolTier::Iron,
            base_count_minimum: 4,
            base_count_maximum: 5,
            experience_minimum: 1,
            experience_maximum: 5,
            furnace_experience: 0.7,
            configured_size: 8,
        }),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OreBreakInput {
    pub material: MaterialItem,
    pub variant_index: usize,
    pub correct_tool: bool,
    pub silk_touch: bool,
    pub base_count: u8,
    pub fortune_level: u8,
    pub fortune_draw: u8,
    pub explosion_survivors: Option<u8>,
    pub experience_draw: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OreOutput {
    None,
    OreBlock { item_id: u32 },
    Material { item: MaterialItem, count: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OreBreak {
    pub output: OreOutput,
    pub experience: u8,
}

pub fn break_ore(input: OreBreakInput) -> Result<OreBreak, LootInputError> {
    let profile = ore_profile(input.material).ok_or(LootInputError::NotAnOre(input.material))?;
    let variant = profile
        .variants
        .get(input.variant_index)
        .ok_or(LootInputError::Variant(input.variant_index))?;
    if !input.correct_tool {
        return Ok(OreBreak {
            output: OreOutput::None,
            experience: 0,
        });
    }
    if input.silk_touch {
        return Ok(OreBreak {
            output: OreOutput::OreBlock {
                item_id: variant.item_id,
            },
            experience: 0,
        });
    }
    if !(profile.base_count_minimum..=profile.base_count_maximum).contains(&input.base_count) {
        return Err(LootInputError::BaseCount(input.base_count));
    }
    if !(profile.experience_minimum..=profile.experience_maximum).contains(&input.experience_draw) {
        return Err(LootInputError::Experience(input.experience_draw));
    }

    let before_explosion = if input.material == MaterialItem::Redstone {
        if input.fortune_draw > input.fortune_level {
            return Err(LootInputError::FortuneDraw(input.fortune_draw));
        }
        input.base_count.saturating_add(input.fortune_draw)
    } else {
        let maximum_draw = input.fortune_level.saturating_add(1);
        if input.fortune_draw > maximum_draw {
            return Err(LootInputError::FortuneDraw(input.fortune_draw));
        }
        input.base_count.saturating_mul(input.fortune_draw.max(1))
    };
    let count = apply_explosion_decay(before_explosion, input.explosion_survivors)?;
    Ok(OreBreak {
        output: if count == 0 {
            OreOutput::None
        } else {
            OreOutput::Material {
                item: input.material,
                count,
            }
        },
        experience: input.experience_draw,
    })
}

fn apply_explosion_decay(count: u8, survivors: Option<u8>) -> Result<u8, LootInputError> {
    match survivors {
        Some(survivors) if survivors <= count => Ok(survivors),
        Some(survivors) => Err(LootInputError::ExplosionSurvivors { count, survivors }),
        None => Ok(count),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedstoneOreContact {
    pub spawn_exposed_face_particles: bool,
    pub write_lit_state: bool,
    pub write_flags: u8,
}

pub const fn contact_redstone_ore(server_side: bool, currently_lit: bool) -> RedstoneOreContact {
    RedstoneOreContact {
        spawn_exposed_face_particles: true,
        write_lit_state: server_side && !currently_lit,
        write_flags: if server_side && !currently_lit { 3 } else { 0 },
    }
}

pub const fn redstone_random_tick(_currently_lit: bool) -> bool {
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GravelDrop {
    Gravel,
    Flint,
    Nothing,
}

pub fn gravel_drop(
    silk_touch: bool,
    explosion_survived: bool,
    fortune_level: u8,
    probability_draw: f32,
) -> Result<GravelDrop, LootInputError> {
    if silk_touch {
        return Ok(GravelDrop::Gravel);
    }
    if !explosion_survived {
        return Ok(GravelDrop::Nothing);
    }
    validate_unit_draw(probability_draw)?;
    let chance = match fortune_level {
        0 => 0.1,
        1 => 0.142_857_15,
        2 => 0.25,
        _ => 1.0,
    };
    Ok(if probability_draw < chance {
        GravelDrop::Flint
    } else {
        GravelDrop::Gravel
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlowstoneDrop {
    Glowstone,
    Dust(u8),
}

pub fn glowstone_drop(
    silk_touch: bool,
    base_count: u8,
    fortune_level: u8,
    fortune_draw: u8,
    explosion_survivors: Option<u8>,
) -> Result<GlowstoneDrop, LootInputError> {
    if silk_touch {
        return Ok(GlowstoneDrop::Glowstone);
    }
    if !(2..=4).contains(&base_count) {
        return Err(LootInputError::BaseCount(base_count));
    }
    if fortune_draw > fortune_level {
        return Err(LootInputError::FortuneDraw(fortune_draw));
    }
    let count = base_count.saturating_add(fortune_draw).clamp(1, 4);
    Ok(GlowstoneDrop::Dust(apply_explosion_decay(
        count,
        explosion_survivors,
    )?))
}

pub fn looting_count(
    base_count: u8,
    living_attacker: bool,
    looting_level: u8,
    uniform_draw: f32,
) -> Result<u8, LootInputError> {
    if !living_attacker || looting_level == 0 {
        return Ok(base_count);
    }
    validate_unit_draw(uniform_draw)?;
    let bonus = (f32::from(looting_level) * uniform_draw).round() as u8;
    Ok(base_count.saturating_add(bonus))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BirdKind {
    Chicken,
    Parrot,
}

pub fn bird_feather_drop(
    kind: BirdKind,
    adult: bool,
    base_count: u8,
    living_attacker: bool,
    looting_level: u8,
    uniform_draw: f32,
) -> Result<u8, LootInputError> {
    if kind == BirdKind::Chicken && !adult {
        return Ok(0);
    }
    let range = match kind {
        BirdKind::Chicken => 0..=2,
        BirdKind::Parrot => 1..=2,
    };
    if !range.contains(&base_count) {
        return Err(LootInputError::BaseCount(base_count));
    }
    looting_count(base_count, living_attacker, looting_level, uniform_draw)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeatherSource {
    Cow,
    Donkey,
    Horse,
    Llama,
    Mooshroom,
    Mule,
    TraderLlama,
    Hoglin,
}

pub fn leather_drop(
    source: LeatherSource,
    base_count: u8,
    living_attacker: bool,
    looting_level: u8,
    uniform_draw: f32,
) -> Result<u8, LootInputError> {
    let maximum = if source == LeatherSource::Hoglin {
        1
    } else {
        2
    };
    if base_count > maximum {
        return Err(LootInputError::BaseCount(base_count));
    }
    looting_count(base_count, living_attacker, looting_level, uniform_draw)
}

pub fn slime_ball_drop(
    slime_size: u8,
    source_is_frog: bool,
    base_count: u8,
    living_attacker: bool,
    looting_level: u8,
    uniform_draw: f32,
) -> Result<u8, LootInputError> {
    if slime_size != 1 {
        return Ok(0);
    }
    if source_is_frog {
        return Ok(1);
    }
    if base_count > 2 {
        return Err(LootInputError::BaseCount(base_count));
    }
    looting_count(base_count, living_attacker, looting_level, uniform_draw)
}

pub const fn panda_sneeze_emits_slime_ball(draw: u16) -> Option<bool> {
    if draw < 700 { Some(draw == 0) } else { None }
}

pub fn leaf_stick_drop(
    shears: bool,
    silk_touch: bool,
    fortune_level: u8,
    probability_draw: f32,
    count_draw: u8,
    explosion_survivors: Option<u8>,
) -> Result<u8, LootInputError> {
    if shears || silk_touch {
        return Ok(0);
    }
    validate_unit_draw(probability_draw)?;
    let chance = match fortune_level {
        0 => 0.02,
        1 => 0.022_222_223,
        2 => 0.025,
        3 => 0.033_333_335,
        _ => 0.1,
    };
    if probability_draw >= chance {
        return Ok(0);
    }
    if !(1..=2).contains(&count_draw) {
        return Err(LootInputError::BaseCount(count_draw));
    }
    apply_explosion_decay(count_draw, explosion_survivors)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadBushDrop {
    DeadBush,
    Sticks(u8),
}

pub fn dead_bush_drop(
    shears: bool,
    count_draw: u8,
    explosion_survivors: Option<u8>,
) -> Result<DeadBushDrop, LootInputError> {
    if shears {
        return Ok(DeadBushDrop::DeadBush);
    }
    if count_draw > 2 {
        return Err(LootInputError::BaseCount(count_draw));
    }
    Ok(DeadBushDrop::Sticks(apply_explosion_decay(
        count_draw,
        explosion_survivors,
    )?))
}

fn validate_unit_draw(draw: f32) -> Result<(), LootInputError> {
    if draw.is_finite() && (0.0..1.0).contains(&draw) {
        Ok(())
    } else {
        Err(LootInputError::ProbabilityDraw(draw.to_bits()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LootInputError {
    #[error("{0:?} is not an ore material")]
    NotAnOre(MaterialItem),
    #[error("ore variant index {0} is out of bounds")]
    Variant(usize),
    #[error("base count draw {0} is outside the audited range")]
    BaseCount(u8),
    #[error("fortune draw {0} is outside the audited range")]
    FortuneDraw(u8),
    #[error("experience draw {0} is outside the audited range")]
    Experience(u8),
    #[error("{survivors} explosion survivors exceed pre-decay count {count}")]
    ExplosionSurvivors { count: u8, survivors: u8 },
    #[error("probability draw bits {0:#010x} are outside [0, 1)")]
    ProbabilityDraw(u32),
}
