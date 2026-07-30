//! Ordinary, server-player, and subtype death-entry ordering.

use std::f32::consts::PI;

use crate::entity::runtime::ent_005::knockback::Vector3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrdinaryDeathAdmission {
    pub admitted: bool,
    pub award_kill_score_first: bool,
}

#[must_use]
pub const fn ordinary_death_admission(
    removed: bool,
    already_dead: bool,
    kill_credit_present: bool,
) -> OrdinaryDeathAdmission {
    let admitted = !removed && !already_dead;
    OrdinaryDeathAdmission {
        admitted,
        award_kill_score_first: admitted && kill_credit_present,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdinaryDeathStage {
    AwardKillScore,
    StopSleeping,
    StopUsingItem,
    LogCustomName,
    HandleKillingBlow,
    RecheckCombat,
    CausingKilledEntity,
    EntityDieEvent,
    Drops,
    WitherRose,
    BroadcastEvent3,
    SetDyingPose,
}

pub const ORDINARY_DEATH_ORDER: [OrdinaryDeathStage; 12] = [
    OrdinaryDeathStage::AwardKillScore,
    OrdinaryDeathStage::StopSleeping,
    OrdinaryDeathStage::StopUsingItem,
    OrdinaryDeathStage::LogCustomName,
    OrdinaryDeathStage::HandleKillingBlow,
    OrdinaryDeathStage::RecheckCombat,
    OrdinaryDeathStage::CausingKilledEntity,
    OrdinaryDeathStage::EntityDieEvent,
    OrdinaryDeathStage::Drops,
    OrdinaryDeathStage::WitherRose,
    OrdinaryDeathStage::BroadcastEvent3,
    OrdinaryDeathStage::SetDyingPose,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrdinaryDeathResult {
    pub emit_entity_die: bool,
    pub run_drops: bool,
    pub attempt_wither_rose: bool,
    pub broadcast_event: u8,
    pub set_dying_pose: bool,
}

#[must_use]
pub const fn ordinary_death_result(callback: CausingCallbackResult) -> OrdinaryDeathResult {
    OrdinaryDeathResult {
        emit_entity_die: callback.continue_death,
        run_drops: callback.continue_death,
        attempt_wither_rose: callback.continue_death,
        broadcast_event: 3,
        set_dying_pose: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CausingCallback {
    Absent,
    Default,
    Player,
    ChargedCreeper {
        loot_gate: bool,
        already_dropped_skull: bool,
        emitted_stacks: u8,
    },
    ZombieVillager {
        difficulty: Difficulty,
        normal_skip_draw: bool,
        conversion_succeeded: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CausingCallbackResult {
    pub continue_death: bool,
    pub conversion_draw_consumed: bool,
    pub evaluate_charged_creeper_loot: bool,
    pub set_dropped_skulls: bool,
}

#[must_use]
pub const fn causing_callback(callback: CausingCallback) -> CausingCallbackResult {
    match callback {
        CausingCallback::Absent | CausingCallback::Default | CausingCallback::Player => {
            CausingCallbackResult {
                continue_death: true,
                conversion_draw_consumed: false,
                evaluate_charged_creeper_loot: false,
                set_dropped_skulls: false,
            }
        }
        CausingCallback::ChargedCreeper {
            loot_gate,
            already_dropped_skull,
            emitted_stacks,
        } => {
            let evaluate = loot_gate && !already_dropped_skull;
            CausingCallbackResult {
                continue_death: true,
                conversion_draw_consumed: false,
                evaluate_charged_creeper_loot: evaluate,
                set_dropped_skulls: evaluate && emitted_stacks > 0,
            }
        }
        CausingCallback::ZombieVillager {
            difficulty,
            normal_skip_draw,
            conversion_succeeded,
        } => {
            let eligible = matches!(difficulty, Difficulty::Normal | Difficulty::Hard);
            let draw_consumed = matches!(difficulty, Difficulty::Normal);
            let attempt =
                eligible && (!matches!(difficulty, Difficulty::Normal) || !normal_skip_draw);
            CausingCallbackResult {
                continue_death: !(attempt && conversion_succeeded),
                conversion_draw_consumed: draw_consumed,
                evaluate_charged_creeper_loot: false,
                set_dropped_skulls: false,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitherRose {
    None,
    PlaceBlockIgnoringResult { flags: u8 },
    SpawnItemWithoutPickupDelay,
}

#[must_use]
pub const fn wither_rose(
    wither_kill_credit: bool,
    mob_griefing: bool,
    target_air: bool,
    rose_survives: bool,
) -> WitherRose {
    if !wither_kill_credit {
        WitherRose::None
    } else if mob_griefing && target_air && rose_survives {
        WitherRose::PlaceBlockIgnoringResult { flags: 3 }
    } else {
        WitherRose::SpawnItemWithoutPickupDelay
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientDeathEvent {
    pub play_sound: bool,
    pub pitch: f32,
    pub set_health_zero: bool,
    pub enter_local_generic_death: bool,
}

#[must_use]
pub fn client_death_event(
    player_entity: bool,
    first_draw: f32,
    second_draw: f32,
) -> ClientDeathEvent {
    ClientDeathEvent {
        play_sound: true,
        pitch: 1.0 + (first_draw - second_draw) * 0.2,
        set_health_zero: !player_entity,
        enter_local_generic_death: !player_entity,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamDeathVisibility {
    Always,
    OwnTeam,
    OtherTeams,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerPlayerDeath {
    pub emit_entity_die: bool,
    pub combat_packet_has_message: bool,
    pub broadcast_real_message: bool,
    pub try_shoulders: bool,
    pub shoulder_attempts: u8,
    pub forgive_neutral_mobs: bool,
    pub award_death_objectives: bool,
    pub award_kill_credit_score: bool,
    pub attempt_wither_rose: bool,
    pub run_items_and_xp: bool,
    pub reset_death_stats: bool,
    pub clear_fire_and_frozen: bool,
    pub record_last_death_location: bool,
    pub recheck_combat: bool,
    pub broadcast_event: u8,
    pub set_dead: bool,
    pub set_dying_pose: bool,
    pub mark_client_unloaded: bool,
}

#[must_use]
pub const fn server_player_death(input: ServerPlayerDeathInput) -> ServerPlayerDeath {
    let try_shoulders = input.shoulder_time.saturating_add(20) < input.game_time;
    ServerPlayerDeath {
        emit_entity_die: true,
        combat_packet_has_message: input.show_death_messages,
        broadcast_real_message: input.show_death_messages
            && !matches!(input.team_visibility, TeamDeathVisibility::Never),
        try_shoulders,
        shoulder_attempts: if try_shoulders { 2 } else { 0 },
        forgive_neutral_mobs: input.forgive_dead_players,
        award_death_objectives: true,
        award_kill_credit_score: input.kill_credit_present,
        attempt_wither_rose: input.kill_credit_present,
        run_items_and_xp: !input.spectator,
        reset_death_stats: true,
        clear_fire_and_frozen: true,
        record_last_death_location: true,
        recheck_combat: true,
        broadcast_event: 3,
        set_dead: false,
        set_dying_pose: false,
        mark_client_unloaded: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerPlayerDeathInput {
    pub show_death_messages: bool,
    pub team_visibility: TeamDeathVisibility,
    pub shoulder_time: u64,
    pub game_time: u64,
    pub forgive_dead_players: bool,
    pub spectator: bool,
    pub kill_credit_present: bool,
}

#[must_use]
pub fn nonserver_player_death_velocity(
    source_present: bool,
    hurt_direction: f32,
    yaw: f32,
) -> Vector3 {
    if !source_present {
        return Vector3::new(0.0, 0.1, 0.0);
    }
    let angle = (hurt_direction + yaw) * 0.017_453_292;
    Vector3::new(
        f64::from(-angle.cos() * 0.1),
        f64::from(0.1_f32),
        f64::from(-angle.sin() * 0.1),
    )
}

#[must_use]
pub const fn dragon_killing_blow(sitting: bool) -> (bool, f32) {
    (!sitting, if sitting { 0.0 } else { 1.0 })
}

#[must_use]
pub const fn death_message_radius() -> (i32, i32, i32) {
    (32, 10, 32)
}

#[must_use]
pub const fn radians_per_degree() -> f32 {
    PI / 180.0
}
