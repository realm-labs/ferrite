use std::fs;
use std::path::PathBuf;

use ferrite_registry::bundle::ContentBundle;
use ferrite_world::generation::structure::block_tags::{BlockTagResolver, FileBlockTagResolver};
use ferrite_world::generation::structure::processor::{
    Axis, LimitProvider, PositionPredicate, Processor,
};
use ferrite_world::generation::structure::processor_catalog::{ProcessorAudit, ProcessorCatalog};
use ferrite_world::generation::worldgen_catalog::WorldgenCatalog;

#[test]
fn file_block_tags_resolve_nested_members_once() {
    let Some(root) = local_resource_root() else {
        return;
    };
    let mut tags = FileBlockTagResolver::new(root);
    let doors = tags.resolve_block_tag("#minecraft:doors").unwrap();
    assert!(doors.contains("minecraft:oak_door"));
    assert!(doors.contains("minecraft:iron_door"));
    assert!(tags.cached_count() >= 2);
    let cached = tags.cached_count();
    assert_eq!(tags.resolve_block_tag("minecraft:doors").unwrap(), doors);
    assert_eq!(tags.cached_count(), cached);
}

#[test]
fn locked_processor_lists_decode_to_the_exact_runtime_census() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let Some(root) = local_resource_root() else {
        return;
    };
    let worldgen = WorldgenCatalog::from_bundle(&bundle).unwrap();
    let mut tags = FileBlockTagResolver::new(root);
    let catalog = ProcessorCatalog::decode(worldgen, &mut tags).unwrap();

    assert_eq!(
        catalog.audit(),
        ProcessorAudit {
            lists: 40,
            top_level: 52,
            rules: 164,
            rule_processors: 35,
            protected: 7,
            block_rot: 6,
            capped: 4,
            input_always: 1,
            input_block: 23,
            input_state: 8,
            input_random_block: 123,
            input_tag: 9,
            location_always: 154,
            location_block: 10,
            position_always: 163,
            position_axis_linear: 1,
            modifier_passthrough: 160,
            modifier_append_loot: 4,
        }
    );

    let [Processor::Rule(rules)] = catalog.get("minecraft:high_rampart").unwrap() else {
        panic!("high_rampart processor layout");
    };
    assert!(rules.iter().any(|rule| {
        rule.position
            == PositionPredicate::AxisAlignedLinear {
                axis: Axis::Y,
                minimum_distance: 0,
                maximum_distance: 100,
                minimum_chance: 0.0,
                maximum_chance: 0.05,
            }
    }));

    let Processor::Capped { delegate, limit } = catalog
        .get("minecraft:trail_ruins_houses_archaeology")
        .unwrap()
        .iter()
        .find(|processor| {
            matches!(
                processor,
                Processor::Capped {
                    limit: LimitProvider::Constant(6),
                    ..
                }
            )
        })
        .unwrap()
    else {
        panic!("trail archaeology processor layout");
    };
    assert!(matches!(delegate.as_ref(), Processor::Rule(rules) if rules.len() == 1));
    assert_eq!(*limit, LimitProvider::Constant(6));
}

fn local_bundle() -> Option<ContentBundle> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/ferrite-content/26.2/content-bundle.json");
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn local_resource_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/mc-reference/26.2/client-classes");
    root.join("data/minecraft/tags/block")
        .is_dir()
        .then_some(root)
}
