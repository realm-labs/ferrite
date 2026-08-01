//! Bounded player-collision snapshots from committed authoritative chunks.

use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_gameplay::player::collision::{
    Aabb, CollisionProbe, CollisionScene, CollisionWorld, SceneCollisionWorld, player_bounds,
};
use ferrite_gameplay::player::state::{PlayerSessionState, Vec3};
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use ferrite_world::id::BlockStateId;
use ferrite_world::projection::ChunkSnapshot;

use crate::composite::gateway::{CompositeGatewayError, CompositeRegionRouter};

const QUERY_MARGIN: f64 = 1.0e-7;
const SUPPORT_QUERY_DEPTH: f64 = 0.55;
const STEP_QUERY_HEIGHT: f64 = 0.6;
const MAX_QUERY_CELLS: usize = 65_536;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum AuthoritativePlayerCollision {
    Scene(SceneCollisionWorld),
    Unavailable,
    NotMovement,
}

impl AuthoritativePlayerCollision {
    pub(super) fn capture(
        router: &CompositeRegionRouter,
        state: &PlayerSessionState,
        packet: &PlayServerboundEntryPacket,
    ) -> Result<Self, CompositeGatewayError> {
        let Some(target) = movement_target(packet, state.last_good_position()) else {
            return Ok(Self::NotMovement);
        };
        let origin = state.last_good_position();
        if !finite(origin) || !finite(target) {
            return Ok(Self::Unavailable);
        }
        let Some(query) = CollisionQuery::new(origin, target) else {
            return Ok(Self::Unavailable);
        };
        let snapshots = router.projectable_world_snapshots(query.chunks())?;
        Ok(Self::from_snapshots(query, &snapshots))
    }

    fn from_snapshots(
        query: CollisionQuery,
        snapshots: &BTreeMap<ChunkPos, ChunkSnapshot>,
    ) -> Self {
        if query
            .chunks()
            .any(|position| !snapshots.contains_key(&position))
        {
            return Self::Unavailable;
        }
        let Some(layout) = snapshots.values().next().map(ChunkSnapshot::layout) else {
            return Self::Unavailable;
        };
        if snapshots
            .values()
            .any(|snapshot| snapshot.layout() != layout)
        {
            return Self::Unavailable;
        }
        let minimum_y = layout.sections().minimum().saturating_mul(16);
        let maximum_y = layout.sections().maximum_exclusive().saturating_mul(16);
        if query.minimum.y < minimum_y || query.maximum.y >= maximum_y {
            return Self::Unavailable;
        }

        let mut shapes = Vec::new();
        for y in query.minimum.y..=query.maximum.y {
            for z in query.minimum.z..=query.maximum.z {
                for x in query.minimum.x..=query.maximum.x {
                    let position = BlockPos::new(x, y, z);
                    let Some(snapshot) = snapshots.get(&position.chunk()) else {
                        return Self::Unavailable;
                    };
                    let Some(state) = snapshot_block_state(snapshot, position) else {
                        return Self::Unavailable;
                    };
                    if let Some(shape) = collision_shape(state, position) {
                        shapes.push(shape);
                    }
                }
            }
        }
        Self::Scene(SceneCollisionWorld::new(CollisionScene {
            block_shapes: shapes,
            ..CollisionScene::default()
        }))
    }
}

impl CollisionWorld for AuthoritativePlayerCollision {
    fn probe_player_movement(&self, origin: Vec3, requested: Vec3) -> CollisionProbe {
        match self {
            Self::Scene(scene) => scene.probe_player_movement(origin, requested),
            Self::NotMovement => CollisionProbe::unobstructed(requested),
            Self::Unavailable => CollisionProbe {
                actual_displacement: Vec3::ZERO,
                old_box_collision_free: false,
                introduced_collision: true,
                supporting_collision_before: true,
                nearby_block_below: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CollisionQuery {
    minimum: BlockPos,
    maximum: BlockPos,
}

impl CollisionQuery {
    fn new(origin: Vec3, target: Vec3) -> Option<Self> {
        if !finite(origin) || !finite(target) {
            return None;
        }
        let requested = target.subtract(origin);
        let bounds = player_bounds(origin).expand_towards(requested).inflate(
            QUERY_MARGIN,
            SUPPORT_QUERY_DEPTH,
            QUERY_MARGIN,
        );
        let minimum = BlockPos::new(
            floor_i32(bounds.min.x)?,
            floor_i32(bounds.min.y)?,
            floor_i32(bounds.min.z)?,
        );
        let maximum = BlockPos::new(
            floor_i32(bounds.max.x + QUERY_MARGIN)?,
            floor_i32(bounds.max.y + STEP_QUERY_HEIGHT + QUERY_MARGIN)?,
            floor_i32(bounds.max.z + QUERY_MARGIN)?,
        );
        let cells = inclusive_length(minimum.x, maximum.x)?
            .checked_mul(inclusive_length(minimum.y, maximum.y)?)?
            .checked_mul(inclusive_length(minimum.z, maximum.z)?)?;
        (cells <= MAX_QUERY_CELLS).then_some(Self { minimum, maximum })
    }

    fn chunks(self) -> impl Iterator<Item = ChunkPos> {
        let minimum_x = self.minimum.x.div_euclid(16);
        let maximum_x = self.maximum.x.div_euclid(16);
        let minimum_z = self.minimum.z.div_euclid(16);
        let maximum_z = self.maximum.z.div_euclid(16);
        let mut chunks = BTreeSet::new();
        for z in minimum_z..=maximum_z {
            for x in minimum_x..=maximum_x {
                chunks.insert(ChunkPos::new(x, z));
            }
        }
        chunks.into_iter()
    }
}

fn movement_target(packet: &PlayServerboundEntryPacket, current: Vec3) -> Option<Vec3> {
    let position = match packet {
        PlayServerboundEntryPacket::MovePlayerPosition(packet) => packet.position,
        PlayServerboundEntryPacket::MovePlayerPositionRotation(packet) => packet.position,
        PlayServerboundEntryPacket::MovePlayerRotation(_)
        | PlayServerboundEntryPacket::MovePlayerStatusOnly(_) => return Some(current),
        _ => return None,
    };
    Some(Vec3::new(position.x, position.y, position.z))
}

fn snapshot_block_state(snapshot: &ChunkSnapshot, position: BlockPos) -> Option<BlockStateId> {
    if snapshot.position() != position.chunk() {
        return None;
    }
    let section_y = position.section().y;
    let sections = snapshot.layout().sections();
    if !sections.contains(section_y) {
        return None;
    }
    let index = usize::try_from(section_y - sections.minimum()).ok()?;
    Some(snapshot.sections().get(index)?.block(position.local()))
}

fn collision_shape(state: BlockStateId, position: BlockPos) -> Option<Aabb> {
    if ferrite_world::id::has_empty_collision(state) {
        return None;
    }
    let minimum = Vec3::new(
        f64::from(position.x),
        f64::from(position.y),
        f64::from(position.z),
    );
    Some(Aabb::new(
        minimum,
        Vec3::new(minimum.x + 1.0, minimum.y + 1.0, minimum.z + 1.0),
    ))
}

fn finite(position: Vec3) -> bool {
    position.x.is_finite() && position.y.is_finite() && position.z.is_finite()
}

fn floor_i32(value: f64) -> Option<i32> {
    let value = value.floor();
    (value >= f64::from(i32::MIN) && value <= f64::from(i32::MAX)).then_some(value as i32)
}

fn inclusive_length(minimum: i32, maximum: i32) -> Option<usize> {
    let difference = i64::from(maximum)
        .checked_sub(i64::from(minimum))?
        .checked_add(1)?;
    usize::try_from(difference).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_world::chunk::{ChunkColumn, ChunkLayout, VerticalSectionRange};
    use ferrite_world::id::BiomeId;
    use ferrite_world::projection::LightSnapshot;

    fn snapshot_with_wall() -> ChunkSnapshot {
        let layout = ChunkLayout::new(
            VerticalSectionRange::new(3, 2).unwrap(),
            BlockStateId::new(0),
            BiomeId::new(0),
        );
        let mut chunk = ChunkColumn::new(ChunkPos::new(0, 0), layout);
        chunk
            .set_block(BlockPos::new(1, 65, 0), BlockStateId::new(2))
            .unwrap();
        chunk
            .set_uniform_section(3, BlockStateId::new(1), BiomeId::new(0))
            .unwrap();
        chunk
            .snapshot(
                LightSnapshot::full_sky(layout.sections().count()).unwrap(),
                |_, state| state != BlockStateId::new(0),
            )
            .unwrap()
    }

    #[test]
    fn authoritative_states_clip_a_wall_and_missing_chunks_fail_closed() {
        let origin = Vec3::new(0.5, 64.0, 0.5);
        let target = Vec3::new(1.5, 64.0, 0.5);
        let query = CollisionQuery::new(origin, target).unwrap();
        let snapshots = BTreeMap::from([(ChunkPos::new(0, 0), snapshot_with_wall())]);
        let collision = AuthoritativePlayerCollision::from_snapshots(query, &snapshots);
        let probe = collision.probe_player_movement(origin, target.subtract(origin));
        assert!((probe.actual_displacement.x - 0.2).abs() < QUERY_MARGIN);
        assert!(probe.introduced_collision);

        let unavailable = AuthoritativePlayerCollision::from_snapshots(query, &BTreeMap::new());
        let probe = unavailable.probe_player_movement(origin, target.subtract(origin));
        assert_eq!(probe.actual_displacement, Vec3::ZERO);
        assert!(probe.introduced_collision);
    }

    #[test]
    fn oversized_and_nonfinite_queries_are_rejected_before_iteration() {
        assert!(CollisionQuery::new(Vec3::ZERO, Vec3::new(10_000.0, 0.0, 0.0)).is_none());
        assert!(CollisionQuery::new(Vec3::ZERO, Vec3::new(f64::NAN, 0.0, 0.0)).is_none());
    }
}
