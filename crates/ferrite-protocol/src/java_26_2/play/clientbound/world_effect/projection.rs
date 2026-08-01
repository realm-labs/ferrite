use crate::java_26_2::play::clientbound::world_effect::packet::LevelEvent;
use crate::java_26_2::value::identifier::Identifier;

const BLOCK_STATE_COUNT: i32 = 32_366;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlameKind {
    Flame,
    SoulFireFlame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtinguishKind {
    BlockFire,
    EntityFire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockStateSelection {
    RawId(i32),
    AirFallback,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LevelEventData {
    Ignored,
    Extinguish(ExtinguishKind),
    JukeboxSong(Identifier),
    Composter { successful: bool },
    BoneMeal { count: i32 },
    DirectionalSmoke(Direction),
    BlockState(BlockStateSelection),
    PotionColor([u8; 3]),
    DragonBreath { play_explosion_sound: bool },
    GrowthParticles { count: i32 },
    SmashAttack { strength: i32 },
    ElectricSpark { axis: Option<Axis> },
    SculkCharge { count: i32, face_mask: u8 },
    TrialFlame(FlameKind),
    DetectionParticles { loop_bound: i32 },
    VaultFlame(FlameKind),
    OminousActivation { volume: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalLevelEffect {
    WitherSpawn,
    EnderDragonDeath,
    EndPortalSpawn,
}

impl GlobalLevelEffect {
    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::WitherSpawn => "minecraft:wither_spawn",
            Self::EnderDragonDeath => "minecraft:ender_dragon_death",
            Self::EndPortalSpawn => "minecraft:end_portal_spawn",
        }
    }

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::WitherSpawn => 1023,
            Self::EnderDragonDeath => 1028,
            Self::EndPortalSpawn => 1038,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelEventFault {
    TrialFlameIndexOutOfBounds,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LevelEventProjection {
    NoOp,
    Local {
        identity: &'static str,
        data: LevelEventData,
    },
    Global(GlobalLevelEffect),
    HandlerFault {
        identity: &'static str,
        fault: LevelEventFault,
        retained_prefix: bool,
    },
}

#[must_use]
pub fn project_level_event(
    packet: &LevelEvent,
    jukebox_songs: &[Identifier],
) -> LevelEventProjection {
    if packet.global {
        return match packet.event_type {
            1023 => LevelEventProjection::Global(GlobalLevelEffect::WitherSpawn),
            1028 => LevelEventProjection::Global(GlobalLevelEffect::EnderDragonDeath),
            1038 => LevelEventProjection::Global(GlobalLevelEffect::EndPortalSpawn),
            _ => LevelEventProjection::NoOp,
        };
    }

    let Some(identity) = local_identity(packet.event_type) else {
        return LevelEventProjection::NoOp;
    };
    let data = match packet.event_type {
        1009 => match packet.data {
            0 => LevelEventData::Extinguish(ExtinguishKind::BlockFire),
            1 => LevelEventData::Extinguish(ExtinguishKind::EntityFire),
            _ => return LevelEventProjection::NoOp,
        },
        1010 => {
            let Ok(raw_id) = usize::try_from(packet.data) else {
                return LevelEventProjection::NoOp;
            };
            let Some(song) = jukebox_songs.get(raw_id) else {
                return LevelEventProjection::NoOp;
            };
            LevelEventData::JukeboxSong(song.clone())
        }
        1500 => LevelEventData::Composter {
            successful: packet.data > 0,
        },
        1505 => LevelEventData::BoneMeal { count: packet.data },
        2000 | 2010 => LevelEventData::DirectionalSmoke(direction(packet.data)),
        2001 | 3008 => LevelEventData::BlockState(block_state(packet.data)),
        2002 | 2007 => LevelEventData::PotionColor([
            ((packet.data >> 16) & 0xff) as u8,
            ((packet.data >> 8) & 0xff) as u8,
            (packet.data & 0xff) as u8,
        ]),
        2006 => LevelEventData::DragonBreath {
            play_explosion_sound: packet.data == 1,
        },
        2011 | 2012 => LevelEventData::GrowthParticles { count: packet.data },
        2013 => LevelEventData::SmashAttack {
            strength: packet.data,
        },
        3002 => LevelEventData::ElectricSpark {
            axis: match packet.data {
                0 => Some(Axis::X),
                1 => Some(Axis::Y),
                2 => Some(Axis::Z),
                _ => None,
            },
        },
        3006 => {
            let count = packet.data >> 6;
            LevelEventData::SculkCharge {
                count,
                face_mask: if count > 0 {
                    (packet.data & 0x3f) as u8
                } else {
                    0
                },
            }
        }
        3011 | 3012 | 3021 => match trial_flame(packet.data) {
            Ok(flame) => LevelEventData::TrialFlame(flame),
            Err(fault) => {
                return LevelEventProjection::HandlerFault {
                    identity,
                    fault,
                    retained_prefix: matches!(packet.event_type, 3012 | 3021),
                };
            }
        },
        3013 | 3019 => LevelEventData::DetectionParticles {
            loop_bound: 30_i32.wrapping_add(packet.data.min(10).wrapping_mul(5)),
        },
        3015 | 3016 => LevelEventData::VaultFlame(if packet.data == 0 {
            FlameKind::Flame
        } else {
            FlameKind::SoulFireFlame
        }),
        3020 => LevelEventData::OminousActivation {
            volume: if packet.data == 0 { 0.3 } else { 1.0 },
        },
        _ => LevelEventData::Ignored,
    };
    LevelEventProjection::Local { identity, data }
}

fn direction(data: i32) -> Direction {
    match (data % 6).abs() {
        0 => Direction::Down,
        1 => Direction::Up,
        2 => Direction::North,
        3 => Direction::South,
        4 => Direction::West,
        _ => Direction::East,
    }
}

fn block_state(raw_id: i32) -> BlockStateSelection {
    if (0..BLOCK_STATE_COUNT).contains(&raw_id) {
        BlockStateSelection::RawId(raw_id)
    } else {
        BlockStateSelection::AirFallback
    }
}

fn trial_flame(data: i32) -> Result<FlameKind, LevelEventFault> {
    match data {
        1 => Ok(FlameKind::SoulFireFlame),
        2 => Err(LevelEventFault::TrialFlameIndexOutOfBounds),
        _ => Ok(FlameKind::Flame),
    }
}

#[must_use]
pub fn local_identity(event_type: i32) -> Option<&'static str> {
    LOCAL_EVENT_IDENTITIES
        .iter()
        .find_map(|(id, identity)| (*id == event_type).then_some(*identity))
}

#[must_use]
pub fn local_event_id(identity: &Identifier) -> Option<i32> {
    if identity.namespace() != "minecraft" {
        return None;
    }
    LOCAL_EVENT_IDENTITIES.iter().find_map(|(id, candidate)| {
        candidate
            .strip_prefix("minecraft:")
            .is_some_and(|path| path == identity.path())
            .then_some(*id)
    })
}

pub const LOCAL_EVENT_IDENTITIES: [(i32, &str); 80] = [
    (1000, "minecraft:dispenser_dispense"),
    (1001, "minecraft:dispenser_fail"),
    (1002, "minecraft:dispenser_launch"),
    (1004, "minecraft:firework_shoot"),
    (1009, "minecraft:extinguish_fire"),
    (1010, "minecraft:jukebox_play"),
    (1011, "minecraft:jukebox_stop"),
    (1015, "minecraft:ghast_warn"),
    (1016, "minecraft:ghast_shoot"),
    (1017, "minecraft:ender_dragon_shoot"),
    (1018, "minecraft:blaze_shoot"),
    (1019, "minecraft:zombie_attack_wooden_door"),
    (1020, "minecraft:zombie_attack_iron_door"),
    (1021, "minecraft:zombie_break_wooden_door"),
    (1022, "minecraft:wither_break_block"),
    (1024, "minecraft:wither_shoot"),
    (1025, "minecraft:bat_takeoff"),
    (1026, "minecraft:zombie_infect"),
    (1027, "minecraft:zombie_villager_converted"),
    (1029, "minecraft:anvil_destroy"),
    (1030, "minecraft:anvil_use"),
    (1031, "minecraft:anvil_land"),
    (1032, "minecraft:portal_travel"),
    (1033, "minecraft:chorus_flower_grow"),
    (1034, "minecraft:chorus_flower_death"),
    (1035, "minecraft:brewing_stand_brew"),
    (1039, "minecraft:phantom_bite"),
    (1040, "minecraft:zombie_to_drowned"),
    (1041, "minecraft:husk_to_zombie"),
    (1042, "minecraft:grindstone_use"),
    (1043, "minecraft:book_page_turn"),
    (1044, "minecraft:smithing_table_use"),
    (1045, "minecraft:pointed_dripstone_land"),
    (1046, "minecraft:drip_lava_into_cauldron"),
    (1047, "minecraft:drip_water_into_cauldron"),
    (1048, "minecraft:skeleton_to_stray"),
    (1049, "minecraft:crafter_craft"),
    (1050, "minecraft:crafter_fail"),
    (1051, "minecraft:wind_charge_throw"),
    (1052, "minecraft:sulfur_spike_land"),
    (1500, "minecraft:composter_fill"),
    (1501, "minecraft:lava_extinguish"),
    (1502, "minecraft:redstone_torch_burnout"),
    (1503, "minecraft:end_portal_frame_fill"),
    (1504, "minecraft:pointed_dripstone_drip"),
    (1505, "minecraft:bone_meal_growth"),
    (2000, "minecraft:directional_smoke"),
    (2001, "minecraft:block_destroy"),
    (2002, "minecraft:splash_potion"),
    (2003, "minecraft:ender_eye_break"),
    (2004, "minecraft:mob_spawn"),
    (2006, "minecraft:dragon_breath"),
    (2007, "minecraft:instant_splash_potion"),
    (2008, "minecraft:block_explosion"),
    (2009, "minecraft:evaporate"),
    (2010, "minecraft:directional_white_smoke"),
    (2011, "minecraft:bee_growth"),
    (2012, "minecraft:turtle_egg_placement"),
    (2013, "minecraft:smash_attack"),
    (3000, "minecraft:end_gateway_spawn"),
    (3001, "minecraft:ender_dragon_growl"),
    (3002, "minecraft:electric_spark"),
    (3003, "minecraft:wax_on"),
    (3004, "minecraft:wax_off"),
    (3005, "minecraft:scrape"),
    (3006, "minecraft:sculk_charge"),
    (3007, "minecraft:sculk_shriek"),
    (3008, "minecraft:brush_complete"),
    (3009, "minecraft:egg_crack"),
    (3011, "minecraft:trial_spawner_spawn"),
    (3012, "minecraft:trial_spawner_spawn_mob"),
    (3013, "minecraft:trial_spawner_detect_player"),
    (3014, "minecraft:trial_spawner_eject_item"),
    (3015, "minecraft:vault_activate"),
    (3016, "minecraft:vault_deactivate"),
    (3017, "minecraft:vault_eject_item"),
    (3018, "minecraft:cobweb_place"),
    (3019, "minecraft:trial_spawner_detect_ominous_player"),
    (3020, "minecraft:trial_spawner_ominous_activate"),
    (3021, "minecraft:trial_spawner_spawn_item"),
];
