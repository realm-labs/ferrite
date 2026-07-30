//! Tame ownership, trust, horse temper, and owner teleport.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TameCommit {
    pub tame_flag_bit: u8,
    pub sitting_pose_bit: u8,
    pub assign_owner: bool,
    pub trigger_server_player_criterion: bool,
    pub set_generic_persistence: bool,
}

pub const TAME_COMMIT: TameCommit = TameCommit {
    tame_flag_bit: 4,
    sitting_pose_bit: 1,
    assign_owner: true,
    trigger_server_player_criterion: true,
    set_generic_persistence: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TameAttempt {
    NotAdmitted,
    Success {
        event: u8,
        clear_navigation_and_target: bool,
        order_sit: bool,
    },
    Failure {
        event: u8,
    },
}

#[must_use]
pub const fn tame_attempt(
    admitted: bool,
    draw: u8,
    denominator: u8,
    clear_navigation_and_target: bool,
    order_sit: bool,
) -> TameAttempt {
    if !admitted {
        TameAttempt::NotAdmitted
    } else if denominator > 0 && draw.is_multiple_of(denominator) {
        TameAttempt::Success {
            event: 7,
            clear_navigation_and_target,
            order_sit,
        }
    } else {
        TameAttempt::Failure { event: 6 }
    }
}

#[must_use]
pub const fn ocelot_trust_admitted(
    tempt_goal_running: bool,
    already_trusting: bool,
    tagged_food: bool,
    distance_squared: f64,
) -> bool {
    tempt_goal_running && !already_trusting && tagged_food && distance_squared < 9.0
}

#[must_use]
pub const fn trust_event(success: bool) -> u8 {
    if success { 41 } else { 40 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorseTame {
    NotChecked,
    Success,
    Failure { new_temper: u16 },
}

#[must_use]
pub const fn horse_tame(
    cadence_draw: u32,
    adjusted_delay: u32,
    maximum_temper: u16,
    temper: u16,
    temper_draw: u16,
) -> HorseTame {
    if adjusted_delay == 0 || !cadence_draw.is_multiple_of(adjusted_delay) {
        HorseTame::NotChecked
    } else if maximum_temper > 0 && temper_draw % maximum_temper < temper {
        HorseTame::Success
    } else {
        HorseTame::Failure {
            new_temper: {
                let increased = temper.saturating_add(5);
                if increased > maximum_temper {
                    maximum_temper
                } else {
                    increased
                }
            },
        }
    }
}

#[must_use]
pub const fn owner_teleport_admitted(
    distance_squared: f64,
    sitting: bool,
    passenger: bool,
    leashed: bool,
    owner_spectator: bool,
) -> bool {
    distance_squared >= 144.0 && !sitting && !passenger && !leashed && !owner_spectator
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeleportOffset {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub horizontal_far_enough: bool,
}

#[must_use]
pub const fn teleport_offset(x_draw: u8, y_draw: u8, z_draw: u8) -> TeleportOffset {
    let x = x_draw as i32 % 7 - 3;
    let y = y_draw as i32 % 3 - 1;
    let z = z_draw as i32 % 7 - 3;
    TeleportOffset {
        x,
        y,
        z,
        horizontal_far_enough: x.abs() >= 2 || z.abs() >= 2,
    }
}

#[must_use]
pub const fn teleport_candidate(
    offset_far_enough: bool,
    walkable: bool,
    leaves: bool,
    flying: bool,
    collision_free: bool,
) -> bool {
    offset_far_enough && walkable && (flying || !leaves) && collision_free
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnerTeleportCommit {
    pub attempts: u8,
    pub snap_block_center: bool,
    pub stop_path: bool,
}

pub const OWNER_TELEPORT_COMMIT: OwnerTeleportCommit = OwnerTeleportCommit {
    attempts: 10,
    snap_block_center: true,
    stop_path: true,
};
