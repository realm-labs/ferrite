use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::{Axis, Direction};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::block::runtime::catalog::{OWNERS, owner_for_family, verify_owned_families};
use ferrite_gameplay::block::runtime::contact::{
    CauldronItem, CauldronState, HoneySlide, HoneySlideInput, InteractionResult, StickyBlock,
    blocks_stick, honey_slide, lava_cauldron_interaction, magma_hurts, moving_slime_velocity,
    slime_step_velocity,
};
use ferrite_gameplay::block::runtime::geometry::{
    DyeColor, QuarterTurn, banner_rotation, exceptional_physics, extend_beacon_color, rotate_axis,
};
use ferrite_gameplay::block::runtime::operator::{
    JIGSAW_ORIENTATIONS, JigsawJoint, JigsawRecord, RedstoneStructureAction, StructureAction,
    StructureEdit, StructureEditOutcome, StructureMirror, StructureMode, StructureRecord,
    TemplateProbe, apply_structure_edit, detect_structure_bounds, jigsaw_placement,
    structure_redstone_edge,
};
use ferrite_gameplay::block::runtime::storage::{
    BannerLayer, BannerMap, BannerMarker, BannerToggle, BannerWash, DecoratedPot, PotDecorations,
    PotInsert, SideChainPart, Stack, admitted_shelf_chain, banner_render_layers,
    banner_tooltip_layers, canonical_shelf_parts, shelf_comparator, shelf_hit_slot,
    swap_powered_shelves, wash_banner,
};
use ferrite_gameplay::player::state::Vec3;
use ferrite_registry::block_state::{PropertyName, PropertyValue};
use ferrite_registry::bundle::{
    BundleEntry, BundleRegistry, CatalogClassification, CatalogFamily, ContentBundle, FamilyName,
    Sha1Digest,
};
use ferrite_registry::digest::ContentDigest;
use ferrite_registry::minecraft_block::{BlockCatalogError, MinecraftBlockCatalog};
use ferrite_registry::provenance::{ContentProvenance, ProvenanceKind};
use ferrite_registry::registry::{PersistentId, RegistryName};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const EXPECTED_SLICES: [&str; 41] = [
    "BLK-AIR-RUNTIME-001",
    "BLK-AMETHYST-BLOCK-RUNTIME-001",
    "BLK-BANNER-RUNTIME-001",
    "BLK-BASE-DEEPSLATE-RUNTIME-001",
    "BLK-BEACON-STORAGE-RUNTIME-001",
    "BLK-BEDROCK-RUNTIME-001",
    "BLK-BONE-BLOCK-RUNTIME-001",
    "BLK-BRICKS-RUNTIME-001",
    "BLK-CONCRETE-RUNTIME-001",
    "BLK-DECORATED-POT-RUNTIME-001",
    "BLK-DEEPSLATE-MASONRY-RUNTIME-001",
    "BLK-GEODE-SHELL-IDENTITIES-001",
    "BLK-GLASS-RUNTIME-001",
    "BLK-GLAZED-TERRACOTTA-RUNTIME-001",
    "BLK-HONEY-RUNTIME-001",
    "BLK-HONEYCOMB-BLOCK-RUNTIME-001",
    "BLK-JIGSAW-RUNTIME-001",
    "BLK-LAPIS-BLOCK-RUNTIME-001",
    "BLK-LAVA-CAULDRON-RUNTIME-001",
    "BLK-MAGMA-RUNTIME-001",
    "BLK-MUD-BRICKS-RUNTIME-001",
    "BLK-PACKED-MUD-RUNTIME-001",
    "BLK-POLISHED-BASALT-RUNTIME-001",
    "BLK-PURPUR-BLOCK-RUNTIME-001",
    "BLK-QUARTZ-RUNTIME-001",
    "BLK-RAW-STORAGE-RUNTIME-001",
    "BLK-RED-NETHER-BRICKS-RUNTIME-001",
    "BLK-REDSTONE-BLOCK-RUNTIME-001",
    "BLK-REINFORCED-DEEPSLATE-RUNTIME-001",
    "BLK-SANDSTONE-RUNTIME-001",
    "BLK-SHELF-RUNTIME-001",
    "BLK-SLIME-RUNTIME-001",
    "BLK-SOUL-SAND-RUNTIME-001",
    "BLK-STAINED-GLASS-RUNTIME-001",
    "BLK-STATE-SCHEMA-001",
    "BLK-STONE-BRICK-RUNTIME-001",
    "BLK-STONE-VARIANT-RUNTIME-001",
    "BLK-STRUCTURE-RUNTIME-001",
    "BLK-STRUCTURE-VOID-RUNTIME-001",
    "BLK-TERRACOTTA-RUNTIME-001",
    "BLK-TINTED-GLASS-RUNTIME-001",
];

#[test]
fn all_blk_001_slices_have_closed_runtime_ownership() {
    let catalog = owned_catalog();
    let coverage = verify_owned_families(&catalog).unwrap();
    assert_eq!(coverage.families, 40);
    assert_eq!(
        coverage.blocks,
        OWNERS
            .iter()
            .map(|owner| owner.expected_blocks)
            .sum::<usize>()
    );
    assert_eq!(coverage.states as usize, coverage.blocks);

    let mut actual = OWNERS
        .iter()
        .map(|owner| owner.slice)
        .chain(["BLK-STATE-SCHEMA-001"])
        .collect::<Vec<_>>();
    actual.sort_unstable();
    let mut expected = EXPECTED_SLICES.to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected);
    assert!(owner_for_family("unknown-family").is_none());
}

#[test]
fn locally_imported_locked_catalog_conforms_when_available() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("../../target/ferrite-content/26.2/content-bundle.json");
    if !path.is_file() {
        eprintln!(
            "locked local artifact bundle is absent; `cargo ferrite content verify` owns that gate"
        );
        return;
    }
    let bytes = fs::read(path).unwrap();
    let bundle = serde_json::from_slice::<ContentBundle>(&bytes).unwrap();
    let registry = bundle
        .registries()
        .find(|registry| registry.name().to_string() == "minecraft:block")
        .unwrap();
    let catalog = MinecraftBlockCatalog::from_registry(registry).unwrap();
    assert_eq!(catalog.definitions().len(), 1_196);
    let state_count = catalog
        .definitions()
        .map(|definition| definition.schema().state_count())
        .sum::<u32>();
    assert_eq!(state_count, 32_366);
    for raw in 0..state_count {
        assert!(
            catalog.state_by_raw(raw).is_some(),
            "missing raw state {raw}"
        );
    }
    let coverage = verify_owned_families(&catalog).unwrap();
    assert_eq!(coverage.families, 40);
    assert_eq!(coverage.states, 1_309);
    assert_eq!(
        coverage.blocks,
        OWNERS
            .iter()
            .map(|owner| owner.expected_blocks)
            .sum::<usize>()
    );
}

#[test]
fn imported_state_schema_is_closed_and_component_patching_is_lenient() {
    let registry = registry(
        vec![family("shelf-runtime", "BLK-SHELF-001")],
        vec![entry(
            "oak_shelf",
            "shelf-runtime",
            json!({
                "properties": {
                    "facing": ["north", "south", "west", "east"],
                    "powered": ["true", "false"]
                },
                "states": [
                    state(10, [("facing", "north"), ("powered", "true")], false),
                    state(11, [("facing", "north"), ("powered", "false")], true),
                    state(12, [("facing", "south"), ("powered", "true")], false),
                    state(13, [("facing", "south"), ("powered", "false")], false),
                    state(14, [("facing", "west"), ("powered", "true")], false),
                    state(15, [("facing", "west"), ("powered", "false")], false),
                    state(16, [("facing", "east"), ("powered", "true")], false),
                    state(17, [("facing", "east"), ("powered", "false")], false)
                ]
            }),
        )],
    );
    let catalog = MinecraftBlockCatalog::from_registry(&registry).unwrap();
    let (definition, default) = catalog.state_by_raw(11).unwrap();
    assert_eq!(definition.schema().default_state(), default);

    let powered = definition
        .schema()
        .set_value(
            &default,
            &PropertyName::new("powered").unwrap(),
            &PropertyValue::new("true").unwrap(),
        )
        .unwrap();
    assert_eq!(definition.raw_state_of(&powered).unwrap(), 10);
    assert!(
        definition
            .schema()
            .set_value(
                &default,
                &PropertyName::new("missing").unwrap(),
                &PropertyValue::new("true").unwrap()
            )
            .is_err()
    );

    let patch = BTreeMap::from([
        ("facing".to_owned(), "east".to_owned()),
        ("powered".to_owned(), "invalid".to_owned()),
        ("unknown".to_owned(), "ignored".to_owned()),
    ]);
    let patched = definition
        .schema()
        .apply_component_patch(&default, &patch)
        .unwrap();
    assert_eq!(definition.raw_state_of(&patched).unwrap(), 17);
    assert_eq!(catalog.state_by_raw(17).unwrap().1, patched);
}

#[test]
fn malformed_imported_state_reports_fail_closed() {
    let malformed = registry(
        vec![family("air-runtime", "BLK-AIR-001")],
        vec![entry(
            "air",
            "air-runtime",
            json!({
                "states": [
                    {"id": 0, "default": true},
                    {"id": 0}
                ]
            }),
        )],
    );
    assert!(matches!(
        MinecraftBlockCatalog::from_registry(&malformed),
        Err(BlockCatalogError::StateCardinality { .. })
            | Err(BlockCatalogError::DuplicateRawState { .. })
    ));
}

#[test]
fn orientation_color_light_and_exceptional_physics_match_locked_rules() {
    assert_eq!(rotate_axis(Axis::X, QuarterTurn::Clockwise90), Axis::Z);
    assert_eq!(rotate_axis(Axis::Y, QuarterTurn::Clockwise90), Axis::Y);
    assert_eq!(banner_rotation(-180.0), 0);
    assert_eq!(banner_rotation(0.0), 8);
    assert_eq!(DyeColor::ALL.len(), 16);
    assert_eq!(DyeColor::White.diffuse_rgb(), 0xF9FFFE);
    assert_eq!(DyeColor::Black.diffuse_rgb(), 0x1D1D21);

    let red = extend_beacon_color(None, DyeColor::Red);
    let mixed = extend_beacon_color(Some(red), DyeColor::Blue);
    let blue = DyeColor::Blue.beacon_rgb();
    for index in 0..3 {
        assert_eq!(mixed[index], (red[index] + blue[index]) / 2.0);
    }
    let air = exceptional_physics("air").unwrap();
    assert!(air.replaceable);
    assert!(!air.full_collision);
    let bedrock = exceptional_physics("bedrock").unwrap();
    assert_eq!(bedrock.hardness, -1.0);
    assert_eq!(bedrock.resistance, 3_600_000.0);
    let tinted = exceptional_physics("tinted_glass").unwrap();
    assert_eq!(tinted.light_dampening, 15);
    let lava = exceptional_physics("lava_cauldron").unwrap();
    assert_eq!(lava.light_emission, 15);
    assert_eq!(lava.light_dampening, 0);
}

#[test]
fn slime_honey_magma_and_lava_cauldron_boundaries_are_exact() {
    let velocity = Vec3::new(1.0, 0.05, -2.0);
    let slowed = slime_step_velocity(velocity, false);
    assert!((slowed.x - 0.41).abs() < f64::EPSILON);
    assert_eq!(slowed.y, 0.05);
    assert!((slowed.z + 0.82).abs() < f64::EPSILON);
    assert_eq!(slime_step_velocity(velocity, true), velocity);
    assert_eq!(
        moving_slime_velocity(velocity, Axis::Z, -1),
        Vec3::new(1.0, 0.05, -1.0)
    );
    assert!(blocks_stick(StickyBlock::Slime, StickyBlock::Ordinary));
    assert!(!blocks_stick(StickyBlock::Slime, StickyBlock::Honey));

    let slide = honey_slide(HoneySlideInput {
        velocity: Vec3::new(0.4, -0.3, -0.2),
        on_ground: false,
        entity_y: 0.8,
        block_y: 0,
        entity_width: 0.6,
        center_offset_x: 0.8,
        center_offset_z: 0.0,
    });
    assert!(matches!(slide, HoneySlide::Accepted { .. }));
    assert!(magma_hurts(true, false));
    assert!(!magma_hurts(true, true));
    assert!(!magma_hurts(false, false));

    let blocked = lava_cauldron_interaction(CauldronItem::LavaBucket, true, true);
    assert_eq!(blocked.result, InteractionResult::Consume);
    assert!(!blocked.mutates_inventory);
    let water = lava_cauldron_interaction(CauldronItem::WaterBucket, true, true);
    assert_eq!(water.replacement, Some(CauldronState::WaterLevelThree));
    let client = lava_cauldron_interaction(CauldronItem::EmptyBucket, false, false);
    assert_eq!(client.result, InteractionResult::Success);
    assert_eq!(client.replacement, None);
}

#[test]
fn banner_limits_washing_and_map_capacity_preserve_quirks() {
    let layers = (0..17)
        .map(|index| BannerLayer {
            pattern: minecraft(&format!("pattern_{index}")),
            color: DyeColor::Red,
        })
        .collect::<Vec<_>>();
    assert_eq!(banner_tooltip_layers(&layers).len(), 6);
    assert_eq!(banner_render_layers(&layers).len(), 16);
    assert!(matches!(
        wash_banner(&[], true),
        BannerWash::TryWithEmptyHand
    ));
    assert!(matches!(
        wash_banner(&layers, true),
        BannerWash::ServerCleaned(cleaned) if cleaned.len() == 16
    ));

    let marker = BannerMarker {
        position: BlockPos::new(63, 64, -63),
        color: DyeColor::Blue,
        custom_name: Some("edge".to_owned()),
    };
    let mut map = BannerMap::new(0, 0, 0);
    map.set_tracked_decorations(256);
    assert_eq!(map.toggle(marker.clone()), BannerToggle::Added);
    map.set_tracked_decorations(257);
    assert_eq!(map.toggle(marker.clone()), BannerToggle::Removed);
    map.set_tracked_decorations(257);
    assert_eq!(map.toggle(marker.clone()), BannerToggle::Full);
    assert_eq!(
        map.toggle(BannerMarker {
            position: BlockPos::new(64, 64, 0),
            color: DyeColor::Blue,
            custom_name: None,
        }),
        BannerToggle::OutOfBounds
    );
}

#[test]
fn shelf_chain_slots_comparator_and_powered_hotbar_mapping_are_locked() {
    assert_eq!(
        canonical_shelf_parts(3).unwrap(),
        [
            SideChainPart::Left,
            SideChainPart::Center,
            SideChainPart::Right
        ]
    );
    assert_eq!(admitted_shelf_chain(2, 1), (true, false));
    assert_eq!(
        shelf_hit_slot(Direction::North, Direction::North, 1.0, 0.5),
        Some(0)
    );
    assert_eq!(
        shelf_hit_slot(Direction::North, Direction::South, 0.5, 0.5),
        None
    );

    let slots = [stack("a", 1), Stack::empty(), stack("c", 1)];
    assert_eq!(
        shelf_comparator(Direction::North, Direction::South, &slots),
        5
    );
    assert_eq!(
        shelf_comparator(Direction::North, Direction::North, &slots),
        0
    );

    let mut shelves = [Some([stack("s0", 1), Stack::empty(), stack("s2", 1)])];
    let mut hotbar = std::array::from_fn(|index| stack(&format!("h{index}"), 1));
    let swapped = swap_powered_shelves(&mut shelves, &mut hotbar);
    assert_eq!(swapped.pairs_changed, 3);
    assert_eq!(swapped.shelves_updated, 1);
    assert_eq!(hotbar[6].item.as_ref().unwrap().path(), "s0");
    assert!(hotbar[7].item.is_none());
    assert_eq!(hotbar[8].item.as_ref().unwrap().path(), "s2");
}

#[test]
fn decorated_pot_components_insertion_comparator_and_wobble_are_locked() {
    let brick = minecraft("brick");
    let angler = minecraft("angler_pottery_sherd");
    let decorations =
        PotDecorations::decode(&[brick.clone(), angler.clone(), brick.clone(), brick.clone()])
            .unwrap();
    assert_eq!(decorations.encoded()[0], brick);
    assert_eq!(decorations.encoded()[1], angler);
    assert_eq!(decorations.tooltip_order()[2].map(ResourceId::path), None);
    let too_many = std::array::from_fn::<_, 5, _>(|_| minecraft("brick"));
    assert!(PotDecorations::decode(&too_many).is_err());

    let mut pot = DecoratedPot::empty();
    let mut hand = stack_with_max("apple", 64, 64);
    assert_eq!(
        pot.insert(&mut hand, false, 10),
        PotInsert::Inserted { comparator: 1 }
    );
    assert_eq!(hand.count, 63);
    assert_ne!(pot.visible_wobble_yaw(11, 0.5), 0.0);
    let mut wrong = stack("stone", 1);
    assert_eq!(pot.insert(&mut wrong, false, 20), PotInsert::Rejected);
    assert_eq!(pot.wobble.unwrap().duration(), 10);
}

#[test]
fn jigsaw_and_structure_records_preserve_edit_order_and_edge_actions() {
    assert_eq!(JIGSAW_ORIENTATIONS.len(), 12);
    assert_eq!(
        jigsaw_placement(Direction::Up, Direction::East),
        ferrite_gameplay::block::runtime::operator::FrontAndTop {
            front: Direction::Up,
            top: Direction::West
        }
    );
    assert_eq!(JigsawRecord::fresh().joint, JigsawJoint::Rollable);
    assert_eq!(
        JigsawRecord::load_defaults(Direction::North).joint,
        JigsawJoint::Aligned
    );

    let mut record = StructureRecord::fresh(StructureMode::Load);
    let outcome = apply_structure_edit(
        &mut record,
        structure_edit("invalid uppercase", StructureAction::SaveArea),
        TemplateProbe::SaveSucceeded,
    );
    assert_eq!(outcome, StructureEditOutcome::InvalidName);
    assert_eq!(record.mode, StructureMode::Save);
    assert_eq!(record.offset, [48, -48, 5]);
    assert_eq!(record.integrity, 1.0);

    let outcome = apply_structure_edit(
        &mut record,
        structure_edit("test", StructureAction::LoadArea),
        TemplateProbe::UnequalSize([9, 8, 7]),
    );
    assert_eq!(outcome, StructureEditOutcome::Prepared);
    assert_eq!(record.size, [9, 8, 7]);

    assert_eq!(
        detect_structure_bounds(
            BlockPos::new(5, 5, 5),
            &[BlockPos::new(1, 2, 3), BlockPos::new(9, 10, 11)]
        ),
        Some(([-3, -2, -1], [7, 7, 7]))
    );
    record.mode = StructureMode::Corner;
    assert_eq!(
        structure_redstone_edge(&mut record, true),
        RedstoneStructureAction::RemoveCached
    );
    assert_eq!(
        structure_redstone_edge(&mut record, true),
        RedstoneStructureAction::None
    );
    assert_eq!(
        structure_redstone_edge(&mut record, false),
        RedstoneStructureAction::None
    );
}

fn structure_edit(name: &str, action: StructureAction) -> StructureEdit {
    StructureEdit {
        mode: StructureMode::Save,
        raw_name: name.to_owned(),
        offset: [99, -99, 5],
        size: [99, -1, 5],
        mirror: StructureMirror::FrontBack,
        rotation: QuarterTurn::Clockwise90,
        metadata: "x".repeat(200),
        ignore_entities: false,
        strict: true,
        show_air: true,
        show_bounding_box: false,
        integrity: 2.0,
        seed: 42,
        action,
    }
}

fn owned_catalog() -> MinecraftBlockCatalog {
    let families = OWNERS
        .iter()
        .map(|owner| family(owner.family, owner.slice))
        .collect();
    let mut raw = 0_u32;
    let mut entries = Vec::new();
    for owner in OWNERS {
        for ordinal in 0..owner.expected_blocks {
            entries.push(entry(
                &format!("{}_{}", owner.family.replace('-', "_"), ordinal),
                owner.family,
                json!({"states": [{"id": raw, "default": true}]}),
            ));
            raw += 1;
        }
    }
    MinecraftBlockCatalog::from_registry(&registry(families, entries)).unwrap()
}

fn registry(families: Vec<CatalogFamily>, entries: Vec<BundleEntry>) -> BundleRegistry {
    BundleRegistry::new(
        RegistryName::new(minecraft("block")),
        Sha1Digest::new("0000000000000000000000000000000000000000").unwrap(),
        families,
        entries,
    )
    .unwrap()
}

fn family(name: &str, rule: &str) -> CatalogFamily {
    CatalogFamily::new(
        FamilyName::new(name).unwrap(),
        CatalogClassification::BehaviorFamily,
        vec![rule.to_owned()],
    )
    .unwrap()
}

fn entry(path: &str, family: &str, value: Value) -> BundleEntry {
    BundleEntry::new(
        PersistentId::new(minecraft(path)),
        FamilyName::new(family).unwrap(),
        value,
        ContentProvenance::new(
            ProvenanceKind::ProjectAuthored,
            ResourceId::new("ferrite", "tests/blk_001").unwrap(),
            "v1",
            ContentDigest::blake3(b"blk-001"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn state<const N: usize>(raw: u32, properties: [(&str, &str); N], default: bool) -> Value {
    let properties = properties
        .into_iter()
        .map(|(name, value)| (name.to_owned(), Value::String(value.to_owned())))
        .collect::<Map<_, _>>();
    let mut state = Map::from_iter([
        ("id".to_owned(), Value::from(raw)),
        ("properties".to_owned(), Value::Object(properties)),
    ]);
    if default {
        state.insert("default".to_owned(), Value::Bool(true));
    }
    Value::Object(state)
}

fn minecraft(path: &str) -> ResourceId {
    ResourceId::minecraft(path).unwrap()
}

fn stack(path: &str, count: u16) -> Stack {
    stack_with_max(path, count, 64)
}

fn stack_with_max(path: &str, count: u16, maximum: u16) -> Stack {
    Stack {
        item: Some(minecraft(path)),
        count,
        maximum,
        component_fingerprint: 0,
    }
}
