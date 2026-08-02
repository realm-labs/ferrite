use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const NORMALIZATION_SCHEMA: &str = "ferrite:worldgen-semantic-chunk/1";
pub const BLOCKS_PER_SECTION: usize = 4_096;
pub const BIOMES_PER_SECTION: usize = 64;
pub const LIGHT_BYTES_PER_SECTION: usize = 2_048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticSource {
    OfficialMinecraft26_2,
    Ferrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChunkCoordinate {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticChunk {
    pub schema: String,
    pub source: SemanticSource,
    pub reference_version: String,
    pub data_version: i32,
    pub dimension: String,
    pub position: ChunkCoordinate,
    pub status: String,
    pub sections: Vec<SemanticSection>,
    pub heightmaps: BTreeMap<String, Vec<i32>>,
    pub block_entities: Vec<SemanticBlockEntity>,
    pub post_processing: CanonicalNbt,
    pub structure_starts: CanonicalNbt,
    pub structure_references: CanonicalNbt,
    pub scheduled_block_ticks: CanonicalNbt,
    pub scheduled_fluid_ticks: CanonicalNbt,
    pub light_initialized: bool,
    pub inhabited_time: i64,
    pub generation_metadata: BTreeMap<String, CanonicalNbt>,
}

impl SemanticChunk {
    #[must_use]
    pub fn canonical_digest(&self) -> String {
        #[derive(Serialize)]
        struct SemanticDigest<'a> {
            schema: &'a str,
            reference_version: &'a str,
            data_version: i32,
            dimension: &'a str,
            position: ChunkCoordinate,
            status: &'a str,
            sections: &'a [SemanticSection],
            heightmaps: &'a BTreeMap<String, Vec<i32>>,
            block_entities: &'a [SemanticBlockEntity],
            post_processing: &'a CanonicalNbt,
            structure_starts: &'a CanonicalNbt,
            structure_references: &'a CanonicalNbt,
            scheduled_block_ticks: &'a CanonicalNbt,
            scheduled_fluid_ticks: &'a CanonicalNbt,
            light_initialized: bool,
            inhabited_time: i64,
            generation_metadata: &'a BTreeMap<String, CanonicalNbt>,
        }
        let semantic = SemanticDigest {
            schema: &self.schema,
            reference_version: &self.reference_version,
            data_version: self.data_version,
            dimension: &self.dimension,
            position: self.position,
            status: &self.status,
            sections: &self.sections,
            heightmaps: &self.heightmaps,
            block_entities: &self.block_entities,
            post_processing: &self.post_processing,
            structure_starts: &self.structure_starts,
            structure_references: &self.structure_references,
            scheduled_block_ticks: &self.scheduled_block_ticks,
            scheduled_fluid_ticks: &self.scheduled_fluid_ticks,
            light_initialized: self.light_initialized,
            inhabited_time: self.inhabited_time,
            generation_metadata: &self.generation_metadata,
        };
        let bytes = serde_json::to_vec(&semantic).expect("semantic chunk is serializable");
        blake3::hash(&bytes).to_hex().to_string()
    }

    pub fn validate_shape(&self) -> Result<(), String> {
        if self.schema != NORMALIZATION_SCHEMA {
            return Err(format!("unsupported normalization schema {}", self.schema));
        }
        if self.reference_version != "26.2" {
            return Err(format!(
                "unsupported reference version {}",
                self.reference_version
            ));
        }
        let mut previous = None;
        for section in &self.sections {
            if previous.is_some_and(|value| value >= section.y) {
                return Err("sections are not strictly ordered by Y".to_owned());
            }
            previous = Some(section.y);
            section.validate_shape()?;
        }
        for (kind, values) in &self.heightmaps {
            if values.len() != 256 {
                return Err(format!("heightmap {kind} has {} entries", values.len()));
            }
        }
        if !self
            .block_entities
            .windows(2)
            .all(|pair| pair[0].position < pair[1].position)
        {
            return Err("block entities are not strictly coordinate ordered".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSection {
    pub y: i32,
    pub block_states: Vec<String>,
    pub fluid_states: Vec<String>,
    pub biomes: Vec<String>,
    pub sky_light: Option<Vec<u8>>,
    pub block_light: Option<Vec<u8>>,
}

impl SemanticSection {
    fn validate_shape(&self) -> Result<(), String> {
        if self.block_states.len() != BLOCKS_PER_SECTION {
            return Err(format!(
                "section {} has {} block states",
                self.y,
                self.block_states.len()
            ));
        }
        if self.fluid_states.len() != BLOCKS_PER_SECTION {
            return Err(format!(
                "section {} has {} fluid states",
                self.y,
                self.fluid_states.len()
            ));
        }
        if self.biomes.len() != BIOMES_PER_SECTION {
            return Err(format!(
                "section {} has {} biomes",
                self.y,
                self.biomes.len()
            ));
        }
        for (name, layer) in [
            ("sky", self.sky_light.as_ref()),
            ("block", self.block_light.as_ref()),
        ] {
            if layer.is_some_and(|bytes| bytes.len() != LIGHT_BYTES_PER_SECTION) {
                return Err(format!("section {} has invalid {name} light", self.y));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticBlockEntity {
    pub position: [i32; 3],
    pub kind: String,
    pub data: CanonicalNbt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "value", rename_all = "snake_case")]
pub enum CanonicalNbt {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    FloatBits(u32),
    DoubleBits(u64),
    String(String),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    List(Vec<Self>),
    Compound(BTreeMap<String, Self>),
}

impl CanonicalNbt {
    #[must_use]
    pub fn empty_list() -> Self {
        Self::List(Vec::new())
    }

    #[must_use]
    pub fn empty_compound() -> Self {
        Self::Compound(BTreeMap::new())
    }

    pub(crate) fn from_fast(value: &fastnbt::Value) -> Self {
        match value {
            fastnbt::Value::Byte(value) => Self::Byte(*value),
            fastnbt::Value::Short(value) => Self::Short(*value),
            fastnbt::Value::Int(value) => Self::Int(*value),
            fastnbt::Value::Long(value) => Self::Long(*value),
            fastnbt::Value::Float(value) => Self::FloatBits(value.to_bits()),
            fastnbt::Value::Double(value) => Self::DoubleBits(value.to_bits()),
            fastnbt::Value::String(value) => Self::String(value.clone()),
            fastnbt::Value::ByteArray(value) => Self::ByteArray(value.iter().copied().collect()),
            fastnbt::Value::IntArray(value) => Self::IntArray(value.iter().copied().collect()),
            fastnbt::Value::LongArray(value) => Self::LongArray(value.iter().copied().collect()),
            fastnbt::Value::List(values) => {
                Self::List(values.iter().map(Self::from_fast).collect())
            }
            fastnbt::Value::Compound(values) => Self::Compound(
                values
                    .iter()
                    .map(|(name, value)| (name.clone(), Self::from_fast(value)))
                    .collect(),
            ),
        }
    }
}
