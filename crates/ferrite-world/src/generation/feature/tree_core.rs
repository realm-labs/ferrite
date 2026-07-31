//! Tree orchestration, clearance, attempted-position sets, and leaf-distance repair.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::java_hash_set::JavaBlockPosSet;
use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeFeatureSize {
    TwoLayers {
        limit: i32,
        lower_size: i32,
        upper_size: i32,
        minimum_clipped_height: Option<i32>,
    },
    ThreeLayers {
        limit: i32,
        upper_limit: i32,
        lower_size: i32,
        middle_size: i32,
        upper_size: i32,
        minimum_clipped_height: Option<i32>,
    },
}

impl TreeFeatureSize {
    fn radius(self, requested_height: i32, relative_y: i32) -> i32 {
        match self {
            Self::TwoLayers {
                limit,
                lower_size,
                upper_size,
                ..
            } => {
                if relative_y < limit {
                    lower_size
                } else {
                    upper_size
                }
            }
            Self::ThreeLayers {
                limit,
                upper_limit,
                lower_size,
                middle_size,
                upper_size,
                ..
            } => {
                if relative_y < limit {
                    lower_size
                } else if relative_y >= requested_height - upper_limit {
                    upper_size
                } else {
                    middle_size
                }
            }
        }
    }

    const fn minimum_clipped_height(self) -> Option<i32> {
        match self {
            Self::TwoLayers {
                minimum_clipped_height,
                ..
            }
            | Self::ThreeLayers {
                minimum_clipped_height,
                ..
            } => minimum_clipped_height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeCoreConfig {
    pub base_height: u32,
    pub height_random_a: u32,
    pub height_random_b: u32,
    pub ignore_vines: bool,
    pub size: TreeFeatureSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoliageAttachment {
    pub position: BlockPos,
    pub radius_offset: i32,
    pub double_trunk: bool,
}

pub trait TreeWorld {
    fn minimum_y(&self) -> i32;

    fn maximum_y(&self) -> i32;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn is_replaceable_by_trees(&self, state: BlockStateId) -> bool;

    fn is_log(&self, state: BlockStateId) -> bool;

    fn is_vine(&self, state: BlockStateId) -> bool;

    fn optional_leaf_distance(&self, state: BlockStateId) -> Option<u8>;

    fn with_leaf_distance(&self, state: BlockStateId, distance: u8) -> BlockStateId;

    fn offer_tree_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn update_tree_shape_at_edge(
        &mut self,
        radius: u32,
        minimum: BlockPos,
        maximum: BlockPos,
        filled: &[BlockPos],
    );
}

pub trait TreePlan<R, W>
where
    R: GenerationRandom,
    W: TreeWorld,
{
    fn foliage_height(&mut self, requested_height: i32, random: &mut R) -> i32;

    fn foliage_radius(&mut self, random: &mut R) -> i32;

    fn trunk_origin(
        &mut self,
        world: &mut W,
        origin: BlockPos,
        random: &mut R,
    ) -> Result<BlockPos, TreeCoreError>;

    fn place_roots(
        &mut self,
        context: &mut TreePlacementContext<'_, W>,
        origin: BlockPos,
        trunk_origin: BlockPos,
        usable_height: i32,
        random: &mut R,
    ) -> Result<bool, TreeCoreError>;

    fn place_trunk(
        &mut self,
        context: &mut TreePlacementContext<'_, W>,
        trunk_origin: BlockPos,
        usable_height: i32,
        random: &mut R,
    ) -> Result<Vec<FoliageAttachment>, TreeCoreError>;

    fn place_foliage(
        &mut self,
        context: &mut TreePlacementContext<'_, W>,
        attachment: FoliageAttachment,
        usable_height: i32,
        foliage_height: i32,
        foliage_radius: i32,
        random: &mut R,
    ) -> Result<(), TreeCoreError>;

    fn decorate(
        &mut self,
        context: &mut TreePlacementContext<'_, W>,
        random: &mut R,
    ) -> Result<(), TreeCoreError>;
}

pub struct TreePlacementContext<'a, W: TreeWorld> {
    world: &'a mut W,
    roots: JavaBlockPosSet,
    trunks: JavaBlockPosSet,
    foliage: JavaBlockPosSet,
    decorators: JavaBlockPosSet,
}

impl<'a, W: TreeWorld> TreePlacementContext<'a, W> {
    pub fn new(world: &'a mut W) -> Self {
        Self {
            world,
            roots: JavaBlockPosSet::new(),
            trunks: JavaBlockPosSet::new(),
            foliage: JavaBlockPosSet::new(),
            decorators: JavaBlockPosSet::new(),
        }
    }

    pub fn world(&mut self) -> &mut W {
        self.world
    }

    pub fn offer_root(&mut self, position: BlockPos, state: BlockStateId) {
        self.roots.insert(position);
        let _ = self.world.offer_tree_block(position, state, 19);
    }

    pub fn offer_trunk(&mut self, position: BlockPos, state: BlockStateId) {
        self.trunks.insert(position);
        let _ = self.world.offer_tree_block(position, state, 19);
    }

    pub fn offer_foliage(&mut self, position: BlockPos, state: BlockStateId) {
        self.foliage.insert(position);
        let _ = self.world.offer_tree_block(position, state, 19);
    }

    pub fn offer_decorator(&mut self, position: BlockPos, state: BlockStateId) {
        self.decorators.insert(position);
        let _ = self.world.offer_tree_block(position, state, 19);
    }

    pub fn ordered_roots(&self) -> Vec<BlockPos> {
        ordered_by_y(&self.roots)
    }

    pub fn ordered_trunks(&self) -> Vec<BlockPos> {
        ordered_by_y(&self.trunks)
    }

    pub fn ordered_foliage(&self) -> Vec<BlockPos> {
        ordered_by_y(&self.foliage)
    }

    pub fn foliage_attempted(&self, position: BlockPos) -> bool {
        self.foliage.contains(&position)
    }

    pub fn lowest_trunk_or_root(&self) -> Vec<BlockPos> {
        let mut logs = self.ordered_trunks();
        let roots = self.ordered_roots();
        if roots.is_empty() {
            return logs;
        }
        if logs
            .first()
            .zip(roots.first())
            .is_some_and(|(log, root)| log.y == root.y)
        {
            logs.extend(roots);
            logs
        } else {
            roots
        }
    }
}

pub fn place_tree_core<R, W, P>(
    world: &mut W,
    origin: BlockPos,
    config: TreeCoreConfig,
    plan: &mut P,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TreeCoreError>
where
    R: GenerationRandom,
    W: TreeWorld,
    P: TreePlan<R, W>,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    validate_config(config)?;
    let requested_height = i32::try_from(config.base_height)
        .ok()
        .and_then(|base| {
            let a = random.next_u32(
                NonZeroU32::new(config.height_random_a + 1)
                    .expect("tree height A bound is nonzero"),
            );
            let b = random.next_u32(
                NonZeroU32::new(config.height_random_b + 1)
                    .expect("tree height B bound is nonzero"),
            );
            base.checked_add(a as i32)?.checked_add(b as i32)
        })
        .ok_or(TreeCoreError::HeightOverflow)?;
    let foliage_height = plan.foliage_height(requested_height, random);
    let trunk_below_foliage = requested_height - foliage_height;
    let foliage_radius = plan.foliage_radius(random);
    let trunk_origin = plan.trunk_origin(world, origin, random)?;
    if !inside_build_bounds(world, origin, trunk_origin, requested_height)? {
        return Ok(false);
    }
    let usable_height = maximum_free_height(world, trunk_origin, requested_height, config)?;
    if usable_height < requested_height
        && config
            .size
            .minimum_clipped_height()
            .is_none_or(|minimum| usable_height < minimum)
    {
        return Ok(false);
    }

    let mut context = TreePlacementContext::new(world);
    if !plan.place_roots(&mut context, origin, trunk_origin, usable_height, random)? {
        return Ok(false);
    }
    let attachments = plan.place_trunk(&mut context, trunk_origin, usable_height, random)?;
    for attachment in attachments {
        plan.place_foliage(
            &mut context,
            attachment,
            usable_height,
            foliage_height,
            foliage_radius,
            random,
        )?;
    }
    let _ = trunk_below_foliage;
    if context.trunks.is_empty() && context.foliage.is_empty() {
        return Ok(false);
    }
    plan.decorate(&mut context, random)?;
    repair_leaf_distances(&mut context)?;
    Ok(true)
}

fn inside_build_bounds<W: TreeWorld>(
    world: &W,
    origin: BlockPos,
    trunk_origin: BlockPos,
    height: i32,
) -> Result<bool, TreeCoreError> {
    let minimum_allowed = world
        .minimum_y()
        .checked_add(1)
        .ok_or(TreeCoreError::HeightOverflow)?;
    let top = origin
        .y
        .max(trunk_origin.y)
        .checked_add(height)
        .and_then(|value| value.checked_add(1))
        .ok_or(TreeCoreError::HeightOverflow)?;
    let maximum_allowed = world
        .maximum_y()
        .checked_add(1)
        .ok_or(TreeCoreError::HeightOverflow)?;
    Ok(origin.y.min(trunk_origin.y) >= minimum_allowed && top <= maximum_allowed)
}

fn maximum_free_height<W: TreeWorld>(
    world: &mut W,
    trunk_origin: BlockPos,
    requested_height: i32,
    config: TreeCoreConfig,
) -> Result<i32, TreeCoreError> {
    for y in 0..=requested_height + 1 {
        let radius = config.size.radius(requested_height, y);
        for x in -radius..=radius {
            for z in -radius..=radius {
                let position = offset_xyz(trunk_origin, x, y, z)?;
                let state = world.block_state(position);
                let free = world.is_air(state)
                    || world.is_replaceable_by_trees(state)
                    || world.is_log(state);
                if !free || (!config.ignore_vines && world.is_vine(state)) {
                    return Ok(y - 2);
                }
            }
        }
    }
    Ok(requested_height)
}

fn repair_leaf_distances<W: TreeWorld>(
    context: &mut TreePlacementContext<'_, W>,
) -> Result<(), TreeCoreError> {
    let all_positions = context
        .roots
        .iter()
        .chain(context.trunks.iter())
        .chain(context.foliage.iter())
        .chain(context.decorators.iter())
        .collect::<Vec<_>>();
    let Some((minimum, maximum)) = bounding_box(&all_positions) else {
        return Ok(());
    };
    let mut filled = JavaBlockPosSet::new();
    for position in context.roots.iter().chain(context.decorators.iter()) {
        filled.insert(position);
    }
    let mut frontiers = std::array::from_fn::<JavaBlockPosSet, 7, _>(|_| JavaBlockPosSet::new());
    for position in context.trunks.iter() {
        frontiers[0].insert(position);
    }
    let mut processed = JavaBlockPosSet::new();
    loop {
        let next = frontiers
            .iter()
            .enumerate()
            .find_map(|(distance, frontier)| {
                frontier
                    .iter()
                    .find(|position| !processed.contains(position))
                    .map(|position| (distance, position))
            });
        let Some((distance, position)) = next else {
            break;
        };
        processed.insert(position);
        filled.insert(position);
        if distance > 0 {
            let state = context.world.block_state(position);
            let state = context.world.with_leaf_distance(state, distance as u8);
            let _ = context.world.offer_tree_block(position, state, 19);
        }
        for direction in Direction::ALL {
            let neighbor = offset(position, direction)?;
            if !inside_box(neighbor, minimum, maximum) || filled.contains(&neighbor) {
                continue;
            }
            let state = context.world.block_state(neighbor);
            if let Some(old_distance) = context.world.optional_leaf_distance(state) {
                let target = old_distance.min(distance as u8 + 1);
                if target < 7 {
                    frontiers[target as usize].insert(neighbor);
                }
            }
        }
    }
    let filled = filled.iter().collect::<Vec<_>>();
    context
        .world
        .update_tree_shape_at_edge(3, minimum, maximum, &filled);
    Ok(())
}

fn bounding_box(positions: &[BlockPos]) -> Option<(BlockPos, BlockPos)> {
    let first = *positions.first()?;
    let mut minimum = first;
    let mut maximum = first;
    for position in positions.iter().copied().skip(1) {
        minimum.x = minimum.x.min(position.x);
        minimum.y = minimum.y.min(position.y);
        minimum.z = minimum.z.min(position.z);
        maximum.x = maximum.x.max(position.x);
        maximum.y = maximum.y.max(position.y);
        maximum.z = maximum.z.max(position.z);
    }
    Some((minimum, maximum))
}

fn inside_box(position: BlockPos, minimum: BlockPos, maximum: BlockPos) -> bool {
    (minimum.x..=maximum.x).contains(&position.x)
        && (minimum.y..=maximum.y).contains(&position.y)
        && (minimum.z..=maximum.z).contains(&position.z)
}

fn ordered_by_y(set: &JavaBlockPosSet) -> Vec<BlockPos> {
    let mut positions = set.iter().collect::<Vec<_>>();
    positions.sort_by_key(|position| position.y);
    positions
}

fn validate_config(config: TreeCoreConfig) -> Result<(), TreeCoreError> {
    if config.base_height > 32 || config.height_random_a > 24 || config.height_random_b > 24 {
        return Err(TreeCoreError::InvalidHeightConfiguration);
    }
    Ok(())
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TreeCoreError {
    #[error("tree height configuration exceeds codec bounds")]
    InvalidHeightConfiguration,
    #[error("tree height arithmetic overflow")]
    HeightOverflow,
    #[error("tree position overflow")]
    PositionOverflow,
}
