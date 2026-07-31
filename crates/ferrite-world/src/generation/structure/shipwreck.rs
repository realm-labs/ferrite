//! Shipwreck selection, deferred terrain anchoring, and marker loot placement.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};
use crate::generation::structure::BlockBox;
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::piece::PieceWorld;
use crate::generation::structure::processor::{
    Heightmap, Processor, ProcessorSettings, ProcessorWorld, SettingsRandom,
};
use crate::generation::structure::template_manager::{
    TemplateManager, TemplateManagerError, TemplateSource,
};
use crate::generation::structure::template_place::{
    TemplateMirror, TemplatePlaceSettings, TemplateRotation, TemplateTransform, TemplateWorld,
    data_markers, place_template,
};

const PIVOT: BlockPos = BlockPos::new(4, 0, 15);

const BEACHED_TEMPLATES: [&str; 11] = [
    "with_mast",
    "sideways_full",
    "sideways_fronthalf",
    "sideways_backhalf",
    "rightsideup_full",
    "rightsideup_fronthalf",
    "rightsideup_backhalf",
    "with_mast_degraded",
    "rightsideup_full_degraded",
    "rightsideup_fronthalf_degraded",
    "rightsideup_backhalf_degraded",
];

const OCEAN_TEMPLATES: [&str; 20] = [
    "with_mast",
    "upsidedown_full",
    "upsidedown_fronthalf",
    "upsidedown_backhalf",
    "sideways_full",
    "sideways_fronthalf",
    "sideways_backhalf",
    "rightsideup_full",
    "rightsideup_fronthalf",
    "rightsideup_backhalf",
    "with_mast_degraded",
    "upsidedown_full_degraded",
    "upsidedown_fronthalf_degraded",
    "upsidedown_backhalf_degraded",
    "sideways_full_degraded",
    "sideways_fronthalf_degraded",
    "sideways_backhalf_degraded",
    "rightsideup_full_degraded",
    "rightsideup_fronthalf_degraded",
    "rightsideup_backhalf_degraded",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipwreckPiece {
    pub template: String,
    pub position: BlockPos,
    pub rotation: Rotation,
    pub is_beached: bool,
    pub height_adjusted: bool,
    pub bounding_box: BlockBox,
    size: [i32; 3],
}

impl ShipwreckPiece {
    pub fn is_too_big_for_worldgen_region(&self) -> bool {
        self.size[0] > 32 || self.size[1] > 32
    }

    fn set_height(&mut self, y: i32) {
        self.position.y = y;
        self.height_adjusted = true;
        self.bounding_box = transformed_box(self.size, self.transform());
    }

    fn transform(&self) -> TemplateTransform {
        TemplateTransform {
            origin: self.position,
            pivot: PIVOT,
            mirror: TemplateMirror::None,
            rotation: template_rotation(self.rotation),
        }
    }
}

pub struct ShipwreckRuntime<'a, S> {
    pub templates: &'a mut TemplateManager<S>,
}

impl<S> ShipwreckRuntime<'_, S>
where
    S: TemplateSource,
{
    pub fn generate_piece(
        &mut self,
        chunk_minimum: BlockPos,
        is_beached: bool,
        random: &mut impl GenerationRandom,
    ) -> Result<ShipwreckPiece, ShipwreckError> {
        let rotation =
            Rotation::ALL[random.next_u32(NonZeroU32::new(4).expect("four rotations")) as usize];
        let choices: &[&str] = if is_beached {
            &BEACHED_TEMPLATES
        } else {
            &OCEAN_TEMPLATES
        };
        let choice = choices[random
            .next_u32(NonZeroU32::new(choices.len() as u32).expect("nonempty choices"))
            as usize];
        let template_name = format!("minecraft:shipwreck/{choice}");
        let template = self.templates.require(&template_name)?.template;
        let position = BlockPos::new(chunk_minimum.x, 90, chunk_minimum.z);
        let transform = TemplateTransform {
            origin: position,
            pivot: PIVOT,
            mirror: TemplateMirror::None,
            rotation: template_rotation(rotation),
        };
        Ok(ShipwreckPiece {
            template: template_name,
            position,
            rotation,
            is_beached,
            height_adjusted: false,
            bounding_box: transformed_box(template.size, transform),
            size: template.size,
        })
    }

    pub fn adjust_oversized_at_start(
        &mut self,
        world: &mut impl ProcessorWorld,
        piece: &mut ShipwreckPiece,
        random: &mut impl GenerationRandom,
    ) -> bool {
        if !piece.is_too_big_for_worldgen_region() {
            return false;
        }
        let bounds = piece.bounding_box;
        let spans = bounds.size();
        let samples = [
            (bounds.minimum.x, bounds.minimum.z),
            (bounds.minimum.x, bounds.minimum.z.wrapping_add(spans[2])),
            (bounds.minimum.x.wrapping_add(spans[0]), bounds.minimum.z),
            (
                bounds.minimum.x.wrapping_add(spans[0]),
                bounds.minimum.z.wrapping_add(spans[2]),
            ),
        ];
        let heights = samples.map(|(x, z)| world.height(Heightmap::WorldSurfaceWorldgen, x, z));
        let y = if piece.is_beached {
            heights.into_iter().min().expect("four heights")
                - piece.size[1] / 2
                - random.next_u32(NonZeroU32::new(3).expect("three offsets")) as i32
        } else {
            heights.into_iter().sum::<i32>() / 4
        };
        piece.set_height(y);
        true
    }

    pub fn place<W, R, F>(
        &mut self,
        world: &mut W,
        piece: &mut ShipwreckPiece,
        clip: &BlockBox,
        level_max_y: i32,
        caller_random: &mut R,
        loot_seed: &mut F,
    ) -> Result<bool, ShipwreckError>
    where
        W: TemplateWorld,
        R: GenerationRandom,
        F: FnMut() -> i64,
    {
        if !piece.height_adjusted && !piece.is_too_big_for_worldgen_region() {
            let y = deferred_height(world, piece, level_max_y, caller_random);
            piece.set_height(y);
        }
        let template = self.templates.require(&piece.template)?.template;
        let transform = piece.transform();
        piece.bounding_box = transformed_box(piece.size, transform);
        let mut palette_random = LegacyRandom::new(world.positional_seed(piece.position));
        let palette = palette_random.next_u32(NonZeroU32::new(8).expect("eight palettes")) as usize;
        let processors = [Processor::BlockIgnore(BTreeSet::from([
            "minecraft:structure_block".to_owned(),
            "minecraft:air".to_owned(),
        ]))];
        let placed = place_template(
            world,
            &template,
            TemplatePlaceSettings {
                transform,
                clip,
                palette,
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
            for marker in data_markers(&template, palette, transform, clip) {
                let table = match marker.metadata.as_str() {
                    "map_chest" => "minecraft:chests/shipwreck_map",
                    "supply_chest" => "minecraft:chests/shipwreck_supply",
                    "treasure_chest" => "minecraft:chests/shipwreck_treasure",
                    _ => continue,
                };
                let chest = BlockPos::new(
                    marker.position.x,
                    marker.position.y.wrapping_sub(1),
                    marker.position.z,
                );
                if PieceWorld::is_loot_container(world, chest) {
                    PieceWorld::install_loot(world, chest, table, loot_seed());
                }
            }
        }
        Ok(placed)
    }
}

fn deferred_height(
    world: &mut impl ProcessorWorld,
    piece: &ShipwreckPiece,
    level_max_y: i32,
    random: &mut impl GenerationRandom,
) -> i32 {
    let width = piece.size[0];
    let depth = piece.size[2];
    let heightmap = if piece.is_beached {
        Heightmap::WorldSurfaceWorldgen
    } else {
        Heightmap::OceanFloorWorldgen
    };
    if width <= 0 || depth <= 0 {
        return if piece.is_beached {
            level_max_y.wrapping_add(1)
                - piece.size[1] / 2
                - random.next_u32(NonZeroU32::new(3).expect("three offsets")) as i32
        } else {
            world.height(heightmap, piece.position.x, piece.position.z)
        };
    }
    let mut minimum = level_max_y.wrapping_add(1);
    let mut sum = 0_i64;
    for x in piece.position.x..piece.position.x.wrapping_add(width) {
        for z in piece.position.z..piece.position.z.wrapping_add(depth) {
            let height = world.height(heightmap, x, z);
            minimum = minimum.min(height);
            sum += i64::from(height);
        }
    }
    if piece.is_beached {
        minimum
            - piece.size[1] / 2
            - random.next_u32(NonZeroU32::new(3).expect("three offsets")) as i32
    } else {
        i32::try_from(sum / i64::from(width * depth)).expect("mean of i32 heights fits i32")
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
    if size.into_iter().any(|axis| axis <= 0) {
        return BlockBox::point(transform.origin);
    }
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
pub enum ShipwreckError {
    #[error(transparent)]
    Template(#[from] TemplateManagerError),
}
