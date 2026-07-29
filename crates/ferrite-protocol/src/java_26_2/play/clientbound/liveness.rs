//! Client-side deferred keepalive echo behavior for frozen event polling.

use std::collections::VecDeque;

use thiserror::Error;

use crate::java_26_2::play::clientbound::projection::PlayClientAction;

const KEEP_ALIVE_DEFERRAL_MILLIS: i64 = 60_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredKeepAliveEchoes {
    capacity: usize,
    entries: VecDeque<DeferredKeepAlive>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeferredKeepAlive {
    challenge: i64,
    expiration_millis: i64,
}

impl DeferredKeepAliveEchoes {
    pub fn new(capacity: usize) -> Result<Self, DeferredKeepAliveError> {
        if capacity == 0 {
            return Err(DeferredKeepAliveError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            entries: VecDeque::new(),
        })
    }

    pub fn receive(
        &mut self,
        challenge: i64,
        now_millis: i64,
        render_frozen: bool,
    ) -> Result<Option<PlayClientAction>, DeferredKeepAliveError> {
        if !render_frozen {
            return Ok(Some(PlayClientAction::EchoKeepAlive(challenge)));
        }
        if self.entries.len() == self.capacity {
            return Err(DeferredKeepAliveError::Full {
                capacity: self.capacity,
            });
        }
        let expiration_millis = now_millis
            .checked_add(KEEP_ALIVE_DEFERRAL_MILLIS)
            .ok_or(DeferredKeepAliveError::DeadlineOverflow)?;
        self.entries.push_back(DeferredKeepAlive {
            challenge,
            expiration_millis,
        });
        Ok(None)
    }

    pub fn poll(&mut self, now_millis: i64, render_frozen: bool) -> Vec<PlayClientAction> {
        let count = self.entries.len();
        let mut actions = Vec::new();
        for _ in 0..count {
            let entry = self
                .entries
                .pop_front()
                .expect("bounded poll count matches queue length");
            if !render_frozen {
                actions.push(PlayClientAction::EchoKeepAlive(entry.challenge));
            } else if entry.expiration_millis > now_millis {
                self.entries.push_back(entry);
            }
        }
        actions
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DeferredKeepAliveError {
    #[error("deferred keepalive capacity cannot be zero")]
    ZeroCapacity,
    #[error("deferred keepalive queue reached its {capacity}-entry bound")]
    Full { capacity: usize },
    #[error("deferred keepalive expiration overflowed")]
    DeadlineOverflow,
}
