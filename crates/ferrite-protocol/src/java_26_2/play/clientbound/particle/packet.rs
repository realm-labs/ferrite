use crate::java_26_2::play::clientbound::entity_effects::particle::Particle;
use crate::java_26_2::play::clientbound::packet::Vector3;

#[derive(Debug, Clone, PartialEq)]
pub struct LevelParticles {
    pub override_limiter: bool,
    pub always_show: bool,
    pub position: Vector3,
    pub spread: [f32; 3],
    pub max_speed: f32,
    pub count: i32,
    pub particle: Particle,
}
