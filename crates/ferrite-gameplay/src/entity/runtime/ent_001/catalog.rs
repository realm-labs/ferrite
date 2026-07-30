//! Closed entity-type identity ownership for the ENT-001 partition.

use std::collections::{BTreeMap, BTreeSet};

use ferrite_registry::bundle::BundleRegistry;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityOwner {
    pub path: &'static str,
    pub raw_id: u16,
    pub family: &'static str,
    pub slice: &'static str,
}

const fn owner(
    path: &'static str,
    raw_id: u16,
    family: &'static str,
    slice: &'static str,
) -> EntityOwner {
    EntityOwner {
        path,
        raw_id,
        family,
        slice,
    }
}

pub const OWNERS: [EntityOwner; 37] = [
    owner("bat", 10, "bat-runtime", "ENT-BAT-RUNTIME-001"),
    owner("blaze", 14, "blaze-runtime", "ENT-BLAZE-RUNTIME-001"),
    owner("bogged", 16, "bogged-runtime", "ENT-BOGGED-RUNTIME-001"),
    owner("breeze", 17, "breeze-runtime", "ENT-BREEZE-RUNTIME-001"),
    owner(
        "cave_spider",
        22,
        "spider-runtime",
        "ENT-SPIDER-RUNTIME-001",
    ),
    owner("cod", 27, "cod-entity-runtime", "ENT-COD-RUNTIME-001"),
    owner("dolphin", 35, "dolphin-runtime", "ENT-DOLPHIN-RUNTIME-001"),
    owner(
        "elder_guardian",
        40,
        "elder-guardian-runtime",
        "ENT-ELDER-GUARDIAN-RUNTIME-001",
    ),
    owner(
        "endermite",
        42,
        "endermite-runtime",
        "ENT-ENDERMITE-RUNTIME-001",
    ),
    owner("evoker", 46, "evoker-runtime", "ENT-EVOKER-RUNTIME-001"),
    owner("ghast", 57, "ghast-runtime", "ENT-GHAST-RUNTIME-001"),
    owner("giant", 59, "giant-runtime", "ENT-GIANT-RUNTIME-001"),
    owner(
        "glow_squid",
        61,
        "glow-squid-runtime",
        "ENT-GLOW-SQUID-RUNTIME-001",
    ),
    owner(
        "guardian",
        63,
        "guardian-runtime",
        "ENT-GUARDIAN-RUNTIME-001",
    ),
    owner(
        "illusioner",
        68,
        "illusioner-runtime",
        "ENT-ILLUSIONER-RUNTIME-001",
    ),
    owner(
        "iron_golem",
        70,
        "iron-golem-runtime",
        "ENT-IRON-GOLEM-RUNTIME-001",
    ),
    owner(
        "magma_cube",
        80,
        "slime-family-runtime",
        "ENT-SLIME-FAMILY-RUNTIME-001",
    ),
    owner("parched", 97, "parched-runtime", "ENT-PARCHED-RUNTIME-001"),
    owner("phantom", 99, "phantom-runtime", "ENT-PHANTOM-RUNTIME-001"),
    owner(
        "piglin_brute",
        102,
        "piglin-brute-runtime",
        "ENT-PIGLIN-BRUTE-RUNTIME-001",
    ),
    owner(
        "pillager",
        103,
        "pillager-runtime",
        "ENT-PILLAGER-RUNTIME-001",
    ),
    owner(
        "pufferfish",
        107,
        "pufferfish-entity-runtime",
        "ENT-PUFFERFISH-RUNTIME-001",
    ),
    owner(
        "salmon",
        110,
        "salmon-entity-runtime",
        "ENT-SALMON-RUNTIME-001",
    ),
    owner("shulker", 112, "shulker-runtime", "ENT-SHULKER-RUNTIME-001"),
    owner(
        "skeleton",
        115,
        "skeleton-runtime",
        "ENT-SKELETON-RUNTIME-001",
    ),
    owner(
        "slime",
        117,
        "slime-family-runtime",
        "ENT-SLIME-FAMILY-RUNTIME-001",
    ),
    owner(
        "snow_golem",
        121,
        "snow-golem-runtime",
        "ENT-SNOW-GOLEM-RUNTIME-001",
    ),
    owner("spider", 124, "spider-runtime", "ENT-SPIDER-RUNTIME-001"),
    owner("squid", 127, "squid-runtime", "ENT-SQUID-RUNTIME-001"),
    owner("stray", 128, "stray-runtime", "ENT-STRAY-RUNTIME-001"),
    owner(
        "tadpole",
        131,
        "tadpole-entity-runtime",
        "ENT-TADPOLE-RUNTIME-001",
    ),
    owner(
        "tropical_fish",
        137,
        "tropical-fish-entity-runtime",
        "ENT-TROPICAL-FISH-RUNTIME-001",
    ),
    owner("vex", 139, "vex-runtime", "ENT-VEX-RUNTIME-001"),
    owner(
        "villager",
        140,
        "villager-runtime",
        "ENT-VILLAGER-RUNTIME-001",
    ),
    owner(
        "vindicator",
        141,
        "vindicator-runtime",
        "ENT-VINDICATOR-RUNTIME-001",
    ),
    owner(
        "wandering_trader",
        142,
        "wandering-trader-runtime",
        "ENT-WANDERING-TRADER-RUNTIME-001",
    ),
    owner(
        "wither_skeleton",
        147,
        "wither-skeleton-runtime",
        "ENT-WITHER-SKELETON-RUNTIME-001",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityCoverage {
    pub identities: usize,
    pub slices: usize,
}

pub fn verify_entities(registry: &BundleRegistry) -> Result<EntityCoverage, EntityCoverageError> {
    if registry.name().to_string() != "minecraft:entity_type" {
        return Err(EntityCoverageError::WrongRegistry {
            actual: registry.name().to_string(),
        });
    }

    let expected = OWNERS
        .into_iter()
        .map(|owner| (format!("minecraft:{}", owner.path), owner))
        .collect::<BTreeMap<_, _>>();
    for entry in registry.entries() {
        let id = entry.persistent_id().to_string();
        let Some(owner) = expected.get(&id) else {
            continue;
        };
        if entry.family().as_str() != owner.family {
            return Err(EntityCoverageError::WrongFamily {
                path: owner.path,
                expected: owner.family,
                actual: entry.family().as_str().to_owned(),
            });
        }
        let actual = entry.value()["protocol_id"]
            .as_u64()
            .ok_or(EntityCoverageError::MissingRawId(owner.path))?;
        if actual != u64::from(owner.raw_id) {
            return Err(EntityCoverageError::WrongRawId {
                path: owner.path,
                expected: owner.raw_id,
                actual,
            });
        }
    }

    let actual = registry
        .entries()
        .filter(|entry| expected.contains_key(&entry.persistent_id().to_string()))
        .count();
    if actual != expected.len() {
        return Err(EntityCoverageError::MissingIdentities {
            expected: expected.len(),
            actual,
        });
    }
    Ok(EntityCoverage {
        identities: actual,
        slices: OWNERS
            .into_iter()
            .map(|owner| owner.slice)
            .collect::<BTreeSet<_>>()
            .len(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EntityCoverageError {
    #[error("expected minecraft:entity_type registry, found {actual}")]
    WrongRegistry { actual: String },
    #[error("entity {0} has no protocol_id")]
    MissingRawId(&'static str),
    #[error("entity {path} has raw ID {actual}, expected {expected}")]
    WrongRawId {
        path: &'static str,
        expected: u16,
        actual: u64,
    },
    #[error("entity {path} belongs to family {actual}, expected {expected}")]
    WrongFamily {
        path: &'static str,
        expected: &'static str,
        actual: String,
    },
    #[error("found {actual} owned entity identities, expected {expected}")]
    MissingIdentities { expected: usize, actual: usize },
}
