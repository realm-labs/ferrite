use super::*;
use crate::artifact::{manifest_metadata_is_current, verify_file};
use crate::catalog::{
    classify, ids_from_files, registry_entry, registry_ids, server_data_prefix,
    validate_family_selectors,
};
use crate::surface::{
    command_report_paths, expected_surface_kinds, validate_command_root_map,
    validate_cross_system_join_map, validate_exact_protocol_family_partition,
};
use crate::symbols::{
    descriptor_matches, parse_javap_batch, read_symbol_cache, symbol_cache_directory,
    symbol_cache_file, write_symbol_cache,
};
use crate::verification::completion_slice_has_ownership;
use tempfile::tempdir;

#[test]
fn ids_are_namespaced() {
    assert_eq!(normalize_id("stone").unwrap(), "minecraft:stone");
    assert_eq!(
        normalize_id("minecraft:oak_log").unwrap(),
        "minecraft:oak_log"
    );
    assert!(normalize_id("Bad ID").is_err());
}

#[test]
fn digest_is_sorted_and_newline_terminated() {
    let ids = BTreeSet::from(["minecraft:b".into(), "minecraft:a".into()]);
    assert_eq!(ids_digest(&ids), sha1_bytes(b"minecraft:a\nminecraft:b\n"));
}

#[test]
fn parses_manifest() {
    let manifest: Manifest =
        serde_json::from_str(r#"{"versions":[{"id":"26.2","url":"u","sha1":"s"}]}"#).unwrap();
    assert_eq!(manifest.versions[0].id, "26.2");
}

#[test]
fn accepts_live_manifest_metadata_drift_for_a_locked_version() {
    let manifest: Manifest =
        serde_json::from_str(r#"{"versions":[{"id":"26.2","url":"revised","sha1":"revised"}]}"#)
            .unwrap();
    let locked = Artifact {
        url: "locked".into(),
        sha1: "locked".into(),
        size: None,
    };
    assert!(!manifest_metadata_is_current(&manifest, "26.2", &locked).unwrap());
    assert!(manifest_metadata_is_current(&manifest, "missing", &locked).is_err());
}

#[test]
fn command_root_map_requires_an_exact_owned_partition() {
    let official = BTreeSet::from(["help".to_string()]);
    let rules = BTreeSet::from(["SIM-001".to_string()]);
    let mut map = CommandRootMap {
        version: "26.2".into(),
        inventory: CommandRootInventoryLock {
            expected_count: 1,
            roots_sha1: ids_digest(&official),
            expected_executable_count: 1,
            executable_paths_sha1: ids_digest(&official),
            expected_redirect_count: 0,
            redirect_paths_sha1: ids_digest(&BTreeSet::new()),
        },
        family: vec![CommandRootFamily {
            name: "informational".into(),
            roots: vec!["help".into()],
            owners: vec!["SIM-001".into()],
            state_domains: vec!["feedback".into()],
            status: CommandRootStatus::InProgress,
            remaining_work: vec!["audit leaves".into()],
        }],
    };
    validate_command_root_map(&map, &official, &rules).unwrap();
    map.family[0].roots.push("stale".into());
    assert!(validate_command_root_map(&map, &official, &rules).is_err());
}

#[test]
fn command_report_paths_lock_executables_and_redirects() {
    let report = serde_json::json!({
        "type": "root",
        "children": {
            "alias": {"type": "literal", "redirect": ["run"]},
            "run": {
                "type": "literal",
                "executable": true,
                "children": {
                    "target": {"type": "argument", "executable": true}
                }
            }
        }
    });
    let (executables, redirects) = command_report_paths(&report).unwrap();
    assert_eq!(
        executables,
        BTreeSet::from(["run".to_string(), "run target".to_string()])
    );
    assert_eq!(redirects, BTreeSet::from(["alias -> run".to_string()]));
}

#[test]
fn cross_system_join_map_requires_every_unordered_root_pair() {
    let surfaces = BTreeSet::from([
        BehaviorSurfaceKind::TickScheduler,
        BehaviorSurfaceKind::NetworkIngress,
        BehaviorSurfaceKind::CrossSystemOrdering,
    ]);
    let rules = BTreeSet::from(["SIM-001".to_string()]);
    let mut map = CrossSystemJoinMap {
        version: "26.2".into(),
        join: vec![CrossSystemJoin {
            left: BehaviorSurfaceKind::TickScheduler,
            right: BehaviorSurfaceKind::NetworkIngress,
            shared_domains: vec!["server thread".into()],
            owners: vec!["SIM-001".into()],
            status: CrossSystemJoinStatus::InProgress,
            remaining_work: vec!["specify ordering".into()],
        }],
    };
    validate_cross_system_join_map(&map, &surfaces, &rules).unwrap();
    map.join[0].right = BehaviorSurfaceKind::CrossSystemOrdering;
    assert!(validate_cross_system_join_map(&map, &surfaces, &rules).is_err());
}

#[test]
fn network_ingress_requires_exact_serverbound_family_partition() {
    let expected = BTreeSet::from(["required".to_string(), "optional".to_string()]);
    validate_exact_protocol_family_partition(&expected, &expected, "NetworkIngress").unwrap();
    let incomplete = BTreeSet::from(["required".to_string()]);
    assert!(
        validate_exact_protocol_family_partition(&incomplete, &expected, "NetworkIngress").is_err()
    );
}

#[test]
fn catalog_requires_exactly_one_family() {
    let catalog = Catalog {
        category: vec![Category {
            kind: "block".into(),
            source: "reports/blocks.json".into(),
            expected_count: 1,
            ids_sha1: "x".into(),
            family: vec![Family {
                name: "generic".into(),
                classification: Classification::DataOnly,
                rules: vec!["BLK-001".into()],
                exact: vec![],
                patterns: vec![],
                block_items: false,
                remaining: true,
            }],
        }],
    };
    assert_eq!(
        classify(&catalog, "block", "minecraft:stone", None)
            .unwrap()
            .family
            .name,
        "generic"
    );
}

#[test]
fn compiled_catalog_selectors_preserve_exact_pattern_and_fallback_matching() {
    let family = |name: &str, exact: Vec<&str>, patterns: Vec<&str>, remaining| Family {
        name: name.into(),
        classification: Classification::BehaviorFamily,
        rules: vec!["BLK-001".into()],
        exact: exact.into_iter().map(str::to_string).collect(),
        patterns: patterns.into_iter().map(str::to_string).collect(),
        block_items: false,
        remaining,
    };
    let catalog = Catalog {
        category: vec![Category {
            kind: "block".into(),
            source: "reports/blocks.json".into(),
            expected_count: 3,
            ids_sha1: "x".into(),
            family: vec![
                family("exact", vec!["stone"], vec![], false),
                family("pattern", vec![], vec!["*_stairs"], false),
                family("fallback", vec![], vec![], true),
            ],
        }],
    };
    assert_eq!(
        classify(&catalog, "block", "minecraft:stone", None)
            .unwrap()
            .family
            .name,
        "exact"
    );
    assert_eq!(
        classify(&catalog, "block", "minecraft:oak_stairs", None)
            .unwrap()
            .family
            .name,
        "pattern"
    );
    assert_eq!(
        classify(&catalog, "block", "minecraft:dirt", None)
            .unwrap()
            .family
            .name,
        "fallback"
    );
}

#[test]
fn catalog_rejects_stale_exact_ids_and_zero_match_patterns() {
    let category = Category {
        kind: "entity_type".into(),
        source: "reports/registries.json#minecraft:entity_type".into(),
        expected_count: 1,
        ids_sha1: "x".into(),
        family: vec![Family {
            name: "projectile".into(),
            classification: Classification::BehaviorFamily,
            rules: vec!["ENT-004".into()],
            exact: vec!["removed_projectile".into()],
            patterns: vec!["*_missing_pattern".into()],
            block_items: false,
            remaining: false,
        }],
    };
    let ids = BTreeSet::from(["minecraft:arrow".to_string()]);
    assert!(validate_family_selectors(&category, &ids, &BTreeSet::new()).is_err());
}

#[test]
fn catalog_rejects_special_remaining_fallbacks() {
    let category = Category {
        kind: "item".into(),
        source: "reports/minecraft/components/item/<id>.json".into(),
        expected_count: 1,
        ids_sha1: "x".into(),
        family: vec![Family {
            name: "remaining-special-items".into(),
            classification: Classification::Special,
            rules: vec!["ITM-001".into()],
            exact: vec![],
            patterns: vec![],
            block_items: false,
            remaining: true,
        }],
    };
    let ids = BTreeSet::from(["minecraft:stick".to_string()]);
    assert!(validate_family_selectors(&category, &ids, &BTreeSet::new()).is_err());
}

#[test]
fn catalog_rejects_unapproved_data_only_fallbacks() {
    let category = Category {
        kind: "worldgen".into(),
        source: "data/minecraft/worldgen/**".into(),
        expected_count: 1,
        ids_sha1: "x".into(),
        family: vec![Family {
            name: "remaining-worldgen".into(),
            classification: Classification::DataOnly,
            rules: vec!["WGEN-001".into()],
            exact: vec![],
            patterns: vec![],
            block_items: false,
            remaining: true,
        }],
    };
    let ids = BTreeSet::from(["minecraft:worldgen/example".to_string()]);
    assert!(validate_family_selectors(&category, &ids, &BTreeSet::new()).is_err());
}

#[test]
fn unreviewed_fallback_is_not_reported_as_data_only() {
    let catalog = Catalog {
        category: vec![Category {
            kind: "block".into(),
            source: "reports/blocks.json".into(),
            expected_count: 1,
            ids_sha1: "x".into(),
            family: vec![Family {
                name: "unreviewed-block".into(),
                classification: Classification::Unreviewed,
                rules: vec!["BLK-001".into()],
                exact: vec![],
                patterns: vec![],
                block_items: false,
                remaining: true,
            }],
        }],
    };
    let matched = classify(&catalog, "block", "minecraft:stone", None).unwrap();
    assert_eq!(matched.family.classification, Classification::Unreviewed);
}

#[test]
fn verifies_cached_artifact_hash_and_size() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("artifact.bin");
    fs::write(&path, b"locked").unwrap();
    verify_file(&path, &sha1_bytes(b"locked"), Some(6)).unwrap();
    assert!(verify_file(&path, &sha1_bytes(b"changed"), Some(6)).is_err());
    assert!(verify_file(&path, &sha1_bytes(b"locked"), Some(7)).is_err());
}

#[test]
fn parses_batched_javap_output_by_class() {
    let classes = vec![
        "net.minecraft.Test".to_string(),
        "net.minecraft.Test$Nested".to_string(),
    ];
    let output = "\
Classfile jar:file:///locked.jar!/net/minecraft/Test.class
  Compiled from \"Test.java\"
public class net.minecraft.Test {
  public void tick();
    descriptor: ()V
}
Classfile jar:file:///locked.jar!/net/minecraft/Test$Nested.class
  Compiled from \"Test.java\"
public class net.minecraft.Test$Nested {
  public int value();
    descriptor: ()I
}
";
    let parsed = parse_javap_batch(output, &classes).unwrap();
    assert!(parsed["net.minecraft.Test"].contains("tick"));
    assert!(parsed["net.minecraft.Test$Nested"].contains("value"));
}

#[test]
fn persistent_symbol_cache_round_trips_and_rejects_corruption() {
    let directory = tempdir().unwrap();
    write_symbol_cache(directory.path(), "net.minecraft.Test", "class output").unwrap();
    assert_eq!(
        read_symbol_cache(directory.path(), "net.minecraft.Test").unwrap(),
        Some("class output".to_string())
    );

    let path = symbol_cache_file(directory.path(), "net.minecraft.Test");
    let mut corrupted = fs::read_to_string(&path).unwrap();
    corrupted.push_str("corruption");
    fs::write(path, corrupted).unwrap();
    assert_eq!(
        read_symbol_cache(directory.path(), "net.minecraft.Test").unwrap(),
        None
    );
}

#[test]
fn symbol_cache_directory_changes_with_artifact_and_tool_identity() {
    let root = tempdir().unwrap();
    let context = Context {
        workspace: root.path().to_path_buf(),
        reference: root.path().to_path_buf(),
        cache: root.path().join("cache"),
        lock: toml::from_str(
            r#"
version = "26.2"
manifest_url = "https://example.invalid/manifest"
java_major = 21
data_pack = "107.1"
resource_pack = "88.0"
[metadata]
url = "https://example.invalid/metadata"
sha1 = "metadata"
[client]
url = "https://example.invalid/client"
sha1 = "client"
[server]
url = "https://example.invalid/server"
sha1 = "server"
"#,
        )
        .unwrap(),
    };
    let baseline = symbol_cache_directory(&context, "jar-a", "javap", "21.0.11");
    assert_ne!(
        baseline,
        symbol_cache_directory(&context, "jar-b", "javap", "21.0.11")
    );
    assert_ne!(
        baseline,
        symbol_cache_directory(&context, "jar-a", "javap", "22")
    );
}

#[test]
fn parses_report_id_paths() {
    let directory = tempdir().unwrap();
    let nested = directory.path().join("boats");
    fs::create_dir(&nested).unwrap();
    fs::write(directory.path().join("stone.json"), b"{}").unwrap();
    fs::write(nested.join("oak.json"), b"{}").unwrap();
    let ids = ids_from_files(directory.path(), "json").unwrap();
    assert_eq!(
        ids,
        BTreeSet::from([
            "minecraft:stone".to_string(),
            "minecraft:boats/oak".to_string()
        ])
    );
}

#[test]
fn generic_registry_queries_support_new_catalog_kinds() {
    let registries = serde_json::json!({
        "minecraft:ticket_type": {
            "entries": {
                "minecraft:portal": { "protocol_id": 6 },
                "minecraft:forced": { "protocol_id": 5 }
            }
        },
        "minecraft:worldgen/density_function_type": {
            "entries": {
                "minecraft:constant": { "protocol_id": 0 }
            }
        },
        "minecraft:worldgen/material_condition": {
            "entries": {
                "minecraft:stone_depth": { "protocol_id": 10 }
            }
        },
        "minecraft:worldgen/material_rule": {
            "entries": {
                "minecraft:sequence": { "protocol_id": 2 }
            }
        },
        "minecraft:worldgen/structure_type": {
            "entries": {
                "minecraft:buried_treasure": { "protocol_id": 0 }
            }
        },
        "minecraft:worldgen/pool_alias_binding": {
            "entries": {
                "minecraft:direct": { "protocol_id": 2 }
            }
        },
        "minecraft:worldgen/structure_pool_element": {
            "entries": {
                "minecraft:list_pool_element": { "protocol_id": 1 }
            }
        },
        "minecraft:worldgen/structure_processor": {
            "entries": {
                "minecraft:rule": { "protocol_id": 10 }
            }
        }
    });
    assert_eq!(
        registry_ids(&registries, "ticket_type").unwrap(),
        BTreeSet::from([
            "minecraft:forced".to_string(),
            "minecraft:portal".to_string()
        ])
    );
    assert_eq!(
        registry_entry(&registries, "ticket_type", "minecraft:portal").unwrap()["protocol_id"],
        6
    );
    assert_eq!(
        registry_ids(&registries, "density_function_type").unwrap(),
        BTreeSet::from(["minecraft:constant".to_string()])
    );
    assert_eq!(
        registry_entry(&registries, "density_function_type", "minecraft:constant").unwrap()["protocol_id"],
        0
    );
    assert_eq!(
        registry_ids(&registries, "material_condition").unwrap(),
        BTreeSet::from(["minecraft:stone_depth".to_string()])
    );
    assert_eq!(
        registry_entry(&registries, "material_rule", "minecraft:sequence").unwrap()["protocol_id"],
        2
    );
    assert_eq!(
        registry_ids(&registries, "structure_type").unwrap(),
        BTreeSet::from(["minecraft:buried_treasure".to_string()])
    );
    assert_eq!(
        registry_entry(&registries, "structure_type", "minecraft:buried_treasure").unwrap()["protocol_id"],
        0
    );
    assert_eq!(
        registry_entry(&registries, "pool_alias_binding", "minecraft:direct").unwrap()["protocol_id"],
        2
    );
    assert_eq!(
        registry_ids(&registries, "structure_pool_element").unwrap(),
        BTreeSet::from(["minecraft:list_pool_element".to_string()])
    );
    assert_eq!(
        registry_entry(&registries, "structure_processor", "minecraft:rule").unwrap()["protocol_id"],
        10
    );
    assert!(registry_entry(&registries, "ticket_type", "minecraft:removed").is_err());
}

#[test]
fn data_backed_catalog_kinds_have_locked_jar_paths() {
    assert_eq!(
        server_data_prefix("sulfur_cube_archetype"),
        Some("data/minecraft/sulfur_cube_archetype")
    );
    assert_eq!(server_data_prefix("entity_type"), None);
}

#[test]
fn parses_experiment_definition_schema() {
    let file: ExperimentFile = toml::from_str(
        r#"
                [[experiment]]
                id = "EXP-TST-001"
                rules = ["SIM-001"]
                mode = "gametest"
                status = "planned"
                repeats = 1
                initial_state = ["empty"]
                action = [{ tick = 0, value = "act" }]
                observation = [{ tick = 1, value = "observe" }]
                expected = ["done"]
            "#,
    )
    .unwrap();
    assert_eq!(file.experiment[0].id, "EXP-TST-001");
    assert_eq!(file.experiment[0].observation[0].tick, 1);
}

#[test]
fn parses_completion_ledger_schema() {
    let completion: CompletionFile = toml::from_str(
        r#"
                version = "26.2"
                [[slice]]
                id = "TST-SLICE-001"
                subsystem = "test"
                parents = ["SIM-001"]
                leaves = ["SIM-PIPELINE-001"]
                registry_kinds = []
                selectors = ["minecraft:stone"]
                symbols = ["net.minecraft.Test#tick"]
                data_paths = []
                status = "SourceInconclusive"
                unknowns = ["Client presentation is outside the server source boundary."]
                reproduction = ["Observe one client tick after the server event."]
                experiments = ["EXP-SIM-001"]
                last_commit = "deadbee"

                [[registry]]
                id = "minecraft:block"
                scope = "GameplayBehavior"
                reason = "Blocks select gameplay behavior."
            "#,
    )
    .unwrap();
    assert_eq!(completion.version, "26.2");
    assert!(completion_slice_has_ownership(&completion.slice[0]));
    assert!(completion.slice[0].registry_kinds.is_empty());
    assert_eq!(
        completion.slice[0].status,
        CompletionStatus::SourceInconclusive
    );
    assert_eq!(completion.registry[0].id, "minecraft:block");
}

#[test]
fn parses_protocol_completion_ledger_schema() {
    let parsed: ProtocolCompletionFile = toml::from_str(
        r#"
version = "26.2"
[inventory]
expected_count = 1
entries_sha1 = "abc"
[[family]]
id = "PROTO-STATUS-001"
level = "C0"
state = "status"
direction = "serverbound"
patterns = ["minecraft:status_request"]
status = "Todo"
responsibility = "Required"
owner = "protocol/handshake-and-status"
specification = ""
evidence = ["OFF-REPORT-001"]
fields = []
mappings = []
transitions = []
ordering = []
vectors = []
unknowns = ["field layout"]
reproduction = ["trace codec"]
last_commit = ""
"#,
    )
    .unwrap();
    assert_eq!(parsed.inventory.expected_count, 1);
    assert_eq!(parsed.family.len(), 1);
    assert_eq!(parsed.family[0].level, ProtocolLevel::C0);
}

#[test]
fn parses_behavior_surface_ledger_schema() {
    let parsed: BehaviorSurfaceFile = toml::from_str(
        r#"
version = "26.2"
[[surface]]
id = "SURFACE-TICK-SCHEDULER-001"
kind = "TickScheduler"
boundary = "server tick"
triggers = ["fixed tick"]
inventory_sources = ["OfficialServerSymbols"]
selectors = ["tick roots"]
owners = ["SIM-001"]
state_domains = ["world state"]
persistence = ["clock continuity"]
client_projection = ["time update"]
protocol_families = []
status = "Mapped"
evidence = ["OFF-SERVER-001"]
unknowns = []
reproduction = ["run the tick vector"]
last_commit = "deadbee"
"#,
    )
    .unwrap();
    assert_eq!(parsed.version, "26.2");
    assert_eq!(parsed.surface.len(), 1);
    assert_eq!(parsed.surface[0].kind, BehaviorSurfaceKind::TickScheduler);
    assert_eq!(parsed.surface[0].status, BehaviorSurfaceStatus::Mapped);
    assert_eq!(expected_surface_kinds().len(), 10);
}

#[test]
fn matches_jvm_descriptors_instead_of_generic_declarations() {
    let javap = "  public void tick(net.minecraft.server.level.ServerLevel, E);\n    descriptor: (Lnet/minecraft/server/level/ServerLevel;Lnet/minecraft/world/entity/LivingEntity;)V\n";
    assert!(descriptor_matches(
        javap,
        "tick",
        "(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity)"
    ));
}
