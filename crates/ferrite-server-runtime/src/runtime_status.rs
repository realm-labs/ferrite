//! Bounded operator snapshot for the formal Minecraft runtime.

use std::sync::Mutex;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MinecraftRuntimeStatus {
    pub(crate) committed_tick: u64,
    pub(crate) composite_region_commits: usize,
    pub(crate) sessions: Vec<MinecraftSessionStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MinecraftSessionStatus {
    pub(crate) session_id: u64,
    pub(crate) player: Option<String>,
    pub(crate) region_x: Option<i32>,
    pub(crate) region_z: Option<i32>,
    pub(crate) region_transfers: u64,
    pub(crate) last_dispatch: Option<ServerboundDispatchStatus>,
    pub(crate) last_unsupported_dispatch: Option<ServerboundDispatchStatus>,
    pub(crate) last_block_result: Option<BlockResultStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct ServerboundDispatchStatus {
    pub(crate) packet: &'static str,
    pub(crate) responsibility: &'static str,
    pub(crate) disposition: &'static str,
    pub(crate) detail: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct BlockResultStatus {
    pub(crate) command_sequence: u64,
    pub(crate) outcome: &'static str,
    pub(crate) corrections: usize,
}

#[derive(Default)]
pub(crate) struct RuntimeStatus {
    minecraft: Mutex<Option<MinecraftRuntimeStatus>>,
}

impl RuntimeStatus {
    pub(crate) fn minecraft(&self) -> Result<Option<MinecraftRuntimeStatus>, RuntimeStatusError> {
        self.minecraft
            .lock()
            .map(|status| status.clone())
            .map_err(|_| RuntimeStatusError::Poisoned)
    }

    pub(crate) fn update_minecraft(
        &self,
        status: MinecraftRuntimeStatus,
    ) -> Result<(), RuntimeStatusError> {
        *self
            .minecraft
            .lock()
            .map_err(|_| RuntimeStatusError::Poisoned)? = Some(status);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum RuntimeStatusError {
    #[error("runtime status is poisoned")]
    Poisoned,
}
