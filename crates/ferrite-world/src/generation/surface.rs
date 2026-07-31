//! Surface-column traversal over default terrain blocks.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::surface_rule::{SurfaceContext, SurfaceEnvironment, SurfaceRule};
use crate::id::BlockStateId;

pub trait SurfaceWorld: SurfaceEnvironment {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_air(&self, state: BlockStateId) -> bool;

    fn has_nonempty_fluid(&self, state: BlockStateId) -> bool;

    fn offer_surface(&mut self, position: BlockPos, state: BlockStateId) -> bool;
}

pub fn build_surface_column<W>(
    context: &mut SurfaceContext<'_, W>,
    rule: &SurfaceRule<W::Biome>,
    x: i32,
    z: i32,
    top_y: i32,
    minimum_y: i32,
    default_block: BlockStateId,
) -> Result<(), SurfaceError>
where
    W: SurfaceWorld,
{
    context.update_xz(x, z);
    let mut stone_depth_above = 0_i32;
    let mut water_height = None;
    let mut next_ceiling_stone_y = i32::MAX;
    for y in (minimum_y..=top_y).rev() {
        let position = BlockPos::new(x, y, z);
        let state = context.environment_mut().block_state(position);
        if context.environment_mut().is_air(state) {
            stone_depth_above = 0;
            water_height = None;
            continue;
        }
        if context.environment_mut().has_nonempty_fluid(state) {
            if water_height.is_none() {
                water_height = Some(y.wrapping_add(1));
            }
            continue;
        }
        if next_ceiling_stone_y >= y {
            next_ceiling_stone_y = find_next_ceiling_stone_y(context, x, y, z, minimum_y);
        }
        let stone_depth_below = y.wrapping_sub(next_ceiling_stone_y).wrapping_add(1);
        stone_depth_above = stone_depth_above.wrapping_add(1);
        context.update_y(y, stone_depth_above, stone_depth_below, water_height);
        if state != default_block {
            continue;
        }
        if let Some(replacement) = rule.evaluate(context) {
            let _ = context
                .environment_mut()
                .offer_surface(position, replacement);
        }
    }
    Ok(())
}

fn find_next_ceiling_stone_y<W>(
    context: &mut SurfaceContext<'_, W>,
    x: i32,
    y: i32,
    z: i32,
    minimum_y: i32,
) -> i32
where
    W: SurfaceWorld,
{
    let mut next_ceiling = i32::MIN;
    for scan_y in (minimum_y.wrapping_sub(1)..y).rev() {
        let state = context
            .environment_mut()
            .block_state(BlockPos::new(x, scan_y, z));
        if context.environment_mut().is_air(state)
            || context.environment_mut().has_nonempty_fluid(state)
        {
            next_ceiling = scan_y.wrapping_add(1);
            break;
        }
    }
    next_ceiling
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SurfaceError {
    #[error("surface-column position arithmetic overflow")]
    PositionOverflow,
}
