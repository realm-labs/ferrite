//! Straight, giant, mega-jungle, forking, bending, and upward-branching trunks.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::{Axis, Direction};

use crate::generation::feature::provider::IntProvider;
use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::tree_core::{
    FoliageAttachment, TreeCoreError, TreePlacementContext, TreeWorld,
};
use crate::id::BlockStateId;

pub trait TrunkWorld: TreeWorld {
    fn sample_below_trunk<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> Option<BlockStateId>;

    fn sample_trunk<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        random: &mut R,
    ) -> BlockStateId;

    fn with_trunk_axis(&self, state: BlockStateId, axis: Axis) -> BlockStateId;

    fn can_upward_branch_grow_through(&self, state: BlockStateId) -> bool;

    fn is_leaves(&self, _state: BlockStateId) -> bool {
        false
    }
}

pub fn place_straight_trunk<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    height: i32,
    origin: BlockPos,
) -> Result<Vec<FoliageAttachment>, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    place_below(context, random, offset(origin, Direction::Down)?)?;
    for y in 0..height {
        let position = offset_xyz(origin, 0, y, 0)?;
        let _ = place_log(context, random, position, None, false);
    }
    Ok(vec![FoliageAttachment {
        position: offset_xyz(origin, 0, height, 0)?,
        radius_offset: 0,
        double_trunk: false,
    }])
}

pub fn place_giant_trunk<R, W>(
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
    for y in 0..height {
        let _ = place_log_if_free(context, random, offset_xyz(origin, 0, y, 0)?, None, false);
        if y < height - 1 {
            for (x, z) in [(1, 0), (1, 1), (0, 1)] {
                let _ =
                    place_log_if_free(context, random, offset_xyz(origin, x, y, z)?, None, false);
            }
        }
    }
    Ok(vec![FoliageAttachment {
        position: offset_xyz(origin, 0, height, 0)?,
        radius_offset: 0,
        double_trunk: true,
    }])
}

pub fn place_mega_jungle_trunk<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    height: i32,
    origin: BlockPos,
) -> Result<Vec<FoliageAttachment>, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let mut attachments = place_giant_trunk(context, random, height, origin)?;
    let mut branch_height = height - 2 - bounded_i32(random, 4);
    while branch_height > height / 2 {
        let angle = random.next_f32() * (std::f32::consts::PI * 2.0);
        let mut branch_x = 0;
        let mut branch_z = 0;
        for index in 0..5 {
            branch_x = (1.5 + minecraft_cos(angle) * index as f32) as i32;
            branch_z = (1.5 + minecraft_sin(angle) * index as f32) as i32;
            let position = offset_xyz(origin, branch_x, branch_height - 3 + index / 2, branch_z)?;
            let _ = place_log(context, random, position, None, false);
        }
        attachments.push(FoliageAttachment {
            position: offset_xyz(origin, branch_x, branch_height, branch_z)?,
            radius_offset: -2,
            double_trunk: false,
        });
        branch_height -= 2 + bounded_i32(random, 4);
    }
    Ok(attachments)
}

pub fn place_forking_trunk<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    height: i32,
    origin: BlockPos,
) -> Result<Vec<FoliageAttachment>, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    place_below(context, random, offset(origin, Direction::Down)?)?;
    let mut attachments = Vec::new();
    let lean_direction = horizontal_direction(random);
    let lean_height = height - bounded_i32(random, 4) - 1;
    let mut lean_steps = 3 - bounded_i32(random, 3);
    let mut x = origin.x;
    let mut z = origin.z;
    let mut end_y = None;
    for y_offset in 0..height {
        if y_offset >= lean_height && lean_steps > 0 {
            let [dx, _, dz] = lean_direction.step();
            x += dx;
            z += dz;
            lean_steps -= 1;
        }
        let y = origin.y + y_offset;
        if place_log(context, random, BlockPos::new(x, y, z), None, false) {
            end_y = Some(y + 1);
        }
    }
    if let Some(y) = end_y {
        attachments.push(FoliageAttachment {
            position: BlockPos::new(x, y, z),
            radius_offset: 1,
            double_trunk: false,
        });
    }
    x = origin.x;
    z = origin.z;
    let branch_direction = horizontal_direction(random);
    if branch_direction != lean_direction {
        let branch_start = lean_height - bounded_i32(random, 2) - 1;
        let mut branch_steps = 1 + bounded_i32(random, 3);
        end_y = None;
        let mut y_offset = branch_start;
        while y_offset < height && branch_steps > 0 {
            if y_offset >= 1 {
                let [dx, _, dz] = branch_direction.step();
                x += dx;
                z += dz;
                let y = origin.y + y_offset;
                if place_log(context, random, BlockPos::new(x, y, z), None, false) {
                    end_y = Some(y + 1);
                }
            }
            y_offset += 1;
            branch_steps -= 1;
        }
        if let Some(y) = end_y {
            attachments.push(FoliageAttachment {
                position: BlockPos::new(x, y, z),
                radius_offset: 0,
                double_trunk: false,
            });
        }
    }
    Ok(attachments)
}

pub fn place_bending_trunk<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    height: i32,
    origin: BlockPos,
    minimum_height_for_leaves: i32,
    bend_length: &IntProvider,
) -> Result<Vec<FoliageAttachment>, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let direction = horizontal_direction(random);
    let last_vertical_index = height - 1;
    let mut position = origin;
    place_below(context, random, offset(origin, Direction::Down)?)?;
    let mut attachments = Vec::new();
    for index in 0..=last_vertical_index {
        if index + 1 >= last_vertical_index + bounded_i32(random, 2) {
            position = offset(position, direction)?;
        }
        if valid_tree_position(context.world(), position) {
            let _ = place_log(context, random, position, None, false);
        }
        if index >= minimum_height_for_leaves {
            attachments.push(attachment(position));
        }
        position = offset(position, Direction::Up)?;
    }
    let length = bend_length
        .sample(random)
        .map_err(|_| TreeCoreError::HeightOverflow)?;
    for _ in 0..=length {
        if valid_tree_position(context.world(), position) {
            let _ = place_log(context, random, position, None, false);
        }
        attachments.push(attachment(position));
        position = offset(position, direction)?;
    }
    Ok(attachments)
}

pub fn place_upward_branching_trunk<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    height: i32,
    origin: BlockPos,
    branch_steps: &IntProvider,
    branch_probability: f32,
    branch_length: &IntProvider,
) -> Result<Vec<FoliageAttachment>, TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let mut attachments = Vec::new();
    for height_offset in 0..height {
        let current_y = origin.y + height_offset;
        let position = BlockPos::new(origin.x, current_y, origin.z);
        let placed = place_log(context, random, position, None, true);
        if placed && height_offset < height - 1 && random.next_f32() < branch_probability {
            let direction = horizontal_direction(random);
            let first_length = branch_length
                .sample(random)
                .map_err(|_| TreeCoreError::HeightOverflow)?;
            let second_length = branch_length
                .sample(random)
                .map_err(|_| TreeCoreError::HeightOverflow)?;
            let start = (first_length - second_length - 1).max(0);
            let steps = branch_steps
                .sample(random)
                .map_err(|_| TreeCoreError::HeightOverflow)?;
            let inputs = UpwardBranchInputs {
                tree_height: height,
                current_y,
                direction,
                start,
                steps,
            };
            place_upward_branch_inner(context, random, &mut attachments, position, inputs)?;
        }
        if height_offset == height - 1 {
            attachments.push(attachment(BlockPos::new(origin.x, current_y + 1, origin.z)));
        }
    }
    Ok(attachments)
}

struct UpwardBranchInputs {
    tree_height: i32,
    current_y: i32,
    direction: Direction,
    start: i32,
    steps: i32,
}

fn place_upward_branch_inner<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    attachments: &mut Vec<FoliageAttachment>,
    base_position: BlockPos,
    inputs: UpwardBranchInputs,
) -> Result<(), TreeCoreError>
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let mut last_y = inputs.current_y + inputs.start;
    let mut x = base_position.x;
    let mut z = base_position.z;
    let mut index = inputs.start;
    let mut remaining = inputs.steps;
    while index < inputs.tree_height && remaining > 0 {
        if index >= 1 {
            let [dx, _, dz] = inputs.direction.step();
            x += dx;
            z += dz;
            let y = inputs.current_y + index;
            last_y = y;
            let position = BlockPos::new(x, y, z);
            if place_log(context, random, position, None, true) {
                last_y += 1;
            }
            attachments.push(attachment(position));
        }
        index += 1;
        remaining -= 1;
    }
    if last_y - inputs.current_y > 1 {
        let foliage = BlockPos::new(x, last_y, z);
        attachments.push(attachment(foliage));
        attachments.push(attachment(offset_xyz(foliage, 0, -2, 0)?));
    }
    Ok(())
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

fn place_log_if_free<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    position: BlockPos,
    axis: Option<Axis>,
    allow_grow_through: bool,
) -> bool
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let state = context.world().block_state(position);
    let free = {
        let world = context.world();
        world.is_air(state) || world.is_replaceable_by_trees(state) || world.is_log(state)
    };
    free && place_log(context, random, position, axis, allow_grow_through)
}

fn place_log<R, W>(
    context: &mut TreePlacementContext<'_, W>,
    random: &mut R,
    position: BlockPos,
    axis: Option<Axis>,
    allow_grow_through: bool,
) -> bool
where
    R: GenerationRandom,
    W: TrunkWorld,
{
    let state = context.world().block_state(position);
    let valid = {
        let world = context.world();
        world.is_air(state)
            || world.is_replaceable_by_trees(state)
            || (allow_grow_through && world.can_upward_branch_grow_through(state))
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

fn valid_tree_position<W: TrunkWorld>(world: &mut W, position: BlockPos) -> bool {
    let state = world.block_state(position);
    world.is_air(state) || world.is_replaceable_by_trees(state)
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
    ][bounded_i32(random, 4) as usize]
}

fn bounded_i32(random: &mut impl GenerationRandom, bound: u32) -> i32 {
    random.next_u32(NonZeroU32::new(bound).expect("trunk bound is nonzero")) as i32
}

fn minecraft_sin(value: f32) -> f32 {
    let index = ((value * 10_430.378) as i32 & 65_535) as u32;
    (f64::from(index) * std::f64::consts::TAU / 65_536.0).sin() as f32
}

fn minecraft_cos(value: f32) -> f32 {
    minecraft_sin(value + std::f32::consts::FRAC_PI_2)
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
