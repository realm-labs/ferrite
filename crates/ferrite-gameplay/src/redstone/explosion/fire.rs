//! Post-block explosion fire draws and resulting-world admission.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

pub const FIRE_RANDOM_BOUND: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireCandidate {
    pub position: BlockPos,
    pub current_is_air: bool,
    pub below_is_solid_render: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirePlan {
    pub random_draws: usize,
    pub writes: Vec<BlockPos>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FirePlanError {
    #[error("explosion fire needs one bounded draw for every sampled position")]
    MissingBoundedDraw,
    #[error("fire draw {draw} is outside the exclusive bound {FIRE_RANDOM_BOUND}")]
    DrawOutOfRange { draw: u32 },
}

pub fn plan_fire(
    candidates: &[FireCandidate],
    bounded_draws: &[u32],
) -> Result<FirePlan, FirePlanError> {
    if bounded_draws.len() < candidates.len() {
        return Err(FirePlanError::MissingBoundedDraw);
    }
    let mut writes = Vec::new();
    for (candidate, draw) in candidates.iter().zip(bounded_draws.iter().copied()) {
        if draw >= FIRE_RANDOM_BOUND {
            return Err(FirePlanError::DrawOutOfRange { draw });
        }
        if draw == 0 && candidate.current_is_air && candidate.below_is_solid_render {
            writes.push(candidate.position);
        }
    }
    Ok(FirePlan {
        random_draws: candidates.len(),
        writes,
    })
}
