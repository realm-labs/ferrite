//! Strict client block/entity pick comparison and range filtering.

use crate::player::interaction::HitTarget;
use crate::player::state::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickCandidate {
    pub target: HitTarget,
    pub distance_squared: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickRanges {
    pub block: f64,
    pub entity: f64,
}

#[must_use]
pub fn select_pick(
    eye: Vec3,
    block: Option<PickCandidate>,
    entities_in_swept_order: &[PickCandidate],
    ranges: PickRanges,
) -> HitTarget {
    let mut selected = block;
    let block_limit = block.map_or(ranges.block.max(ranges.entity).powi(2), |candidate| {
        candidate.distance_squared
    });
    let mut nearest_entity_distance = block_limit;
    for candidate in entities_in_swept_order {
        if !matches!(candidate.target, HitTarget::Entity(_)) {
            continue;
        }
        if candidate.distance_squared < nearest_entity_distance {
            selected = Some(*candidate);
            nearest_entity_distance = candidate.distance_squared;
        }
    }
    let Some(selected) = selected else {
        return HitTarget::Miss { location: eye };
    };
    let range = match selected.target {
        HitTarget::Entity(_) => ranges.entity,
        HitTarget::Block(_) | HitTarget::Miss { .. } => ranges.block,
    };
    if selected.distance_squared < range * range {
        selected.target
    } else {
        HitTarget::Miss {
            location: selected.target.location(),
        }
    }
}

#[must_use]
pub fn select_with_attack_range(
    custom: Option<PickCandidate>,
    ordinary: HitTarget,
    block_range: f64,
) -> HitTarget {
    let Some(custom) = custom else {
        return ordinary;
    };
    match custom.target {
        HitTarget::Miss { .. } => ordinary,
        HitTarget::Block(_) if custom.distance_squared >= block_range * block_range => ordinary,
        target => target,
    }
}

#[cfg(test)]
mod tests {
    use ferrite_foundation::coordinate::BlockPos;
    use ferrite_foundation::direction::Direction;

    use crate::player::interaction::{BlockHit, EntityHit};

    use super::*;

    #[test]
    fn exact_entity_block_tie_keeps_block_and_exact_range_becomes_miss() {
        let eye = Vec3::ZERO;
        let block = PickCandidate {
            target: HitTarget::Block(BlockHit {
                position: BlockPos::default(),
                location: Vec3::new(3.0, 0.0, 0.0),
                face: Direction::East,
            }),
            distance_squared: 9.0,
        };
        let entity = PickCandidate {
            target: HitTarget::Entity(EntityHit {
                entity_id: 1,
                location: Vec3::new(3.0, 0.0, 0.0),
                relative_location: Vec3::ZERO,
            }),
            distance_squared: 9.0,
        };
        assert!(matches!(
            select_pick(
                eye,
                Some(block),
                &[entity],
                PickRanges {
                    block: 4.5,
                    entity: 3.0
                }
            ),
            HitTarget::Block(_)
        ));
        assert!(matches!(
            select_pick(
                eye,
                None,
                &[entity],
                PickRanges {
                    block: 4.5,
                    entity: 3.0
                }
            ),
            HitTarget::Miss { .. }
        ));
    }
}
