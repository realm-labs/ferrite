//! Ocean-ruin piece selection, live restacking, archaeology, and data markers.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::piece::{FluidState, PieceWorld};
use crate::generation::structure::processor::{
    BlockPredicate, Heightmap, LimitProvider, NbtModifier, PositionPredicate, Processor,
    ProcessorRule, ProcessorSettings, ProcessorWorld, SettingsRandom, StructureState,
};
use crate::generation::structure::template_manager::{
    TemplateManager, TemplateManagerError, TemplateSource,
};
use crate::generation::structure::template_place::{
    TemplateMirror, TemplatePlaceSettings, TemplateRotation, TemplateTransform, TemplateWorld,
    data_markers, place_template,
};

const SMALL_SIZE: [i32; 3] = [6, 7, 7];
const LARGE_SIZE: [i32; 3] = [16, 16, 16];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OceanRuinTemperature {
    Warm,
    Cold,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OceanRuinPiece {
    pub template: String,
    pub position: BlockPos,
    pub rotation: Rotation,
    pub integrity: f32,
    pub large: bool,
    pub temperature: OceanRuinTemperature,
    pub bounding_box: BlockBox,
    size: [i32; 3],
}

impl OceanRuinPiece {
    fn new(
        template: String,
        position: BlockPos,
        rotation: Rotation,
        integrity: f32,
        large: bool,
        temperature: OceanRuinTemperature,
    ) -> Self {
        let size = if large { LARGE_SIZE } else { SMALL_SIZE };
        let transform = transform(position, rotation);
        Self {
            template,
            position,
            rotation,
            integrity,
            large,
            temperature,
            bounding_box: transformed_box(size, transform),
            size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OceanRuinDrownedSpawn {
    pub position: BlockPos,
    pub persistent: bool,
    pub finalize_structure_spawn: bool,
    pub offer_with_passengers: bool,
}

pub trait OceanRuinWorld: TemplateWorld {
    fn minimum_y(&self) -> i32;

    fn sea_level(&self) -> i32;

    /// Runs the complete locked drowned STRUCTURE finalizer and insertion path.
    /// False means entity creation failed, in which case the marker is retained.
    fn spawn_ocean_ruin_drowned(&mut self, request: OceanRuinDrownedSpawn) -> bool;
}

pub fn generate_ocean_ruin_pieces(
    chunk_minimum: BlockPos,
    temperature: OceanRuinTemperature,
    random: &mut impl GenerationRandom,
) -> Vec<OceanRuinPiece> {
    let rotation = random_rotation(random);
    let large = random.next_f32() <= 0.3;
    let integrity = if large { 0.9 } else { 0.8 };
    let position = BlockPos::new(chunk_minimum.x, 90, chunk_minimum.z);
    let mut pieces = add_random_ruin(position, rotation, temperature, large, integrity, random);
    if large && random.next_f32() <= 0.9 {
        add_cluster(&mut pieces, position, rotation, temperature, random);
    }
    pieces
}

pub struct OceanRuinRuntime<'a, S> {
    pub templates: &'a mut TemplateManager<S>,
}

impl<S> OceanRuinRuntime<'_, S>
where
    S: TemplateSource,
{
    pub fn place<W, R, F>(
        &mut self,
        world: &mut W,
        piece: &mut OceanRuinPiece,
        clip: &BlockBox,
        caller_random: &mut R,
        loot_seed: &mut F,
    ) -> Result<bool, OceanRuinError>
    where
        W: OceanRuinWorld,
        R: GenerationRandom,
        F: FnMut() -> i64,
    {
        restack(world, piece);
        let template = self.templates.require(&piece.template)?.template;
        let transform = transform(piece.position, piece.rotation);
        piece.bounding_box = transformed_box(piece.size, transform);
        let archaeology = archaeology_processor(piece.temperature);
        let processors = [
            Processor::BlockRot {
                integrity: piece.integrity,
                rottable: None,
            },
            Processor::BlockIgnore(BTreeSet::from([
                "minecraft:structure_block".to_owned(),
                "minecraft:air".to_owned(),
            ])),
            archaeology,
        ];
        let placed = place_template(
            world,
            &template,
            TemplatePlaceSettings {
                transform,
                clip,
                palette: 0,
                processors: &processors,
                processor_settings: ProcessorSettings {
                    clip: Some(*clip),
                    random: SettingsRandom::PositionDerived,
                    keep_jigsaws: false,
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
            for marker in data_markers(&template, 0, transform, clip) {
                match marker.metadata.as_str() {
                    "chest" => place_chest(world, marker.position, piece.large, loot_seed),
                    "drowned" => place_drowned(world, marker.position),
                    _ => {}
                }
            }
        }
        Ok(placed)
    }
}

fn add_random_ruin(
    position: BlockPos,
    rotation: Rotation,
    temperature: OceanRuinTemperature,
    large: bool,
    integrity: f32,
    random: &mut impl GenerationRandom,
) -> Vec<OceanRuinPiece> {
    match temperature {
        OceanRuinTemperature::Warm => {
            let (prefix, first, count) = if large {
                ("big_warm_", 4, 4)
            } else {
                ("warm_", 1, 8)
            };
            let suffix = first + bounded(random, count);
            vec![OceanRuinPiece::new(
                format!("minecraft:underwater_ruin/{prefix}{suffix}"),
                position,
                rotation,
                integrity,
                large,
                temperature,
            )]
        }
        OceanRuinTemperature::Cold => {
            let suffixes: &[i32] = if large {
                &[1, 2, 3, 8]
            } else {
                &[1, 2, 3, 4, 5, 6, 7, 8]
            };
            let suffix = suffixes[bounded(random, suffixes.len() as u32) as usize];
            [("brick", integrity), ("cracked", 0.7), ("mossy", 0.5)]
                .into_iter()
                .map(|(material, layer_integrity)| {
                    let big = if large { "big_" } else { "" };
                    OceanRuinPiece::new(
                        format!("minecraft:underwater_ruin/{big}{material}_{suffix}"),
                        position,
                        rotation,
                        layer_integrity,
                        large,
                        temperature,
                    )
                })
                .collect()
        }
    }
}

fn add_cluster(
    pieces: &mut Vec<OceanRuinPiece>,
    parent_position: BlockPos,
    parent_rotation: Rotation,
    temperature: OceanRuinTemperature,
    random: &mut impl GenerationRandom,
) {
    let parent_corner =
        transform(parent_position, parent_rotation).position(BlockPos::new(15, 0, 15));
    let origin = BlockPos::new(
        parent_position.x.min(parent_corner.x),
        90,
        parent_position.z.min(parent_corner.z),
    );
    let mut candidates = vec![
        offset_candidate(
            origin,
            -16 + inclusive(random, 1, 8),
            16 + inclusive(random, 1, 7),
        ),
        offset_candidate(
            origin,
            -16 + inclusive(random, 1, 8),
            inclusive(random, 1, 7),
        ),
        offset_candidate(
            origin,
            -16 + inclusive(random, 1, 8),
            -16 + inclusive(random, 4, 8),
        ),
        offset_candidate(
            origin,
            inclusive(random, 1, 7),
            16 + inclusive(random, 1, 7),
        ),
        offset_candidate(
            origin,
            inclusive(random, 1, 7),
            -16 + inclusive(random, 4, 6),
        ),
        offset_candidate(
            origin,
            16 + inclusive(random, 1, 7),
            16 + inclusive(random, 3, 8),
        ),
        offset_candidate(
            origin,
            16 + inclusive(random, 1, 7),
            inclusive(random, 1, 7),
        ),
        offset_candidate(
            origin,
            16 + inclusive(random, 1, 7),
            -16 + inclusive(random, 4, 8),
        ),
    ];
    let attempts = inclusive(random, 4, 8);
    let parent_box = transformed_box(LARGE_SIZE, transform(parent_position, parent_rotation));
    for _ in 0..attempts {
        let selected = bounded(random, candidates.len() as u32) as usize;
        let candidate = candidates.remove(selected);
        let rotation = random_rotation(random);
        let candidate_box = transformed_box(SMALL_SIZE, transform(candidate, rotation));
        if candidate_box.intersects(parent_box) {
            continue;
        }
        pieces.extend(add_random_ruin(
            candidate,
            rotation,
            temperature,
            false,
            0.8,
            random,
        ));
    }
}

fn restack(world: &mut impl OceanRuinWorld, piece: &mut OceanRuinPiece) {
    let sampled_y = ProcessorWorld::height(
        world,
        Heightmap::OceanFloorWorldgen,
        piece.position.x,
        piece.position.z,
    );
    piece.position.y = sampled_y;
    let corner = transform(piece.position, piece.rotation).position(BlockPos::new(
        piece.size[0] - 1,
        0,
        piece.size[2] - 1,
    ));
    let x_min = piece.position.x.min(corner.x);
    let x_max = piece.position.x.max(corner.x);
    let z_min = piece.position.z.min(corner.z);
    let z_max = piece.position.z.max(corner.z);
    let top = sampled_y.wrapping_sub(1);
    let mut minimum = top;
    let mut deep_area = 0_i32;
    for z in z_min..=z_max {
        for x in x_min..=x_max {
            let mut y = top;
            while y > world.minimum_y().wrapping_add(1)
                && replaceable_column(world, BlockPos::new(x, y, z))
            {
                y = y.wrapping_sub(1);
            }
            minimum = minimum.min(y);
            if y < sampled_y.wrapping_sub(3) {
                deep_area += 1;
            }
        }
    }
    let width = piece.position.x.abs_diff(corner.x) as i32;
    if top.wrapping_sub(minimum) > 2 && deep_area > width.wrapping_sub(2) {
        piece.position.y = minimum.wrapping_add(1);
    }
}

fn replaceable_column(world: &mut impl OceanRuinWorld, position: BlockPos) -> bool {
    let state = PieceWorld::state_at(world, position);
    state.block == "minecraft:air"
        || PieceWorld::fluid_at(world, position) == FluidState::Water
        || matches!(
            state.block.as_str(),
            "minecraft:ice"
                | "minecraft:packed_ice"
                | "minecraft:blue_ice"
                | "minecraft:frosted_ice"
        )
}

fn archaeology_processor(temperature: OceanRuinTemperature) -> Processor {
    let (input, output, table) = match temperature {
        OceanRuinTemperature::Warm => (
            "minecraft:sand",
            "minecraft:suspicious_sand",
            "minecraft:archaeology/ocean_ruin_warm",
        ),
        OceanRuinTemperature::Cold => (
            "minecraft:gravel",
            "minecraft:suspicious_gravel",
            "minecraft:archaeology/ocean_ruin_cold",
        ),
    };
    Processor::Capped {
        delegate: Box::new(Processor::Rule(vec![ProcessorRule {
            input: BlockPredicate::Block(input.into()),
            location: BlockPredicate::Always,
            position: PositionPredicate::Always,
            output: StructureState::new(output),
            modifier: NbtModifier::AppendLoot(table.into()),
        }])),
        limit: LimitProvider::Constant(5),
    }
}

fn place_chest(
    world: &mut impl OceanRuinWorld,
    position: BlockPos,
    large: bool,
    loot_seed: &mut impl FnMut() -> i64,
) {
    let waterlogged = PieceWorld::fluid_at(world, position) == FluidState::Water;
    let mut chest = StructureState::new("minecraft:chest");
    chest
        .properties
        .insert("waterlogged".into(), waterlogged.to_string());
    PieceWorld::set_state(world, position, chest, 2);
    if PieceWorld::is_loot_container(world, position) {
        let table = if large {
            "minecraft:chests/underwater_ruin_big"
        } else {
            "minecraft:chests/underwater_ruin_small"
        };
        PieceWorld::install_loot(world, position, table, loot_seed());
    }
}

fn place_drowned(world: &mut impl OceanRuinWorld, position: BlockPos) {
    if !world.spawn_ocean_ruin_drowned(OceanRuinDrownedSpawn {
        position,
        persistent: true,
        finalize_structure_spawn: true,
        offer_with_passengers: true,
    }) {
        return;
    }
    let replacement = if position.y > world.sea_level() {
        StructureState::new("minecraft:air")
    } else {
        StructureState::new("minecraft:water")
    };
    PieceWorld::set_state(world, position, replacement, 2);
}

fn transform(position: BlockPos, rotation: Rotation) -> TemplateTransform {
    TemplateTransform {
        origin: position,
        pivot: BlockPos::new(0, 0, 0),
        mirror: TemplateMirror::None,
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

fn random_rotation(random: &mut impl GenerationRandom) -> Rotation {
    Rotation::ALL[bounded(random, 4) as usize]
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("positive structure bound"))
}

fn inclusive(random: &mut impl GenerationRandom, minimum: i32, maximum: i32) -> i32 {
    minimum + bounded(random, (maximum - minimum + 1) as u32) as i32
}

fn offset_candidate(origin: BlockPos, x: i32, z: i32) -> BlockPos {
    BlockPos::new(origin.x.wrapping_add(x), 90, origin.z.wrapping_add(z))
}

#[derive(Debug, Error)]
pub enum OceanRuinError {
    #[error(transparent)]
    Template(#[from] TemplateManagerError),
}
