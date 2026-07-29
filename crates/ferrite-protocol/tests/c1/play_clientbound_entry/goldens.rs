use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    BorderInitialization, ChangeDifficulty, DefaultSpawnPosition, EntityEvent, GameEvent,
    GlobalBlockPosition, PlayClientboundPacket, PlayerAbilities, PlayerPosition, SetTime,
    TickingState, Vector3,
};
use ferrite_protocol::java_26_2::play::clientbound::player_info::{
    PlayerInfoActions, PlayerInfoUpdate,
};
use ferrite_protocol::java_26_2::play::clientbound::recipe::{RecipeBookAdd, RecipeBookSettings};

use super::fixtures::{
    context, empty_commands, empty_recipes, golden_frame, hex, id, login, registries,
};

#[test]
fn matches_every_locked_empty_or_default_play_entry_golden() {
    let registries = registries();
    let cases = [
        (
            login(),
            "480031000000010001136d696e6563726166743a6f766572776f726c6414020200010000136d696e6563726166743a6f766572776f726c64000000000000000000ff000000003f0000",
        ),
        (
            PlayClientboundPacket::ChangeDifficulty(ChangeDifficulty {
                raw_difficulty: 2,
                locked: false,
            }),
            "04000a0200",
        ),
        (
            PlayClientboundPacket::PlayerAbilities(PlayerAbilities {
                flags: 0,
                flying_speed: 0.05,
                walking_speed: 0.1,
            }),
            "0b0040003d4ccccd3dcccccd",
        ),
        (PlayClientboundPacket::SetHeldSlot(0), "03006900"),
        (
            PlayClientboundPacket::Commands(empty_commands()),
            "06001001000000",
        ),
        (
            PlayClientboundPacket::RecipeBookSettings(RecipeBookSettings::default()),
            "0a004c0000000000000000",
        ),
        (
            PlayClientboundPacket::RecipeBookAdd(RecipeBookAdd {
                entries: Vec::new(),
                replace: true,
            }),
            "04004a0001",
        ),
        (
            PlayClientboundPacket::UpdateRecipes(empty_recipes()),
            "050085010000",
        ),
        (
            PlayClientboundPacket::PlayerInfoUpdate(PlayerInfoUpdate {
                actions: PlayerInfoActions::all(),
                entries: Vec::new(),
            }),
            "040046ff00",
        ),
        (
            PlayClientboundPacket::PlayerPosition(PlayerPosition {
                teleport_id: 1,
                position: Vector3::default(),
                motion: Vector3::default(),
                yaw: 0.0,
                pitch: 0.0,
                relative_flags: 0,
            }),
            "3f004801000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        ),
        (
            PlayClientboundPacket::GameEvent(GameEvent {
                event: 13,
                parameter: 0.0,
            }),
            "0700260d00000000",
        ),
        (
            PlayClientboundPacket::InitializeBorder(BorderInitialization {
                center_x: 0.0,
                center_z: 0.0,
                old_size: 59_999_968.0,
                new_size: 59_999_968.0,
                lerp_millis: 0,
                absolute_maximum: 29_999_984,
                warning_blocks: 5,
                warning_time: 15,
            }),
            "29002b00000000000000000000000000000000418c9c3700000000418c9c370000000000f086a70e050f",
        ),
        (
            PlayClientboundPacket::SetDefaultSpawnPosition(DefaultSpawnPosition {
                position: GlobalBlockPosition {
                    dimension: id("minecraft:overworld"),
                    packed_position: 64,
                },
                yaw: 0.0,
                pitch: 0.0,
            }),
            "260061136d696e6563726166743a6f766572776f726c6400000000000000400000000000000000",
        ),
        (
            PlayClientboundPacket::SetTime(SetTime {
                game_time: 0,
                clocks: BTreeMap::new(),
            }),
            "0b0071000000000000000000",
        ),
        (
            PlayClientboundPacket::TickingState(TickingState {
                tick_rate: 20.0,
                frozen: false,
            }),
            "07007f41a0000000",
        ),
        (PlayClientboundPacket::TickingStep(0), "0400800100"),
    ];

    for (packet, expected) in cases {
        let body = encode_packet(&packet, &registries).unwrap();
        assert_eq!(golden_frame(&body), hex(expected));
        assert_eq!(decode_packet(&body, context(&registries)).unwrap(), packet);
    }
}

#[test]
fn entity_event_fixed_width_fields_round_trip() {
    let registries = registries();
    let packet = PlayClientboundPacket::EntityEvent(EntityEvent {
        entity_id: -7,
        event: -128,
    });
    let body = encode_packet(&packet, &registries).unwrap();
    assert_eq!(decode_packet(&body, context(&registries)).unwrap(), packet);
}
