mod tags;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use ferrite_protocol::java_26_2::configuration::clientbound::packet::RegistryTags;
use ferrite_protocol::java_26_2::configuration::registry::SYNCHRONIZED_REGISTRY_IDENTITIES;
use ferrite_protocol::java_26_2::connection::bootstrap::{
    ConfigurationSnapshot, RegistryProjection, RegistryProjectionEntry,
};
use ferrite_protocol::java_26_2::connection::settings::{
    DisconnectMessages, ServerConnectionSettings,
};
use ferrite_protocol::java_26_2::handshake::codec as handshake_codec;
use ferrite_protocol::java_26_2::handshake::packet::{ClientIntention, ClientIntentionPacket};
use ferrite_protocol::java_26_2::login::profile::GameProfile;
use ferrite_protocol::java_26_2::play::clientbound::codec as play_clientbound;
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    EntityEvent, GameMode, PlayClientboundPacket,
};
use ferrite_protocol::java_26_2::play::clientbound::player_info::{
    AddedProfile, PlayerInfoActions, PlayerInfoEntry, PlayerInfoUpdate,
};
use ferrite_protocol::java_26_2::play::clientbound::terrain::packet::{
    ChunkCoordinate, FullChunk, LightData, LightLayerUpdate, SectionData, TerrainPacket,
};
use ferrite_protocol::java_26_2::play::registry::{BIOME, PlayRegistries};
use ferrite_protocol::java_26_2::status::clientbound::packet::{ServerStatus, StatusDescription};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::known_pack::KnownPack;
use ferrite_protocol::java_26_2::wire::compression::CompressionMode;
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::stream::{PacketStreamDecoder, PacketStreamEncoder};
use serde_json::Value;

use crate::DynError;

pub(crate) const SERVER_SESSION_ID: u128 = 0x4645_5252_4954_452d_4330_4331;
pub(crate) const CLIENT_JAR_SHA1: &str = "2dc72797acbc1b63fc16a11c4ac393605f453754";
const MAX_KNOWN_PACK_ENTRIES: usize = 8_192;
const MAX_DATA_DIRECTORY_DEPTH: usize = 16;

pub(crate) fn identifier(value: &str) -> Result<Identifier, DynError> {
    Ok(Identifier::parse(value)?)
}

pub(crate) fn core_pack() -> KnownPack {
    KnownPack::vanilla_core()
}

pub(crate) fn compact_settings() -> Result<ServerConnectionSettings, DynError> {
    let registries = SYNCHRONIZED_REGISTRY_IDENTITIES
        .iter()
        .map(|identity| {
            Ok(RegistryProjection {
                registry: identifier(identity)?,
                entries: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>, DynError>>()?;
    settings(ConfigurationSnapshot::new(
        "Ferrite".to_owned(),
        BTreeSet::from([identifier("minecraft:vanilla")?]),
        vec![core_pack()],
        registries,
        Vec::<RegistryTags>::new(),
    )?)
}

pub(crate) fn vanilla_settings(report_path: &Path) -> Result<ServerConnectionSettings, DynError> {
    let document: Value = serde_json::from_slice(&fs::read(report_path)?)?;
    let root = document
        .as_object()
        .ok_or("registry report root must be an object")?;
    let data_root = report_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or("registry report path has no version root")?
        .join("server-classes/data/minecraft");
    let mut registries = Vec::with_capacity(SYNCHRONIZED_REGISTRY_IDENTITIES.len());
    for identity in SYNCHRONIZED_REGISTRY_IDENTITIES {
        let ordered =
            report_entries(root, identity)?.unwrap_or(data_entries(&data_root, identity)?);
        registries.push(RegistryProjection {
            registry: identifier(identity)?,
            entries: ordered
                .into_iter()
                .map(|entry| {
                    Ok(RegistryProjectionEntry {
                        id: identifier(&entry)?,
                        data: None,
                        source_pack: Some(core_pack()),
                    })
                })
                .collect::<Result<Vec<_>, DynError>>()?,
        });
    }
    settings(ConfigurationSnapshot::new(
        "Ferrite".to_owned(),
        BTreeSet::from([identifier("minecraft:vanilla")?]),
        vec![core_pack()],
        registries,
        tags::load(root, &data_root)?,
    )?)
}

fn report_entries(
    root: &serde_json::Map<String, Value>,
    identity: &str,
) -> Result<Option<Vec<String>>, DynError> {
    let Some(entries) = root
        .get(identity)
        .and_then(|registry| registry.get("entries"))
        .and_then(Value::as_object)
    else {
        return Ok(None);
    };
    let mut ordered = entries
        .iter()
        .map(|(entry, value)| {
            let protocol_id = value
                .get("protocol_id")
                .and_then(Value::as_u64)
                .ok_or_else(|| format!("{identity}/{entry} has no protocol_id"))?;
            Ok((protocol_id, entry.clone()))
        })
        .collect::<Result<Vec<_>, DynError>>()?;
    ordered.sort_unstable_by_key(|(protocol_id, _)| *protocol_id);
    for (expected, (actual, _)) in ordered.iter().enumerate() {
        if *actual != expected as u64 {
            return Err(format!(
                "{identity} protocol IDs are not contiguous at {expected}: found {actual}"
            )
            .into());
        }
    }
    Ok(Some(
        ordered.into_iter().map(|(_, identity)| identity).collect(),
    ))
}

fn data_entries(data_root: &Path, identity: &str) -> Result<Vec<String>, DynError> {
    let relative = identity.strip_prefix("minecraft:").ok_or_else(|| {
        format!("synchronized registry is not in minecraft namespace: {identity}")
    })?;
    let registry_root = data_root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    let mut files = Vec::new();
    collect_json_files(&registry_root, &registry_root, 0, &mut files)?;
    if files.is_empty() {
        return Err(format!("known-pack data has no entries for {identity}").into());
    }
    files.sort();
    if identity == "minecraft:dimension_type"
        && let Some(index) = files
            .iter()
            .position(|entry| entry == "minecraft:overworld")
    {
        files.swap(0, index);
    }
    Ok(files)
}

fn collect_json_files(
    root: &Path,
    directory: &Path,
    depth: usize,
    output: &mut Vec<String>,
) -> Result<(), DynError> {
    if depth > MAX_DATA_DIRECTORY_DEPTH {
        return Err(
            format!("known-pack data exceeds directory depth {MAX_DATA_DIRECTORY_DEPTH}").into(),
        );
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_dir() {
            collect_json_files(root, &path, depth + 1, output)?;
        } else if file_type.is_file() && path.extension().is_some_and(|value| value == "json") {
            if output.len() == MAX_KNOWN_PACK_ENTRIES {
                return Err(format!(
                    "known-pack registry exceeds {MAX_KNOWN_PACK_ENTRIES} entries"
                )
                .into());
            }
            let relative = path.strip_prefix(root)?;
            let mut identity = format!(
                "minecraft:{}",
                relative.to_string_lossy().replace('\\', "/")
            );
            identity.truncate(identity.len() - ".json".len());
            output.push(identity);
        }
    }
    Ok(())
}

fn settings(snapshot: ConfigurationSnapshot) -> Result<ServerConnectionSettings, DynError> {
    let status = ServerStatus {
        description: StatusDescription::literal("Ferrite"),
        ..ServerStatus::default()
    };
    Ok(ServerConnectionSettings::with_required_defaults(
        Some(status),
        snapshot,
        DisconnectMessages::standard()?,
    ))
}

pub(crate) fn frame(body: &[u8], compression: CompressionMode) -> Result<Vec<u8>, DynError> {
    Ok(PacketStreamEncoder::new(FrameLimits::default(), compression).encode(body)?)
}

pub(crate) fn frame_body(bytes: &[u8], compression: CompressionMode) -> Result<Vec<u8>, DynError> {
    let mut decoder = PacketStreamDecoder::new(FrameLimits::default(), compression);
    decoder.push(bytes)?;
    let body = decoder
        .next_packet()?
        .ok_or("outbound frame did not contain one packet")?;
    decoder.finish()?;
    Ok(body)
}

pub(crate) fn intention(
    intention: ClientIntention,
    protocol_version: i32,
) -> Result<Vec<u8>, DynError> {
    let body = handshake_codec::encode_packet(&ClientIntentionPacket {
        protocol_version,
        host: "localhost".to_owned(),
        port: 25_565,
        intention,
    })?;
    frame(&body, CompressionMode::Disabled)
}

pub(crate) fn decode_hex(value: &str) -> Result<Vec<u8>, DynError> {
    if !value.len().is_multiple_of(2) {
        return Err("hex fixture has an odd number of digits".into());
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair)?;
            Ok(u8::from_str_radix(pair, 16)?)
        })
        .collect()
}

pub(crate) fn play_entry_frames(profile: &GameProfile) -> Result<Vec<Vec<u8>>, DynError> {
    let mut frames = PLAY_ENTRY_PREFIX_HEX
        .iter()
        .map(|value| decode_hex(value))
        .collect::<Result<Vec<_>, _>>()?;
    let permission = PlayClientboundPacket::EntityEvent(EntityEvent {
        entity_id: 1,
        event: 24,
    });
    let body = play_clientbound::encode_packet(&permission, &PlayRegistries::default())?;
    frames.push(frame(&body, CompressionMode::enabled(256)?)?);
    frames.extend(
        PLAY_ENTRY_RECIPE_HEX
            .iter()
            .map(|value| decode_hex(value))
            .collect::<Result<Vec<_>, _>>()?,
    );
    let self_info = PlayClientboundPacket::PlayerInfoUpdate(PlayerInfoUpdate {
        actions: PlayerInfoActions::all(),
        entries: vec![PlayerInfoEntry {
            profile_id: profile.id,
            added_profile: Some(AddedProfile {
                name: profile.name.clone(),
                properties: profile.properties.clone(),
            }),
            chat_session: Some(None),
            game_mode: Some(GameMode::Survival),
            listed: Some(true),
            latency_millis: Some(0),
            display_name: Some(None),
            list_order: Some(0),
            show_hat: Some(true),
        }],
    });
    let body = play_clientbound::encode_packet(&self_info, &PlayRegistries::default())?;
    frames.push(frame(&body, CompressionMode::enabled(256)?)?);
    frames.extend(
        PLAY_ENTRY_SUFFIX_HEX
            .iter()
            .map(|value| decode_hex(value))
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(frames)
}

pub(crate) fn playable_terrain_frames() -> Result<Vec<Vec<u8>>, DynError> {
    let mut registries = PlayRegistries::default();
    registries.insert(identifier(BIOME)?, vec![identifier("minecraft:plains")?]);
    let packets = [
        PlayClientboundPacket::Terrain(TerrainPacket::SetChunkCacheCenter(ChunkCoordinate {
            x: 0,
            z: 0,
        })),
        PlayClientboundPacket::Terrain(TerrainPacket::SetChunkCacheRadius(2)),
        PlayClientboundPacket::Terrain(TerrainPacket::SetSimulationDistance(10)),
        PlayClientboundPacket::Terrain(TerrainPacket::ChunkBatchStart),
        PlayClientboundPacket::Terrain(TerrainPacket::LevelChunkWithLight(playable_chunk())),
        PlayClientboundPacket::Terrain(TerrainPacket::ChunkBatchFinished(1)),
    ];
    let compression = CompressionMode::enabled(256)?;
    packets
        .iter()
        .map(|packet| {
            let body = play_clientbound::encode_packet(packet, &registries)?;
            frame(&body, compression)
        })
        .collect()
}

fn playable_chunk() -> FullChunk {
    let section = SectionData {
        non_empty_blocks: 0,
        fluid_count: 0,
        block_states: vec![0; 4_096],
        biomes: vec![0; 64],
    };
    FullChunk {
        position: ChunkCoordinate { x: 0, z: 0 },
        heightmaps: Default::default(),
        sections: vec![section; 24],
        block_entities: Vec::new(),
        light: LightData {
            sky: vec![LightLayerUpdate::Data(Box::new([0xff; 2_048])); 26],
            block: vec![LightLayerUpdate::Empty; 26],
        },
    }
}

const PLAY_ENTRY_PREFIX_HEX: [&str; 5] = [
    "480031000000010001136d696e6563726166743a6f766572776f726c6414020200010000136d696e6563726166743a6f766572776f726c64000000000000000000ff000000003f0000",
    "04000a0200",
    "0b0040003d4ccccd3dcccccd",
    "03006900",
    "050085010000",
];

const PLAY_ENTRY_RECIPE_HEX: [&str; 4] = [
    "06001001000000",
    "0a004c0000000000000000",
    "04004a0001",
    "040046ff00",
];

const PLAY_ENTRY_SUFFIX_HEX: [&str; 7] = [
    "3f004801000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "0700260d00000000",
    "29002b00000000000000000000000000000000418c9c3700000000418c9c370000000000f086a70e050f",
    "260061136d696e6563726166743a6f766572776f726c6400000000000000400000000000000000",
    "0b0071000000000000000000",
    "07007f41a0000000",
    "0400800100",
];
