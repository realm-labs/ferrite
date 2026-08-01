use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::Path;

use ferrite_foundation::identity::DimensionId;
use ferrite_protocol::java_26_2::configuration::clientbound::packet::RegistryTags;
use ferrite_protocol::java_26_2::configuration::registry::SYNCHRONIZED_REGISTRY_IDENTITIES;
use ferrite_protocol::java_26_2::connection::bootstrap::{
    ConfigurationSnapshot, RegistryProjection, RegistryProjectionEntry,
};
use ferrite_protocol::java_26_2::connection::settings::{
    DisconnectMessages, ServerConnectionSettings,
};
use ferrite_protocol::java_26_2::play::registry::{BIOME, DIMENSION_TYPE, PlayRegistries};
use ferrite_protocol::java_26_2::status::clientbound::packet::{ServerStatus, StatusDescription};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::known_pack::KnownPack;
use serde_json::Value;
use thiserror::Error;

use crate::chunk::projection::JavaTerrainRegistryMap;
use ferrite_world::id::{
    AIR, END_PORTAL, END_STONE, FIRE, GRASS_BLOCK, LAVA, NETHER_PORTAL_X, NETHER_PORTAL_Z,
    NETHERRACK, OBSIDIAN, STONE, WATER,
};

use crate::minecraft::tags;
use crate::world_service::dimension::{NETHER_WASTES, OVERWORLD_BIOMES, THE_END};

type DynError = Box<dyn Error + Send + Sync>;

const MAX_KNOWN_PACK_ENTRIES: usize = 8_192;
const MAX_DATA_DIRECTORY_DEPTH: usize = 16;

#[derive(Debug, Clone)]
pub(super) struct ProtocolBootstrap {
    pub(super) settings: ServerConnectionSettings,
    pub(super) registries: PlayRegistries,
    pub(super) terrain_registries: JavaTerrainRegistryMap,
}

pub(super) fn load(
    report_path: Option<&Path>,
    enabled_dimensions: &[DimensionId],
) -> Result<ProtocolBootstrap, SettingsError> {
    load_inner(report_path, enabled_dimensions).map_err(|source| SettingsError { source })
}

fn load_inner(
    report_path: Option<&Path>,
    enabled_dimensions: &[DimensionId],
) -> Result<ProtocolBootstrap, DynError> {
    let (configuration, registries) = if let Some(report_path) = report_path {
        vanilla_configuration(report_path)?
    } else {
        compact_configuration()?
    };
    let mut settings = ServerConnectionSettings::with_required_defaults(
        Some(ServerStatus {
            description: StatusDescription::literal("Ferrite"),
            ..ServerStatus::default()
        }),
        configuration,
        DisconnectMessages::standard()?,
    );
    settings.play_registries = registries.clone();
    for dimension in enabled_dimensions {
        registries.raw_id(DIMENSION_TYPE, &identifier(&dimension.to_string())?)?;
    }

    let plains = registries.raw_id(BIOME, &identifier("minecraft:plains")?)?;
    let snowy_plains = registries.raw_id(BIOME, &identifier("minecraft:snowy_plains")?)?;
    let forest = registries.raw_id(BIOME, &identifier("minecraft:forest")?)?;
    let nether_wastes = registries.raw_id(BIOME, &identifier("minecraft:nether_wastes")?)?;
    let the_end = registries.raw_id(BIOME, &identifier("minecraft:the_end")?)?;
    let raw_block_states = match report_path {
        Some(report_path) => [
            reported_block_state_id(report_path, "minecraft:air")?,
            reported_block_state_id(report_path, "minecraft:stone")?,
            reported_block_state_id(report_path, "minecraft:grass_block")?,
            reported_block_state_id(report_path, "minecraft:water")?,
            reported_block_state_id(report_path, "minecraft:lava")?,
            reported_block_state_id(report_path, "minecraft:fire")?,
            reported_block_state_id(report_path, "minecraft:netherrack")?,
            reported_block_state_id(report_path, "minecraft:end_stone")?,
            reported_block_state_id(report_path, "minecraft:obsidian")?,
            reported_block_state_with_property(
                report_path,
                "minecraft:nether_portal",
                "axis",
                "x",
            )?,
            reported_block_state_with_property(
                report_path,
                "minecraft:nether_portal",
                "axis",
                "z",
            )?,
            reported_block_state_id(report_path, "minecraft:end_portal")?,
        ],
        None => [
            0, 1, 8, 86, 102, 3_406, 6_997, 9_477, 3_369, 7_017, 7_018, 9_468,
        ],
    };
    let mut terrain_registries = JavaTerrainRegistryMap::new(12, AIR)?;
    for (state, raw_id) in [
        AIR,
        STONE,
        GRASS_BLOCK,
        WATER,
        LAVA,
        FIRE,
        NETHERRACK,
        END_STONE,
        OBSIDIAN,
        NETHER_PORTAL_X,
        NETHER_PORTAL_Z,
        END_PORTAL,
    ]
    .into_iter()
    .zip(raw_block_states)
    {
        terrain_registries.insert_block_state(state, raw_id)?;
    }
    terrain_registries.insert_biome(OVERWORLD_BIOMES[0], plains)?;
    terrain_registries.insert_biome(OVERWORLD_BIOMES[1], snowy_plains)?;
    terrain_registries.insert_biome(OVERWORLD_BIOMES[2], forest)?;
    terrain_registries.insert_biome(NETHER_WASTES, nether_wastes)?;
    terrain_registries.insert_biome(THE_END, the_end)?;

    Ok(ProtocolBootstrap {
        settings,
        registries,
        terrain_registries,
    })
}

fn reported_block_state_id(report_path: &Path, block: &str) -> Result<i32, DynError> {
    let blocks_path = report_path.with_file_name("blocks.json");
    let document: Value = serde_json::from_slice(&fs::read(&blocks_path)?)?;
    let raw_id = document
        .get(block)
        .and_then(|block| block.get("states"))
        .and_then(Value::as_array)
        .and_then(|states| {
            states
                .iter()
                .find(|state| state.get("default").and_then(Value::as_bool) == Some(true))
        })
        .and_then(|state| state.get("id"))
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("block report has no default state ID for {block}"))?;
    i32::try_from(raw_id)
        .map_err(|_| format!("block state ID for {block} exceeds i32: {raw_id}").into())
}

fn reported_block_state_with_property(
    report_path: &Path,
    block: &str,
    property: &str,
    value: &str,
) -> Result<i32, DynError> {
    let blocks_path = report_path.with_file_name("blocks.json");
    let document: Value = serde_json::from_slice(&fs::read(&blocks_path)?)?;
    let raw_id = document
        .get(block)
        .and_then(|block| block.get("states"))
        .and_then(Value::as_array)
        .and_then(|states| {
            states.iter().find(|state| {
                state
                    .get("properties")
                    .and_then(|properties| properties.get(property))
                    .and_then(Value::as_str)
                    == Some(value)
            })
        })
        .and_then(|state| state.get("id"))
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            format!("block report has no state ID for {block} with {property}={value}")
        })?;
    i32::try_from(raw_id)
        .map_err(|_| format!("block state ID for {block} exceeds i32: {raw_id}").into())
}

fn compact_configuration() -> Result<(ConfigurationSnapshot, PlayRegistries), DynError> {
    let projections = SYNCHRONIZED_REGISTRY_IDENTITIES
        .iter()
        .map(|identity| {
            Ok(RegistryProjection {
                registry: identifier(identity)?,
                entries: Vec::new(),
            })
        })
        .collect::<Result<Vec<_>, DynError>>()?;
    let configuration = ConfigurationSnapshot::new(
        "Ferrite".to_owned(),
        BTreeSet::from([identifier("minecraft:vanilla")?]),
        vec![KnownPack::vanilla_core()],
        projections,
        Vec::<RegistryTags>::new(),
    )?;
    let mut registries = PlayRegistries::default();
    registries.insert(
        identifier(DIMENSION_TYPE)?,
        vec![
            identifier("minecraft:overworld")?,
            identifier("minecraft:the_nether")?,
            identifier("minecraft:the_end")?,
        ],
    );
    registries.insert(
        identifier(BIOME)?,
        vec![
            identifier("minecraft:plains")?,
            identifier("minecraft:snowy_plains")?,
            identifier("minecraft:forest")?,
            identifier("minecraft:nether_wastes")?,
            identifier("minecraft:the_end")?,
        ],
    );
    Ok((configuration, registries))
}

fn vanilla_configuration(
    report_path: &Path,
) -> Result<(ConfigurationSnapshot, PlayRegistries), DynError> {
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
    let mut projections = Vec::with_capacity(SYNCHRONIZED_REGISTRY_IDENTITIES.len());
    let mut registries = PlayRegistries::default();
    for identity in SYNCHRONIZED_REGISTRY_IDENTITIES {
        let ordered =
            report_entries(root, identity)?.unwrap_or(data_entries(&data_root, identity)?);
        let identifiers = ordered
            .iter()
            .map(|entry| identifier(entry))
            .collect::<Result<Vec<_>, _>>()?;
        registries.insert(identifier(identity)?, identifiers.clone());
        projections.push(RegistryProjection {
            registry: identifier(identity)?,
            entries: identifiers
                .into_iter()
                .map(|id| RegistryProjectionEntry {
                    id,
                    data: None,
                    source_pack: Some(KnownPack::vanilla_core()),
                })
                .collect(),
        });
    }
    let configuration = ConfigurationSnapshot::new(
        "Ferrite".to_owned(),
        BTreeSet::from([identifier("minecraft:vanilla")?]),
        vec![KnownPack::vanilla_core()],
        projections,
        tags::load(root, &data_root)?,
    )?;
    Ok((configuration, registries))
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

pub(super) fn data_entries(data_root: &Path, identity: &str) -> Result<Vec<String>, DynError> {
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
    if identity == DIMENSION_TYPE
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

fn identifier(value: &str) -> Result<Identifier, DynError> {
    Ok(Identifier::parse(value)?)
}

#[derive(Debug, Error)]
#[error("Minecraft 26.2 protocol bootstrap failed: {source}")]
pub(super) struct SettingsError {
    #[source]
    source: DynError,
}
