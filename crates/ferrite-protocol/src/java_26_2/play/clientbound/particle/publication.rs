use crate::java_26_2::play::clientbound::entity_effects::particle::Particle;
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::play::clientbound::particle::packet::LevelParticles;

#[derive(Debug, Clone, PartialEq)]
pub struct AuthoredParticle {
    pub override_limiter: bool,
    pub always_show: bool,
    pub position: Vector3,
    pub spread: [f64; 3],
    pub max_speed: f64,
    pub count: i32,
    pub particle: Particle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleViewer {
    pub id: u64,
    pub level: u64,
    pub block_position: [i32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleDelivery {
    pub recipient: u64,
    pub packet: LevelParticles,
}

#[must_use]
pub fn publish_particles(
    source_level: u64,
    authored: &AuthoredParticle,
    viewers: &[ParticleViewer],
) -> Vec<ParticleDelivery> {
    let packet = packet_from_authored(authored);
    viewers
        .iter()
        .filter(|viewer| viewer_is_in_audience(source_level, authored, viewer))
        .map(|viewer| ParticleDelivery {
            recipient: viewer.id,
            packet: packet.clone(),
        })
        .collect()
}

#[must_use]
pub fn publish_particle_to(
    source_level: u64,
    authored: &AuthoredParticle,
    viewer: ParticleViewer,
) -> Option<LevelParticles> {
    viewer_is_in_audience(source_level, authored, &viewer).then(|| packet_from_authored(authored))
}

fn packet_from_authored(authored: &AuthoredParticle) -> LevelParticles {
    LevelParticles {
        override_limiter: authored.override_limiter,
        always_show: authored.always_show,
        position: authored.position,
        spread: authored.spread.map(|value| value as f32),
        max_speed: authored.max_speed as f32,
        count: authored.count,
        particle: authored.particle.clone(),
    }
}

fn viewer_is_in_audience(
    source_level: u64,
    authored: &AuthoredParticle,
    viewer: &ParticleViewer,
) -> bool {
    if viewer.level != source_level {
        return false;
    }
    let range = if authored.override_limiter {
        512.0
    } else {
        32.0
    };
    let center = viewer
        .block_position
        .map(|coordinate| f64::from(coordinate) + 0.5);
    squared_distance(center, authored.position) < range * range
}

fn squared_distance(left: [f64; 3], right: Vector3) -> f64 {
    (left[0] - right.x).powi(2) + (left[1] - right.y).powi(2) + (left[2] - right.z).powi(2)
}
