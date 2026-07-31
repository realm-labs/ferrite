use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use ferrite_foundation::resource::ResourceId;
use ferrite_registry::bundle::ContentBundle;
use ferrite_world::generation::structure::jigsaw::{ElementKind, Joint, Projection, Rotation};
use ferrite_world::generation::structure::payload_audit::audit_locked_jigsaw_payload;
use ferrite_world::generation::structure::pool_catalog::{PoolElementCounts, TemplatePoolCatalog};
use ferrite_world::generation::structure::template_manager::{
    FileTemplateSource, TemplateManager, TemplateSource, TemplateSourceError,
};
use ferrite_world::generation::worldgen_catalog::WorldgenCatalog;

#[derive(Debug, Default)]
struct MissingSource {
    calls: Cell<usize>,
}

impl TemplateSource for MissingSource {
    fn load_template(&self, _id: &ResourceId) -> Result<Option<Vec<u8>>, TemplateSourceError> {
        self.calls.set(self.calls.get() + 1);
        Ok(None)
    }
}

#[test]
fn missing_templates_become_one_cached_zero_sized_resource() {
    let mut manager = TemplateManager::new(MissingSource::default());
    let first = manager.get_or_create("ancient_city/walls/missing").unwrap();
    let second = manager
        .get_or_create("minecraft:ancient_city/walls/missing")
        .unwrap();

    assert!(first.missing && second.missing);
    assert_eq!(first.id.to_string(), "minecraft:ancient_city/walls/missing");
    assert_eq!(first.template.size, [0; 3]);
    assert!(first.template.palettes.is_empty());
    assert!(Arc::ptr_eq(&first.template, &second.template));
    assert_eq!(manager.cached_count(), 1);
    assert_eq!(manager.source().calls.get(), 1);
    assert!(manager.require("ancient_city/walls/missing").is_err());
    assert_eq!(manager.source().calls.get(), 1);
}

#[test]
fn file_source_maps_namespaces_without_allowing_path_traversal() {
    let source = FileTemplateSource::new("resource-root");
    let id = ResourceId::new("ferrite", "village/house").unwrap();
    assert_eq!(
        source.path_for(&id),
        PathBuf::from("resource-root/data/ferrite/structure/village/house.nbt")
    );
    let mut manager = TemplateManager::new(source);
    assert!(manager.get_or_create("minecraft:../escape").is_err());
    assert_eq!(manager.cached_count(), 0);
}

#[test]
fn locked_pool_catalog_loads_real_templates_and_connector_metadata() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let Some(resource_root) = local_resource_root() else {
        return;
    };
    let catalog = WorldgenCatalog::from_bundle(&bundle).unwrap();
    let mut templates = TemplateManager::new(FileTemplateSource::new(resource_root));
    let pools = TemplatePoolCatalog::decode(catalog, &mut templates).unwrap();
    let audit = pools.audit();

    assert_eq!(audit.pools, 188);
    assert_eq!(audit.weighted_entries, 1_198);
    assert_eq!(audit.expanded_weight, 4_880);
    assert_eq!(
        audit.elements,
        PoolElementCounts {
            empty: 31,
            feature: 36,
            legacy_single: 601,
            list: 3,
            single: 527,
        }
    );
    assert_eq!((audit.rigid, audit.terrain_matching), (984, 183));
    assert_eq!(audit.referenced_templates.len(), 989);
    assert_eq!(
        audit.missing_templates,
        ["minecraft:ancient_city/walls/intact_horizontal_wall_stairs_5".to_owned()]
            .into_iter()
            .collect()
    );
    assert_eq!(templates.cached_count(), 989);

    let center = &pools.pools()["minecraft:ancient_city/city_center"].expanded()[0];
    assert_eq!(center.size, [18, 31, 41]);
    assert_eq!(center.projection, Projection::Rigid);
    assert_eq!(
        center.processor_list.as_deref(),
        Some("minecraft:ancient_city_start_degradation")
    );
    assert!(!center.connectors.is_empty());
    assert!(
        center
            .connectors
            .iter()
            .all(|connector| matches!(connector.joint, Joint::Aligned | Joint::Rollable))
    );
    assert!(
        center
            .box_at(Default::default(), Rotation::Clockwise90)
            .is_some()
    );

    let sculk = pools.pools()["minecraft:ancient_city/sculk"]
        .expanded()
        .iter()
        .find(|element| matches!(element.kind, ElementKind::Feature { .. }))
        .unwrap();
    assert_eq!(sculk.size, [0; 3]);
    assert_eq!(sculk.connectors.len(), 1);
    assert!(sculk.box_at(Default::default(), Rotation::None).is_some());
}

#[test]
fn six_jigsaw_families_match_the_locked_physical_payload_boundary() {
    let Some(bundle) = local_bundle() else {
        return;
    };
    let Some(resource_root) = local_resource_root() else {
        return;
    };
    let catalog = WorldgenCatalog::from_bundle(&bundle).unwrap();
    let mut templates = TemplateManager::new(FileTemplateSource::new(&resource_root));
    let pools = TemplatePoolCatalog::decode(catalog, &mut templates).unwrap();
    let audit =
        audit_locked_jigsaw_payload(resource_root, &pools.audit().referenced_templates).unwrap();

    assert_eq!(audit.total.templates, 994);
    assert_eq!(audit.total.encoded_blocks, 869_846);
    assert_eq!(audit.total.explicit_air, 393_131);
    assert_eq!(audit.total.jigsaws, 3_754);
    assert_eq!(audit.total.other_block_nbt, 426);
    assert_eq!(audit.total.entities, 62);
    assert_eq!(audit.total.duplicate_positions, 0);
    assert_eq!(audit.total.structure_void, 0);
    assert_eq!(audit.total.structure_blocks, 1);
    assert_eq!(audit.total.connectors, 3_754);
    assert_eq!(audit.total.aligned_connectors, 1_840);
    assert_eq!(audit.total.rollable_connectors, 1_914);
    assert_eq!(audit.connector_pools.len(), 160);
    assert_eq!(audit.connector_final_states.len(), 70);
    assert_eq!(
        audit.selection_priorities,
        [(0, 3_717), (1, 34), (2, 3)].into_iter().collect()
    );
    assert_eq!(
        audit.placement_priorities,
        [(0, 3_704), (1, 44), (2, 5), (3, 1)].into_iter().collect()
    );
    assert_eq!(
        audit
            .families
            .iter()
            .map(|(family, counts)| (family.as_str(), counts.templates))
            .collect::<Vec<_>>(),
        [
            ("ancient_city", 58),
            ("bastion", 167),
            ("pillager_outpost", 11),
            ("trail_ruins", 84),
            ("trial_chambers", 191),
            ("village", 483),
        ]
    );
    assert_eq!(
        audit.missing_references,
        ["minecraft:ancient_city/walls/intact_horizontal_wall_stairs_5".to_owned()]
            .into_iter()
            .collect()
    );
    assert_eq!(
        audit.unreferenced_templates,
        [
            "minecraft:ancient_city/city_center/walls/bottom_right_corner",
            "minecraft:village/decays/grass_11x13",
            "minecraft:village/decays/grass_16x16",
            "minecraft:village/decays/grass_9x9",
            "minecraft:village/snowy/streets/crossroad_01",
            "minecraft:village/snowy/zombie/streets/crossroad_01",
        ]
        .map(str::to_owned)
        .into_iter()
        .collect()
    );
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
    root.join("data/minecraft/structure")
        .is_dir()
        .then_some(root)
}
