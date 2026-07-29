use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_gameplay::block::break_hook::{
    BreakHookKind, ExperienceProvider, HookPoints, break_experience_provider, break_hook,
    experience_provider, hook_points, sample_experience,
};
use ferrite_gameplay::block::breaking::{
    BreakCommitEffect, BreakCommitInputs, BreakProgressTracker, MiningInputs, ProgressEffect,
    ProgressRecord, destroy_progress, plan_break_commit,
};
use ferrite_gameplay::block::placement::{
    BED_FOOT_FLAGS, BlockItemKind, COMPONENT_PATCH_FLAGS, DOUBLE_HIGH_CLEAR_FLAGS, DoorHinge,
    PlacementKind, PlacementRequest, PlacementWriteResults, SECOND_HALF_FLAGS, block_item_kind,
    door_hinge, plan_placement, scaffolding_horizontal_extension_allowed,
};
use ferrite_registry::bundle::ContentBundle;
use ferrite_registry::minecraft_block::MinecraftBlockCatalog;
use ferrite_simulation::random::DeterministicRng;
use ferrite_world::id::BlockStateId;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const TARGET: BlockPos = BlockPos::new(1, 64, 1);
const UPPER: BlockPos = BlockPos::new(1, 65, 1);

#[test]
fn placement_preserves_partial_writes_and_post_write_success() {
    let request = PlacementRequest {
        target: TARGET,
        candidate: BlockStateId::new(10),
        second_half: Some((UPPER, BlockStateId::new(11))),
        kind: PlacementKind::DoubleHigh {
            upper_replacement: BlockStateId::new(0),
        },
        component_patch: Some(BlockStateId::new(12)),
        consumes_item: true,
    };
    let failed = plan_placement(
        request,
        PlacementWriteResults {
            initial: false,
            current_has_candidate_block: false,
        },
    );
    assert!(!failed.success);
    assert_eq!(failed.writes.len(), 2);
    assert_eq!(failed.writes[0].flags, DOUBLE_HIGH_CLEAR_FLAGS);
    assert!(!failed.writes[0].result_matters);
    assert!(failed.writes[1].result_matters);

    let replaced = plan_placement(
        request,
        PlacementWriteResults {
            initial: true,
            current_has_candidate_block: false,
        },
    );
    assert!(replaced.success);
    assert!(replaced.emits_sound_and_game_event);
    assert!(replaced.consumes_item);
    assert!(!replaced.calls_set_placed_by);
    assert_eq!(replaced.writes.len(), 2);

    let complete = plan_placement(
        request,
        PlacementWriteResults {
            initial: true,
            current_has_candidate_block: true,
        },
    );
    assert_eq!(
        complete
            .writes
            .iter()
            .map(|write| write.flags)
            .collect::<Vec<_>>(),
        [
            DOUBLE_HIGH_CLEAR_FLAGS,
            11,
            COMPONENT_PATCH_FLAGS,
            SECOND_HALF_FLAGS
        ]
    );
    assert!(complete.applies_block_entity_data);
    assert!(complete.emits_placed_criterion);
}

#[test]
fn bed_and_scaffolding_boundaries_use_locked_flags_and_limit() {
    let bed = plan_placement(
        PlacementRequest {
            target: TARGET,
            candidate: BlockStateId::new(20),
            second_half: Some((UPPER, BlockStateId::new(21))),
            kind: PlacementKind::Bed,
            component_patch: None,
            consumes_item: true,
        },
        PlacementWriteResults {
            initial: true,
            current_has_candidate_block: true,
        },
    );
    assert_eq!(bed.writes[0].flags, BED_FOOT_FLAGS);
    assert_eq!(bed.writes[1].flags, SECOND_HALF_FLAGS);
    assert!(scaffolding_horizontal_extension_allowed(6));
    assert!(!scaffolding_horizontal_extension_allowed(7));
    assert_eq!(block_item_kind("oak_door"), BlockItemKind::DoubleHigh);
    assert_eq!(block_item_kind("white_bed"), BlockItemKind::Bed);
    assert_eq!(
        block_item_kind("oak_hanging_sign"),
        BlockItemKind::StandingAndWall
    );
    assert_eq!(
        block_item_kind("powder_snow_bucket"),
        BlockItemKind::SolidBucket
    );
    assert_eq!(
        door_hinge(Direction::North, false, false, 0, 0, 0.5, 0.5),
        DoorHinge::Left
    );
    assert_eq!(
        door_hinge(Direction::East, false, false, 0, 0, 0.5, 0.75),
        DoorHinge::Right
    );
}

#[test]
fn mining_speed_uses_java_float_order_and_locked_divisors() {
    let base = MiningInputs {
        hardness: 2.0,
        item_speed: 4.0,
        mining_efficiency: 2.0,
        dig_speed_amplifier: Some(0),
        mining_fatigue_amplifier: None,
        block_break_speed: 1.0,
        submerged_mining_speed: 0.2,
        eyes_in_water: true,
        on_ground: false,
        correct_tool: true,
    };
    let expected = ((((4.0_f32 + 2.0) * 1.2) * 0.2) / 5.0) / 2.0 / 30.0;
    assert_eq!(destroy_progress(base), expected);
    assert_eq!(
        destroy_progress(MiningInputs {
            hardness: -1.0,
            ..base
        }),
        0.0
    );
    assert!(
        destroy_progress(MiningInputs {
            hardness: 0.0,
            ..base
        })
        .is_infinite()
    );
    assert_eq!(
        destroy_progress(MiningInputs {
            correct_tool: false,
            ..base
        }),
        expected * 0.3
    );
}

#[test]
fn progress_tracker_preserves_active_delayed_and_stage_quirks() {
    let mut tracker = BreakProgressTracker::default();
    assert_eq!(
        tracker.start(TARGET, true, 0.0),
        [ProgressEffect::Publish {
            position: TARGET,
            stage: 10
        }]
    );
    let other = BlockPos::new(2, 64, 1);
    assert_eq!(
        tracker.start(other, false, 0.1),
        [
            ProgressEffect::Correct(TARGET),
            ProgressEffect::Publish {
                position: other,
                stage: 1
            }
        ]
    );
    tracker.game_ticks = 5;
    assert_eq!(
        tracker.start(TARGET, false, 1.0),
        [ProgressEffect::Destroy(TARGET)]
    );
    assert!(tracker.is_destroying);
    assert_eq!(tracker.destroy_record.unwrap().position, other);
    assert_eq!(tracker.destroy_record.unwrap().started_at, 5);
    assert!(tracker.stop(other, false, 0.2).is_empty());
    assert!(!tracker.is_destroying);
    assert!(tracker.delayed.is_some());

    tracker.is_destroying = true;
    tracker.destroy_record = Some(ProgressRecord {
        position: TARGET,
        started_at: 0,
    });
    let effects = tracker.tick(false, 0.6);
    assert!(tracker.delayed.is_none());
    assert!(tracker.is_destroying);
    assert!(effects.contains(&ProgressEffect::Destroy(other)));
    assert_eq!(
        tracker.abort(other),
        [
            ProgressEffect::Publish {
                position: TARGET,
                stage: -1
            },
            ProgressEffect::Publish {
                position: other,
                stage: -1
            }
        ]
    );
    assert!(!tracker.is_destroying);
    assert!(tracker.destroy_record.is_some());
}

#[test]
fn generic_break_commit_preserves_callbacks_and_failed_removal_success() {
    let baseline = BreakCommitInputs {
        item_allows_destroy: true,
        game_master_allows_destroy: true,
        action_restricted: false,
        removal_succeeded: false,
        prevents_drops: false,
        tool_component_present: true,
        damage_per_block: 2,
        destroyed_hardness_nonzero: true,
        shears_on_fire: false,
        correct_tool: true,
        block_drops_enabled: true,
    };
    let commit = plan_break_commit(baseline);
    assert!(commit.accepted);
    assert!(commit.effects.contains(&BreakCommitEffect::MineBlock));
    assert!(commit.effects.contains(&BreakCommitEffect::DamageTool(2)));
    assert!(
        !commit
            .effects
            .contains(&BreakCommitEffect::BlockDestroyHook)
    );
    assert!(!commit.effects.contains(&BreakCommitEffect::EvaluateLoot));

    let no_drops = plan_break_commit(BreakCommitInputs {
        removal_succeeded: true,
        block_drops_enabled: false,
        ..baseline
    });
    assert!(no_drops.effects.contains(&BreakCommitEffect::EvaluateLoot));
    assert!(
        no_drops
            .effects
            .contains(&BreakCommitEffect::SpawnAfterBreak)
    );
    assert!(!no_drops.effects.contains(&BreakCommitEffect::SpawnLoot));
}

#[test]
fn concrete_break_hook_map_is_exhaustive_for_locked_catalog() {
    let Some(catalog) = local_catalog() else {
        eprintln!("locked local artifact bundle absent; content verification owns that gate");
        return;
    };
    let hooks = catalog
        .definitions()
        .filter_map(|definition| break_hook(definition.persistent_id().resource().path()))
        .collect::<Vec<_>>();
    assert_eq!(hooks.len(), 110);
    assert_eq!(hooks.into_iter().collect::<BTreeSet<_>>().len(), 23);
    assert_eq!(break_hook("stone"), None);
    assert_eq!(break_hook("white_bed"), Some(BreakHookKind::Bed));
    assert_eq!(break_hook("oak_door"), Some(BreakHookKind::Door));
    assert!(hook_points(BreakHookKind::Beehive).contains(HookPoints::PLAYER_WILL_DESTROY));
    assert!(hook_points(BreakHookKind::Beehive).contains(HookPoints::PLAYER_DESTROY));
    assert!(hook_points(BreakHookKind::Spawner).contains(HookPoints::SPAWN_AFTER_BREAK));
}

#[test]
fn experience_providers_preserve_bounds_and_rng_cardinality() {
    assert_eq!(
        experience_provider("diamond_ore"),
        Some(ExperienceProvider::Uniform {
            minimum: 3,
            maximum: 7
        })
    );
    assert_eq!(
        break_experience_provider("redstone_ore"),
        Some(ExperienceProvider::Uniform {
            minimum: 1,
            maximum: 5
        })
    );
    assert_eq!(
        break_experience_provider("sculk_sensor"),
        Some(ExperienceProvider::Constant(5))
    );
    let mut constant_rng = DeterministicRng::from_seed(7);
    let unchanged = constant_rng.state();
    assert_eq!(
        sample_experience(ExperienceProvider::Constant(5), &mut constant_rng),
        5
    );
    assert_eq!(constant_rng.state(), unchanged);

    for seed in 0..64 {
        let mut random = DeterministicRng::from_seed(seed);
        let value = sample_experience(ExperienceProvider::SpawnerTriangular, &mut random);
        assert!((15..=43).contains(&value));
    }
}

fn local_catalog() -> Option<MinecraftBlockCatalog> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("../../target/ferrite-content/26.2/content-bundle.json");
    let bytes = fs::read(path).ok()?;
    let bundle = serde_json::from_slice::<ContentBundle>(&bytes).ok()?;
    let registry = bundle
        .registries()
        .find(|registry| registry.name().to_string() == "minecraft:block")?;
    MinecraftBlockCatalog::from_registry(registry).ok()
}
