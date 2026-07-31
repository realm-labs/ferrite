//! Dark-oak, fancy, and cherry trunk placers.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::{Axis, Direction};

use crate::generation::feature::provider::IntProvider;
use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::tree_core::{
    FoliageAttachment, TreeCoreError, TreePlacementContext,
};
use crate::generation::feature::tree_trunk::TrunkWorld;

pub fn place_dark_oak_trunk<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    height: i32,
    origin: BlockPos,
) -> Result<Vec<FoliageAttachment>, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let below = offset(origin, Direction::Down)?;
    for (x, z) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        place_below(context, random, offset_xyz(below, x, 0, z)?)?;
    }
    let bend_direction = horizontal_direction(random);
    let bend_start = height - bounded(random, 4);
    let mut bend_steps = 2 - bounded(random, 3);
    let mut x = origin.x;
    let mut z = origin.z;
    let attachment_y = origin
        .y
        .checked_add(height - 1)
        .ok_or(TreeCoreError::PositionOverflow)?;
    for y in 0..height {
        if y >= bend_start && bend_steps > 0 {
            let [dx, _, dz] = bend_direction.step();
            x = x.checked_add(dx).ok_or(TreeCoreError::PositionOverflow)?;
            z = z.checked_add(dz).ok_or(TreeCoreError::PositionOverflow)?;
            bend_steps -= 1;
        }
        let northwest = BlockPos::new(
            x,
            origin
                .y
                .checked_add(y)
                .ok_or(TreeCoreError::PositionOverflow)?,
            z,
        );
        let state = context.world().block_state(northwest);
        let air_or_leaves = {
            let world = context.world();
            world.is_air(state) || world.is_leaves(state)
        };
        if !air_or_leaves {
            continue;
        }
        for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let _ = place_log(context, random, offset_xyz(northwest, dx, 0, dz)?, None);
        }
    }
    let mut attachments = vec![FoliageAttachment {
        position: BlockPos::new(x, attachment_y, z),
        radius_offset: 0,
        double_trunk: true,
    }];
    for offset_x in -1..=2 {
        for offset_z in -1..=2 {
            if (0..=1).contains(&offset_x) && (0..=1).contains(&offset_z) {
                continue;
            }
            if bounded(random, 3) > 0 {
                continue;
            }
            let length = 2 + bounded(random, 3);
            for branch_y in 0..length {
                let position = BlockPos::new(
                    origin
                        .x
                        .checked_add(offset_x)
                        .ok_or(TreeCoreError::PositionOverflow)?,
                    attachment_y
                        .checked_sub(branch_y + 1)
                        .ok_or(TreeCoreError::PositionOverflow)?,
                    origin
                        .z
                        .checked_add(offset_z)
                        .ok_or(TreeCoreError::PositionOverflow)?,
                );
                let _ = place_log(context, random, position, None);
            }
            attachments.push(FoliageAttachment {
                position: BlockPos::new(origin.x + offset_x, attachment_y, origin.z + offset_z),
                radius_offset: 0,
                double_trunk: false,
            });
        }
    }
    Ok(attachments)
}

#[derive(Debug, Clone, Copy)]
struct FancyFoliageCoordinate {
    attachment: FoliageAttachment,
    branch_base_y: i32,
}

pub fn place_fancy_trunk<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    requested_height: i32,
    origin: BlockPos,
) -> Result<Vec<FoliageAttachment>, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let height = requested_height
        .checked_add(2)
        .ok_or(TreeCoreError::HeightOverflow)?;
    let trunk_height = (f64::from(height) * 0.618).floor() as i32;
    place_below(context, random, offset(origin, Direction::Down)?)?;
    let trunk_top = origin
        .y
        .checked_add(trunk_height)
        .ok_or(TreeCoreError::PositionOverflow)?;
    let initial_y = height - 5;
    let mut foliage = vec![FancyFoliageCoordinate {
        attachment: attachment(offset_xyz(origin, 0, initial_y, 0)?),
        branch_base_y: trunk_top,
    }];
    let clusters_per_y = 1_i32.min((1.382 + (f64::from(height) / 13.0).powi(2)).floor() as i32);
    for relative_y in (0..=initial_y).rev() {
        let shape = tree_shape(height, relative_y);
        if shape < 0.0 {
            continue;
        }
        for _ in 0..clusters_per_y {
            let radius = f64::from(shape) * (f64::from(random.next_f32()) + 0.328);
            let angle = f64::from(random.next_f32() * 2.0) * std::f64::consts::PI;
            let x = (radius * angle.sin() + 0.5).floor() as i32;
            let z = (radius * angle.cos() + 0.5).floor() as i32;
            let candidate = offset_xyz(origin, x, relative_y - 1, z)?;
            let candidate_top = offset_xyz(candidate, 0, 5, 0)?;
            if !make_fancy_limb(context, random, candidate, candidate_top, false)? {
                continue;
            }
            let dx = origin.x - candidate.x;
            let dz = origin.z - candidate.z;
            let branch_height =
                f64::from(candidate.y) - f64::from(dx * dx + dz * dz).sqrt() * 0.381;
            let branch_y = if branch_height > f64::from(trunk_top) {
                trunk_top
            } else {
                branch_height as i32
            };
            let branch_base = BlockPos::new(origin.x, branch_y, origin.z);
            if make_fancy_limb(context, random, branch_base, candidate, false)? {
                foliage.push(FancyFoliageCoordinate {
                    attachment: attachment(candidate),
                    branch_base_y: branch_y,
                });
            }
        }
    }
    let trunk_end = offset_xyz(origin, 0, trunk_height, 0)?;
    let _ = make_fancy_limb(context, random, origin, trunk_end, true)?;
    for coordinate in foliage.iter().copied() {
        let branch_base = BlockPos::new(origin.x, coordinate.branch_base_y, origin.z);
        if branch_base != coordinate.attachment.position
            && trim_fancy_branch(height, coordinate.branch_base_y - origin.y)
        {
            let _ = make_fancy_limb(
                context,
                random,
                branch_base,
                coordinate.attachment.position,
                true,
            )?;
        }
    }
    Ok(foliage
        .into_iter()
        .filter(|coordinate| trim_fancy_branch(height, coordinate.branch_base_y - origin.y))
        .map(|coordinate| coordinate.attachment)
        .collect())
}

#[derive(Debug, Clone, PartialEq)]
pub struct CherryTrunkConfig {
    pub branch_count: IntProvider,
    pub branch_horizontal_length: IntProvider,
    pub branch_start_minimum: i32,
    pub branch_start_maximum: i32,
    pub branch_end_offset_from_top: IntProvider,
}

pub fn place_cherry_trunk<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    height: i32,
    origin: BlockPos,
    config: &CherryTrunkConfig,
) -> Result<Vec<FoliageAttachment>, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    validate_cherry(config)?;
    place_below(context, random, offset(origin, Direction::Down)?)?;
    let first = (height - 1
        + sample_uniform(
            random,
            config.branch_start_minimum,
            config.branch_start_maximum,
        ))
    .max(0);
    let mut second = (height - 1
        + sample_uniform(
            random,
            config.branch_start_minimum,
            config.branch_start_maximum - 1,
        ))
    .max(0);
    if second >= first {
        second += 1;
    }
    let branch_count = sample_provider(&config.branch_count, random, 1, 3)?;
    let has_middle = branch_count == 3;
    let has_both = branch_count >= 2;
    let trunk_height = if has_middle {
        height
    } else if has_both {
        first.max(second) + 1
    } else {
        first + 1
    };
    for y in 0..trunk_height {
        let _ = place_log(context, random, offset_xyz(origin, 0, y, 0)?, None);
    }
    let mut attachments = Vec::new();
    if has_middle {
        attachments.push(attachment(offset_xyz(origin, 0, trunk_height, 0)?));
    }
    let direction = horizontal_direction(random);
    attachments.push(generate_cherry_branch_inner(
        context,
        random,
        CherryBranchInputs {
            height,
            origin,
            config,
            direction,
            start: first,
            extends: first < trunk_height - 1,
        },
    )?);
    if has_both {
        attachments.push(generate_cherry_branch_inner(
            context,
            random,
            CherryBranchInputs {
                height,
                origin,
                config,
                direction: direction.opposite(),
                start: second,
                extends: second < trunk_height - 1,
            },
        )?);
    }
    Ok(attachments)
}

struct CherryBranchInputs<'a> {
    height: i32,
    origin: BlockPos,
    config: &'a CherryTrunkConfig,
    direction: Direction,
    start: i32,
    extends: bool,
}

fn generate_cherry_branch_inner<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    inputs: CherryBranchInputs<'_>,
) -> Result<FoliageAttachment, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let mut cursor = offset_xyz(inputs.origin, 0, inputs.start, 0)?;
    let end_y = inputs.height - 1
        + sample_provider(&inputs.config.branch_end_offset_from_top, random, -16, 16)?;
    let extended = inputs.extends || end_y < inputs.start;
    let distance = sample_provider(&inputs.config.branch_horizontal_length, random, 2, 16)?
        + i32::from(extended);
    let endpoint = move_by(
        offset_xyz(inputs.origin, 0, end_y, 0)?,
        inputs.direction,
        distance,
    )?;
    let horizontal_steps = if extended { 2 } else { 1 };
    for _ in 0..horizontal_steps {
        cursor = offset(cursor, inputs.direction)?;
        let _ = place_log(context, random, cursor, Some(inputs.direction.axis()));
    }
    let vertical = if endpoint.y > cursor.y {
        Direction::Up
    } else {
        Direction::Down
    };
    loop {
        let remaining = manhattan(cursor, endpoint);
        if remaining == 0 {
            break;
        }
        let vertical_distance = endpoint.y.abs_diff(cursor.y) as f32;
        let grow_vertically = random.next_f32() < vertical_distance / remaining as f32;
        let direction = if grow_vertically {
            vertical
        } else {
            inputs.direction
        };
        cursor = offset(cursor, direction)?;
        let axis = (!grow_vertically).then_some(inputs.direction.axis());
        let _ = place_log(context, random, cursor, axis);
    }
    Ok(attachment(offset(endpoint, Direction::Up)?))
}

fn make_fancy_limb<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    start: BlockPos,
    end: BlockPos,
    place: bool,
) -> Result<bool, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    if !place && start == end {
        return Ok(true);
    }
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let dz = end.z - start.z;
    let steps = dx.abs().max(dy.abs()).max(dz.abs());
    let step_x = dx as f32 / steps as f32;
    let step_y = dy as f32 / steps as f32;
    let step_z = dz as f32 / steps as f32;
    for index in 0..=steps {
        let position = offset_xyz(
            start,
            (0.5 + index as f32 * step_x).floor() as i32,
            (0.5 + index as f32 * step_y).floor() as i32,
            (0.5 + index as f32 * step_z).floor() as i32,
        )?;
        if place {
            let x_difference = position.x.abs_diff(start.x);
            let z_difference = position.z.abs_diff(start.z);
            let axis = if x_difference.max(z_difference) == 0 {
                Axis::Y
            } else if x_difference >= z_difference {
                Axis::X
            } else {
                Axis::Z
            };
            let _ = place_log(context, random, position, Some(axis));
        } else if !is_free(context.world(), position) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn tree_shape(height: i32, y: i32) -> f32 {
    if (y as f32) < height as f32 * 0.3 {
        return -1.0;
    }
    let radius = height as f32 / 2.0;
    let adjacent = radius - y as f32;
    let mut distance = (radius * radius - adjacent * adjacent).sqrt();
    if adjacent == 0.0 {
        distance = radius;
    } else if adjacent.abs() >= radius {
        return 0.0;
    }
    distance * 0.5
}

fn trim_fancy_branch(height: i32, local_y: i32) -> bool {
    f64::from(local_y) >= f64::from(height) * 0.2
}

fn validate_cherry(config: &CherryTrunkConfig) -> Result<(), TreeCoreError> {
    if !(-16..=0).contains(&config.branch_start_minimum)
        || !(-16..=0).contains(&config.branch_start_maximum)
        || config.branch_start_maximum - config.branch_start_minimum < 1
    {
        Err(TreeCoreError::InvalidHeightConfiguration)
    } else {
        Ok(())
    }
}

fn sample_provider(
    provider: &IntProvider,
    random: &mut impl GenerationRandom,
    minimum: i32,
    maximum: i32,
) -> Result<i32, TreeCoreError> {
    let value = provider
        .sample(random)
        .map_err(|_| TreeCoreError::HeightOverflow)?;
    if (minimum..=maximum).contains(&value) {
        Ok(value)
    } else {
        Err(TreeCoreError::InvalidHeightConfiguration)
    }
}

fn sample_uniform(random: &mut impl GenerationRandom, minimum: i32, maximum: i32) -> i32 {
    minimum + bounded(random, (maximum - minimum + 1) as u32)
}

fn place_below<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    position: BlockPos,
) -> Result<(), TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    if let Some(state) = context.world().sample_below_trunk(position, random) {
        context.offer_trunk(position, state);
    }
    Ok(())
}

fn place_log<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    position: BlockPos,
    axis: Option<Axis>,
) -> bool
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let state = context.world().block_state(position);
    let valid = {
        let world = context.world();
        world.is_air(state) || world.is_replaceable_by_trees(state)
    };
    if !valid {
        return false;
    }
    let mut trunk = context.world().sample_trunk(position, random);
    if let Some(axis) = axis {
        trunk = context.world().with_trunk_axis(trunk, axis);
    }
    context.offer_trunk(position, trunk);
    true
}

fn is_free<W: TrunkWorld>(world: &mut W, position: BlockPos) -> bool {
    let state = world.block_state(position);
    world.is_air(state) || world.is_replaceable_by_trees(state) || world.is_log(state)
}

fn attachment(position: BlockPos) -> FoliageAttachment {
    FoliageAttachment {
        position,
        radius_offset: 0,
        double_trunk: false,
    }
}

fn horizontal_direction(random: &mut impl GenerationRandom) -> Direction {
    [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ][bounded(random, 4) as usize]
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> i32 {
    random.next_u32(NonZeroU32::new(bound).expect("trunk bound is nonzero")) as i32
}

fn manhattan(left: BlockPos, right: BlockPos) -> i32 {
    left.x.abs_diff(right.x) as i32
        + left.y.abs_diff(right.y) as i32
        + left.z.abs_diff(right.z) as i32
}

fn move_by(
    position: BlockPos,
    direction: Direction,
    distance: i32,
) -> Result<BlockPos, TreeCoreError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x * distance, y * distance, z * distance)
}

fn offset(position: BlockPos, direction: Direction) -> Result<BlockPos, TreeCoreError> {
    let [x, y, z] = direction.step();
    offset_xyz(position, x, y, z)
}

fn offset_xyz(position: BlockPos, x: i32, y: i32, z: i32) -> Result<BlockPos, TreeCoreError> {
    Ok(BlockPos::new(
        position
            .x
            .checked_add(x)
            .ok_or(TreeCoreError::PositionOverflow)?,
        position
            .y
            .checked_add(y)
            .ok_or(TreeCoreError::PositionOverflow)?,
        position
            .z
            .checked_add(z)
            .ok_or(TreeCoreError::PositionOverflow)?,
    ))
}
