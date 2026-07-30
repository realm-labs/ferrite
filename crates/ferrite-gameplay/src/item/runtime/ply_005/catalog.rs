//! Closed imported item-family ownership for the PLY-005 partition.

use ferrite_registry::bundle::BundleRegistry;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FamilyOwner {
    pub family: &'static str,
    pub slice: &'static str,
    pub expected_items: usize,
}

const fn owner(family: &'static str, slice: &'static str, expected_items: usize) -> FamilyOwner {
    FamilyOwner {
        family,
        slice,
        expected_items,
    }
}

pub const OWNERS: [FamilyOwner; 44] = [
    owner(
        "ancient-city-relic-ingredient",
        "ITM-ANCIENT-CITY-RELIC-RUNTIME-001",
        2,
    ),
    owner(
        "armadillo-scute-runtime",
        "ITM-ARMADILLO-SCUTE-RUNTIME-001",
        1,
    ),
    owner("armor-stand-runtime", "ITM-ARMOR-STAND-RUNTIME-001", 1),
    owner(
        "arrow-ammunition-runtime",
        "ITM-ARROW-AMMUNITION-RUNTIME-001",
        3,
    ),
    owner(
        "blaze-material-runtime",
        "ITM-BLAZE-MATERIAL-RUNTIME-001",
        2,
    ),
    owner("bone-runtime", "ITM-BONE-RUNTIME-001", 1),
    owner("bread-runtime", "ITM-BREAD-RUNTIME-001", 1),
    owner("breeze-rod-runtime", "ITM-BREEZE-ROD-RUNTIME-001", 1),
    owner("bundle-runtime", "ITM-BUNDLE-RUNTIME-001", 17),
    owner("cod-runtime", "ITM-COD-RUNTIME-001", 2),
    owner(
        "conduit-material-runtime",
        "ITM-CONDUIT-MATERIAL-RUNTIME-001",
        2,
    ),
    owner("dragon-breath-runtime", "ITM-DRAGON-BREATH-RUNTIME-001", 1),
    owner(
        "drink-container-runtime",
        "ITM-DRINK-CONTAINER-RUNTIME-001",
        2,
    ),
    owner("egg-runtime", "ITM-EGG-RUNTIME-001", 3),
    owner("end-crystal-runtime", "ITM-END-CRYSTAL-RUNTIME-001", 1),
    owner(
        "fermented-spider-eye-runtime",
        "ITM-FERMENTED-SPIDER-EYE-RUNTIME-001",
        1,
    ),
    owner("ghast-tear-runtime", "ITM-GHAST-TEAR-RUNTIME-001", 1),
    owner(
        "glistering-melon-slice-runtime",
        "ITM-GLISTERING-MELON-SLICE-RUNTIME-001",
        1,
    ),
    owner("golden-carrot-runtime", "ITM-GOLDEN-CARROT-RUNTIME-001", 1),
    owner(
        "hanging-decoration-runtime",
        "ITM-HANGING-DECORATION-RUNTIME-001",
        3,
    ),
    owner(
        "knowledge-book-runtime",
        "ITM-KNOWLEDGE-BOOK-RUNTIME-001",
        1,
    ),
    owner("magma-cream-runtime", "ITM-MAGMA-CREAM-RUNTIME-001", 1),
    owner("minecart-item-runtime", "ITM-MINECART-RUNTIME-001", 5),
    owner(
        "command-block-minecart-item-runtime",
        "ITM-MINECART-RUNTIME-001",
        1,
    ),
    owner("mob-bucket-runtime", "ITM-MOB-BUCKET-RUNTIME-001", 7),
    owner(
        "nautilus-armor-runtime",
        "ITM-NAUTILUS-ARMOR-RUNTIME-001",
        5,
    ),
    owner("nether-star-runtime", "ITM-NETHER-STAR-RUNTIME-001", 1),
    owner(
        "ominous-bottle-runtime",
        "ITM-OMINOUS-BOTTLE-RUNTIME-001",
        1,
    ),
    owner(
        "phantom-membrane-runtime",
        "ITM-PHANTOM-MEMBRANE-RUNTIME-001",
        1,
    ),
    owner("potion-runtime", "ITM-POTION-RUNTIME-001", 1),
    owner("pottery-sherd-runtime", "ITM-POTTERY-SHERD-RUNTIME-001", 23),
    owner("pufferfish-runtime", "ITM-PUFFERFISH-RUNTIME-001", 1),
    owner("rabbit-foot-runtime", "ITM-RABBIT-FOOT-RUNTIME-001", 1),
    owner("rotten-flesh-runtime", "ITM-ROTTEN-FLESH-RUNTIME-001", 1),
    owner("salmon-runtime", "ITM-SALMON-RUNTIME-001", 2),
    owner("shulker-shell-runtime", "ITM-SHULKER-SHELL-RUNTIME-001", 1),
    owner(
        "smithing-template-runtime",
        "ITM-SMITHING-TEMPLATE-RUNTIME-001",
        19,
    ),
    owner("spear-item-runtime", "ITM-SPEAR-RUNTIME-001", 7),
    owner("spider-eye-runtime", "ITM-SPIDER-EYE-RUNTIME-001", 1),
    owner(
        "steering-stick-item-runtime",
        "ITM-STEERING-STICK-RUNTIME-001",
        2,
    ),
    owner("stew-and-bowl-runtime", "ITM-STEW-RUNTIME-001", 5),
    owner("sugar-runtime", "ITM-SUGAR-RUNTIME-001", 1),
    owner("trial-key-runtime", "ITM-TRIAL-KEY-RUNTIME-001", 2),
    owner("tropical-fish-runtime", "ITM-TROPICAL-FISH-RUNTIME-001", 1),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FamilyCoverage {
    pub families: usize,
    pub slices: usize,
    pub items: usize,
}

pub fn verify_families(registry: &BundleRegistry) -> Result<FamilyCoverage, FamilyCoverageError> {
    if registry.name().to_string() != "minecraft:item" {
        return Err(FamilyCoverageError::WrongRegistry {
            actual: registry.name().to_string(),
        });
    }
    let mut expected = BTreeMap::new();
    let mut slices = BTreeSet::new();
    for owner in OWNERS {
        if expected.insert(owner.family, owner).is_some() {
            return Err(FamilyCoverageError::DuplicateFamily(owner.family));
        }
        slices.insert(owner.slice);
    }
    let mut actual = BTreeMap::<&str, usize>::new();
    for entry in registry.entries() {
        let family = entry.family().as_str();
        if expected.contains_key(family) {
            *actual.entry(family).or_default() += 1;
        }
    }
    for owner in OWNERS {
        let count = actual.get(owner.family).copied().unwrap_or_default();
        if count != owner.expected_items {
            return Err(FamilyCoverageError::FamilyCount {
                family: owner.family,
                expected: owner.expected_items,
                actual: count,
            });
        }
    }
    Ok(FamilyCoverage {
        families: expected.len(),
        slices: slices.len(),
        items: actual.values().sum(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FamilyCoverageError {
    #[error("expected minecraft:item registry, found {actual}")]
    WrongRegistry { actual: String },
    #[error("duplicate family owner {0}")]
    DuplicateFamily(&'static str),
    #[error("owned family {family} contains {actual} items, expected {expected}")]
    FamilyCount {
        family: &'static str,
        expected: usize,
        actual: usize,
    },
}
