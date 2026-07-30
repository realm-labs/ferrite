//! Simulation-ticket filtering, timeout, and purge semantics.

use crate::random_tick::tracker::{ABSENT_SIMULATION_LEVEL, pack_chunk};
use ferrite_foundation::coordinate::ChunkPos;
use std::collections::BTreeMap;

pub const FLAG_PERSIST: u8 = 1;
pub const FLAG_LOADING: u8 = 2;
pub const FLAG_SIMULATION: u8 = 4;
pub const FLAG_KEEP_DIMENSION_ACTIVE: u8 = 8;
pub const FLAG_CAN_EXPIRE_IF_UNLOADED: u8 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TicketKind {
    PlayerSpawn,
    SpawnSearch,
    Dragon,
    PlayerLoading,
    PlayerSimulation,
    Forced,
    Portal,
    EnderPearl,
    Unknown,
}

impl TicketKind {
    pub const ALL: [Self; 9] = [
        Self::PlayerSpawn,
        Self::SpawnSearch,
        Self::Dragon,
        Self::PlayerLoading,
        Self::PlayerSimulation,
        Self::Forced,
        Self::Portal,
        Self::EnderPearl,
        Self::Unknown,
    ];

    pub const fn timeout(self) -> i64 {
        match self {
            Self::PlayerSpawn => 20,
            Self::SpawnSearch | Self::Unknown => 1,
            Self::Portal => 300,
            Self::EnderPearl => 40,
            Self::Dragon | Self::PlayerLoading | Self::PlayerSimulation | Self::Forced => 0,
        }
    }

    pub const fn flags(self) -> u8 {
        match self {
            Self::PlayerSpawn | Self::SpawnSearch | Self::PlayerLoading => FLAG_LOADING,
            Self::Dragon => FLAG_LOADING | FLAG_SIMULATION,
            Self::PlayerSimulation => FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
            Self::Forced | Self::Portal => {
                FLAG_PERSIST | FLAG_LOADING | FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE
            }
            Self::EnderPearl => FLAG_LOADING | FLAG_SIMULATION | FLAG_KEEP_DIMENSION_ACTIVE,
            Self::Unknown => FLAG_LOADING | FLAG_CAN_EXPIRE_IF_UNLOADED,
        }
    }

    pub const fn persists(self) -> bool {
        self.flags() & FLAG_PERSIST != 0
    }

    pub const fn loads(self) -> bool {
        self.flags() & FLAG_LOADING != 0
    }

    pub const fn simulates(self) -> bool {
        self.flags() & FLAG_SIMULATION != 0
    }

    pub const fn keeps_dimension_active(self) -> bool {
        self.flags() & FLAG_KEEP_DIMENSION_ACTIVE != 0
    }

    pub const fn can_expire_if_unloaded(self) -> bool {
        self.flags() & FLAG_CAN_EXPIRE_IF_UNLOADED != 0
    }

    pub const fn has_timeout(self) -> bool {
        self.timeout() != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticket {
    kind: TicketKind,
    level: i32,
    ticks_left: i64,
}

impl Ticket {
    pub const fn new(kind: TicketKind, level: i32) -> Self {
        Self {
            kind,
            level,
            ticks_left: kind.timeout(),
        }
    }

    pub const fn kind(&self) -> TicketKind {
        self.kind
    }

    pub const fn level(&self) -> i32 {
        self.level
    }

    pub const fn ticks_left(&self) -> i64 {
        self.ticks_left
    }

    pub fn reset_timeout(&mut self) {
        self.ticks_left = self.kind.timeout();
    }

    pub fn decrement_timeout(&mut self) {
        if self.kind.has_timeout() {
            self.ticks_left = self.ticks_left.wrapping_sub(1);
        }
    }

    pub const fn is_timed_out(&self) -> bool {
        self.kind.has_timeout() && self.ticks_left < 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdatingHolderState {
    Missing,
    ReadyForSaving,
    NotReadyForSaving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpiredTicket {
    pub chunk: ChunkPos,
    pub kind: TicketKind,
    pub level: i32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TicketStorage {
    by_chunk: BTreeMap<i64, Vec<Ticket>>,
}

impl TicketStorage {
    pub fn add(&mut self, chunk: ChunkPos, ticket: Ticket) -> bool {
        let tickets = self.by_chunk.entry(pack_chunk(chunk)).or_default();
        if let Some(existing) = tickets
            .iter_mut()
            .find(|existing| existing.kind == ticket.kind && existing.level == ticket.level)
        {
            existing.reset_timeout();
            return false;
        }
        tickets.push(ticket);
        true
    }

    pub fn remove(&mut self, chunk: ChunkPos, kind: TicketKind, level: i32) -> bool {
        let key = pack_chunk(chunk);
        let Some(tickets) = self.by_chunk.get_mut(&key) else {
            return false;
        };
        let Some(index) = tickets
            .iter()
            .position(|ticket| ticket.kind == kind && ticket.level == level)
        else {
            return false;
        };
        tickets.remove(index);
        if tickets.is_empty() {
            self.by_chunk.remove(&key);
        }
        true
    }

    pub fn simulation_level(&self, chunk: ChunkPos) -> u8 {
        self.lowest_level(pack_chunk(chunk), true)
    }

    pub fn loading_level(&self, chunk: ChunkPos) -> u8 {
        self.lowest_level(pack_chunk(chunk), false)
    }

    pub fn tickets(&self, chunk: ChunkPos) -> &[Ticket] {
        self.by_chunk
            .get(&pack_chunk(chunk))
            .map_or(&[], Vec::as_slice)
    }

    pub fn purge_stale(
        &mut self,
        runs_normally: bool,
        mut holder_state: impl FnMut(ChunkPos) -> UpdatingHolderState,
    ) -> Vec<ExpiredTicket> {
        if !runs_normally {
            return Vec::new();
        }
        let mut expired = Vec::new();
        for (key, tickets) in &mut self.by_chunk {
            let chunk = crate::random_tick::tracker::unpack_chunk(*key);
            tickets.retain_mut(|ticket| {
                let can_decrement = ticket.kind.has_timeout()
                    && (ticket.kind.can_expire_if_unloaded()
                        || holder_state(chunk) != UpdatingHolderState::NotReadyForSaving);
                if !can_decrement {
                    return true;
                }
                ticket.decrement_timeout();
                if !ticket.is_timed_out() {
                    return true;
                }
                expired.push(ExpiredTicket {
                    chunk,
                    kind: ticket.kind,
                    level: ticket.level,
                });
                false
            });
        }
        self.by_chunk.retain(|_, tickets| !tickets.is_empty());
        expired
    }

    fn lowest_level(&self, key: i64, simulation: bool) -> u8 {
        self.by_chunk
            .get(&key)
            .into_iter()
            .flatten()
            .filter(|ticket| {
                if simulation {
                    ticket.kind.simulates()
                } else {
                    ticket.kind.loads()
                }
            })
            .map(|ticket| ticket.level)
            .min()
            .and_then(|level| u8::try_from(level).ok())
            .unwrap_or(ABSENT_SIMULATION_LEVEL)
    }
}
