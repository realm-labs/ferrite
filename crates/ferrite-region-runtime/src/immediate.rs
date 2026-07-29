//! Generation-fenced same-phase effects between Region owners.

use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::tick::{GameTick, TickPhase};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const MAX_IMMEDIATE_EFFECT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmediateEffectHeader {
    pub tick: GameTick,
    pub phase: TickPhase,
    pub source: SimulationRegionKey,
    pub target: SimulationRegionKey,
    pub source_generation: ActivationGeneration,
    pub target_generation: ActivationGeneration,
    pub source_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmediateBoundaryEffect {
    header: ImmediateEffectHeader,
    kind: ResourceId,
    payload: Vec<u8>,
}

impl ImmediateBoundaryEffect {
    pub fn new(
        header: ImmediateEffectHeader,
        kind: ResourceId,
        payload: Vec<u8>,
    ) -> Result<Self, ImmediateEffectError> {
        validate_endpoints(&header.source, &header.target)?;
        if payload.len() > MAX_IMMEDIATE_EFFECT_BYTES {
            return Err(ImmediateEffectError::PayloadTooLarge {
                actual: payload.len(),
                maximum: MAX_IMMEDIATE_EFFECT_BYTES,
            });
        }
        Ok(Self {
            header,
            kind,
            payload,
        })
    }

    pub const fn tick(&self) -> GameTick {
        self.header.tick
    }

    pub const fn phase(&self) -> TickPhase {
        self.header.phase
    }

    pub const fn source(&self) -> &SimulationRegionKey {
        &self.header.source
    }

    pub const fn target(&self) -> &SimulationRegionKey {
        &self.header.target
    }

    pub const fn source_generation(&self) -> ActivationGeneration {
        self.header.source_generation
    }

    pub const fn target_generation(&self) -> ActivationGeneration {
        self.header.target_generation
    }

    pub const fn source_sequence(&self) -> u64 {
        self.header.source_sequence
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    fn order_key(&self) -> ImmediateEffectOrderKey {
        ImmediateEffectOrderKey {
            tick: self.tick(),
            phase: self.phase(),
            target: self.target().clone(),
            source: self.source().clone(),
            source_sequence: self.source_sequence(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ImmediateEffectQueue {
    capacity: usize,
    pending: BTreeMap<ImmediateEffectOrderKey, ImmediateBoundaryEffect>,
    admitted: BTreeSet<ImmediateEffectOrderKey>,
}

impl ImmediateEffectQueue {
    pub(crate) fn new(capacity: usize) -> Result<Self, ImmediateEffectError> {
        if capacity == 0 {
            return Err(ImmediateEffectError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            pending: BTreeMap::new(),
            admitted: BTreeSet::new(),
        })
    }

    pub(crate) fn admit(
        &mut self,
        effect: ImmediateBoundaryEffect,
        source_generation: ActivationGeneration,
        target_generation: ActivationGeneration,
        committed_tick: GameTick,
    ) -> Result<(), ImmediateEffectError> {
        if effect.source_generation() != source_generation {
            return Err(ImmediateEffectError::StaleSourceGeneration);
        }
        if effect.target_generation() != target_generation {
            return Err(ImmediateEffectError::StaleTargetGeneration);
        }
        if effect.tick() <= committed_tick {
            return Err(ImmediateEffectError::AlreadyCommitted);
        }
        let key = effect.order_key();
        if self.admitted.contains(&key) {
            return Err(ImmediateEffectError::Duplicate);
        }
        if self.pending.len() == self.capacity || self.admitted.len() == self.capacity {
            return Err(ImmediateEffectError::Full {
                capacity: self.capacity,
            });
        }
        self.admitted.insert(key.clone());
        self.pending.insert(key, effect);
        Ok(())
    }

    pub(crate) fn drain(
        &mut self,
        tick: GameTick,
        phase: TickPhase,
    ) -> Vec<ImmediateBoundaryEffect> {
        let keys = self
            .pending
            .keys()
            .filter(|key| key.tick == tick && key.phase == phase)
            .cloned()
            .collect::<Vec<_>>();
        keys.into_iter()
            .filter_map(|key| self.pending.remove(&key))
            .collect()
    }

    pub(crate) fn prune_committed(&mut self, committed_tick: GameTick) {
        self.admitted.retain(|key| key.tick > committed_tick);
    }

    pub(crate) fn has_tick_at_or_before(&self, tick: GameTick) -> bool {
        self.pending.keys().any(|key| key.tick <= tick)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ImmediateEffectOrderKey {
    tick: GameTick,
    phase: TickPhase,
    target: SimulationRegionKey,
    source: SimulationRegionKey,
    source_sequence: u64,
}

fn validate_endpoints(
    source: &SimulationRegionKey,
    target: &SimulationRegionKey,
) -> Result<(), ImmediateEffectError> {
    if source == target {
        return Err(ImmediateEffectError::SelfTarget);
    }
    if source.world() != target.world()
        || source.dimension() != target.dimension()
        || source.mapping_version() != target.mapping_version()
    {
        return Err(ImmediateEffectError::IncompatibleEndpoints);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ImmediateEffectError {
    #[error("immediate-effect queue capacity cannot be zero")]
    ZeroCapacity,
    #[error("an immediate boundary effect cannot target its source Region")]
    SelfTarget,
    #[error("immediate boundary effect endpoints are in different ownership domains")]
    IncompatibleEndpoints,
    #[error("immediate effect payload has {actual} bytes, exceeding the {maximum}-byte limit")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error("immediate boundary effect has a stale source generation")]
    StaleSourceGeneration,
    #[error("immediate boundary effect has a stale target generation")]
    StaleTargetGeneration,
    #[error("immediate boundary effect targets an already committed tick")]
    AlreadyCommitted,
    #[error("immediate boundary effect order key is already admitted")]
    Duplicate,
    #[error("immediate-effect queue reached its {capacity}-effect bound")]
    Full { capacity: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMappingVersion};

    fn region(x: i32) -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(x, 0),
            RegionMappingVersion::V1,
        )
    }

    fn effect(source: i32, target: i32, sequence: u64) -> ImmediateBoundaryEffect {
        ImmediateBoundaryEffect::new(
            ImmediateEffectHeader {
                tick: GameTick::new(1),
                phase: TickPhase::ImmediateNeighbors,
                source: region(source),
                target: region(target),
                source_generation: ActivationGeneration::INITIAL,
                target_generation: ActivationGeneration::INITIAL,
                source_sequence: sequence,
            },
            ResourceId::new("ferrite", "effect/test").unwrap(),
            vec![1],
        )
        .unwrap()
    }

    #[test]
    fn effects_sort_for_each_target_and_fence_both_generations() {
        let mut queue = ImmediateEffectQueue::new(4).unwrap();
        queue
            .admit(
                effect(2, 0, 1),
                ActivationGeneration::INITIAL,
                ActivationGeneration::INITIAL,
                GameTick::ZERO,
            )
            .unwrap();
        queue
            .admit(
                effect(1, 0, 2),
                ActivationGeneration::INITIAL,
                ActivationGeneration::INITIAL,
                GameTick::ZERO,
            )
            .unwrap();
        let effects = queue.drain(GameTick::new(1), TickPhase::ImmediateNeighbors);
        assert_eq!(effects[0].source().coordinate().x(), 1);
        assert!(
            queue
                .admit(
                    effect(1, 0, 3),
                    ActivationGeneration::new(2).unwrap(),
                    ActivationGeneration::INITIAL,
                    GameTick::ZERO,
                )
                .is_err()
        );
    }

    #[test]
    fn invalid_endpoints_and_duplicates_fail_closed() {
        assert!(
            ImmediateBoundaryEffect::new(
                ImmediateEffectHeader {
                    tick: GameTick::new(1),
                    phase: TickPhase::ImmediateNeighbors,
                    source: region(0),
                    target: region(0),
                    source_generation: ActivationGeneration::INITIAL,
                    target_generation: ActivationGeneration::INITIAL,
                    source_sequence: 1,
                },
                ResourceId::new("ferrite", "effect/test").unwrap(),
                vec![],
            )
            .is_err()
        );
        let effect = effect(1, 0, 1);
        let mut queue = ImmediateEffectQueue::new(1).unwrap();
        queue
            .admit(
                effect.clone(),
                ActivationGeneration::INITIAL,
                ActivationGeneration::INITIAL,
                GameTick::ZERO,
            )
            .unwrap();
        assert!(
            queue
                .admit(
                    effect,
                    ActivationGeneration::INITIAL,
                    ActivationGeneration::INITIAL,
                    GameTick::ZERO,
                )
                .is_err()
        );
    }
}
