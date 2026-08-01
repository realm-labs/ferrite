//! Integer force-field shape admission and border-aware ray clipping.

use super::BorderPoint3;
use crate::generation::border::geometry::BorderAabb;
use crate::generation::border::state::WorldBorder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BorderInteriorBox {
    pub minimum_x: i64,
    pub maximum_x: i64,
    pub minimum_z: i64,
    pub maximum_z: i64,
    pub infinite_vertical: bool,
    pub complemented: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BorderFace {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderRayHit {
    pub location: BorderPoint3,
    pub face: BorderFace,
    pub world_border_hit: bool,
}

impl WorldBorder {
    pub fn collision_shape(&self) -> BorderInteriorBox {
        let edges = self.edges();
        BorderInteriorBox {
            minimum_x: edges.minimum_x.floor() as i64,
            maximum_x: edges.maximum_x.ceil() as i64,
            minimum_z: edges.minimum_z.floor() as i64,
            maximum_z: edges.maximum_z.ceil() as i64,
            infinite_vertical: true,
            complemented: true,
        }
    }

    pub fn collision_shape_for(&self, entity_bounds: BorderAabb) -> Option<BorderInteriorBox> {
        let distance = entity_bounds
            .width()
            .abs()
            .max(entity_bounds.depth().abs())
            .max(1.0);
        let center = entity_bounds.center();
        (self.distance_to_border(center.x, center.z) < 2.0 * distance
            && self.contains_point_with_radius(center.x, center.z, distance))
        .then(|| self.collision_shape())
    }

    pub fn clip_including_border(
        &self,
        start: BorderPoint3,
        ordinary_hit: BorderRayHit,
    ) -> BorderRayHit {
        if !self.contains_point(start.x, start.z)
            || self.contains_point(ordinary_hit.location.x, ordinary_hit.location.z)
        {
            return ordinary_hit;
        }
        let face = approximate_face(start, ordinary_hit.location);
        let location = self.clamp_vector(ordinary_hit.location);
        BorderRayHit {
            location,
            face,
            world_border_hit: true,
        }
    }
}

fn approximate_face(start: BorderPoint3, end: BorderPoint3) -> BorderFace {
    let dx = (end.x - start.x) as f32;
    let dy = (end.y - start.y) as f32;
    let dz = (end.z - start.z) as f32;
    let directions = [
        (BorderFace::Down, 0.0, -1.0, 0.0),
        (BorderFace::Up, 0.0, 1.0, 0.0),
        (BorderFace::North, 0.0, 0.0, -1.0),
        (BorderFace::South, 0.0, 0.0, 1.0),
        (BorderFace::West, -1.0, 0.0, 0.0),
        (BorderFace::East, 1.0, 0.0, 0.0),
    ];
    let mut face = BorderFace::North;
    let mut best_dot = f32::MIN_POSITIVE;
    for (candidate, normal_x, normal_y, normal_z) in directions {
        let dot = dx * normal_x + dy * normal_y + dz * normal_z;
        if dot > best_dot {
            face = candidate;
            best_dot = dot;
        }
    }
    face
}
