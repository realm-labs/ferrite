use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::worldgen_oracle::model::NORMALIZATION_SCHEMA;

const REQUIRED_FIELDS: [&str; 16] = [
    "chunk_status",
    "block_states",
    "fluid_states",
    "biomes",
    "block_entities",
    "post_processing",
    "heightmaps",
    "structure_starts",
    "structure_references",
    "scheduled_block_ticks",
    "scheduled_fluid_ticks",
    "sky_light",
    "block_light",
    "light_initialized",
    "inhabited_time",
    "generation_metadata",
];

#[derive(Debug, Clone, Deserialize)]
pub struct ExactnessContract {
    schema_version: u32,
    reference_version: String,
    official_server_sha1: String,
    normalization_schema: String,
    acceptance: String,
    oracle_batch: String,
    population_batch: String,
    semantic_fields: Vec<String>,
    excluded_representation_fields: Vec<String>,
    dimension: Vec<DimensionContract>,
    population: Vec<PopulationContract>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DimensionContract {
    id: String,
    directory: String,
    minimum_y: i32,
    height: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct PopulationContract {
    id: String,
    data_pack: String,
    seeds: Vec<i64>,
    dimensions: Vec<String>,
    chunks: Vec<String>,
    request_plans: Vec<String>,
}

impl ExactnessContract {
    pub fn locked() -> Result<Self, ExactnessContractError> {
        Self::read(&locked_path())
    }

    pub fn read(path: &Path) -> Result<Self, ExactnessContractError> {
        let text = fs::read_to_string(path).map_err(|source| ExactnessContractError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let contract: Self = toml::from_str(&text).map_err(ExactnessContractError::Parse)?;
        contract.validate()?;
        Ok(contract)
    }

    pub fn dimension(&self, id: &str) -> Result<&DimensionContract, ExactnessContractError> {
        self.dimension
            .iter()
            .find(|dimension| dimension.id == id)
            .ok_or_else(|| ExactnessContractError::UnknownDimension(id.to_owned()))
    }

    #[must_use]
    pub fn semantic_fields(&self) -> &[String] {
        &self.semantic_fields
    }

    fn validate(&self) -> Result<(), ExactnessContractError> {
        require(self.schema_version == 1, "schema_version must be 1")?;
        require(
            self.reference_version == "26.2",
            "reference_version must be 26.2",
        )?;
        require(
            self.official_server_sha1 == "823e2250d24b3ddac457a60c92a6a941943fcd6a",
            "official_server_sha1 does not match OFF-SERVER-001",
        )?;
        require(
            self.normalization_schema == NORMALIZATION_SCHEMA,
            "normalization_schema is unsupported",
        )?;
        require(
            self.acceptance == "ZeroUnexplainedSemanticDivergence",
            "acceptance must require zero unexplained divergence",
        )?;
        require(
            self.oracle_batch == "G01-P8-B4",
            "oracle_batch must be G01-P8-B4",
        )?;
        require(
            self.population_batch == "G01-P8-B5",
            "population_batch must be G01-P8-B5",
        )?;
        let actual = self
            .semantic_fields
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        require(
            actual == BTreeSet::from(REQUIRED_FIELDS),
            "semantic_fields do not match the frozen denominator",
        )?;
        require(
            self.excluded_representation_fields.len()
                == self
                    .excluded_representation_fields
                    .iter()
                    .collect::<BTreeSet<_>>()
                    .len(),
            "excluded representation fields contain duplicates",
        )?;

        let ids = self
            .dimension
            .iter()
            .map(|dimension| dimension.id.as_str())
            .collect::<BTreeSet<_>>();
        require(
            ids == BTreeSet::from([
                "minecraft:overworld",
                "minecraft:the_nether",
                "minecraft:the_end",
            ]),
            "dimension denominator must contain Overworld, Nether, and End",
        )?;
        for dimension in &self.dimension {
            require(dimension.height > 0, "dimension height must be positive")?;
            require(
                dimension.minimum_y % 16 == 0 && dimension.height.is_multiple_of(16),
                "dimension vertical range must be section aligned",
            )?;
            if dimension.id == "minecraft:overworld" {
                require(
                    dimension.directory == "dimensions/minecraft/overworld",
                    "Overworld directory must match the locked 26.2 layout",
                )?;
            }
        }

        let mut populations = BTreeSet::new();
        for population in &self.population {
            require(
                populations.insert(population.id.as_str()),
                "population IDs must be unique",
            )?;
            require(
                !population.data_pack.is_empty(),
                "population data pack is empty",
            )?;
            require(!population.seeds.is_empty(), "population seeds are empty")?;
            require(!population.chunks.is_empty(), "population chunks are empty")?;
            require(
                !population.request_plans.is_empty(),
                "population request plans are empty",
            )?;
            require(
                population
                    .dimensions
                    .iter()
                    .all(|dimension| ids.contains(dimension.as_str())),
                "population names an unknown dimension",
            )?;
            for coordinate in &population.chunks {
                parse_chunk_coordinate(coordinate)?;
            }
        }
        require(
            populations == BTreeSet::from(["supported-data-pack", "vanilla-core"]),
            "required worldgen populations are missing",
        )
    }
}

impl DimensionContract {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn directory(&self) -> &str {
        &self.directory
    }

    #[must_use]
    pub const fn minimum_y(&self) -> i32 {
        self.minimum_y
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }
}

fn locked_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../goals/minecraft-java-26.2/worldgen-exactness.toml")
}

fn require(condition: bool, message: &'static str) -> Result<(), ExactnessContractError> {
    if condition {
        Ok(())
    } else {
        Err(ExactnessContractError::Invalid(message))
    }
}

fn parse_chunk_coordinate(value: &str) -> Result<(i32, i32), ExactnessContractError> {
    let (x, z) = value
        .split_once(',')
        .ok_or(ExactnessContractError::Invalid(
            "chunk coordinate has no comma",
        ))?;
    Ok((
        x.parse()
            .map_err(|_| ExactnessContractError::Invalid("chunk X is not i32"))?,
        z.parse()
            .map_err(|_| ExactnessContractError::Invalid("chunk Z is not i32"))?,
    ))
}

#[derive(Debug, Error)]
pub enum ExactnessContractError {
    #[error("cannot read exactness contract {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot parse exactness contract: {0}")]
    Parse(toml::de::Error),
    #[error("invalid exactness contract: {0}")]
    Invalid(&'static str),
    #[error("unknown exactness dimension {0}")]
    UnknownDimension(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_contract_freezes_the_exact_denominator() {
        let contract = ExactnessContract::locked().unwrap();
        assert_eq!(contract.semantic_fields().len(), REQUIRED_FIELDS.len());
        assert_eq!(
            contract
                .dimension("minecraft:overworld")
                .unwrap()
                .minimum_y(),
            -64
        );
        assert_eq!(
            contract.dimension("minecraft:the_end").unwrap().height(),
            256
        );
    }
}
