//! Client-level load readiness and delayed acknowledgement.

use thiserror::Error;

const LOAD_TIMEOUT_MILLIS: i64 = 30_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerChunkObservation {
    pub player_outside_build_height: bool,
    pub camera_outside_build_height: bool,
    pub spectator: bool,
    pub alive: bool,
}

impl Default for PlayerChunkObservation {
    fn default() -> Self {
        Self {
            player_outside_build_height: false,
            camera_outside_build_height: false,
            spectator: false,
            alive: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelLoadState {
    WaitingForServer {
        deadline_millis: i64,
    },
    WaitingForPlayerChunk {
        deadline_millis: i64,
        player_section_compiled: bool,
    },
    ClientLevelReady {
        ready_at_millis: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelLoadTracker {
    close_delay_millis: i64,
    state: Option<LevelLoadState>,
}

impl LevelLoadTracker {
    pub fn new(close_delay_millis: i64) -> Result<Self, LevelLoadError> {
        if close_delay_millis < 0 {
            return Err(LevelLoadError::NegativeCloseDelay(close_delay_millis));
        }
        Ok(Self {
            close_delay_millis,
            state: None,
        })
    }

    pub fn start_client_load(&mut self, now_millis: i64) {
        self.state = Some(LevelLoadState::WaitingForServer {
            deadline_millis: now_millis.wrapping_add(LOAD_TIMEOUT_MILLIS),
        });
    }

    pub fn loading_packets_received(&mut self) {
        if let Some(LevelLoadState::WaitingForServer { deadline_millis }) = self.state {
            self.state = Some(LevelLoadState::WaitingForPlayerChunk {
                deadline_millis,
                player_section_compiled: false,
            });
        }
    }

    pub fn player_section_compiled(&mut self) {
        if let Some(LevelLoadState::WaitingForPlayerChunk {
            player_section_compiled,
            ..
        }) = self.state.as_mut()
        {
            *player_section_compiled = true;
        }
    }

    pub fn tick(&mut self, now_millis: i64, observation: PlayerChunkObservation) {
        let Some(LevelLoadState::WaitingForPlayerChunk {
            deadline_millis,
            player_section_compiled,
        }) = self.state
        else {
            return;
        };
        let ready = now_millis > deadline_millis
            || player_section_compiled
            || observation.player_outside_build_height
            || observation.camera_outside_build_height
            || observation.spectator
            || !observation.alive;
        if ready {
            self.state = Some(LevelLoadState::ClientLevelReady {
                ready_at_millis: now_millis,
            });
        }
    }

    #[must_use]
    pub fn take_player_loaded(&mut self, now_millis: i64) -> bool {
        let Some(LevelLoadState::ClientLevelReady { ready_at_millis }) = self.state else {
            return false;
        };
        if now_millis < ready_at_millis.wrapping_add(self.close_delay_millis) {
            return false;
        }
        self.state = None;
        true
    }

    #[must_use]
    pub const fn state(&self) -> Option<LevelLoadState> {
        self.state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LevelLoadError {
    #[error("level-load close delay cannot be negative: {0}")]
    NegativeCloseDelay(i64),
}
