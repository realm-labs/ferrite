use crate::player::state::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionProbe {
    pub actual_displacement: Vec3,
    pub old_box_collision_free: bool,
    pub introduced_collision: bool,
    pub supporting_collision_before: bool,
    pub nearby_block_below: bool,
}

impl CollisionProbe {
    #[must_use]
    pub const fn unobstructed(requested: Vec3) -> Self {
        Self {
            actual_displacement: requested,
            old_box_collision_free: true,
            introduced_collision: false,
            supporting_collision_before: false,
            nearby_block_below: false,
        }
    }
}

pub trait CollisionWorld {
    fn probe_player_movement(&self, origin: Vec3, requested: Vec3) -> CollisionProbe;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoCollision;

impl CollisionWorld for NoCollision {
    fn probe_player_movement(&self, _origin: Vec3, requested: Vec3) -> CollisionProbe {
        CollisionProbe::unobstructed(requested)
    }
}

/// Collision projection for the bootstrap flat world. `ground_y` is the player's minimum feet Y.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlatWorldCollision {
    pub ground_y: f64,
}

impl CollisionWorld for FlatWorldCollision {
    fn probe_player_movement(&self, origin: Vec3, requested: Vec3) -> CollisionProbe {
        let target_y = origin.y + requested.y;
        let actual_y = if origin.y >= self.ground_y && target_y < self.ground_y {
            self.ground_y - origin.y
        } else {
            requested.y
        };
        let standing = origin.y <= self.ground_y;
        CollisionProbe {
            actual_displacement: Vec3::new(requested.x, actual_y, requested.z),
            old_box_collision_free: origin.y >= self.ground_y,
            introduced_collision: target_y < self.ground_y,
            supporting_collision_before: standing,
            nearby_block_below: origin.y <= self.ground_y + 0.55,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_world_clips_downward_motion_at_feet_height() {
        let world = FlatWorldCollision { ground_y: 65.0 };
        let probe =
            world.probe_player_movement(Vec3::new(1.0, 66.0, 2.0), Vec3::new(0.0, -2.0, 0.0));
        assert_eq!(probe.actual_displacement.y, -1.0);
        assert!(probe.introduced_collision);
    }
}
