//! Ordered, deliberately non-atomic block-item placement transaction.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::id::BlockStateId;

pub const GENERIC_PLACE_FLAGS: u32 = 11;
pub const BED_FOOT_FLAGS: u32 = 26;
pub const DOUBLE_HIGH_CLEAR_FLAGS: u32 = 27;
pub const SECOND_HALF_FLAGS: u32 = 3;
pub const COMPONENT_PATCH_FLAGS: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementKind {
    Generic,
    DoubleHigh { upper_replacement: BlockStateId },
    Bed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockItemKind {
    Generic,
    DoubleHigh,
    Bed,
    StandingAndWall,
    PlaceOnWater,
    Scaffolding,
    GameMaster,
    SolidBucket,
}

pub fn block_item_kind(path: &str) -> BlockItemKind {
    if path.ends_with("_door") || is_double_high_plant(path) {
        BlockItemKind::DoubleHigh
    } else if path.ends_with("_bed") {
        BlockItemKind::Bed
    } else if is_standing_and_wall_item(path) {
        BlockItemKind::StandingAndWall
    } else {
        match path {
            "lily_pad" | "frogspawn" => BlockItemKind::PlaceOnWater,
            "scaffolding" => BlockItemKind::Scaffolding,
            "command_block"
            | "chain_command_block"
            | "repeating_command_block"
            | "structure_block"
            | "jigsaw" => BlockItemKind::GameMaster,
            "powder_snow_bucket" => BlockItemKind::SolidBucket,
            _ => BlockItemKind::Generic,
        }
    }
}

fn is_double_high_plant(path: &str) -> bool {
    matches!(
        path,
        "small_dripleaf"
            | "sunflower"
            | "lilac"
            | "rose_bush"
            | "peony"
            | "tall_grass"
            | "large_fern"
    )
}

fn is_standing_and_wall_item(path: &str) -> bool {
    path.ends_with("_sign")
        || path.ends_with("_hanging_sign")
        || path.ends_with("_banner")
        || path.ends_with("_coral_fan")
        || path.ends_with("_skull")
        || path.ends_with("_head")
        || matches!(
            path,
            "torch" | "soul_torch" | "redstone_torch" | "copper_torch"
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementRequest {
    pub target: BlockPos,
    pub candidate: BlockStateId,
    pub second_half: Option<(BlockPos, BlockStateId)>,
    pub kind: PlacementKind,
    pub component_patch: Option<BlockStateId>,
    pub consumes_item: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementWrite {
    pub position: BlockPos,
    pub state: BlockStateId,
    pub flags: u32,
    pub result_matters: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementWriteResults {
    pub initial: bool,
    pub current_has_candidate_block: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementTransaction {
    pub writes: Vec<PlacementWrite>,
    pub applies_block_entity_data: bool,
    pub calls_set_placed_by: bool,
    pub emits_placed_criterion: bool,
    pub emits_sound_and_game_event: bool,
    pub consumes_item: bool,
    pub success: bool,
}

pub fn plan_placement(
    request: PlacementRequest,
    results: PlacementWriteResults,
) -> PlacementTransaction {
    let mut writes = Vec::with_capacity(4);
    if let PlacementKind::DoubleHigh { upper_replacement } = request.kind
        && let Some((position, _)) = request.second_half
    {
        writes.push(PlacementWrite {
            position,
            state: upper_replacement,
            flags: DOUBLE_HIGH_CLEAR_FLAGS,
            result_matters: false,
        });
    }
    let initial_flags = match request.kind {
        PlacementKind::Bed => BED_FOOT_FLAGS,
        PlacementKind::Generic | PlacementKind::DoubleHigh { .. } => GENERIC_PLACE_FLAGS,
    };
    writes.push(PlacementWrite {
        position: request.target,
        state: request.candidate,
        flags: initial_flags,
        result_matters: true,
    });
    if !results.initial {
        return PlacementTransaction {
            writes,
            applies_block_entity_data: false,
            calls_set_placed_by: false,
            emits_placed_criterion: false,
            emits_sound_and_game_event: false,
            consumes_item: false,
            success: false,
        };
    }

    let same_block = results.current_has_candidate_block;
    if same_block {
        if let Some(patched) = request.component_patch {
            writes.push(PlacementWrite {
                position: request.target,
                state: patched,
                flags: COMPONENT_PATCH_FLAGS,
                result_matters: false,
            });
        }
        if let Some((position, state)) = request.second_half {
            writes.push(PlacementWrite {
                position,
                state,
                flags: SECOND_HALF_FLAGS,
                result_matters: false,
            });
        }
    }
    PlacementTransaction {
        writes,
        applies_block_entity_data: same_block,
        calls_set_placed_by: same_block,
        emits_placed_criterion: same_block,
        emits_sound_and_game_event: true,
        consumes_item: request.consumes_item,
        success: true,
    }
}

pub const fn placement_target(
    hit: BlockPos,
    adjacent: BlockPos,
    replace_clicked: bool,
) -> BlockPos {
    if replace_clicked { hit } else { adjacent }
}

pub const fn scaffolding_horizontal_extension_allowed(traversed: u8) -> bool {
    traversed < 7
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorHinge {
    Left,
    Right,
}

pub fn door_hinge(
    facing: Direction,
    left_lower_door: bool,
    right_lower_door: bool,
    left_full_blocks: u8,
    right_full_blocks: u8,
    click_x: f64,
    click_z: f64,
) -> DoorHinge {
    if left_lower_door && !right_lower_door {
        return DoorHinge::Right;
    }
    if right_lower_door && !left_lower_door {
        return DoorHinge::Left;
    }
    let score = i16::from(right_full_blocks) - i16::from(left_full_blocks);
    if score > 0 {
        return DoorHinge::Right;
    }
    if score < 0 {
        return DoorHinge::Left;
    }
    let [step_x, _, step_z] = facing.step();
    if (step_x < 0 && click_z < 0.5)
        || (step_x > 0 && click_z > 0.5)
        || (step_z < 0 && click_x > 0.5)
        || (step_z > 0 && click_x < 0.5)
    {
        DoorHinge::Right
    } else {
        DoorHinge::Left
    }
}
