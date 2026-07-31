use ferrite_world::generation::carver_dispatch::{
    CarverAttempt, ConfiguredCarver, apply_carver_sources,
};
use ferrite_world::generation::carver_path::{
    CarverFloatProvider, CarverHeightProvider, CaveFamily, CavePathConfig,
};
use ferrite_world::generation::feature::provider::{HeightAnchor, HeightContext};
use ferrite_world::generation::feature::random::{GenerationRandom, LegacyRandom};

#[test]
fn source_order_is_x_major_and_list_indices_restart_for_each_source() {
    let mut sources = Vec::new();
    let mut attempts = Vec::new();
    let mut volumes = Vec::new();

    apply_carver_sources(
        99,
        HeightContext {
            minimum_y: 0,
            depth: 64,
        },
        [10, -3],
        true,
        |source| {
            sources.push(source);
            vec![
                ConfiguredCarver::Cave(config()),
                ConfiguredCarver::Cave(config()),
            ]
        },
        |attempt| attempts.push(attempt),
        |origin, volume| volumes.push((origin, volume)),
    )
    .unwrap();

    assert_eq!(sources.len(), 17 * 17);
    assert_eq!(sources[0], [2, -11]);
    assert_eq!(sources[1], [2, -10]);
    assert_eq!(sources[16], [2, 5]);
    assert_eq!(sources[17], [3, -11]);
    assert_eq!(sources[288], [18, 5]);
    assert_eq!(attempts.len(), 17 * 17 * 2);
    assert_eq!(
        attempts
            .iter()
            .take(4)
            .map(|attempt| (attempt.source_chunk, attempt.list_index))
            .collect::<Vec<_>>(),
        [([2, -11], 0), ([2, -11], 1), ([2, -10], 0), ([2, -10], 1)]
    );
    assert!(
        attempts
            .iter()
            .all(|attempt| { attempt.started && attempt.debug_void_skipped })
    );
    assert!(volumes.is_empty());
}

#[test]
fn every_entry_reseeds_from_world_seed_plus_its_list_index() {
    let mut attempts = Vec::new();
    apply_carver_sources(
        -17,
        HeightContext {
            minimum_y: 0,
            depth: 64,
        },
        [0, 0],
        true,
        |_| {
            vec![
                ConfiguredCarver::Cave(config()),
                ConfiguredCarver::Cave(config()),
            ]
        },
        |attempt| attempts.push(attempt),
        |_, _| panic!("debug-void target must not dispatch geometry"),
    )
    .unwrap();

    assert_attempt_matches_legacy(attempts[0]);
    assert_attempt_matches_legacy(attempts[1]);
    assert_eq!(attempts[0].seed, -17);
    assert_eq!(attempts[1].seed, -16);
}

fn assert_attempt_matches_legacy(attempt: CarverAttempt) {
    let mut expected = LegacyRandom::new(0);
    expected.set_large_feature_seed(
        attempt.seed,
        attempt.source_chunk[0],
        attempt.source_chunk[1],
    );
    assert_eq!(attempt.start_roll, expected.next_f32());
}

fn config() -> CavePathConfig {
    CavePathConfig {
        probability: 1.0,
        y: CarverHeightProvider::Constant(HeightAnchor::Absolute(20)),
        horizontal_radius_multiplier: CarverFloatProvider::Constant(1.0),
        vertical_radius_multiplier: CarverFloatProvider::Constant(1.0),
        floor_level: CarverFloatProvider::Constant(-0.7),
        y_scale: CarverFloatProvider::Constant(1.0),
        family: CaveFamily::Ordinary,
    }
}
