//! Client-predicted automatic-jump detection over contextual collision geometry.

use crate::player::collision::Aabb;
use crate::player::input::Vec2;
use crate::player::state::Vec3;

const DEG_TO_RADIANS: f32 = 0.017_453_292;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[must_use]
    pub fn containing(position: Vec3) -> Self {
        Self {
            x: floor_to_i32(position.x),
            y: floor_to_i32(position.y),
            z: floor_to_i32(position.z),
        }
    }

    #[must_use]
    pub const fn above(self, amount: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + amount,
            z: self.z,
        }
    }
}

pub trait AutoJumpWorld {
    /// Returns the contextual collision shape in world coordinates.
    fn collision_shape(&self, position: BlockPos) -> Option<Aabb>;

    /// Returned shapes must retain entity iteration and `toAabbs` order.
    fn entity_collision_shapes(&self, query: Aabb) -> Vec<Aabb>;

    /// Returned shapes must retain block traversal and `toAabbs` order.
    fn block_collision_shapes(&self, query: Aabb) -> Vec<Aabb>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoJumpContext {
    pub position: Vec3,
    pub bounds: Aabb,
    pub actual_horizontal: Vec2,
    pub raw_movement: Vec2,
    pub yaw: f32,
    pub pitch: f32,
    pub movement_speed: f32,
    pub on_ground: bool,
    pub stay_on_ground: bool,
    pub passenger: bool,
    pub block_jump_factor: f32,
    pub jump_boost_amplifier: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoJumpDecision {
    Rejected,
    Scheduled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoJumpState {
    pub option_cache: bool,
    pub timer: u8,
}

impl Default for AutoJumpState {
    fn default() -> Self {
        Self {
            option_cache: true,
            timer: 0,
        }
    }
}

impl AutoJumpState {
    pub fn detect(
        &mut self,
        context: AutoJumpContext,
        world: &impl AutoJumpWorld,
    ) -> AutoJumpDecision {
        if !self.option_cache
            || self.timer > 0
            || !context.on_ground
            || context.stay_on_ground
            || context.passenger
            || context.raw_movement.length_squared() <= 0.0
            || context.block_jump_factor < 1.0
        {
            return AutoJumpDecision::Rejected;
        }

        let Some((direction, inverse_length, actual)) = selected_direction(context) else {
            return AutoJumpDecision::Rejected;
        };
        let forward = look_forward(context.yaw, context.pitch);
        let dot = (direction.x * forward.x + direction.z * forward.z) as f32;
        if dot < -0.15 {
            return AutoJumpDecision::Rejected;
        }

        let head = BlockPos::containing(Vec3::new(
            context.position.x,
            context.bounds.max.y,
            context.position.z,
        ));
        if world.collision_shape(head).is_some() || world.collision_shape(head.above(1)).is_some() {
            return AutoJumpDecision::Rejected;
        }

        let maximum_rise = 1.2_f32
            + context.jump_boost_amplifier.map_or(0.0, |amplifier| {
                0.75_f32 * f32::from(amplifier.saturating_add(1))
            });
        let look_ahead = (context.movement_speed * 7.0_f32).max(1.0_f32 / inverse_length);
        let start = context.position;
        let base_end = start.add(actual);
        let end = base_end.add(direction.scale(f64::from(look_ahead)));
        let width = context.bounds.width();
        let height = context.bounds.height();
        let query =
            spanning_box(start, end.add(Vec3::new(0.0, height, 0.0))).inflate(width, 0.0, width);
        let probe_start = start.add(Vec3::new(0.0, 0.509_999_990_463_256_8, 0.0));
        let probe_end = end.add(Vec3::new(0.0, 0.509_999_990_463_256_8, 0.0));
        let perpendicular = direction.cross(Vec3::new(0.0, 1.0, 0.0));
        let offset = perpendicular.scale(f64::from(width as f32 * 0.5_f32));
        let left_start = probe_start.subtract(offset);
        let left_end = probe_end.subtract(offset);
        let right_start = probe_start.add(offset);
        let right_end = probe_end.add(offset);

        let mut obstacle_top = f32::MIN_POSITIVE;
        let ordered = world
            .entity_collision_shapes(query)
            .into_iter()
            .chain(world.block_collision_shapes(query));
        for shape in ordered {
            if !shape.intersects_segment(left_start, left_end)
                && !shape.intersects_segment(right_start, right_end)
            {
                continue;
            }
            obstacle_top = shape.max.y as f32;
            let center = BlockPos::containing(shape.center());
            let mut height_index = 1;
            while (height_index as f32) < maximum_rise {
                if let Some(overhead) = world.collision_shape(center.above(height_index)) {
                    obstacle_top = overhead.max.y as f32;
                    if f64::from(obstacle_top) - context.position.y > f64::from(maximum_rise) {
                        return AutoJumpDecision::Rejected;
                    }
                }
                if height_index > 1 && world.collision_shape(head.above(height_index)).is_some() {
                    return AutoJumpDecision::Rejected;
                }
                height_index += 1;
            }
        }
        if obstacle_top == f32::MIN_POSITIVE {
            return AutoJumpDecision::Rejected;
        }
        let rise = (f64::from(obstacle_top) - context.position.y) as f32;
        if rise > 0.5_f32 && rise <= maximum_rise {
            self.timer = 1;
            AutoJumpDecision::Scheduled
        } else {
            AutoJumpDecision::Rejected
        }
    }

    #[must_use]
    pub fn consume(&mut self, sampled_jump: bool) -> bool {
        if self.timer == 0 {
            sampled_jump
        } else {
            self.timer -= 1;
            true
        }
    }

    pub const fn refresh_option_cache(&mut self, enabled: bool) {
        self.option_cache = enabled;
    }
}

fn selected_direction(context: AutoJumpContext) -> Option<(Vec3, f32, Vec3)> {
    let mut actual = Vec3::new(
        f64::from(context.actual_horizontal.x),
        0.0,
        f64::from(context.actual_horizontal.y),
    );
    let mut squared = actual.length_squared() as f32;
    if squared <= 0.001_f32 || squared.is_nan() {
        let strafe = context.movement_speed * context.raw_movement.x;
        let forward = context.movement_speed * context.raw_movement.y;
        let angle = context.yaw * DEG_TO_RADIANS;
        let sine = angle.sin();
        let cosine = angle.cos();
        actual = Vec3::new(
            f64::from(strafe * cosine - forward * sine),
            0.0,
            f64::from(forward * cosine + strafe * sine),
        );
        squared = actual.length_squared() as f32;
    }
    if squared <= 0.001_f32 || squared.is_nan() {
        return None;
    }
    let inverse_length = 1.0_f32 / squared.sqrt();
    Some((
        actual.scale(f64::from(inverse_length)),
        inverse_length,
        actual,
    ))
}

fn look_forward(yaw: f32, pitch: f32) -> Vec3 {
    let yaw = yaw * DEG_TO_RADIANS;
    let pitch = pitch * DEG_TO_RADIANS;
    let horizontal = pitch.cos();
    Vec3::new(
        f64::from(-yaw.sin() * horizontal),
        f64::from(-pitch.sin()),
        f64::from(yaw.cos() * horizontal),
    )
}

fn spanning_box(first: Vec3, second: Vec3) -> Aabb {
    Aabb::new(
        Vec3::new(
            first.x.min(second.x),
            first.y.min(second.y),
            first.z.min(second.z),
        ),
        Vec3::new(
            first.x.max(second.x),
            first.y.max(second.y),
            first.z.max(second.z),
        ),
    )
}

fn floor_to_i32(value: f64) -> i32 {
    let floored = value.floor();
    if floored < f64::from(i32::MIN) {
        i32::MIN
    } else if floored > f64::from(i32::MAX) {
        i32::MAX
    } else {
        floored as i32
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[derive(Default)]
    struct World {
        contextual: BTreeMap<BlockPos, Aabb>,
        entities: Vec<Aabb>,
        blocks: Vec<Aabb>,
    }

    impl AutoJumpWorld for World {
        fn collision_shape(&self, position: BlockPos) -> Option<Aabb> {
            self.contextual.get(&position).copied()
        }

        fn entity_collision_shapes(&self, _query: Aabb) -> Vec<Aabb> {
            self.entities.clone()
        }

        fn block_collision_shapes(&self, _query: Aabb) -> Vec<Aabb> {
            self.blocks.clone()
        }
    }

    fn context() -> AutoJumpContext {
        AutoJumpContext {
            position: Vec3::new(0.0, 65.0, 0.0),
            bounds: Aabb::new(Vec3::new(-0.3, 65.0, -0.3), Vec3::new(0.3, 66.8, 0.3)),
            actual_horizontal: Vec2::new(0.0, 0.2),
            raw_movement: Vec2::new(0.0, 1.0),
            yaw: 0.0,
            pitch: 0.0,
            movement_speed: 0.1,
            on_ground: true,
            stay_on_ground: false,
            passenger: false,
            block_jump_factor: 1.0,
            jump_boost_amplifier: None,
        }
    }

    #[test]
    fn ordered_last_intersection_sets_rise_and_consumes_next_input() {
        let world = World {
            entities: vec![Aabb::new(
                Vec3::new(-0.5, 65.0, 0.4),
                Vec3::new(0.5, 65.8, 0.7),
            )],
            blocks: vec![Aabb::new(
                Vec3::new(-0.5, 65.0, 0.8),
                Vec3::new(0.5, 66.0, 1.1),
            )],
            ..World::default()
        };
        let mut state = AutoJumpState::default();
        assert_eq!(state.detect(context(), &world), AutoJumpDecision::Scheduled);
        assert_eq!(state.timer, 1);
        assert!(state.consume(false));
        assert_eq!(state.timer, 0);
        assert!(!state.consume(false));
    }

    #[test]
    fn exact_half_block_and_backward_view_are_rejected() {
        let world = World {
            blocks: vec![Aabb::new(
                Vec3::new(-0.5, 65.0, 0.4),
                Vec3::new(0.5, 65.5, 0.7),
            )],
            ..World::default()
        };
        let mut state = AutoJumpState::default();
        assert_eq!(state.detect(context(), &world), AutoJumpDecision::Rejected);
        let mut backwards = context();
        backwards.yaw = 180.0;
        assert_eq!(state.detect(backwards, &world), AutoJumpDecision::Rejected);
    }
}
