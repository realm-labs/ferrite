//! Closed identity mapping for the ten item slices primarily owned by `BLK-001`.

use ferrite_registry::bundle::BundleRegistry;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemKind {
    AmethystShard,
    Apple,
    Brick,
    Coal,
    Charcoal,
    RawCopper,
    CopperIngot,
    CopperNugget,
    EnchantedGoldenApple,
    RawGold,
    GoldIngot,
    GoldNugget,
    GoldenApple,
    RawIron,
    IronIngot,
    IronNugget,
    NetheriteIngot,
    NetheriteScrap,
    PrismarineShard,
    PrismarineCrystals,
    BakedPotato,
    Beef,
    CookedBeef,
    Book,
    EnchantedBook,
    WritableBook,
    WrittenBook,
    Chicken,
    CookedChicken,
    Cookie,
    Mutton,
    CookedMutton,
    Porkchop,
    CookedPorkchop,
    PumpkinPie,
    Rabbit,
    CookedRabbit,
    RabbitHide,
}

impl ItemKind {
    pub const ALL: [Self; 18] = [
        Self::AmethystShard,
        Self::Apple,
        Self::Brick,
        Self::Coal,
        Self::Charcoal,
        Self::RawCopper,
        Self::CopperIngot,
        Self::CopperNugget,
        Self::EnchantedGoldenApple,
        Self::RawGold,
        Self::GoldIngot,
        Self::GoldNugget,
        Self::GoldenApple,
        Self::RawIron,
        Self::IronIngot,
        Self::IronNugget,
        Self::NetheriteIngot,
        Self::NetheriteScrap,
    ];
    pub const PRISMARINE: [Self; 2] = [Self::PrismarineShard, Self::PrismarineCrystals];
    pub const BLK_003: [Self; 18] = [
        Self::BakedPotato,
        Self::Beef,
        Self::CookedBeef,
        Self::Book,
        Self::EnchantedBook,
        Self::WritableBook,
        Self::WrittenBook,
        Self::Chicken,
        Self::CookedChicken,
        Self::Cookie,
        Self::Mutton,
        Self::CookedMutton,
        Self::Porkchop,
        Self::CookedPorkchop,
        Self::PumpkinPie,
        Self::Rabbit,
        Self::CookedRabbit,
        Self::RabbitHide,
    ];

    pub const fn path(self) -> &'static str {
        match self {
            Self::AmethystShard => "amethyst_shard",
            Self::Apple => "apple",
            Self::Brick => "brick",
            Self::Coal => "coal",
            Self::Charcoal => "charcoal",
            Self::RawCopper => "raw_copper",
            Self::CopperIngot => "copper_ingot",
            Self::CopperNugget => "copper_nugget",
            Self::EnchantedGoldenApple => "enchanted_golden_apple",
            Self::RawGold => "raw_gold",
            Self::GoldIngot => "gold_ingot",
            Self::GoldNugget => "gold_nugget",
            Self::GoldenApple => "golden_apple",
            Self::RawIron => "raw_iron",
            Self::IronIngot => "iron_ingot",
            Self::IronNugget => "iron_nugget",
            Self::NetheriteIngot => "netherite_ingot",
            Self::NetheriteScrap => "netherite_scrap",
            Self::PrismarineShard => "prismarine_shard",
            Self::PrismarineCrystals => "prismarine_crystals",
            Self::BakedPotato => "baked_potato",
            Self::Beef => "beef",
            Self::CookedBeef => "cooked_beef",
            Self::Book => "book",
            Self::EnchantedBook => "enchanted_book",
            Self::WritableBook => "writable_book",
            Self::WrittenBook => "written_book",
            Self::Chicken => "chicken",
            Self::CookedChicken => "cooked_chicken",
            Self::Cookie => "cookie",
            Self::Mutton => "mutton",
            Self::CookedMutton => "cooked_mutton",
            Self::Porkchop => "porkchop",
            Self::CookedPorkchop => "cooked_porkchop",
            Self::PumpkinPie => "pumpkin_pie",
            Self::Rabbit => "rabbit",
            Self::CookedRabbit => "cooked_rabbit",
            Self::RabbitHide => "rabbit_hide",
        }
    }

    pub const fn raw_id(self) -> u32 {
        match self {
            Self::AmethystShard => 930,
            Self::Apple => 921,
            Self::Brick => 1054,
            Self::Coal => 924,
            Self::Charcoal => 925,
            Self::RawCopper => 933,
            Self::CopperIngot => 934,
            Self::CopperNugget => 1336,
            Self::EnchantedGoldenApple => 1015,
            Self::RawGold => 935,
            Self::GoldIngot => 936,
            Self::GoldNugget => 1147,
            Self::GoldenApple => 1014,
            Self::RawIron => 931,
            Self::IronIngot => 932,
            Self::IronNugget => 1335,
            Self::NetheriteIngot => 937,
            Self::NetheriteScrap => 938,
            Self::PrismarineShard => 1277,
            Self::PrismarineCrystals => 1278,
            Self::BakedPotato => 1259,
            Self::Beef => 1139,
            Self::CookedBeef => 1140,
            Self::Book => 1058,
            Self::EnchantedBook => 1274,
            Self::WritableBook => 1250,
            Self::WrittenBook => 1251,
            Self::Chicken => 1141,
            Self::CookedChicken => 1142,
            Self::Cookie => 1131,
            Self::Mutton => 1294,
            Self::CookedMutton => 1295,
            Self::Porkchop => 1011,
            Self::CookedPorkchop => 1012,
            Self::PumpkinPie => 1271,
            Self::Rabbit => 1279,
            Self::CookedRabbit => 1280,
            Self::RabbitHide => 1283,
        }
    }

    pub const fn family(self) -> &'static str {
        match self {
            Self::AmethystShard => "amethyst-shard-runtime",
            Self::Apple => "apple-runtime",
            Self::Brick => "brick-runtime",
            Self::Coal | Self::Charcoal => "coal-runtime",
            Self::RawCopper | Self::CopperIngot | Self::CopperNugget => "copper-material-runtime",
            Self::EnchantedGoldenApple => "enchanted-golden-apple-runtime",
            Self::RawGold | Self::GoldIngot | Self::GoldNugget => "gold-material-runtime",
            Self::GoldenApple => "golden-apple-runtime",
            Self::RawIron | Self::IronIngot | Self::IronNugget => "iron-material-runtime",
            Self::NetheriteIngot | Self::NetheriteScrap => "netherite-material-runtime",
            Self::PrismarineShard | Self::PrismarineCrystals => "prismarine-material-runtime",
            Self::BakedPotato => "baked-potato-runtime",
            Self::Beef | Self::CookedBeef => "beef-runtime",
            Self::Book | Self::EnchantedBook | Self::WritableBook | Self::WrittenBook => {
                "book-family-runtime"
            }
            Self::Chicken | Self::CookedChicken => "chicken-runtime",
            Self::Cookie => "cookie-runtime",
            Self::Mutton | Self::CookedMutton => "mutton-runtime",
            Self::Porkchop | Self::CookedPorkchop => "porkchop-runtime",
            Self::PumpkinPie => "pumpkin-pie-runtime",
            Self::Rabbit | Self::CookedRabbit | Self::RabbitHide => "rabbit-material-runtime",
        }
    }

    pub const fn slice(self) -> &'static str {
        match self {
            Self::AmethystShard => "ITM-AMETHYST-SHARD-RUNTIME-001",
            Self::Apple => "ITM-APPLE-RUNTIME-001",
            Self::Brick => "ITM-BRICK-RUNTIME-001",
            Self::Coal | Self::Charcoal => "ITM-COAL-RUNTIME-001",
            Self::RawCopper | Self::CopperIngot | Self::CopperNugget => {
                "ITM-COPPER-MATERIAL-RUNTIME-001"
            }
            Self::EnchantedGoldenApple => "ITM-ENCHANTED-GOLDEN-APPLE-RUNTIME-001",
            Self::RawGold | Self::GoldIngot | Self::GoldNugget => "ITM-GOLD-MATERIAL-RUNTIME-001",
            Self::GoldenApple => "ITM-GOLDEN-APPLE-RUNTIME-001",
            Self::RawIron | Self::IronIngot | Self::IronNugget => "ITM-IRON-MATERIAL-RUNTIME-001",
            Self::NetheriteIngot | Self::NetheriteScrap => "ITM-NETHERITE-MATERIAL-RUNTIME-001",
            Self::PrismarineShard | Self::PrismarineCrystals => {
                "ITM-PRISMARINE-MATERIAL-RUNTIME-001"
            }
            Self::BakedPotato => "ITM-BAKED-POTATO-RUNTIME-001",
            Self::Beef | Self::CookedBeef => "ITM-BEEF-RUNTIME-001",
            Self::Book | Self::EnchantedBook | Self::WritableBook | Self::WrittenBook => {
                "ITM-BOOK-FAMILY-RUNTIME-001"
            }
            Self::Chicken | Self::CookedChicken => "ITM-CHICKEN-RUNTIME-001",
            Self::Cookie => "ITM-COOKIE-RUNTIME-001",
            Self::Mutton | Self::CookedMutton => "ITM-MUTTON-RUNTIME-001",
            Self::Porkchop | Self::CookedPorkchop => "ITM-PORKCHOP-RUNTIME-001",
            Self::PumpkinPie => "ITM-PUMPKIN-PIE-RUNTIME-001",
            Self::Rabbit | Self::CookedRabbit | Self::RabbitHide => {
                "ITM-RABBIT-MATERIAL-RUNTIME-001"
            }
        }
    }

    pub const fn maximum_stack(self) -> u16 {
        match self {
            Self::EnchantedBook | Self::WritableBook => 1,
            Self::WrittenBook => 16,
            _ => 64,
        }
    }

    pub const fn rarity(self) -> Rarity {
        match self {
            Self::EnchantedGoldenApple | Self::EnchantedBook => Rarity::Rare,
            _ => Rarity::Common,
        }
    }

    pub const fn forced_glint(self) -> bool {
        matches!(
            self,
            Self::EnchantedGoldenApple | Self::EnchantedBook | Self::WrittenBook
        )
    }

    pub const fn resists_fire_damage(self) -> bool {
        matches!(self, Self::NetheriteIngot | Self::NetheriteScrap)
    }

    pub const fn trim_material(self) -> Option<&'static str> {
        match self {
            Self::AmethystShard => Some("minecraft:amethyst"),
            Self::CopperIngot => Some("minecraft:copper"),
            Self::GoldIngot => Some("minecraft:gold"),
            Self::IronIngot => Some("minecraft:iron"),
            Self::NetheriteIngot => Some("minecraft:netherite"),
            _ => None,
        }
    }

    pub fn from_path(path: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .chain(Self::PRISMARINE)
            .chain(Self::BLK_003)
            .find(|item| item.path() == path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Rare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemOwner {
    pub family: &'static str,
    pub slice: &'static str,
    pub expected_items: usize,
}

pub const OWNERS: [ItemOwner; 10] = [
    owner(
        "amethyst-shard-runtime",
        "ITM-AMETHYST-SHARD-RUNTIME-001",
        1,
    ),
    owner("apple-runtime", "ITM-APPLE-RUNTIME-001", 1),
    owner("brick-runtime", "ITM-BRICK-RUNTIME-001", 1),
    owner("coal-runtime", "ITM-COAL-RUNTIME-001", 2),
    owner(
        "copper-material-runtime",
        "ITM-COPPER-MATERIAL-RUNTIME-001",
        3,
    ),
    owner(
        "enchanted-golden-apple-runtime",
        "ITM-ENCHANTED-GOLDEN-APPLE-RUNTIME-001",
        1,
    ),
    owner("gold-material-runtime", "ITM-GOLD-MATERIAL-RUNTIME-001", 3),
    owner("golden-apple-runtime", "ITM-GOLDEN-APPLE-RUNTIME-001", 1),
    owner("iron-material-runtime", "ITM-IRON-MATERIAL-RUNTIME-001", 3),
    owner(
        "netherite-material-runtime",
        "ITM-NETHERITE-MATERIAL-RUNTIME-001",
        2,
    ),
];
pub const PRISMARINE_OWNER: [ItemOwner; 1] = [owner(
    "prismarine-material-runtime",
    "ITM-PRISMARINE-MATERIAL-RUNTIME-001",
    2,
)];
pub const BLK_003_OWNERS: [ItemOwner; 9] = [
    owner("baked-potato-runtime", "ITM-BAKED-POTATO-RUNTIME-001", 1),
    owner("beef-runtime", "ITM-BEEF-RUNTIME-001", 2),
    owner("book-family-runtime", "ITM-BOOK-FAMILY-RUNTIME-001", 4),
    owner("chicken-runtime", "ITM-CHICKEN-RUNTIME-001", 2),
    owner("cookie-runtime", "ITM-COOKIE-RUNTIME-001", 1),
    owner("mutton-runtime", "ITM-MUTTON-RUNTIME-001", 2),
    owner("porkchop-runtime", "ITM-PORKCHOP-RUNTIME-001", 2),
    owner("pumpkin-pie-runtime", "ITM-PUMPKIN-PIE-RUNTIME-001", 1),
    owner(
        "rabbit-material-runtime",
        "ITM-RABBIT-MATERIAL-RUNTIME-001",
        3,
    ),
];

const fn owner(family: &'static str, slice: &'static str, expected_items: usize) -> ItemOwner {
    ItemOwner {
        family,
        slice,
        expected_items,
    }
}

pub fn owner_for_family(family: &str) -> Option<&'static ItemOwner> {
    OWNERS
        .iter()
        .chain(&PRISMARINE_OWNER)
        .chain(&BLK_003_OWNERS)
        .find(|owner| owner.family == family)
}

pub fn verify_owned_families(
    registry: &BundleRegistry,
) -> Result<OwnedItemCoverage, ItemCatalogError> {
    verify_partition(registry, &ItemKind::ALL, &OWNERS)
}

pub fn verify_prismarine_family(
    registry: &BundleRegistry,
) -> Result<OwnedItemCoverage, ItemCatalogError> {
    verify_partition(registry, &ItemKind::PRISMARINE, &PRISMARINE_OWNER)
}

pub fn verify_blk_003_families(
    registry: &BundleRegistry,
) -> Result<OwnedItemCoverage, ItemCatalogError> {
    verify_partition(registry, &ItemKind::BLK_003, &BLK_003_OWNERS)
}

fn verify_partition(
    registry: &BundleRegistry,
    items: &[ItemKind],
    owners: &[ItemOwner],
) -> Result<OwnedItemCoverage, ItemCatalogError> {
    if registry.name().to_string() != "minecraft:item" {
        return Err(ItemCatalogError::WrongRegistry {
            actual: registry.name().to_string(),
        });
    }

    let mut actual_by_family = BTreeMap::<&str, usize>::new();
    let mut found = 0_usize;
    for entry in registry.entries() {
        let actual_id = entry.persistent_id().to_string();
        let owned_family = owners
            .iter()
            .any(|owner| owner.family == entry.family().as_str());
        let Some(path) = actual_id.strip_prefix("minecraft:") else {
            if owned_family {
                return Err(ItemCatalogError::UnexpectedFamilyMember {
                    family: entry.family().as_str().to_owned(),
                    item: actual_id,
                });
            }
            continue;
        };
        let Some(expected) = ItemKind::from_path(path) else {
            if owned_family {
                return Err(ItemCatalogError::UnexpectedFamilyMember {
                    family: entry.family().as_str().to_owned(),
                    item: actual_id,
                });
            }
            continue;
        };
        if !items.contains(&expected) {
            continue;
        }
        if entry.family().as_str() != expected.family() {
            return Err(ItemCatalogError::Family {
                item: expected,
                expected: expected.family(),
                actual: entry.family().as_str().to_owned(),
            });
        }
        *actual_by_family.entry(expected.family()).or_default() += 1;
        found += 1;
    }

    if found != items.len() {
        return Err(ItemCatalogError::MissingOwnedItems {
            expected: items.len(),
            actual: found,
        });
    }
    for owner in owners {
        let actual = actual_by_family
            .get(owner.family)
            .copied()
            .unwrap_or_default();
        if actual != owner.expected_items {
            return Err(ItemCatalogError::FamilyCount {
                family: owner.family,
                expected: owner.expected_items,
                actual,
            });
        }
    }

    Ok(OwnedItemCoverage {
        families: owners.len(),
        items: found,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedItemCoverage {
    pub families: usize,
    pub items: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ItemCatalogError {
    #[error("expected minecraft:item registry, found {actual}")]
    WrongRegistry { actual: String },
    #[error("item {item:?} expected family {expected}, found {actual}")]
    Family {
        item: ItemKind,
        expected: &'static str,
        actual: String,
    },
    #[error("owned family {family} contains unexpected item {item}")]
    UnexpectedFamilyMember { family: String, item: String },
    #[error("owned item registry contains {actual} of {expected} required identities")]
    MissingOwnedItems { expected: usize, actual: usize },
    #[error("owned family {family} contains {actual} items, expected {expected}")]
    FamilyCount {
        family: &'static str,
        expected: usize,
        actual: usize,
    },
}
