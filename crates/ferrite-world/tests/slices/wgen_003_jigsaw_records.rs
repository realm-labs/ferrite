use std::fs;
use std::num::NonZeroU32;
use std::path::Path;

use ferrite_registry::bundle::ContentBundle;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::jigsaw::Padding;
use ferrite_world::generation::structure::records::{
    JigsawStructureRecord, StartHeight, StructureSetRecord, TerrainAdaptation,
};
use ferrite_world::generation::worldgen_catalog::{WorldgenCatalog, WorldgenRecordKind};

#[test]
fn locked_wgen_003_catalog_has_every_runtime_record() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let catalog = WorldgenCatalog::from_bundle(&bundle).unwrap();
    catalog.validate_wgen_003_inventory().unwrap();
    assert_eq!(
        WorldgenRecordKind::ALL_WGEN_003
            .into_iter()
            .map(WorldgenRecordKind::locked_count)
            .sum::<usize>(),
        282
    );
}

#[test]
fn ten_jigsaw_records_decode_to_the_locked_control_table() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let catalog = WorldgenCatalog::from_bundle(&bundle).unwrap();
    let names = [
        "ancient_city",
        "bastion_remnant",
        "pillager_outpost",
        "trail_ruins",
        "trial_chambers",
        "village_desert",
        "village_plains",
        "village_savanna",
        "village_snowy",
        "village_taiga",
    ];
    let records = names.map(|name| {
        JigsawStructureRecord::decode(catalog.entry(WorldgenRecordKind::Structure, name).unwrap())
            .unwrap()
    });
    let summary = records
        .iter()
        .map(|record| {
            (
                record.name.as_str(),
                record.terrain_adaptation,
                record.start_pool.as_str(),
                record.start_height,
                record.project_to_world_surface,
                record.size,
                record.maximum_distance_from_center,
                record.expansion_hack,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        summary,
        [
            (
                "ancient_city",
                TerrainAdaptation::BeardBox,
                "minecraft:ancient_city/city_center",
                StartHeight::Absolute(-27),
                false,
                7,
                116,
                false,
            ),
            (
                "bastion_remnant",
                TerrainAdaptation::None,
                "minecraft:bastion/starts",
                StartHeight::Absolute(33),
                false,
                6,
                80,
                false,
            ),
            (
                "pillager_outpost",
                TerrainAdaptation::BeardThin,
                "minecraft:pillager_outpost/base_plates",
                StartHeight::Absolute(0),
                true,
                7,
                80,
                true,
            ),
            (
                "trail_ruins",
                TerrainAdaptation::Bury,
                "minecraft:trail_ruins/tower",
                StartHeight::Absolute(-15),
                true,
                7,
                80,
                false,
            ),
            (
                "trial_chambers",
                TerrainAdaptation::Encapsulate,
                "minecraft:trial_chambers/chamber/end",
                StartHeight::UniformAbsolute {
                    minimum: -40,
                    maximum: -20,
                },
                false,
                20,
                116,
                false,
            ),
            (
                "village_desert",
                TerrainAdaptation::BeardThin,
                "minecraft:village/desert/town_centers",
                StartHeight::Absolute(0),
                true,
                6,
                80,
                true,
            ),
            (
                "village_plains",
                TerrainAdaptation::BeardThin,
                "minecraft:village/plains/town_centers",
                StartHeight::Absolute(0),
                true,
                6,
                80,
                true,
            ),
            (
                "village_savanna",
                TerrainAdaptation::BeardThin,
                "minecraft:village/savanna/town_centers",
                StartHeight::Absolute(0),
                true,
                6,
                80,
                true,
            ),
            (
                "village_snowy",
                TerrainAdaptation::BeardThin,
                "minecraft:village/snowy/town_centers",
                StartHeight::Absolute(0),
                true,
                6,
                80,
                true,
            ),
            (
                "village_taiga",
                TerrainAdaptation::BeardThin,
                "minecraft:village/taiga/town_centers",
                StartHeight::Absolute(0),
                true,
                6,
                80,
                true,
            ),
        ]
    );

    let ancient = &records[0];
    assert_eq!(
        ancient.start_jigsaw_name.as_deref(),
        Some("minecraft:city_anchor")
    );
    assert_eq!(ancient.spawn_overrides.len(), 8);
    let trial = &records[4];
    assert_eq!(trial.dimension_padding, Padding::new(10, 10));
    assert_eq!(trial.maximum_vertical_distance, Some(116));
    let trial_config = trial.start_config(-64, 319).unwrap();
    assert_eq!(trial_config.horizontal_distance, 116);
    assert_eq!(trial_config.vertical_distance, 116);
    assert!(trial.ignore_waterlogging);
    assert_eq!(trial.pool_aliases.len(), 3);
    assert_eq!(trial.spawn_overrides.len(), 8);
    let mut minimum = Draw(0);
    let mut maximum = Draw(20);
    assert_eq!(trial.sample_start_y(None, &mut minimum), -40);
    assert_eq!(trial.sample_start_y(None, &mut maximum), -20);
    assert_eq!(records[3].sample_start_y(Some(80), &mut minimum), 65);
}

#[test]
fn selecting_structure_sets_preserve_weights_placement_and_exclusion() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let catalog = WorldgenCatalog::from_bundle(&bundle).unwrap();
    let expected = [
        ("ancient_cities", 24, 8, 20_083_232, 1),
        ("nether_complexes", 27, 4, 30_084_232, 2),
        ("pillager_outposts", 32, 8, 165_745_296, 1),
        ("trail_ruins", 34, 8, 83_469_867, 1),
        ("trial_chambers", 34, 12, 94_251_327, 1),
        ("villages", 34, 8, 10_387_312, 5),
    ];
    let records = expected.map(|(name, spacing, separation, salt, count)| {
        let record = StructureSetRecord::decode(
            catalog
                .entry(WorldgenRecordKind::StructureSet, name)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            (
                record.spacing,
                record.separation,
                record.salt,
                record.structures.len(),
            ),
            (spacing, separation, salt, count)
        );
        record
    });
    assert_eq!(
        records[1]
            .structures
            .iter()
            .map(|entry| (entry.structure.as_str(), entry.weight))
            .collect::<Vec<_>>(),
        [("minecraft:fortress", 2), ("minecraft:bastion_remnant", 3)]
    );
    assert_eq!(records[2].frequency, 0.2);
    assert_eq!(records[2].frequency_reduction_method, "legacy_type_1");
    assert_eq!(
        records[2].exclusion_zone,
        Some(("minecraft:villages".into(), 10))
    );
}

fn local_bundle() -> Option<ContentBundle> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/ferrite-content/26.2/content-bundle.json");
    fs::read(path)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap())
}

struct Draw(u32);

impl GenerationRandom for Draw {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.0.min(bound.get() - 1)
    }

    fn next_f32(&mut self) -> f32 {
        0.0
    }

    fn next_f64(&mut self) -> f64 {
        0.0
    }

    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
