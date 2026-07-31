//! End-city template pieces and marker transactions.

use std::collections::BTreeSet;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::piece::PieceWorld;
use crate::generation::structure::processor::{Processor, ProcessorSettings, SettingsRandom};
use crate::generation::structure::template_manager::{
    TemplateManager, TemplateManagerError, TemplateSource,
};
use crate::generation::structure::template_place::{
    TemplateMirror, TemplatePlaceSettings, TemplateRotation, TemplateTransform, TemplateWorld,
    data_markers, place_template,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndCityFrameFacing {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndCityShulkerSpawn {
    pub position: [f64; 3],
    pub structure_creation: bool,
    pub finalize_spawn: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndCityElytraFrameSpawn {
    pub position: BlockPos,
    pub facing: EndCityFrameFacing,
    pub item: &'static str,
    pub play_item_sound: bool,
}

pub trait EndCityWorld: TemplateWorld {
    fn is_spawnable_bounds(&self, position: BlockPos) -> bool;

    fn spawn_end_city_shulker(&mut self, request: EndCityShulkerSpawn);

    fn spawn_end_city_elytra_frame(&mut self, request: EndCityElytraFrameSpawn);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndCityPiece {
    pub template: String,
    pub position: BlockPos,
    pub rotation: Rotation,
    pub overwrite: bool,
    pub generation_depth: i32,
    pub bounding_box: BlockBox,
    size: [i32; 3],
}

impl EndCityPiece {
    fn transform(&self) -> TemplateTransform {
        TemplateTransform {
            origin: self.position,
            pivot: BlockPos::new(0, 0, 0),
            mirror: TemplateMirror::None,
            rotation: template_rotation(self.rotation),
        }
    }
}

pub struct EndCityRuntime<'a, S> {
    pub templates: &'a mut TemplateManager<S>,
}

impl<S> EndCityRuntime<'_, S>
where
    S: TemplateSource,
{
    pub fn create_piece(
        &mut self,
        name: &str,
        position: BlockPos,
        rotation: Rotation,
        overwrite: bool,
    ) -> Result<EndCityPiece, EndCityError> {
        let template_name = format!("minecraft:end_city/{name}");
        let template = self.templates.require(&template_name)?.template;
        let transform = TemplateTransform {
            origin: position,
            pivot: BlockPos::new(0, 0, 0),
            mirror: TemplateMirror::None,
            rotation: template_rotation(rotation),
        };
        Ok(EndCityPiece {
            template: template_name,
            position,
            rotation,
            overwrite,
            generation_depth: 0,
            bounding_box: transformed_box(template.size, transform),
            size: template.size,
        })
    }

    pub fn connect_piece(
        &mut self,
        parent: &EndCityPiece,
        name: &str,
        offset: BlockPos,
        rotation: Rotation,
        overwrite: bool,
    ) -> Result<EndCityPiece, EndCityError> {
        let rotated = rotate_offset(offset, parent.rotation);
        self.create_piece(
            name,
            BlockPos::new(
                parent.position.x.wrapping_add(rotated.x),
                parent.position.y.wrapping_add(rotated.y),
                parent.position.z.wrapping_add(rotated.z),
            ),
            rotation,
            overwrite,
        )
    }

    pub fn place<W, R, F>(
        &mut self,
        world: &mut W,
        piece: &mut EndCityPiece,
        clip: &BlockBox,
        caller_random: &mut R,
        loot_seed: &mut F,
    ) -> Result<bool, EndCityError>
    where
        W: EndCityWorld,
        R: GenerationRandom,
        F: FnMut() -> i64,
    {
        let template = self.templates.require(&piece.template)?.template;
        let transform = piece.transform();
        piece.bounding_box = transformed_box(piece.size, transform);
        if !piece.bounding_box.intersects(*clip) {
            return Ok(false);
        }
        let ignored = if piece.overwrite {
            BTreeSet::from(["minecraft:structure_block".to_owned()])
        } else {
            BTreeSet::from([
                "minecraft:structure_block".to_owned(),
                "minecraft:air".to_owned(),
            ])
        };
        let processors = [Processor::BlockIgnore(ignored)];
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
            &mut *loot_seed,
        )
        .is_some();
        if placed {
            handle_markers(world, &template, transform, piece.rotation, clip, loot_seed);
        }
        Ok(placed)
    }
}

fn handle_markers(
    world: &mut impl EndCityWorld,
    template: &crate::generation::structure::template::StructureTemplate,
    transform: TemplateTransform,
    rotation: Rotation,
    clip: &BlockBox,
    loot_seed: &mut impl FnMut() -> i64,
) {
    for marker in data_markers(template, 0, transform, clip) {
        if marker.metadata.starts_with("Chest") {
            let chest = BlockPos::new(
                marker.position.x,
                marker.position.y.wrapping_sub(1),
                marker.position.z,
            );
            if clip.contains(chest) && PieceWorld::is_loot_container(world, chest) {
                PieceWorld::install_loot(
                    world,
                    chest,
                    "minecraft:chests/end_city_treasure",
                    loot_seed(),
                );
            }
        } else if marker.metadata.starts_with("Sentry") {
            if world.is_spawnable_bounds(marker.position) {
                world.spawn_end_city_shulker(EndCityShulkerSpawn {
                    position: [
                        f64::from(marker.position.x) + 0.5,
                        f64::from(marker.position.y),
                        f64::from(marker.position.z) + 0.5,
                    ],
                    structure_creation: true,
                    finalize_spawn: false,
                });
            }
        } else if marker.metadata.starts_with("Elytra") {
            world.spawn_end_city_elytra_frame(EndCityElytraFrameSpawn {
                position: marker.position,
                facing: rotate_south(rotation),
                item: "minecraft:elytra",
                play_item_sound: false,
            });
        }
    }
}

fn rotate_south(rotation: Rotation) -> EndCityFrameFacing {
    match rotation {
        Rotation::None => EndCityFrameFacing::South,
        Rotation::Clockwise90 => EndCityFrameFacing::West,
        Rotation::Clockwise180 => EndCityFrameFacing::North,
        Rotation::CounterClockwise90 => EndCityFrameFacing::East,
    }
}

fn rotate_offset(offset: BlockPos, rotation: Rotation) -> BlockPos {
    match rotation {
        Rotation::None => offset,
        Rotation::Clockwise90 => BlockPos::new(-offset.z, offset.y, offset.x),
        Rotation::Clockwise180 => BlockPos::new(-offset.x, offset.y, -offset.z),
        Rotation::CounterClockwise90 => BlockPos::new(offset.z, offset.y, -offset.x),
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
pub enum EndCityError {
    #[error(transparent)]
    Template(#[from] TemplateManagerError),
}
