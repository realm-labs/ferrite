//! Fixed Region tick state machine and explicit phase barriers.

use crate::journal::{ActiveTickJournal, CommittedTickJournal, JournalDomain, JournalError};
use crate::tick::{GameTick, PhaseBarrier, TickPhase};
use ferrite_foundation::resource::ResourceId;
use thiserror::Error;

#[derive(Debug)]
pub struct RegionTickPipeline {
    committed_tick: GameTick,
    journal_capacity: usize,
    active: Option<ActiveTick>,
}

impl RegionTickPipeline {
    pub fn new(committed_tick: GameTick, journal_capacity: usize) -> Result<Self, PipelineError> {
        if journal_capacity == 0 {
            return Err(PipelineError::Journal(JournalError::ZeroCapacity));
        }
        Ok(Self {
            committed_tick,
            journal_capacity,
            active: None,
        })
    }

    pub const fn committed_tick(&self) -> GameTick {
        self.committed_tick
    }

    pub fn active_tick(&self) -> Option<GameTick> {
        self.active.as_ref().map(|active| active.tick)
    }

    pub fn current_phase(&self) -> Option<TickPhase> {
        self.active.as_ref().map(|active| active.phase)
    }

    pub fn begin_tick(&mut self, tick: GameTick) -> Result<(), PipelineError> {
        if self.active.is_some() {
            return Err(PipelineError::TickAlreadyActive);
        }
        let expected = self
            .committed_tick
            .checked_next()
            .map_err(|_| PipelineError::TickExhausted)?;
        if tick != expected {
            return Err(PipelineError::UnexpectedTick {
                expected,
                actual: tick,
            });
        }
        self.active = Some(ActiveTick {
            tick,
            phase: TickPhase::Begin,
            structural_changes_applied: false,
            boundary_emitted: false,
            reconciliation_complete: false,
            journal: ActiveTickJournal::new(tick, self.journal_capacity)?,
        });
        Ok(())
    }

    pub fn append_journal(
        &mut self,
        domain: JournalDomain,
        kind: ResourceId,
        payload: Vec<u8>,
    ) -> Result<u64, PipelineError> {
        let active = self.active.as_mut().ok_or(PipelineError::NoActiveTick)?;
        Ok(active.journal.append(active.phase, domain, kind, payload)?)
    }

    pub fn satisfy_current_barrier(&mut self) -> Result<(), PipelineError> {
        let active = self.active.as_mut().ok_or(PipelineError::NoActiveTick)?;
        match active.phase.contract().barrier() {
            PhaseBarrier::StructuralChanges => active.structural_changes_applied = true,
            PhaseBarrier::BoundaryEmission => active.boundary_emitted = true,
            PhaseBarrier::RequiredReconciliation => active.reconciliation_complete = true,
            PhaseBarrier::None | PhaseBarrier::Commit => {
                return Err(PipelineError::NoSatisfiableBarrier(active.phase));
            }
        }
        Ok(())
    }

    pub fn advance_phase(&mut self) -> Result<TickPhase, PipelineError> {
        let active = self.active.as_mut().ok_or(PipelineError::NoActiveTick)?;
        ensure_barrier(active)?;
        let next = active.phase.next().ok_or(PipelineError::CommitRequired)?;
        active.phase = next;
        Ok(next)
    }

    pub fn commit_tick(&mut self) -> Result<CommittedTickJournal, PipelineError> {
        let active = self.active.as_ref().ok_or(PipelineError::NoActiveTick)?;
        if active.phase != TickPhase::Commit {
            return Err(PipelineError::WrongCommitPhase(active.phase));
        }
        let active = self.active.take().expect("active tick was just validated");
        self.committed_tick = active.tick;
        Ok(active.journal.commit())
    }
}

#[derive(Debug)]
struct ActiveTick {
    tick: GameTick,
    phase: TickPhase,
    structural_changes_applied: bool,
    boundary_emitted: bool,
    reconciliation_complete: bool,
    journal: ActiveTickJournal,
}

fn ensure_barrier(active: &ActiveTick) -> Result<(), PipelineError> {
    let satisfied = match active.phase.contract().barrier() {
        PhaseBarrier::None => true,
        PhaseBarrier::StructuralChanges => active.structural_changes_applied,
        PhaseBarrier::BoundaryEmission => active.boundary_emitted,
        PhaseBarrier::RequiredReconciliation => active.reconciliation_complete,
        PhaseBarrier::Commit => false,
    };
    if satisfied {
        Ok(())
    } else {
        Err(PipelineError::BarrierPending(active.phase))
    }
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("a Region tick is already active")]
    TickAlreadyActive,
    #[error("no Region tick is active")]
    NoActiveTick,
    #[error("expected tick {expected:?}, got {actual:?}")]
    UnexpectedTick {
        expected: GameTick,
        actual: GameTick,
    },
    #[error("logical tick is exhausted")]
    TickExhausted,
    #[error("phase {0:?} has no satisfiable external barrier")]
    NoSatisfiableBarrier(TickPhase),
    #[error("phase {0:?} barrier is not satisfied")]
    BarrierPending(TickPhase),
    #[error("commit must use commit_tick instead of advancing beyond Commit")]
    CommitRequired,
    #[error("cannot commit from phase {0:?}")]
    WrongCommitPhase(TickPhase),
    #[error(transparent)]
    Journal(#[from] JournalError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_phase_runs_in_locked_order_and_commit_publishes_journal() {
        let mut pipeline = RegionTickPipeline::new(GameTick::ZERO, 4).unwrap();
        pipeline.begin_tick(GameTick::new(1)).unwrap();
        assert!(pipeline.begin_tick(GameTick::new(1)).is_err());
        pipeline
            .append_journal(
                JournalDomain::Command,
                ResourceId::new("ferrite", "command/test").unwrap(),
                vec![1],
            )
            .unwrap();
        for expected in TickPhase::ALL.into_iter().skip(1) {
            let phase = pipeline.current_phase().unwrap();
            if !matches!(
                phase.contract().barrier(),
                PhaseBarrier::None | PhaseBarrier::Commit
            ) {
                pipeline.satisfy_current_barrier().unwrap();
            }
            assert_eq!(pipeline.advance_phase().unwrap(), expected);
        }
        let committed = pipeline.commit_tick().unwrap();
        assert_eq!(pipeline.committed_tick(), GameTick::new(1));
        assert_eq!(committed.entries().len(), 1);
        assert_eq!(committed.entries()[0].phase(), TickPhase::Begin);
    }

    #[test]
    fn skipped_barriers_and_non_monotonic_ticks_fail_closed() {
        let mut pipeline = RegionTickPipeline::new(GameTick::ZERO, 1).unwrap();
        assert!(pipeline.begin_tick(GameTick::new(2)).is_err());
        pipeline.begin_tick(GameTick::new(1)).unwrap();
        while pipeline.current_phase() != Some(TickPhase::EcsStructuralChanges) {
            pipeline.advance_phase().unwrap();
        }
        assert!(pipeline.advance_phase().is_err());
        assert_eq!(
            pipeline.current_phase(),
            Some(TickPhase::EcsStructuralChanges)
        );
    }
}
