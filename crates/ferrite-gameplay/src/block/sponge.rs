//! Sponge breadth-first absorption decisions and Wet-Sponge transitions.

use ferrite_foundation::direction::Direction;

pub const SPONGE_STATE_ID: u32 = 560;
pub const WET_SPONGE_STATE_ID: u32 = 561;
pub const ABSORPTION_DEPTH: u8 = 6;
pub const ABSORPTION_NODE_CAP: u8 = 65;
pub const MAX_REMOVED_WATER_CELLS: u8 = 64;
pub const NEIGHBOR_ORDER: [Direction; 6] = Direction::ALL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterCandidate {
    NotWater,
    BucketPickup { returned_nonempty: bool },
    LiquidBlock,
    WaterPlant,
    OtherWaterBearing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbsorbAction {
    Skip,
    AcceptBucketPickup,
    AcceptLiquidWriteAir,
    AcceptDropPlantThenWriteAir,
}

pub fn absorb_candidate(candidate: WaterCandidate) -> AbsorbAction {
    match candidate {
        WaterCandidate::NotWater | WaterCandidate::OtherWaterBearing => AbsorbAction::Skip,
        WaterCandidate::BucketPickup {
            returned_nonempty: true,
        } => AbsorbAction::AcceptBucketPickup,
        WaterCandidate::BucketPickup {
            returned_nonempty: false,
        } => AbsorbAction::Skip,
        WaterCandidate::LiquidBlock => AbsorbAction::AcceptLiquidWriteAir,
        WaterCandidate::WaterPlant => AbsorbAction::AcceptDropPlantThenWriteAir,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbsorptionResult {
    pub wet_write: bool,
    pub play_absorb_sound: bool,
}

pub fn absorption_result(accepted_nodes_including_origin: u8) -> AbsorptionResult {
    let success = accepted_nodes_including_origin > 1;
    AbsorptionResult {
        wet_write: success,
        play_absorb_sound: success,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WetSpongeDry {
    pub write_dry_flags: u16,
    pub level_event: u16,
    pub pitch: f32,
}

pub fn wet_sponge_on_place(water_evaporates: bool, next_float: f32) -> Option<WetSpongeDry> {
    if !water_evaporates {
        return None;
    }
    Some(WetSpongeDry {
        write_dry_flags: 3,
        level_event: 2_009,
        pitch: (1.0 + next_float * 0.2) * 0.7,
    })
}

pub fn furnace_bucket_result(input_wet_sponge: bool, fuel_exact_bucket: bool) -> bool {
    input_wet_sponge && fuel_exact_bucket
}
