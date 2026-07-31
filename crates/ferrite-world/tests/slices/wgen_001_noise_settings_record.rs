use std::fs;
use std::path::Path;

use ferrite_registry::bundle::ContentBundle;
use ferrite_world::generation::noise_settings_record::NoiseGeneratorSettingsRecord;
use ferrite_world::generation::worldgen_catalog::{WorldgenCatalog, WorldgenRecordKind};

#[test]
fn locked_wgen_001_runtime_inventory_has_every_expected_record() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let catalog = WorldgenCatalog::from_bundle(&bundle).unwrap();
    catalog.validate_wgen_001_inventory().unwrap();
    assert_eq!(
        WorldgenRecordKind::ALL_WGEN_001
            .into_iter()
            .map(WorldgenRecordKind::locked_count)
            .sum::<usize>(),
        681
    );
    assert!(
        catalog
            .entry(WorldgenRecordKind::Noise, "temperature")
            .is_some()
    );
    assert!(
        catalog
            .entry(WorldgenRecordKind::DensityFunction, "overworld/depth")
            .is_some()
    );
}

#[test]
fn all_seven_noise_settings_decode_to_the_locked_behavior_table() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let catalog = WorldgenCatalog::from_bundle(&bundle).unwrap();
    let records = catalog
        .entries(WorldgenRecordKind::NoiseSettings)
        .map(NoiseGeneratorSettingsRecord::decode)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let summary = records
        .iter()
        .map(|record| {
            (
                record.name.as_str(),
                (
                    record.noise.minimum_y,
                    record.noise.height,
                    record.noise.horizontal_size,
                    record.noise.vertical_size,
                ),
                record.default_block.as_str(),
                record.default_fluid.as_str(),
                record.sea_level,
                record.aquifers_enabled,
                record.ore_veins_enabled,
                record.disable_mob_generation,
                record.legacy_random_source,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        summary,
        [
            (
                "amplified",
                (-64, 384, 1, 2),
                "minecraft:stone",
                "minecraft:water",
                63,
                true,
                true,
                false,
                false,
            ),
            (
                "caves",
                (-64, 192, 1, 2),
                "minecraft:stone",
                "minecraft:water",
                32,
                false,
                false,
                false,
                true,
            ),
            (
                "end",
                (0, 128, 2, 1),
                "minecraft:end_stone",
                "minecraft:air",
                0,
                false,
                false,
                true,
                true,
            ),
            (
                "floating_islands",
                (0, 256, 2, 1),
                "minecraft:stone",
                "minecraft:water",
                -64,
                false,
                false,
                false,
                true,
            ),
            (
                "large_biomes",
                (-64, 384, 1, 2),
                "minecraft:stone",
                "minecraft:water",
                63,
                true,
                true,
                false,
                false,
            ),
            (
                "nether",
                (0, 128, 1, 2),
                "minecraft:netherrack",
                "minecraft:lava",
                32,
                false,
                false,
                false,
                true,
            ),
            (
                "overworld",
                (-64, 384, 1, 2),
                "minecraft:stone",
                "minecraft:water",
                63,
                true,
                true,
                false,
                false,
            ),
        ]
    );
    assert_eq!(
        records
            .iter()
            .find(|record| record.name == "overworld")
            .unwrap()
            .spawn_target
            .len(),
        2
    );
}

fn local_bundle() -> Option<ContentBundle> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/ferrite-content/26.2/content-bundle.json");
    fs::read(path)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap())
}
