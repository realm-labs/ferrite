//! Two-stage readiness, bounded admission, and drain completion accounting.

use serde::Serialize;
use std::collections::BTreeSet;
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodePhase {
    AwaitingMembership,
    AwaitingPlacement,
    Ready,
    Draining,
    Drained,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LifecycleSnapshot {
    pub phase: NodePhase,
    pub healthy: bool,
    pub ready: bool,
    pub admission_open: bool,
    pub membership_ready: bool,
    pub required_domains: BTreeSet<String>,
    pub ready_domains: BTreeSet<String>,
    pub active_sessions: usize,
    pub active_region_authorities: usize,
    pub pending_commits: usize,
    pub failure: Option<String>,
}

pub struct NodeLifecycle {
    state: Mutex<LifecycleState>,
    changed: Condvar,
}

impl NodeLifecycle {
    pub fn new(required_domains: BTreeSet<String>) -> Self {
        Self {
            state: Mutex::new(LifecycleState {
                phase: NodePhase::AwaitingMembership,
                membership_ready: false,
                required_domains,
                ready_domains: BTreeSet::new(),
                active_sessions: 0,
                active_region_authorities: 0,
                pending_commits: 0,
                failure: None,
            }),
            changed: Condvar::new(),
        }
    }

    pub fn snapshot(&self) -> Result<LifecycleSnapshot, LifecycleError> {
        let state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        Ok(state.snapshot())
    }

    pub fn mark_membership_ready(&self) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        state.ensure_bootstrapping()?;
        state.membership_ready = true;
        state.refresh_readiness();
        self.changed.notify_all();
        Ok(())
    }

    pub fn mark_membership_lost(&self) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        state.ensure_bootstrapping()?;
        state.membership_ready = false;
        state.phase = NodePhase::AwaitingMembership;
        self.changed.notify_all();
        Ok(())
    }

    pub fn mark_placement_domain_ready(&self, domain: &str) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        state.ensure_bootstrapping()?;
        if !state.required_domains.contains(domain) {
            return Err(LifecycleError::UnexpectedDomain(domain.to_owned()));
        }
        state.ready_domains.insert(domain.to_owned());
        state.refresh_readiness();
        self.changed.notify_all();
        Ok(())
    }

    pub fn mark_placement_domain_lost(&self, domain: &str) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        state.ensure_bootstrapping()?;
        if !state.required_domains.contains(domain) {
            return Err(LifecycleError::UnexpectedDomain(domain.to_owned()));
        }
        state.ready_domains.remove(domain);
        state.refresh_readiness();
        self.changed.notify_all();
        Ok(())
    }

    pub fn admit_session(&self, maximum: usize) -> Result<(), AdmissionRejection> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AdmissionRejection::Unavailable)?;
        if state.phase != NodePhase::Ready {
            return Err(AdmissionRejection::Unavailable);
        }
        if state.active_sessions >= maximum {
            return Err(AdmissionRejection::Capacity);
        }
        state.active_sessions += 1;
        self.changed.notify_all();
        Ok(())
    }

    pub fn complete_session(&self) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        state.active_sessions = state
            .active_sessions
            .checked_sub(1)
            .ok_or(LifecycleError::CounterUnderflow("active sessions"))?;
        state.refresh_drain();
        self.changed.notify_all();
        Ok(())
    }

    pub fn set_active_region_authorities(&self, count: usize) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        state.active_region_authorities = count;
        state.refresh_drain();
        self.changed.notify_all();
        Ok(())
    }

    pub fn set_pending_commits(&self, count: usize) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        state.pending_commits = count;
        state.refresh_drain();
        self.changed.notify_all();
        Ok(())
    }

    pub fn begin_drain(&self) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        match state.phase {
            NodePhase::AwaitingMembership
            | NodePhase::AwaitingPlacement
            | NodePhase::Ready
            | NodePhase::Draining => {
                state.phase = NodePhase::Draining;
                state.refresh_drain();
                self.changed.notify_all();
                Ok(())
            }
            phase => Err(LifecycleError::InvalidTransition {
                from: phase,
                operation: "begin drain",
            }),
        }
    }

    pub fn mark_stopped(&self) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        if state.phase != NodePhase::Drained {
            return Err(LifecycleError::InvalidTransition {
                from: state.phase,
                operation: "mark stopped",
            });
        }
        state.phase = NodePhase::Stopped;
        self.changed.notify_all();
        Ok(())
    }

    pub fn fail(&self, reason: impl Into<String>) -> Result<(), LifecycleError> {
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        state.phase = NodePhase::Failed;
        state.failure = Some(reason.into());
        self.changed.notify_all();
        Ok(())
    }

    pub fn wait_for_phase(
        &self,
        expected: NodePhase,
        timeout: Duration,
    ) -> Result<bool, LifecycleError> {
        let deadline = Instant::now() + timeout;
        let mut state = self.state.lock().map_err(|_| LifecycleError::Poisoned)?;
        while state.phase != expected {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Ok(false);
            }
            let (next, result) = self
                .changed
                .wait_timeout(state, remaining)
                .map_err(|_| LifecycleError::Poisoned)?;
            state = next;
            if result.timed_out() && state.phase != expected {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Debug)]
struct LifecycleState {
    phase: NodePhase,
    membership_ready: bool,
    required_domains: BTreeSet<String>,
    ready_domains: BTreeSet<String>,
    active_sessions: usize,
    active_region_authorities: usize,
    pending_commits: usize,
    failure: Option<String>,
}

impl LifecycleState {
    fn snapshot(&self) -> LifecycleSnapshot {
        LifecycleSnapshot {
            phase: self.phase,
            healthy: !matches!(self.phase, NodePhase::Stopped | NodePhase::Failed),
            ready: self.phase == NodePhase::Ready,
            admission_open: self.phase == NodePhase::Ready,
            membership_ready: self.membership_ready,
            required_domains: self.required_domains.clone(),
            ready_domains: self.ready_domains.clone(),
            active_sessions: self.active_sessions,
            active_region_authorities: self.active_region_authorities,
            pending_commits: self.pending_commits,
            failure: self.failure.clone(),
        }
    }

    fn ensure_bootstrapping(&self) -> Result<(), LifecycleError> {
        match self.phase {
            NodePhase::AwaitingMembership | NodePhase::AwaitingPlacement | NodePhase::Ready => {
                Ok(())
            }
            phase => Err(LifecycleError::InvalidTransition {
                from: phase,
                operation: "change readiness",
            }),
        }
    }

    fn refresh_readiness(&mut self) {
        self.phase = if !self.membership_ready {
            NodePhase::AwaitingMembership
        } else if self.required_domains.is_subset(&self.ready_domains) {
            NodePhase::Ready
        } else {
            NodePhase::AwaitingPlacement
        };
    }

    fn refresh_drain(&mut self) {
        if self.phase == NodePhase::Draining
            && self.active_sessions == 0
            && self.active_region_authorities == 0
            && self.pending_commits == 0
        {
            self.phase = NodePhase::Drained;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AdmissionRejection {
    #[error("node admission is unavailable")]
    Unavailable,
    #[error("node admission capacity is exhausted")]
    Capacity,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LifecycleError {
    #[error("node lifecycle state is poisoned")]
    Poisoned,
    #[error("cannot {operation} from lifecycle phase {from:?}")]
    InvalidTransition {
        from: NodePhase,
        operation: &'static str,
    },
    #[error("placement domain is not required by this node: {0}")]
    UnexpectedDomain(String),
    #[error("{0} counter underflow")]
    CounterUnderflow(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lifecycle() -> NodeLifecycle {
        NodeLifecycle::new(BTreeSet::from([
            "ferrite-region-v1".to_owned(),
            "player-session-v1".to_owned(),
        ]))
    }

    #[test]
    fn readiness_requires_membership_then_every_domain() {
        let lifecycle = lifecycle();
        lifecycle
            .mark_placement_domain_ready("ferrite-region-v1")
            .unwrap();
        assert_eq!(
            lifecycle.snapshot().unwrap().phase,
            NodePhase::AwaitingMembership
        );
        lifecycle.mark_membership_ready().unwrap();
        assert_eq!(
            lifecycle.snapshot().unwrap().phase,
            NodePhase::AwaitingPlacement
        );
        lifecycle
            .mark_placement_domain_ready("player-session-v1")
            .unwrap();
        assert_eq!(lifecycle.snapshot().unwrap().phase, NodePhase::Ready);
        lifecycle.mark_membership_lost().unwrap();
        assert_eq!(
            lifecycle.admit_session(1),
            Err(AdmissionRejection::Unavailable)
        );
    }

    #[test]
    fn bounded_admission_and_drain_wait_for_all_durable_work() {
        let lifecycle = NodeLifecycle::new(BTreeSet::new());
        lifecycle.mark_membership_ready().unwrap();
        lifecycle.admit_session(1).unwrap();
        assert_eq!(
            lifecycle.admit_session(1),
            Err(AdmissionRejection::Capacity)
        );
        lifecycle.set_active_region_authorities(2).unwrap();
        lifecycle.set_pending_commits(1).unwrap();
        lifecycle.begin_drain().unwrap();
        assert_eq!(lifecycle.snapshot().unwrap().phase, NodePhase::Draining);
        assert_eq!(
            lifecycle.admit_session(1),
            Err(AdmissionRejection::Unavailable)
        );

        lifecycle.complete_session().unwrap();
        lifecycle.set_active_region_authorities(0).unwrap();
        assert_eq!(lifecycle.snapshot().unwrap().phase, NodePhase::Draining);
        lifecycle.set_pending_commits(0).unwrap();
        assert_eq!(lifecycle.snapshot().unwrap().phase, NodePhase::Drained);
        lifecycle.mark_stopped().unwrap();
    }
}
