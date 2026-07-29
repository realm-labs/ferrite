//! Bounded block/sky light propagation, publication, and client import policy.

pub const MAX_LIGHT: u8 = 15;
pub const NIBBLE_SECTION_BYTES: usize = 2_048;
pub const SERVER_TASK_BATCH: usize = 1_000;
pub const CLIENT_ALL_TASK_THRESHOLD: usize = 1_000;
pub const CLIENT_MIN_TASK_BUDGET: usize = 10;
pub const EMITTING_BLOCK_COUNT: usize = 109;
pub const SKY_LIGHT_ATTRIBUTE_DEFAULT: f32 = 15.0;
pub const SKY_LIGHT_ATTRIBUTE_MIN: f32 = 0.0;
pub const SKY_LIGHT_ATTRIBUTE_MAX: f32 = 15.0;
pub const DEFERRED_EXPERIMENT: &str = "EXP-ENV-004";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightChannel {
    Block,
    Sky,
}

pub const CHECK_ORDER: [LightChannel; 2] = [LightChannel::Block, LightChannel::Sky];
pub const DRAIN_ORDER: [LightWork; 2] = [LightWork::Decrease, LightWork::Increase];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightWork {
    Decrease,
    Increase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropagationWrite {
    pub value: u8,
    pub enqueue_increase: bool,
}

pub const fn propagation_candidate(
    source: u8,
    target_dampening: u8,
    stored_target: u8,
    faces_occlude: bool,
) -> Option<PropagationWrite> {
    if faces_occlude {
        return None;
    }
    let attenuation = if target_dampening < 1 {
        1
    } else {
        target_dampening
    };
    let candidate = source.saturating_sub(attenuation);
    if candidate <= stored_target {
        None
    } else {
        Some(PropagationWrite {
            value: candidate,
            enqueue_increase: true,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceChangePlan {
    pub stored_value: u8,
    pub decrease_value: Option<u8>,
    pub increase_value: Option<u8>,
}

pub const fn block_source_change(
    stored: u8,
    current_emission: u8,
    alternative_neighbour: u8,
    source_enabled: bool,
) -> SourceChangePlan {
    let emission = if source_enabled { current_emission } else { 0 };
    if emission < stored {
        SourceChangePlan {
            stored_value: 0,
            decrease_value: Some(stored),
            increase_value: if alternative_neighbour > stored {
                Some(alternative_neighbour)
            } else {
                None
            },
        }
    } else if emission > stored {
        SourceChangePlan {
            stored_value: stored,
            decrease_value: None,
            increase_value: Some(emission),
        }
    } else {
        SourceChangePlan {
            stored_value: stored,
            decrease_value: None,
            increase_value: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerLightStage {
    PreUpdate,
    BlockDecrease,
    BlockIncrease,
    SkyDecrease,
    SkyIncrease,
    ReconcileSections,
    PublishVisible,
    PostUpdate,
}

pub const SERVER_DRAIN_STAGES: [ServerLightStage; 8] = [
    ServerLightStage::PreUpdate,
    ServerLightStage::BlockDecrease,
    ServerLightStage::BlockIncrease,
    ServerLightStage::SkyDecrease,
    ServerLightStage::SkyIncrease,
    ServerLightStage::ReconcileSections,
    ServerLightStage::PublishVisible,
    ServerLightStage::PostUpdate,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerTaskBatch {
    pub selected: usize,
    pub remaining: usize,
    pub drains_engines_completely: bool,
}

pub const fn server_task_batch(queued: usize) -> ServerTaskBatch {
    let selected = if queued < SERVER_TASK_BATCH {
        queued
    } else {
        SERVER_TASK_BATCH
    };
    ServerTaskBatch {
        selected,
        remaining: queued - selected,
        drains_engines_completely: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionStorageTransition {
    pub create_layer: bool,
    pub defer_removal: bool,
    pub reference_delta: i8,
}

pub const fn section_storage_transition(
    previous_nonempty: bool,
    current_nonempty: bool,
) -> SectionStorageTransition {
    match (previous_nonempty, current_nonempty) {
        (false, true) => SectionStorageTransition {
            create_layer: true,
            defer_removal: false,
            reference_delta: 27,
        },
        (true, false) => SectionStorageTransition {
            create_layer: false,
            defer_removal: true,
            reference_delta: -27,
        },
        _ => SectionStorageTransition {
            create_layer: false,
            defer_removal: false,
            reference_delta: 0,
        },
    }
}

pub const fn queued_layer_installed(section_stores_light: bool) -> bool {
    section_stores_light
}

pub const fn sky_engine_enabled(dimension_has_skylight: bool) -> bool {
    dimension_has_skylight
}

pub const fn first_write_copies_layer(already_changed: bool) -> bool {
    !already_changed
}

pub fn affected_sections(center: [i32; 3]) -> Vec<[i32; 3]> {
    let mut sections = Vec::with_capacity(27);
    for y in -1..=1 {
        for z in -1..=1 {
            for x in -1..=1 {
                sections.push([center[0] + x, center[1] + y, center[2] + z]);
            }
        }
    }
    sections
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightPublication {
    pub copy_updating_to_visible: bool,
    pub mark_chunk_unsaved: bool,
    pub channel: LightChannel,
    pub affected_sections: Vec<[i32; 3]>,
    pub packet_bit_set: bool,
}

pub fn publish_light(channel: LightChannel, section: [i32; 3]) -> LightPublication {
    LightPublication {
        copy_updating_to_visible: true,
        mark_chunk_unsaved: true,
        channel,
        affected_sections: affected_sections(section),
        packet_bit_set: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightPacketPlan {
    pub send: bool,
    pub clear_masks_after_send: bool,
}

pub const fn light_packet_plan(
    ticking_visible_holder: bool,
    tracking_players: bool,
    mask_nonempty: bool,
) -> LightPacketPlan {
    let send = ticking_visible_holder && tracking_players && mask_nonempty;
    LightPacketPlan {
        send,
        clear_masks_after_send: send,
    }
}

pub const fn missing_block_layer_value(layer: Option<u8>) -> u8 {
    match layer {
        Some(value) => value,
        None => 0,
    }
}

pub const fn sky_query(
    source_column_enabled: bool,
    above_top_data_section: bool,
    own_layer: Option<u8>,
    next_stored_layer_above: Option<u8>,
) -> u8 {
    if above_top_data_section {
        return MAX_LIGHT;
    }
    if !source_column_enabled {
        return 0;
    }
    match own_layer {
        Some(value) => value,
        None => match next_stored_layer_above {
            Some(value) => value,
            None => 0,
        },
    }
}

pub const fn direct_sky_source(position_y: i32, lowest_source_y: i32) -> u8 {
    if position_y >= lowest_source_y {
        MAX_LIGHT
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkyEdge {
    pub below_y: i32,
    pub below_dampening: u8,
    pub joined_vertical_faces_occlude: bool,
}

pub fn lowest_sky_source(edges_high_to_low: &[SkyEdge], minimum_y: i32) -> i32 {
    edges_high_to_low
        .iter()
        .find(|edge| edge.below_dampening != 0 || edge.joined_vertical_faces_occlude)
        .map(|edge| edge.below_y + 1)
        .unwrap_or(minimum_y)
}

pub const fn prefill_sky_value(position_y: i32, column_threshold_y: i32) -> u8 {
    direct_sky_source(position_y, column_threshold_y)
}

pub const fn raw_brightness(block: Option<u8>, sky: Option<u8>, darken: u8) -> u8 {
    let block = match block {
        Some(value) => value,
        None => 0,
    };
    let sky = match sky {
        Some(value) => value.saturating_sub(darken),
        None => 0,
    };
    if block > sky { block } else { sky }
}

pub fn sky_darken(sky_light_level: f32) -> i32 {
    (15.0_f32 - sky_light_level.clamp(SKY_LIGHT_ATTRIBUTE_MIN, SKY_LIGHT_ATTRIBUTE_MAX)) as i32
}

pub const fn has_different_light_properties(
    old_dampening: u8,
    new_dampening: u8,
    old_emission: u8,
    new_emission: u8,
    old_uses_shape: bool,
    new_uses_shape: bool,
) -> bool {
    old_dampening != new_dampening
        || old_emission != new_emission
        || old_uses_shape
        || new_uses_shape
}

pub const fn client_light_task_budget(queued: usize) -> usize {
    if queued >= CLIENT_ALL_TASK_THRESHOLD {
        queued
    } else {
        let tenth = queued / 10;
        if tenth < CLIENT_MIN_TASK_BUDGET {
            CLIENT_MIN_TASK_BUDGET
        } else {
            tenth
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientLightStage {
    ImportSky,
    ImportBlock,
    MarkSectionsDirty,
    EnableChunk,
    DrainLighting,
    RendererUpdate,
}

pub const CLIENT_IMPORT_ORDER: [ClientLightStage; 6] = [
    ClientLightStage::ImportSky,
    ClientLightStage::ImportBlock,
    ClientLightStage::MarkSectionsDirty,
    ClientLightStage::EnableChunk,
    ClientLightStage::DrainLighting,
    ClientLightStage::RendererUpdate,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyPolicy {
    pub experiment: &'static str,
    pub universal_tick_deadline: Option<u64>,
    pub universal_frame_deadline: Option<u64>,
    pub claims_vanilla_bound: bool,
}

pub const LATENCY_POLICY: LatencyPolicy = LatencyPolicy {
    experiment: DEFERRED_EXPERIMENT,
    universal_tick_deadline: None,
    universal_frame_deadline: None,
    claims_vanilla_bound: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopperOxidation {
    Unaffected,
    Exposed,
    Weathered,
    Oxidized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialSpawnerLight {
    Inactive,
    Cooldown,
    WaitingForPlayers,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitterState<'a> {
    Candle {
        lit: bool,
        candles: u8,
    },
    CandleCake {
        lit: bool,
    },
    CopperBulb {
        lit: bool,
        oxidation: CopperOxidation,
    },
    Campfire {
        lit: bool,
        soul: bool,
    },
    FurnaceFamily {
        lit: bool,
    },
    RedstoneTorch {
        lit: bool,
    },
    RedstoneOre {
        lit: bool,
    },
    CaveVines {
        berries: bool,
    },
    SeaPickle {
        waterlogged: bool,
        pickles: u8,
    },
    RespawnAnchor {
        charges: u8,
    },
    LightBlock {
        level: u8,
    },
    GlowLichen {
        any_face: bool,
    },
    RedstoneLamp {
        lit: bool,
    },
    TrialSpawner(TrialSpawnerLight),
    Vault {
        inactive: bool,
    },
    Static(&'a str),
}

pub fn emission(state: EmitterState<'_>) -> u8 {
    match state {
        EmitterState::Candle { lit, candles } => {
            if lit {
                candles.min(4) * 3
            } else {
                0
            }
        }
        EmitterState::CandleCake { lit } => u8::from(lit) * 3,
        EmitterState::CopperBulb { lit, oxidation } => {
            if !lit {
                return 0;
            }
            match oxidation {
                CopperOxidation::Unaffected => 15,
                CopperOxidation::Exposed => 12,
                CopperOxidation::Weathered => 8,
                CopperOxidation::Oxidized => 4,
            }
        }
        EmitterState::Campfire { lit, soul } => {
            if lit {
                if soul { 10 } else { 15 }
            } else {
                0
            }
        }
        EmitterState::FurnaceFamily { lit } => u8::from(lit) * 13,
        EmitterState::RedstoneTorch { lit } => u8::from(lit) * 7,
        EmitterState::RedstoneOre { lit } => u8::from(lit) * 9,
        EmitterState::CaveVines { berries } => u8::from(berries) * 14,
        EmitterState::SeaPickle {
            waterlogged,
            pickles,
        } => {
            if waterlogged {
                3 * (pickles.min(4) + 1)
            } else {
                0
            }
        }
        EmitterState::RespawnAnchor { charges } => {
            if charges == 0 {
                0
            } else {
                4 * charges.min(4) - 1
            }
        }
        EmitterState::LightBlock { level } => level.min(MAX_LIGHT),
        EmitterState::GlowLichen { any_face } => u8::from(any_face) * 7,
        EmitterState::RedstoneLamp { lit } => u8::from(lit) * 15,
        EmitterState::TrialSpawner(state) => match state {
            TrialSpawnerLight::Inactive | TrialSpawnerLight::Cooldown => 0,
            TrialSpawnerLight::WaitingForPlayers => 4,
            TrialSpawnerLight::Active => 8,
        },
        EmitterState::Vault { inactive } => {
            if inactive {
                6
            } else {
                12
            }
        }
        EmitterState::Static(path) => static_emission(path),
    }
}

fn static_emission(path: &str) -> u8 {
    if matches!(
        path,
        "brewing_stand"
            | "brown_mushroom"
            | "sculk_sensor"
            | "calibrated_sculk_sensor"
            | "dragon_egg"
            | "end_portal_frame"
            | "small_amethyst_bud"
    ) {
        1
    } else if matches!(path, "firefly_bush" | "medium_amethyst_bud") {
        2
    } else if path == "magma_block" {
        3
    } else if path == "large_amethyst_bud" {
        4
    } else if path == "amethyst_cluster" {
        5
    } else if path == "sculk_catalyst" {
        6
    } else if matches!(path, "enchanting_table" | "ender_chest") {
        7
    } else if matches!(
        path,
        "crying_obsidian" | "soul_fire" | "soul_lantern" | "soul_torch" | "soul_wall_torch"
    ) {
        10
    } else if path == "nether_portal" {
        11
    } else if matches!(
        path,
        "copper_torch" | "copper_wall_torch" | "end_rod" | "torch" | "wall_torch"
    ) {
        14
    } else if matches!(
        path,
        "beacon"
            | "conduit"
            | "end_gateway"
            | "end_portal"
            | "fire"
            | "glowstone"
            | "jack_o_lantern"
            | "lantern"
            | "lava"
            | "lava_cauldron"
            | "ochre_froglight"
            | "pearlescent_froglight"
            | "verdant_froglight"
            | "sea_lantern"
            | "shroomlight"
    ) || path.ends_with("copper_lantern")
    {
        15
    } else {
        0
    }
}
