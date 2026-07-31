//! Partial-tick border edges, containment, distance, and clamping.

use ferrite_foundation::coordinate::BlockPos;

use super::{BORDER_EPSILON, BorderPoint3, java_clamp, java_floor_i32, java_min};
use crate::generation::border::state::WorldBorder;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderEdges {
    pub minimum_x: f64,
    pub maximum_x: f64,
    pub minimum_z: f64,
    pub maximum_z: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderAabb {
    pub minimum_x: f64,
    pub minimum_y: f64,
    pub minimum_z: f64,
    pub maximum_x: f64,
    pub maximum_y: f64,
    pub maximum_z: f64,
}

impl BorderAabb {
    pub fn width(self) -> f64 {
        self.maximum_x - self.minimum_x
    }

    pub fn depth(self) -> f64 {
        self.maximum_z - self.minimum_z
    }

    pub fn center(self) -> BorderPoint3 {
        BorderPoint3 {
            x: (self.minimum_x + self.maximum_x) * 0.5,
            y: (self.minimum_y + self.maximum_y) * 0.5,
            z: (self.minimum_z + self.maximum_z) * 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BorderChunk {
    pub x: i32,
    pub z: i32,
}

impl BorderChunk {
    pub const fn minimum_block_x(self) -> i32 {
        self.x.wrapping_mul(16)
    }

    pub const fn maximum_block_x(self) -> i32 {
        self.minimum_block_x().wrapping_add(15)
    }

    pub const fn minimum_block_z(self) -> i32 {
        self.z.wrapping_mul(16)
    }

    pub const fn maximum_block_z(self) -> i32 {
        self.minimum_block_z().wrapping_add(15)
    }
}

impl WorldBorder {
    pub fn edges(&self) -> BorderEdges {
        self.edges_at(0.0)
    }

    pub fn edges_at(&self, partial_tick: f64) -> BorderEdges {
        let half_size = self.extent.size_at(partial_tick) * 0.5;
        let absolute = f64::from(self.absolute_max);
        BorderEdges {
            minimum_x: java_clamp(self.center_x - half_size, -absolute, absolute),
            maximum_x: java_clamp(self.center_x + half_size, -absolute, absolute),
            minimum_z: java_clamp(self.center_z - half_size, -absolute, absolute),
            maximum_z: java_clamp(self.center_z + half_size, -absolute, absolute),
        }
    }

    pub fn contains_point(&self, x: f64, z: f64) -> bool {
        self.contains_point_with_radius(x, z, 0.0)
    }

    pub fn contains_point_with_radius(&self, x: f64, z: f64, radius: f64) -> bool {
        let edges = self.edges();
        x >= edges.minimum_x - radius
            && x < edges.maximum_x + radius
            && z >= edges.minimum_z - radius
            && z < edges.maximum_z + radius
    }

    pub fn contains_block(&self, position: BlockPos) -> bool {
        self.contains_point(f64::from(position.x), f64::from(position.z))
    }

    pub fn contains_chunk(&self, chunk: BorderChunk) -> bool {
        self.contains_point(
            f64::from(chunk.minimum_block_x()),
            f64::from(chunk.minimum_block_z()),
        ) && self.contains_point(
            f64::from(chunk.maximum_block_x()),
            f64::from(chunk.maximum_block_z()),
        )
    }

    pub fn contains_aabb(&self, bounds: BorderAabb) -> bool {
        self.contains_point(bounds.minimum_x, bounds.minimum_z)
            && self.contains_point(
                bounds.maximum_x - BORDER_EPSILON,
                bounds.maximum_z - BORDER_EPSILON,
            )
    }

    pub fn distance_to_border(&self, x: f64, z: f64) -> f64 {
        let edges = self.edges();
        let distance = java_min(x - edges.minimum_x, edges.maximum_x - x);
        let distance = java_min(distance, z - edges.minimum_z);
        java_min(distance, edges.maximum_z - z)
    }

    pub fn clamp_vector(&self, point: BorderPoint3) -> BorderPoint3 {
        let edges = self.edges();
        BorderPoint3 {
            x: java_clamp(point.x, edges.minimum_x, edges.maximum_x - BORDER_EPSILON),
            y: point.y,
            z: java_clamp(point.z, edges.minimum_z, edges.maximum_z - BORDER_EPSILON),
        }
    }

    pub fn clamp_block(&self, point: BorderPoint3) -> BlockPos {
        let clamped = self.clamp_vector(point);
        BlockPos::new(
            java_floor_i32(clamped.x),
            java_floor_i32(clamped.y),
            java_floor_i32(clamped.z),
        )
    }
}
