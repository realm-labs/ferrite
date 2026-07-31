//! Woodland-mansion template placement, marker effects, and foundation pass.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::mansion_pieces::MansionPieceSpec;
use crate::generation::structure::piece::{FluidState, PieceWorld};
use crate::generation::structure::processor::{
    Processor, ProcessorSettings, SettingsRandom, StructureState,
};
use crate::generation::structure::template::StructureTemplate;
use crate::generation::structure::template_manager::{
    TemplateManager, TemplateManagerError, TemplateSource,
};
use crate::generation::structure::template_place::{
    TemplateMirror, TemplatePlaceSettings, TemplateRotation, TemplateTransform, TemplateWorld,
    data_markers, place_template,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MansionMob {
    Evoker,
    Vindicator,
    Allay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MansionMobSpawn {
    pub mob: MansionMob,
    pub position: BlockPos,
    pub persistent: bool,
    pub finalize_for_local_difficulty: bool,
    pub structure_spawn_reason: bool,
    pub add_with_passengers: bool,
}

pub trait MansionWorld: TemplateWorld {
    /// Returns the world RNG draw used by allay group markers.
    fn mansion_world_random(&mut self, bound: NonZeroU32) -> u32;

    /// Creates, initializes, and offers a mob; false models a null entity factory result.
    fn spawn_mansion_mob(&mut self, request: MansionMobSpawn) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MansionPiece {
    pub template: String,
    pub position: BlockPos,
    pub rotation: Rotation,
    pub mirror: TemplateMirror,
    pub generation_depth: i32,
    pub bounding_box: BlockBox,
    size: [i32; 3],
}

impl MansionPiece {
    fn transform(&self) -> TemplateTransform {
        TemplateTransform {
            origin: self.position,
            pivot: BlockPos::new(0, 0, 0),
            mirror: self.mirror,
            rotation: template_rotation(self.rotation),
        }
    }
}

pub struct MansionRuntime<'a, S> {
    pub templates: &'a mut TemplateManager<S>,
}

impl<S: TemplateSource> MansionRuntime<'_, S> {
    pub fn materialize(
        &mut self,
        specs: &[MansionPieceSpec],
    ) -> Result<Vec<MansionPiece>, MansionError> {
        specs.iter().map(|spec| self.create_piece(spec)).collect()
    }

    pub fn create_piece(&mut self, spec: &MansionPieceSpec) -> Result<MansionPiece, MansionError> {
        let template_name = format!("minecraft:woodland_mansion/{}", spec.template);
        let template = self.templates.require(&template_name)?.template;
        let transform = TemplateTransform {
            origin: spec.position,
            pivot: BlockPos::new(0, 0, 0),
            mirror: spec.mirror,
            rotation: template_rotation(spec.rotation),
        };
        Ok(MansionPiece {
            template: template_name,
            position: spec.position,
            rotation: spec.rotation,
            mirror: spec.mirror,
            generation_depth: 0,
            bounding_box: transformed_box(template.size, transform),
            size: template.size,
        })
    }

    pub fn place<W, R, F>(
        &mut self,
        world: &mut W,
        piece: &mut MansionPiece,
        clip: &BlockBox,
        caller_random: &mut R,
        loot_seed: &mut F,
    ) -> Result<bool, MansionError>
    where
        W: MansionWorld,
        R: GenerationRandom,
        F: FnMut() -> i64,
    {
        let template = self.templates.require(&piece.template)?.template;
        let transform = piece.transform();
        piece.bounding_box = transformed_box(piece.size, transform);
        if !piece.bounding_box.intersects(*clip) {
            return Ok(false);
        }
        let processors = [Processor::BlockIgnore(BTreeSet::from([
            "minecraft:structure_block".to_owned(),
        ]))];
        // Mansion template block entities retain their fixed NBT. Marker chests own the only
        // caller-stream loot seed, so the generic template callback is deliberately inert.
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
                include_entities: false,
                finalize_entities: false,
            },
            caller_random,
            || 0,
        )
        .is_some();
        if placed {
            handle_markers(world, &template, transform, piece.rotation, clip, loot_seed);
        }
        Ok(placed)
    }
}

fn handle_markers(
    world: &mut impl MansionWorld,
    template: &StructureTemplate,
    transform: TemplateTransform,
    rotation: Rotation,
    clip: &BlockBox,
    loot_seed: &mut impl FnMut() -> i64,
) {
    for marker in data_markers(template, 0, transform, clip) {
        if marker.metadata.starts_with("Chest") {
            place_marker_chest(
                world,
                marker.position,
                &marker.metadata,
                rotation,
                clip,
                loot_seed,
            );
            continue;
        }
        let (mob, count) = match marker.metadata.as_str() {
            "Mage" => (MansionMob::Evoker, 1),
            "Warrior" => (MansionMob::Vindicator, 1),
            "Group of Allays" => (
                MansionMob::Allay,
                world.mansion_world_random(NonZeroU32::new(3).expect("three is nonzero")) + 1,
            ),
            _ => continue,
        };
        for _ in 0..count {
            let created = world.spawn_mansion_mob(MansionMobSpawn {
                mob,
                position: marker.position,
                persistent: true,
                finalize_for_local_difficulty: true,
                structure_spawn_reason: true,
                add_with_passengers: true,
            });
            if created {
                PieceWorld::set_state(
                    world,
                    marker.position,
                    StructureState::new("minecraft:air"),
                    2,
                );
            }
        }
    }
}

fn place_marker_chest(
    world: &mut impl MansionWorld,
    position: BlockPos,
    marker: &str,
    rotation: Rotation,
    clip: &BlockBox,
    loot_seed: &mut impl FnMut() -> i64,
) {
    if !clip.contains(position) || PieceWorld::state_at(world, position).block == "minecraft:chest"
    {
        return;
    }
    let local_facing = match marker {
        "ChestWest" => Direction::West,
        "ChestEast" => Direction::East,
        "ChestSouth" => Direction::South,
        _ => Direction::North,
    };
    let facing = rotation.rotate_direction(local_facing);
    let mut chest = StructureState::new("minecraft:chest");
    chest
        .properties
        .insert("facing".into(), facing_name(facing).into());
    PieceWorld::set_state(world, position, chest, 2);
    if PieceWorld::is_loot_container(world, position) {
        PieceWorld::install_loot(
            world,
            position,
            "minecraft:chests/woodland_mansion",
            loot_seed(),
        );
    }
}

pub fn place_foundation(
    world: &mut impl MansionWorld,
    pieces: &[MansionPiece],
    chunk_box: &BlockBox,
    minimum_y: i32,
) {
    let Some(union) = pieces
        .iter()
        .map(|piece| piece.bounding_box)
        .reduce(BlockBox::union)
    else {
        return;
    };
    let y_start = union.minimum.y;
    for x in chunk_box.minimum.x..=chunk_box.maximum.x {
        'column: for z in chunk_box.minimum.z..=chunk_box.maximum.z {
            let seed = BlockPos::new(x, y_start, z);
            if is_air(&PieceWorld::state_at(world, seed).block)
                || !union.contains(seed)
                || !pieces.iter().any(|piece| piece.bounding_box.contains(seed))
            {
                continue;
            }
            for y in (minimum_y + 1..y_start).rev() {
                let position = BlockPos::new(x, y, z);
                let state = PieceWorld::state_at(world, position);
                let fluid = PieceWorld::fluid_at(world, position);
                if !is_air(&state.block) && fluid == FluidState::Empty {
                    continue 'column;
                }
                PieceWorld::set_state(
                    world,
                    position,
                    StructureState::new("minecraft:cobblestone"),
                    2,
                );
            }
        }
    }
}

fn is_air(block: &str) -> bool {
    matches!(
        block,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

fn facing_name(direction: Direction) -> &'static str {
    match direction {
        Direction::North => "north",
        Direction::East => "east",
        Direction::South => "south",
        Direction::West => "west",
        Direction::Up => "up",
        Direction::Down => "down",
    }
}

fn template_rotation(rotation: Rotation) -> TemplateRotation {
    match rotation {
        Rotation::None => TemplateRotation::None,
        Rotation::Clockwise90 => TemplateRotation::Clockwise90,
        Rotation::Clockwise180 => TemplateRotation::Clockwise180,
        Rotation::CounterClockwise90 => TemplateRotation::Counterclockwise90,
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

#[derive(Debug, Error)]
pub enum MansionError {
    #[error(transparent)]
    Template(#[from] TemplateManagerError),
}
