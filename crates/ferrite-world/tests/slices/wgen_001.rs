use std::cell::RefCell;
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::resource::ResourceId;
use ferrite_world::generation::access::{GenerationAccessError, GenerationPyramid};
use ferrite_world::generation::feature::direct_write::{
    DIRECT_WRITE_FLAGS, DirectWriteWorld, ReplacementRule, ReplacementTarget, fill_layer,
    replace_single_block,
};
use ferrite_world::generation::feature::modifier::{
    PlacementContext, PlacementModifierSpec, PlacementWorld, VerticalDirection,
};
use ferrite_world::generation::feature::placement::{
    PlacementError, PlacementModifier, place_with_modifiers,
};
use ferrite_world::generation::feature::platform::{
    END_PLATFORM_WRITE_FLAGS, EndPlatformStates, PlatformWorld, VOID_PLATFORM_WRITE_FLAGS,
    VoidPlatformStates, create_end_platform, place_end_platform, place_void_start_platform,
};
use ferrite_world::generation::feature::predicate::{
    BlockPredicate, PredicateOffset, PredicateWorld,
};
use ferrite_world::generation::feature::provider::{
    HeightAnchor, HeightContext, IntProvider, WeightedInt, uniform_height,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::selector::{
    random_boolean_selector, random_selector, sequence, simple_random_selector,
    weighted_random_selector,
};
use ferrite_world::generation::feature::{place_configured, place_no_op};
use ferrite_world::generation::status::{ChunkStatus, GenerationHeightmap};
use ferrite_world::generation::task::{PyramidMode, StatusTaskPlan, TaskOperation, TaskOptions};
use ferrite_world::id::BlockStateId;

#[test]
fn direct_dependency_table_matches_every_radius() {
    use ChunkStatus::{
        Biomes, Carvers, Empty, Features, Full, InitializeLight, Light, Noise, Spawn,
        StructureReferences, StructureStarts, Surface,
    };

    let expected = [
        (Empty, [None; 9]),
        (
            StructureStarts,
            [Some(Empty), None, None, None, None, None, None, None, None],
        ),
        (StructureReferences, [Some(StructureStarts); 9]),
        (
            Biomes,
            [
                Some(StructureReferences),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
            ],
        ),
        (
            Noise,
            [
                Some(Biomes),
                Some(Biomes),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
            ],
        ),
        (
            Surface,
            [
                Some(Noise),
                Some(Biomes),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
            ],
        ),
        (
            Carvers,
            [
                Some(Surface),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
            ],
        ),
        (
            Features,
            [
                Some(Carvers),
                Some(Carvers),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
                Some(StructureStarts),
            ],
        ),
        (
            InitializeLight,
            [
                Some(Features),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            Light,
            [
                Some(InitializeLight),
                Some(InitializeLight),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            Spawn,
            [
                Some(Light),
                Some(Biomes),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ),
        (
            Full,
            [Some(Spawn), None, None, None, None, None, None, None, None],
        ),
    ];
    for (target, statuses) in expected {
        assert_eq!(
            (0..=8)
                .map(|radius| target.direct_requirement(radius))
                .collect::<Vec<_>>(),
            statuses
        );
        assert_eq!(target.direct_requirement(9), None);
    }
}

#[test]
fn pyramid_fails_closed_for_missing_or_old_dependencies() {
    let center = ChunkPos::new(-3, 4);
    let dependencies = dependencies(center, ChunkStatus::Surface);
    let pyramid = GenerationPyramid::new(
        center,
        ChunkStatus::Surface,
        dependencies.clone(),
        false,
        -64..320,
    )
    .unwrap();
    assert_eq!(pyramid.resolve(center).unwrap(), ChunkStatus::Noise);

    let mut missing = dependencies.clone();
    missing.remove(&center);
    assert!(matches!(
        GenerationPyramid::new(center, ChunkStatus::Surface, missing, false, -64..320,),
        Err(GenerationAccessError::MissingDependency { .. })
    ));

    let mut old = dependencies;
    old.insert(center, ChunkStatus::Biomes);
    assert!(matches!(
        GenerationPyramid::new(center, ChunkStatus::Surface, old, false, -64..320),
        Err(GenerationAccessError::DependencyTooOld { .. })
    ));
}

#[test]
fn write_radius_and_retrogen_height_are_both_authoritative() {
    let center = ChunkPos::new(0, 0);
    let features = GenerationPyramid::new(
        center,
        ChunkStatus::Features,
        dependencies(center, ChunkStatus::Features),
        false,
        -64..320,
    )
    .unwrap();
    assert!(features.ensure_can_write(BlockPos::new(31, 319, 31)));
    assert!(!features.ensure_can_write(BlockPos::new(32, 64, 0)));

    let retrogen = GenerationPyramid::new(
        center,
        ChunkStatus::Noise,
        dependencies(center, ChunkStatus::Noise),
        true,
        0..64,
    )
    .unwrap();
    assert!(retrogen.ensure_can_write(BlockPos::new(0, 63, 0)));
    assert!(!retrogen.ensure_can_write(BlockPos::new(0, 64, 0)));
    assert!(!retrogen.ensure_can_write(BlockPos::new(16, 32, 0)));

    let no_writes = GenerationPyramid::new(
        center,
        ChunkStatus::Light,
        dependencies(center, ChunkStatus::Light),
        false,
        -64..320,
    )
    .unwrap();
    assert!(!no_writes.ensure_can_write(BlockPos::new(0, 0, 0)));
}

#[test]
fn generation_and_loading_task_plans_lock_all_branch_ordering() {
    use TaskOperation::{
        ApplyBedrockHoleMask, ApplyCarvers, BuildSurface, CreateBiomes, CreateStructureManager,
        CreateStructureReferences, DecorateBiomes, FillFromNoise, GenerateBlendingBorderTicks,
        GenerateStructureStarts, InstallOldChunkCarvingMask, NotifyStructureStartsAvailable,
        ReplaceOldBedrock,
    };

    let options = TaskOptions {
        below_zero_retrogen: true,
        apply_bedrock_hole_mask: true,
        ..TaskOptions::default()
    };
    assert_eq!(
        plan(
            ChunkStatus::StructureStarts,
            PyramidMode::Generation,
            options
        ),
        vec![GenerateStructureStarts, NotifyStructureStartsAvailable]
    );
    assert_eq!(
        plan(ChunkStatus::StructureStarts, PyramidMode::Loading, options),
        vec![NotifyStructureStartsAvailable]
    );
    assert_eq!(
        plan(
            ChunkStatus::StructureReferences,
            PyramidMode::Generation,
            options
        ),
        vec![CreateStructureManager, CreateStructureReferences]
    );
    assert_eq!(
        plan(ChunkStatus::Biomes, PyramidMode::Generation, options),
        vec![CreateStructureManager, CreateBiomes]
    );
    assert_eq!(
        plan(ChunkStatus::Noise, PyramidMode::Generation, options),
        vec![FillFromNoise, ReplaceOldBedrock, ApplyBedrockHoleMask]
    );
    assert_eq!(
        plan(ChunkStatus::Surface, PyramidMode::Generation, options),
        vec![BuildSurface]
    );
    assert_eq!(
        plan(ChunkStatus::Carvers, PyramidMode::Generation, options),
        vec![InstallOldChunkCarvingMask, ApplyCarvers]
    );
    let features = plan(ChunkStatus::Features, PyramidMode::Generation, options);
    assert_eq!(features.len(), 6);
    assert_eq!(features[4], DecorateBiomes);
    assert_eq!(features[5], GenerateBlendingBorderTicks);
}

#[test]
fn light_spawn_feature_and_full_edge_branches_are_explicit() {
    let lighted = TaskOptions {
        light_correct: true,
        ..TaskOptions::default()
    };
    assert_eq!(
        StatusTaskPlan::new(
            ChunkStatus::Light,
            PyramidMode::Generation,
            ChunkStatus::Light,
            lighted,
        )
        .operations,
        [TaskOperation::LightChunk {
            already_lighted: true
        }]
    );
    let debug = TaskOptions {
        debug_disable_features: true,
        ..TaskOptions::default()
    };
    let debug_features = plan(ChunkStatus::Features, PyramidMode::Generation, debug);
    assert!(!debug_features.contains(&TaskOperation::DecorateBiomes));
    assert_eq!(
        debug_features.last(),
        Some(&TaskOperation::GenerateBlendingBorderTicks)
    );
    let upgrading = TaskOptions {
        upgrading: true,
        ..TaskOptions::default()
    };
    assert!(plan(ChunkStatus::Spawn, PyramidMode::Generation, upgrading).is_empty());
    assert_eq!(
        plan(
            ChunkStatus::Full,
            PyramidMode::Generation,
            TaskOptions::default()
        ),
        [
            TaskOperation::ResolveOrConstructLevelChunk,
            TaskOperation::ReplaceProtochunkWhenConstructed,
            TaskOperation::InstallFullStatusSupplier,
            TaskOperation::LoadPostLoadEntities,
            TaskOperation::MarkLoaded,
            TaskOperation::RegisterBlockEntities,
            TaskOperation::RegisterTickContainers,
            TaskOperation::InstallUnsavedListener,
        ]
    );
}

#[test]
fn configured_feature_origin_gate_precedes_context_and_no_op() {
    let origin = BlockPos::new(8, 64, 8);
    let mut algorithm_calls = 0;
    assert!(!place_configured(
        origin,
        |position| {
            assert_eq!(position, origin);
            false
        },
        || {
            algorithm_calls += 1;
            true
        }
    ));
    assert_eq!(algorithm_calls, 0);
    assert!(place_configured(
        origin,
        |position| position == origin,
        || {
            algorithm_calls += 1;
            place_no_op()
        }
    ));
    assert_eq!(algorithm_calls, 1);
}

#[test]
fn all_five_composite_selectors_lock_draw_and_short_circuit_order() {
    let mut random = ScriptedRandom::new([0, 1, 0], [0.5, 0.25, 0.75]);
    let mut calls = Vec::new();
    assert!(
        !random_selector(&[0.5, 0.5], &mut random, |index| {
            calls.push(index);
            false
        })
        .unwrap()
    );
    assert_eq!(calls, [1]);
    assert_eq!(random.float_draws, 2);

    let mut random = ScriptedRandom::new([3], []);
    calls.clear();
    assert!(
        weighted_random_selector(&[0, 2, 3], &mut random, |index| {
            calls.push(index);
            true
        })
        .unwrap()
    );
    assert_eq!(calls, [2]);
    assert_eq!(random.integer_bounds, [5]);

    let mut no_draw = ScriptedRandom::new([], []);
    assert!(
        !weighted_random_selector(&[0, 0], &mut no_draw, |_| {
            panic!("zero total must not call a child")
        })
        .unwrap()
    );
    assert!(no_draw.integer_bounds.is_empty());

    let mut random = ScriptedRandom::new([2, 1], []);
    assert!(simple_random_selector(3, &mut random, |index| index == 2).unwrap());
    assert!(random_boolean_selector(
        &mut random,
        || true,
        || panic!("true draw must select the true child")
    ));
    calls.clear();
    assert!(
        !sequence(4, |index| {
            calls.push(index);
            index < 2
        })
        .unwrap()
    );
    assert_eq!(calls, [0, 1, 2]);
}

#[test]
fn placed_modifier_flat_map_is_depth_first_shared_and_bounded() {
    let first = OffsetModifier {
        offsets: vec![(1, 0), (2, 0)],
    };
    let second = OffsetModifier {
        offsets: vec![(0, 1), (0, 2)],
    };
    let modifiers: [&dyn PlacementModifier<ScriptedRandom>; 2] = [&first, &second];
    let mut random = ScriptedRandom::new([], []);
    let mut visited = Vec::new();
    let report = place_with_modifiers(
        BlockPos::new(0, 64, 0),
        &modifiers,
        &mut random,
        4,
        |position| {
            visited.push(position);
            position == BlockPos::new(1, 64, 2)
        },
    )
    .unwrap();
    assert_eq!(
        visited,
        [
            BlockPos::new(1, 64, 1),
            BlockPos::new(1, 64, 2),
            BlockPos::new(2, 64, 1),
            BlockPos::new(2, 64, 2),
        ]
    );
    assert_eq!(report.terminal_positions, 4);
    assert!(report.any_placed);

    let mut visited = Vec::new();
    assert_eq!(
        place_with_modifiers(
            BlockPos::new(0, 64, 0),
            &modifiers,
            &mut random,
            2,
            |position| {
                visited.push(position);
                true
            },
        ),
        Err(PlacementError::TerminalCapacity { capacity: 2 })
    );
    assert_eq!(visited.len(), 2);
}

#[test]
fn integer_and_height_providers_lock_draw_order_and_bounds() {
    let mut random = ScriptedRandom::new([4, 3, 1, 4, 2, 2, 1], []);
    assert_eq!(
        IntProvider::Uniform {
            minimum: -2,
            maximum: 2,
        }
        .sample(&mut random)
        .unwrap(),
        2
    );
    assert_eq!(
        IntProvider::BiasedToBottom {
            minimum: 10,
            maximum: 14,
        }
        .sample(&mut random)
        .unwrap(),
        11
    );
    assert_eq!(
        IntProvider::ZeroPlateauTrapezoid { radius: 4 }
            .sample(&mut random)
            .unwrap(),
        2
    );
    let weighted = IntProvider::Weighted(vec![
        WeightedInt {
            weight: NonZeroU32::new(1).unwrap(),
            provider: IntProvider::Constant(10),
        },
        WeightedInt {
            weight: NonZeroU32::new(3).unwrap(),
            provider: IntProvider::Uniform {
                minimum: 20,
                maximum: 22,
            },
        },
    ]);
    assert_eq!(weighted.sample(&mut random).unwrap(), 21);
    assert_eq!(random.integer_bounds, [5, 5, 4, 5, 5, 4, 3]);

    random.gaussian = Some(1.75);
    assert_eq!(
        IntProvider::ClampedNormal {
            mean: -1.0,
            deviation: 2.0,
            minimum: -3,
            maximum: 3,
        }
        .sample(&mut random)
        .unwrap(),
        2
    );

    let context = HeightContext {
        minimum_y: -64,
        depth: 384,
    };
    assert_eq!(HeightAnchor::AboveBottom(10).resolve(context).unwrap(), -54);
    assert_eq!(HeightAnchor::BelowTop(5).resolve(context).unwrap(), 314);
    let mut no_draw = ScriptedRandom::new([], []);
    assert_eq!(
        uniform_height(
            HeightAnchor::Absolute(10),
            HeightAnchor::Absolute(5),
            context,
            &mut no_draw,
        )
        .unwrap(),
        10
    );
    assert!(no_draw.integer_bounds.is_empty());
}

#[test]
fn concrete_modifiers_lock_rng_order_strictness_and_duplicates() {
    let world = ModifierWorld::default();
    let context = PlacementContext::plain(&world);
    let origin = BlockPos::new(-16, 64, 32);
    let mut output = Vec::new();
    let mut random = ScriptedRandom::new([15, 2], [0.25, 0.249]);

    PlacementModifierSpec::InSquare
        .apply(context, origin, &mut random, &mut output)
        .unwrap();
    assert_eq!(output, [BlockPos::new(-1, 64, 34)]);
    assert_eq!(random.integer_bounds, [16, 16]);

    output.clear();
    let rarity = PlacementModifierSpec::RarityFilter {
        chance: NonZeroU32::new(4).unwrap(),
    };
    rarity
        .apply(context, origin, &mut random, &mut output)
        .unwrap();
    assert!(output.is_empty(), "equality must fail the strict gate");
    rarity
        .apply(context, origin, &mut random, &mut output)
        .unwrap();
    assert_eq!(output, [origin]);

    output.clear();
    PlacementModifierSpec::Count {
        count: IntProvider::Constant(3),
    }
    .apply(context, origin, &mut random, &mut output)
    .unwrap();
    assert_eq!(output, [origin, origin, origin]);

    output.clear();
    let mut random = ScriptedRandom::new([2, 0, 4], []);
    PlacementModifierSpec::RandomOffset {
        horizontal: IntProvider::Uniform {
            minimum: -2,
            maximum: 2,
        },
        vertical: IntProvider::Uniform {
            minimum: 7,
            maximum: 7,
        },
    }
    .apply(context, origin, &mut random, &mut output)
    .unwrap();
    assert_eq!(output, [BlockPos::new(-16, 71, 34)]);
    assert_eq!(random.integer_bounds, [5, 1, 5]);

    output.clear();
    PlacementModifierSpec::Count {
        count: IntProvider::Constant(4_097),
    }
    .apply(context, origin, &mut random, &mut output)
    .unwrap_err();
    assert!(output.is_empty());
}

#[test]
fn world_query_modifiers_preserve_read_order_and_biome_boundary() {
    let world = ModifierWorld {
        biome_matches: true,
        noise: 0.25,
        ..ModifierWorld::default()
    };
    let origin = BlockPos::new(8, 64, 9);
    let mut output = Vec::new();
    let mut random = ScriptedRandom::new([], []);

    PlacementModifierSpec::SurfaceWaterDepth {
        maximum_water_depth: 4,
    }
    .apply(
        PlacementContext::plain(&world),
        origin,
        &mut random,
        &mut output,
    )
    .unwrap();
    assert_eq!(output, [origin]);
    assert_eq!(
        world.take_trace(),
        ["height:ocean_floor", "height:world_surface"]
    );

    output.clear();
    let feature = ResourceId::minecraft("placed_feature/test").unwrap();
    let biome = PlacementModifierSpec::Biome;
    assert_eq!(
        biome.apply(
            PlacementContext::plain(&world),
            origin,
            &mut random,
            &mut output,
        ),
        Err(PlacementError::MissingTopFeature)
    );
    assert!(world.take_trace().is_empty());
    biome
        .apply(
            PlacementContext::with_biome_check(&world, &feature),
            origin,
            &mut random,
            &mut output,
        )
        .unwrap();
    assert_eq!(output, [origin]);
    assert_eq!(world.take_trace(), ["biome"]);

    output.clear();
    PlacementModifierSpec::NoiseThresholdCount {
        noise_level: 0.25,
        below_noise: 1,
        above_noise: 2,
    }
    .apply(
        PlacementContext::plain(&world),
        origin,
        &mut random,
        &mut output,
    )
    .unwrap();
    assert_eq!(output, [origin, origin], "noise equality selects above");
    assert_eq!(world.take_trace(), ["noise"]);
    assert!(random.integer_bounds.is_empty());
    assert_eq!(random.float_draws, 0);
}

#[test]
fn predicate_filter_and_environment_scan_preserve_exact_test_order() {
    let world = ModifierWorld::default();
    let origin = BlockPos::new(0, 64, 0);
    let allowed = BlockPredicate::MatchingBlocks {
        offset: PredicateOffset::ZERO,
        blocks: vec![BlockStateId::new(1)],
    };
    let target = BlockPredicate::MatchingBlocks {
        offset: PredicateOffset::ZERO,
        blocks: vec![BlockStateId::new(2)],
    };
    let scan = PlacementModifierSpec::EnvironmentScan {
        direction: VerticalDirection::Up,
        maximum_steps: 4,
        target,
        allowed_search: allowed,
    };
    let mut output = Vec::new();
    let mut random = ScriptedRandom::new([], []);
    scan.apply(
        PlacementContext::plain(&world),
        origin,
        &mut random,
        &mut output,
    )
    .unwrap();
    assert_eq!(output, [BlockPos::new(0, 65, 0)]);
    assert_eq!(
        world.take_trace(),
        ["block:64", "block:64", "block:65", "block:65"]
    );

    output.clear();
    let filter = PlacementModifierSpec::BlockPredicateFilter(BlockPredicate::AlwaysTrue);
    filter
        .apply(
            PlacementContext::plain(&world),
            origin,
            &mut random,
            &mut output,
        )
        .unwrap();
    assert_eq!(output, [origin]);
    assert!(world.take_trace().is_empty());
    assert!(random.integer_bounds.is_empty());
    assert_eq!(random.float_draws, 0);
}

#[test]
fn replace_single_block_rereads_and_stops_after_first_match() {
    let mut world = DirectWorld::fixed(BlockStateId::new(7));
    let miss = MatchRule {
        expected: BlockStateId::new(6),
        draw: true,
    };
    let hit = MatchRule {
        expected: BlockStateId::new(7),
        draw: false,
    };
    let unreachable = MatchRule {
        expected: BlockStateId::new(7),
        draw: false,
    };
    let targets = [
        ReplacementTarget {
            rule: &miss,
            state: BlockStateId::new(10),
        },
        ReplacementTarget {
            rule: &hit,
            state: BlockStateId::new(11),
        },
        ReplacementTarget {
            rule: &unreachable,
            state: BlockStateId::new(12),
        },
    ];
    let origin = BlockPos::new(2, 70, 3);
    let mut random = ScriptedRandom::new([0], []);
    assert!(replace_single_block(
        &mut world,
        origin,
        &mut random,
        &targets,
        |_| true,
    ));
    assert_eq!(world.reads, [origin, origin]);
    assert_eq!(
        world.offers,
        [(origin, BlockStateId::new(11), DIRECT_WRITE_FLAGS)]
    );
    assert_eq!(random.integer_bounds, [1]);

    world.reads.clear();
    world.offers.clear();
    assert!(!replace_single_block(
        &mut world,
        origin,
        &mut random,
        &targets,
        |_| false,
    ));
    assert!(world.reads.is_empty());
    assert!(world.offers.is_empty());
}

#[test]
fn fill_layer_uses_positive_offsets_x_outer_and_ignores_write_results() {
    let origin = BlockPos::new(-3, 999, 5);
    let mut world = DirectWorld::parity_air();
    assert!(fill_layer(&mut world, origin, 4_064, BlockStateId::new(9), |_| true,).unwrap());
    assert_eq!(world.reads.len(), 256);
    assert_eq!(world.reads[0], BlockPos::new(-3, 4_000, 5));
    assert_eq!(world.reads[1], BlockPos::new(-3, 4_000, 6));
    assert_eq!(world.reads[16], BlockPos::new(-2, 4_000, 5));
    assert_eq!(world.reads[255], BlockPos::new(12, 4_000, 20));
    assert_eq!(world.offers.len(), 128);
    assert!(
        world
            .offers
            .iter()
            .all(|(_, state, flags)| *state == BlockStateId::new(9)
                && *flags == DIRECT_WRITE_FLAGS)
    );
    assert!(fill_layer(&mut world, origin, 4_065, BlockStateId::new(9), |_| true,).is_err());
}

#[test]
fn void_start_platform_tiles_the_nine_near_chunks_in_z_x_order() {
    let states = VoidPlatformStates {
        cobblestone: BlockStateId::new(10),
        stone: BlockStateId::new(11),
    };
    let expected = [[64, 128, 72], [128, 256, 144], [72, 144, 81]];
    let mut world = PlatformFixture::new(BlockStateId::new(99));
    for (x_index, chunk_x) in (-1..=1).enumerate() {
        for (z_index, chunk_z) in (-1..=1).enumerate() {
            world.offers.clear();
            let origin = BlockPos::new(chunk_x * 16, 40, chunk_z * 16);
            assert!(place_void_start_platform(&mut world, origin, states, |_| true).unwrap());
            assert_eq!(world.offers.len(), expected[x_index][z_index]);
            assert!(
                world
                    .offers
                    .iter()
                    .all(|(_, _, flags)| *flags == VOID_PLATFORM_WRITE_FLAGS)
            );
        }
    }
    world.offers.clear();
    assert!(
        place_void_start_platform(&mut world, BlockPos::new(32, 40, 0), states, |_| true).unwrap()
    );
    assert!(world.offers.is_empty());
    assert!(world.reads.is_empty());
    assert!(world.destroyed.is_empty());
}

#[test]
fn end_platform_uses_z_x_y_order_and_optional_destroy_before_offer() {
    let states = EndPlatformStates {
        obsidian: BlockStateId::new(20),
        air: BlockStateId::new(21),
    };
    let origin = BlockPos::new(4, 70, -5);
    let mut world = PlatformFixture::new(BlockStateId::new(99));
    assert!(place_end_platform(&mut world, origin, states, |_| true).unwrap());
    assert_eq!(world.reads.len(), 100);
    assert_eq!(world.offers.len(), 100);
    assert!(world.destroyed.is_empty());
    assert_eq!(world.reads[0], BlockPos::new(2, 69, -7));
    assert_eq!(world.reads[1], BlockPos::new(2, 70, -7));
    assert_eq!(world.reads[4], BlockPos::new(3, 69, -7));
    assert_eq!(world.reads[99], BlockPos::new(6, 72, -3));
    assert_eq!(
        world
            .offers
            .iter()
            .filter(|(_, state, _)| *state == states.obsidian)
            .count(),
        25
    );
    assert!(
        world
            .offers
            .iter()
            .all(|(_, _, flags)| *flags == END_PLATFORM_WRITE_FLAGS)
    );

    world.reads.clear();
    world.offers.clear();
    create_end_platform(&mut world, origin, states, true).unwrap();
    assert_eq!(world.destroyed.len(), 100);
    assert_eq!(world.offers.len(), 100);
    assert_eq!(world.destroyed[0], world.offers[0].0);
}

fn plan(target: ChunkStatus, mode: PyramidMode, options: TaskOptions) -> Vec<TaskOperation> {
    StatusTaskPlan::new(target, mode, ChunkStatus::Empty, options).operations
}

#[derive(Debug)]
struct ModifierWorld {
    trace: RefCell<Vec<String>>,
    biome_matches: bool,
    noise: f64,
}

impl Default for ModifierWorld {
    fn default() -> Self {
        Self {
            trace: RefCell::new(Vec::new()),
            biome_matches: false,
            noise: 0.0,
        }
    }
}

impl ModifierWorld {
    fn take_trace(&self) -> Vec<String> {
        self.trace.take()
    }
}

impl PredicateWorld for ModifierWorld {
    fn block_state(&self, position: BlockPos) -> BlockStateId {
        self.trace
            .borrow_mut()
            .push(format!("block:{}", position.y));
        match position.y {
            64 => BlockStateId::new(1),
            65 => BlockStateId::new(2),
            _ => BlockStateId::new(0),
        }
    }

    fn block_in_tag(&self, _state: BlockStateId, _tag: &ResourceId) -> bool {
        false
    }

    fn fluid_matches(&self, _state: BlockStateId, _fluid: &ResourceId) -> bool {
        false
    }

    fn can_replace(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_solid(&self, _state: BlockStateId) -> bool {
        false
    }

    fn would_survive(&self, _state: BlockStateId, _position: BlockPos) -> bool {
        false
    }
}

impl PlacementWorld for ModifierWorld {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn generation_depth(&self) -> i32 {
        384
    }

    fn height(&self, heightmap: GenerationHeightmap, _x: i32, _z: i32) -> i32 {
        match heightmap {
            GenerationHeightmap::OceanFloor => {
                self.trace.borrow_mut().push("height:ocean_floor".into());
                60
            }
            GenerationHeightmap::WorldSurface => {
                self.trace.borrow_mut().push("height:world_surface".into());
                64
            }
            _ => {
                self.trace.borrow_mut().push("height:other".into());
                -64
            }
        }
    }

    fn biome_contains_feature(&self, _position: BlockPos, _feature: &ResourceId) -> bool {
        self.trace.borrow_mut().push("biome".into());
        self.biome_matches
    }

    fn biome_info_noise(&self, _x: f64, _z: f64) -> f64 {
        self.trace.borrow_mut().push("noise".into());
        self.noise
    }
}

#[derive(Debug)]
struct MatchRule {
    expected: BlockStateId,
    draw: bool,
}

impl ReplacementRule<ScriptedRandom> for MatchRule {
    fn test(&self, state: BlockStateId, random: &mut ScriptedRandom) -> bool {
        if self.draw {
            let _ = random.next_u32(NonZeroU32::new(1).unwrap());
        }
        state == self.expected
    }
}

#[derive(Debug, Clone, Copy)]
enum DirectWorldMode {
    Fixed(BlockStateId),
    ParityAir,
}

#[derive(Debug)]
struct DirectWorld {
    mode: DirectWorldMode,
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl DirectWorld {
    fn fixed(state: BlockStateId) -> Self {
        Self {
            mode: DirectWorldMode::Fixed(state),
            reads: Vec::new(),
            offers: Vec::new(),
        }
    }

    fn parity_air() -> Self {
        Self {
            mode: DirectWorldMode::ParityAir,
            reads: Vec::new(),
            offers: Vec::new(),
        }
    }
}

impl DirectWriteWorld for DirectWorld {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        match self.mode {
            DirectWorldMode::Fixed(state) => state,
            DirectWorldMode::ParityAir if (position.x + position.z) & 1 == 0 => {
                BlockStateId::new(0)
            }
            DirectWorldMode::ParityAir => BlockStateId::new(1),
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn offer_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct PlatformFixture {
    state: BlockStateId,
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    destroyed: Vec<BlockPos>,
}

impl PlatformFixture {
    fn new(state: BlockStateId) -> Self {
        Self {
            state,
            reads: Vec::new(),
            offers: Vec::new(),
            destroyed: Vec::new(),
        }
    }
}

impl PlatformWorld for PlatformFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        self.state
    }

    fn offer_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn destroy_block_with_drops(&mut self, position: BlockPos) -> bool {
        self.destroyed.push(position);
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: std::collections::VecDeque<u32>,
    floats: std::collections::VecDeque<f32>,
    integer_bounds: Vec<u32>,
    float_draws: usize,
    gaussian: Option<f64>,
}

impl ScriptedRandom {
    fn new(integers: impl IntoIterator<Item = u32>, floats: impl IntoIterator<Item = f32>) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            floats: floats.into_iter().collect(),
            integer_bounds: Vec::new(),
            float_draws: 0,
            gaussian: None,
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.integer_bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer draw");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        self.floats.pop_front().expect("scripted float draw")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("this fixture does not script double draws")
    }

    fn next_gaussian(&mut self) -> f64 {
        self.gaussian.take().expect("scripted gaussian draw")
    }
}

#[derive(Debug)]
struct OffsetModifier {
    offsets: Vec<(i32, i32)>,
}

impl PlacementModifier<ScriptedRandom> for OffsetModifier {
    fn apply(
        &self,
        input: BlockPos,
        _random: &mut ScriptedRandom,
        output: &mut Vec<BlockPos>,
    ) -> Result<(), PlacementError> {
        output.extend(
            self.offsets
                .iter()
                .map(|(x, z)| BlockPos::new(input.x + x, input.y, input.z + z)),
        );
        Ok(())
    }
}

fn dependencies(center: ChunkPos, target: ChunkStatus) -> BTreeMap<ChunkPos, ChunkStatus> {
    let mut dependencies = BTreeMap::new();
    for radius in 0..=8_i32 {
        let Some(required) = target.direct_requirement(radius as u8) else {
            continue;
        };
        for chunk in square_ring(center, radius) {
            dependencies.insert(chunk, required);
        }
    }
    dependencies
}

fn square_ring(center: ChunkPos, radius: i32) -> Vec<ChunkPos> {
    if radius == 0 {
        return vec![center];
    }
    let mut chunks = Vec::new();
    for x in -radius..=radius {
        chunks.push(ChunkPos::new(center.x + x, center.z - radius));
        chunks.push(ChunkPos::new(center.x + x, center.z + radius));
    }
    for z in (-radius + 1)..radius {
        chunks.push(ChunkPos::new(center.x - radius, center.z + z));
        chunks.push(ChunkPos::new(center.x + radius, center.z + z));
    }
    chunks
}
