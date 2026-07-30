//! Closed imported catalog ownership for the SIM-004 material partition.

use ferrite_registry::bundle::BundleRegistry;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialItem {
    Diamond,
    DriedKelp,
    Emerald,
    Feather,
    FireworkStar,
    Flint,
    GlowstoneDust,
    Gunpowder,
    LapisLazuli,
    Leather,
    Quartz,
    Redstone,
    SlimeBall,
    Stick,
    TurtleScute,
}

impl MaterialItem {
    pub const ALL: [Self; 15] = [
        Self::Diamond,
        Self::DriedKelp,
        Self::Emerald,
        Self::Feather,
        Self::FireworkStar,
        Self::Flint,
        Self::GlowstoneDust,
        Self::Gunpowder,
        Self::LapisLazuli,
        Self::Leather,
        Self::Quartz,
        Self::Redstone,
        Self::SlimeBall,
        Self::Stick,
        Self::TurtleScute,
    ];

    pub const fn path(self) -> &'static str {
        match self {
            Self::Diamond => "diamond",
            Self::DriedKelp => "dried_kelp",
            Self::Emerald => "emerald",
            Self::Feather => "feather",
            Self::FireworkStar => "firework_star",
            Self::Flint => "flint",
            Self::GlowstoneDust => "glowstone_dust",
            Self::Gunpowder => "gunpowder",
            Self::LapisLazuli => "lapis_lazuli",
            Self::Leather => "leather",
            Self::Quartz => "quartz",
            Self::Redstone => "redstone",
            Self::SlimeBall => "slime_ball",
            Self::Stick => "stick",
            Self::TurtleScute => "turtle_scute",
        }
    }

    pub const fn raw_id(self) -> u32 {
        match self {
            Self::Diamond => 926,
            Self::DriedKelp => 1_136,
            Self::Emerald => 927,
            Self::Feather => 977,
            Self::FireworkStar => 1_273,
            Self::Flint => 1_010,
            Self::GlowstoneDust => 1_085,
            Self::Gunpowder => 978,
            Self::LapisLazuli => 928,
            Self::Leather => 1_045,
            Self::Quartz => 929,
            Self::Redstone => 745,
            Self::SlimeBall => 1_059,
            Self::Stick => 974,
            Self::TurtleScute => 916,
        }
    }

    pub const fn family(self) -> &'static str {
        match self {
            Self::Diamond => "diamond-runtime",
            Self::DriedKelp => "dried-kelp-runtime",
            Self::Emerald => "emerald-runtime",
            Self::Feather => "feather-runtime",
            Self::FireworkStar => "firework-star-runtime",
            Self::Flint => "flint-runtime",
            Self::GlowstoneDust => "glowstone-dust-runtime",
            Self::Gunpowder => "gunpowder-runtime",
            Self::LapisLazuli => "lapis-lazuli-runtime",
            Self::Leather => "leather-runtime",
            Self::Quartz => "quartz-runtime",
            Self::Redstone => "redstone-dust-runtime",
            Self::SlimeBall => "slime-ball-runtime",
            Self::Stick => "stick-runtime",
            Self::TurtleScute => "turtle-scute-runtime",
        }
    }

    pub const fn slice(self) -> &'static str {
        match self {
            Self::Diamond => "ITM-DIAMOND-RUNTIME-001",
            Self::DriedKelp => "ITM-DRIED-KELP-RUNTIME-001",
            Self::Emerald => "ITM-EMERALD-RUNTIME-001",
            Self::Feather => "ITM-FEATHER-RUNTIME-001",
            Self::FireworkStar => "ITM-FIREWORK-STAR-RUNTIME-001",
            Self::Flint => "ITM-FLINT-RUNTIME-001",
            Self::GlowstoneDust => "ITM-GLOWSTONE-DUST-RUNTIME-001",
            Self::Gunpowder => "ITM-GUNPOWDER-RUNTIME-001",
            Self::LapisLazuli => "ITM-LAPIS-LAZULI-RUNTIME-001",
            Self::Leather => "ITM-LEATHER-RUNTIME-001",
            Self::Quartz => "ITM-QUARTZ-RUNTIME-001",
            Self::Redstone => "ITM-REDSTONE-RUNTIME-001",
            Self::SlimeBall => "ITM-SLIME-BALL-RUNTIME-001",
            Self::Stick => "ITM-STICK-RUNTIME-001",
            Self::TurtleScute => "ITM-TURTLE-SCUTE-RUNTIME-001",
        }
    }

    pub fn from_path(path: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|item| item.path() == path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialCatalogCoverage {
    pub families: usize,
    pub slices: usize,
    pub items: usize,
}

pub fn verify_families(
    registry: &BundleRegistry,
) -> Result<MaterialCatalogCoverage, MaterialCatalogError> {
    if registry.name().to_string() != "minecraft:item" {
        return Err(MaterialCatalogError::WrongRegistry(
            registry.name().to_string(),
        ));
    }

    let mut family_counts = BTreeMap::<&str, usize>::new();
    let mut found = 0;
    for entry in registry.entries() {
        let family = entry.family().as_str();
        let Some(expected) = MaterialItem::ALL
            .into_iter()
            .find(|item| item.family() == family)
        else {
            continue;
        };
        let expected_id = format!("minecraft:{}", expected.path());
        let actual_id = entry.persistent_id().to_string();
        if actual_id != expected_id {
            return Err(MaterialCatalogError::UnexpectedFamilyMember {
                family: family.to_owned(),
                item: actual_id,
            });
        }
        let components = entry.value()["components"]
            .as_object()
            .ok_or(MaterialCatalogError::MissingComponents(expected))?;
        if components["minecraft:max_stack_size"] != 64
            || components["minecraft:rarity"] != "common"
        {
            return Err(MaterialCatalogError::WrongDefaults(expected));
        }
        *family_counts.entry(family).or_default() += 1;
        found += 1;
    }

    for item in MaterialItem::ALL {
        let actual = family_counts
            .get(item.family())
            .copied()
            .unwrap_or_default();
        if actual != 1 {
            return Err(MaterialCatalogError::FamilyCount {
                family: item.family(),
                actual,
            });
        }
    }

    Ok(MaterialCatalogCoverage {
        families: family_counts.len(),
        slices: MaterialItem::ALL.len(),
        items: found,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MaterialCatalogError {
    #[error("expected minecraft:item registry, found {0}")]
    WrongRegistry(String),
    #[error("owned family {family} contains unexpected item {item}")]
    UnexpectedFamilyMember { family: String, item: String },
    #[error("{0:?} has no component object")]
    MissingComponents(MaterialItem),
    #[error("{0:?} does not retain the common 64-stack defaults")]
    WrongDefaults(MaterialItem),
    #[error("owned family {family} contains {actual} entries, expected one")]
    FamilyCount { family: &'static str, actual: usize },
}
