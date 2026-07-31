//! Bounded depth-first placed-feature modifier execution.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::feature::{predicate::PredicateError, provider::ProviderError};

pub trait PlacementModifier<R: GenerationRandom> {
    fn apply(
        &self,
        input: BlockPos,
        random: &mut R,
        output: &mut Vec<BlockPos>,
    ) -> Result<(), PlacementError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementReport {
    pub terminal_positions: usize,
    pub any_placed: bool,
}

pub fn place_with_modifiers<R: GenerationRandom>(
    origin: BlockPos,
    modifiers: &[&dyn PlacementModifier<R>],
    random: &mut R,
    maximum_terminal_positions: usize,
    mut place: impl FnMut(BlockPos) -> bool,
) -> Result<PlacementReport, PlacementError> {
    if maximum_terminal_positions == 0 {
        return Err(PlacementError::ZeroTerminalCapacity);
    }
    let mut traversal = Traversal {
        modifiers,
        random,
        maximum_terminal_positions,
        terminal_positions: 0,
        any_placed: false,
        place: &mut place,
    };
    traversal.visit(0, origin)?;
    Ok(PlacementReport {
        terminal_positions: traversal.terminal_positions,
        any_placed: traversal.any_placed,
    })
}

struct Traversal<'a, R, F> {
    modifiers: &'a [&'a dyn PlacementModifier<R>],
    random: &'a mut R,
    maximum_terminal_positions: usize,
    terminal_positions: usize,
    any_placed: bool,
    place: &'a mut F,
}

impl<R: GenerationRandom, F: FnMut(BlockPos) -> bool> Traversal<'_, R, F> {
    fn visit(&mut self, modifier_index: usize, position: BlockPos) -> Result<(), PlacementError> {
        let Some(modifier) = self.modifiers.get(modifier_index) else {
            if self.terminal_positions == self.maximum_terminal_positions {
                return Err(PlacementError::TerminalCapacity {
                    capacity: self.maximum_terminal_positions,
                });
            }
            self.terminal_positions += 1;
            self.any_placed |= (self.place)(position);
            return Ok(());
        };
        let mut outputs = Vec::new();
        modifier.apply(position, self.random, &mut outputs)?;
        for output in outputs {
            self.visit(modifier_index + 1, output)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PlacementError {
    #[error("placed-feature terminal capacity cannot be zero")]
    ZeroTerminalCapacity,
    #[error("placed-feature emitted more than {capacity} terminal positions")]
    TerminalCapacity { capacity: usize },
    #[error("placement modifier output is invalid")]
    InvalidModifierOutput,
    #[error("placement modifier sampled an invalid provider value")]
    Provider(#[from] ProviderError),
    #[error("placement modifier predicate failed")]
    Predicate(#[from] PredicateError),
    #[error("biome placement requires a top placed-feature identity")]
    MissingTopFeature,
    #[error("placement position arithmetic overflowed")]
    PositionOverflow,
    #[error("placement modifier count {count} exceeds the supported range 0..={maximum}")]
    CountOutOfRange { count: i32, maximum: u32 },
    #[error("environment scan steps must be in the inclusive range 1..=32")]
    InvalidEnvironmentScanSteps,
}
