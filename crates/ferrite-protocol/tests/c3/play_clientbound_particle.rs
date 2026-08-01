use std::collections::{BTreeSet, VecDeque};

use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::codec::EntityEffectsCodecError;
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::particle::{
    PARTICLE_TYPE_COUNT, Particle, ParticleOptions, ParticleVector, PositionSource,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use ferrite_protocol::java_26_2::play::clientbound::particle::packet::LevelParticles;
use ferrite_protocol::java_26_2::play::clientbound::particle::projection::{
    ParticleAttemptOutcome, ParticleProviderOutcome, ParticleRandom, ParticleSetting,
    project_particles, type_overrides_limiter,
};
use ferrite_protocol::java_26_2::play::clientbound::particle::publication::{
    AuthoredParticle, ParticleViewer, publish_particle_to, publish_particles,
};
use ferrite_protocol::java_26_2::play::clientbound::projection::{
    PlayEntryProjection, PlayProjectionError,
};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::item::{DataComponentPatch, ItemStackTemplate};
use ferrite_protocol::java_26_2::play::registry::{DATA_COMPONENT_TYPE, ITEM, PlayRegistries};
use ferrite_protocol::java_26_2::value::identifier::Identifier;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(ITEM),
        vec![id("minecraft:stone"), id("minecraft:diamond")],
    );
    registries.insert(id(DATA_COMPONENT_TYPE), Vec::new());
    registries
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn particle(raw_type: i32) -> Particle {
    let options = match raw_type {
        1 | 2 | 36 | 118 | 122 => ParticleOptions::BlockState(32_365),
        7 | 10 => ParticleOptions::Geyser { water_blocks: -7 },
        8 | 9 => ParticleOptions::GeyserBase {
            water_blocks: 11,
            burst_impulse_base: f32::from_bits(1),
        },
        15 | 45 => ParticleOptions::Power(f32::INFINITY),
        21 => ParticleOptions::Dust {
            color: i32::MIN,
            scale: 1.25,
        },
        22 => ParticleOptions::DustTransition {
            from_color: i32::MIN,
            to_color: i32::MAX,
            scale: 2.5,
        },
        23 | 53 => ParticleOptions::Spell {
            color: -1,
            power: f32::NEG_INFINITY,
        },
        28 | 43 | 49 => ParticleOptions::Color(i32::MIN),
        54 => ParticleOptions::Item(ItemStackTemplate {
            item: id("minecraft:diamond"),
            count: -17,
            components: DataComponentPatch::default(),
        }),
        55 => ParticleOptions::Vibration {
            destination: PositionSource::Entity {
                entity_id: i32::MIN,
                y_offset: f32::NEG_INFINITY,
            },
            arrival_ticks: -1,
        },
        56 => ParticleOptions::Trail {
            target: ParticleVector {
                x: f64::from_bits(1),
                y: f64::INFINITY,
                z: f64::NEG_INFINITY,
            },
            color: i32::MIN,
            duration: i32::MAX,
        },
        112 => ParticleOptions::Shriek(i32::MIN),
        _ => ParticleOptions::Simple,
    };
    Particle { raw_type, options }
}

fn packet(raw_type: i32) -> LevelParticles {
    LevelParticles {
        override_limiter: false,
        always_show: false,
        position: Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        spread: [2.0, 3.0, 4.0],
        max_speed: 0.5,
        count: 0,
        particle: particle(raw_type),
    }
}

fn assert_roundtrip(packet: LevelParticles) {
    let registries = registries();
    let packet = PlayClientboundPacket::LevelParticles(Box::new(packet));
    let encoded = encode_packet(&packet, &registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(&registries)).unwrap(),
        packet
    );
}

#[derive(Debug, Default)]
struct ScriptedRandom {
    gaussians: VecDeque<f64>,
    bounded: VecDeque<u32>,
    gaussian_calls: usize,
    bounded_calls: Vec<u32>,
}

impl ParticleRandom for ScriptedRandom {
    fn next_gaussian(&mut self) -> f64 {
        self.gaussian_calls += 1;
        self.gaussians.pop_front().unwrap_or(0.0)
    }

    fn next_bounded(&mut self, bound: u32) -> u32 {
        self.bounded_calls.push(bound);
        self.bounded.pop_front().unwrap_or(1) % bound
    }
}

#[test]
fn c3_gold_clientbound_level_particles_locks_the_empty_simple_body() {
    let registries = PlayRegistries::default();
    let empty = LevelParticles {
        override_limiter: false,
        always_show: false,
        position: Vector3::default(),
        spread: [0.0; 3],
        max_speed: 0.0,
        count: 0,
        particle: particle(0),
    };
    let mut expected = vec![0x2f];
    expected.extend([0; 47]);
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::LevelParticles(Box::new(empty)),
            &registries,
        )
        .unwrap(),
        expected
    );
}

#[test]
fn c3_particle_codec_roundtrips_all_125_options_and_ieee_signed_domains() {
    for raw_type in 0..PARTICLE_TYPE_COUNT {
        let mut value = packet(raw_type);
        value.override_limiter = true;
        value.always_show = true;
        value.position = Vector3 {
            x: f64::from_bits(1),
            y: f64::INFINITY,
            z: f64::NEG_INFINITY,
        };
        value.spread = [f32::from_bits(1), f32::INFINITY, f32::NEG_INFINITY];
        value.max_speed = -0.0;
        value.count = i32::MIN;
        assert_roundtrip(value);
    }
}

#[test]
fn c3_particle_codec_normalizes_booleans_and_faults_mapping_and_framing() {
    let registries = PlayRegistries::default();
    let mut noncanonical = vec![0x2f, 2, 0xff];
    noncanonical.extend([0; 45]);
    let decoded = decode_packet(&noncanonical, context(&registries)).unwrap();
    let normalized = encode_packet(&decoded, &registries).unwrap();
    assert_eq!(&normalized[..3], &[0x2f, 1, 1]);

    let mut unknown = vec![0x2f];
    unknown.extend([0; 46]);
    unknown.push(125);
    assert_eq!(
        decode_packet(&unknown, context(&registries)),
        Err(PlayClientboundCodecError::EntityEffects(
            EntityEffectsCodecError::UnknownParticleType { raw_type: 125 }
        ))
    );
    let mut mismatched = packet(0);
    mismatched.particle.options = ParticleOptions::Power(1.0);
    assert!(matches!(
        encode_packet(
            &PlayClientboundPacket::LevelParticles(Box::new(mismatched)),
            &registries,
        ),
        Err(PlayClientboundCodecError::EntityEffects(
            EntityEffectsCodecError::ParticleOptionsMismatch { raw_type: 0 }
        ))
    ));
    assert!(decode_packet(&[0x2f], context(&registries)).is_err());
    let mut trailing = encode_packet(
        &PlayClientboundPacket::LevelParticles(Box::new(packet(0))),
        &registries,
    )
    .unwrap();
    trailing.push(0);
    assert!(decode_packet(&trailing, context(&registries)).is_err());
}

#[test]
fn c3_particle_count_forms_preserve_float_products_and_gaussian_order() {
    let mut random = ScriptedRandom::default();
    let negative = project_particles(
        &LevelParticles {
            count: -1,
            ..packet(0)
        },
        ParticleSetting::All,
        0.0,
        &mut random,
        |_, _, _| ParticleProviderOutcome::Created,
    );
    assert!(negative.attempts.is_empty());
    assert_eq!(random.gaussian_calls, 0);

    let zero = project_particles(
        &packet(0),
        ParticleSetting::All,
        0.0,
        &mut random,
        |_, _, _| ParticleProviderOutcome::Created,
    );
    assert_eq!(zero.gaussian_draws, 0);
    assert_eq!(zero.attempts[0].position, [1.0, 2.0, 3.0]);
    assert_eq!(zero.attempts[0].velocity, [1.0, 1.5, 2.0]);

    random.gaussians = VecDeque::from([
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, -1.0, -2.0, -3.0, -4.0, -5.0, -6.0,
    ]);
    let positive = project_particles(
        &LevelParticles {
            count: 2,
            ..packet(0)
        },
        ParticleSetting::All,
        0.0,
        &mut random,
        |_, _, _| ParticleProviderOutcome::Created,
    );
    assert_eq!(positive.gaussian_draws, 12);
    assert_eq!(positive.attempts[0].position, [3.0, 8.0, 15.0]);
    assert_eq!(positive.attempts[0].velocity, [2.0, 2.5, 3.0]);
    assert_eq!(positive.attempts[1].position, [-1.0, -4.0, -9.0]);
}

#[test]
fn c3_particle_limiter_uses_exact_overrides_distance_and_setting_draws() {
    let expected = BTreeSet::from([
        2, 7, 8, 9, 10, 14, 24, 29, 30, 31, 33, 34, 35, 45, 46, 55, 66, 72, 73, 74, 84, 85, 106,
        107, 108, 109, 110, 111, 115, 116, 117, 119,
    ]);
    assert_eq!(
        (0..PARTICLE_TYPE_COUNT)
            .filter(|raw_type| type_overrides_limiter(*raw_type))
            .collect::<BTreeSet<_>>(),
        expected
    );

    let mut random = ScriptedRandom::default();
    let boundary = project_particles(
        &packet(0),
        ParticleSetting::All,
        1_024.0,
        &mut random,
        |_, _, _| ParticleProviderOutcome::Created,
    );
    assert_eq!(
        boundary.attempts[0].outcome,
        ParticleAttemptOutcome::Created
    );
    let distant = project_particles(
        &packet(0),
        ParticleSetting::All,
        1_024.000_001,
        &mut random,
        |_, _, _| ParticleProviderOutcome::Created,
    );
    assert_eq!(
        distant.attempts[0].outcome,
        ParticleAttemptOutcome::Rejected
    );

    random.bounded = VecDeque::from([0, 1]);
    let promoted = project_particles(
        &LevelParticles {
            always_show: true,
            ..packet(0)
        },
        ParticleSetting::Minimal,
        0.0,
        &mut random,
        |_, _, _| ParticleProviderOutcome::Created,
    );
    assert_eq!(promoted.option_draws, 2);
    assert_eq!(
        promoted.attempts[0].outcome,
        ParticleAttemptOutcome::Created
    );
    assert_eq!(random.bounded_calls, [10, 3]);

    random.bounded = VecDeque::from([1]);
    let overridden = project_particles(
        &LevelParticles {
            override_limiter: true,
            always_show: true,
            ..packet(0)
        },
        ParticleSetting::Minimal,
        f64::INFINITY,
        &mut random,
        |_, _, _| ParticleProviderOutcome::Created,
    );
    assert_eq!(overridden.option_draws, 1);
    assert_eq!(
        overridden.attempts[0].outcome,
        ParticleAttemptOutcome::Created
    );
}

#[test]
fn c3_particle_provider_null_fault_and_rejection_keep_attempt_order_and_rng_prefix() {
    let mut random = ScriptedRandom {
        gaussians: VecDeque::from([0.0; 18]),
        ..ScriptedRandom::default()
    };
    let mut calls = 0;
    let result = project_particles(
        &LevelParticles {
            count: 3,
            ..packet(0)
        },
        ParticleSetting::All,
        0.0,
        &mut random,
        |_, _, _| {
            calls += 1;
            match calls {
                1 => ParticleProviderOutcome::Missing,
                2 => ParticleProviderOutcome::Fault,
                _ => ParticleProviderOutcome::Created,
            }
        },
    );
    assert_eq!(result.gaussian_draws, 12);
    assert_eq!(result.provider_calls, 2);
    assert_eq!(result.logged_failures, 1);
    assert!(result.abandoned_after_fault);
    assert_eq!(
        result
            .attempts
            .iter()
            .map(|attempt| attempt.outcome)
            .collect::<Vec<_>>(),
        [
            ParticleAttemptOutcome::MissingProvider,
            ParticleAttemptOutcome::Fault,
        ]
    );

    let rejected = project_particles(
        &packet(0),
        ParticleSetting::Minimal,
        0.0,
        &mut ScriptedRandom::default(),
        |_, _, _| panic!("rejected attempts do not reach the provider"),
    );
    assert_eq!(rejected.provider_calls, 0);
    assert_eq!(
        rejected.attempts[0].outcome,
        ParticleAttemptOutcome::Rejected
    );
}

#[test]
fn c3_particle_publication_uses_block_centers_strict_ranges_and_one_float_narrowing() {
    let authored = AuthoredParticle {
        override_limiter: false,
        always_show: true,
        position: Vector3 {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        },
        spread: [1.000_000_06, f64::INFINITY, f64::NEG_INFINITY],
        max_speed: f64::from_bits(1),
        count: i32::MIN,
        particle: particle(0),
    };
    let viewers = [
        ParticleViewer {
            id: 1,
            level: 7,
            block_position: [31, 0, 0],
        },
        ParticleViewer {
            id: 2,
            level: 7,
            block_position: [32, 0, 0],
        },
        ParticleViewer {
            id: 3,
            level: 8,
            block_position: [0, 0, 0],
        },
    ];
    let deliveries = publish_particles(7, &authored, &viewers);
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].recipient, 1);
    assert_eq!(deliveries[0].packet.position, authored.position);
    assert_eq!(deliveries[0].packet.spread[0], authored.spread[0] as f32);
    assert_eq!(deliveries[0].packet.max_speed, authored.max_speed as f32);

    let mut overridden = authored;
    overridden.override_limiter = true;
    let at_512 = ParticleViewer {
        id: 4,
        level: 7,
        block_position: [512, 0, 0],
    };
    assert!(publish_particle_to(7, &overridden, at_512).is_none());
    assert!(
        publish_particle_to(
            7,
            &overridden,
            ParticleViewer {
                block_position: [511, 0, 0],
                ..at_512
            }
        )
        .is_some()
    );
}

#[test]
fn c3_particle_projection_requires_an_installed_play_level() {
    assert_eq!(
        PlayEntryProjection::default()
            .apply(PlayClientboundPacket::LevelParticles(Box::new(packet(0)))),
        Err(PlayProjectionError::LevelNotInstalled)
    );
}
