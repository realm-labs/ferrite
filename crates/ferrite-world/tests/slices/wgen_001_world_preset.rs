use std::fs;
use std::path::Path;

use ferrite_registry::bundle::ContentBundle;
use ferrite_world::generation::world_preset::{
    BiomeSourceDescriptor, DimensionSlot, GeneratorDescriptor, StructureSelection, WorldPreset,
};

#[test]
fn all_seven_locked_world_presets_decode_and_compose_expected_generators() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let registry = bundle
        .registries()
        .find(|registry| registry.name().to_string() == "minecraft:worldgen")
        .unwrap();
    let entries = registry
        .entries()
        .filter(|entry| {
            entry
                .persistent_id()
                .resource()
                .path()
                .starts_with("world_preset/")
        })
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 7);

    let presets = entries
        .into_iter()
        .map(WorldPreset::decode)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        presets
            .iter()
            .map(|preset| preset.name.as_str())
            .collect::<Vec<_>>(),
        [
            "amplified",
            "debug_all_block_states",
            "flat",
            "flat_all_dimensions",
            "large_biomes",
            "normal",
            "single_biome_surface",
        ]
    );

    for preset in &presets {
        assert_eq!(preset.dimensions.len(), 3);
        if preset.name != "flat_all_dimensions" {
            assert!(matches!(
                preset.dimensions[&DimensionSlot::Nether].generator,
                GeneratorDescriptor::Noise {
                    biome_source: BiomeSourceDescriptor::MultiNoise { ref preset },
                    ..
                } if preset == "minecraft:nether"
            ));
            assert!(matches!(
                preset.dimensions[&DimensionSlot::End].generator,
                GeneratorDescriptor::Noise {
                    biome_source: BiomeSourceDescriptor::TheEnd,
                    ..
                }
            ));
        }
    }
}

#[test]
fn flat_presets_preserve_absent_empty_and_listed_structure_semantics() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let registry = bundle
        .registries()
        .find(|registry| registry.name().to_string() == "minecraft:worldgen")
        .unwrap();
    let decode = |name: &str| {
        let entry = registry
            .entries()
            .find(|entry| entry.persistent_id().resource().path() == name)
            .unwrap();
        WorldPreset::decode(entry).unwrap()
    };

    let flat = decode("world_preset/flat");
    let GeneratorDescriptor::Flat(settings) = &flat.dimensions[&DimensionSlot::Overworld].generator
    else {
        panic!("flat preset did not decode a flat Overworld");
    };
    assert_eq!(
        settings.structures,
        StructureSelection::Listed(vec![
            "minecraft:strongholds".into(),
            "minecraft:villages".into()
        ])
    );
    assert_eq!(
        settings
            .layers
            .iter()
            .map(|layer| u32::from(layer.height))
            .sum::<u32>(),
        4
    );

    let all_flat = decode("world_preset/flat_all_dimensions");
    for dimension in all_flat.dimensions.values() {
        let GeneratorDescriptor::Flat(settings) = &dimension.generator else {
            panic!("all-flat preset contains a noise dimension");
        };
        assert_eq!(settings.structures, StructureSelection::All);
    }
}

fn local_bundle() -> Option<ContentBundle> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/ferrite-content/26.2/content-bundle.json");
    fs::read(path)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap())
}
