use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use fastnbt::Value;
use flate2::read::{GzDecoder, ZlibDecoder};
use thiserror::Error;

use crate::worldgen_oracle::contract::{DimensionContract, ExactnessContract};
use crate::worldgen_oracle::model::{
    BIOMES_PER_SECTION, BLOCKS_PER_SECTION, CanonicalNbt, ChunkCoordinate, NORMALIZATION_SCHEMA,
    SemanticBlockEntity, SemanticChunk, SemanticSection, SemanticSource,
};

pub fn normalize_official_chunk(
    world_root: &Path,
    dimension: &str,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<SemanticChunk, AnvilChunkError> {
    let contract = ExactnessContract::locked()?;
    let dimension_contract = contract.dimension(dimension)?;
    let root = if dimension_contract.directory() == "." {
        world_root.to_path_buf()
    } else {
        world_root.join(dimension_contract.directory())
    };
    let region_x = chunk_x.div_euclid(32);
    let region_z = chunk_z.div_euclid(32);
    let path = root
        .join("region")
        .join(format!("r.{region_x}.{region_z}.mca"));
    let nbt = read_region_chunk(&path, chunk_x, chunk_z)?;
    normalize_nbt(nbt, dimension_contract, chunk_x, chunk_z)
}

fn read_region_chunk(
    path: &Path,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<HashMap<String, Value>, AnvilChunkError> {
    let mut file = File::open(path).map_err(|source| AnvilChunkError::Open {
        path: path.to_path_buf(),
        source,
    })?;
    let local_x = chunk_x.rem_euclid(32) as u64;
    let local_z = chunk_z.rem_euclid(32) as u64;
    let location_offset = (local_x + local_z * 32) * 4;
    file.seek(SeekFrom::Start(location_offset))?;
    let mut location = [0_u8; 4];
    file.read_exact(&mut location)?;
    let sector =
        (u32::from(location[0]) << 16) | (u32::from(location[1]) << 8) | u32::from(location[2]);
    if sector == 0 || location[3] == 0 {
        return Err(AnvilChunkError::MissingChunk { chunk_x, chunk_z });
    }
    file.seek(SeekFrom::Start(u64::from(sector) * 4_096))?;
    let mut length = [0_u8; 4];
    file.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length);
    if length < 2 || length > u32::from(location[3]) * 4_096 - 4 {
        return Err(AnvilChunkError::InvalidLength(length));
    }
    let mut compression = [0_u8; 1];
    file.read_exact(&mut compression)?;
    if compression[0] & 0x80 != 0 {
        return Err(AnvilChunkError::ExternalChunkStream);
    }
    let mut compressed = vec![0_u8; length as usize - 1];
    file.read_exact(&mut compressed)?;
    let mut bytes = Vec::new();
    match compression[0] {
        1 => {
            GzDecoder::new(compressed.as_slice()).read_to_end(&mut bytes)?;
        }
        2 => {
            ZlibDecoder::new(compressed.as_slice()).read_to_end(&mut bytes)?;
        }
        3 => bytes = compressed,
        codec => return Err(AnvilChunkError::UnsupportedCompression(codec)),
    }
    fastnbt::from_bytes(&bytes).map_err(AnvilChunkError::Nbt)
}

fn normalize_nbt(
    mut root: HashMap<String, Value>,
    dimension: &DimensionContract,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<SemanticChunk, AnvilChunkError> {
    let actual_x = integer(&root, "xPos")?;
    let actual_z = integer(&root, "zPos")?;
    if actual_x != chunk_x || actual_z != chunk_z {
        return Err(AnvilChunkError::CoordinateMismatch {
            expected_x: chunk_x,
            expected_z: chunk_z,
            actual_x,
            actual_z,
        });
    }
    let data_version = integer(&root, "DataVersion")?;
    let status = string(&root, "Status")?.to_owned();
    let light_initialized = optional_integer(&root, "isLightOn").unwrap_or_default() != 0;
    let inhabited_time = optional_long(&root, "InhabitedTime").unwrap_or_default();
    let sections = normalize_sections(&root, dimension)?;
    let heightmaps = normalize_heightmaps(&root, dimension)?;
    let mut block_entities = normalize_block_entities(&root)?;
    block_entities.sort_by_key(|block_entity| block_entity.position);

    let post_processing = take_canonical(&mut root, "PostProcessing", CanonicalNbt::empty_list());
    let structures = root.remove("structures");
    let (structure_starts, structure_references) = split_structures(structures.as_ref());
    let scheduled_block_ticks =
        take_canonical(&mut root, "block_ticks", CanonicalNbt::empty_list());
    let scheduled_fluid_ticks =
        take_canonical(&mut root, "fluid_ticks", CanonicalNbt::empty_list());
    let generation_metadata = [
        "blending_data",
        "below_zero_retrogen",
        "UpgradeData",
        "CarvingMasks",
    ]
    .into_iter()
    .filter_map(|name| {
        root.remove(name)
            .map(|value| (name.to_owned(), CanonicalNbt::from_fast(&value)))
    })
    .collect();

    let chunk = SemanticChunk {
        schema: NORMALIZATION_SCHEMA.to_owned(),
        source: SemanticSource::OfficialMinecraft26_2,
        reference_version: "26.2".to_owned(),
        data_version,
        dimension: dimension.id().to_owned(),
        position: ChunkCoordinate {
            x: chunk_x,
            z: chunk_z,
        },
        status,
        sections,
        heightmaps,
        block_entities,
        post_processing,
        structure_starts,
        structure_references,
        scheduled_block_ticks,
        scheduled_fluid_ticks,
        light_initialized,
        inhabited_time,
        generation_metadata,
    };
    chunk.validate_shape().map_err(AnvilChunkError::Shape)?;
    Ok(chunk)
}

fn normalize_sections(
    root: &HashMap<String, Value>,
    dimension: &DimensionContract,
) -> Result<Vec<SemanticSection>, AnvilChunkError> {
    let values = list(field(root, "sections")?, "sections")?;
    let mut raw_by_y = BTreeMap::new();
    for value in values {
        let section = compound(value, "section")?;
        let y = integer(section, "Y")?;
        if raw_by_y.insert(y, section).is_some() {
            return Err(AnvilChunkError::DuplicateSection(y));
        }
    }
    let minimum_section = dimension.minimum_y().div_euclid(16);
    let section_count = dimension.height() / 16;
    let mut sections = Vec::with_capacity(section_count as usize);
    for offset in 0..section_count {
        let y = minimum_section + offset as i32;
        let section = raw_by_y.get(&y).ok_or(AnvilChunkError::MissingSection(y))?;
        let block_states = decode_block_states(field(section, "block_states")?)?;
        let fluid_states = block_states
            .iter()
            .map(|state| infer_fluid_state(state))
            .collect();
        let biomes = decode_string_palette(field(section, "biomes")?, BIOMES_PER_SECTION, 1)?;
        let sky_light = optional_byte_array(section, "SkyLight");
        let block_light = optional_byte_array(section, "BlockLight");
        sections.push(SemanticSection {
            y,
            block_states,
            fluid_states,
            biomes,
            sky_light,
            block_light,
        });
    }
    Ok(sections)
}

fn decode_block_states(value: &Value) -> Result<Vec<String>, AnvilChunkError> {
    let container = compound(value, "block_states")?;
    let palette = list(field(container, "palette")?, "block palette")?
        .iter()
        .map(canonical_block_state)
        .collect::<Result<Vec<_>, _>>()?;
    decode_palette(container.get("data"), &palette, BLOCKS_PER_SECTION, 4)
}

fn canonical_block_state(value: &Value) -> Result<String, AnvilChunkError> {
    let state = compound(value, "block state")?;
    let name = string(state, "Name")?;
    let Some(properties) = state.get("Properties") else {
        return Ok(name.to_owned());
    };
    let properties = compound(properties, "block properties")?;
    let mut values = properties
        .iter()
        .map(|(name, value)| Ok(format!("{name}={}", value_string(value)?)))
        .collect::<Result<Vec<_>, AnvilChunkError>>()?;
    values.sort();
    Ok(format!("{name}[{}]", values.join(",")))
}

fn decode_string_palette(
    value: &Value,
    count: usize,
    minimum_bits: u32,
) -> Result<Vec<String>, AnvilChunkError> {
    let container = compound(value, "paletted container")?;
    let palette = list(field(container, "palette")?, "palette")?
        .iter()
        .map(|value| value_string(value).map(str::to_owned))
        .collect::<Result<Vec<_>, _>>()?;
    decode_palette(container.get("data"), &palette, count, minimum_bits)
}

fn decode_palette(
    data: Option<&Value>,
    palette: &[String],
    count: usize,
    minimum_bits: u32,
) -> Result<Vec<String>, AnvilChunkError> {
    if palette.is_empty() {
        return Err(AnvilChunkError::EmptyPalette);
    }
    if palette.len() == 1 {
        return Ok(vec![palette[0].clone(); count]);
    }
    let bits = minimum_bits.max(usize::BITS - (palette.len() - 1).leading_zeros());
    let values_per_long = 64 / bits;
    let longs = long_array(data.ok_or(AnvilChunkError::MissingPaletteData)?)?;
    let expected = count.div_ceil(values_per_long as usize);
    if longs.len() != expected {
        return Err(AnvilChunkError::PaletteDataLength {
            expected,
            actual: longs.len(),
        });
    }
    let mask = (1_u64 << bits) - 1;
    (0..count)
        .map(|index| {
            let long_index = index / values_per_long as usize;
            let bit_index = (index % values_per_long as usize) as u32 * bits;
            let palette_index = ((longs[long_index] as u64 >> bit_index) & mask) as usize;
            palette
                .get(palette_index)
                .cloned()
                .ok_or(AnvilChunkError::PaletteIndex(palette_index))
        })
        .collect()
}

fn normalize_heightmaps(
    root: &HashMap<String, Value>,
    dimension: &DimensionContract,
) -> Result<BTreeMap<String, Vec<i32>>, AnvilChunkError> {
    let maps = compound(field(root, "Heightmaps")?, "Heightmaps")?;
    let bits = u32::BITS - dimension.height().leading_zeros();
    maps.iter()
        .map(|(name, value)| {
            let longs = long_array(value)?;
            let values_per_long = 64 / bits;
            let expected = 256_usize.div_ceil(values_per_long as usize);
            if longs.len() != expected {
                return Err(AnvilChunkError::HeightmapDataLength {
                    kind: name.clone(),
                    expected,
                    actual: longs.len(),
                });
            }
            let mask = (1_u64 << bits) - 1;
            let heights = (0..256)
                .map(|index| {
                    let long_index = index / values_per_long as usize;
                    let bit_index = (index % values_per_long as usize) as u32 * bits;
                    dimension.minimum_y()
                        + (((longs[long_index] as u64 >> bit_index) & mask) as i32)
                })
                .collect();
            Ok((name.clone(), heights))
        })
        .collect()
}

fn normalize_block_entities(
    root: &HashMap<String, Value>,
) -> Result<Vec<SemanticBlockEntity>, AnvilChunkError> {
    list(field(root, "block_entities")?, "block_entities")?
        .iter()
        .map(|value| {
            let mut data = compound(value, "block entity")?.clone();
            let position = [
                integer(&data, "x")?,
                integer(&data, "y")?,
                integer(&data, "z")?,
            ];
            let kind = string(&data, "id")?.to_owned();
            for field in ["x", "y", "z", "id", "keepPacked"] {
                data.remove(field);
            }
            Ok(SemanticBlockEntity {
                position,
                kind,
                data: CanonicalNbt::from_fast(&Value::Compound(data)),
            })
        })
        .collect()
}

fn split_structures(value: Option<&Value>) -> (CanonicalNbt, CanonicalNbt) {
    let Some(Value::Compound(structures)) = value else {
        return (
            CanonicalNbt::empty_compound(),
            CanonicalNbt::empty_compound(),
        );
    };
    let starts = structures
        .get("starts")
        .map(CanonicalNbt::from_fast)
        .unwrap_or_else(CanonicalNbt::empty_compound);
    let references = structures
        .get("References")
        .map(CanonicalNbt::from_fast)
        .unwrap_or_else(CanonicalNbt::empty_compound);
    (starts, references)
}

fn infer_fluid_state(state: &str) -> String {
    let name = state.split_once('[').map_or(state, |(name, _)| name);
    if name == "minecraft:water" || state.contains("waterlogged=true") {
        let level = property(state, "level").unwrap_or("0");
        return format!("minecraft:water[level={level}]");
    }
    if name == "minecraft:lava" {
        let level = property(state, "level").unwrap_or("0");
        return format!("minecraft:lava[level={level}]");
    }
    "minecraft:empty".to_owned()
}

fn property<'a>(state: &'a str, key: &str) -> Option<&'a str> {
    let (_, properties) = state.split_once('[')?;
    properties
        .trim_end_matches(']')
        .split(',')
        .find_map(|property| property.split_once('=').filter(|(name, _)| *name == key))
        .map(|(_, value)| value)
}

fn field<'a>(
    root: &'a HashMap<String, Value>,
    name: &'static str,
) -> Result<&'a Value, AnvilChunkError> {
    root.get(name).ok_or(AnvilChunkError::MissingField(name))
}

fn compound<'a>(
    value: &'a Value,
    context: &'static str,
) -> Result<&'a HashMap<String, Value>, AnvilChunkError> {
    match value {
        Value::Compound(value) => Ok(value),
        _ => Err(AnvilChunkError::WrongType(context)),
    }
}

fn list<'a>(value: &'a Value, context: &'static str) -> Result<&'a [Value], AnvilChunkError> {
    match value {
        Value::List(value) => Ok(value),
        _ => Err(AnvilChunkError::WrongType(context)),
    }
}

fn string<'a>(
    root: &'a HashMap<String, Value>,
    name: &'static str,
) -> Result<&'a str, AnvilChunkError> {
    value_string(field(root, name)?)
}

fn value_string(value: &Value) -> Result<&str, AnvilChunkError> {
    match value {
        Value::String(value) => Ok(value),
        _ => Err(AnvilChunkError::WrongType("string")),
    }
}

fn integer(root: &HashMap<String, Value>, name: &'static str) -> Result<i32, AnvilChunkError> {
    i32::try_from(
        field(root, name)?
            .as_i64()
            .ok_or(AnvilChunkError::WrongType(name))?,
    )
    .map_err(|_| AnvilChunkError::IntegerRange(name))
}

fn optional_integer(root: &HashMap<String, Value>, name: &str) -> Option<i32> {
    root.get(name)
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

fn optional_long(root: &HashMap<String, Value>, name: &str) -> Option<i64> {
    root.get(name).and_then(Value::as_i64)
}

fn long_array(value: &Value) -> Result<Vec<i64>, AnvilChunkError> {
    match value {
        Value::LongArray(values) => Ok(values.iter().copied().collect()),
        _ => Err(AnvilChunkError::WrongType("long array")),
    }
}

fn optional_byte_array(root: &HashMap<String, Value>, name: &str) -> Option<Vec<u8>> {
    match root.get(name) {
        Some(Value::ByteArray(values)) => Some(values.iter().map(|value| *value as u8).collect()),
        _ => None,
    }
}

fn take_canonical(
    root: &mut HashMap<String, Value>,
    name: &str,
    default: CanonicalNbt,
) -> CanonicalNbt {
    root.remove(name)
        .as_ref()
        .map(CanonicalNbt::from_fast)
        .unwrap_or(default)
}

#[derive(Debug, Error)]
pub enum AnvilChunkError {
    #[error("cannot open Anvil region {path}: {source}")]
    Open {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Contract(#[from] crate::worldgen_oracle::contract::ExactnessContractError),
    #[error("chunk {chunk_x},{chunk_z} is absent from its Anvil region")]
    MissingChunk { chunk_x: i32, chunk_z: i32 },
    #[error("invalid Anvil chunk length {0}")]
    InvalidLength(u32),
    #[error("external Anvil chunk streams are unsupported")]
    ExternalChunkStream,
    #[error("unsupported Anvil compression codec {0}")]
    UnsupportedCompression(u8),
    #[error("cannot decode chunk NBT: {0}")]
    Nbt(fastnbt::error::Error),
    #[error("chunk coordinates are {actual_x},{actual_z}, expected {expected_x},{expected_z}")]
    CoordinateMismatch {
        expected_x: i32,
        expected_z: i32,
        actual_x: i32,
        actual_z: i32,
    },
    #[error("missing chunk field {0}")]
    MissingField(&'static str),
    #[error("chunk field {0} has the wrong NBT type")]
    WrongType(&'static str),
    #[error("chunk field {0} is outside i32 range")]
    IntegerRange(&'static str),
    #[error("chunk has duplicate section Y {0}")]
    DuplicateSection(i32),
    #[error("chunk is missing semantic section Y {0}")]
    MissingSection(i32),
    #[error("paletted container has an empty palette")]
    EmptyPalette,
    #[error("paletted container has no data for a multi-entry palette")]
    MissingPaletteData,
    #[error("palette data has {actual} longs, expected {expected}")]
    PaletteDataLength { expected: usize, actual: usize },
    #[error("palette index {0} is out of range")]
    PaletteIndex(usize),
    #[error("heightmap {kind} has {actual} longs, expected {expected}")]
    HeightmapDataLength {
        kind: String,
        expected: usize,
        actual: usize,
    },
    #[error("normalized chunk shape is invalid: {0}")]
    Shape(String),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;

    use fastnbt::LongArray;
    use flate2::Compression;
    use flate2::write::ZlibEncoder;

    use super::*;

    #[test]
    fn normalizes_a_project_authored_anvil_fixture() {
        let directory = tempfile::tempdir().unwrap();
        let region = directory
            .path()
            .join("dimensions/minecraft/overworld/region/r.0.0.mca");
        fs::create_dir_all(region.parent().unwrap()).unwrap();
        write_region_fixture(&region);

        let chunk =
            normalize_official_chunk(directory.path(), "minecraft:overworld", 0, 0).unwrap();
        assert_eq!(chunk.sections.len(), 24);
        assert_eq!(chunk.sections[0].y, -4);
        assert_eq!(chunk.sections[0].block_states[0], "minecraft:air");
        assert_eq!(chunk.sections[0].fluid_states[0], "minecraft:empty");
        assert_eq!(chunk.heightmaps["WORLD_SURFACE"][0], -64);
        assert_eq!(chunk.canonical_digest().len(), 64);
    }

    fn write_region_fixture(path: &Path) {
        let sections = (-4..20)
            .map(|y| {
                Value::Compound(HashMap::from([
                    ("Y".to_owned(), Value::Byte(y)),
                    (
                        "block_states".to_owned(),
                        Value::Compound(HashMap::from([(
                            "palette".to_owned(),
                            Value::List(vec![Value::Compound(HashMap::from([(
                                "Name".to_owned(),
                                Value::String("minecraft:air".to_owned()),
                            )]))]),
                        )])),
                    ),
                    (
                        "biomes".to_owned(),
                        Value::Compound(HashMap::from([(
                            "palette".to_owned(),
                            Value::List(vec![Value::String("minecraft:plains".to_owned())]),
                        )])),
                    ),
                ]))
            })
            .collect();
        let empty_structures = Value::Compound(HashMap::from([
            ("starts".to_owned(), Value::Compound(HashMap::new())),
            ("References".to_owned(), Value::Compound(HashMap::new())),
        ]));
        let root = HashMap::from([
            ("DataVersion".to_owned(), Value::Int(1)),
            ("xPos".to_owned(), Value::Int(0)),
            ("zPos".to_owned(), Value::Int(0)),
            (
                "Status".to_owned(),
                Value::String("minecraft:full".to_owned()),
            ),
            ("sections".to_owned(), Value::List(sections)),
            (
                "Heightmaps".to_owned(),
                Value::Compound(HashMap::from([(
                    "WORLD_SURFACE".to_owned(),
                    Value::LongArray(LongArray::new(vec![0; 37])),
                )])),
            ),
            ("block_entities".to_owned(), Value::List(Vec::new())),
            ("PostProcessing".to_owned(), Value::List(Vec::new())),
            ("structures".to_owned(), empty_structures),
            ("block_ticks".to_owned(), Value::List(Vec::new())),
            ("fluid_ticks".to_owned(), Value::List(Vec::new())),
            ("isLightOn".to_owned(), Value::Byte(1)),
            ("InhabitedTime".to_owned(), Value::Long(0)),
        ]);
        let encoded = fastnbt::to_bytes(&root).unwrap();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&encoded).unwrap();
        let compressed = encoder.finish().unwrap();
        let length = compressed.len() + 1;
        let sectors = (length + 4).div_ceil(4_096);
        let mut bytes = vec![0_u8; (2 + sectors) * 4_096];
        bytes[0..4].copy_from_slice(&[0, 0, 2, sectors as u8]);
        let start = 2 * 4_096;
        bytes[start..start + 4].copy_from_slice(&(length as u32).to_be_bytes());
        bytes[start + 4] = 2;
        bytes[start + 5..start + 5 + compressed.len()].copy_from_slice(&compressed);
        fs::write(path, bytes).unwrap();
    }
}
