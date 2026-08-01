//! Client particle distribution and presentation filtering.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleSetting {
    All,
    Decreased,
    Minimal,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParticleOptionDraws {
    pub one_in_ten: u32,
    pub one_in_three: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticlePacket {
    pub position: [f64; 3],
    pub spread: [f32; 3],
    pub speed: f32,
    pub count: i32,
    pub override_limiter: bool,
    pub always_show: bool,
    pub type_overrides_limiter: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleAttempt {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleResult {
    pub attempts: Vec<ParticleAttempt>,
    pub gaussian_draws: usize,
    pub option_draws: usize,
    pub logged_failures: usize,
    pub stopped_after_failure: bool,
}

#[must_use]
pub fn present_particles(
    packet: ParticlePacket,
    setting: ParticleSetting,
    camera_distance_squared: f64,
    gaussian_draws: &[[f64; 6]],
    option_draws: &[ParticleOptionDraws],
    failing_attempt: Option<usize>,
) -> ParticleResult {
    if packet.count == 0 {
        let failed = failing_attempt == Some(0);
        let (created, draws) = if failed {
            (false, 0)
        } else {
            should_create(
                packet,
                setting,
                camera_distance_squared,
                option_draws.first(),
            )
        };
        return ParticleResult {
            attempts: vec![ParticleAttempt {
                position: packet.position,
                velocity: [
                    f64::from(packet.speed * packet.spread[0]),
                    f64::from(packet.speed * packet.spread[1]),
                    f64::from(packet.speed * packet.spread[2]),
                ],
                created,
            }],
            gaussian_draws: 0,
            option_draws: draws,
            logged_failures: usize::from(failed),
            stopped_after_failure: false,
        };
    }

    let count = packet.count.max(0) as usize;
    let mut attempts = Vec::with_capacity(count);
    let mut used_gaussians = 0;
    let mut used_options = 0;
    let mut logged_failures = 0;
    let mut stopped_after_failure = false;
    for index in 0..count {
        let draws = gaussian_draws.get(index).copied().unwrap_or([0.0; 6]);
        used_gaussians += 6;
        if failing_attempt == Some(index) {
            logged_failures += 1;
            stopped_after_failure = true;
            break;
        }
        let (created, draws_used) = should_create(
            packet,
            setting,
            camera_distance_squared,
            option_draws.get(index),
        );
        used_options += draws_used;
        attempts.push(ParticleAttempt {
            position: [
                packet.position[0] + draws[0] * f64::from(packet.spread[0]),
                packet.position[1] + draws[1] * f64::from(packet.spread[1]),
                packet.position[2] + draws[2] * f64::from(packet.spread[2]),
            ],
            velocity: [
                draws[3] * f64::from(packet.speed),
                draws[4] * f64::from(packet.speed),
                draws[5] * f64::from(packet.speed),
            ],
            created,
        });
    }
    ParticleResult {
        attempts,
        gaussian_draws: used_gaussians,
        option_draws: used_options,
        logged_failures,
        stopped_after_failure,
    }
}

fn should_create(
    packet: ParticlePacket,
    mut setting: ParticleSetting,
    camera_distance_squared: f64,
    draws: Option<&ParticleOptionDraws>,
) -> (bool, usize) {
    let draws = draws.copied().unwrap_or_default();
    let mut used = 0;
    if packet.always_show && setting == ParticleSetting::Minimal {
        used += 1;
        if draws.one_in_ten % 10 == 0 {
            setting = ParticleSetting::Decreased;
        }
    }
    if setting == ParticleSetting::Decreased {
        used += 1;
        if draws.one_in_three % 3 == 0 {
            setting = ParticleSetting::Minimal;
        }
    }
    let override_limiter = packet.override_limiter || packet.type_overrides_limiter;
    (
        override_limiter
            || (camera_distance_squared <= 1_024.0 && setting != ParticleSetting::Minimal),
        used,
    )
}
