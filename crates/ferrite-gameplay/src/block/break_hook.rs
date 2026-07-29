//! Exhaustive concrete block ownership for non-base break hooks.

use ferrite_simulation::random::DeterministicRng;
use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BreakHookKind {
    DragonEgg,
    NoteBlock,
    RedstoneOre,
    MovingPiston,
    Beehive,
    DoublePlant,
    Ice,
    TurtleEgg,
    Fire,
    Bed,
    CreakingHeart,
    DecoratedPot,
    Door,
    ShulkerBox,
    Tnt,
    Tripwire,
    PistonHead,
    ExperienceBlock,
    InfestedBlock,
    SculkCatalyst,
    SculkSensor,
    SculkShrieker,
    Spawner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookPoints(u8);

impl HookPoints {
    pub const ATTACK: Self = Self(1);
    pub const PLAYER_WILL_DESTROY: Self = Self(1 << 1);
    pub const DESTROY: Self = Self(1 << 2);
    pub const PLAYER_DESTROY: Self = Self(1 << 3);
    pub const SPAWN_AFTER_BREAK: Self = Self(1 << 4);

    pub const fn contains(self, point: Self) -> bool {
        self.0 & point.0 == point.0
    }

    const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

pub fn break_hook(path: &str) -> Option<BreakHookKind> {
    let hook = match path {
        "dragon_egg" => BreakHookKind::DragonEgg,
        "note_block" => BreakHookKind::NoteBlock,
        "redstone_ore" | "deepslate_redstone_ore" => BreakHookKind::RedstoneOre,
        "moving_piston" => BreakHookKind::MovingPiston,
        "bee_nest" | "beehive" => BreakHookKind::Beehive,
        path if is_double_plant(path) => BreakHookKind::DoublePlant,
        "ice" | "frosted_ice" => BreakHookKind::Ice,
        "turtle_egg" => BreakHookKind::TurtleEgg,
        "fire" | "soul_fire" => BreakHookKind::Fire,
        path if path.ends_with("_bed") => BreakHookKind::Bed,
        "creaking_heart" => BreakHookKind::CreakingHeart,
        "decorated_pot" => BreakHookKind::DecoratedPot,
        path if path.ends_with("_door") => BreakHookKind::Door,
        path if path == "shulker_box" || path.ends_with("_shulker_box") => {
            BreakHookKind::ShulkerBox
        }
        "tnt" => BreakHookKind::Tnt,
        "tripwire" => BreakHookKind::Tripwire,
        "piston_head" => BreakHookKind::PistonHead,
        path if experience_provider(path).is_some() => BreakHookKind::ExperienceBlock,
        path if path.starts_with("infested_") => BreakHookKind::InfestedBlock,
        "sculk_catalyst" => BreakHookKind::SculkCatalyst,
        "sculk_sensor" | "calibrated_sculk_sensor" => BreakHookKind::SculkSensor,
        "sculk_shrieker" => BreakHookKind::SculkShrieker,
        "spawner" => BreakHookKind::Spawner,
        _ => return None,
    };
    Some(hook)
}

pub const fn hook_points(kind: BreakHookKind) -> HookPoints {
    match kind {
        BreakHookKind::DragonEgg | BreakHookKind::NoteBlock => HookPoints::ATTACK,
        BreakHookKind::RedstoneOre => HookPoints::ATTACK.union(HookPoints::SPAWN_AFTER_BREAK),
        BreakHookKind::MovingPiston => HookPoints::DESTROY,
        BreakHookKind::Beehive => HookPoints::PLAYER_WILL_DESTROY.union(HookPoints::PLAYER_DESTROY),
        BreakHookKind::DoublePlant => {
            HookPoints::PLAYER_WILL_DESTROY.union(HookPoints::PLAYER_DESTROY)
        }
        BreakHookKind::Ice | BreakHookKind::TurtleEgg => HookPoints::PLAYER_DESTROY,
        BreakHookKind::ExperienceBlock
        | BreakHookKind::InfestedBlock
        | BreakHookKind::SculkCatalyst
        | BreakHookKind::SculkSensor
        | BreakHookKind::SculkShrieker
        | BreakHookKind::Spawner => HookPoints::SPAWN_AFTER_BREAK,
        BreakHookKind::Fire
        | BreakHookKind::Bed
        | BreakHookKind::CreakingHeart
        | BreakHookKind::DecoratedPot
        | BreakHookKind::Door
        | BreakHookKind::ShulkerBox
        | BreakHookKind::Tnt
        | BreakHookKind::Tripwire
        | BreakHookKind::PistonHead => HookPoints::PLAYER_WILL_DESTROY,
    }
}

fn is_double_plant(path: &str) -> bool {
    matches!(
        path,
        "tall_seagrass"
            | "sunflower"
            | "lilac"
            | "rose_bush"
            | "peony"
            | "tall_grass"
            | "large_fern"
            | "pitcher_crop"
            | "pitcher_plant"
            | "small_dripleaf"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperienceProvider {
    Constant(u8),
    Uniform { minimum: u8, maximum: u8 },
    SpawnerTriangular,
}

pub fn experience_provider(path: &str) -> Option<ExperienceProvider> {
    let provider = match path {
        "gold_ore"
        | "deepslate_gold_ore"
        | "iron_ore"
        | "deepslate_iron_ore"
        | "copper_ore"
        | "deepslate_copper_ore" => ExperienceProvider::Constant(0),
        "coal_ore" | "deepslate_coal_ore" => ExperienceProvider::Uniform {
            minimum: 0,
            maximum: 2,
        },
        "nether_gold_ore" => ExperienceProvider::Uniform {
            minimum: 0,
            maximum: 1,
        },
        "lapis_ore" | "deepslate_lapis_ore" | "nether_quartz_ore" => ExperienceProvider::Uniform {
            minimum: 2,
            maximum: 5,
        },
        "diamond_ore" | "deepslate_diamond_ore" | "emerald_ore" | "deepslate_emerald_ore" => {
            ExperienceProvider::Uniform {
                minimum: 3,
                maximum: 7,
            }
        }
        "sculk" => ExperienceProvider::Constant(1),
        _ => return None,
    };
    Some(provider)
}

pub fn break_experience_provider(path: &str) -> Option<ExperienceProvider> {
    experience_provider(path).or(match path {
        "redstone_ore" | "deepslate_redstone_ore" => Some(ExperienceProvider::Uniform {
            minimum: 1,
            maximum: 5,
        }),
        "sculk_catalyst" | "sculk_sensor" | "calibrated_sculk_sensor" | "sculk_shrieker" => {
            Some(ExperienceProvider::Constant(5))
        }
        "spawner" => Some(ExperienceProvider::SpawnerTriangular),
        _ => None,
    })
}

pub fn sample_experience(provider: ExperienceProvider, random: &mut DeterministicRng) -> u8 {
    match provider {
        ExperienceProvider::Constant(value) => value,
        ExperienceProvider::Uniform { minimum, maximum } => {
            let width = u64::from(maximum - minimum + 1);
            let upper = NonZeroU64::new(width).expect("experience range is nonempty");
            minimum + random.uniform_u64(upper) as u8
        }
        ExperienceProvider::SpawnerTriangular => 15 + bounded_15(random) + bounded_15(random),
    }
}

fn bounded_15(random: &mut DeterministicRng) -> u8 {
    let upper = NonZeroU64::new(15).expect("15 is nonzero");
    random.uniform_u64(upper) as u8
}
