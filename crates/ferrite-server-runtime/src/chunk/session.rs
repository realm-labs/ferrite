use ferrite_foundation::coordinate::ChunkPos;
use ferrite_protocol::semantic::{PlayAdmission, SessionId};
use ferrite_world::projection::ChunkSnapshot;
use thiserror::Error;

use crate::chunk::interest::{ClientInterest, InterestError};
use crate::chunk::stream::{ChunkStream, ChunkStreamError, ChunkStreamEvent};
use crate::chunk::ticket::{
    ACCESSIBLE_LEVEL, ChunkTicket, ChunkTicketBook, ChunkTicketError, TicketLevel, TicketSource,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkSessionLimits {
    pub maximum_tracked_chunks: usize,
    pub maximum_tickets: usize,
    pub maximum_chunks_per_batch: usize,
}

#[derive(Debug, Clone)]
pub struct ClientChunkSession {
    session: SessionId,
    stream: ChunkStream,
    tickets: ChunkTicketBook,
    generation: u64,
}

#[derive(Debug, Clone)]
pub struct PreparedChunkBatch {
    base_generation: u64,
    next_stream: ChunkStream,
    events: Vec<ChunkStreamEvent>,
}

impl PreparedChunkBatch {
    #[must_use]
    pub fn events(&self) -> &[ChunkStreamEvent] {
        &self.events
    }
}

impl ClientChunkSession {
    pub fn join(
        admission: &PlayAdmission,
        server_view_distance: u16,
        simulation_distance: u16,
        limits: ChunkSessionLimits,
    ) -> Result<Self, ClientChunkSessionError> {
        let interest = ClientInterest::new(
            admission.spawn_chunk,
            admission.requested_view_distance,
            server_view_distance,
            simulation_distance,
            limits.maximum_tracked_chunks,
        )?;
        let stream = ChunkStream::new(interest, limits.maximum_chunks_per_batch)?;
        let tickets = ChunkTicketBook::new(limits.maximum_tickets)?;
        let mut session = Self {
            session: admission.session,
            stream,
            tickets,
            generation: 0,
        };
        session.replace_tickets()?;
        Ok(session)
    }

    #[must_use]
    pub fn initial_events(&self) -> [ChunkStreamEvent; 3] {
        self.stream.initial_events()
    }

    pub fn recenter(
        &mut self,
        center: ChunkPos,
        alive: bool,
    ) -> Result<Vec<ChunkStreamEvent>, ClientChunkSessionError> {
        let mut candidate = self.clone();
        let events = candidate.stream.recenter(center, alive)?;
        candidate.replace_tickets()?;
        candidate.advance_generation()?;
        *self = candidate;
        Ok(events)
    }

    pub fn restart_dimension(
        &mut self,
        center: ChunkPos,
    ) -> Result<Vec<ChunkStreamEvent>, ClientChunkSessionError> {
        let mut candidate = self.clone();
        let events = candidate.stream.restart(center)?.to_vec();
        candidate.replace_tickets()?;
        candidate.advance_generation()?;
        *self = candidate;
        Ok(events)
    }

    pub fn mark_ready(&mut self, position: ChunkPos) -> Result<bool, ClientChunkSessionError> {
        self.next_generation()?;
        let changed = self.stream.mark_ready(position)?;
        if changed {
            self.advance_generation()?;
        }
        Ok(changed)
    }

    pub fn prepare_next_batch(
        &self,
        snapshot: impl FnMut(ChunkPos) -> Option<ChunkSnapshot>,
    ) -> Result<Option<PreparedChunkBatch>, ClientChunkSessionError> {
        let mut next_stream = self.stream.clone();
        let events = next_stream.next_batch(snapshot)?;
        if events.is_empty() {
            return Ok(None);
        }
        self.generation
            .checked_add(1)
            .ok_or(ClientChunkSessionError::GenerationExhausted)?;
        Ok(Some(PreparedChunkBatch {
            base_generation: self.generation,
            next_stream,
            events,
        }))
    }

    pub fn commit_prepared_batch(
        &mut self,
        prepared: PreparedChunkBatch,
    ) -> Result<(), ClientChunkSessionError> {
        if prepared.base_generation != self.generation {
            return Err(ClientChunkSessionError::StalePreparedBatch {
                prepared: prepared.base_generation,
                current: self.generation,
            });
        }
        self.stream = prepared.next_stream;
        self.advance_generation()
    }

    pub fn acknowledge_batch(
        &mut self,
        desired_chunks_per_tick: f32,
    ) -> Result<(), ClientChunkSessionError> {
        self.next_generation()?;
        self.stream.acknowledge_batch(desired_chunks_per_tick);
        self.advance_generation()
    }

    #[must_use]
    pub const fn stream(&self) -> &ChunkStream {
        &self.stream
    }

    #[must_use]
    pub const fn tickets(&self) -> &ChunkTicketBook {
        &self.tickets
    }

    fn replace_tickets(&mut self) -> Result<(), ClientChunkSessionError> {
        let view_source = TicketSource::PlayerView(self.session);
        let tickets = self
            .stream
            .interest()
            .view()
            .iter()
            .map(|position| ChunkTicket {
                source: view_source.clone(),
                position: *position,
                level: TicketLevel::new(ACCESSIBLE_LEVEL),
                expires_at: None,
            })
            .collect();
        self.tickets.replace_source(&view_source, tickets)?;

        let simulation_source = TicketSource::PlayerSimulation(self.session);
        let simulation_level =
            31u8.saturating_sub(self.stream.interest().simulation_distance().min(31) as u8);
        self.tickets.replace_source(
            &simulation_source,
            vec![ChunkTicket {
                source: simulation_source.clone(),
                position: self.stream.interest().center(),
                level: TicketLevel::new(simulation_level),
                expires_at: None,
            }],
        )?;
        Ok(())
    }

    fn advance_generation(&mut self) -> Result<(), ClientChunkSessionError> {
        self.generation = self.next_generation()?;
        Ok(())
    }

    fn next_generation(&self) -> Result<u64, ClientChunkSessionError> {
        self.generation
            .checked_add(1)
            .ok_or(ClientChunkSessionError::GenerationExhausted)
    }
}

#[derive(Debug, Error)]
pub enum ClientChunkSessionError {
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Stream(#[from] ChunkStreamError),
    #[error(transparent)]
    Ticket(#[from] ChunkTicketError),
    #[error("chunk-session mutation generation is exhausted")]
    GenerationExhausted,
    #[error("prepared chunk batch generation {prepared} is stale; current generation is {current}")]
    StalePreparedBatch { prepared: u64, current: u64 },
}
