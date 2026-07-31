use ferrite_world::generation::overworld_biomes::{
    is_deep_dark_region, nether_points, overworld_points, overworld_spawn_targets,
};
use serde_json::Value;

#[test]
fn normal_overworld_builder_emits_locked_partition_counts_and_order() {
    let points = overworld_points();
    assert_eq!(points.len(), 7_594);
    assert_eq!(
        points[..22]
            .iter()
            .filter(|point| point.biome == "mushroom_fields")
            .count(),
        2
    );
    assert_eq!(points[0].parameters[4], [0.0, 0.0]);
    assert_eq!(points[1].parameters[4], [1.0, 1.0]);
    assert_eq!(points[22].parameters[5], [-1.0, -0.93333334]);
    assert_eq!(points[7_590].biome, "dripstone_caves");
    assert_eq!(points[7_591].biome, "lush_caves");
    assert_eq!(points[7_592].biome, "sulfur_caves");
    assert_eq!(points[7_593].biome, "deep_dark");
}

#[test]
fn every_surface_entry_is_an_adjacent_depth_zero_then_one_pair() {
    let points = overworld_points();
    for pair in points[..7_590].chunks_exact(2) {
        assert_eq!(pair[0].biome, pair[1].biome);
        assert_eq!(pair[0].parameters[4], [0.0, 0.0]);
        assert_eq!(pair[1].parameters[4], [1.0, 1.0]);
        for coordinate in [0, 1, 2, 3, 5] {
            assert_eq!(
                pair[0].parameters[coordinate],
                pair[1].parameters[coordinate]
            );
        }
    }
}

#[test]
fn boundary_order_keeps_closed_weirdness_slices_in_source_sequence() {
    let points = overworld_points();
    let boundary = -0.93333334_f32;
    let first = points
        .iter()
        .position(|point| point.parameters[5][1] == boundary)
        .unwrap();
    let next = points
        .iter()
        .skip(first + 1)
        .position(|point| point.parameters[5][0] == boundary)
        .map(|offset| first + 1 + offset)
        .unwrap();
    assert!(first < next);
    assert_eq!(points[first].parameters[5], [-1.0, boundary]);
    assert_eq!(points[next].parameters[5], [boundary, -0.7666667]);
}

#[test]
fn preset_auxiliary_points_and_deep_dark_thresholds_are_exact() {
    let nether = nether_points();
    assert_eq!(nether.len(), 5);
    assert_eq!(nether[3].biome, "warped_forest");
    assert_eq!(nether[3].offset, 0.375);
    assert_eq!(nether[4].offset, 0.175);

    let spawn = overworld_spawn_targets();
    assert_eq!(spawn[0][2], [-0.11, 1.0]);
    assert_eq!(spawn[0][5], [-1.0, -0.16]);
    assert_eq!(spawn[1][5], [0.16, 1.0]);

    assert!(!is_deep_dark_region(f64::from(-0.225_f32), 1.0));
    assert!(!is_deep_dark_region(-1.0, f64::from(0.9_f32)));
    assert!(is_deep_dark_region(
        f64::from(-0.225_f32) - f64::EPSILON,
        f64::from(0.9_f32) + f64::EPSILON
    ));
}

#[test]
fn local_locked_report_matches_all_7594_emitted_points_when_available() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../target/mc-reference/26.2/generated/reports/biome_parameters/minecraft/overworld.json",
    );
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };
    let report: Value = serde_json::from_slice(&bytes).unwrap();
    let expected = report["biomes"].as_array().unwrap();
    let actual = overworld_points();
    assert_eq!(expected.len(), actual.len());
    for (index, (expected, actual)) in expected.iter().zip(&actual).enumerate() {
        assert_eq!(
            expected["biome"].as_str().unwrap(),
            format!("minecraft:{}", actual.biome),
            "biome mismatch at point {index}"
        );
        let parameters = &expected["parameters"];
        for (coordinate, key) in [
            (0, "temperature"),
            (1, "humidity"),
            (2, "continentalness"),
            (3, "erosion"),
            (4, "depth"),
            (5, "weirdness"),
        ] {
            assert_eq!(
                quantized_range(json_range(&parameters[key])),
                quantized_range(actual.parameters[coordinate]),
                "{key} mismatch at point {index}"
            );
        }
        assert_eq!(
            parameters["offset"].as_f64().unwrap() as f32,
            actual.offset,
            "offset mismatch at point {index}"
        );
    }
}

fn json_range(value: &Value) -> [f32; 2] {
    if let Some(value) = value.as_f64() {
        return [value as f32; 2];
    }
    let values = value.as_array().unwrap();
    [
        values[0].as_f64().unwrap() as f32,
        values[1].as_f64().unwrap() as f32,
    ]
}

fn quantized_range(range: [f32; 2]) -> [i64; 2] {
    range.map(|value| (value * 10_000.0) as i64)
}
