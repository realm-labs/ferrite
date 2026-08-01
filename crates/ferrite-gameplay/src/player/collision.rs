//! Generic player/entity collision projection and source-ordered movement clipping.

use crate::player::state::Vec3;

const SHAPE_EPSILON: f64 = 1.0e-7;
const MOVEMENT_EQUALITY_EPSILON: f64 = 9.999_999_747_378_752e-6;
const EDGE_BACKOFF_STEP: f64 = 0.05;
const PLAYER_HALF_WIDTH: f64 = 0.3;
const PLAYER_HEIGHT: f64 = 1.8;
const PLAYER_MAX_UP_STEP: f64 = 0.6;
const PLAYER_NEARBY_BELOW: f64 = 0.55;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    #[must_use]
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[must_use]
    pub const fn move_by(self, movement: Vec3) -> Self {
        Self::new(self.min.add(movement), self.max.add(movement))
    }

    #[must_use]
    pub const fn expand_towards(self, movement: Vec3) -> Self {
        Self::new(
            Vec3::new(
                if movement.x < 0.0 {
                    self.min.x + movement.x
                } else {
                    self.min.x
                },
                if movement.y < 0.0 {
                    self.min.y + movement.y
                } else {
                    self.min.y
                },
                if movement.z < 0.0 {
                    self.min.z + movement.z
                } else {
                    self.min.z
                },
            ),
            Vec3::new(
                if movement.x > 0.0 {
                    self.max.x + movement.x
                } else {
                    self.max.x
                },
                if movement.y > 0.0 {
                    self.max.y + movement.y
                } else {
                    self.max.y
                },
                if movement.z > 0.0 {
                    self.max.z + movement.z
                } else {
                    self.max.z
                },
            ),
        )
    }

    #[must_use]
    pub const fn inflate(self, x: f64, y: f64, z: f64) -> Self {
        Self::new(
            Vec3::new(self.min.x - x, self.min.y - y, self.min.z - z),
            Vec3::new(self.max.x + x, self.max.y + y, self.max.z + z),
        )
    }

    #[must_use]
    pub const fn center(self) -> Vec3 {
        Vec3::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
            (self.min.z + self.max.z) * 0.5,
        )
    }

    #[must_use]
    pub const fn width(self) -> f64 {
        self.max.x - self.min.x
    }

    #[must_use]
    pub const fn height(self) -> f64 {
        self.max.y - self.min.y
    }

    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.max.x > other.min.x
            && self.min.x < other.max.x
            && self.max.y > other.min.y
            && self.min.y < other.max.y
            && self.max.z > other.min.z
            && self.min.z < other.max.z
    }

    #[must_use]
    pub fn intersects_segment(self, start: Vec3, end: Vec3) -> bool {
        let delta = end.subtract(start);
        let mut near = 0.0_f64;
        let mut far = 1.0_f64;
        for (origin, direction, minimum, maximum) in [
            (start.x, delta.x, self.min.x, self.max.x),
            (start.y, delta.y, self.min.y, self.max.y),
            (start.z, delta.z, self.min.z, self.max.z),
        ] {
            if direction.abs() < f64::EPSILON {
                if origin < minimum || origin > maximum {
                    return false;
                }
                continue;
            }
            let inverse = 1.0 / direction;
            let first = (minimum - origin) * inverse;
            let second = (maximum - origin) * inverse;
            near = near.max(first.min(second));
            far = far.min(first.max(second));
            if near > far {
                return false;
            }
        }
        true
    }
}

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

/// Already-flattened collision shapes in the exact entity, border, then block order.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CollisionScene {
    pub entity_shapes: Vec<Aabb>,
    pub world_border_shape: Option<Aabb>,
    pub block_shapes: Vec<Aabb>,
}

impl CollisionScene {
    fn ordered_shapes(&self) -> impl Iterator<Item = Aabb> + '_ {
        self.entity_shapes
            .iter()
            .copied()
            .chain(self.world_border_shape)
            .chain(self.block_shapes.iter().copied())
    }

    #[must_use]
    pub fn collision_free(&self, bounds: Aabb) -> bool {
        !self.ordered_shapes().any(|shape| shape.intersects(bounds))
    }
}

/// Immutable collision geometry captured from an authoritative world view.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneCollisionWorld {
    scene: CollisionScene,
}

impl SceneCollisionWorld {
    #[must_use]
    pub const fn new(scene: CollisionScene) -> Self {
        Self { scene }
    }

    #[must_use]
    pub const fn scene(&self) -> &CollisionScene {
        &self.scene
    }
}

impl CollisionWorld for SceneCollisionWorld {
    fn probe_player_movement(&self, origin: Vec3, requested: Vec3) -> CollisionProbe {
        let bounds = player_bounds(origin);
        let supporting_collision_before = has_support(bounds, 0.0, 0.0, SHAPE_EPSILON, &self.scene);
        let target = bounds.move_by(requested);
        let introduced_collision = self
            .scene
            .ordered_shapes()
            .any(|shape| shape.intersects(target) && !shape.intersects(bounds));
        CollisionProbe {
            actual_displacement: collide(
                requested,
                bounds,
                PLAYER_MAX_UP_STEP,
                supporting_collision_before,
                &self.scene,
            ),
            old_box_collision_free: self.scene.collision_free(bounds),
            introduced_collision,
            supporting_collision_before,
            nearby_block_below: has_support(bounds, 0.0, 0.0, PLAYER_NEARBY_BELOW, &self.scene),
        }
    }
}

#[must_use]
pub const fn player_bounds(feet: Vec3) -> Aabb {
    Aabb::new(
        Vec3::new(
            feet.x - PLAYER_HALF_WIDTH,
            feet.y,
            feet.z - PLAYER_HALF_WIDTH,
        ),
        Vec3::new(
            feet.x + PLAYER_HALF_WIDTH,
            feet.y + PLAYER_HEIGHT,
            feet.z + PLAYER_HALF_WIDTH,
        ),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoverType {
    SelfMovement,
    Player,
    Piston,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementRecord {
    pub from: Vec3,
    pub to: Vec3,
    pub requested: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityMotion {
    pub position: Vec3,
    pub bounds: Aabb,
    pub velocity: Vec3,
    pub stuck_speed_multiplier: Vec3,
    pub piston_deltas: Vec3,
    pub piston_tick: i64,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    pub movement_records: Vec<MovementRecord>,
}

impl EntityMotion {
    #[must_use]
    pub fn new(position: Vec3, bounds: Aabb) -> Self {
        Self {
            position,
            bounds,
            velocity: Vec3::ZERO,
            stuck_speed_multiplier: Vec3::ZERO,
            piston_deltas: Vec3::ZERO,
            piston_tick: i64::MIN,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            movement_records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveContext {
    pub mover_type: MoverType,
    pub game_time: i64,
    pub no_physics: bool,
    pub locally_authoritative: bool,
    pub on_ground: bool,
    pub stay_on_ground: bool,
    pub flying: bool,
    pub fall_distance: f32,
    pub max_up_step: f32,
    pub block_speed_factor: f32,
    pub entity_bounciness: f32,
    pub block_bounce: f32,
    pub suppress_bounce: bool,
    pub effective_gravity: f64,
}

impl Default for MoveContext {
    fn default() -> Self {
        Self {
            mover_type: MoverType::SelfMovement,
            game_time: 0,
            no_physics: false,
            locally_authoritative: true,
            on_ground: false,
            stay_on_ground: false,
            flying: false,
            fall_distance: 0.0,
            max_up_step: 0.6,
            block_speed_factor: 1.0,
            entity_bounciness: 0.0,
            block_bounce: 0.0,
            suppress_bounce: false,
            effective_gravity: 0.08,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveResult {
    pub requested: Vec3,
    pub actual: Vec3,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    pub bounced: bool,
}

pub fn move_entity(
    motion: &mut EntityMotion,
    mut requested: Vec3,
    context: MoveContext,
    scene: &CollisionScene,
) -> MoveResult {
    if context.no_physics {
        install_position(motion, requested);
        motion.horizontal_collision = false;
        motion.vertical_collision = false;
        motion.vertical_collision_below = false;
        return result(requested, requested, false, false, false, false);
    }
    if context.mover_type == MoverType::Piston {
        requested = limit_piston_movement(motion, requested, context.game_time);
        if requested == Vec3::ZERO {
            return result(requested, Vec3::ZERO, false, false, false, false);
        }
    } else if motion.stuck_speed_multiplier.length_squared() > SHAPE_EPSILON {
        requested = requested.multiply(motion.stuck_speed_multiplier);
        motion.stuck_speed_multiplier = Vec3::ZERO;
        motion.velocity = Vec3::ZERO;
    }
    if context.stay_on_ground
        && !context.flying
        && requested.y <= 0.0
        && matches!(
            context.mover_type,
            MoverType::SelfMovement | MoverType::Player
        )
    {
        requested = back_off_from_edge(
            motion.bounds,
            requested,
            f64::from(context.max_up_step),
            context.on_ground,
            context.fall_distance,
            scene,
        );
    }

    let actual = collide(
        requested,
        motion.bounds,
        f64::from(context.max_up_step),
        context.on_ground,
        scene,
    );
    if actual.length_squared() > SHAPE_EPSILON
        || requested.length_squared() - actual.length_squared() < SHAPE_EPSILON
    {
        record_movement(motion, requested, actual);
        install_position(motion, actual);
    }
    let x_clipped = !movement_equal(requested.x, actual.x);
    let z_clipped = !movement_equal(requested.z, actual.z);
    let horizontal_collision = x_clipped || z_clipped;
    let vertical_collision =
        (requested.y != 0.0 || context.locally_authoritative) && requested.y != actual.y;
    let vertical_collision_below = vertical_collision && requested.y < 0.0;
    motion.horizontal_collision = horizontal_collision;
    motion.vertical_collision = vertical_collision;
    motion.vertical_collision_below = vertical_collision_below;

    if x_clipped {
        motion.velocity.x = -motion.velocity.x * f64::from(context.entity_bounciness);
    }
    if z_clipped {
        motion.velocity.z = -motion.velocity.z * f64::from(context.entity_bounciness);
    }
    let bounced = if vertical_collision_below {
        restitute_vertical(motion, context)
    } else {
        if vertical_collision {
            motion.velocity.y = 0.0;
        }
        false
    };
    motion.velocity.x *= f64::from(context.block_speed_factor);
    motion.velocity.z *= f64::from(context.block_speed_factor);
    result(
        requested,
        actual,
        horizontal_collision,
        vertical_collision,
        vertical_collision_below,
        bounced,
    )
}

#[must_use]
pub fn collide(
    requested: Vec3,
    bounds: Aabb,
    max_up_step: f64,
    on_ground: bool,
    scene: &CollisionScene,
) -> Vec3 {
    if requested == Vec3::ZERO {
        return Vec3::ZERO;
    }
    let normal = clip_with_shapes(requested, bounds, scene);
    let horizontal_clipped = requested.x != normal.x || requested.z != normal.z;
    let downward_clipped = requested.y < 0.0 && requested.y != normal.y;
    if max_up_step <= 0.0 || !horizontal_clipped || (!downward_clipped && !on_ground) {
        return normal;
    }

    let base = if downward_clipped {
        bounds.move_by(Vec3::new(0.0, normal.y, 0.0))
    } else {
        bounds
    };
    let base_offset = base.min.y - bounds.min.y;
    let mut heights = candidate_step_heights(scene, base, normal.y as f32, max_up_step as f32);
    heights.sort_unstable_by(f32::total_cmp);
    for height in heights {
        let candidate = clip_with_shapes(
            Vec3::new(requested.x, f64::from(height), requested.z),
            base,
            scene,
        );
        if candidate.horizontal_length_squared() > normal.horizontal_length_squared() {
            return Vec3::new(candidate.x, candidate.y + base_offset, candidate.z);
        }
    }
    normal
}

#[must_use]
pub fn back_off_from_edge(
    bounds: Aabb,
    mut movement: Vec3,
    max_up_step: f64,
    on_ground: bool,
    fall_distance: f32,
    scene: &CollisionScene,
) -> Vec3 {
    if on_ground
        || (f64::from(fall_distance) < max_up_step
            && has_support(
                bounds,
                0.0,
                0.0,
                max_up_step - f64::from(fall_distance),
                scene,
            ))
    {
        return movement;
    }
    while movement.x != 0.0 && !has_support(bounds, movement.x, 0.0, max_up_step, scene) {
        movement.x = approach_zero(movement.x);
    }
    while movement.z != 0.0 && !has_support(bounds, 0.0, movement.z, max_up_step, scene) {
        movement.z = approach_zero(movement.z);
    }
    while movement.x != 0.0
        && movement.z != 0.0
        && !has_support(bounds, movement.x, movement.z, max_up_step, scene)
    {
        movement.x = approach_zero(movement.x);
        movement.z = approach_zero(movement.z);
    }
    movement
}

fn clip_with_shapes(requested: Vec3, bounds: Aabb, scene: &CollisionScene) -> Vec3 {
    let mut working = bounds;
    let mut accepted = requested;
    accepted.y = clip_axis(Axis::Y, working, scene, accepted.y);
    working = working.move_by(Vec3::new(0.0, accepted.y, 0.0));
    if requested.x.abs() < requested.z.abs() {
        accepted.z = clip_axis(Axis::Z, working, scene, accepted.z);
        working = working.move_by(Vec3::new(0.0, 0.0, accepted.z));
        accepted.x = clip_axis(Axis::X, working, scene, accepted.x);
    } else {
        accepted.x = clip_axis(Axis::X, working, scene, accepted.x);
        working = working.move_by(Vec3::new(accepted.x, 0.0, 0.0));
        accepted.z = clip_axis(Axis::Z, working, scene, accepted.z);
    }
    accepted
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

fn clip_axis(axis: Axis, bounds: Aabb, scene: &CollisionScene, mut amount: f64) -> f64 {
    for shape in scene.ordered_shapes() {
        if amount.abs() < SHAPE_EPSILON {
            return 0.0;
        }
        amount = clip_shape(axis, bounds, shape, amount);
    }
    amount
}

fn clip_shape(axis: Axis, bounds: Aabb, shape: Aabb, amount: f64) -> f64 {
    let overlaps = match axis {
        Axis::X => {
            overlaps(bounds.min.y, bounds.max.y, shape.min.y, shape.max.y)
                && overlaps(bounds.min.z, bounds.max.z, shape.min.z, shape.max.z)
        }
        Axis::Y => {
            overlaps(bounds.min.x, bounds.max.x, shape.min.x, shape.max.x)
                && overlaps(bounds.min.z, bounds.max.z, shape.min.z, shape.max.z)
        }
        Axis::Z => {
            overlaps(bounds.min.x, bounds.max.x, shape.min.x, shape.max.x)
                && overlaps(bounds.min.y, bounds.max.y, shape.min.y, shape.max.y)
        }
    };
    if !overlaps {
        return amount;
    }
    let (minimum, maximum, shape_minimum, shape_maximum) = match axis {
        Axis::X => (bounds.min.x, bounds.max.x, shape.min.x, shape.max.x),
        Axis::Y => (bounds.min.y, bounds.max.y, shape.min.y, shape.max.y),
        Axis::Z => (bounds.min.z, bounds.max.z, shape.min.z, shape.max.z),
    };
    if amount > 0.0 && maximum <= shape_minimum {
        amount.min(shape_minimum - maximum)
    } else if amount < 0.0 && minimum >= shape_maximum {
        amount.max(shape_maximum - minimum)
    } else {
        amount
    }
}

const fn overlaps(minimum: f64, maximum: f64, other_min: f64, other_max: f64) -> bool {
    maximum > other_min && minimum < other_max
}

fn candidate_step_heights(
    scene: &CollisionScene,
    base: Aabb,
    normal_y: f32,
    max_up_step: f32,
) -> Vec<f32> {
    let mut heights = Vec::new();
    for shape in scene.ordered_shapes() {
        for coordinate in [shape.min.y, shape.max.y] {
            let height = (coordinate - base.min.y) as f32;
            if height >= 0.0
                && height != normal_y
                && height <= max_up_step
                && !heights.contains(&height)
            {
                heights.push(height);
            }
        }
    }
    heights
}

fn has_support(bounds: Aabb, x: f64, z: f64, down_distance: f64, scene: &CollisionScene) -> bool {
    let test = Aabb::new(
        Vec3::new(
            bounds.min.x + SHAPE_EPSILON + x,
            bounds.min.y - down_distance - SHAPE_EPSILON,
            bounds.min.z + SHAPE_EPSILON + z,
        ),
        Vec3::new(
            bounds.max.x - SHAPE_EPSILON + x,
            bounds.max.y,
            bounds.max.z - SHAPE_EPSILON + z,
        ),
    );
    !scene.collision_free(test)
}

fn approach_zero(value: f64) -> f64 {
    if value.abs() <= EDGE_BACKOFF_STEP {
        0.0
    } else {
        value - EDGE_BACKOFF_STEP.copysign(value)
    }
}

fn limit_piston_movement(motion: &mut EntityMotion, requested: Vec3, game_time: i64) -> Vec3 {
    if motion.piston_tick != game_time {
        motion.piston_tick = game_time;
        motion.piston_deltas = Vec3::ZERO;
    }
    let (old, requested_axis, axis) = if requested.x != 0.0 {
        (motion.piston_deltas.x, requested.x, Axis::X)
    } else if requested.y != 0.0 {
        (motion.piston_deltas.y, requested.y, Axis::Y)
    } else {
        (motion.piston_deltas.z, requested.z, Axis::Z)
    };
    let new = (old + requested_axis).clamp(-0.51, 0.51);
    let permitted = new - old;
    match axis {
        Axis::X => motion.piston_deltas.x = new,
        Axis::Y => motion.piston_deltas.y = new,
        Axis::Z => motion.piston_deltas.z = new,
    }
    if permitted.abs() <= MOVEMENT_EQUALITY_EPSILON {
        Vec3::ZERO
    } else {
        match axis {
            Axis::X => Vec3::new(permitted, 0.0, 0.0),
            Axis::Y => Vec3::new(0.0, permitted, 0.0),
            Axis::Z => Vec3::new(0.0, 0.0, permitted),
        }
    }
}

fn record_movement(motion: &mut EntityMotion, requested: Vec3, actual: Vec3) {
    if motion.movement_records.len() >= 100 {
        let first = motion.movement_records.remove(0);
        let second = motion.movement_records.remove(0);
        motion.movement_records.insert(
            0,
            MovementRecord {
                from: first.from,
                to: second.to,
                requested: first.requested,
            },
        );
    }
    motion.movement_records.push(MovementRecord {
        from: motion.position,
        to: motion.position.add(actual),
        requested,
    });
}

fn install_position(motion: &mut EntityMotion, movement: Vec3) {
    motion.position = motion.position.add(movement);
    motion.bounds = motion.bounds.move_by(movement);
}

fn restitute_vertical(motion: &mut EntityMotion, context: MoveContext) -> bool {
    if context.suppress_bounce || -motion.velocity.y < context.effective_gravity {
        motion.velocity.y = 0.0;
        return false;
    }
    let restitution = context.entity_bounciness.max(context.block_bounce);
    if restitution <= 0.0 {
        motion.velocity.y = 0.0;
        false
    } else {
        motion.velocity.y = -motion.velocity.y * f64::from(restitution);
        true
    }
}

const fn movement_equal(left: f64, right: f64) -> bool {
    (left - right).abs() < MOVEMENT_EQUALITY_EPSILON
}

const fn result(
    requested: Vec3,
    actual: Vec3,
    horizontal_collision: bool,
    vertical_collision: bool,
    vertical_collision_below: bool,
    bounced: bool,
) -> MoveResult {
    MoveResult {
        requested,
        actual,
        horizontal_collision,
        vertical_collision,
        vertical_collision_below,
        bounced,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player_box() -> Aabb {
        Aabb::new(Vec3::new(-0.3, 65.0, -0.3), Vec3::new(0.3, 66.8, 0.3))
    }

    #[test]
    fn flat_world_clips_downward_motion_at_feet_height() {
        let world = FlatWorldCollision { ground_y: 65.0 };
        let probe =
            world.probe_player_movement(Vec3::new(1.0, 66.0, 2.0), Vec3::new(0.0, -2.0, 0.0));
        assert_eq!(probe.actual_displacement.y, -1.0);
        assert!(probe.introduced_collision);
    }

    #[test]
    fn collision_clips_y_before_equal_xz_and_selects_first_improving_step() {
        let scene = CollisionScene {
            block_shapes: vec![
                Aabb::new(Vec3::new(0.3, 64.0, -1.0), Vec3::new(1.0, 65.5, 1.0)),
                Aabb::new(Vec3::new(-1.0, 64.0, 0.8), Vec3::new(1.0, 66.0, 1.2)),
            ],
            ..CollisionScene::default()
        };
        let actual = collide(Vec3::new(1.0, 0.0, 1.0), player_box(), 0.6, true, &scene);
        assert_eq!(actual.y, 0.5);
        assert!(actual.x > 0.0);
    }

    #[test]
    fn scene_collision_world_clips_falls_and_rejects_new_walls() {
        let world = SceneCollisionWorld::new(CollisionScene {
            block_shapes: vec![
                Aabb::new(Vec3::new(-2.0, 64.0, -2.0), Vec3::new(2.0, 65.0, 2.0)),
                Aabb::new(Vec3::new(1.0, 65.0, 0.0), Vec3::new(2.0, 67.0, 1.0)),
            ],
            ..CollisionScene::default()
        });
        let fall =
            world.probe_player_movement(Vec3::new(0.5, 65.5, 0.5), Vec3::new(0.0, -1.0, 0.0));
        assert_eq!(fall.actual_displacement.y, -0.5);
        assert!(fall.nearby_block_below);

        let wall = world.probe_player_movement(Vec3::new(0.5, 65.0, 0.5), Vec3::new(1.0, 0.0, 0.0));
        assert!((wall.actual_displacement.x - 0.2).abs() < SHAPE_EPSILON);
        assert!(wall.introduced_collision);
        assert!(wall.supporting_collision_before);
    }

    #[test]
    fn piston_cap_and_record_compaction_are_source_bounded() {
        let mut motion = EntityMotion::new(Vec3::new(0.0, 65.0, 0.0), player_box());
        let scene = CollisionScene::default();
        let context = MoveContext {
            mover_type: MoverType::Piston,
            game_time: 1,
            ..MoveContext::default()
        };
        assert_eq!(
            move_entity(&mut motion, Vec3::new(1.0, 0.0, 0.0), context, &scene)
                .actual
                .x,
            0.51
        );
        assert_eq!(
            move_entity(&mut motion, Vec3::new(1.0, 0.0, 0.0), context, &scene).actual,
            Vec3::ZERO
        );
        for _ in 0..101 {
            move_entity(
                &mut motion,
                Vec3::new(0.01, 0.0, 0.0),
                MoveContext::default(),
                &scene,
            );
        }
        assert_eq!(motion.movement_records.len(), 100);
    }
}
