use ferrite_foundation::coordinate::BlockPos;
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::codec::EntityEffectsCodecError;
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::packet::{
    Explosion, ExplosionParticle, RemoveMobEffect, SoundEventHolder, UpdateMobEffect,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::particle::{
    PARTICLE_IDENTITIES, PARTICLE_TYPE_COUNT, Particle, ParticleOptions, ParticleVector,
    PositionSource, particle_identity,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::projection::{
    EntityEffectsAction, EntityEffectsClientProjection, ExplosionTrackerError, TrackedEffectEntity,
};
use ferrite_protocol::java_26_2::play::clientbound::entity_effects::publication::{
    EFFECT_AUDIENCE, EffectPacketRecipient, EffectPublicationStep, EffectUpdateCause,
    EffectUpdatePublication, RidingEffectStep, dismount_effect_plan, effect_removal_plan,
    effect_update_plan, explosion_recipient, initial_self_effect_replay, mount_effect_plan,
    publish_effect_removal, publish_effect_update, publish_explosion, selected_explosion_particle,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::{PlayClientboundPacket, Vector3};
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::item::{DataComponentPatch, ItemStackTemplate};
use ferrite_protocol::java_26_2::play::registry::{
    DATA_COMPONENT_TYPE, ITEM, MOB_EFFECT, PlayRegistries, SOUND_EVENT,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;

static COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(MOB_EFFECT),
        vec![id("minecraft:speed"), id("minecraft:slowness")],
    );
    registries.insert(
        id(SOUND_EVENT),
        vec![
            id("minecraft:entity.generic.explode"),
            id("minecraft:block.anvil.land"),
        ],
    );
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

fn simple_particle(raw_type: i32) -> Particle {
    Particle {
        raw_type,
        options: ParticleOptions::Simple,
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

fn explosion() -> Explosion {
    Explosion {
        center: Vector3 {
            x: 1.0,
            y: -2.0,
            z: f64::INFINITY,
        },
        radius: 3.0,
        block_count: 600,
        knockback: Some(Vector3 {
            x: 0.25,
            y: -0.5,
            z: 1.0,
        }),
        particle: simple_particle(30),
        sound: SoundEventHolder::Registered(id("minecraft:entity.generic.explode")),
        block_particles: vec![ExplosionParticle {
            particle: simple_particle(66),
            scaling: 0.75,
            speed: 2.0,
            weight: 3,
        }],
    }
}

fn explosion_packet(explosion: Explosion) -> PlayClientboundPacket {
    PlayClientboundPacket::Explosion(Box::new(explosion))
}

fn assert_roundtrip(packet: PlayClientboundPacket, registries: &PlayRegistries) {
    let encoded = encode_packet(&packet, registries).unwrap();
    assert_eq!(
        decode_packet(&encoded, context(registries)).unwrap(),
        packet
    );
}

#[test]
fn c3_gold_entity_effects_locks_all_three_packet_bodies() {
    let registries = registries();
    let direct_explosion = explosion_packet(Explosion {
        center: Vector3::default(),
        radius: 1.0,
        block_count: -1,
        knockback: None,
        particle: simple_particle(30),
        sound: SoundEventHolder::Direct {
            identity: id("minecraft:entity.generic.explode"),
            fixed_range: None,
        },
        block_particles: Vec::new(),
    });
    let mut expected = vec![0x24];
    expected.extend_from_slice(&[0; 24]);
    expected.extend_from_slice(&1.0_f32.to_be_bytes());
    expected.extend_from_slice(&(-1_i32).to_be_bytes());
    expected.extend_from_slice(&[0x00, 0x1e, 0x00, 0x20]);
    expected.extend_from_slice(b"minecraft:entity.generic.explode");
    expected.extend_from_slice(&[0x00, 0x00]);
    assert_eq!(
        encode_packet(&direct_explosion, &registries).unwrap(),
        expected
    );

    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::RemoveMobEffect(RemoveMobEffect {
                entity_id: -1,
                effect: id("minecraft:speed"),
            }),
            &registries,
        )
        .unwrap(),
        [0x4e, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x00]
    );
    assert_eq!(
        encode_packet(
            &PlayClientboundPacket::UpdateMobEffect(UpdateMobEffect {
                entity_id: -1,
                effect: id("minecraft:speed"),
                amplifier: -1,
                duration: -1,
                flags: 0xff,
            }),
            &registries,
        )
        .unwrap(),
        [
            0x84, 0x01, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x00, 0xff, 0xff, 0xff, 0xff, 0x0f, 0xff,
            0xff, 0xff, 0xff, 0x0f, 0xff,
        ]
    );
}

#[test]
fn c3_entity_effect_codecs_preserve_signed_ieee_flags_and_holder_forms() {
    let registries = registries();
    assert_roundtrip(
        PlayClientboundPacket::UpdateMobEffect(UpdateMobEffect {
            entity_id: i32::MIN,
            effect: id("minecraft:slowness"),
            amplifier: i32::MAX,
            duration: i32::MIN,
            flags: 0xff,
        }),
        &registries,
    );
    assert_roundtrip(
        explosion_packet(Explosion {
            sound: SoundEventHolder::Direct {
                identity: id("custom:detonation"),
                fixed_range: Some(f32::NEG_INFINITY),
            },
            ..explosion()
        }),
        &registries,
    );
}

#[test]
fn c3_particle_table_and_all_125_option_shapes_roundtrip() {
    let registries = registries();
    assert_eq!(PARTICLE_IDENTITIES.len(), PARTICLE_TYPE_COUNT as usize);
    assert_eq!(particle_identity(0), Some("minecraft:angry_villager"));
    assert_eq!(particle_identity(124), Some("minecraft:sulfur_cube_goo"));
    assert_eq!(particle_identity(-1), None);
    assert_eq!(particle_identity(125), None);

    for raw_type in 0..PARTICLE_TYPE_COUNT {
        assert_roundtrip(
            explosion_packet(Explosion {
                particle: particle(raw_type),
                block_particles: Vec::new(),
                ..explosion()
            }),
            &registries,
        );
    }

    assert_roundtrip(
        explosion_packet(Explosion {
            particle: Particle {
                raw_type: 55,
                options: ParticleOptions::Vibration {
                    destination: PositionSource::Block(BlockPos::new(-3, 255, 9)),
                    arrival_ticks: 0,
                },
            },
            block_particles: Vec::new(),
            ..explosion()
        }),
        &registries,
    );
}

#[test]
fn c3_entity_effects_reject_invalid_particles_weights_registries_and_framing() {
    let registries = registries();
    let mismatch = explosion_packet(Explosion {
        particle: simple_particle(1),
        ..explosion()
    });
    assert!(matches!(
        encode_packet(&mismatch, &registries),
        Err(PlayClientboundCodecError::EntityEffects(
            EntityEffectsCodecError::ParticleOptionsMismatch { raw_type: 1 }
        ))
    ));
    let invalid_state = explosion_packet(Explosion {
        particle: Particle {
            raw_type: 1,
            options: ParticleOptions::BlockState(32_366),
        },
        ..explosion()
    });
    assert!(matches!(
        encode_packet(&invalid_state, &registries),
        Err(PlayClientboundCodecError::EntityEffects(
            EntityEffectsCodecError::InvalidBlockState { state: 32_366 }
        ))
    ));
    let negative = explosion_packet(Explosion {
        block_particles: vec![ExplosionParticle {
            weight: -1,
            ..explosion().block_particles[0].clone()
        }],
        ..explosion()
    });
    assert!(matches!(
        encode_packet(&negative, &registries),
        Err(PlayClientboundCodecError::EntityEffects(
            EntityEffectsCodecError::NegativeParticleWeight { weight: -1 }
        ))
    ));
    let overflow = explosion_packet(Explosion {
        block_particles: vec![
            ExplosionParticle {
                weight: i32::MAX,
                ..explosion().block_particles[0].clone()
            },
            ExplosionParticle {
                weight: 1,
                ..explosion().block_particles[0].clone()
            },
        ],
        ..explosion()
    });
    assert!(matches!(
        encode_packet(&overflow, &registries),
        Err(PlayClientboundCodecError::EntityEffects(
            EntityEffectsCodecError::ParticleWeightOverflow
        ))
    ));
    let unknown_effect = PlayClientboundPacket::RemoveMobEffect(RemoveMobEffect {
        entity_id: 1,
        effect: id("minecraft:unknown"),
    });
    assert!(encode_packet(&unknown_effect, &registries).is_err());

    let mut unknown_particle = encode_packet(&explosion_packet(explosion()), &registries).unwrap();
    unknown_particle[58] = 125;
    assert!(matches!(
        decode_packet(&unknown_particle, context(&registries)),
        Err(PlayClientboundCodecError::EntityEffects(
            EntityEffectsCodecError::UnknownParticleType { raw_type: 125 }
        ))
    ));
    for malformed in [
        vec![0x24],
        vec![0x4e],
        vec![0x84, 0x01],
        vec![0x4e, 0x00, 0x00, 0x00],
    ] {
        assert!(decode_packet(&malformed, context(&registries)).is_err());
    }
}

#[test]
fn c3_client_effect_projection_gates_clamps_replaces_and_removes() {
    let registries = registries();
    let mut client = EntityEffectsClientProjection::default();
    client.track_entity(
        7,
        TrackedEffectEntity {
            living: true,
            can_be_affected: true,
            ..TrackedEffectEntity::default()
        },
    );
    client.track_entity(
        8,
        TrackedEffectEntity {
            living: false,
            can_be_affected: true,
            ..TrackedEffectEntity::default()
        },
    );
    let effect = id("minecraft:speed");
    let first = PlayClientboundPacket::UpdateMobEffect(UpdateMobEffect {
        entity_id: 7,
        effect: effect.clone(),
        amplifier: -100,
        duration: -1,
        flags: 0xf9,
    });
    assert_eq!(
        client.apply(&first, (0.0, 0.0)),
        EntityEffectsAction::EffectAdded
    );
    let installed = &client.entity(7).unwrap().effects[&effect];
    assert_eq!(installed.amplifier, 0);
    assert!(installed.infinite);
    assert!(installed.has_remaining_tick);
    assert!(installed.flags.ambient);
    assert!(installed.flags.blend);
    assert!(!installed.flags.visible);

    let replacement = PlayClientboundPacket::UpdateMobEffect(UpdateMobEffect {
        entity_id: 7,
        effect: effect.clone(),
        amplifier: 999,
        duration: 0,
        flags: 0x06,
    });
    assert_eq!(
        client.apply(&replacement, (0.0, 0.0)),
        EntityEffectsAction::EffectReplaced
    );
    let installed = &client.entity(7).unwrap().effects[&effect];
    assert_eq!(installed.amplifier, 255);
    assert!(!installed.has_remaining_tick);
    assert!(installed.blend_state.blending);
    assert_eq!(installed.blend_state.replacement_copies, 1);
    assert!(!installed.hidden_effect);

    assert_eq!(
        client.apply(
            &PlayClientboundPacket::UpdateMobEffect(UpdateMobEffect {
                entity_id: 8,
                effect: effect.clone(),
                amplifier: 0,
                duration: 1,
                flags: 0,
            }),
            (0.0, 0.0),
        ),
        EntityEffectsAction::Ignored
    );
    let removal = PlayClientboundPacket::RemoveMobEffect(RemoveMobEffect {
        entity_id: 7,
        effect: effect.clone(),
    });
    assert_eq!(
        client.apply(&removal, (0.0, 0.0)),
        EntityEffectsAction::EffectRemoved
    );
    assert_eq!(
        client.apply(&removal, (0.0, 0.0)),
        EntityEffectsAction::Ignored
    );
    assert_roundtrip(first, &registries);
}

#[test]
fn c3_explosion_projection_orders_sound_particle_tracker_and_knockback() {
    let mut client = EntityEffectsClientProjection::default();
    client.set_player_motion(Vector3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    });
    let packet = explosion_packet(explosion());
    let EntityEffectsAction::ExplosionPresented(presentation) = client.apply(&packet, (0.75, 0.25))
    else {
        panic!("expected explosion presentation");
    };
    assert_eq!(presentation.sound_volume, 4.0);
    assert_eq!(presentation.sound_pitch, 0.77);
    assert_eq!(
        presentation.primary_velocity,
        Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0
        }
    );
    assert_eq!(
        presentation.resulting_player_motion,
        Vector3 {
            x: 1.25,
            y: 1.5,
            z: 4.0,
        }
    );
    assert!(presentation.tracker_queued);
    assert_eq!(client.queued_explosions(), 1);
    assert_eq!(client.tick_tracker(true).unwrap().attempted_samples, 512);
    assert_eq!(client.queued_explosions(), 0);

    let empty_recipe = explosion_packet(Explosion {
        block_particles: Vec::new(),
        ..explosion()
    });
    let EntityEffectsAction::ExplosionPresented(presentation) =
        client.apply(&empty_recipe, (0.0, 0.0))
    else {
        panic!("expected explosion presentation");
    };
    assert!(!presentation.tracker_queued);
    assert_eq!(client.tick_tracker(false).unwrap().attempted_samples, 0);
}

#[test]
fn c3_explosion_tracker_faults_on_signed_block_count_overflow() {
    let mut client = EntityEffectsClientProjection::default();
    for block_count in [i32::MAX, 1] {
        client.apply(
            &explosion_packet(Explosion {
                block_count,
                ..explosion()
            }),
            (0.0, 0.0),
        );
    }
    assert_eq!(
        client.tick_tracker(true),
        Err(ExplosionTrackerError::BlockCountOverflow)
    );
}

#[test]
fn c3_publication_locks_audience_particle_choice_and_effect_flags() {
    assert!(explosion_recipient(4_095.999));
    assert!(!explosion_recipient(4_096.0));
    let small = simple_particle(30);
    let large = simple_particle(29);
    assert_eq!(
        selected_explosion_particle(1.999, true, small.clone(), large.clone()),
        small
    );
    assert_eq!(
        selected_explosion_particle(2.0, false, small.clone(), large.clone()),
        small
    );
    assert_eq!(
        selected_explosion_particle(2.0, true, small, large.clone()),
        large
    );
    assert_eq!(
        EFFECT_AUDIENCE,
        ferrite_protocol::java_26_2::play::clientbound::entity_effects::publication::EffectAudience {
            direct_player_passengers: true,
            self_player: true,
            indirect_passengers: false,
            ordinary_tracking_viewers: false,
        }
    );

    let update = publish_effect_update(EffectUpdatePublication {
        entity_id: 9,
        effect: id("minecraft:speed"),
        amplifier: 3,
        duration: 40,
        ambient: true,
        visible: true,
        show_icon: false,
        self_player_new_effect: true,
    });
    assert!(matches!(
        update,
        PlayClientboundPacket::UpdateMobEffect(UpdateMobEffect { flags: 0x0b, .. })
    ));
    assert!(matches!(
        publish_effect_removal(9, id("minecraft:speed")),
        PlayClientboundPacket::RemoveMobEffect(_)
    ));
    assert!(matches!(
        publish_explosion(explosion()),
        PlayClientboundPacket::Explosion(_)
    ));
}

#[test]
fn c3_effect_publication_preserves_lifecycle_and_riding_order() {
    assert_eq!(
        effect_update_plan(EffectUpdateCause::Added, 2, true),
        vec![
            EffectPublicationStep::MarkParticleMetadataDirty,
            EffectPublicationStep::AddAttributeModifiers,
            EffectPublicationStep::SendUpdate {
                recipient: EffectPacketRecipient::DirectPlayerPassenger(0),
                blend: false,
            },
            EffectPublicationStep::SendUpdate {
                recipient: EffectPacketRecipient::DirectPlayerPassenger(1),
                blend: false,
            },
            EffectPublicationStep::SendUpdate {
                recipient: EffectPacketRecipient::SelfPlayer,
                blend: true,
            },
        ]
    );
    assert_eq!(
        effect_update_plan(EffectUpdateCause::PeriodicDurationRefresh, 0, true),
        vec![
            EffectPublicationStep::MarkParticleMetadataDirty,
            EffectPublicationStep::SendUpdate {
                recipient: EffectPacketRecipient::SelfPlayer,
                blend: false,
            },
        ]
    );
    assert_eq!(
        effect_removal_plan(1, true),
        vec![
            EffectPublicationStep::RemoveAttributeModifiers,
            EffectPublicationStep::SendRemoval {
                recipient: EffectPacketRecipient::DirectPlayerPassenger(0),
            },
            EffectPublicationStep::RefreshAffectedAttributes,
            EffectPublicationStep::SendRemoval {
                recipient: EffectPacketRecipient::SelfPlayer,
            },
        ]
    );

    let effects = vec![id("minecraft:slowness"), id("minecraft:speed")];
    assert_eq!(initial_self_effect_replay(&effects), effects);
    assert_eq!(
        mount_effect_plan(&effects),
        vec![
            RidingEffectStep::PositionAndChallenge,
            RidingEffectStep::UpdateVehicleEffect {
                effect: id("minecraft:slowness"),
                blend: false,
            },
            RidingEffectStep::UpdateVehicleEffect {
                effect: id("minecraft:speed"),
                blend: false,
            },
            RidingEffectStep::SendPassengerList,
        ]
    );
    assert_eq!(
        dismount_effect_plan(&effects),
        vec![
            RidingEffectStep::RemoveVehicleEffect {
                effect: id("minecraft:slowness"),
            },
            RidingEffectStep::RemoveVehicleEffect {
                effect: id("minecraft:speed"),
            },
            RidingEffectStep::SendPassengerList,
        ]
    );
}
