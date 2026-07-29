//! Bounded deterministic command admission.

use crate::tick::GameTick;
use ferrite_foundation::identity::StableEntityId;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const MAX_COMMAND_PAYLOAD_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CommandSource {
    System(ResourceId),
    Player(StableEntityId),
    Region(SimulationRegionKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionCommand {
    target: SimulationRegionKey,
    tick: GameTick,
    source: CommandSource,
    sequence: u64,
    kind: ResourceId,
    payload: Vec<u8>,
}

impl RegionCommand {
    pub fn new(
        target: SimulationRegionKey,
        tick: GameTick,
        source: CommandSource,
        sequence: u64,
        kind: ResourceId,
        payload: Vec<u8>,
    ) -> Result<Self, CommandError> {
        if payload.len() > MAX_COMMAND_PAYLOAD_BYTES {
            return Err(CommandError::PayloadTooLarge {
                actual: payload.len(),
                maximum: MAX_COMMAND_PAYLOAD_BYTES,
            });
        }
        Ok(Self {
            target,
            tick,
            source,
            sequence,
            kind,
            payload,
        })
    }

    pub const fn target(&self) -> &SimulationRegionKey {
        &self.target
    }

    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub const fn source(&self) -> &CommandSource {
        &self.source
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    fn order_key(&self) -> CommandOrderKey {
        CommandOrderKey {
            tick: self.tick,
            source: self.source.clone(),
            sequence: self.sequence,
        }
    }
}

#[derive(Debug)]
pub struct CommandInbox {
    target: SimulationRegionKey,
    capacity: usize,
    maximum_future_ticks: u64,
    pending: BTreeMap<CommandOrderKey, RegionCommand>,
    admitted: BTreeSet<CommandOrderKey>,
}

impl CommandInbox {
    pub fn new(
        target: SimulationRegionKey,
        capacity: usize,
        maximum_future_ticks: u64,
    ) -> Result<Self, CommandError> {
        if capacity == 0 {
            return Err(CommandError::ZeroCapacity);
        }
        Ok(Self {
            target,
            capacity,
            maximum_future_ticks,
            pending: BTreeMap::new(),
            admitted: BTreeSet::new(),
        })
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn admit(
        &mut self,
        command: RegionCommand,
        committed_tick: GameTick,
    ) -> Result<(), CommandError> {
        if command.target != self.target {
            return Err(CommandError::WrongTarget);
        }
        let minimum = committed_tick
            .checked_next()
            .map_err(|_| CommandError::TickExhausted)?;
        let maximum = committed_tick
            .get()
            .checked_add(self.maximum_future_ticks)
            .ok_or(CommandError::TickExhausted)?;
        if command.tick < minimum {
            return Err(CommandError::Late {
                command: command.tick,
                minimum,
            });
        }
        if command.tick.get() > maximum {
            return Err(CommandError::TooFarInFuture {
                command: command.tick,
                maximum: GameTick::new(maximum),
            });
        }
        let key = command.order_key();
        if self.admitted.contains(&key) {
            return Err(CommandError::Duplicate);
        }
        if self.pending.len() == self.capacity || self.admitted.len() == self.capacity {
            return Err(CommandError::Full {
                capacity: self.capacity,
            });
        }
        self.admitted.insert(key.clone());
        self.pending.insert(key, command);
        Ok(())
    }

    pub fn drain_tick(&mut self, tick: GameTick) -> Vec<RegionCommand> {
        let keys = self
            .pending
            .keys()
            .filter(|key| key.tick == tick)
            .cloned()
            .collect::<Vec<_>>();
        keys.into_iter()
            .filter_map(|key| self.pending.remove(&key))
            .collect()
    }

    pub fn prune_committed(&mut self, committed_tick: GameTick) {
        self.admitted.retain(|key| key.tick > committed_tick);
    }

    pub fn has_pending_at_or_before(&self, tick: GameTick) -> bool {
        self.pending.keys().any(|key| key.tick <= tick)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CommandOrderKey {
    tick: GameTick,
    source: CommandSource,
    sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommandError {
    #[error("command inbox capacity cannot be zero")]
    ZeroCapacity,
    #[error("command payload has {actual} bytes, exceeding the {maximum}-byte limit")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error("command targets another Region")]
    WrongTarget,
    #[error("command tick {command:?} is earlier than minimum {minimum:?}")]
    Late {
        command: GameTick,
        minimum: GameTick,
    },
    #[error("command tick {command:?} exceeds maximum {maximum:?}")]
    TooFarInFuture {
        command: GameTick,
        maximum: GameTick,
    },
    #[error("command tick arithmetic is exhausted")]
    TickExhausted,
    #[error("command order key is already admitted")]
    Duplicate,
    #[error("command inbox reached its {capacity}-command bound")]
    Full { capacity: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};

    fn region() -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        )
    }

    fn command(source: CommandSource, sequence: u64) -> RegionCommand {
        RegionCommand::new(
            region(),
            GameTick::new(1),
            source,
            sequence,
            ResourceId::new("ferrite", "command/test").unwrap(),
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn arrival_order_does_not_change_drain_order() {
        let player = StableEntityId::new(1).unwrap();
        let commands = [
            command(CommandSource::Player(player), 2),
            command(
                CommandSource::System(ResourceId::new("ferrite", "admin").unwrap()),
                3,
            ),
            command(CommandSource::Player(player), 1),
        ];
        let mut first = CommandInbox::new(region(), 8, 2).unwrap();
        let mut second = CommandInbox::new(region(), 8, 2).unwrap();
        for command in commands.clone() {
            first.admit(command, GameTick::ZERO).unwrap();
        }
        for command in commands.into_iter().rev() {
            second.admit(command, GameTick::ZERO).unwrap();
        }
        let first = first.drain_tick(GameTick::new(1));
        let second = second.drain_tick(GameTick::new(1));
        assert_eq!(first, second);
        assert_eq!(first[1].sequence(), 1);
        assert_eq!(first[2].sequence(), 2);
    }

    #[test]
    fn invalid_duplicate_and_overload_admission_do_not_mutate() {
        let mut inbox = CommandInbox::new(region(), 1, 1).unwrap();
        let command = command(
            CommandSource::System(ResourceId::new("ferrite", "admin").unwrap()),
            1,
        );
        inbox.admit(command.clone(), GameTick::ZERO).unwrap();
        assert_eq!(inbox.drain_tick(GameTick::new(1)).len(), 1);
        assert!(inbox.admit(command, GameTick::ZERO).is_err());
        assert_eq!(inbox.len(), 0);
        inbox.prune_committed(GameTick::new(1));
        assert!(inbox.is_empty());
    }
}
