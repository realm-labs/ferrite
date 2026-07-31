//! Executable placement dispatch for all five jigsaw pool-element families.

use std::collections::BTreeSet;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::jigsaw::{ElementKind, PoolElement, Projection, Rotation};
use crate::generation::structure::processor::{
    Heightmap, Processor, ProcessorSettings, SettingsRandom,
};
use crate::generation::structure::processor_catalog::ProcessorCatalog;
use crate::generation::structure::template_manager::{
    TemplateManager, TemplateManagerError, TemplateSource,
};
use crate::generation::structure::template_place::{
    TemplateMirror, TemplatePlaceSettings, TemplateRotation, TemplateTransform, TemplateWorld,
    place_template,
};

pub trait PoolElementWorld: TemplateWorld {
    fn place_pool_feature(
        &mut self,
        name: &str,
        position: BlockPos,
        random: &mut dyn GenerationRandom,
    ) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct PoolPlacementSettings<'a> {
    pub origin: BlockPos,
    pub rotation: Rotation,
    pub clip: &'a BlockBox,
    pub reference_position: BlockPos,
    pub keep_jigsaws: bool,
    pub keep_liquids: bool,
}

pub struct PoolPlacementRuntime<'a, S> {
    pub templates: &'a mut TemplateManager<S>,
    pub processors: &'a ProcessorCatalog,
}

impl<S> PoolPlacementRuntime<'_, S>
where
    S: TemplateSource,
{
    pub fn place<W, R, F>(
        &mut self,
        world: &mut W,
        element: &PoolElement,
        settings: PoolPlacementSettings<'_>,
        random: &mut R,
        loot_seed: &mut F,
    ) -> Result<bool, PoolPlacementError>
    where
        W: PoolElementWorld,
        R: GenerationRandom,
        F: FnMut() -> i64,
    {
        match &element.kind {
            ElementKind::Empty => Ok(true),
            ElementKind::Feature { name } => {
                Ok(world.place_pool_feature(name, settings.origin, random))
            }
            ElementKind::Single { .. } => {
                self.place_single(world, element, settings, random, loot_seed)
            }
            ElementKind::List(children) => {
                for child in children {
                    if !self.place(world, child, settings, random, loot_seed)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }

    fn place_single<W, R, F>(
        &mut self,
        world: &mut W,
        element: &PoolElement,
        settings: PoolPlacementSettings<'_>,
        random: &mut R,
        loot_seed: &mut F,
    ) -> Result<bool, PoolPlacementError>
    where
        W: PoolElementWorld,
        R: GenerationRandom,
        F: FnMut() -> i64,
    {
        let ElementKind::Single {
            template: name,
            legacy,
        } = &element.kind
        else {
            unreachable!("single placement is dispatched only for single elements")
        };
        let template = self.templates.get_or_create(name)?.template;
        let processors = self.processor_chain(element, *legacy)?;
        let transform = positive_box_transform(settings.origin, element.size, settings.rotation);
        let result = place_template(
            world,
            &template,
            TemplatePlaceSettings {
                transform,
                clip: settings.clip,
                palette: 0,
                processors: &processors,
                processor_settings: ProcessorSettings {
                    clip: Some(*settings.clip),
                    random: SettingsRandom::PositionDerived,
                    keep_jigsaws: settings.keep_jigsaws,
                },
                reference_position: settings.reference_position,
                block_flags: 18,
                keep_liquids: settings.keep_liquids,
                known_shape: true,
                include_entities: true,
                finalize_entities: true,
            },
            random,
            loot_seed,
        );
        Ok(result.is_some())
    }

    pub fn processor_chain(
        &self,
        element: &PoolElement,
        legacy: bool,
    ) -> Result<Vec<Processor>, PoolPlacementError> {
        let mut processors = vec![Processor::BlockIgnore(BTreeSet::from([
            "minecraft:structure_block".to_owned(),
        ]))];
        processors.push(Processor::JigsawReplacement);
        if let Some(name) = &element.processor_list {
            let configured = self
                .processors
                .get(name)
                .ok_or_else(|| PoolPlacementError::MissingProcessorList(name.clone()))?;
            processors.extend_from_slice(configured);
        }
        if element.projection == Projection::TerrainMatching {
            processors.push(Processor::Gravity {
                heightmap: Heightmap::WorldSurfaceWorldgen,
                offset: -1,
            });
        }
        if legacy {
            processors.push(Processor::BlockIgnore(BTreeSet::from([
                "minecraft:air".to_owned(),
                "minecraft:structure_block".to_owned(),
            ])));
        }
        Ok(processors)
    }
}

pub fn positive_box_transform(
    origin: BlockPos,
    size: [i32; 3],
    rotation: Rotation,
) -> TemplateTransform {
    let (offset_x, offset_z, rotation) = match rotation {
        Rotation::None => (0, 0, TemplateRotation::None),
        Rotation::Clockwise90 => (size[2].wrapping_sub(1), 0, TemplateRotation::Clockwise90),
        Rotation::Clockwise180 => (
            size[0].wrapping_sub(1),
            size[2].wrapping_sub(1),
            TemplateRotation::Clockwise180,
        ),
        Rotation::CounterClockwise90 => (
            0,
            size[0].wrapping_sub(1),
            TemplateRotation::Counterclockwise90,
        ),
    };
    TemplateTransform {
        origin: BlockPos::new(
            origin.x.wrapping_add(offset_x),
            origin.y,
            origin.z.wrapping_add(offset_z),
        ),
        pivot: BlockPos::new(0, 0, 0),
        mirror: TemplateMirror::None,
        rotation,
    }
}

#[derive(Debug, Error)]
pub enum PoolPlacementError {
    #[error(transparent)]
    Template(#[from] TemplateManagerError),
    #[error("pool element references missing processor list {0}")]
    MissingProcessorList(String),
}
