//! Entity UUID, section visibility, passenger tree, and removal transactions.

use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::identity::StableEntityId;
use thiserror::Error;

pub const BOARDING_COOLDOWN_TICKS: u8 = 60;
pub const EMPTY_LEVEL_TICK_LIMIT: u16 = 300;
pub const CRAMMING_DAMAGE: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SectionKey {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Visibility {
    Hidden,
    Accessible,
    Ticking,
}

impl Visibility {
    #[must_use]
    pub const fn tracked(self) -> bool {
        !matches!(self, Self::Hidden)
    }

    #[must_use]
    pub const fn ticking(self) -> bool {
        matches!(self, Self::Ticking)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityClass {
    Ordinary,
    Player,
    AlwaysTicking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalReason {
    Killed,
    Discarded,
    UnloadedToChunk,
    UnloadedWithPlayer,
    ChangedDimension,
}

impl RemovalReason {
    #[must_use]
    pub const fn destroys(self) -> bool {
        matches!(self, Self::Killed | Self::Discarded)
    }

    #[must_use]
    pub const fn should_save(self) -> bool {
        matches!(self, Self::UnloadedToChunk)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEffect {
    KnownUuidAdded,
    DuplicateOrdinaryRejected,
    DuplicatePlayerDiscarded,
    SectionAdded(SectionKey),
    SectionRemoved(SectionKey),
    CallbackInstalled,
    Created,
    TrackingStarted,
    TrackingStopped,
    TickingStarted,
    TickingStopped,
    DynamicListenerMoved,
    SectionChanged,
    RideStopped,
    StandingPoseSet,
    PassengerLinked,
    MountEvent,
    IndirectPlayerCriterion,
    DismountEvent,
    BoardingCooldownSet(u8),
    TickRoot,
    TickPassenger,
    DespawnChecked,
    RemovedCallback(RemovalReason),
    RemovalHook(RemovalReason),
    Destroyed,
    KnownUuidRemoved,
    CallbackCleared,
    SavedRoot,
    SameDimensionPlaced(StableEntityId),
    RidingPlayerCorrected,
    PassengerTransferred(StableEntityId),
    PassengerRemounted(StableEntityId),
    DestinationRootCreated,
    DestinationActivityReset,
    PostTransition,
    SpectatorsTransferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRecord {
    pub id: StableEntityId,
    pub class: EntityClass,
    pub section: SectionKey,
    pub visibility: Visibility,
    pub removed: Option<RemovalReason>,
    pub frozen: bool,
    pub serializable: bool,
    pub vehicle: Option<StableEntityId>,
    pub passengers: Vec<StableEntityId>,
    pub boarding_cooldown: u8,
    pub tick_count: u64,
}

impl EntityRecord {
    #[must_use]
    pub const fn new(
        id: StableEntityId,
        class: EntityClass,
        section: SectionKey,
        visibility: Visibility,
    ) -> Self {
        Self {
            id,
            class,
            section,
            visibility,
            removed: None,
            frozen: false,
            serializable: true,
            vehicle: None,
            passengers: Vec::new(),
            boarding_cooldown: 0,
            tick_count: 0,
        }
    }

    #[must_use]
    pub const fn effective_visibility(&self) -> Visibility {
        if matches!(self.class, EntityClass::AlwaysTicking) {
            Visibility::Ticking
        } else {
            self.visibility
        }
    }
}

#[derive(Debug, Default)]
pub struct EntityLifecycle {
    entities: BTreeMap<StableEntityId, EntityRecord>,
    known_uuids: BTreeSet<StableEntityId>,
}

impl EntityLifecycle {
    pub fn add(
        &mut self,
        entity: EntityRecord,
        created: bool,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        if entity.removed.is_some() {
            return Err(LifecycleError::RemovedInsertion(entity.id));
        }
        let mut effects = Vec::new();
        if self.known_uuids.contains(&entity.id) {
            if !matches!(entity.class, EntityClass::Player) {
                return Ok(vec![LifecycleEffect::DuplicateOrdinaryRejected]);
            }
            let existing_id = entity.id;
            self.stop_riding(existing_id, &mut effects)?;
            effects.extend(self.remove(existing_id, RemovalReason::Discarded)?);
            effects.push(LifecycleEffect::DuplicatePlayerDiscarded);
        }
        let visibility = entity.effective_visibility();
        let section = entity.section;
        self.known_uuids.insert(entity.id);
        self.entities.insert(entity.id, entity);
        effects.extend([
            LifecycleEffect::KnownUuidAdded,
            LifecycleEffect::SectionAdded(section),
            LifecycleEffect::CallbackInstalled,
        ]);
        if created {
            effects.push(LifecycleEffect::Created);
        }
        if visibility.tracked() {
            effects.push(LifecycleEffect::TrackingStarted);
        }
        if visibility.ticking() {
            effects.push(LifecycleEffect::TickingStarted);
        }
        Ok(effects)
    }

    pub fn move_section(
        &mut self,
        id: StableEntityId,
        section: SectionKey,
        visibility: Visibility,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        let entity = self.entity_mut(id)?;
        let old_section = entity.section;
        let old_visibility = entity.effective_visibility();
        entity.section = section;
        entity.visibility = visibility;
        let next_visibility = entity.effective_visibility();
        let mut effects = vec![
            LifecycleEffect::SectionRemoved(old_section),
            LifecycleEffect::SectionAdded(section),
        ];
        if old_visibility.tracked() && !next_visibility.tracked() {
            effects.push(LifecycleEffect::TrackingStopped);
        } else if !old_visibility.tracked() && next_visibility.tracked() {
            effects.push(LifecycleEffect::TrackingStarted);
        }
        if old_visibility.ticking() && !next_visibility.ticking() {
            effects.push(LifecycleEffect::TickingStopped);
        } else if !old_visibility.ticking() && next_visibility.ticking() {
            effects.push(LifecycleEffect::TickingStarted);
        }
        if next_visibility.tracked() {
            effects.push(LifecycleEffect::DynamicListenerMoved);
        }
        effects.push(LifecycleEffect::SectionChanged);
        Ok(effects)
    }

    pub fn start_riding(
        &mut self,
        passenger: StableEntityId,
        vehicle: StableEntityId,
        admission: RideAdmission,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        if passenger == vehicle || self.is_descendant(passenger, vehicle)? {
            return Err(LifecycleError::RideCycle);
        }
        let passenger_state = self.entity(passenger)?;
        let passenger_class = passenger_state.class;
        let boarding_cooldown = passenger_state.boarding_cooldown;
        if !admission.force && (admission.shifting || boarding_cooldown > 0) {
            return Err(LifecycleError::RideRejected);
        }
        let vehicle_state = self.entity(vehicle)?;
        if !admission.vehicle_accepts
            || !admission.ride_admitted
            || (!vehicle_state.serializable && admission.server_side)
        {
            return Err(LifecycleError::RideRejected);
        }
        let mut effects = Vec::new();
        self.stop_riding(passenger, &mut effects)?;
        effects.push(LifecycleEffect::StandingPoseSet);
        self.entity_mut(passenger)?.vehicle = Some(vehicle);
        let first_passenger_class = self
            .entity(vehicle)?
            .passengers
            .first()
            .and_then(|first| self.entities.get(first))
            .map(|first| first.class);
        let insert_at_front = matches!(passenger_class, EntityClass::Player)
            && !matches!(first_passenger_class, Some(EntityClass::Player));
        let passengers = &mut self.entity_mut(vehicle)?.passengers;
        if insert_at_front {
            passengers.insert(0, passenger);
        } else {
            passengers.push(passenger);
        }
        effects.extend([
            LifecycleEffect::PassengerLinked,
            LifecycleEffect::MountEvent,
            LifecycleEffect::IndirectPlayerCriterion,
        ]);
        Ok(effects)
    }

    pub fn eject_passengers(
        &mut self,
        vehicle: StableEntityId,
    ) -> Result<Vec<(StableEntityId, Vec<LifecycleEffect>)>, LifecycleError> {
        let passengers = self.entity(vehicle)?.passengers.clone();
        passengers
            .into_iter()
            .rev()
            .map(|passenger| {
                let mut effects = Vec::new();
                self.stop_riding(passenger, &mut effects)?;
                Ok((passenger, effects))
            })
            .collect()
    }

    pub fn tick(
        &mut self,
        entity_ticking_sections: &BTreeSet<SectionKey>,
    ) -> Result<Vec<(StableEntityId, LifecycleEffect)>, LifecycleError> {
        let ids = self.entities.keys().copied().collect::<Vec<_>>();
        let mut effects = Vec::new();
        for id in ids {
            let entity = self.entity(id)?;
            if entity.removed.is_some() || entity.frozen {
                continue;
            }
            let class = entity.class;
            let section = entity.section;
            let vehicle = entity.vehicle;
            if let Some(vehicle) = vehicle {
                let valid_vehicle = self
                    .entities
                    .get(&vehicle)
                    .is_some_and(|vehicle| vehicle.passengers.contains(&id));
                if valid_vehicle {
                    continue;
                }
                let mut ignored = Vec::new();
                self.stop_riding(id, &mut ignored)?;
            }
            effects.push((id, LifecycleEffect::DespawnChecked));
            if matches!(class, EntityClass::Player) || entity_ticking_sections.contains(&section) {
                self.tick_tree(id, false, &mut effects)?;
            }
        }
        Ok(effects)
    }

    pub fn remove(
        &mut self,
        id: StableEntityId,
        reason: RemovalReason,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        let stored = self.entity(id)?.removed;
        if stored.is_none() {
            self.entity_mut(id)?.removed = Some(reason);
        }
        let mut effects = Vec::new();
        if reason.destroys() {
            self.stop_riding(id, &mut effects)?;
        }
        for (_, passenger_effects) in self.eject_passengers(id)? {
            effects.extend(passenger_effects);
        }
        let entity = self.entity(id)?;
        let effective = entity.effective_visibility();
        effects.extend([
            LifecycleEffect::RemovedCallback(reason),
            LifecycleEffect::RemovalHook(reason),
            LifecycleEffect::SectionRemoved(entity.section),
        ]);
        if effective.ticking() {
            effects.push(LifecycleEffect::TickingStopped);
        }
        if effective.tracked() {
            effects.push(LifecycleEffect::TrackingStopped);
        }
        if reason.destroys() {
            effects.push(LifecycleEffect::Destroyed);
        }
        effects.extend([
            LifecycleEffect::KnownUuidRemoved,
            LifecycleEffect::CallbackCleared,
        ]);
        self.known_uuids.remove(&id);
        Ok(effects)
    }

    pub fn unload_root(
        &mut self,
        root: StableEntityId,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        let mut effects = Vec::new();
        if self.entity(root)?.serializable {
            effects.push(LifecycleEffect::SavedRoot);
        }
        let tree = self.passengers_and_self(root)?;
        for id in tree {
            effects.extend(self.remove(id, RemovalReason::UnloadedToChunk)?);
        }
        Ok(effects)
    }

    pub fn teleport_same_dimension(
        &self,
        root: StableEntityId,
        riding_player: bool,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        let mut postorder = Vec::new();
        self.passenger_postorder(root, &mut postorder)?;
        let mut effects = postorder
            .into_iter()
            .map(LifecycleEffect::SameDimensionPlaced)
            .collect::<Vec<_>>();
        if riding_player {
            effects.push(LifecycleEffect::RidingPlayerCorrected);
        }
        effects.push(LifecycleEffect::PostTransition);
        Ok(effects)
    }

    pub fn teleport_cross_dimension(
        &mut self,
        root: StableEntityId,
        destination_created: bool,
    ) -> Result<Vec<LifecycleEffect>, LifecycleError> {
        let mut destination = self.entity(root)?.clone();
        let passengers = self.entity(root)?.passengers.clone();
        let mut effects = Vec::new();
        for passenger in passengers.iter().rev().copied() {
            self.stop_riding(passenger, &mut effects)?;
            effects.push(LifecycleEffect::PassengerTransferred(passenger));
        }
        if !destination_created {
            return Ok(effects);
        }
        effects.push(LifecycleEffect::DestinationRootCreated);
        effects.extend(self.remove(root, RemovalReason::ChangedDimension)?);
        destination.removed = None;
        destination.vehicle = None;
        destination.passengers.clear();
        effects.extend(self.add(destination, false)?);
        effects.push(LifecycleEffect::SameDimensionPlaced(root));
        for passenger in passengers {
            self.entity_mut(passenger)?.vehicle = Some(root);
            self.entity_mut(root)?.passengers.push(passenger);
            effects.push(LifecycleEffect::PassengerRemounted(passenger));
        }
        effects.extend([
            LifecycleEffect::DestinationActivityReset,
            LifecycleEffect::PostTransition,
            LifecycleEffect::SpectatorsTransferred,
        ]);
        Ok(effects)
    }

    pub fn entity(&self, id: StableEntityId) -> Result<&EntityRecord, LifecycleError> {
        self.entities
            .get(&id)
            .ok_or(LifecycleError::UnknownEntity(id))
    }

    fn entity_mut(&mut self, id: StableEntityId) -> Result<&mut EntityRecord, LifecycleError> {
        self.entities
            .get_mut(&id)
            .ok_or(LifecycleError::UnknownEntity(id))
    }

    fn stop_riding(
        &mut self,
        passenger: StableEntityId,
        effects: &mut Vec<LifecycleEffect>,
    ) -> Result<(), LifecycleError> {
        let Some(vehicle) = self.entity(passenger)?.vehicle else {
            return Ok(());
        };
        self.entity_mut(passenger)?.vehicle = None;
        if let Ok(vehicle_state) = self.entity_mut(vehicle) {
            vehicle_state.passengers.retain(|id| *id != passenger);
        }
        self.entity_mut(passenger)?.boarding_cooldown = BOARDING_COOLDOWN_TICKS;
        effects.extend([
            LifecycleEffect::RideStopped,
            LifecycleEffect::BoardingCooldownSet(BOARDING_COOLDOWN_TICKS),
            LifecycleEffect::DismountEvent,
        ]);
        Ok(())
    }

    fn is_descendant(
        &self,
        passenger: StableEntityId,
        candidate_vehicle: StableEntityId,
    ) -> Result<bool, LifecycleError> {
        let mut cursor = Some(candidate_vehicle);
        while let Some(id) = cursor {
            if id == passenger {
                return Ok(true);
            }
            cursor = self.entity(id)?.vehicle;
        }
        Ok(false)
    }

    fn tick_tree(
        &mut self,
        id: StableEntityId,
        passenger: bool,
        effects: &mut Vec<(StableEntityId, LifecycleEffect)>,
    ) -> Result<(), LifecycleError> {
        let children = self.entity(id)?.passengers.clone();
        let next_tick = self.entity(id)?.tick_count.wrapping_add(1);
        self.entity_mut(id)?.tick_count = next_tick;
        effects.push((
            id,
            if passenger {
                LifecycleEffect::TickPassenger
            } else {
                LifecycleEffect::TickRoot
            },
        ));
        for child in children {
            if self.entity(child)?.vehicle == Some(id)
                && (matches!(self.entity(child)?.class, EntityClass::Player)
                    || self.entity(child)?.effective_visibility().ticking())
            {
                self.tick_tree(child, true, effects)?;
            } else if self.entity(child)?.vehicle != Some(id) {
                let mut ignored = Vec::new();
                self.stop_riding(child, &mut ignored)?;
            }
        }
        Ok(())
    }

    fn passengers_and_self(
        &self,
        root: StableEntityId,
    ) -> Result<Vec<StableEntityId>, LifecycleError> {
        let mut output = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            output.push(id);
            stack.extend(self.entity(id)?.passengers.iter().rev());
        }
        Ok(output)
    }

    fn passenger_postorder(
        &self,
        id: StableEntityId,
        output: &mut Vec<StableEntityId>,
    ) -> Result<(), LifecycleError> {
        for passenger in &self.entity(id)?.passengers {
            self.passenger_postorder(*passenger, output)?;
        }
        output.push(id);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RideAdmission {
    pub force: bool,
    pub shifting: bool,
    pub vehicle_accepts: bool,
    pub ride_admitted: bool,
    pub server_side: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrammingInput {
    pub server_side: bool,
    pub limit: usize,
    pub raw_neighbors: usize,
    pub nonpassenger_neighbors: usize,
    pub draw_four: u8,
    pub damage_admitted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrammingOutcome {
    pub damage: u8,
    pub pushed_neighbors: usize,
}

#[must_use]
pub const fn cramming(input: CrammingInput) -> CrammingOutcome {
    let damage = if input.server_side
        && input.limit > 0
        && input.raw_neighbors >= input.limit
        && input.draw_four == 0
        && input.nonpassenger_neighbors >= input.limit
        && input.damage_admitted
    {
        CRAMMING_DAMAGE
    } else {
        0
    };
    CrammingOutcome {
        damage,
        pushed_neighbors: input.raw_neighbors,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LifecycleError {
    #[error("removed entity {0} cannot be inserted")]
    RemovedInsertion(StableEntityId),
    #[error("entity {0} is not managed")]
    UnknownEntity(StableEntityId),
    #[error("ride graph would contain a cycle")]
    RideCycle,
    #[error("ride admission failed")]
    RideRejected,
}
