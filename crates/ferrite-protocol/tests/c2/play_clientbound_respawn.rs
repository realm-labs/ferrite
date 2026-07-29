use ferrite_protocol::java_26_2::play::clientbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    CommonSpawnInfo, GameMode, PlayClientboundPacket,
};
use ferrite_protocol::java_26_2::play::clientbound::session::Respawn;
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::{BIOME, DIMENSION_TYPE, PlayRegistries};
use ferrite_protocol::java_26_2::value::identifier::Identifier;

static REJECT_COMPONENTS: RejectComponentValues = RejectComponentValues;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(id(DIMENSION_TYPE), vec![id("minecraft:overworld")]);
    registries.insert(id(BIOME), vec![id("minecraft:plains")]);
    registries
}

#[test]
fn respawn_codec_preserves_independent_keep_bits_and_ignores_high_bits() {
    let registries = registries();
    let context = PlayDecodeContext {
        registries: &registries,
        component_values: &REJECT_COMPONENTS,
        dimension_section_count: 2,
    };
    let spawn = CommonSpawnInfo {
        dimension_type: id("minecraft:overworld"),
        dimension: id("minecraft:overworld"),
        obfuscated_seed: -7,
        game_mode: GameMode::Survival,
        previous_game_mode: Some(GameMode::Creative),
        is_debug: false,
        is_flat: true,
        last_death: None,
        portal_cooldown: 20,
        sea_level: 63,
    };
    for (mask, attributes, entity_data) in [
        (0, false, false),
        (1, true, false),
        (2, false, true),
        (3, true, true),
        (-1, true, true),
    ] {
        let respawn = Respawn {
            spawn: spawn.clone(),
            data_to_keep: mask,
        };
        assert_eq!(respawn.retention().attributes, attributes);
        assert_eq!(respawn.retention().entity_data, entity_data);
        let packet = PlayClientboundPacket::Respawn(respawn);
        let encoded = encode_packet(&packet, &registries).unwrap();
        assert_eq!(encoded[0], 82);
        assert_eq!(decode_packet(&encoded, context).unwrap(), packet);
    }
}
