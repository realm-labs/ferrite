//! Closed mapping from imported block families to audited runtime owners.

use ferrite_registry::minecraft_block::MinecraftBlockCatalog;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BehaviorOwner {
    pub family: &'static str,
    pub slice: &'static str,
    pub expected_blocks: usize,
}

pub const OWNERS: [BehaviorOwner; 40] = [
    owner("air-runtime", "BLK-AIR-RUNTIME-001", 3),
    owner(
        "amethyst-block-runtime",
        "BLK-AMETHYST-BLOCK-RUNTIME-001",
        1,
    ),
    owner("banner-runtime", "BLK-BANNER-RUNTIME-001", 32),
    owner(
        "base-deepslate-runtime",
        "BLK-BASE-DEEPSLATE-RUNTIME-001",
        1,
    ),
    owner(
        "beacon-storage-runtime",
        "BLK-BEACON-STORAGE-RUNTIME-001",
        5,
    ),
    owner("bedrock-runtime", "BLK-BEDROCK-RUNTIME-001", 1),
    owner("bone-block-runtime", "BLK-BONE-BLOCK-RUNTIME-001", 1),
    owner("bricks-runtime", "BLK-BRICKS-RUNTIME-001", 1),
    owner("concrete-runtime", "BLK-CONCRETE-RUNTIME-001", 16),
    owner("decorated-pot-runtime", "BLK-DECORATED-POT-RUNTIME-001", 1),
    owner(
        "deepslate-masonry-runtime",
        "BLK-DEEPSLATE-MASONRY-RUNTIME-001",
        7,
    ),
    owner(
        "geode-shell-identities",
        "BLK-GEODE-SHELL-IDENTITIES-001",
        2,
    ),
    owner("glass-runtime", "BLK-GLASS-RUNTIME-001", 1),
    owner(
        "glazed-terracotta-runtime",
        "BLK-GLAZED-TERRACOTTA-RUNTIME-001",
        16,
    ),
    owner("honey-runtime", "BLK-HONEY-RUNTIME-001", 1),
    owner(
        "honeycomb-block-runtime",
        "BLK-HONEYCOMB-BLOCK-RUNTIME-001",
        1,
    ),
    owner("jigsaw-runtime", "BLK-JIGSAW-RUNTIME-001", 1),
    owner("lapis-block-runtime", "BLK-LAPIS-BLOCK-RUNTIME-001", 1),
    owner("lava-cauldron-runtime", "BLK-LAVA-CAULDRON-RUNTIME-001", 1),
    owner("magma-runtime", "BLK-MAGMA-RUNTIME-001", 1),
    owner("mud-bricks-runtime", "BLK-MUD-BRICKS-RUNTIME-001", 1),
    owner("packed-mud-runtime", "BLK-PACKED-MUD-RUNTIME-001", 1),
    owner(
        "polished-basalt-runtime",
        "BLK-POLISHED-BASALT-RUNTIME-001",
        1,
    ),
    owner("purpur-block-runtime", "BLK-PURPUR-BLOCK-RUNTIME-001", 2),
    owner("quartz-block-runtime", "BLK-QUARTZ-RUNTIME-001", 5),
    owner("raw-storage-runtime", "BLK-RAW-STORAGE-RUNTIME-001", 3),
    owner(
        "red-nether-bricks-runtime",
        "BLK-RED-NETHER-BRICKS-RUNTIME-001",
        1,
    ),
    owner(
        "redstone-block-runtime",
        "BLK-REDSTONE-BLOCK-RUNTIME-001",
        1,
    ),
    owner(
        "reinforced-deepslate-runtime",
        "BLK-REINFORCED-DEEPSLATE-RUNTIME-001",
        1,
    ),
    owner("sandstone-block-runtime", "BLK-SANDSTONE-RUNTIME-001", 8),
    owner("shelf-runtime", "BLK-SHELF-RUNTIME-001", 12),
    owner("slime-runtime", "BLK-SLIME-RUNTIME-001", 1),
    owner("soul-sand-runtime", "BLK-SOUL-SAND-RUNTIME-001", 1),
    owner("stained-glass-runtime", "BLK-STAINED-GLASS-RUNTIME-001", 16),
    owner("stone-brick-runtime", "BLK-STONE-BRICK-RUNTIME-001", 4),
    owner("stone-variant-runtime", "BLK-STONE-VARIANT-RUNTIME-001", 6),
    owner("structure-block-runtime", "BLK-STRUCTURE-RUNTIME-001", 1),
    owner(
        "structure-void-runtime",
        "BLK-STRUCTURE-VOID-RUNTIME-001",
        1,
    ),
    owner("terracotta-runtime", "BLK-TERRACOTTA-RUNTIME-001", 17),
    owner("tinted-glass-runtime", "BLK-TINTED-GLASS-RUNTIME-001", 1),
];

const fn owner(family: &'static str, slice: &'static str, expected_blocks: usize) -> BehaviorOwner {
    BehaviorOwner {
        family,
        slice,
        expected_blocks,
    }
}

pub fn owner_for_family(family: &str) -> Option<&'static BehaviorOwner> {
    OWNERS.iter().find(|owner| owner.family == family)
}

pub fn verify_owned_families(
    catalog: &MinecraftBlockCatalog,
) -> Result<OwnedFamilyCoverage, FamilyCoverageError> {
    let mut actual = BTreeMap::<&str, usize>::new();
    let mut states = 0_u32;
    for definition in catalog.definitions() {
        let family = definition.family().as_str();
        if owner_for_family(family).is_some() {
            *actual.entry(family).or_default() += 1;
            states = states
                .checked_add(definition.schema().state_count())
                .ok_or(FamilyCoverageError::StateCountOverflow)?;
        }
    }
    for owner in OWNERS {
        let count = actual.get(owner.family).copied().unwrap_or_default();
        if count != owner.expected_blocks {
            return Err(FamilyCoverageError::FamilyCount {
                family: owner.family,
                expected: owner.expected_blocks,
                actual: count,
            });
        }
    }
    Ok(OwnedFamilyCoverage {
        families: OWNERS.len(),
        blocks: actual.values().sum(),
        states,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedFamilyCoverage {
    pub families: usize,
    pub blocks: usize,
    pub states: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FamilyCoverageError {
    #[error("owned family {family} has {actual} blocks, expected {expected}")]
    FamilyCount {
        family: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("owned block state count exceeds u32")]
    StateCountOverflow,
}
