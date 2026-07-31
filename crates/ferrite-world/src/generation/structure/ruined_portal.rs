//! Ruined-portal start selection, center-owned template placement, and apron decoration.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::piece::PieceWorld;
use crate::generation::structure::processor::{
    BlockPredicate, Heightmap, NbtModifier, PositionPredicate, Processor, ProcessorRule,
    ProcessorSettings, ProcessorWorld, SettingsRandom, StructureState,
};
use crate::generation::structure::template::StructureTemplate;
use crate::generation::structure::template_manager::{
    TemplateManager, TemplateManagerError, TemplateSource,
};
use crate::generation::structure::template_place::{
    TemplateMirror, TemplatePlaceSettings, TemplateRotation, TemplateTransform, TemplateWorld,
    place_template,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuinedPortalRecord {
    Standard,
    Desert,
    Jungle,
    Swamp,
    Mountain,
    Ocean,
    Nether,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalPlacement {
    LandSurface,
    PartlyBuried,
    OceanFloor,
    InMountain,
    Underground,
    InNether,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuinedPortalProperties {
    pub air_pocket: bool,
    pub mossiness: f32,
    pub overgrown: bool,
    pub vines: bool,
    pub replace_with_blackstone: bool,
    pub cold: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuinedPortalPiece {
    pub template: String,
    pub position: BlockPos,
    pub rotation: Rotation,
    pub mirror_front_back: bool,
    pub vertical_placement: VerticalPlacement,
    pub properties: RuinedPortalProperties,
    pub bounding_box: BlockBox,
    size: [i32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalFace {
    North,
    East,
    South,
    West,
}

pub trait RuinedPortalGenerationWorld: ProcessorWorld {
    fn minimum_y(&self) -> i32;

    fn sea_level(&self) -> i32;

    fn opaque_in_generator_column(&mut self, heightmap: Heightmap, position: BlockPos) -> bool;

    fn cold_enough_to_snow(&mut self, position: BlockPos, sea_level: i32) -> bool;
}

pub trait RuinedPortalWorld: TemplateWorld {
    fn minimum_y(&self) -> i32;

    fn supports_vine_face(&mut self, position: BlockPos, face: HorizontalFace) -> bool;
}

pub struct RuinedPortalRuntime<'a, S> {
    pub templates: &'a mut TemplateManager<S>,
}

impl<S> RuinedPortalRuntime<'_, S>
where
    S: TemplateSource,
{
    pub fn generate_piece(
        &mut self,
        world: &mut impl RuinedPortalGenerationWorld,
        record: RuinedPortalRecord,
        chunk_minimum: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> Result<RuinedPortalPiece, RuinedPortalError> {
        let setup = select_setup(record, random);
        let air_pocket = chance(random, setup.air_pocket_chance);
        let giant = random.next_f32() < 0.05;
        let count = if giant { 3 } else { 10 };
        let index = bounded(random, count) + 1;
        let template_name = if giant {
            format!("minecraft:ruined_portal/giant_portal_{index}")
        } else {
            format!("minecraft:ruined_portal/portal_{index}")
        };
        let template = self.templates.require(&template_name)?.template;
        let rotation = Rotation::ALL[bounded(random, 4) as usize];
        let mirror_front_back = random.next_f32() >= 0.5;
        let pivot = BlockPos::new(template.size[0] / 2, 0, template.size[2] / 2);
        let base_transform = piece_transform(chunk_minimum, rotation, mirror_front_back, pivot);
        let zero_box = transformed_box(template.size, base_transform);
        let heightmap = if setup.placement == VerticalPlacement::OceanFloor {
            Heightmap::OceanFloorWorldgen
        } else {
            Heightmap::WorldSurfaceWorldgen
        };
        let center = zero_box.center();
        let surface = world.height(heightmap, center.x, center.z).wrapping_sub(1);
        let floor = world.minimum_y().wrapping_add(15);
        let span = template.size[1];
        let projected = projected_y(setup.placement, air_pocket, surface, floor, span, random);
        let y = find_suitable_y(world, zero_box, heightmap, projected, floor);
        let position = BlockPos::new(chunk_minimum.x, y, chunk_minimum.z);
        let cold = setup.can_be_cold && world.cold_enough_to_snow(position, world.sea_level());
        let transform = piece_transform(position, rotation, mirror_front_back, pivot);
        Ok(RuinedPortalPiece {
            template: template_name,
            position,
            rotation,
            mirror_front_back,
            vertical_placement: setup.placement,
            properties: RuinedPortalProperties {
                air_pocket,
                mossiness: setup.mossiness,
                overgrown: setup.overgrown,
                vines: setup.vines,
                replace_with_blackstone: setup.replace_with_blackstone,
                cold,
            },
            bounding_box: transformed_box(template.size, transform),
            size: template.size,
        })
    }

    pub fn place<W, R, F>(
        &mut self,
        world: &mut W,
        piece: &mut RuinedPortalPiece,
        processing_box: &BlockBox,
        caller_random: &mut R,
        loot_seed: &mut F,
    ) -> Result<bool, RuinedPortalError>
    where
        W: RuinedPortalWorld,
        R: GenerationRandom,
        F: FnMut() -> i64,
    {
        let template = self.templates.require(&piece.template)?.template;
        let pivot = BlockPos::new(piece.size[0] / 2, 0, piece.size[2] / 2);
        let transform = piece_transform(
            piece.position,
            piece.rotation,
            piece.mirror_front_back,
            pivot,
        );
        piece.bounding_box = transformed_box(piece.size, transform);
        if !processing_box.contains(piece.bounding_box.center()) {
            return Ok(false);
        }
        let expanded_clip = processing_box.union(piece.bounding_box);
        let processors = portal_processors(piece);
        let placed = place_template(
            world,
            &template,
            TemplatePlaceSettings {
                transform,
                clip: &expanded_clip,
                palette: 0,
                processors: &processors,
                processor_settings: ProcessorSettings {
                    clip: Some(expanded_clip),
                    random: SettingsRandom::PositionDerived,
                    keep_jigsaws: true,
                },
                reference_position: piece.position,
                block_flags: 2,
                keep_liquids: true,
                known_shape: false,
                include_entities: true,
                finalize_entities: true,
            },
            caller_random,
            &mut *loot_seed,
        )
        .is_some();
        if placed {
            place_jigsaw_finals(world, &template, transform);
        }
        spread_netherrack(world, piece, caller_random);
        if piece.properties.vines || piece.properties.overgrown {
            decorate_box(world, piece, caller_random);
        }
        Ok(true)
    }
}

#[derive(Debug, Clone, Copy)]
struct Setup {
    placement: VerticalPlacement,
    air_pocket_chance: f32,
    mossiness: f32,
    overgrown: bool,
    vines: bool,
    can_be_cold: bool,
    replace_with_blackstone: bool,
}

fn select_setup(record: RuinedPortalRecord, random: &mut impl GenerationRandom) -> Setup {
    let (placement, air, moss, overgrown, vines, cold, blackstone) = match record {
        RuinedPortalRecord::Standard => {
            if random.next_f32() < 0.5 {
                (
                    VerticalPlacement::Underground,
                    1.0,
                    0.2,
                    false,
                    false,
                    true,
                    false,
                )
            } else {
                (
                    VerticalPlacement::LandSurface,
                    0.5,
                    0.2,
                    false,
                    false,
                    true,
                    false,
                )
            }
        }
        RuinedPortalRecord::Desert => (
            VerticalPlacement::PartlyBuried,
            0.0,
            0.0,
            false,
            false,
            false,
            false,
        ),
        RuinedPortalRecord::Jungle => (
            VerticalPlacement::LandSurface,
            0.5,
            0.8,
            true,
            true,
            false,
            false,
        ),
        RuinedPortalRecord::Swamp => (
            VerticalPlacement::OceanFloor,
            0.0,
            0.5,
            false,
            true,
            false,
            false,
        ),
        RuinedPortalRecord::Mountain => {
            if random.next_f32() < 0.5 {
                (
                    VerticalPlacement::InMountain,
                    1.0,
                    0.2,
                    false,
                    false,
                    true,
                    false,
                )
            } else {
                (
                    VerticalPlacement::LandSurface,
                    0.5,
                    0.2,
                    false,
                    false,
                    true,
                    false,
                )
            }
        }
        RuinedPortalRecord::Ocean => (
            VerticalPlacement::OceanFloor,
            0.0,
            0.8,
            false,
            false,
            true,
            false,
        ),
        RuinedPortalRecord::Nether => (
            VerticalPlacement::InNether,
            0.5,
            0.0,
            false,
            false,
            false,
            true,
        ),
    };
    Setup {
        placement,
        air_pocket_chance: air,
        mossiness: moss,
        overgrown,
        vines,
        can_be_cold: cold,
        replace_with_blackstone: blackstone,
    }
}

fn chance(random: &mut impl GenerationRandom, probability: f32) -> bool {
    if probability <= 0.0 {
        false
    } else if probability >= 1.0 {
        true
    } else {
        random.next_f32() < probability
    }
}

fn projected_y(
    placement: VerticalPlacement,
    air_pocket: bool,
    surface: i32,
    floor: i32,
    span: i32,
    random: &mut impl GenerationRandom,
) -> i32 {
    let upper = surface.wrapping_sub(span);
    match placement {
        VerticalPlacement::LandSurface | VerticalPlacement::OceanFloor => surface,
        VerticalPlacement::PartlyBuried => upper.wrapping_add(inclusive(random, 2, 8)),
        VerticalPlacement::InMountain => conditional_inclusive(random, 70, upper),
        VerticalPlacement::Underground => conditional_inclusive(random, floor, upper),
        VerticalPlacement::InNether if air_pocket => inclusive(random, 32, 100),
        VerticalPlacement::InNether => {
            if random.next_f32() < 0.5 {
                inclusive(random, 27, 29)
            } else {
                inclusive(random, 29, 100)
            }
        }
    }
}

fn find_suitable_y(
    world: &mut impl RuinedPortalGenerationWorld,
    box_at_zero: BlockBox,
    heightmap: Heightmap,
    mut y: i32,
    floor: i32,
) -> i32 {
    let corners = [
        (box_at_zero.minimum.x, box_at_zero.minimum.z),
        (box_at_zero.maximum.x, box_at_zero.minimum.z),
        (box_at_zero.minimum.x, box_at_zero.maximum.z),
        (box_at_zero.maximum.x, box_at_zero.maximum.z),
    ];
    while y > floor {
        let mut opaque = 0;
        for (x, z) in corners {
            if world.opaque_in_generator_column(heightmap, BlockPos::new(x, y, z)) {
                opaque += 1;
                if opaque == 3 {
                    return y;
                }
            }
        }
        y = y.wrapping_sub(1);
    }
    floor
}

fn portal_processors(piece: &RuinedPortalPiece) -> Vec<Processor> {
    let ignored = if piece.properties.air_pocket {
        BTreeSet::from(["minecraft:structure_block".to_owned()])
    } else {
        BTreeSet::from([
            "minecraft:structure_block".to_owned(),
            "minecraft:air".to_owned(),
        ])
    };
    let mut rules = vec![random_rule("minecraft:gold_block", 0.3, "minecraft:air")];
    if piece.vertical_placement == VerticalPlacement::OceanFloor {
        rules.push(block_rule("minecraft:lava", "minecraft:magma_block"));
    } else if piece.properties.cold {
        rules.push(block_rule("minecraft:lava", "minecraft:netherrack"));
    } else {
        rules.push(random_rule("minecraft:lava", 0.2, "minecraft:magma_block"));
        rules.push(random_rule(
            "minecraft:netherrack",
            0.07,
            "minecraft:magma_block",
        ));
    }
    let mut processors = vec![
        Processor::BlockIgnore(ignored),
        Processor::Rule(rules),
        Processor::BlockAge {
            mossiness: piece.properties.mossiness,
        },
        Processor::ProtectedBlocks(protected_blocks()),
        Processor::LavaSubmerged,
    ];
    if piece.properties.replace_with_blackstone {
        processors.push(Processor::BlackstoneReplace);
    }
    processors
}

fn random_rule(input: &str, probability: f32, output: &str) -> ProcessorRule {
    ProcessorRule {
        input: BlockPredicate::RandomBlock {
            block: input.into(),
            probability,
        },
        location: BlockPredicate::Always,
        position: PositionPredicate::Always,
        output: StructureState::new(output),
        modifier: NbtModifier::Passthrough,
    }
}

fn block_rule(input: &str, output: &str) -> ProcessorRule {
    ProcessorRule {
        input: BlockPredicate::Block(input.into()),
        location: BlockPredicate::Always,
        position: PositionPredicate::Always,
        output: StructureState::new(output),
        modifier: NbtModifier::Passthrough,
    }
}

fn protected_blocks() -> BTreeSet<String> {
    [
        "minecraft:bedrock",
        "minecraft:spawner",
        "minecraft:chest",
        "minecraft:end_portal_frame",
        "minecraft:reinforced_deepslate",
        "minecraft:trial_spawner",
        "minecraft:vault",
    ]
    .map(str::to_owned)
    .into_iter()
    .collect()
}

fn place_jigsaw_finals(
    world: &mut impl RuinedPortalWorld,
    template: &StructureTemplate,
    transform: TemplateTransform,
) {
    let Some(palette) = template.palettes.first() else {
        return;
    };
    for block in &template.blocks {
        if palette
            .states
            .get(block.state_index)
            .is_none_or(|state| state.block != "minecraft:jigsaw")
        {
            continue;
        }
        let final_state = block
            .nbt
            .as_ref()
            .and_then(|nbt| nbt.get("final_state"))
            .and_then(crate::generation::structure::nbt::NbtValue::as_str)
            .unwrap_or("minecraft:air");
        let state = if final_state.starts_with("minecraft:netherrack") {
            StructureState::new("minecraft:netherrack")
        } else {
            StructureState::new("minecraft:air")
        };
        PieceWorld::set_state(world, transform.position(block.position), state, 3);
    }
}

fn spread_netherrack(
    world: &mut impl RuinedPortalWorld,
    piece: &RuinedPortalPiece,
    random: &mut impl GenerationRandom,
) {
    const PROBABILITIES: [f64; 14] = [
        1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.9, 0.9, 0.8, 0.7, 0.6, 0.4, 0.2,
    ];
    let center = piece.bounding_box.center();
    let size = piece.bounding_box.size();
    let average_width = (size[0] + size[2]) / 2;
    let adjustment_bound = (8 - average_width / 2).max(1) as u32;
    let adjustment = bounded(random, adjustment_bound) as i32;
    let protected = protected_blocks();
    for x in center.x.wrapping_sub(14)..=center.x.wrapping_add(14) {
        for z in center.z.wrapping_sub(14)..=center.z.wrapping_add(14) {
            let distance = x.abs_diff(center.x).wrapping_add(z.abs_diff(center.z)) as i32;
            let index = distance.wrapping_add(adjustment);
            let Some(probability) = usize::try_from(index)
                .ok()
                .and_then(|index| PROBABILITIES.get(index))
            else {
                continue;
            };
            if random.next_f64() >= *probability {
                continue;
            }
            let surface = match piece.vertical_placement {
                VerticalPlacement::LandSurface => {
                    ProcessorWorld::height(world, Heightmap::WorldSurfaceWorldgen, x, z)
                        .wrapping_sub(1)
                }
                VerticalPlacement::OceanFloor => {
                    ProcessorWorld::height(world, Heightmap::OceanFloorWorldgen, x, z)
                        .wrapping_sub(1)
                }
                _ => ProcessorWorld::height(world, Heightmap::WorldSurfaceWorldgen, x, z)
                    .wrapping_sub(1)
                    .min(piece.bounding_box.minimum.y),
            };
            let position = BlockPos::new(x, surface, z);
            let live = PieceWorld::state_at(world, position);
            if surface.abs_diff(piece.bounding_box.minimum.y) > 3
                || live.block == "minecraft:air"
                || live.block == "minecraft:obsidian"
                || protected.contains(&live.block)
                || (live.block == "minecraft:lava"
                    && piece.vertical_placement != VerticalPlacement::InNether)
            {
                continue;
            }
            place_spread_state(world, piece, position, random);
            if piece.properties.overgrown && random.next_f32() < 0.5 {
                maybe_add_leaves(world, position);
            }
            add_drip(world, piece, position, random);
        }
    }
    for x in piece.bounding_box.minimum.x.wrapping_add(1)..piece.bounding_box.maximum.x {
        for z in piece.bounding_box.minimum.z.wrapping_add(1)..piece.bounding_box.maximum.z {
            let position = BlockPos::new(x, piece.bounding_box.minimum.y, z);
            if PieceWorld::state_at(world, position).block == "minecraft:netherrack" {
                add_drip(world, piece, position, random);
            }
        }
    }
}

fn add_drip(
    world: &mut impl RuinedPortalWorld,
    piece: &RuinedPortalPiece,
    start: BlockPos,
    random: &mut impl GenerationRandom,
) {
    let mut position = BlockPos::new(start.x, start.y.wrapping_sub(1), start.z);
    place_spread_state(world, piece, position, random);
    for _ in 0..8 {
        if random.next_f32() >= 0.5 {
            break;
        }
        position.y = position.y.wrapping_sub(1);
        place_spread_state(world, piece, position, random);
    }
}

fn place_spread_state(
    world: &mut impl RuinedPortalWorld,
    piece: &RuinedPortalPiece,
    position: BlockPos,
    random: &mut impl GenerationRandom,
) {
    let magma = !piece.properties.cold && random.next_f32() < 0.07;
    let block = if magma {
        "minecraft:magma_block"
    } else {
        "minecraft:netherrack"
    };
    PieceWorld::set_state(world, position, StructureState::new(block), 3);
}

fn decorate_box(
    world: &mut impl RuinedPortalWorld,
    piece: &RuinedPortalPiece,
    random: &mut impl GenerationRandom,
) {
    for z in piece.bounding_box.minimum.z..=piece.bounding_box.maximum.z {
        for y in piece.bounding_box.minimum.y..=piece.bounding_box.maximum.y {
            for x in piece.bounding_box.minimum.x..=piece.bounding_box.maximum.x {
                let position = BlockPos::new(x, y, z);
                if piece.properties.vines {
                    maybe_add_vine(world, position, random);
                }
                if piece.properties.overgrown && random.next_f32() < 0.5 {
                    maybe_add_leaves(world, position);
                }
            }
        }
    }
}

fn maybe_add_vine(
    world: &mut impl RuinedPortalWorld,
    position: BlockPos,
    random: &mut impl GenerationRandom,
) {
    let state = PieceWorld::state_at(world, position);
    if matches!(state.block.as_str(), "minecraft:air" | "minecraft:vine") {
        return;
    }
    let face = [
        HorizontalFace::North,
        HorizontalFace::East,
        HorizontalFace::South,
        HorizontalFace::West,
    ][bounded(random, 4) as usize];
    let neighbor = offset_face(position, face);
    if PieceWorld::state_at(world, neighbor).block != "minecraft:air"
        || !world.supports_vine_face(position, face)
    {
        return;
    }
    let mut vine = StructureState::new("minecraft:vine");
    vine.properties
        .insert(face_name(opposite(face)).into(), "true".into());
    PieceWorld::set_state(world, neighbor, vine, 3);
}

fn maybe_add_leaves(world: &mut impl RuinedPortalWorld, position: BlockPos) {
    if PieceWorld::state_at(world, position).block != "minecraft:netherrack" {
        return;
    }
    let above = BlockPos::new(position.x, position.y.wrapping_add(1), position.z);
    if PieceWorld::state_at(world, above).block == "minecraft:air" {
        let mut leaves = StructureState::new("minecraft:jungle_leaves");
        leaves.properties.insert("persistent".into(), "true".into());
        PieceWorld::set_state(world, above, leaves, 3);
    }
}

fn offset_face(position: BlockPos, face: HorizontalFace) -> BlockPos {
    match face {
        HorizontalFace::North => BlockPos::new(position.x, position.y, position.z - 1),
        HorizontalFace::East => BlockPos::new(position.x + 1, position.y, position.z),
        HorizontalFace::South => BlockPos::new(position.x, position.y, position.z + 1),
        HorizontalFace::West => BlockPos::new(position.x - 1, position.y, position.z),
    }
}

const fn opposite(face: HorizontalFace) -> HorizontalFace {
    match face {
        HorizontalFace::North => HorizontalFace::South,
        HorizontalFace::East => HorizontalFace::West,
        HorizontalFace::South => HorizontalFace::North,
        HorizontalFace::West => HorizontalFace::East,
    }
}

const fn face_name(face: HorizontalFace) -> &'static str {
    match face {
        HorizontalFace::North => "north",
        HorizontalFace::East => "east",
        HorizontalFace::South => "south",
        HorizontalFace::West => "west",
    }
}

fn piece_transform(
    position: BlockPos,
    rotation: Rotation,
    mirror_front_back: bool,
    pivot: BlockPos,
) -> TemplateTransform {
    TemplateTransform {
        origin: position,
        pivot,
        mirror: if mirror_front_back {
            TemplateMirror::FrontBack
        } else {
            TemplateMirror::None
        },
        rotation: match rotation {
            Rotation::None => TemplateRotation::None,
            Rotation::Clockwise90 => TemplateRotation::Clockwise90,
            Rotation::Clockwise180 => TemplateRotation::Clockwise180,
            Rotation::CounterClockwise90 => TemplateRotation::Counterclockwise90,
        },
    }
}

fn transformed_box(size: [i32; 3], transform: TemplateTransform) -> BlockBox {
    let mut bounds = BlockBox::point(transform.position(BlockPos::new(0, 0, 0)));
    for x in [0, size[0] - 1] {
        for y in [0, size[1] - 1] {
            for z in [0, size[2] - 1] {
                bounds = bounds.union(BlockBox::point(transform.position(BlockPos::new(x, y, z))));
            }
        }
    }
    bounds
}

fn conditional_inclusive(random: &mut impl GenerationRandom, minimum: i32, maximum: i32) -> i32 {
    if minimum < maximum {
        inclusive(random, minimum, maximum)
    } else {
        maximum
    }
}

fn inclusive(random: &mut impl GenerationRandom, minimum: i32, maximum: i32) -> i32 {
    minimum + bounded(random, (maximum - minimum + 1) as u32) as i32
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("positive portal bound"))
}

#[derive(Debug, Error)]
pub enum RuinedPortalError {
    #[error(transparent)]
    Template(#[from] TemplateManagerError),
}
