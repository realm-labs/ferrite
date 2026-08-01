use std::collections::BTreeMap;

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::resource::ResourceId;
use ferrite_protocol::semantic::SessionId;
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

pub const ENTITY_TICKING_LEVEL: u8 = 31;
pub const BLOCK_TICKING_LEVEL: u8 = 32;
pub const ACCESSIBLE_LEVEL: u8 = 33;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TicketSource {
    PlayerView(SessionId),
    PlayerSimulation(SessionId),
    Portal(ResourceId),
    Forced(ResourceId),
    Generation(ResourceId),
    PendingSave(ResourceId),
    ScheduledBlock(ResourceId),
    Administration(ResourceId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TicketLevel(u8);

impl TicketLevel {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkTicket {
    pub source: TicketSource,
    pub position: ChunkPos,
    pub level: TicketLevel,
    pub expires_at: Option<GameTick>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkActivation {
    pub loaded: bool,
    pub visible_to_clients: bool,
    pub ticking_blocks: bool,
    pub ticking_entities: bool,
}

impl ChunkActivation {
    const fn from_level(level: Option<TicketLevel>) -> Self {
        let Some(level) = level else {
            return Self {
                loaded: false,
                visible_to_clients: false,
                ticking_blocks: false,
                ticking_entities: false,
            };
        };
        Self {
            loaded: true,
            visible_to_clients: level.get() <= ACCESSIBLE_LEVEL,
            ticking_blocks: level.get() <= BLOCK_TICKING_LEVEL,
            ticking_entities: level.get() <= ENTITY_TICKING_LEVEL,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkTicketBook {
    maximum_tickets: usize,
    tickets: BTreeMap<(ChunkPos, TicketSource), ChunkTicket>,
}

impl ChunkTicketBook {
    pub fn new(maximum_tickets: usize) -> Result<Self, ChunkTicketError> {
        if maximum_tickets == 0 {
            return Err(ChunkTicketError::ZeroCapacity);
        }
        Ok(Self {
            maximum_tickets,
            tickets: BTreeMap::new(),
        })
    }

    pub fn upsert(&mut self, ticket: ChunkTicket) -> Result<(), ChunkTicketError> {
        let key = (ticket.position, ticket.source.clone());
        if !self.tickets.contains_key(&key) && self.tickets.len() == self.maximum_tickets {
            return Err(ChunkTicketError::Full {
                maximum: self.maximum_tickets,
            });
        }
        self.tickets.insert(key, ticket);
        Ok(())
    }

    pub fn remove(&mut self, position: ChunkPos, source: &TicketSource) -> Option<ChunkTicket> {
        self.tickets.remove(&(position, source.clone()))
    }

    pub fn remove_source(&mut self, source: &TicketSource) -> usize {
        let before = self.tickets.len();
        self.tickets.retain(|(_, candidate), _| candidate != source);
        before - self.tickets.len()
    }

    pub fn replace_source(
        &mut self,
        source: &TicketSource,
        replacements: Vec<ChunkTicket>,
    ) -> Result<(), ChunkTicketError> {
        if replacements.iter().any(|ticket| &ticket.source != source) {
            return Err(ChunkTicketError::SourceMismatch);
        }
        let mut unique = replacements
            .iter()
            .map(|ticket| ticket.position)
            .collect::<Vec<_>>();
        unique.sort_unstable();
        unique.dedup();
        if unique.len() != replacements.len() {
            return Err(ChunkTicketError::DuplicateReplacement);
        }
        let retained = self
            .tickets
            .keys()
            .filter(|(_, candidate)| candidate != source)
            .count();
        let required = retained
            .checked_add(replacements.len())
            .ok_or(ChunkTicketError::Full {
                maximum: self.maximum_tickets,
            })?;
        if required > self.maximum_tickets {
            return Err(ChunkTicketError::Full {
                maximum: self.maximum_tickets,
            });
        }
        self.tickets.retain(|(_, candidate), _| candidate != source);
        for ticket in replacements {
            self.tickets
                .insert((ticket.position, ticket.source.clone()), ticket);
        }
        Ok(())
    }

    pub fn expire(&mut self, now: GameTick) -> usize {
        let before = self.tickets.len();
        self.tickets
            .retain(|_, ticket| ticket.expires_at.is_none_or(|expiration| expiration > now));
        before - self.tickets.len()
    }

    #[must_use]
    pub fn effective_level(&self, position: ChunkPos) -> Option<TicketLevel> {
        self.tickets
            .iter()
            .filter_map(|((candidate, _), ticket)| (*candidate == position).then_some(ticket.level))
            .min()
    }

    #[must_use]
    pub fn activation(&self, position: ChunkPos) -> ChunkActivation {
        ChunkActivation::from_level(self.effective_level(position))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tickets.len()
    }

    pub fn tickets(&self) -> impl Iterator<Item = &ChunkTicket> {
        self.tickets.values()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tickets.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChunkTicketError {
    #[error("chunk ticket capacity cannot be zero")]
    ZeroCapacity,
    #[error("chunk ticket book reached its {maximum}-ticket bound")]
    Full { maximum: usize },
    #[error("replacement ticket source differs from the replaced source")]
    SourceMismatch,
    #[error("replacement ticket set repeats a chunk position")]
    DuplicateReplacement,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strongest_ticket_controls_distinct_activation_thresholds() {
        let position = ChunkPos::new(-2, 4);
        let mut tickets = ChunkTicketBook::new(4).unwrap();
        tickets
            .upsert(ChunkTicket {
                source: TicketSource::PlayerView(SessionId::new(1).unwrap()),
                position,
                level: TicketLevel::new(ACCESSIBLE_LEVEL),
                expires_at: None,
            })
            .unwrap();
        assert_eq!(
            tickets.activation(position),
            ChunkActivation {
                loaded: true,
                visible_to_clients: true,
                ticking_blocks: false,
                ticking_entities: false,
            }
        );
        tickets
            .upsert(ChunkTicket {
                source: TicketSource::PlayerSimulation(SessionId::new(1).unwrap()),
                position,
                level: TicketLevel::new(ENTITY_TICKING_LEVEL),
                expires_at: Some(GameTick::new(10)),
            })
            .unwrap();
        assert!(tickets.activation(position).ticking_entities);
        assert_eq!(tickets.expire(GameTick::new(10)), 1);
        assert!(!tickets.activation(position).ticking_blocks);
    }

    #[test]
    fn source_replacement_preflights_capacity_without_partial_change() {
        let source = TicketSource::PlayerView(SessionId::new(2).unwrap());
        let retained = TicketSource::Forced(ResourceId::minecraft("spawn").unwrap());
        let mut tickets = ChunkTicketBook::new(2).unwrap();
        for (position, source) in [
            (ChunkPos::new(0, 0), source.clone()),
            (ChunkPos::new(5, 5), retained),
        ] {
            tickets
                .upsert(ChunkTicket {
                    source,
                    position,
                    level: TicketLevel::new(ACCESSIBLE_LEVEL),
                    expires_at: None,
                })
                .unwrap();
        }

        let replacements = [ChunkPos::new(1, 0), ChunkPos::new(2, 0)]
            .into_iter()
            .map(|position| ChunkTicket {
                source: source.clone(),
                position,
                level: TicketLevel::new(ACCESSIBLE_LEVEL),
                expires_at: None,
            })
            .collect();
        assert_eq!(
            tickets.replace_source(&source, replacements).unwrap_err(),
            ChunkTicketError::Full { maximum: 2 }
        );
        assert_eq!(tickets.len(), 2);
        assert!(tickets.effective_level(ChunkPos::new(0, 0)).is_some());
        assert!(tickets.effective_level(ChunkPos::new(1, 0)).is_none());
    }
}
