use std::collections::BTreeMap;

use ferrite_testkit::worldgen_oracle::compare::compare_chunks;
use ferrite_testkit::worldgen_oracle::contract::ExactnessContract;
use ferrite_testkit::worldgen_oracle::model::{
    CanonicalNbt, ChunkCoordinate, SemanticChunk, SemanticSection, SemanticSource,
};

#[test]
fn locked_oracle_accepts_identity_and_reports_first_semantic_divergence() {
    let contract = ExactnessContract::locked().unwrap();
    assert_eq!(contract.semantic_fields().len(), 16);

    let official = fixture(SemanticSource::OfficialMinecraft26_2);
    let mut ferrite = fixture(SemanticSource::Ferrite);
    assert_eq!(compare_chunks(&official, &ferrite), Ok(()));

    ferrite.sections[0].biomes[6] = "minecraft:forest".to_owned();
    let divergence = compare_chunks(&official, &ferrite).unwrap_err();
    assert_eq!(divergence.stage, "biomes");
    assert_eq!(divergence.field, "biomes");
    assert_eq!(divergence.coordinate, Some([8, -64, 4]));
}

fn fixture(source: SemanticSource) -> SemanticChunk {
    SemanticChunk {
        schema: "ferrite:worldgen-semantic-chunk/1".to_owned(),
        source,
        reference_version: "26.2".to_owned(),
        data_version: 1,
        dimension: "minecraft:overworld".to_owned(),
        position: ChunkCoordinate { x: 0, z: 0 },
        status: "minecraft:full".to_owned(),
        sections: vec![SemanticSection {
            y: -4,
            block_states: vec!["minecraft:air".to_owned(); 4_096],
            fluid_states: vec!["minecraft:empty".to_owned(); 4_096],
            biomes: vec!["minecraft:plains".to_owned(); 64],
            sky_light: Some(vec![0xff; 2_048]),
            block_light: None,
        }],
        heightmaps: BTreeMap::from([("WORLD_SURFACE".to_owned(), vec![-64; 256])]),
        block_entities: Vec::new(),
        post_processing: CanonicalNbt::empty_list(),
        structure_starts: CanonicalNbt::empty_compound(),
        structure_references: CanonicalNbt::empty_compound(),
        scheduled_block_ticks: CanonicalNbt::empty_list(),
        scheduled_fluid_ticks: CanonicalNbt::empty_list(),
        light_initialized: true,
        inhabited_time: 0,
        generation_metadata: BTreeMap::new(),
    }
}
