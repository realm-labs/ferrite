use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::nbt::{NbtCompound, NbtValue};
use ferrite_world::generation::structure::processor::{
    Axis, BlockPredicate, Heightmap, LimitProvider, NbtModifier, PositionPredicate,
    ProcessedPalette, Processor, ProcessorRule, ProcessorSettings, ProcessorWorld, SettingsRandom,
    StructureBlock, StructureState, process_blocks,
};

#[test]
fn capped_presence_defers_clipping_for_the_entire_transaction() {
    let blocks = [block(0, "minecraft:stone"), block(10, "minecraft:stone")];
    let settings = settings(Some(BlockBox::point(pos(0))));
    let mut world = TestWorld::default();

    let ordinary = run(
        &mut world,
        &[Processor::NoOp],
        &blocks,
        settings,
        ScriptRandom::default(),
    );
    assert_eq!(ordinary.processed.len(), 1);

    let capped = run(
        &mut world,
        &[Processor::Capped {
            delegate: Box::new(Processor::NoOp),
            limit: LimitProvider::Constant(0),
        }],
        &blocks,
        settings,
        ScriptRandom::default(),
    );
    assert_eq!(capped.raw, blocks);
    assert_eq!(capped.processed, blocks);
}

#[test]
fn block_rot_keeps_threshold_equality_and_nonmembers_draw_nothing() {
    let mut world = TestWorld::default();
    let member = block(0, "minecraft:stone");
    let mut equality = ScriptRandom::floats([0.5]);
    let retained = process_blocks(
        &mut world,
        &[Processor::BlockRot {
            integrity: 0.5,
            rottable: None,
        }],
        std::slice::from_ref(&member),
        pos(0),
        pos(0),
        caller_settings(),
        &mut equality,
    );
    assert_eq!(retained.processed, [member]);
    assert_eq!(equality.float_draws, 1);

    let mut skipped = ScriptRandom::floats([0.9]);
    let untouched = process_blocks(
        &mut world,
        &[Processor::BlockRot {
            integrity: 0.0,
            rottable: Some(BTreeSet::from(["minecraft:dirt".into()])),
        }],
        &[block(0, "minecraft:stone")],
        pos(0),
        pos(0),
        caller_settings(),
        &mut skipped,
    );
    assert_eq!(untouched.processed.len(), 1);
    assert_eq!(skipped.float_draws, 0);
}

#[test]
fn protected_gravity_and_lava_use_live_world_state() {
    let mut world = TestWorld::default();
    world
        .states
        .insert(pos(0), StructureState::new("minecraft:bedrock"));
    let protected = run(
        &mut world,
        &[Processor::ProtectedBlocks(BTreeSet::from([
            "minecraft:bedrock".into(),
        ]))],
        &[block(0, "minecraft:stone")],
        settings(None),
        ScriptRandom::default(),
    );
    assert!(protected.processed.is_empty());

    world.surface = 70;
    let mut elevated = block(2, "minecraft:stone");
    elevated.raw_position.y = 3;
    let gravity = run(
        &mut world,
        &[Processor::Gravity {
            heightmap: Heightmap::WorldSurfaceWorldgen,
            offset: -1,
        }],
        &[elevated],
        settings(None),
        ScriptRandom::default(),
    );
    assert_eq!(gravity.processed[0].position.y, 72);

    world
        .states
        .insert(pos(4), StructureState::new("minecraft:lava"));
    let lava = run(
        &mut world,
        &[Processor::LavaSubmerged],
        &[block(4, "minecraft:oak_fence")],
        settings(None),
        ScriptRandom::default(),
    );
    assert_eq!(lava.processed[0].state.block, "minecraft:lava");
}

#[test]
fn jigsaw_missing_invalid_void_and_valid_final_states_are_distinct() {
    let missing = block(0, "minecraft:jigsaw");
    let mut invalid = block(1, "minecraft:jigsaw");
    invalid.nbt = Some(NbtCompound::from_iter([(
        "final_state".into(),
        NbtValue::String("not valid ]".into()),
    )]));
    let mut void = block(2, "minecraft:jigsaw");
    void.nbt = Some(NbtCompound::from_iter([(
        "final_state".into(),
        NbtValue::String("minecraft:structure_void".into()),
    )]));
    let mut valid = block(3, "minecraft:jigsaw");
    valid.nbt = Some(NbtCompound::from_iter([(
        "final_state".into(),
        NbtValue::String("minecraft:oak_stairs[facing=east,half=top]".into()),
    )]));
    let mut world = TestWorld::default();
    let output = run(
        &mut world,
        &[Processor::JigsawReplacement],
        &[missing.clone(), invalid, void, valid],
        settings(None),
        ScriptRandom::default(),
    );
    assert_eq!(output.processed.len(), 2);
    assert_eq!(output.processed[0], missing);
    assert_eq!(output.processed[1].state.block, "minecraft:oak_stairs");
    assert_eq!(output.processed[1].state.properties["facing"], "east");
    assert!(output.processed[1].nbt.is_none());
}

#[test]
fn blackstone_mapping_only_copies_compatible_properties() {
    let mut source = block(0, "minecraft:mossy_stone_brick_stairs");
    source.state.properties.extend(BTreeMap::from([
        ("facing".into(), "west".into()),
        ("half".into(), "top".into()),
        ("waterlogged".into(), "true".into()),
    ]));
    let mut world = TestWorld::default();
    let output = run(
        &mut world,
        &[Processor::BlackstoneReplace],
        &[source],
        settings(None),
        ScriptRandom::default(),
    );
    assert_eq!(
        output.processed[0].state.block,
        "minecraft:polished_blackstone_brick_stairs"
    );
    assert_eq!(output.processed[0].state.properties.len(), 2);
    assert_eq!(output.processed[0].state.properties["facing"], "west");
}

#[test]
fn rule_short_circuit_uses_strict_random_and_writes_loot_nbt() {
    let position = pos(5);
    let mut world = TestWorld::default();
    world.seeds.insert(position, 0);
    world
        .states
        .insert(position, StructureState::new("minecraft:water"));
    let rules = vec![
        ProcessorRule {
            input: BlockPredicate::Block("minecraft:dirt".into()),
            location: BlockPredicate::RandomBlock {
                block: "minecraft:water".into(),
                probability: 1.0,
            },
            position: PositionPredicate::Always,
            output: StructureState::new("minecraft:gold_block"),
            modifier: NbtModifier::Passthrough,
        },
        ProcessorRule {
            input: BlockPredicate::Block("minecraft:stone".into()),
            location: BlockPredicate::Block("minecraft:water".into()),
            position: PositionPredicate::AxisAlignedLinear {
                axis: Axis::Y,
                minimum_distance: 0,
                maximum_distance: 10,
                minimum_chance: 1.0,
                maximum_chance: 1.0,
            },
            output: StructureState::new("minecraft:chest"),
            modifier: NbtModifier::AppendLoot("minecraft:chests/test".into()),
        },
    ];
    let output = run(
        &mut world,
        &[Processor::Rule(rules)],
        &[block(5, "minecraft:stone")],
        settings(None),
        ScriptRandom::default(),
    );
    let result = &output.processed[0];
    assert_eq!(result.state.block, "minecraft:chest");
    let nbt = result.nbt.as_ref().unwrap();
    assert_eq!(
        nbt["LootTable"],
        NbtValue::String("minecraft:chests/test".into())
    );
    assert!(matches!(nbt["LootTableSeed"], NbtValue::Long(_)));
}

#[test]
fn capped_null_and_equal_candidates_do_not_remove_or_count() {
    let blocks = [
        block(0, "minecraft:stone"),
        block(1, "minecraft:dirt"),
        block(2, "minecraft:stone"),
    ];
    let mut world = TestWorld::default();
    let ignored = run(
        &mut world,
        &[Processor::Capped {
            delegate: Box::new(Processor::BlockIgnore(BTreeSet::from([
                "minecraft:stone".into()
            ]))),
            limit: LimitProvider::Constant(2),
        }],
        &blocks,
        settings(None),
        ScriptRandom::default(),
    );
    assert_eq!(ignored.processed, blocks);

    let changed = run(
        &mut world,
        &[Processor::Capped {
            delegate: Box::new(Processor::Rule(vec![ProcessorRule {
                input: BlockPredicate::Block("minecraft:stone".into()),
                location: BlockPredicate::Always,
                position: PositionPredicate::Always,
                output: StructureState::new("minecraft:diamond_block"),
                modifier: NbtModifier::Passthrough,
            }])),
            limit: LimitProvider::Constant(1),
        }],
        &blocks,
        settings(None),
        ScriptRandom::default(),
    );
    assert_eq!(
        changed
            .processed
            .iter()
            .filter(|cell| cell.state.block == "minecraft:diamond_block")
            .count(),
        1
    );
    assert_eq!(changed.processed.len(), blocks.len());
}

#[test]
fn block_age_consumes_both_eager_stair_candidates() {
    let mut world = TestWorld::default();
    let mut random = ScriptRandom {
        floats: VecDeque::from([0.0, 0.0]),
        integers: VecDeque::from([0, 0, 1, 1, 0]),
        ..ScriptRandom::default()
    };
    let output = process_blocks(
        &mut world,
        &[Processor::BlockAge { mossiness: 1.0 }],
        &[block(0, "minecraft:stone")],
        pos(0),
        pos(0),
        caller_settings(),
        &mut random,
    );
    assert_eq!(
        output.processed[0].state.block,
        "minecraft:mossy_stone_brick_stairs"
    );
    assert_eq!(random.integer_draws, 5);
    assert_eq!(random.float_draws, 2);
}

fn run(
    world: &mut TestWorld,
    processors: &[Processor],
    blocks: &[StructureBlock],
    settings: ProcessorSettings,
    mut random: ScriptRandom,
) -> ProcessedPalette {
    process_blocks(
        world,
        processors,
        blocks,
        pos(0),
        pos(0),
        settings,
        &mut random,
    )
}

fn block(x: i32, name: &str) -> StructureBlock {
    StructureBlock {
        raw_position: pos(x),
        position: pos(x),
        state: StructureState::new(name),
        nbt: None,
    }
}

fn pos(x: i32) -> BlockPos {
    BlockPos { x, y: 0, z: 0 }
}

fn settings(clip: Option<BlockBox>) -> ProcessorSettings {
    ProcessorSettings {
        clip,
        random: SettingsRandom::PositionDerived,
        keep_jigsaws: false,
    }
}

fn caller_settings() -> ProcessorSettings {
    ProcessorSettings {
        random: SettingsRandom::CallerStream,
        ..settings(None)
    }
}

#[derive(Default)]
struct TestWorld {
    states: BTreeMap<BlockPos, StructureState>,
    seeds: BTreeMap<BlockPos, i64>,
    surface: i32,
}

impl ProcessorWorld for TestWorld {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:air"))
    }

    fn height(&mut self, _heightmap: Heightmap, _x: i32, _z: i32) -> i32 {
        self.surface
    }

    fn is_full_collision(&mut self, _position: BlockPos, state: &StructureState) -> bool {
        state.block.ends_with("_block") || state.block == "minecraft:stone"
    }

    fn positional_seed(&self, position: BlockPos) -> i64 {
        self.seeds
            .get(&position)
            .copied()
            .unwrap_or(i64::from(position.x))
    }

    fn capped_seed(&self, template_origin: BlockPos) -> i64 {
        31_i64
            .wrapping_mul(i64::from(template_origin.x))
            .wrapping_add(7)
    }
}

#[derive(Default)]
struct ScriptRandom {
    floats: VecDeque<f32>,
    integers: VecDeque<u32>,
    float_draws: usize,
    integer_draws: usize,
}

impl ScriptRandom {
    fn floats(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            floats: values.into_iter().collect(),
            ..Self::default()
        }
    }
}

impl GenerationRandom for ScriptRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.integer_draws += 1;
        self.integers.pop_front().unwrap_or(0) % bound.get()
    }

    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        self.floats.pop_front().unwrap_or(0.0)
    }

    fn next_f64(&mut self) -> f64 {
        f64::from(self.next_f32())
    }

    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
