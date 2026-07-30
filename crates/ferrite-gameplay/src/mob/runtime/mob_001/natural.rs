//! Natural-spawn counting, category admission, pack walks, and chunk generation.

use crate::mob::runtime::mob_001::hostile::NaturalCategory;

pub const CATEGORY_ORDER: [NaturalCategory; 7] = [
    NaturalCategory::Monster,
    NaturalCategory::Creature,
    NaturalCategory::Ambient,
    NaturalCategory::Axolotls,
    NaturalCategory::UndergroundWaterCreature,
    NaturalCategory::WaterCreature,
    NaturalCategory::WaterAmbient,
];
pub const REGISTERED_PLACEMENT_COUNT: usize = 83;

#[must_use]
pub const fn base_max(category: NaturalCategory) -> u32 {
    match category {
        NaturalCategory::Monster => 70,
        NaturalCategory::Creature => 10,
        NaturalCategory::Ambient => 15,
        NaturalCategory::Axolotls
        | NaturalCategory::UndergroundWaterCreature
        | NaturalCategory::WaterCreature => 5,
        NaturalCategory::WaterAmbient => 20,
    }
}

#[must_use]
pub const fn friendly(category: NaturalCategory) -> bool {
    !matches!(category, NaturalCategory::Monster)
}

#[must_use]
pub const fn persistent(category: NaturalCategory) -> bool {
    matches!(category, NaturalCategory::Creature)
}

#[must_use]
pub const fn hard_distance(category: NaturalCategory) -> u32 {
    if matches!(category, NaturalCategory::WaterAmbient) {
        64
    } else {
        128
    }
}

#[must_use]
pub const fn filtered_category(
    category: NaturalCategory,
    spawn_enemies: bool,
    spawn_persistent: bool,
) -> bool {
    (spawn_enemies || friendly(category)) && (spawn_persistent || !persistent(category))
}

#[must_use]
pub const fn creature_cadence(game_time: u64) -> bool {
    game_time.is_multiple_of(400)
}

#[must_use]
pub const fn global_cap(category: NaturalCategory, spawnable_chunk_count: u32) -> u32 {
    base_max(category).saturating_mul(spawnable_chunk_count) / 289
}

#[must_use]
pub const fn below_global_cap(
    category: NaturalCategory,
    spawnable_chunk_count: u32,
    current_count: u32,
) -> bool {
    current_count < global_cap(category, spawnable_chunk_count)
}

#[must_use]
pub fn chunk_center_candidate(distance_squared: f64, nonspectator_player_present: bool) -> bool {
    nonspectator_player_present && distance_squared < 16_384.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotAccounting {
    pub count_global: bool,
    pub add_potential_charge: bool,
    pub count_for_nearby_players: bool,
}

#[must_use]
pub const fn snapshot_accounting(input: SnapshotInput) -> SnapshotAccounting {
    let skipped = input.misc_category
        || (input.mob && (input.persistence_required || input.custom_persistence))
        || !input.containing_chunk_queryable;
    SnapshotAccounting {
        count_global: !skipped,
        add_potential_charge: !skipped && input.spawn_cost_defined,
        count_for_nearby_players: !skipped && input.mob,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotInput {
    pub misc_category: bool,
    pub mob: bool,
    pub persistence_required: bool,
    pub custom_persistence: bool,
    pub containing_chunk_queryable: bool,
    pub spawn_cost_defined: bool,
}

#[must_use]
pub fn local_cap_allows(category: NaturalCategory, nearby_player_counts: &[Option<u32>]) -> bool {
    nearby_player_counts.iter().any(|count| match count {
        None => true,
        Some(count) => *count < base_max(category),
    })
}

#[must_use]
pub fn potential_allows(existing_potential: f64, charge: f64, energy_budget: f64) -> bool {
    existing_potential * charge <= energy_budget
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionStart {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub admitted: bool,
}

#[must_use]
pub const fn position_start(input: PositionStartInput) -> PositionStart {
    let height = input
        .surface_plus_one
        .saturating_sub(input.min_y)
        .saturating_add(1);
    let y = if height <= 0 {
        input.min_y
    } else {
        input.min_y + (input.y_draw % height as u32) as i32
    };
    PositionStart {
        x: input.chunk_min_x + (input.x_draw % 16) as i32,
        y,
        z: input.chunk_min_z + (input.z_draw % 16) as i32,
        admitted: y >= input.min_y.saturating_add(1) && !input.starting_block_conductor,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionStartInput {
    pub chunk_min_x: i32,
    pub chunk_min_z: i32,
    pub x_draw: u32,
    pub z_draw: u32,
    pub min_y: i32,
    pub surface_plus_one: i32,
    pub y_draw: u32,
    pub starting_block_conductor: bool,
}

#[must_use]
pub fn provisional_attempts(next_float: f32) -> u8 {
    (next_float * 4.0).ceil() as u8
}

#[must_use]
pub const fn selected_group_count(minimum: u32, maximum: u32, draw: u32) -> u32 {
    if maximum < minimum {
        minimum
    } else {
        minimum + draw % (maximum - minimum + 1)
    }
}

#[must_use]
pub const fn pack_offset(first_draw: u32, second_draw: u32) -> i32 {
    (first_draw % 6) as i32 - (second_draw % 6) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateDistanceFailure {
    NoPlayer,
    PlayerWithinTwentyFour,
    RespawnWithinTwentyFour,
    DestinationChunkInactive,
}

pub fn candidate_distance(
    nearest_player_present: bool,
    player_distance_squared: f64,
    respawn_same_dimension: bool,
    respawn_distance_squared: f64,
    outside_original_chunk: bool,
    destination_chunk_active: bool,
) -> Result<(), CandidateDistanceFailure> {
    if !nearest_player_present {
        Err(CandidateDistanceFailure::NoPlayer)
    } else if player_distance_squared <= 576.0 {
        Err(CandidateDistanceFailure::PlayerWithinTwentyFour)
    } else if respawn_same_dimension && respawn_distance_squared <= 576.0 {
        Err(CandidateDistanceFailure::RespawnWithinTwentyFour)
    } else if outside_original_chunk && !destination_chunk_active {
        Err(CandidateDistanceFailure::DestinationChunkInactive)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnListSelection {
    pub reduced_water_draw_consumed: bool,
    pub use_fortress_list: bool,
    pub select_weighted_entry: bool,
    pub end_group: bool,
}

#[must_use]
pub const fn spawn_list_selection(
    category: NaturalCategory,
    reduced_water_biome: bool,
    reduced_water_draw: f32,
    monster_above_nether_bricks: bool,
    valid_fortress: bool,
    list_nonempty: bool,
) -> SpawnListSelection {
    let reduced = matches!(category, NaturalCategory::WaterAmbient) && reduced_water_biome;
    let rejected = reduced && reduced_water_draw < 0.98;
    let use_fortress = matches!(category, NaturalCategory::Monster)
        && monster_above_nether_bricks
        && valid_fortress;
    SpawnListSelection {
        reduced_water_draw_consumed: reduced,
        use_fortress_list: use_fortress,
        select_weighted_entry: !rejected && list_nonempty,
        end_group: rejected || !list_nonempty,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructionDisposition {
    RejectCandidate,
    EndCategoryPosition,
    FinalizeAndAccount,
}

#[must_use]
pub const fn construction_disposition(
    preconstruction_gates_pass: bool,
    construction_succeeded: bool,
    constructed_is_mob: bool,
    mob_gates_pass: bool,
) -> ConstructionDisposition {
    if !preconstruction_gates_pass {
        ConstructionDisposition::RejectCandidate
    } else if !construction_succeeded || !constructed_is_mob {
        ConstructionDisposition::EndCategoryPosition
    } else if !mob_gates_pass {
        ConstructionDisposition::RejectCandidate
    } else {
        ConstructionDisposition::FinalizeAndAccount
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuccessfulSpawnAccounting {
    pub finalize_with_group_data: bool,
    pub insert_with_passengers: bool,
    pub account_after_insertion_call: bool,
    pub rollback_on_insertion_failure: bool,
}

pub const SUCCESSFUL_SPAWN_ACCOUNTING: SuccessfulSpawnAccounting = SuccessfulSpawnAccounting {
    finalize_with_group_data: true,
    insert_with_passengers: true,
    account_after_insertion_call: true,
    rollback_on_insertion_failure: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackStop {
    Continue,
    EndCurrentWalk,
    EndAllWalks,
}

#[must_use]
pub const fn pack_stop(
    cluster_size: u32,
    max_cluster_size: u32,
    group_size: u32,
    max_group_reached: bool,
) -> PackStop {
    if cluster_size >= max_cluster_size {
        PackStop::EndAllWalks
    } else if max_group_reached && group_size > 0 {
        PackStop::EndCurrentWalk
    } else {
        PackStop::Continue
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementType {
    InWater,
    InLava,
    NoRestrictions,
    OnGround,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heightmap {
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnregisteredPlacement {
    pub placement: PlacementType,
    pub heightmap: Heightmap,
    pub predicate: bool,
}

pub const UNREGISTERED_PLACEMENT: UnregisteredPlacement = UnregisteredPlacement {
    placement: PlacementType::NoRestrictions,
    heightmap: Heightmap::MotionBlockingNoLeaves,
    predicate: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkGenerationAttempt {
    pub attempts_per_member: u8,
    pub continue_after_null_or_exception: bool,
    pub horizontal_walk_draws: u8,
    pub resample_until_inside_chunk: bool,
}

pub const CHUNK_GENERATION_ATTEMPT: ChunkGenerationAttempt = ChunkGenerationAttempt {
    attempts_per_member: 4,
    continue_after_null_or_exception: true,
    horizontal_walk_draws: 4,
    resample_until_inside_chunk: true,
};

#[must_use]
pub fn chunk_generation_group_admitted(
    spawn_mobs: bool,
    probability_draw: f32,
    creature_probability: f32,
) -> bool {
    spawn_mobs && probability_draw < creature_probability
}
