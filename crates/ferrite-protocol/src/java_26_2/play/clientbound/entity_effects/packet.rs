use crate::java_26_2::play::clientbound::entity_effects::particle::Particle;
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub enum SoundEventHolder {
    Direct {
        identity: Identifier,
        fixed_range: Option<f32>,
    },
    Registered(Identifier),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionParticle {
    pub particle: Particle,
    pub scaling: f32,
    pub speed: f32,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Explosion {
    pub center: Vector3,
    pub radius: f32,
    pub block_count: i32,
    pub knockback: Option<Vector3>,
    pub particle: Particle,
    pub sound: SoundEventHolder,
    pub block_particles: Vec<ExplosionParticle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveMobEffect {
    pub entity_id: i32,
    pub effect: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMobEffect {
    pub entity_id: i32,
    pub effect: Identifier,
    pub amplifier: i32,
    pub duration: i32,
    pub flags: u8,
}
