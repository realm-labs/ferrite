use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::portal::end_portal::{
    CreditsContact, Direction, END_PORTAL_BLOCK_ENTITY_CONTRACT, END_PORTAL_COLLISION_SHAPE,
    END_PORTAL_COLORS, END_PORTAL_EVENT, END_PORTAL_PROPERTIES, END_PORTAL_RENDER_PIPELINE,
    END_PORTAL_SCALE_TRANSLATE, END_SPAWN, EndPortalDesiredBlock, SavedRespawn, animate_tick,
    clone_stack, contact_shape_contains, end_portal_credits_contact, end_portal_shader_layer,
    enter_end, entering_end_platform, leave_end, should_render_block_entity_face,
    special_model_quads, world_block_entity_quads,
};
use ferrite_world::generation::portal::{Rotation, Vec3};

#[test]
fn end_portal_block_surface_locks_shape_physics_clone_and_fluid_gates() {
    assert_eq!(END_PORTAL_COLLISION_SHAPE, None);
    assert!(contact_shape_contains([0.5, 6.0 / 16.0, 0.5]));
    assert!(contact_shape_contains([0.5, 12.0 / 16.0, 0.5]));
    assert!(!contact_shape_contains([0.5, 6.0 / 16.0 - 0.0001, 0.5]));
    assert!(!contact_shape_contains([0.5, 12.0 / 16.0 + 0.0001, 0.5]));
    assert_eq!(END_PORTAL_PROPERTIES.light, 15);
    assert_eq!(END_PORTAL_PROPERTIES.hardness, -1.0);
    assert_eq!(END_PORTAL_PROPERTIES.explosion_resistance, 3_600_000.0);
    assert_eq!(
        (
            END_PORTAL_PROPERTIES.piston_blocks,
            END_PORTAL_PROPERTIES.has_loot_table,
            END_PORTAL_PROPERTIES.replaceable_by_fluid,
            END_PORTAL_PROPERTIES.ordinary_block_model,
        ),
        (true, false, false, false)
    );
    assert_eq!(clone_stack(false), None);
    assert_eq!(clone_stack(true), None);
}

#[test]
fn animate_tick_consumes_two_doubles_and_emits_one_stationary_smoke_particle() {
    let mut values = [0.25, 0.75].into_iter();
    let particle = animate_tick(BlockPos::new(10, 20, 30), || values.next().unwrap());
    assert_eq!(
        particle.position,
        Vec3 {
            x: 10.25,
            y: 20.8,
            z: 30.75
        }
    );
    assert_eq!(particle.velocity, Vec3::ZERO);
    assert_eq!(particle.random_draws, 2);
    assert_eq!(values.next(), None);
}

#[test]
fn block_entity_and_special_model_emit_exact_faces_transform_and_pipeline() {
    for direction in [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ] {
        assert_eq!(
            should_render_block_entity_face(direction),
            matches!(direction, Direction::Down | Direction::Up)
        );
    }
    let world = world_block_entity_quads();
    assert_eq!(world.len(), 2);
    assert!(world[0].vertices.iter().all(|vertex| vertex[1] == 0.375));
    assert!(world[1].vertices.iter().all(|vertex| vertex[1] == 0.75));
    let special = special_model_quads();
    assert_eq!(special.len(), 6);
    assert_eq!(
        special
            .iter()
            .map(|quad| quad.direction)
            .collect::<Vec<_>>(),
        [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ]
    );
    assert_eq!(END_PORTAL_RENDER_PIPELINE.portal_layers, 15);
    assert_eq!(
        (
            END_PORTAL_RENDER_PIPELINE.position_only,
            END_PORTAL_RENDER_PIPELINE.default_depth_state,
            END_PORTAL_RENDER_PIPELINE.applies_environment_fog,
            END_PORTAL_RENDER_PIPELINE.ignores_light_overlay_foil_outline,
        ),
        (true, true, true, true)
    );
    assert!(
        END_PORTAL_RENDER_PIPELINE
            .sampler_zero
            .ends_with("end_sky.png")
    );
    assert!(
        END_PORTAL_RENDER_PIPELINE
            .sampler_one
            .ends_with("end_portal.png")
    );
}

#[test]
fn block_entity_contract_and_locked_shader_match_the_noop_subtype_and_fifteen_layers() {
    assert_eq!(
        (
            END_PORTAL_BLOCK_ENTITY_CONTRACT.has_subtype_state,
            END_PORTAL_BLOCK_ENTITY_CONTRACT.persists_custom_data,
            END_PORTAL_BLOCK_ENTITY_CONTRACT.has_update_packet,
            END_PORTAL_BLOCK_ENTITY_CONTRACT.has_ticker,
            END_PORTAL_BLOCK_ENTITY_CONTRACT.clears_reusable_face_set,
            END_PORTAL_BLOCK_ENTITY_CONTRACT.neighbor_culling,
        ),
        (false, false, false, false, true, false)
    );
    assert_eq!(
        END_PORTAL_COLORS,
        [
            [0.022_087, 0.098_399, 0.110_818],
            [0.011_892, 0.095_924, 0.089_485],
            [0.027_636, 0.101_689, 0.100_326],
            [0.046_564, 0.109_883, 0.114_838],
            [0.064_901, 0.117_696, 0.097_189],
            [0.063_761, 0.086_895, 0.123_646],
            [0.084_817, 0.111_994, 0.166_380],
            [0.097_489, 0.154_120, 0.091_064],
            [0.106_152, 0.131_144, 0.195_191],
            [0.097_721, 0.110_188, 0.187_229],
            [0.133_516, 0.138_278, 0.148_582],
            [0.070_006, 0.243_332, 0.235_792],
            [0.196_766, 0.142_899, 0.214_696],
            [0.047_281, 0.315_338, 0.321_970],
            [0.204_675, 0.390_010, 0.302_066],
            [0.080_955, 0.314_821, 0.661_491],
        ]
    );
    assert_eq!(END_PORTAL_SCALE_TRANSLATE[0], [0.5, 0.0, 0.0, 0.25]);
    assert_eq!(END_PORTAL_SCALE_TRANSLATE[1], [0.0, 0.5, 0.0, 0.25]);
    let first = end_portal_shader_layer(1, 0.25).unwrap();
    assert_eq!(first.color, END_PORTAL_COLORS[0]);
    assert_eq!(first.translation, [17.0, 1.0]);
    assert_eq!(first.scale, 8.5);
    let last = end_portal_shader_layer(15, 0.25).unwrap();
    assert_eq!(last.color, END_PORTAL_COLORS[14]);
    assert_eq!(last.scale, 1.5);
    assert_eq!(last.translation, [17.0 / 15.0, 4.5]);
    assert_ne!(
        end_portal_shader_layer(15, 0.75).unwrap().translation,
        last.translation
    );
    assert!(end_portal_shader_layer(0, 0.0).is_none());
    assert!(end_portal_shader_layer(16, 0.0).is_none());
}

#[test]
fn entering_platform_is_five_by_five_by_four_with_drop_destroy_before_replace() {
    assert_eq!(END_SPAWN, BlockPos::new(100, 50, 0));
    let writes = entering_end_platform();
    assert_eq!(writes.len(), 100);
    assert!(writes.iter().all(|write| write.destroy_mismatch_with_drops));
    assert_eq!(
        writes
            .iter()
            .filter(|write| write.desired == EndPortalDesiredBlock::Obsidian)
            .count(),
        25
    );
    assert_eq!(
        writes
            .iter()
            .filter(|write| write.desired == EndPortalDesiredBlock::Air)
            .count(),
        75
    );
    assert!(
        writes
            .iter()
            .filter(|write| write.desired == EndPortalDesiredBlock::Obsidian)
            .all(|write| write.position.y == 48)
    );
}

#[test]
fn unseen_credits_contact_bypasses_processor_and_only_first_win_sends_event() {
    let first = end_portal_credits_contact(CreditsContact {
        source_is_literal_end: true,
        is_server_player: true,
        seen_credits: false,
        won_game: false,
    });
    assert!(first.bypass_processor && first.dismount_and_remove);
    assert!(first.set_won_game && first.set_seen_credits && first.send_win_game_event);
    let already_won = end_portal_credits_contact(CreditsContact {
        won_game: true,
        ..CreditsContact {
            source_is_literal_end: true,
            is_server_player: true,
            seen_credits: false,
            won_game: false,
        }
    });
    assert!(already_won.bypass_processor && already_won.dismount_and_remove);
    assert!(!already_won.set_won_game && !already_won.send_win_game_event);
    assert!(
        !end_portal_credits_contact(CreditsContact {
            source_is_literal_end: false,
            ..CreditsContact::default()
        })
        .bypass_processor
    );
}

#[test]
fn entering_end_builds_platform_targets_player_one_lower_and_preserves_motion_pitch() {
    let velocity = Vec3 {
        x: 1.0,
        y: -2.0,
        z: 3.0,
    };
    let player = enter_end(true, true, velocity, -20.0).unwrap();
    assert_eq!(player.destination_key, "minecraft:the_end");
    assert_eq!(
        player.position,
        Vec3 {
            x: 100.5,
            y: 49.0,
            z: 0.5
        }
    );
    assert_eq!(player.velocity, velocity);
    assert_eq!(
        player.rotation,
        Rotation {
            yaw: 90.0,
            pitch: -20.0
        }
    );
    assert!(player.yaw_is_absolute && player.pitch_is_relative && player.build_platform);
    assert_eq!(player.player_level_event, Some(END_PORTAL_EVENT));
    assert_eq!(player.ticket.unwrap().radius, 3);
    let entity = enter_end(true, false, velocity, 5.0).unwrap();
    assert_eq!(entity.position.y, 50.0);
    assert_eq!(entity.player_level_event, None);
    assert!(enter_end(false, true, velocity, 0.0).is_none());
}

#[test]
fn leaving_end_uses_saved_dimension_and_separates_player_resolver_from_nonplayer_posteffect() {
    let respawn = SavedRespawn {
        position: BlockPos::new(-4, 80, 7),
        yaw: 30.0,
        pitch: 10.0,
    };
    let player = leave_end(true, "example:respawn", respawn, true, Vec3::ZERO).unwrap();
    assert_eq!(player.destination_key, "example:respawn");
    assert!(player.use_player_respawn_resolver);
    assert_eq!(player.ticket, None);
    assert_eq!(player.player_level_event, None);

    let entity = leave_end(
        true,
        "example:respawn",
        respawn,
        false,
        Vec3 {
            x: 2.0,
            y: 3.0,
            z: 4.0,
        },
    )
    .unwrap();
    assert!(!entity.use_player_respawn_resolver);
    assert_eq!(
        entity.position,
        Vec3 {
            x: -3.5,
            y: 80.0,
            z: 7.5
        }
    );
    assert_eq!(
        entity.rotation,
        Rotation {
            yaw: 30.0,
            pitch: 10.0
        }
    );
    assert_eq!(entity.ticket.unwrap().position, BlockPos::new(-4, 80, 7));
    assert!(leave_end(false, "minecraft:overworld", respawn, false, Vec3::ZERO).is_none());
}
