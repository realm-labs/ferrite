//! Runtime view over the version-locked worldgen partition in a content bundle.

use ferrite_registry::bundle::{BundleEntry, BundleRegistry, ContentBundle};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldgenRecordKind {
    Biome,
    ConfiguredCarver,
    ConfiguredFeature,
    DensityFunction,
    FlatPreset,
    MultiNoisePreset,
    Noise,
    NoiseSettings,
    PlacedFeature,
    WorldPreset,
}

impl WorldgenRecordKind {
    pub const ALL_WGEN_001: [Self; 10] = [
        Self::Biome,
        Self::ConfiguredCarver,
        Self::ConfiguredFeature,
        Self::DensityFunction,
        Self::FlatPreset,
        Self::MultiNoisePreset,
        Self::Noise,
        Self::NoiseSettings,
        Self::PlacedFeature,
        Self::WorldPreset,
    ];

    pub const fn path(self) -> &'static str {
        match self {
            Self::Biome => "biome",
            Self::ConfiguredCarver => "configured_carver",
            Self::ConfiguredFeature => "configured_feature",
            Self::DensityFunction => "density_function",
            Self::FlatPreset => "flat_level_generator_preset",
            Self::MultiNoisePreset => "multi_noise_biome_source_parameter_list",
            Self::Noise => "noise",
            Self::NoiseSettings => "noise_settings",
            Self::PlacedFeature => "placed_feature",
            Self::WorldPreset => "world_preset",
        }
    }

    pub const fn locked_count(self) -> usize {
        match self {
            Self::Biome => 66,
            Self::ConfiguredCarver => 4,
            Self::ConfiguredFeature => 226,
            Self::DensityFunction => 35,
            Self::FlatPreset => 9,
            Self::MultiNoisePreset => 2,
            Self::Noise => 63,
            Self::NoiseSettings => 7,
            Self::PlacedFeature => 262,
            Self::WorldPreset => 7,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorldgenCatalog<'a> {
    registry: &'a BundleRegistry,
}

impl<'a> WorldgenCatalog<'a> {
    pub fn from_bundle(bundle: &'a ContentBundle) -> Result<Self, WorldgenCatalogError> {
        let registry = bundle
            .registries()
            .find(|registry| registry.name().to_string() == "minecraft:worldgen")
            .ok_or(WorldgenCatalogError::MissingRegistry)?;
        Ok(Self { registry })
    }

    pub fn entries(&self, kind: WorldgenRecordKind) -> impl Iterator<Item = &'a BundleEntry> + 'a {
        let prefix = format!("{}/", kind.path());
        self.registry
            .entries()
            .filter(move |entry| entry.persistent_id().resource().path().starts_with(&prefix))
    }

    pub fn entry(&self, kind: WorldgenRecordKind, name: &str) -> Option<&'a BundleEntry> {
        let path = format!("{}/{}", kind.path(), name);
        self.registry
            .entries()
            .find(|entry| entry.persistent_id().resource().path() == path)
    }

    pub fn validate_wgen_001_inventory(&self) -> Result<(), WorldgenCatalogError> {
        for kind in WorldgenRecordKind::ALL_WGEN_001 {
            let actual = self.entries(kind).count();
            let expected = kind.locked_count();
            if actual != expected {
                return Err(WorldgenCatalogError::Count {
                    kind,
                    expected,
                    actual,
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorldgenCatalogError {
    #[error("content bundle has no minecraft:worldgen registry")]
    MissingRegistry,
    #[error("{kind:?} has {actual} records, expected {expected}")]
    Count {
        kind: WorldgenRecordKind,
        expected: usize,
        actual: usize,
    },
}
