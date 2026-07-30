//! Firework Star component construction, fading, Rocket copying, and tint.

use thiserror::Error;

pub const FIREWORK_STAR_ITEM_ID: u32 = 1_273;
pub const DEFAULT_STAR_TINT: u32 = 0xff8a_8a8a;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionShape {
    SmallBall,
    LargeBall,
    Star,
    Creeper,
    Burst,
}

impl ExplosionShape {
    pub const fn stream_id(self) -> u8 {
        match self {
            Self::SmallBall => 0,
            Self::LargeBall => 1,
            Self::Star => 2,
            Self::Creeper => 3,
            Self::Burst => 4,
        }
    }

    pub const fn from_stream_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::SmallBall),
            1 => Some(Self::LargeBall),
            2 => Some(Self::Star),
            3 => Some(Self::Creeper),
            4 => Some(Self::Burst),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkExplosion {
    pub shape: ExplosionShape,
    pub primary_colors: Vec<u32>,
    pub fade_colors: Vec<u32>,
    pub has_trail: bool,
    pub has_twinkle: bool,
}

impl Default for FireworkExplosion {
    fn default() -> Self {
        Self {
            shape: ExplosionShape::SmallBall,
            primary_colors: Vec::new(),
            fade_colors: Vec::new(),
            has_trail: false,
            has_twinkle: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkStar {
    pub explosion: Option<FireworkExplosion>,
    pub unrelated_patch_fingerprint: u64,
}

impl FireworkStar {
    pub const fn componentless() -> Self {
        Self {
            explosion: None,
            unrelated_patch_fingerprint: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ingredient {
    Gunpowder,
    Dye { firework_color: u32 },
    TaggedDyeWithoutComponent,
    Diamond,
    GlowstoneDust,
    FireCharge,
    GoldNugget,
    Skull,
    Feather,
    Paper,
    FireworkStar(FireworkStar),
    Other,
}

pub fn craft_base_star(grid: &[Ingredient]) -> Result<FireworkStar, FireworkRecipeError> {
    validate_grid(grid)?;
    if grid.len() < 2 {
        return Err(FireworkRecipeError::TooFewIngredients);
    }
    let mut fuel = false;
    let mut trail = false;
    let mut twinkle = false;
    let mut shape = None;
    let mut colors = Vec::new();
    for ingredient in grid {
        match ingredient {
            Ingredient::GlowstoneDust if !twinkle => twinkle = true,
            Ingredient::GlowstoneDust => return Err(FireworkRecipeError::DuplicateTwinkle),
            Ingredient::Diamond if !trail => trail = true,
            Ingredient::Diamond => return Err(FireworkRecipeError::DuplicateTrail),
            Ingredient::Gunpowder if !fuel => fuel = true,
            Ingredient::Gunpowder => return Err(FireworkRecipeError::DuplicateFuel),
            Ingredient::Dye { firework_color } => colors.push(*firework_color),
            Ingredient::FireCharge
            | Ingredient::GoldNugget
            | Ingredient::Skull
            | Ingredient::Feather => {
                let selected = shape_for(ingredient);
                if shape.replace(selected).is_some() {
                    return Err(FireworkRecipeError::DuplicateShape);
                }
            }
            Ingredient::TaggedDyeWithoutComponent => {
                return Err(FireworkRecipeError::DyeWithoutComponent);
            }
            Ingredient::Paper | Ingredient::FireworkStar(_) | Ingredient::Other => {
                return Err(FireworkRecipeError::ForeignIngredient);
            }
        }
    }
    if !fuel {
        return Err(FireworkRecipeError::MissingFuel);
    }
    if colors.is_empty() {
        return Err(FireworkRecipeError::MissingDye);
    }
    Ok(FireworkStar {
        explosion: Some(FireworkExplosion {
            shape: shape.unwrap_or(ExplosionShape::SmallBall),
            primary_colors: colors,
            fade_colors: Vec::new(),
            has_trail: trail,
            has_twinkle: twinkle,
        }),
        unrelated_patch_fingerprint: 0,
    })
}

fn shape_for(ingredient: &Ingredient) -> ExplosionShape {
    match ingredient {
        Ingredient::FireCharge => ExplosionShape::LargeBall,
        Ingredient::GoldNugget => ExplosionShape::Star,
        Ingredient::Skull => ExplosionShape::Creeper,
        Ingredient::Feather => ExplosionShape::Burst,
        _ => ExplosionShape::SmallBall,
    }
}

pub fn craft_faded_star(grid: &[Ingredient]) -> Result<FireworkStar, FireworkRecipeError> {
    validate_grid(grid)?;
    let mut target = None;
    let mut fade_colors = Vec::new();
    for ingredient in grid {
        match ingredient {
            Ingredient::Dye { firework_color } => fade_colors.push(*firework_color),
            Ingredient::FireworkStar(star) if target.is_none() => target = Some(star.clone()),
            Ingredient::FireworkStar(_) => return Err(FireworkRecipeError::DuplicateTarget),
            Ingredient::TaggedDyeWithoutComponent => {
                return Err(FireworkRecipeError::DyeWithoutComponent);
            }
            _ => return Err(FireworkRecipeError::ForeignIngredient),
        }
    }
    let target = target.ok_or(FireworkRecipeError::MissingTarget)?;
    if fade_colors.is_empty() {
        return Err(FireworkRecipeError::MissingDye);
    }
    let mut explosion = target.explosion.clone().unwrap_or_default();
    explosion.fade_colors = fade_colors;
    Ok(FireworkStar {
        explosion: Some(explosion),
        unrelated_patch_fingerprint: target.unrelated_patch_fingerprint,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireworkRocket {
    pub count: u8,
    pub flight_duration: u8,
    pub explosions: Vec<FireworkExplosion>,
}

pub fn craft_rockets(grid: &[Ingredient]) -> Result<FireworkRocket, FireworkRecipeError> {
    validate_grid(grid)?;
    let mut paper = false;
    let mut fuel = 0_u8;
    let mut explosions = Vec::new();
    for ingredient in grid {
        match ingredient {
            Ingredient::Paper if !paper => paper = true,
            Ingredient::Paper => return Err(FireworkRecipeError::DuplicatePaper),
            Ingredient::Gunpowder if fuel < 3 => fuel += 1,
            Ingredient::Gunpowder => return Err(FireworkRecipeError::TooMuchFuel),
            Ingredient::FireworkStar(star) => {
                if let Some(explosion) = &star.explosion {
                    explosions.push(explosion.clone());
                }
            }
            _ => return Err(FireworkRecipeError::ForeignIngredient),
        }
    }
    if !paper {
        return Err(FireworkRecipeError::MissingPaper);
    }
    if fuel == 0 {
        return Err(FireworkRecipeError::MissingFuel);
    }
    Ok(FireworkRocket {
        count: 3,
        flight_duration: fuel,
        explosions,
    })
}

pub fn star_tint(star: &FireworkStar) -> u32 {
    let Some(explosion) = &star.explosion else {
        return DEFAULT_STAR_TINT;
    };
    let colors = &explosion.primary_colors;
    if colors.is_empty() {
        return DEFAULT_STAR_TINT;
    }
    if colors.len() == 1 {
        return 0xff00_0000 | (colors[0] & 0x00ff_ffff);
    }
    let (red, green, blue) = colors.iter().fold((0_u64, 0_u64, 0_u64), |sum, color| {
        (
            sum.0 + u64::from((color >> 16) & 0xff),
            sum.1 + u64::from((color >> 8) & 0xff),
            sum.2 + u64::from(color & 0xff),
        )
    });
    let count = colors.len() as u64;
    0xff00_0000
        | ((red / count) as u32) << 16
        | ((green / count) as u32) << 8
        | (blue / count) as u32
}

pub const fn rocket_damage(explosion_count: usize) -> Option<usize> {
    if explosion_count == 0 {
        None
    } else {
        Some(5 + 2 * explosion_count)
    }
}

pub const fn rocket_lifetime(
    flight_duration: u8,
    bounded_six_draw: u8,
    bounded_seven_draw: u8,
) -> Option<u16> {
    if flight_duration < 1
        || flight_duration > 3
        || bounded_six_draw >= 6
        || bounded_seven_draw >= 7
    {
        return None;
    }
    Some(10 * (1 + flight_duration as u16) + bounded_six_draw as u16 + bounded_seven_draw as u16)
}

fn validate_grid(grid: &[Ingredient]) -> Result<(), FireworkRecipeError> {
    if grid.len() > 9 {
        Err(FireworkRecipeError::GridOverflow(grid.len()))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FireworkRecipeError {
    #[error("crafting grid contains {0} occupied slots, maximum is 9")]
    GridOverflow(usize),
    #[error("base Firework Star requires at least two occupied slots")]
    TooFewIngredients,
    #[error("recipe is missing Gunpowder")]
    MissingFuel,
    #[error("recipe contains duplicate Gunpowder")]
    DuplicateFuel,
    #[error("Firework Rocket contains more than three Gunpowder")]
    TooMuchFuel,
    #[error("recipe is missing a component-bearing dye")]
    MissingDye,
    #[error("dye-tag ingredient has no DYE component")]
    DyeWithoutComponent,
    #[error("recipe contains duplicate trail ingredient")]
    DuplicateTrail,
    #[error("recipe contains duplicate twinkle ingredient")]
    DuplicateTwinkle,
    #[error("recipe contains duplicate shape ingredient")]
    DuplicateShape,
    #[error("fade recipe contains duplicate Firework Star")]
    DuplicateTarget,
    #[error("fade recipe is missing its Firework Star")]
    MissingTarget,
    #[error("Rocket recipe contains duplicate Paper")]
    DuplicatePaper,
    #[error("Rocket recipe is missing Paper")]
    MissingPaper,
    #[error("recipe contains an unclassified ingredient")]
    ForeignIngredient,
}
