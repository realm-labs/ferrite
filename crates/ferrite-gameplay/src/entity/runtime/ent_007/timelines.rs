//! Common, Creaking, Ender Dragon, and post-player-death projectile timelines.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalReason {
    Killed,
    Discarded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommonDeathTick {
    pub death_time: u8,
    pub broadcast_event: Option<u8>,
    pub remove: Option<RemovalReason>,
    pub poof_particles: u8,
    pub gaussian_draws: u8,
    pub position_draws: u8,
}

#[must_use]
pub const fn common_death_tick(death_time: u8, server_side: bool) -> CommonDeathTick {
    let death_time = death_time.saturating_add(1);
    let remove = server_side && death_time >= 20;
    CommonDeathTick {
        death_time,
        broadcast_event: if remove { Some(60) } else { None },
        remove: if remove {
            Some(RemovalReason::Killed)
        } else {
            None
        },
        poof_particles: if remove { 20 } else { 0 },
        gaussian_draws: if remove { 60 } else { 0 },
        position_draws: if remove { 60 } else { 0 },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreakingDeathTick {
    pub use_common_timeline: bool,
    pub death_time: u8,
    pub pale_oak_particles: u8,
    pub heart_particles: u8,
    pub play_death_sound: bool,
    pub particle_spread_fraction: f32,
    pub particle_speed: f32,
    pub remove: Option<RemovalReason>,
}

#[must_use]
pub const fn creaking_death_tick(
    heart_bound: bool,
    tearing_down: bool,
    death_time: u8,
    server_side: bool,
) -> CreakingDeathTick {
    if !heart_bound || !tearing_down {
        return CreakingDeathTick {
            use_common_timeline: true,
            death_time,
            pale_oak_particles: 0,
            heart_particles: 0,
            play_death_sound: false,
            particle_spread_fraction: 0.0,
            particle_speed: 0.0,
            remove: None,
        };
    }
    let death_time = death_time.saturating_add(1);
    let remove = server_side && death_time > 45;
    CreakingDeathTick {
        use_common_timeline: false,
        death_time,
        pale_oak_particles: if remove { 100 } else { 0 },
        heart_particles: if remove { 10 } else { 0 },
        play_death_sound: remove,
        particle_spread_fraction: if remove { 0.8 } else { 0.0 },
        particle_speed: if remove { 0.25 } else { 0.0 },
        remove: if remove {
            Some(RemovalReason::Discarded)
        } else {
            None
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonDeathTick {
    pub death_time: u16,
    pub update_fight: bool,
    pub particle: bool,
    pub particle_draws: u8,
    pub upward_motion: f64,
    pub global_event: Option<u16>,
    pub xp: u32,
    pub notify_fight: bool,
    pub remove: Option<RemovalReason>,
    pub emit_entity_die_after_remove: bool,
}

#[must_use]
pub fn dragon_death_tick(
    death_time: u16,
    server_side: bool,
    silent: bool,
    mob_drops: bool,
    first_dragon_kill: bool,
) -> DragonDeathTick {
    let death_time = death_time.saturating_add(1);
    let reward = if first_dragon_kill { 12_000 } else { 500 };
    let periodic_xp = u32::from(mob_drops && death_time > 150 && death_time.is_multiple_of(5))
        * (reward as f64 * 0.08).floor() as u32;
    let final_xp = u32::from(mob_drops && death_time == 200) * (reward as f64 * 0.2).floor() as u32;
    let xp = periodic_xp + final_xp;
    let finish = server_side && death_time == 200;
    DragonDeathTick {
        death_time,
        update_fight: true,
        particle: (180..=200).contains(&death_time),
        particle_draws: if (180..=200).contains(&death_time) {
            3
        } else {
            0
        },
        upward_motion: f64::from(0.1_f32),
        global_event: if server_side && death_time == 1 && !silent {
            Some(1028)
        } else {
            None
        },
        xp,
        notify_fight: finish,
        remove: if finish {
            Some(RemovalReason::Killed)
        } else {
            None
        },
        emit_entity_die_after_remove: finish,
    }
}

#[must_use]
pub const fn pearl_vanishes_before_motion(
    owner_dead: bool,
    owner_won_game: bool,
    server_player_owner: bool,
    vanish_rule: bool,
) -> bool {
    owner_dead && !owner_won_game && server_player_owner && vanish_rule
}
