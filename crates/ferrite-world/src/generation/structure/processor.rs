//! Ordered structure-template block processing.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};
use crate::generation::structure::BlockBox;
use crate::generation::structure::nbt::{NbtCompound, NbtValue};
use ferrite_foundation::coordinate::BlockPos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureState {
    pub block: String,
    pub properties: BTreeMap<String, String>,
}

impl StructureState {
    pub fn new(block: impl Into<String>) -> Self {
        Self {
            block: block.into(),
            properties: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureBlock {
    pub raw_position: BlockPos,
    pub position: BlockPos,
    pub state: StructureState,
    pub nbt: Option<NbtCompound>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heightmap {
    WorldSurfaceWorldgen,
    OceanFloorWorldgen,
}

pub trait ProcessorWorld {
    fn state_at(&mut self, position: BlockPos) -> StructureState;

    fn height(&mut self, heightmap: Heightmap, x: i32, z: i32) -> i32;

    fn is_full_collision(&mut self, position: BlockPos, state: &StructureState) -> bool;

    fn positional_seed(&self, position: BlockPos) -> i64;

    fn capped_seed(&self, template_origin: BlockPos) -> i64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsRandom {
    PositionDerived,
    CallerStream,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessorSettings {
    pub clip: Option<BlockBox>,
    pub random: SettingsRandom,
    pub keep_jigsaws: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Processor {
    NoOp,
    BlockIgnore(BTreeSet<String>),
    ProtectedBlocks(BTreeSet<String>),
    BlockRot {
        integrity: f32,
        rottable: Option<BTreeSet<String>>,
    },
    Gravity {
        heightmap: Heightmap,
        offset: i32,
    },
    LavaSubmerged,
    JigsawReplacement,
    BlackstoneReplace,
    BlockAge {
        mossiness: f32,
    },
    Rule(Vec<ProcessorRule>),
    Capped {
        delegate: Box<Self>,
        limit: LimitProvider,
    },
}

impl Processor {
    fn processes_whole_piece(&self) -> bool {
        matches!(self, Self::Capped { .. })
    }

    fn process_cell<R: GenerationRandom>(
        &self,
        raw: &StructureBlock,
        current: StructureBlock,
        context: &mut CellContext<'_, R>,
    ) -> Option<StructureBlock> {
        match self {
            Self::NoOp | Self::Capped { .. } => Some(current),
            Self::BlockIgnore(blocks) => {
                (!blocks.contains(&current.state.block)).then_some(current)
            }
            Self::ProtectedBlocks(blocks) => {
                let live = context.world.state_at(current.position);
                (!blocks.contains(&live.block)).then_some(current)
            }
            Self::BlockRot {
                integrity,
                rottable,
            } => {
                if rottable
                    .as_ref()
                    .is_some_and(|blocks| !blocks.contains(&current.state.block))
                {
                    return Some(current);
                }
                let draw = context.settings_float(current.position);
                (draw <= *integrity).then_some(current)
            }
            Self::Gravity { heightmap, offset } => {
                let mut moved = current;
                moved.position.y = context
                    .world
                    .height(*heightmap, moved.position.x, moved.position.z)
                    .wrapping_add(*offset)
                    .wrapping_add(raw.raw_position.y);
                Some(moved)
            }
            Self::LavaSubmerged => {
                let live = context.world.state_at(current.position);
                if live.block == "minecraft:lava"
                    && !context
                        .world
                        .is_full_collision(current.position, &current.state)
                {
                    let mut replaced = current;
                    replaced.state = StructureState::new("minecraft:lava");
                    Some(replaced)
                } else {
                    Some(current)
                }
            }
            Self::JigsawReplacement => replace_jigsaw(current, context.settings.keep_jigsaws),
            Self::BlackstoneReplace => Some(replace_blackstone(current)),
            Self::BlockAge { mossiness } => Some(age_block(current, *mossiness, context)),
            Self::Rule(rules) => apply_rules(rules, current, context),
        }
    }
}

pub struct ProcessedPalette {
    pub raw: Vec<StructureBlock>,
    pub processed: Vec<StructureBlock>,
}

pub fn process_blocks<W, R>(
    world: &mut W,
    processors: &[Processor],
    blocks: &[StructureBlock],
    template_origin: BlockPos,
    reference_position: BlockPos,
    settings: ProcessorSettings,
    caller_random: &mut R,
) -> ProcessedPalette
where
    W: ProcessorWorld,
    R: GenerationRandom,
{
    let defer_clip = processors.iter().any(Processor::processes_whole_piece);
    let mut raw_output = Vec::new();
    let mut processed = Vec::new();
    for raw in blocks {
        if !defer_clip
            && settings
                .clip
                .is_some_and(|clip| !clip.contains(raw.position))
        {
            continue;
        }
        let mut current = Some(raw.clone());
        for processor in processors {
            let Some(cell) = current else {
                break;
            };
            let mut context = CellContext {
                world,
                reference_position,
                settings,
                caller_random,
            };
            current = processor.process_cell(raw, cell, &mut context);
        }
        if let Some(current) = current {
            raw_output.push(raw.clone());
            processed.push(current);
        }
    }
    for processor in processors {
        if let Processor::Capped { delegate, limit } = processor {
            let inputs = FinalizeInputs {
                raw: &raw_output,
                template_origin,
                reference_position,
                settings,
            };
            finalize_capped(
                world,
                delegate,
                *limit,
                inputs,
                &mut processed,
                caller_random,
            );
        }
    }
    ProcessedPalette {
        raw: raw_output,
        processed,
    }
}

struct CellContext<'a, R> {
    world: &'a mut dyn ProcessorWorld,
    reference_position: BlockPos,
    settings: ProcessorSettings,
    caller_random: &'a mut R,
}

impl<R: GenerationRandom> CellContext<'_, R> {
    fn settings_float(&mut self, position: BlockPos) -> f32 {
        match self.settings.random {
            SettingsRandom::PositionDerived => {
                LegacyRandom::new(self.world.positional_seed(position)).next_f32()
            }
            SettingsRandom::CallerStream => self.caller_random.next_f32(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitProvider {
    Constant(u32),
    Uniform { minimum: u32, maximum: u32 },
}

impl LimitProvider {
    fn sample(self, random: &mut impl GenerationRandom) -> Option<u32> {
        match self {
            Self::Constant(value) => Some(value),
            Self::Uniform { minimum, maximum } => {
                let width = maximum.checked_sub(minimum)?.checked_add(1)?;
                let bound = NonZeroU32::new(width)?;
                Some(minimum + random.next_u32(bound))
            }
        }
    }
}

struct FinalizeInputs<'a> {
    raw: &'a [StructureBlock],
    template_origin: BlockPos,
    reference_position: BlockPos,
    settings: ProcessorSettings,
}

fn finalize_capped<R: GenerationRandom>(
    world: &mut dyn ProcessorWorld,
    delegate: &Processor,
    limit: LimitProvider,
    inputs: FinalizeInputs<'_>,
    processed: &mut [StructureBlock],
    caller_random: &mut R,
) {
    if inputs.raw.len() != processed.len() || processed.is_empty() {
        return;
    }
    let mut random = LegacyRandom::new(world.capped_seed(inputs.template_origin));
    let Some(maximum) = limit.sample(&mut random) else {
        return;
    };
    let maximum = maximum.min(processed.len() as u32);
    if maximum == 0 {
        return;
    }
    let mut indices = (0..processed.len()).collect::<Vec<_>>();
    shuffle(&mut indices, &mut random);
    let mut replacements = 0_u32;
    for index in indices {
        let current = processed[index].clone();
        let mut context = CellContext {
            world,
            reference_position: inputs.reference_position,
            settings: inputs.settings,
            caller_random,
        };
        let Some(candidate) =
            delegate.process_cell(&inputs.raw[index], current.clone(), &mut context)
        else {
            continue;
        };
        if candidate == current {
            continue;
        }
        processed[index] = candidate;
        replacements += 1;
        if replacements >= maximum {
            break;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessorRule {
    pub input: BlockPredicate,
    pub location: BlockPredicate,
    pub position: PositionPredicate,
    pub output: StructureState,
    pub modifier: NbtModifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockPredicate {
    Always,
    Block(String),
    State(StructureState),
    RandomBlock {
        block: String,
        probability: f32,
    },
    RandomState {
        state: StructureState,
        probability: f32,
    },
    Tag(BTreeSet<String>),
}

impl BlockPredicate {
    fn test(&self, state: &StructureState, random: &mut impl GenerationRandom) -> bool {
        match self {
            Self::Always => true,
            Self::Block(block) => state.block == *block,
            Self::State(expected) => state == expected,
            Self::RandomBlock { block, probability } => {
                state.block == *block && random.next_f32() < *probability
            }
            Self::RandomState {
                state: expected,
                probability,
            } => state == expected && random.next_f32() < *probability,
            Self::Tag(blocks) => blocks.contains(&state.block),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionPredicate {
    Always,
    Linear {
        minimum_distance: i32,
        maximum_distance: i32,
        minimum_chance: f32,
        maximum_chance: f32,
    },
    AxisAlignedLinear {
        axis: Axis,
        minimum_distance: i32,
        maximum_distance: i32,
        minimum_chance: f32,
        maximum_chance: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl PositionPredicate {
    fn test(
        self,
        position: BlockPos,
        reference: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> bool {
        let (distance, minimum_distance, maximum_distance, minimum_chance, maximum_chance) =
            match self {
                Self::Always => return true,
                Self::Linear {
                    minimum_distance,
                    maximum_distance,
                    minimum_chance,
                    maximum_chance,
                } => (
                    position.x.abs_diff(reference.x)
                        + position.y.abs_diff(reference.y)
                        + position.z.abs_diff(reference.z),
                    minimum_distance,
                    maximum_distance,
                    minimum_chance,
                    maximum_chance,
                ),
                Self::AxisAlignedLinear {
                    axis,
                    minimum_distance,
                    maximum_distance,
                    minimum_chance,
                    maximum_chance,
                } => (
                    match axis {
                        Axis::X => position.x.abs_diff(reference.x),
                        Axis::Y => position.y.abs_diff(reference.y),
                        Axis::Z => position.z.abs_diff(reference.z),
                    },
                    minimum_distance,
                    maximum_distance,
                    minimum_chance,
                    maximum_chance,
                ),
            };
        if minimum_distance >= maximum_distance {
            return false;
        }
        let fraction = ((distance as f32 - minimum_distance as f32)
            / (maximum_distance - minimum_distance) as f32)
            .clamp(0.0, 1.0);
        let chance = minimum_chance + fraction * (maximum_chance - minimum_chance);
        random.next_f32() <= chance
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NbtModifier {
    Passthrough,
    Clear,
    AppendStatic(NbtCompound),
    AppendLoot(String),
}

fn apply_rules<R: GenerationRandom>(
    rules: &[ProcessorRule],
    current: StructureBlock,
    context: &mut CellContext<'_, R>,
) -> Option<StructureBlock> {
    let mut random = LegacyRandom::new(context.world.positional_seed(current.position));
    let live = context.world.state_at(current.position);
    for rule in rules {
        if !rule.input.test(&current.state, &mut random)
            || !rule.location.test(&live, &mut random)
            || !rule
                .position
                .test(current.position, context.reference_position, &mut random)
        {
            continue;
        }
        let mut output = current;
        output.state = rule.output.clone();
        output.nbt = apply_modifier(&rule.modifier, output.nbt, &mut random);
        return Some(output);
    }
    Some(current)
}

fn apply_modifier(
    modifier: &NbtModifier,
    nbt: Option<NbtCompound>,
    random: &mut LegacyRandom,
) -> Option<NbtCompound> {
    match modifier {
        NbtModifier::Passthrough => nbt,
        NbtModifier::Clear => Some(NbtCompound::new()),
        NbtModifier::AppendStatic(additions) => {
            let mut result = nbt.unwrap_or_default();
            result.extend(additions.clone());
            Some(result)
        }
        NbtModifier::AppendLoot(table) => {
            let mut result = nbt.unwrap_or_default();
            result.insert("LootTable".into(), NbtValue::String(table.clone()));
            result.insert("LootTableSeed".into(), NbtValue::Long(random.next_i64()));
            Some(result)
        }
    }
}

fn replace_jigsaw(mut current: StructureBlock, keep_jigsaws: bool) -> Option<StructureBlock> {
    if current.state.block != "minecraft:jigsaw" || keep_jigsaws {
        return Some(current);
    }
    let Some(nbt) = current.nbt.take() else {
        return Some(current);
    };
    let final_state = nbt
        .get("final_state")
        .and_then(NbtValue::as_str)
        .unwrap_or("minecraft:air");
    let state = parse_state(final_state)?;
    if state.block == "minecraft:structure_void" {
        None
    } else {
        current.state = state;
        Some(current)
    }
}

fn parse_state(value: &str) -> Option<StructureState> {
    let (block, properties) = match value.split_once('[') {
        Some((block, suffix)) => {
            let properties = suffix.strip_suffix(']')?;
            (block, Some(properties))
        }
        None => (value, None),
    };
    if block.is_empty()
        || !block.contains(':')
        || block
            .chars()
            .any(|character| character.is_whitespace() || character == ']')
    {
        return None;
    }
    let mut state = StructureState::new(block);
    if let Some(properties) = properties {
        for assignment in properties.split(',').filter(|value| !value.is_empty()) {
            let (name, value) = assignment.split_once('=')?;
            if name.is_empty() || value.is_empty() {
                return None;
            }
            state.properties.insert(name.into(), value.into());
        }
    }
    Some(state)
}

fn replace_blackstone(mut current: StructureBlock) -> StructureBlock {
    let replacement = match current.state.block.as_str() {
        "minecraft:cobblestone" | "minecraft:mossy_cobblestone" => "minecraft:blackstone",
        "minecraft:stone" => "minecraft:polished_blackstone",
        "minecraft:stone_bricks" | "minecraft:mossy_stone_bricks" => {
            "minecraft:polished_blackstone_bricks"
        }
        "minecraft:cobblestone_stairs" | "minecraft:mossy_cobblestone_stairs" => {
            "minecraft:blackstone_stairs"
        }
        "minecraft:stone_stairs"
        | "minecraft:stone_brick_stairs"
        | "minecraft:mossy_stone_brick_stairs" => "minecraft:polished_blackstone_brick_stairs",
        "minecraft:cobblestone_slab" | "minecraft:mossy_cobblestone_slab" => {
            "minecraft:blackstone_slab"
        }
        "minecraft:smooth_stone_slab"
        | "minecraft:stone_slab"
        | "minecraft:stone_brick_slab"
        | "minecraft:mossy_stone_brick_slab" => "minecraft:polished_blackstone_brick_slab",
        "minecraft:stone_brick_wall" | "minecraft:mossy_stone_brick_wall" => {
            "minecraft:polished_blackstone_brick_wall"
        }
        "minecraft:cobblestone_wall" | "minecraft:mossy_cobblestone_wall" => {
            "minecraft:blackstone_wall"
        }
        "minecraft:chiseled_stone_bricks" => "minecraft:chiseled_polished_blackstone",
        "minecraft:cracked_stone_bricks" => "minecraft:cracked_polished_blackstone_bricks",
        "minecraft:iron_bars" => "minecraft:chain",
        _ => return current,
    };
    let properties = ["facing", "half", "type"]
        .into_iter()
        .filter_map(|name| {
            current
                .state
                .properties
                .get(name)
                .map(|value| (name.to_owned(), value.clone()))
        })
        .collect();
    current.state = StructureState {
        block: replacement.into(),
        properties,
    };
    current
}

fn age_block<R: GenerationRandom>(
    current: StructureBlock,
    mossiness: f32,
    context: &mut CellContext<'_, R>,
) -> StructureBlock {
    match context.settings.random {
        SettingsRandom::PositionDerived => {
            let mut random = LegacyRandom::new(context.world.positional_seed(current.position));
            age_block_with_random(current, mossiness, &mut random)
        }
        SettingsRandom::CallerStream => {
            age_block_with_random(current, mossiness, context.caller_random)
        }
    }
}

fn age_block_with_random(
    mut current: StructureBlock,
    mossiness: f32,
    random: &mut impl GenerationRandom,
) -> StructureBlock {
    let block = current.state.block.as_str();
    if matches!(
        block,
        "minecraft:stone_bricks" | "minecraft:stone" | "minecraft:chiseled_stone_bricks"
    ) {
        if random.next_f32() >= 0.5 {
            return current;
        }
        // Java constructs both stair candidates eagerly, so all four orientation
        // draws happen before mossiness and the final brick/stair choice.
        let stone_stair = random_stair("minecraft:stone_brick_stairs", random);
        let mossy_stair = random_stair("minecraft:mossy_stone_brick_stairs", random);
        let mossy = random.next_f32() < mossiness;
        let stair = random.next_u32(NonZeroU32::new(2).expect("two is nonzero")) == 0;
        current.state = if stair {
            if mossy { mossy_stair } else { stone_stair }
        } else if mossy {
            StructureState::new("minecraft:mossy_stone_bricks")
        } else {
            StructureState::new("minecraft:stone_bricks")
        };
    } else if block.ends_with("_stairs") && random.next_f32() < 0.5 {
        current.state.block = if random.next_f32() < mossiness {
            "minecraft:mossy_stone_brick_stairs".into()
        } else {
            "minecraft:stone_brick_slab".into()
        };
    } else if (block.ends_with("_slab") || block.ends_with("_wall"))
        && random.next_f32() < mossiness
    {
        current.state.block = if block.ends_with("_slab") {
            "minecraft:mossy_stone_brick_slab".into()
        } else {
            "minecraft:mossy_stone_brick_wall".into()
        };
    } else if block == "minecraft:obsidian" && random.next_f32() < 0.15 {
        current.state.block = "minecraft:crying_obsidian".into();
    }
    current
}

fn random_stair(block: &str, random: &mut impl GenerationRandom) -> StructureState {
    let facing = match random.next_u32(NonZeroU32::new(4).expect("four is nonzero")) {
        0 => "north",
        1 => "south",
        2 => "west",
        _ => "east",
    };
    let half = if random.next_bool() { "top" } else { "bottom" };
    let mut state = StructureState::new(block);
    state.properties.insert("facing".into(), facing.into());
    state.properties.insert("half".into(), half.into());
    state
}

fn shuffle<T>(values: &mut [T], random: &mut impl GenerationRandom) {
    for index in (1..values.len()).rev() {
        let bound = NonZeroU32::new((index + 1) as u32).expect("shuffle bound is nonzero");
        values.swap(index, random.next_u32(bound) as usize);
    }
}
