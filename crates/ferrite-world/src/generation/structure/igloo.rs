//! Igloo piece inventory and live terrain-relative template placement.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};
use crate::generation::structure::BlockBox;
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::piece::PieceWorld;
use crate::generation::structure::processor::{
    Heightmap, Processor, ProcessorSettings, ProcessorWorld, SettingsRandom, StructureState,
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
pub enum IglooPart {
    Top,
    Middle,
    Bottom,
}

impl IglooPart {
    const fn template(self) -> &'static str {
        match self {
            Self::Top => "minecraft:igloo/top",
            Self::Middle => "minecraft:igloo/middle",
            Self::Bottom => "minecraft:igloo/bottom",
        }
    }

    const fn size(self) -> [i32; 3] {
        match self {
            Self::Top => [7, 5, 8],
            Self::Middle => [3, 3, 3],
            Self::Bottom => [7, 6, 9],
        }
    }

    const fn pivot(self) -> BlockPos {
        match self {
            Self::Top => BlockPos::new(3, 5, 5),
            Self::Middle => BlockPos::new(1, 3, 1),
            Self::Bottom => BlockPos::new(3, 6, 7),
        }
    }

    const fn offset(self) -> BlockPos {
        match self {
            Self::Top => BlockPos::new(0, 0, 0),
            Self::Middle => BlockPos::new(2, -3, 4),
            Self::Bottom => BlockPos::new(0, -3, -2),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IglooPiece {
    pub part: IglooPart,
    pub original_position: BlockPos,
    pub rotation: Rotation,
    pub bounding_box: BlockBox,
}

impl IglooPiece {
    fn new(part: IglooPart, original_position: BlockPos, rotation: Rotation) -> Self {
        let transform = part_transform(part, original_position, rotation, 0);
        Self {
            part,
            original_position,
            rotation,
            bounding_box: transformed_box(part.size(), transform),
        }
    }
}

pub fn generate_igloo_pieces(
    chunk_minimum: BlockPos,
    random: &mut impl GenerationRandom,
) -> Vec<IglooPiece> {
    let rotation = Rotation::ALL[random
        .next_u32(NonZeroU32::new(Rotation::ALL.len() as u32).expect("four rotations"))
        as usize];
    let anchor = BlockPos::new(chunk_minimum.x, 90, chunk_minimum.z);
    let mut pieces = Vec::new();
    if random.next_f64() < 0.5 {
        let depth =
            4 + i32::try_from(random.next_u32(NonZeroU32::new(8).expect("eight basement depths")))
                .expect("bounded depth fits i32");
        pieces.push(IglooPiece::new(
            IglooPart::Bottom,
            BlockPos::new(anchor.x, anchor.y - 3 * depth, anchor.z),
            rotation,
        ));
        for index in 0..depth - 1 {
            pieces.push(IglooPiece::new(
                IglooPart::Middle,
                BlockPos::new(anchor.x, anchor.y - 3 * index, anchor.z),
                rotation,
            ));
        }
    }
    pieces.push(IglooPiece::new(IglooPart::Top, anchor, rotation));
    pieces
}

pub struct IglooPlacementRuntime<'a, S> {
    pub templates: &'a mut TemplateManager<S>,
}

impl<S> IglooPlacementRuntime<'_, S>
where
    S: TemplateSource,
{
    pub fn place<W, R, F>(
        &mut self,
        world: &mut W,
        piece: &IglooPiece,
        clip: &BlockBox,
        caller_random: &mut R,
        loot_seed: &mut F,
    ) -> Result<bool, IglooError>
    where
        W: TemplateWorld,
        R: GenerationRandom,
        F: FnMut() -> i64,
    {
        if !piece.bounding_box.intersects(*clip) {
            return Ok(false);
        }
        let (probe_x, probe_z) = terrain_probe(piece.original_position, piece.rotation);
        let height =
            ProcessorWorld::height(world, Heightmap::WorldSurfaceWorldgen, probe_x, probe_z);
        let vertical_shift = height.wrapping_sub(91);
        let transform = part_transform(
            piece.part,
            piece.original_position,
            piece.rotation,
            vertical_shift,
        );
        let template = self.templates.require(piece.part.template())?.template;
        let mut palette_random = LegacyRandom::new(world.positional_seed(transform.origin));
        palette_random.next_u32(NonZeroU32::new(1).expect("one palette"));
        let processors = [Processor::BlockIgnore(BTreeSet::from([
            "minecraft:structure_block".to_owned(),
        ]))];
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
                reference_position: transform.origin,
                block_flags: 2,
                keep_liquids: false,
                known_shape: false,
                include_entities: true,
                finalize_entities: false,
            },
            caller_random,
            &mut *loot_seed,
        )
        .is_some();
        if placed && piece.part == IglooPart::Bottom {
            handle_bottom_marker(world, &template, transform, clip, loot_seed);
        }
        if piece.part == IglooPart::Top {
            repair_top_support(world, transform);
        }
        Ok(placed)
    }
}

fn handle_bottom_marker(
    world: &mut impl TemplateWorld,
    template: &StructureTemplate,
    transform: TemplateTransform,
    clip: &BlockBox,
    loot_seed: &mut impl FnMut() -> i64,
) {
    for marker in data_markers(template, 0, transform, clip) {
        if marker.metadata != "chest" {
            continue;
        }
        PieceWorld::set_state(
            world,
            marker.position,
            StructureState::new("minecraft:air"),
            3,
        );
        let chest = BlockPos::new(
            marker.position.x,
            marker.position.y.wrapping_sub(1),
            marker.position.z,
        );
        if PieceWorld::is_loot_container(world, chest) {
            PieceWorld::install_loot(world, chest, "minecraft:chests/igloo_chest", loot_seed());
        }
    }
}

fn repair_top_support(world: &mut impl TemplateWorld, transform: TemplateTransform) {
    let candidate = transform.position(BlockPos::new(3, 0, 5));
    let below = BlockPos::new(candidate.x, candidate.y.wrapping_sub(1), candidate.z);
    let state = PieceWorld::state_at(world, below);
    if !matches!(state.block.as_str(), "minecraft:air" | "minecraft:ladder") {
        PieceWorld::set_state(
            world,
            candidate,
            StructureState::new("minecraft:snow_block"),
            3,
        );
    }
}

fn terrain_probe(position: BlockPos, rotation: Rotation) -> (i32, i32) {
    let (x, z) = match rotation {
        Rotation::None => (3, 0),
        Rotation::Clockwise90 => (8, 5),
        Rotation::Clockwise180 => (3, 10),
        Rotation::CounterClockwise90 => (-2, 5),
    };
    (position.x.wrapping_add(x), position.z.wrapping_add(z))
}

fn part_transform(
    part: IglooPart,
    position: BlockPos,
    rotation: Rotation,
    vertical_shift: i32,
) -> TemplateTransform {
    let offset = part.offset();
    TemplateTransform {
        origin: BlockPos::new(
            position.x.wrapping_add(offset.x),
            position
                .y
                .wrapping_add(offset.y)
                .wrapping_add(vertical_shift),
            position.z.wrapping_add(offset.z),
        ),
        pivot: part.pivot(),
        mirror: TemplateMirror::None,
        rotation: template_rotation(rotation),
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
pub enum IglooError {
    #[error(transparent)]
    Template(#[from] TemplateManagerError),
}
