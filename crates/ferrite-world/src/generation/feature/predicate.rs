//! Ordered block-predicate evaluation without placement RNG.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::resource::ResourceId;
use thiserror::Error;

use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredicateOffset {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl PredicateOffset {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub const fn new(x: i8, y: i8, z: i8) -> Result<Self, PredicateError> {
        if x < -16 || x > 16 || y < -16 || y > 16 || z < -16 || z > 16 {
            Err(PredicateError::OffsetOutOfRange)
        } else {
            Ok(Self { x, y, z })
        }
    }

    fn apply(self, origin: BlockPos) -> Result<BlockPos, PredicateError> {
        Ok(BlockPos::new(
            origin
                .x
                .checked_add(i32::from(self.x))
                .ok_or(PredicateError::PositionOverflow)?,
            origin
                .y
                .checked_add(i32::from(self.y))
                .ok_or(PredicateError::PositionOverflow)?,
            origin
                .z
                .checked_add(i32::from(self.z))
                .ok_or(PredicateError::PositionOverflow)?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockPredicate {
    MatchingBlocks {
        offset: PredicateOffset,
        blocks: Vec<BlockStateId>,
    },
    MatchingTag {
        offset: PredicateOffset,
        tag: ResourceId,
    },
    MatchingFluid {
        offset: PredicateOffset,
        fluids: Vec<ResourceId>,
    },
    Replaceable {
        offset: PredicateOffset,
    },
    Solid {
        offset: PredicateOffset,
    },
    WouldSurvive {
        offset: PredicateOffset,
        state: BlockStateId,
    },
    AlwaysTrue,
    Not(Box<Self>),
    AllOf(Vec<Self>),
    AnyOf(Vec<Self>),
}

impl BlockPredicate {
    pub fn test(
        &self,
        world: &impl PredicateWorld,
        origin: BlockPos,
    ) -> Result<bool, PredicateError> {
        match self {
            Self::MatchingBlocks { offset, blocks } => {
                Ok(blocks.contains(&world.block_state(offset.apply(origin)?)))
            }
            Self::MatchingTag { offset, tag } => {
                let state = world.block_state(offset.apply(origin)?);
                Ok(world.block_in_tag(state, tag))
            }
            Self::MatchingFluid { offset, fluids } => {
                let state = world.block_state(offset.apply(origin)?);
                Ok(fluids.iter().any(|fluid| world.fluid_matches(state, fluid)))
            }
            Self::Replaceable { offset } => {
                let state = world.block_state(offset.apply(origin)?);
                Ok(world.can_replace(state))
            }
            Self::Solid { offset } => {
                let state = world.block_state(offset.apply(origin)?);
                Ok(world.is_solid(state))
            }
            Self::WouldSurvive { offset, state } => {
                Ok(world.would_survive(*state, offset.apply(origin)?))
            }
            Self::AlwaysTrue => Ok(true),
            Self::Not(child) => Ok(!child.test(world, origin)?),
            Self::AllOf(children) => {
                for child in children {
                    if !child.test(world, origin)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Self::AnyOf(children) => {
                for child in children {
                    if child.test(world, origin)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
}

pub trait PredicateWorld {
    fn block_state(&self, position: BlockPos) -> BlockStateId;

    fn block_in_tag(&self, state: BlockStateId, tag: &ResourceId) -> bool;

    fn fluid_matches(&self, state: BlockStateId, fluid: &ResourceId) -> bool;

    fn can_replace(&self, state: BlockStateId) -> bool;

    fn is_solid(&self, state: BlockStateId) -> bool;

    fn would_survive(&self, state: BlockStateId, position: BlockPos) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PredicateError {
    #[error("block-predicate offset must be within -16..=16 on every axis")]
    OffsetOutOfRange,
    #[error("block-predicate position arithmetic overflowed")]
    PositionOverflow,
}
