//! Dedicated empty-server and integrated-client pause admission.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedicatedPauseDecision {
    AdmitBaseTick,
    Pause {
        first_paused_iteration: bool,
        auto_save: bool,
        tick_connections: bool,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DedicatedPauseState {
    empty_ticks: i32,
}

impl DedicatedPauseState {
    pub const fn empty_ticks(self) -> i32 {
        self.empty_ticks
    }

    pub fn evaluate(
        &mut self,
        pause_when_empty_seconds: i32,
        player_count: usize,
        sprint_scheduled: bool,
    ) -> DedicatedPauseDecision {
        let threshold = pause_when_empty_seconds.wrapping_mul(20);
        if threshold <= 0 {
            return DedicatedPauseDecision::AdmitBaseTick;
        }
        self.empty_ticks = if player_count == 0 && !sprint_scheduled {
            self.empty_ticks.wrapping_add(1)
        } else {
            0
        };
        if self.empty_ticks < threshold {
            DedicatedPauseDecision::AdmitBaseTick
        } else {
            let first = self.empty_ticks == threshold;
            DedicatedPauseDecision::Pause {
                first_paused_iteration: first,
                auto_save: first,
                tick_connections: true,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegratedPauseDecision {
    AdmitBaseTick,
    SaveThenPause {
        tick_connections: bool,
        player_stat_awards: usize,
    },
    ContinuePaused {
        tick_connections: bool,
        player_stat_awards: usize,
    },
    SynchronizeTimeThenAdmit,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IntegratedPauseState {
    paused: bool,
}

impl IntegratedPauseState {
    pub const fn is_paused(self) -> bool {
        self.paused
    }

    pub fn evaluate(
        &mut self,
        client_paused: bool,
        present_players: usize,
    ) -> IntegratedPauseDecision {
        let was_paused = self.paused;
        self.paused = client_paused || present_players == 0;
        match (was_paused, self.paused) {
            (false, true) => IntegratedPauseDecision::SaveThenPause {
                tick_connections: true,
                player_stat_awards: present_players,
            },
            (true, true) => IntegratedPauseDecision::ContinuePaused {
                tick_connections: true,
                player_stat_awards: present_players,
            },
            (true, false) => IntegratedPauseDecision::SynchronizeTimeThenAdmit,
            (false, false) => IntegratedPauseDecision::AdmitBaseTick,
        }
    }
}
