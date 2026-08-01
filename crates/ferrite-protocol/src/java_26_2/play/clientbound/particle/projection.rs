use crate::java_26_2::play::clientbound::entity_effects::particle::Particle;
use crate::java_26_2::play::clientbound::particle::packet::LevelParticles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleSetting {
    All,
    Decreased,
    Minimal,
}

pub trait ParticleRandom {
    fn next_gaussian(&mut self) -> f64;
    fn next_bounded(&mut self, bound: u32) -> u32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleProviderOutcome {
    Created,
    Missing,
    Fault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleAttemptOutcome {
    Created,
    MissingProvider,
    Rejected,
    Fault,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleAttempt {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub outcome: ParticleAttemptOutcome,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProjectionResult {
    pub attempts: Vec<ParticleAttempt>,
    pub gaussian_draws: usize,
    pub option_draws: usize,
    pub provider_calls: usize,
    pub logged_failures: usize,
    pub abandoned_after_fault: bool,
}

pub fn project_particles(
    packet: &LevelParticles,
    setting: ParticleSetting,
    camera_distance_squared: f64,
    random: &mut impl ParticleRandom,
    mut provider: impl FnMut(&Particle, [f64; 3], [f64; 3]) -> ParticleProviderOutcome,
) -> ParticleProjectionResult {
    let mut result = ParticleProjectionResult {
        attempts: Vec::new(),
        gaussian_draws: 0,
        option_draws: 0,
        provider_calls: 0,
        logged_failures: 0,
        abandoned_after_fault: false,
    };
    if packet.count < 0 {
        return result;
    }
    if packet.count == 0 {
        let position = [packet.position.x, packet.position.y, packet.position.z];
        let velocity = [
            f64::from(packet.max_speed * packet.spread[0]),
            f64::from(packet.max_speed * packet.spread[1]),
            f64::from(packet.max_speed * packet.spread[2]),
        ];
        project_attempt(
            packet,
            setting,
            camera_distance_squared,
            random,
            &mut provider,
            ParticleSample { position, velocity },
            &mut result,
        );
        return result;
    }

    for _ in 0..packet.count {
        let draws = [
            random.next_gaussian(),
            random.next_gaussian(),
            random.next_gaussian(),
            random.next_gaussian(),
            random.next_gaussian(),
            random.next_gaussian(),
        ];
        result.gaussian_draws += 6;
        let position = [
            packet.position.x + draws[0] * f64::from(packet.spread[0]),
            packet.position.y + draws[1] * f64::from(packet.spread[1]),
            packet.position.z + draws[2] * f64::from(packet.spread[2]),
        ];
        let velocity = [
            draws[3] * f64::from(packet.max_speed),
            draws[4] * f64::from(packet.max_speed),
            draws[5] * f64::from(packet.max_speed),
        ];
        project_attempt(
            packet,
            setting,
            camera_distance_squared,
            random,
            &mut provider,
            ParticleSample { position, velocity },
            &mut result,
        );
        if result.abandoned_after_fault {
            break;
        }
    }
    result
}

#[derive(Debug, Clone, Copy)]
struct ParticleSample {
    position: [f64; 3],
    velocity: [f64; 3],
}

fn project_attempt(
    packet: &LevelParticles,
    setting: ParticleSetting,
    camera_distance_squared: f64,
    random: &mut impl ParticleRandom,
    provider: &mut impl FnMut(&Particle, [f64; 3], [f64; 3]) -> ParticleProviderOutcome,
    sample: ParticleSample,
    result: &mut ParticleProjectionResult,
) {
    let admitted = particle_admitted(
        packet,
        setting,
        camera_distance_squared,
        random,
        &mut result.option_draws,
    );
    let outcome = if admitted {
        result.provider_calls += 1;
        match provider(&packet.particle, sample.position, sample.velocity) {
            ParticleProviderOutcome::Created => ParticleAttemptOutcome::Created,
            ParticleProviderOutcome::Missing => ParticleAttemptOutcome::MissingProvider,
            ParticleProviderOutcome::Fault => {
                result.logged_failures += 1;
                result.abandoned_after_fault = true;
                ParticleAttemptOutcome::Fault
            }
        }
    } else {
        ParticleAttemptOutcome::Rejected
    };
    result.attempts.push(ParticleAttempt {
        position: sample.position,
        velocity: sample.velocity,
        outcome,
    });
}

fn particle_admitted(
    packet: &LevelParticles,
    mut setting: ParticleSetting,
    camera_distance_squared: f64,
    random: &mut impl ParticleRandom,
    option_draws: &mut usize,
) -> bool {
    if packet.always_show && setting == ParticleSetting::Minimal {
        *option_draws += 1;
        if random.next_bounded(10) == 0 {
            setting = ParticleSetting::Decreased;
        }
    }
    if setting == ParticleSetting::Decreased {
        *option_draws += 1;
        if random.next_bounded(3) == 0 {
            setting = ParticleSetting::Minimal;
        }
    }
    packet.override_limiter
        || type_overrides_limiter(packet.particle.raw_type)
        || (camera_distance_squared <= 1_024.0 && setting != ParticleSetting::Minimal)
}

#[must_use]
pub const fn type_overrides_limiter(raw_type: i32) -> bool {
    matches!(
        raw_type,
        2 | 7
            | 8
            | 9
            | 10
            | 14
            | 24
            | 29
            | 30
            | 31
            | 33
            | 34
            | 35
            | 45
            | 46
            | 55
            | 66
            | 72
            | 73
            | 74
            | 84
            | 85
            | 106
            | 107
            | 108
            | 109
            | 110
            | 111
            | 115
            | 116
            | 117
            | 119
    )
}
