//! Scheduled ordinary-fire, soul-fire, portal, TNT, and contact decisions.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

pub const FIRE_SCHEDULE_BASE: u8 = 30;
pub const FIRE_SCHEDULE_SPREAD: u8 = 10;
pub const DEFAULT_SPREAD_RADIUS: i32 = 128;
pub const UNLIMITED_SPREAD_RADIUS: i32 = -1;
pub const MAX_FIRE_AGE: u8 = 15;
pub const AGE_WRITE_FLAGS: u16 = 260;
pub const SPREAD_WRITE_FLAGS: u16 = 3;
pub const REGISTERED_FIRE_ODDS_COUNT: usize = 207;
pub const INCREASED_BURNOUT_BIOME_COUNT: usize = 8;
pub const SPATIAL_CANDIDATE_COUNT: usize = 53;
pub const SOUL_FIRE_CAN_BURN: bool = true;
pub const SOUL_FIRE_HAS_SCHEDULED_CALLBACK: bool = false;
pub const DO_FIRE_TICK_RULE_PRESENT: bool = false;
pub const REPLACEMENT_AGE_DRAW_BOUND: u8 = 5;
pub const PORTAL_AXIS_DRAW_BOUND: u8 = 2;
pub const CONTACT_PLAYER_DRAW_ORIGIN: u8 = 1;
pub const CONTACT_PLAYER_DRAW_BOUND: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireOdds {
    pub ignite: u8,
    pub burn: u8,
}

impl FireOdds {
    pub const fn new(ignite: u8, burn: u8) -> Self {
        Self { ignite, burn }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireRegistration {
    pub path: &'static str,
    pub odds: FireOdds,
}

const fn registration(path: &'static str, ignite: u8, burn: u8) -> FireRegistration {
    FireRegistration {
        path,
        odds: FireOdds::new(ignite, burn),
    }
}

pub const FIRE_ODDS_TABLE: [FireRegistration; REGISTERED_FIRE_ODDS_COUNT] = [
    registration("oak_log", 5, 5),
    registration("spruce_log", 5, 5),
    registration("birch_log", 5, 5),
    registration("jungle_log", 5, 5),
    registration("acacia_log", 5, 5),
    registration("cherry_log", 5, 5),
    registration("pale_oak_log", 5, 5),
    registration("dark_oak_log", 5, 5),
    registration("mangrove_log", 5, 5),
    registration("bamboo_block", 5, 5),
    registration("stripped_oak_log", 5, 5),
    registration("stripped_spruce_log", 5, 5),
    registration("stripped_birch_log", 5, 5),
    registration("stripped_jungle_log", 5, 5),
    registration("stripped_acacia_log", 5, 5),
    registration("stripped_cherry_log", 5, 5),
    registration("stripped_pale_oak_log", 5, 5),
    registration("stripped_dark_oak_log", 5, 5),
    registration("stripped_mangrove_log", 5, 5),
    registration("oak_wood", 5, 5),
    registration("spruce_wood", 5, 5),
    registration("birch_wood", 5, 5),
    registration("jungle_wood", 5, 5),
    registration("acacia_wood", 5, 5),
    registration("cherry_wood", 5, 5),
    registration("pale_oak_wood", 5, 5),
    registration("dark_oak_wood", 5, 5),
    registration("mangrove_wood", 5, 5),
    registration("stripped_oak_wood", 5, 5),
    registration("stripped_spruce_wood", 5, 5),
    registration("stripped_birch_wood", 5, 5),
    registration("stripped_jungle_wood", 5, 5),
    registration("stripped_acacia_wood", 5, 5),
    registration("stripped_cherry_wood", 5, 5),
    registration("stripped_pale_oak_wood", 5, 5),
    registration("stripped_dark_oak_wood", 5, 5),
    registration("stripped_mangrove_wood", 5, 5),
    registration("stripped_bamboo_block", 5, 5),
    registration("coal_block", 5, 5),
    registration("oak_planks", 5, 20),
    registration("spruce_planks", 5, 20),
    registration("birch_planks", 5, 20),
    registration("jungle_planks", 5, 20),
    registration("acacia_planks", 5, 20),
    registration("cherry_planks", 5, 20),
    registration("dark_oak_planks", 5, 20),
    registration("pale_oak_planks", 5, 20),
    registration("mangrove_planks", 5, 20),
    registration("bamboo_planks", 5, 20),
    registration("bamboo_mosaic", 5, 20),
    registration("oak_slab", 5, 20),
    registration("spruce_slab", 5, 20),
    registration("birch_slab", 5, 20),
    registration("jungle_slab", 5, 20),
    registration("acacia_slab", 5, 20),
    registration("cherry_slab", 5, 20),
    registration("dark_oak_slab", 5, 20),
    registration("pale_oak_slab", 5, 20),
    registration("mangrove_slab", 5, 20),
    registration("bamboo_slab", 5, 20),
    registration("bamboo_mosaic_slab", 5, 20),
    registration("oak_stairs", 5, 20),
    registration("spruce_stairs", 5, 20),
    registration("birch_stairs", 5, 20),
    registration("jungle_stairs", 5, 20),
    registration("acacia_stairs", 5, 20),
    registration("cherry_stairs", 5, 20),
    registration("dark_oak_stairs", 5, 20),
    registration("pale_oak_stairs", 5, 20),
    registration("mangrove_stairs", 5, 20),
    registration("bamboo_stairs", 5, 20),
    registration("bamboo_mosaic_stairs", 5, 20),
    registration("oak_fence", 5, 20),
    registration("spruce_fence", 5, 20),
    registration("birch_fence", 5, 20),
    registration("jungle_fence", 5, 20),
    registration("acacia_fence", 5, 20),
    registration("cherry_fence", 5, 20),
    registration("dark_oak_fence", 5, 20),
    registration("pale_oak_fence", 5, 20),
    registration("mangrove_fence", 5, 20),
    registration("bamboo_fence", 5, 20),
    registration("oak_fence_gate", 5, 20),
    registration("spruce_fence_gate", 5, 20),
    registration("birch_fence_gate", 5, 20),
    registration("jungle_fence_gate", 5, 20),
    registration("acacia_fence_gate", 5, 20),
    registration("cherry_fence_gate", 5, 20),
    registration("dark_oak_fence_gate", 5, 20),
    registration("pale_oak_fence_gate", 5, 20),
    registration("mangrove_fence_gate", 5, 20),
    registration("bamboo_fence_gate", 5, 20),
    registration("mangrove_roots", 5, 20),
    registration("composter", 5, 20),
    registration("beehive", 5, 20),
    registration("pale_moss_block", 5, 100),
    registration("pale_moss_carpet", 5, 100),
    registration("pale_hanging_moss", 5, 100),
    registration("target", 15, 20),
    registration("cave_vines", 15, 60),
    registration("cave_vines_plant", 15, 60),
    registration("tnt", 15, 100),
    registration("vine", 15, 100),
    registration("glow_lichen", 15, 100),
    registration("bookshelf", 30, 20),
    registration("lectern", 30, 20),
    registration("bee_nest", 30, 20),
    registration("acacia_shelf", 30, 20),
    registration("bamboo_shelf", 30, 20),
    registration("birch_shelf", 30, 20),
    registration("cherry_shelf", 30, 20),
    registration("dark_oak_shelf", 30, 20),
    registration("jungle_shelf", 30, 20),
    registration("mangrove_shelf", 30, 20),
    registration("oak_shelf", 30, 20),
    registration("pale_oak_shelf", 30, 20),
    registration("spruce_shelf", 30, 20),
    registration("oak_leaves", 30, 60),
    registration("spruce_leaves", 30, 60),
    registration("birch_leaves", 30, 60),
    registration("jungle_leaves", 30, 60),
    registration("acacia_leaves", 30, 60),
    registration("cherry_leaves", 30, 60),
    registration("pale_oak_leaves", 30, 60),
    registration("dark_oak_leaves", 30, 60),
    registration("mangrove_leaves", 30, 60),
    registration("azalea_leaves", 30, 60),
    registration("flowering_azalea_leaves", 30, 60),
    registration("azalea", 30, 60),
    registration("flowering_azalea", 30, 60),
    registration("hanging_roots", 30, 60),
    registration("dried_kelp_block", 30, 60),
    registration("white_wool", 30, 60),
    registration("orange_wool", 30, 60),
    registration("magenta_wool", 30, 60),
    registration("light_blue_wool", 30, 60),
    registration("yellow_wool", 30, 60),
    registration("lime_wool", 30, 60),
    registration("pink_wool", 30, 60),
    registration("gray_wool", 30, 60),
    registration("light_gray_wool", 30, 60),
    registration("cyan_wool", 30, 60),
    registration("purple_wool", 30, 60),
    registration("blue_wool", 30, 60),
    registration("brown_wool", 30, 60),
    registration("green_wool", 30, 60),
    registration("red_wool", 30, 60),
    registration("black_wool", 30, 60),
    registration("hay_block", 60, 20),
    registration("white_carpet", 60, 20),
    registration("orange_carpet", 60, 20),
    registration("magenta_carpet", 60, 20),
    registration("light_blue_carpet", 60, 20),
    registration("yellow_carpet", 60, 20),
    registration("lime_carpet", 60, 20),
    registration("pink_carpet", 60, 20),
    registration("gray_carpet", 60, 20),
    registration("light_gray_carpet", 60, 20),
    registration("cyan_carpet", 60, 20),
    registration("purple_carpet", 60, 20),
    registration("blue_carpet", 60, 20),
    registration("brown_carpet", 60, 20),
    registration("green_carpet", 60, 20),
    registration("red_carpet", 60, 20),
    registration("black_carpet", 60, 20),
    registration("bamboo", 60, 60),
    registration("scaffolding", 60, 60),
    registration("short_grass", 60, 100),
    registration("fern", 60, 100),
    registration("dead_bush", 60, 100),
    registration("short_dry_grass", 60, 100),
    registration("tall_dry_grass", 60, 100),
    registration("sunflower", 60, 100),
    registration("lilac", 60, 100),
    registration("rose_bush", 60, 100),
    registration("peony", 60, 100),
    registration("tall_grass", 60, 100),
    registration("large_fern", 60, 100),
    registration("dandelion", 60, 100),
    registration("golden_dandelion", 60, 100),
    registration("poppy", 60, 100),
    registration("open_eyeblossom", 60, 100),
    registration("closed_eyeblossom", 60, 100),
    registration("blue_orchid", 60, 100),
    registration("allium", 60, 100),
    registration("azure_bluet", 60, 100),
    registration("red_tulip", 60, 100),
    registration("orange_tulip", 60, 100),
    registration("white_tulip", 60, 100),
    registration("pink_tulip", 60, 100),
    registration("oxeye_daisy", 60, 100),
    registration("cornflower", 60, 100),
    registration("lily_of_the_valley", 60, 100),
    registration("torchflower", 60, 100),
    registration("pitcher_plant", 60, 100),
    registration("wither_rose", 60, 100),
    registration("pink_petals", 60, 100),
    registration("wildflowers", 60, 100),
    registration("leaf_litter", 60, 100),
    registration("cactus_flower", 60, 100),
    registration("sweet_berry_bush", 60, 100),
    registration("spore_blossom", 60, 100),
    registration("big_dripleaf", 60, 100),
    registration("big_dripleaf_stem", 60, 100),
    registration("small_dripleaf", 60, 100),
    registration("firefly_bush", 60, 100),
    registration("bush", 60, 100),
];

pub fn fire_odds(path: &str, waterlogged: bool) -> FireOdds {
    if waterlogged {
        return FireOdds::new(0, 0);
    }
    FIRE_ODDS_TABLE
        .iter()
        .find(|entry| entry.path == path)
        .map_or(FireOdds::new(0, 0), |entry| entry.odds)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseFireKind {
    Ordinary,
    Soul,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrdinaryFireState {
    pub age: u8,
    pub up: bool,
    pub north: bool,
    pub south: bool,
    pub west: bool,
    pub east: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseFireState {
    Air,
    Ordinary(OrdinaryFireState),
    Soul,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireNeighbourOdds {
    pub below: u8,
    pub above: u8,
    pub north: u8,
    pub south: u8,
    pub west: u8,
    pub east: u8,
}

impl FireNeighbourOdds {
    pub const fn any(self) -> bool {
        self.below > 0
            || self.above > 0
            || self.north > 0
            || self.south > 0
            || self.west > 0
            || self.east > 0
    }
}

pub const fn ordinary_fire_state(
    age: u8,
    below_sturdy: bool,
    neighbours: FireNeighbourOdds,
) -> OrdinaryFireState {
    if below_sturdy || neighbours.below > 0 {
        return OrdinaryFireState {
            age,
            up: false,
            north: false,
            south: false,
            west: false,
            east: false,
        };
    }
    OrdinaryFireState {
        age,
        up: neighbours.above > 0,
        north: neighbours.north > 0,
        south: neighbours.south > 0,
        west: neighbours.west > 0,
        east: neighbours.east > 0,
    }
}

pub const fn ordinary_survives(below_sturdy: bool, neighbours: FireNeighbourOdds) -> bool {
    below_sturdy || neighbours.any()
}

pub const fn selected_fire_state(
    age: u8,
    soul_base: bool,
    below_sturdy: bool,
    neighbours: FireNeighbourOdds,
) -> BaseFireState {
    if soul_base {
        BaseFireState::Soul
    } else if ordinary_survives(below_sturdy, neighbours) {
        BaseFireState::Ordinary(ordinary_fire_state(age, below_sturdy, neighbours))
    } else {
        BaseFireState::Air
    }
}

pub const fn soul_shape_update(soul_base: bool) -> BaseFireState {
    if soul_base {
        BaseFireState::Soul
    } else {
        BaseFireState::Air
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalDimension {
    Overworld,
    Nether,
    Other,
}

impl PortalDimension {
    pub const fn admits_portal(self) -> bool {
        matches!(self, Self::Overworld | Self::Nether)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalAxis {
    X,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalPreference {
    pub axes: [PortalAxis; 2],
    pub vertical_draw_consumed: bool,
}

pub const fn preferred_portal_axes(
    clicked_face: Direction,
    vertical_axis_draw: u8,
) -> PortalPreference {
    let preferred = match clicked_face {
        Direction::North | Direction::South => PortalAxis::X,
        Direction::East | Direction::West => PortalAxis::Z,
        Direction::Down | Direction::Up => {
            if vertical_axis_draw == 0 {
                PortalAxis::X
            } else {
                PortalAxis::Z
            }
        }
    };
    PortalPreference {
        axes: [
            preferred,
            match preferred {
                PortalAxis::X => PortalAxis::Z,
                PortalAxis::Z => PortalAxis::X,
            },
        ],
        vertical_draw_consumed: matches!(clicked_face, Direction::Down | Direction::Up),
    }
}

pub const fn fire_schedule_delay(draw: u8) -> u8 {
    FIRE_SCHEDULE_BASE + draw
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementPlan {
    pub create_portal: Option<PortalAxis>,
    pub remove_without_drops: bool,
    pub ordinary_schedule_delay: Option<u8>,
}

pub const fn placement_plan(
    old_same_block: bool,
    dimension: PortalDimension,
    portal_x: bool,
    portal_z: bool,
    selected_survives: bool,
    ordinary_fire: bool,
    delay_draw: u8,
) -> PlacementPlan {
    if old_same_block {
        return PlacementPlan {
            create_portal: None,
            remove_without_drops: false,
            ordinary_schedule_delay: if ordinary_fire {
                Some(fire_schedule_delay(delay_draw))
            } else {
                None
            },
        };
    }
    let portal = if dimension.admits_portal() && portal_x {
        Some(PortalAxis::X)
    } else if dimension.admits_portal() && portal_z {
        Some(PortalAxis::Z)
    } else {
        None
    };
    PlacementPlan {
        create_portal: portal,
        remove_without_drops: portal.is_none() && !selected_survives,
        ordinary_schedule_delay: if ordinary_fire {
            Some(fire_schedule_delay(delay_draw))
        } else {
            None
        },
    }
}

pub const fn can_be_placed(
    target_is_air: bool,
    selected_survives: bool,
    portal_dimension: PortalDimension,
    has_obsidian_neighbour: bool,
    portal_candidate: bool,
) -> bool {
    target_is_air
        && (selected_survives
            || (portal_dimension.admits_portal() && has_obsidian_neighbour && portal_candidate))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FirePlayer {
    pub position: [f64; 3],
    pub spectator: bool,
}

pub fn near_player_admits(position: BlockPos, radius: i32, players: &[FirePlayer]) -> bool {
    if radius == UNLIMITED_SPREAD_RADIUS {
        return true;
    }
    let radius_squared = f64::from(radius).powi(2);
    players.iter().any(|player| {
        if player.spectator {
            return false;
        }
        let dx = player.position[0] - f64::from(position.x);
        let dy = player.position[1] - f64::from(position.y);
        let dz = player.position[2] - f64::from(position.z);
        dx * dx + dy * dy + dz * dz < radius_squared
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireCallbackStage {
    Reschedule,
    NearbyPlayer,
    SurvivalRemoval,
    Infiniburn,
    Rain,
    Age,
    SelfRemoval,
    IncreasedBurnout,
    DirectFuel,
    SpatialSpread,
}

pub const FIRE_CALLBACK_ORDER: [FireCallbackStage; 10] = [
    FireCallbackStage::Reschedule,
    FireCallbackStage::NearbyPlayer,
    FireCallbackStage::SurvivalRemoval,
    FireCallbackStage::Infiniburn,
    FireCallbackStage::Rain,
    FireCallbackStage::Age,
    FireCallbackStage::SelfRemoval,
    FireCallbackStage::IncreasedBurnout,
    FireCallbackStage::DirectFuel,
    FireCallbackStage::SpatialSpread,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurvivalRemovalPlan {
    pub remove_without_drops: bool,
    pub continue_after_removal_attempt: bool,
}

pub const fn survival_removal(captured_survives: bool) -> SurvivalRemovalPlan {
    SurvivalRemovalPlan {
        remove_without_drops: !captured_survives,
        continue_after_removal_attempt: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfiniburnSet {
    Overworld,
    Nether,
    End,
}

pub fn is_infiniburn(set: InfiniburnSet, below_path: &str) -> bool {
    matches!(below_path, "netherrack" | "magma_block")
        || matches!(set, InfiniburnSet::End) && below_path == "bedrock"
}

pub const INCREASED_BURNOUT_BIOMES: [&str; INCREASED_BURNOUT_BIOME_COUNT] = [
    "bamboo_jungle",
    "frozen_peaks",
    "jagged_peaks",
    "jungle",
    "mangrove_swamp",
    "mushroom_fields",
    "snowy_slopes",
    "swamp",
];

pub fn increased_burnout(path: &str) -> bool {
    INCREASED_BURNOUT_BIOMES.contains(&path)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RainProbe {
    Current,
    West,
    East,
    North,
    South,
}

pub const RAIN_PROBE_ORDER: [RainProbe; 5] = [
    RainProbe::Current,
    RainProbe::West,
    RainProbe::East,
    RainProbe::North,
    RainProbe::South,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NearRainResult {
    pub near: bool,
    pub probes: u8,
}

pub fn near_rain(probes: [bool; 5]) -> NearRainResult {
    for (index, raining) in probes.into_iter().enumerate() {
        if raining {
            return NearRainResult {
                near: true,
                probes: index as u8 + 1,
            };
        }
    }
    NearRainResult {
        near: false,
        probes: 5,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RainExtinguishPlan {
    pub draw_consumed: bool,
    pub remove_and_return: bool,
}

pub fn rain_extinguish(
    infiniburn: bool,
    active_rain: bool,
    near: bool,
    age: u8,
    draw: f32,
) -> RainExtinguishPlan {
    let draw_consumed = !infiniburn && active_rain && near;
    RainExtinguishPlan {
        draw_consumed,
        remove_and_return: draw_consumed && draw < 0.2 + f32::from(age) * 0.03,
    }
}

pub const fn next_fire_age(age: u8, draw: u8) -> u8 {
    clamped_age_add(age, draw / 2)
}

const fn clamped_age_add(age: u8, increment: u8) -> u8 {
    let next = age.saturating_add(increment);
    if next < MAX_FIRE_AGE {
        next
    } else {
        MAX_FIRE_AGE
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfRemoval {
    Continue,
    RemoveAndReturn,
    Return,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfRemovalPlan {
    pub age_fifteen_draw_consumed: bool,
    pub outcome: SelfRemoval,
}

pub const fn self_removal(
    infiniburn: bool,
    adjacent_fuel: bool,
    below_sturdy: bool,
    below_fuel: bool,
    captured_age: u8,
    age_fifteen_draw: u8,
) -> SelfRemovalPlan {
    if infiniburn {
        return SelfRemovalPlan {
            age_fifteen_draw_consumed: false,
            outcome: SelfRemoval::Continue,
        };
    }
    if !adjacent_fuel {
        return SelfRemovalPlan {
            age_fifteen_draw_consumed: false,
            outcome: if !below_sturdy || captured_age > 3 {
                SelfRemoval::RemoveAndReturn
            } else {
                SelfRemoval::Return
            },
        };
    }
    let draw_consumed = captured_age == 15;
    SelfRemovalPlan {
        age_fifteen_draw_consumed: draw_consumed,
        outcome: if draw_consumed && age_fifteen_draw == 0 && !below_fuel {
            SelfRemoval::RemoveAndReturn
        } else {
            SelfRemoval::Continue
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectDirection {
    East,
    West,
    Below,
    Above,
    North,
    South,
}

pub const DIRECT_BURN_ORDER: [DirectDirection; 6] = [
    DirectDirection::East,
    DirectDirection::West,
    DirectDirection::Below,
    DirectDirection::Above,
    DirectDirection::North,
    DirectDirection::South,
];

pub const fn direct_denominator(direction: DirectDirection, increased: bool) -> u16 {
    let base = match direction {
        DirectDirection::Below | DirectDirection::Above => 250,
        _ => 300,
    };
    if increased { base - 50 } else { base }
}

pub const fn direct_age_gate_bound(captured_age: u8) -> u8 {
    captured_age + 10
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuelMutation {
    None,
    ReplaceWithFire { kind: BaseFireKind, age: u8 },
    RemoveWithoutDrops,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectBurnPlan {
    pub first_draw_consumed: bool,
    pub age_gate_draw_consumed: bool,
    pub rain_queried: bool,
    pub replacement_age_draw_consumed: bool,
    pub mutation: FuelMutation,
    pub prime_tnt_after_mutation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectBurnProbe {
    pub burn_odds: u8,
    pub denominator_draw: u16,
    pub captured_age: u8,
    pub age_gate_draw: u8,
    pub target_raining: bool,
    pub replacement_age_draw: u8,
    pub replacement_kind: BaseFireKind,
    pub captured_target_is_tnt: bool,
}

pub const fn direct_burn(probe: DirectBurnProbe) -> DirectBurnPlan {
    if probe.denominator_draw >= probe.burn_odds as u16 {
        return DirectBurnPlan {
            first_draw_consumed: true,
            age_gate_draw_consumed: false,
            rain_queried: false,
            replacement_age_draw_consumed: false,
            mutation: FuelMutation::None,
            prime_tnt_after_mutation: false,
        };
    }
    let replace = probe.age_gate_draw < 5 && !probe.target_raining;
    DirectBurnPlan {
        first_draw_consumed: true,
        age_gate_draw_consumed: true,
        rain_queried: probe.age_gate_draw < 5,
        replacement_age_draw_consumed: replace,
        mutation: if replace {
            FuelMutation::ReplaceWithFire {
                kind: probe.replacement_kind,
                age: clamped_age_add(probe.captured_age, probe.replacement_age_draw / 4),
            }
        } else {
            FuelMutation::RemoveWithoutDrops
        },
        prime_tnt_after_mutation: probe.captured_target_is_tnt,
    }
}

pub fn spatial_offsets() -> Vec<[i8; 3]> {
    let mut offsets = Vec::with_capacity(SPATIAL_CANDIDATE_COUNT);
    for x in -1..=1 {
        for z in -1..=1 {
            for y in -1..=4 {
                if x != 0 || y != 0 || z != 0 {
                    offsets.push([x, y, z]);
                }
            }
        }
    }
    offsets
}

pub const fn spatial_denominator(y: i8) -> u16 {
    if y <= 1 { 100 } else { y as u16 * 100 }
}

pub const fn spread_threshold(
    encouragement: u8,
    difficulty_id: u8,
    captured_age: u8,
    increased: bool,
) -> u16 {
    let base = (encouragement as u16 + 40 + difficulty_id as u16 * 7) / (captured_age as u16 + 30);
    if increased { base / 2 } else { base }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpatialSpreadPlan {
    pub spread_draw_consumed: bool,
    pub rain_queried: bool,
    pub age_draw_consumed: bool,
    pub write_fire: Option<(BaseFireKind, u8)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpatialSpreadProbe {
    pub candidate_empty: bool,
    pub encouragement: u8,
    pub difficulty_id: u8,
    pub captured_age: u8,
    pub increased: bool,
    pub spread_draw: u16,
    pub active_rain: bool,
    pub candidate_near_rain: bool,
    pub age_draw: u8,
    pub replacement_kind: BaseFireKind,
}

pub const fn spatial_spread(probe: SpatialSpreadProbe) -> SpatialSpreadPlan {
    if !probe.candidate_empty || probe.encouragement == 0 {
        return SpatialSpreadPlan {
            spread_draw_consumed: false,
            rain_queried: false,
            age_draw_consumed: false,
            write_fire: None,
        };
    }
    let threshold = spread_threshold(
        probe.encouragement,
        probe.difficulty_id,
        probe.captured_age,
        probe.increased,
    );
    if threshold == 0 {
        return SpatialSpreadPlan {
            spread_draw_consumed: false,
            rain_queried: false,
            age_draw_consumed: false,
            write_fire: None,
        };
    }
    if probe.spread_draw > threshold {
        return SpatialSpreadPlan {
            spread_draw_consumed: true,
            rain_queried: false,
            age_draw_consumed: false,
            write_fire: None,
        };
    }
    if probe.active_rain && probe.candidate_near_rain {
        return SpatialSpreadPlan {
            spread_draw_consumed: true,
            rain_queried: true,
            age_draw_consumed: false,
            write_fire: None,
        };
    }
    SpatialSpreadPlan {
        spread_draw_consumed: true,
        rain_queried: probe.active_rain,
        age_draw_consumed: true,
        write_fire: Some((
            probe.replacement_kind,
            clamped_age_add(probe.captured_age, probe.age_draw / 4),
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TntPrimePlan {
    pub entity_admission_attempted: bool,
    pub entity_admitted: bool,
    pub play_primed_sound: bool,
    pub emit_prime_fuse: bool,
    pub centered_at_integer_y: bool,
}

pub const fn tnt_prime(tnt_explodes: bool, admission_succeeded: bool) -> TntPrimePlan {
    TntPrimePlan {
        entity_admission_attempted: tnt_explodes,
        entity_admitted: tnt_explodes && admission_succeeded,
        play_primed_sound: tnt_explodes,
        emit_prime_fuse: tnt_explodes,
        centered_at_integer_y: tnt_explodes,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactStage {
    ClearFreeze,
    FireIgnite,
    QueueDamage,
}

pub const CONTACT_ORDER: [ContactStage; 3] = [
    ContactStage::ClearFreeze,
    ContactStage::FireIgnite,
    ContactStage::QueueDamage,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FireContactPlan {
    pub ignition_aborted: bool,
    pub player_draw_consumed: bool,
    pub remaining_fire: i32,
    pub set_seconds_on_fire: Option<u8>,
    pub queued_in_fire_damage: f32,
}

pub const fn fire_contact(
    kind: BaseFireKind,
    fire_immune: bool,
    server_player: bool,
    remaining_fire: i32,
    player_draw: u8,
) -> FireContactPlan {
    let damage = match kind {
        BaseFireKind::Ordinary => 1.0,
        BaseFireKind::Soul => 2.0,
    };
    if fire_immune {
        return FireContactPlan {
            ignition_aborted: true,
            player_draw_consumed: false,
            remaining_fire,
            set_seconds_on_fire: None,
            queued_in_fire_damage: damage,
        };
    }
    let (remaining_fire, consumed) = if remaining_fire < 0 {
        (remaining_fire + 1, false)
    } else if server_player {
        (remaining_fire + player_draw as i32, true)
    } else {
        (remaining_fire, false)
    };
    FireContactPlan {
        ignition_aborted: false,
        player_draw_consumed: consumed,
        remaining_fire,
        set_seconds_on_fire: if remaining_fire >= 0 { Some(8) } else { None },
        queued_in_fire_damage: damage,
    }
}
