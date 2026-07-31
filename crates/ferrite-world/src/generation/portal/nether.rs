//! Nether-portal destination selection, frame creation, and exit geometry.

use std::cmp::Ordering;

use ferrite_foundation::coordinate::BlockPos;

use super::{ChunkTicket, HorizontalAxis, PortalRectangle, Rotation, Vec3, offset};
use crate::generation::dimension::{DimensionType, Position, scale_command_position};

pub const NETHER_SEARCH_RADIUS: i32 = 16;
pub const OVERWORLD_SEARCH_RADIUS: i32 = 128;
pub const PORTAL_CREATION_RADIUS: i32 = 16;
pub const MAX_PORTAL_RECTANGLE: u8 = 21;
const WORLD_BORDER_EPSILON: f64 = 9.999_999_747_378_752e-6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PortalBorder {
    pub minimum_x: f64,
    pub maximum_x: f64,
    pub minimum_z: f64,
    pub maximum_z: f64,
}

impl PortalBorder {
    pub fn contains(self, position: BlockPos) -> bool {
        f64::from(position.x) >= self.minimum_x
            && f64::from(position.x) < self.maximum_x
            && f64::from(position.z) >= self.minimum_z
            && f64::from(position.z) < self.maximum_z
    }

    pub fn clamp_floor(self, position: Position) -> BlockPos {
        BlockPos::new(
            position
                .x
                .clamp(self.minimum_x, self.maximum_x - WORLD_BORDER_EPSILON)
                .floor() as i32,
            position.y.floor() as i32,
            position
                .z
                .clamp(self.minimum_z, self.maximum_z - WORLD_BORDER_EPSILON)
                .floor() as i32,
        )
    }

    pub fn clamp_block(self, position: BlockPos) -> BlockPos {
        self.clamp_floor(Position {
            x: f64::from(position.x),
            y: f64::from(position.y),
            z: f64::from(position.z),
        })
    }
}

pub fn nether_destination_key(source_key: &str) -> &'static str {
    if source_key == "minecraft:the_nether" {
        "minecraft:overworld"
    } else {
        "minecraft:the_nether"
    }
}

pub fn scaled_search_block(
    position: Position,
    source: &DimensionType,
    destination: &DimensionType,
    border: PortalBorder,
) -> BlockPos {
    border.clamp_floor(scale_command_position(position, source, destination))
}

pub const fn portal_search_radius(destination_key: &str) -> i32 {
    if const_str_eq(destination_key, "minecraft:the_nether") {
        NETHER_SEARCH_RADIUS
    } else {
        OVERWORLD_SEARCH_RADIUS
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PortalSearchPlan {
    pub center: BlockPos,
    pub radius: i32,
    pub ensure_loaded_and_valid: bool,
    pub inclusive_xz_square: bool,
}

pub const fn portal_search_plan(center: BlockPos, destination_key: &str) -> PortalSearchPlan {
    PortalSearchPlan {
        center,
        radius: portal_search_radius(destination_key),
        ensure_loaded_and_valid: true,
        inclusive_xz_square: true,
    }
}

const fn const_str_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PortalPoi {
    pub position: BlockPos,
    pub axis: Option<HorizontalAxis>,
    /// Stable input order from the POI section stream.
    pub encounter_order: u64,
}

pub fn select_portal_poi(
    target: BlockPos,
    destination_key: &str,
    border: PortalBorder,
    points: impl IntoIterator<Item = PortalPoi>,
) -> Option<PortalPoi> {
    let plan = portal_search_plan(target, destination_key);
    points
        .into_iter()
        .filter(|poi| poi.axis.is_some())
        .filter(|poi| border.contains(poi.position))
        .filter(|poi| {
            (poi.position.x - plan.center.x).abs() <= plan.radius
                && (poi.position.z - plan.center.z).abs() <= plan.radius
        })
        .min_by(|left, right| compare_poi(*left, *right, target))
}

fn compare_poi(left: PortalPoi, right: PortalPoi, target: BlockPos) -> Ordering {
    squared_distance(left.position, target)
        .cmp(&squared_distance(right.position, target))
        .then(left.position.y.cmp(&right.position.y))
        .then(left.encounter_order.cmp(&right.encounter_order))
}

fn squared_distance(left: BlockPos, right: BlockPos) -> i128 {
    let dx = i128::from(left.x) - i128::from(right.x);
    let dy = i128::from(left.y) - i128::from(right.y);
    let dz = i128::from(left.z) - i128::from(right.z);
    dx * dx + dy * dy + dz * dz
}

/// Finds the largest all-matching rectangle containing `origin`, capped at 21×21.
pub fn largest_matching_rectangle(
    origin: BlockPos,
    axis: HorizontalAxis,
    mut matches_state_identity: impl FnMut(BlockPos) -> bool,
) -> PortalRectangle {
    let along = axis.positive_step();
    const REACH: i32 = MAX_PORTAL_RECTANGLE as i32 - 1;
    const GRID: usize = REACH as usize * 2 + 1;
    let mut prefix = [[0_u16; GRID + 1]; GRID + 1];
    for vertical in -REACH..=REACH {
        for horizontal in -REACH..=REACH {
            let along_position = offset(origin, along, horizontal);
            let position = BlockPos::new(
                along_position.x,
                along_position.y + vertical,
                along_position.z,
            );
            let row = (vertical + REACH) as usize + 1;
            let column = (horizontal + REACH) as usize + 1;
            let value = u16::from(matches_state_identity(position));
            prefix[row][column] = value + prefix[row - 1][column] + prefix[row][column - 1]
                - prefix[row - 1][column - 1];
        }
    }
    let mut best = (1_i32, 0_i32, 0_i32, 0_i32, 0_i32);
    for left in -REACH..=0 {
        for right in 0..=(left + REACH).min(REACH) {
            let width = right - left + 1;
            for bottom in -REACH..=0 {
                for top in 0..=(bottom + REACH).min(REACH) {
                    let height = top - bottom + 1;
                    let area = width * height;
                    if area > best.0
                        && rectangle_sum(&prefix, left, right, bottom, top) == area as u16
                    {
                        best = (area, left, right, bottom, top);
                    }
                }
            }
        }
    }
    let left = offset(origin, along, best.1);
    PortalRectangle {
        minimum: BlockPos::new(left.x, origin.y + best.3, left.z),
        axis,
        width: (best.2 - best.1 + 1) as u8,
        height: (best.4 - best.3 + 1) as u8,
    }
}

fn rectangle_sum(prefix: &[[u16; 42]; 42], left: i32, right: i32, bottom: i32, top: i32) -> u16 {
    let x0 = (left + 20) as usize;
    let x1 = (right + 20) as usize + 1;
    let y0 = (bottom + 20) as usize;
    let y1 = (top + 20) as usize + 1;
    prefix[y1][x1] + prefix[y0][x0] - prefix[y0][x1] - prefix[y1][x0]
}

pub trait PortalCreationWorld {
    fn border(&self) -> PortalBorder;
    fn motion_blocking_height(&self, x: i32, z: i32) -> i32;
    fn is_dry_replaceable(&self, position: BlockPos) -> bool;
    fn is_solid(&self, position: BlockPos) -> bool;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortalCreationSiteKind {
    Preferred,
    CenterOnly,
    Fallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortalBlock {
    Obsidian,
    Air,
    Portal(HorizontalAxis),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PortalWrite {
    pub position: BlockPos,
    pub block: PortalBlock,
    pub flags: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortalCreation {
    pub rectangle: PortalRectangle,
    pub site_kind: PortalCreationSiteKind,
    pub writes: Vec<PortalWrite>,
}

pub fn create_portal(
    world: &impl PortalCreationWorld,
    target: BlockPos,
    axis: HorizontalAxis,
    min_y: i32,
    max_y: i32,
    logical_height: u32,
) -> Option<PortalCreation> {
    let logical_top = min_y
        .saturating_add(logical_height as i32)
        .saturating_sub(1);
    let mut preferred: Option<(i128, BlockPos)> = None;
    let mut center_only: Option<(i128, BlockPos)> = None;
    for column in spiral_columns(target, PORTAL_CREATION_RADIUS) {
        let forward = offset(column, axis.positive_step(), 1);
        if !world.border().contains(column) || !world.border().contains(forward) {
            continue;
        }
        let start_y = world
            .motion_blocking_height(column.x, column.z)
            .min(max_y)
            .min(logical_top);
        for y in (min_y..=start_y).rev() {
            let base = BlockPos::new(column.x, y, column.z);
            if !world.is_dry_replaceable(base) {
                continue;
            }
            if base.y.saturating_add(4) > logical_top {
                continue;
            }
            if site_fits(world, base, axis, false) {
                let distance = squared_distance(base, target);
                if center_only.is_none_or(|(best, _)| distance < best) {
                    center_only = Some((distance, base));
                }
            }
            if site_fits(world, base, axis, true) {
                let distance = squared_distance(base, target);
                if preferred.is_none_or(|(best, _)| distance < best) {
                    preferred = Some((distance, base));
                }
            }
        }
    }
    if let Some((_, base)) = preferred {
        return Some(build_frame(
            base,
            axis,
            PortalCreationSiteKind::Preferred,
            false,
        ));
    }
    if let Some((_, base)) = center_only {
        return Some(build_frame(
            base,
            axis,
            PortalCreationSiteKind::CenterOnly,
            false,
        ));
    }

    let lower = min_y.saturating_sub(1).max(70);
    let upper = logical_top.saturating_sub(9);
    if lower > upper {
        return None;
    }
    let fallback_y = target.y.clamp(lower, upper);
    let raw = offset(
        BlockPos::new(target.x, fallback_y, target.z),
        axis.positive_step(),
        -1,
    );
    let base = world.border().clamp_block(raw);
    Some(build_frame(
        base,
        axis,
        PortalCreationSiteKind::Fallback,
        true,
    ))
}

fn site_fits(
    world: &impl PortalCreationWorld,
    base: BlockPos,
    axis: HorizontalAxis,
    preferred: bool,
) -> bool {
    let along = axis.positive_step();
    let perpendicular = axis.clockwise_step();
    let offsets: &[i32] = if preferred { &[-1, 0, 1] } else { &[0] };
    offsets.iter().copied().all(|plane| {
        (-1..=2).all(|width| {
            let column = offset(offset(base, perpendicular, plane), along, width);
            world.is_solid(BlockPos::new(column.x, column.y - 1, column.z))
                && (0..=3).all(|dy| {
                    world.is_dry_replaceable(BlockPos::new(column.x, column.y + dy, column.z))
                })
        })
    })
}

fn build_frame(
    base: BlockPos,
    axis: HorizontalAxis,
    site_kind: PortalCreationSiteKind,
    fallback_clearance: bool,
) -> PortalCreation {
    let along = axis.positive_step();
    let perpendicular = axis.clockwise_step();
    let mut writes = Vec::with_capacity(if fallback_clearance { 44 } else { 20 });
    if fallback_clearance {
        for plane in -1..=1 {
            for width in 0..=1 {
                let column = offset(offset(base, perpendicular, plane), along, width);
                for dy in -1..=2 {
                    writes.push(PortalWrite {
                        position: BlockPos::new(column.x, column.y + dy, column.z),
                        block: if dy < 0 {
                            PortalBlock::Obsidian
                        } else {
                            PortalBlock::Air
                        },
                        flags: 3,
                    });
                }
            }
        }
    }
    for width in -1..=2 {
        for dy in -1..=3 {
            let border = width == -1 || width == 2 || dy == -1 || dy == 3;
            if border {
                let position = offset(base, along, width);
                writes.push(PortalWrite {
                    position: BlockPos::new(position.x, position.y + dy, position.z),
                    block: PortalBlock::Obsidian,
                    flags: 3,
                });
            }
        }
    }
    for width in 0..=1 {
        for dy in 0..=2 {
            let position = offset(base, along, width);
            writes.push(PortalWrite {
                position: BlockPos::new(position.x, position.y + dy, position.z),
                block: PortalBlock::Portal(axis),
                flags: 18,
            });
        }
    }
    PortalCreation {
        rectangle: PortalRectangle {
            minimum: base,
            axis,
            width: 2,
            height: 3,
        },
        site_kind,
        writes,
    }
}

/// Source-ordered square spiral beginning at the target, then east and south.
pub fn spiral_columns(target: BlockPos, radius: i32) -> Vec<BlockPos> {
    let radius = radius.max(0);
    let side = radius.saturating_mul(2).saturating_add(1);
    let capacity = i64::from(side).saturating_mul(i64::from(side)) as usize;
    let mut result = Vec::with_capacity(capacity);
    result.push(target);
    let mut x = 0_i32;
    let mut z = 0_i32;
    let mut step_length = 1_i32;
    while result.len() < capacity {
        for (dx, dz) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
            for _ in 0..step_length {
                x += dx;
                z += dz;
                if x.abs() <= radius && z.abs() <= radius {
                    result.push(BlockPos::new(target.x + x, target.y, target.z + z));
                    if result.len() == capacity {
                        return result;
                    }
                }
            }
            if dz != 0 {
                step_length += 1;
            }
        }
    }
    result
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PortalRelativePosition {
    pub horizontal_fraction: f64,
    pub vertical_fraction: f64,
    pub perpendicular_offset: f64,
}

pub fn relative_entry_position(
    rectangle: Option<PortalRectangle>,
    position: Vec3,
    entity_width: f64,
    entity_height: f64,
) -> (HorizontalAxis, PortalRelativePosition) {
    let Some(rectangle) = rectangle else {
        return (
            HorizontalAxis::X,
            PortalRelativePosition {
                horizontal_fraction: 0.5,
                vertical_fraction: 0.0,
                perpendicular_offset: 0.0,
            },
        );
    };
    let portal_width = f64::from(rectangle.width);
    let portal_height = f64::from(rectangle.height);
    let along_position = match rectangle.axis {
        HorizontalAxis::X => position.x,
        HorizontalAxis::Z => position.z,
    };
    let along_minimum = match rectangle.axis {
        HorizontalAxis::X => f64::from(rectangle.minimum.x),
        HorizontalAxis::Z => f64::from(rectangle.minimum.z),
    };
    let horizontal_space = portal_width - entity_width;
    let horizontal_fraction = if horizontal_space > 0.0 {
        ((along_position - along_minimum - entity_width * 0.5) / horizontal_space).clamp(0.0, 1.0)
    } else {
        0.5
    };
    let vertical_space = portal_height - entity_height;
    let vertical_fraction = if vertical_space > 0.0 {
        ((position.y - f64::from(rectangle.minimum.y)) / vertical_space).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let perpendicular_offset = match rectangle.axis {
        HorizontalAxis::X => position.z - (f64::from(rectangle.minimum.z) + 0.5),
        HorizontalAxis::Z => position.x - (f64::from(rectangle.minimum.x) + 0.5),
    };
    (
        rectangle.axis,
        PortalRelativePosition {
            horizontal_fraction,
            vertical_fraction,
            perpendicular_offset,
        },
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetherExit {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Rotation,
    pub ticket: ChunkTicket,
    pub player_level_event: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetherExitInput {
    pub destination: PortalRectangle,
    pub source_axis: HorizontalAxis,
    pub relative: PortalRelativePosition,
    pub entity_size: [f64; 2],
    pub velocity: Vec3,
    pub rotation: Rotation,
    pub is_server_player: bool,
    pub existing_poi: Option<BlockPos>,
}

pub fn nether_exit(
    input: NetherExitInput,
    mut collision_adjustment: impl FnMut(Vec3, [f64; 3]) -> Option<Vec3>,
) -> NetherExit {
    let NetherExitInput {
        destination,
        source_axis,
        relative,
        entity_size,
        velocity,
        rotation,
        is_server_player,
        existing_poi,
    } = input;
    let width = f64::from(destination.width);
    let height = f64::from(destination.height);
    let along = entity_size[0] * 0.5 + (width - entity_size[0]) * relative.horizontal_fraction;
    let y =
        f64::from(destination.minimum.y) + (height - entity_size[1]) * relative.vertical_fraction;
    let perpendicular = 0.5 + relative.perpendicular_offset;
    let computed = match destination.axis {
        HorizontalAxis::X => Vec3 {
            x: f64::from(destination.minimum.x) + along,
            y,
            z: f64::from(destination.minimum.z) + perpendicular,
        },
        HorizontalAxis::Z => Vec3 {
            x: f64::from(destination.minimum.x) + perpendicular,
            y,
            z: f64::from(destination.minimum.z) + along,
        },
    };
    let position = if entity_size[0] <= 4.0 && entity_size[1] <= 4.0 {
        collision_adjustment(computed, [width, height + 1.0, width]).unwrap_or(computed)
    } else {
        computed
    };
    NetherExit {
        position,
        velocity,
        rotation: Rotation {
            yaw: rotation.yaw
                + if source_axis == destination.axis {
                    0.0
                } else {
                    90.0
                },
            pitch: rotation.pitch,
        },
        ticket: ChunkTicket::portal(existing_poi.unwrap_or_else(|| position.containing())),
        player_level_event: is_server_player.then_some(1032),
    }
}
