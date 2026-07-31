//! Vanilla tree foliage placers and their shared leaf-admission pipeline.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::provider::{IntProvider, ProviderError};
use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::tree_core::{
    FoliageAttachment, TreeCoreError, TreePlacementContext, TreeWorld,
};
use crate::id::BlockStateId;

#[derive(Debug, Clone, PartialEq)]
pub struct FoliageConfig {
    pub radius: IntProvider,
    pub offset: IntProvider,
    pub kind: FoliageKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FoliageKind {
    Blob {
        height: i32,
    },
    Bush {
        height: i32,
    },
    Fancy {
        height: i32,
    },
    MegaJungle {
        height: i32,
    },
    Pine {
        height: IntProvider,
    },
    Spruce {
        trunk_height: IntProvider,
    },
    Acacia,
    DarkOak,
    Cherry {
        height: IntProvider,
        wide_bottom_layer_hole_chance: f32,
        corner_hole_chance: f32,
        hanging_leaves_chance: f32,
        hanging_leaves_extension_chance: f32,
    },
    MegaPine {
        crown_height: IntProvider,
    },
    RandomSpread {
        foliage_height: IntProvider,
        leaf_placement_attempts: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EncodedCherryChances {
    pub wide_bottom_layer_hole_chance: f32,
    pub corner_hole_chance: f32,
    pub hanging_leaves_chance: f32,
    pub hanging_leaves_extension_chance: f32,
}

impl FoliageKind {
    pub fn encoded_cherry_chances(&self) -> Option<EncodedCherryChances> {
        let Self::Cherry {
            wide_bottom_layer_hole_chance,
            hanging_leaves_chance,
            hanging_leaves_extension_chance,
            ..
        } = self
        else {
            return None;
        };
        Some(EncodedCherryChances {
            wide_bottom_layer_hole_chance: *wide_bottom_layer_hole_chance,
            // Vanilla's codec getter accidentally reads the wide-bottom field here.
            corner_hole_chance: *wide_bottom_layer_hole_chance,
            hanging_leaves_chance: *hanging_leaves_chance,
            hanging_leaves_extension_chance: *hanging_leaves_extension_chance,
        })
    }
}

pub trait FoliageWorld: TreeWorld {
    fn has_persistent_property_set(&self, state: BlockStateId) -> bool;

    fn sample_foliage(
        &mut self,
        position: BlockPos,
        random: &mut impl GenerationRandom,
    ) -> BlockStateId;

    fn supports_waterlogged(&self, state: BlockStateId) -> bool;

    fn is_source_water(&mut self, position: BlockPos) -> bool;

    fn with_waterlogged(&self, state: BlockStateId, waterlogged: bool) -> BlockStateId;
}

impl FoliageConfig {
    pub fn sample_height(
        &self,
        requested_trunk_height: i32,
        random: &mut impl GenerationRandom,
    ) -> Result<i32, FoliageError> {
        let height = match &self.kind {
            FoliageKind::Blob { height }
            | FoliageKind::Bush { height }
            | FoliageKind::Fancy { height }
            | FoliageKind::MegaJungle { height } => {
                require_range(*height, 0, 16)?;
                *height
            }
            FoliageKind::Pine { height } => sample_in(height, random, 0, 24)?,
            FoliageKind::Spruce { trunk_height } => {
                requested_trunk_height - sample_in(trunk_height, random, 0, 24)?
            }
            FoliageKind::Acacia => 0,
            FoliageKind::DarkOak => 4,
            FoliageKind::Cherry { height, .. } => sample_in(height, random, 4, 16)?,
            FoliageKind::MegaPine { crown_height } => sample_in(crown_height, random, 0, 24)?,
            FoliageKind::RandomSpread { foliage_height, .. } => {
                sample_in(foliage_height, random, 1, 512)?
            }
        };
        Ok(match &self.kind {
            FoliageKind::Spruce { .. } => height.max(4),
            _ => height,
        })
    }

    pub fn sample_radius(
        &self,
        requested_trunk_height: i32,
        random: &mut impl GenerationRandom,
    ) -> Result<i32, FoliageError> {
        let radius = sample_in(&self.radius, random, 0, 16)?;
        if matches!(&self.kind, FoliageKind::Pine { .. }) {
            let bound = u32::try_from(requested_trunk_height.saturating_add(1).max(1))
                .map_err(|_| FoliageError::InvalidConfiguration)?;
            Ok(radius + bounded(random, bound)?)
        } else {
            Ok(radius)
        }
    }

    pub fn place<R, W>(
        &self,
        context: &mut TreePlacementContext<'_, W>,
        attachment: FoliageAttachment,
        foliage_height: i32,
        foliage_radius: i32,
        random: &mut R,
    ) -> Result<(), FoliageError>
    where
        R: GenerationRandom,
        W: FoliageWorld,
    {
        self.validate()?;
        let offset = sample_in(&self.offset, random, 0, 16)?;
        let inputs = PlacementInputs {
            attachment,
            foliage_height,
            foliage_radius,
            offset,
        };
        match &self.kind {
            FoliageKind::Blob { .. } => place_blob(context, random, inputs),
            FoliageKind::Bush { .. } => place_bush(context, random, inputs),
            FoliageKind::Fancy { .. } => place_fancy(context, random, inputs),
            FoliageKind::MegaJungle { .. } => place_mega_jungle(context, random, inputs),
            FoliageKind::Pine { .. } => place_pine(context, random, inputs),
            FoliageKind::Spruce { .. } => place_spruce(context, random, inputs),
            FoliageKind::Acacia => place_acacia(context, random, inputs),
            FoliageKind::DarkOak => place_dark_oak(context, random, inputs),
            FoliageKind::Cherry {
                wide_bottom_layer_hole_chance,
                corner_hole_chance,
                hanging_leaves_chance,
                hanging_leaves_extension_chance,
                ..
            } => place_cherry(
                context,
                random,
                inputs,
                CherryChances {
                    wide_bottom: *wide_bottom_layer_hole_chance,
                    corner: *corner_hole_chance,
                    hanging: *hanging_leaves_chance,
                    extension: *hanging_leaves_extension_chance,
                },
            ),
            FoliageKind::MegaPine { .. } => place_mega_pine(context, random, inputs),
            FoliageKind::RandomSpread {
                leaf_placement_attempts,
                ..
            } => place_random_spread(context, random, inputs, *leaf_placement_attempts),
        }
    }

    fn validate(&self) -> Result<(), FoliageError> {
        match &self.kind {
            FoliageKind::Cherry {
                wide_bottom_layer_hole_chance,
                corner_hole_chance,
                hanging_leaves_chance,
                hanging_leaves_extension_chance,
                ..
            } => {
                for chance in [
                    *wide_bottom_layer_hole_chance,
                    *corner_hole_chance,
                    *hanging_leaves_chance,
                    *hanging_leaves_extension_chance,
                ] {
                    if !(0.0..=1.0).contains(&chance) {
                        return Err(FoliageError::InvalidConfiguration);
                    }
                }
            }
            FoliageKind::RandomSpread {
                leaf_placement_attempts,
                ..
            } if *leaf_placement_attempts > 256 => {
                return Err(FoliageError::InvalidConfiguration);
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct PlacementInputs {
    attachment: FoliageAttachment,
    foliage_height: i32,
    foliage_radius: i32,
    offset: i32,
}

#[derive(Debug, Clone, Copy)]
struct CherryChances {
    wide_bottom: f32,
    corner: f32,
    hanging: f32,
    extension: f32,
}

fn place_blob<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
) -> Result<(), FoliageError> {
    rows_descending(inputs.offset, inputs.foliage_height, |y| {
        let radius = (inputs.foliage_radius + inputs.attachment.radius_offset - 1 - y / 2).max(0);
        place_row(
            context,
            random,
            inputs.attachment,
            radius,
            y,
            |random, x, y, z, r, _| {
                x == r && z == r && (y != 0 || bounded(random, 2).is_ok_and(|draw| draw == 0))
            },
        )
    })
}

fn place_bush<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
) -> Result<(), FoliageError> {
    rows_descending(inputs.offset, inputs.foliage_height, |y| {
        let radius = inputs.foliage_radius + inputs.attachment.radius_offset - 1 - y;
        place_row(
            context,
            random,
            inputs.attachment,
            radius,
            y,
            |random, x, _, z, r, _| {
                x == r && z == r && bounded(random, 2).is_ok_and(|draw| draw == 0)
            },
        )
    })
}

fn place_fancy<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
) -> Result<(), FoliageError> {
    rows_descending(inputs.offset, inputs.foliage_height, |y| {
        let radius = inputs.foliage_radius
            + i32::from(y != inputs.offset && y != inputs.offset - inputs.foliage_height);
        place_row(
            context,
            random,
            inputs.attachment,
            radius,
            y,
            |_, x, _, z, r, _| {
                let x = x as f32 + 0.5;
                let z = z as f32 + 0.5;
                x * x + z * z > (r * r) as f32
            },
        )
    })
}

fn place_mega_jungle<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
) -> Result<(), FoliageError> {
    let span = if inputs.attachment.double_trunk {
        inputs.foliage_height
    } else {
        1 + bounded(random, 2)?
    };
    rows_descending(inputs.offset, span, |y| {
        let radius = inputs.foliage_radius + inputs.attachment.radius_offset + 1 - y;
        place_row(context, random, inputs.attachment, radius, y, mega_skip)
    })
}

fn place_pine<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
) -> Result<(), FoliageError> {
    let maximum = inputs.foliage_radius + inputs.attachment.radius_offset;
    let mut radius = 0;
    for y in (inputs.offset - inputs.foliage_height..=inputs.offset).rev() {
        place_row(context, random, inputs.attachment, radius, y, corner_skip)?;
        if radius >= 1 && y == inputs.offset - inputs.foliage_height + 1 {
            radius -= 1;
        } else if radius < maximum {
            radius += 1;
        }
    }
    Ok(())
}

fn place_spruce<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
) -> Result<(), FoliageError> {
    let maximum = inputs.foliage_radius + inputs.attachment.radius_offset;
    let mut radius = bounded(random, 2)?;
    let mut threshold = 1;
    let mut reset = 0;
    for y in (-inputs.foliage_height..=inputs.offset).rev() {
        place_row(context, random, inputs.attachment, radius, y, corner_skip)?;
        if radius >= threshold {
            radius = reset;
            reset = 1;
            threshold = (threshold + 1).min(maximum);
        } else {
            radius += 1;
        }
    }
    Ok(())
}

fn place_acacia<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
) -> Result<(), FoliageError> {
    let attachment = shifted_attachment(inputs.attachment, inputs.offset)?;
    for (radius, y) in [
        (inputs.foliage_radius + inputs.attachment.radius_offset, -1),
        (inputs.foliage_radius - 1, 0),
        (
            inputs.foliage_radius + inputs.attachment.radius_offset - 1,
            0,
        ),
    ] {
        place_row(
            context,
            random,
            attachment,
            radius,
            y,
            |_, x, y, z, r, _| {
                if y != 0 {
                    x == r && z == r && r > 0
                } else {
                    x != 0 && z != 0 && (x != 1 || z != 1)
                }
            },
        )?;
    }
    Ok(())
}

fn place_dark_oak<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
) -> Result<(), FoliageError> {
    let attachment = shifted_attachment(inputs.attachment, inputs.offset)?;
    let rows: &[(i32, i32)] = if attachment.double_trunk {
        &[
            (inputs.foliage_radius + 2, -1),
            (inputs.foliage_radius + 3, 0),
            (inputs.foliage_radius + 2, 1),
        ]
    } else {
        &[
            (inputs.foliage_radius + 2, -1),
            (inputs.foliage_radius + 1, 0),
        ]
    };
    for &(radius, y) in rows {
        place_dark_oak_row(context, random, attachment, radius, y)?;
    }
    if attachment.double_trunk && random.next_bool() {
        place_dark_oak_row(context, random, attachment, inputs.foliage_radius, 2)?;
    }
    Ok(())
}

fn place_cherry<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
    chances: CherryChances,
) -> Result<(), FoliageError> {
    let attachment = shifted_attachment(inputs.attachment, inputs.offset)?;
    let radius = inputs.foliage_radius + inputs.attachment.radius_offset - 1;
    let rows = [
        (radius - 2, inputs.foliage_height - 3),
        (radius - 1, inputs.foliage_height - 4),
    ];
    for (row_radius, y) in rows {
        cherry_row(context, random, attachment, row_radius, y, chances)?;
    }
    for y in (0..=inputs.foliage_height - 5).rev() {
        cherry_row(context, random, attachment, radius, y, chances)?;
    }
    hanging_row(context, random, attachment, radius, -1, chances)?;
    hanging_row(context, random, attachment, radius - 1, -2, chances)
}

fn place_mega_pine<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
) -> Result<(), FoliageError> {
    let mut previous = 0;
    let attachment_y = inputs.attachment.position.y;
    for absolute_y in
        attachment_y - inputs.foliage_height + inputs.offset..=attachment_y + inputs.offset
    {
        let distance = attachment_y - absolute_y;
        let quotient = distance as f32 / inputs.foliage_height as f32;
        let unexpanded = inputs.foliage_radius
            + inputs.attachment.radius_offset
            + (quotient * 3.5).floor() as i32;
        let radius = if distance > 0 && unexpanded == previous && absolute_y % 2 == 0 {
            unexpanded + 1
        } else {
            unexpanded
        };
        place_row_at(
            context,
            random,
            inputs.attachment,
            radius,
            0,
            absolute_y,
            mega_skip,
        )?;
        previous = unexpanded;
    }
    Ok(())
}

fn place_random_spread<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: PlacementInputs,
    attempts: u32,
) -> Result<(), FoliageError> {
    if attempts > 0 && inputs.foliage_radius == 0 {
        return Err(FoliageError::ZeroRandomSpreadRadius);
    }
    for _ in 0..attempts {
        let x = bounded(random, inputs.foliage_radius as u32)?
            - bounded(random, inputs.foliage_radius as u32)?;
        let y = bounded(random, inputs.foliage_height as u32)?
            - bounded(random, inputs.foliage_height as u32)?;
        let z = bounded(random, inputs.foliage_radius as u32)?
            - bounded(random, inputs.foliage_radius as u32)?;
        let position = offset_xyz(inputs.attachment.position, x, y, z)?;
        let _ = try_place_leaf(context, random, position);
    }
    Ok(())
}

fn cherry_row<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    attachment: FoliageAttachment,
    radius: i32,
    y: i32,
    chances: CherryChances,
) -> Result<(), FoliageError> {
    place_row(
        context,
        random,
        attachment,
        radius,
        y,
        |random, x, y, z, r, _| {
            if y == -1 && (x == r || z == r) && random.next_f32() < chances.wide_bottom {
                return true;
            }
            let corner = x == r && z == r;
            if r > 2 {
                corner || x + z > r * 2 - 2 && random.next_f32() < chances.corner
            } else {
                corner && random.next_f32() < chances.corner
            }
        },
    )
}

fn hanging_row<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    attachment: FoliageAttachment,
    radius: i32,
    y: i32,
    chances: CherryChances,
) -> Result<(), FoliageError> {
    cherry_row(context, random, attachment, radius, y, chances)?;
    let positive_extension = i32::from(attachment.double_trunk);
    let log_position = offset_xyz(attachment.position, 0, -1, 0)?;
    for along in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        let edge = clockwise(along);
        let edge_distance = if is_positive(edge) {
            radius + positive_extension
        } else {
            radius
        };
        let mut position = offset_xyz(attachment.position, 0, y - 1, 0)?;
        position = move_by(position, edge, edge_distance)?;
        position = move_by(position, along, -radius)?;
        for _ in -radius..radius + positive_extension {
            let above = move_by(position, Direction::Up, 1)?;
            if context.foliage_attempted(above)
                && try_extension(context, random, log_position, position, chances.hanging)?
            {
                let lower = move_by(position, Direction::Down, 1)?;
                let _ = try_extension(context, random, log_position, lower, chances.extension)?;
            }
            position = move_by(position, along, 1)?;
        }
    }
    Ok(())
}

fn try_extension<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    log_position: BlockPos,
    position: BlockPos,
    chance: f32,
) -> Result<bool, FoliageError> {
    if manhattan(position, log_position) >= 7 || random.next_f32() > chance {
        return Ok(false);
    }
    Ok(try_place_leaf(context, random, position))
}

fn place_row<R, W, S>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    attachment: FoliageAttachment,
    radius: i32,
    y: i32,
    skip: S,
) -> Result<(), FoliageError>
where
    R: GenerationRandom,
    W: FoliageWorld,
    S: FnMut(&mut R, i32, i32, i32, i32, bool) -> bool,
{
    place_row_at(
        context,
        random,
        attachment,
        radius,
        y,
        attachment.position.y,
        skip,
    )
}

fn place_row_at<R, W, S>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    attachment: FoliageAttachment,
    radius: i32,
    relative_y: i32,
    center_y: i32,
    mut skip: S,
) -> Result<(), FoliageError>
where
    R: GenerationRandom,
    W: FoliageWorld,
    S: FnMut(&mut R, i32, i32, i32, i32, bool) -> bool,
{
    if radius < 0 {
        return Ok(());
    }
    let extra = i32::from(attachment.double_trunk);
    for dx in -radius..=radius + extra {
        for dz in -radius..=radius + extra {
            if normalized_skip(
                dx,
                relative_y,
                dz,
                radius,
                attachment.double_trunk,
                &mut skip,
                random,
            ) {
                continue;
            }
            let center = BlockPos::new(attachment.position.x, center_y, attachment.position.z);
            let position = offset_xyz(center, dx, relative_y, dz)?;
            let _ = try_place_leaf(context, random, position);
        }
    }
    Ok(())
}

fn normalized_skip<R, S>(
    dx: i32,
    y: i32,
    dz: i32,
    radius: i32,
    double_trunk: bool,
    skip: &mut S,
    random: &mut R,
) -> bool
where
    R: GenerationRandom,
    S: FnMut(&mut R, i32, i32, i32, i32, bool) -> bool,
{
    let x = if double_trunk {
        dx.abs().min((dx - 1).abs())
    } else {
        dx.abs()
    };
    let z = if double_trunk {
        dz.abs().min((dz - 1).abs())
    } else {
        dz.abs()
    };
    skip(random, x, y, z, radius, double_trunk)
}

fn place_dark_oak_row<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    attachment: FoliageAttachment,
    radius: i32,
    y: i32,
) -> Result<(), FoliageError> {
    if radius < 0 {
        return Ok(());
    }
    let extra = i32::from(attachment.double_trunk);
    for dx in -radius..=radius + extra {
        for dz in -radius..=radius + extra {
            let outside_double_corner = y == 0
                && attachment.double_trunk
                && (dx == -radius || dx >= radius)
                && (dz == -radius || dz >= radius);
            if outside_double_corner
                || normalized_skip(
                    dx,
                    y,
                    dz,
                    radius,
                    attachment.double_trunk,
                    &mut dark_oak_skip,
                    random,
                )
            {
                continue;
            }
            let position = offset_xyz(attachment.position, dx, y, dz)?;
            let _ = try_place_leaf(context, random, position);
        }
    }
    Ok(())
}

fn try_place_leaf<R: GenerationRandom, W: FoliageWorld>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    position: BlockPos,
) -> bool {
    let state = context.world().block_state(position);
    let admitted = {
        let world = context.world();
        !world.has_persistent_property_set(state)
            && (world.is_air(state) || world.is_replaceable_by_trees(state))
    };
    if !admitted {
        return false;
    }
    let mut foliage = context.world().sample_foliage(position, random);
    if context.world().supports_waterlogged(foliage) {
        let waterlogged = context.world().is_source_water(position);
        foliage = context.world().with_waterlogged(foliage, waterlogged);
    }
    context.offer_foliage(position, foliage);
    true
}

fn rows_descending(
    offset: i32,
    height: i32,
    mut row: impl FnMut(i32) -> Result<(), FoliageError>,
) -> Result<(), FoliageError> {
    for y in (offset - height..=offset).rev() {
        row(y)?;
    }
    Ok(())
}

fn corner_skip<R: GenerationRandom>(
    _random: &mut R,
    x: i32,
    _y: i32,
    z: i32,
    radius: i32,
    _double: bool,
) -> bool {
    radius > 0 && x == radius && z == radius
}

fn mega_skip<R: GenerationRandom>(
    _random: &mut R,
    x: i32,
    _y: i32,
    z: i32,
    radius: i32,
    _double: bool,
) -> bool {
    x + z >= 7 || x * x + z * z > radius * radius
}

fn dark_oak_skip<R: GenerationRandom>(
    _random: &mut R,
    x: i32,
    y: i32,
    z: i32,
    radius: i32,
    double: bool,
) -> bool {
    if y == 0 && double {
        return false;
    }
    if y == -1 && !double {
        return x == radius && z == radius;
    }
    y == 1 && x + z > radius * 2 - 2
}

fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        Direction::Up | Direction::Down => unreachable!("only horizontal directions are used"),
    }
}

fn is_positive(direction: Direction) -> bool {
    matches!(direction, Direction::South | Direction::East)
}

fn move_by(
    position: BlockPos,
    direction: Direction,
    distance: i32,
) -> Result<BlockPos, FoliageError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x * distance, y * distance, z * distance)
}

fn shifted_attachment(
    attachment: FoliageAttachment,
    offset: i32,
) -> Result<FoliageAttachment, FoliageError> {
    Ok(FoliageAttachment {
        position: offset_xyz(attachment.position, 0, offset, 0)?,
        ..attachment
    })
}

fn manhattan(left: BlockPos, right: BlockPos) -> i32 {
    left.x.abs_diff(right.x) as i32
        + left.y.abs_diff(right.y) as i32
        + left.z.abs_diff(right.z) as i32
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> Result<i32, FoliageError> {
    let bound = NonZeroU32::new(bound).ok_or(FoliageError::ZeroRandomSpreadRadius)?;
    Ok(random.next_u32(bound) as i32)
}

fn sample_in(
    provider: &IntProvider,
    random: &mut impl GenerationRandom,
    minimum: i32,
    maximum: i32,
) -> Result<i32, FoliageError> {
    let value = provider.sample(random)?;
    require_range(value, minimum, maximum)?;
    Ok(value)
}

fn require_range(value: i32, minimum: i32, maximum: i32) -> Result<(), FoliageError> {
    if (minimum..=maximum).contains(&value) {
        Ok(())
    } else {
        Err(FoliageError::InvalidConfiguration)
    }
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, FoliageError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(FoliageError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(FoliageError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(FoliageError::PositionOverflow)?,
    ))
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FoliageError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error("foliage configuration violates its codec bounds")]
    InvalidConfiguration,
    #[error("random-spread foliage called nextInt(0)")]
    ZeroRandomSpreadRadius,
    #[error("foliage position overflow")]
    PositionOverflow,
}

impl From<FoliageError> for TreeCoreError {
    fn from(error: FoliageError) -> Self {
        match error {
            FoliageError::PositionOverflow => Self::PositionOverflow,
            FoliageError::Provider(_)
            | FoliageError::InvalidConfiguration
            | FoliageError::ZeroRandomSpreadRadius => Self::HeightOverflow,
        }
    }
}
