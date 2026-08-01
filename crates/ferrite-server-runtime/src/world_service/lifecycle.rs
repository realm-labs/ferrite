use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMappingVersion, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::environment::weather::{WeatherData, WeatherStrengths};
use ferrite_persistence::snapshot::{SnapshotError, SnapshotRecord, SnapshotRecordKind};
use ferrite_world::generation::border::state::{BorderSnapshot, SavedBorder, WorldBorder};
use thiserror::Error;

use crate::continuity::identity::{ContinuityDomain, ContinuityGeneration, domain_id};
use crate::continuity::migration::{ContinuityMigrationError, normalize_records};
use crate::world_service::environment::{
    EnvironmentProjection, LevelEnvironment, LevelEnvironmentError,
};

const LEGACY_LEVEL_MAGIC: &[u8; 4] = b"P8L1";
const LEVEL_MAGIC: &[u8; 4] = b"FWL2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelLifecycleState {
    Constructed,
    Ready,
    Closing,
    Closed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelControlState {
    pub control_region: SimulationRegionKey,
    pub generation: ActivationGeneration,
    pub lifecycle: LevelLifecycleState,
    pub pending_work: usize,
    pub no_save: bool,
    pub border: WorldBorder,
    pub environment: LevelEnvironment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldLifecycleEvent {
    LevelConstructed {
        dimension: DimensionId,
    },
    TicketsReactivated {
        dimension: DimensionId,
    },
    LevelReady {
        dimension: DimensionId,
    },
    NetworkAdmissionClosed,
    PlayersSaved {
        count: usize,
    },
    LevelsSaved,
    PlayersRemoved {
        count: usize,
    },
    NoSaveCleared {
        dimension: DimensionId,
    },
    ClosingTicketsDeactivated {
        dimension: DimensionId,
    },
    WorkDrained,
    LevelsFlushed,
    LevelClosed {
        dimension: DimensionId,
        succeeded: bool,
    },
    SavedDataClosed,
    ResourcesClosed,
    StorageLockClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrepareOutcome {
    Waiting { pending_work: usize },
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldLifecycleState {
    Bootstrapping,
    Running,
    Closing,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldLifecycleBootstrap {
    pub world: WorldId,
    pub mapping: RegionMappingVersion,
    pub overworld: DimensionId,
    pub generation: ActivationGeneration,
    pub seed: i64,
    pub content_manifest: [u8; 32],
    pub event_capacity: usize,
}

#[derive(Debug)]
pub struct WorldLifecycleRuntime {
    world: WorldId,
    content_manifest: [u8; 32],
    order: Vec<DimensionId>,
    levels: BTreeMap<DimensionId, LevelControlState>,
    state: WorldLifecycleState,
    events: VecDeque<WorldLifecycleEvent>,
    event_capacity: usize,
}

impl WorldLifecycleRuntime {
    pub fn bootstrap(
        bootstrap: WorldLifecycleBootstrap,
        remaining_dimensions: impl IntoIterator<Item = DimensionId>,
    ) -> Result<Self, WorldLifecycleError> {
        let WorldLifecycleBootstrap {
            world,
            mapping,
            overworld,
            generation,
            seed,
            content_manifest,
            event_capacity,
        } = bootstrap;
        if event_capacity == 0 {
            return Err(WorldLifecycleError::ZeroEventCapacity);
        }
        let mut order = vec![overworld.clone()];
        let mut seen = BTreeSet::from([overworld]);
        for dimension in remaining_dimensions {
            if !seen.insert(dimension.clone()) {
                return Err(WorldLifecycleError::DuplicateDimension(dimension));
            }
            order.push(dimension);
        }
        if order.len() > event_capacity {
            return Err(WorldLifecycleError::EventCapacity);
        }
        let mut levels = BTreeMap::new();
        let mut events = VecDeque::new();
        for dimension in &order {
            levels.insert(
                dimension.clone(),
                LevelControlState {
                    control_region: SimulationRegionKey::new(
                        world,
                        dimension.clone(),
                        RegionCoord::new(0, 0),
                        mapping,
                    ),
                    generation,
                    lifecycle: LevelLifecycleState::Constructed,
                    pending_work: 0,
                    no_save: false,
                    border: WorldBorder::default(),
                    environment: LevelEnvironment::new(seed, dimension),
                },
            );
            events.push_back(WorldLifecycleEvent::LevelConstructed {
                dimension: dimension.clone(),
            });
        }
        Ok(Self {
            world,
            content_manifest,
            order,
            levels,
            state: WorldLifecycleState::Bootstrapping,
            events,
            event_capacity,
        })
    }

    pub const fn world(&self) -> WorldId {
        self.world
    }

    pub const fn state(&self) -> WorldLifecycleState {
        self.state
    }

    pub const fn content_manifest(&self) -> [u8; 32] {
        self.content_manifest
    }

    pub fn dimensions(&self) -> &[DimensionId] {
        &self.order
    }

    pub fn level(&self, dimension: &DimensionId) -> Option<&LevelControlState> {
        self.levels.get(dimension)
    }

    pub fn set_pending_work(
        &mut self,
        dimension: &DimensionId,
        pending_work: usize,
    ) -> Result<(), WorldLifecycleError> {
        if self.state == WorldLifecycleState::Closed {
            return Err(WorldLifecycleError::AlreadyClosed);
        }
        self.levels
            .get_mut(dimension)
            .ok_or_else(|| WorldLifecycleError::UnknownDimension(dimension.clone()))?
            .pending_work = pending_work;
        Ok(())
    }

    pub fn tick_environment(
        &mut self,
        dimension: &DimensionId,
    ) -> Result<EnvironmentProjection, WorldLifecycleError> {
        if matches!(
            self.state,
            WorldLifecycleState::Bootstrapping | WorldLifecycleState::Closed
        ) {
            return Err(WorldLifecycleError::InvalidWorldState);
        }
        self.levels
            .get_mut(dimension)
            .ok_or_else(|| WorldLifecycleError::UnknownDimension(dimension.clone()))?
            .environment
            .tick(dimension)
            .map_err(Into::into)
    }

    pub fn tick_border(
        &mut self,
        dimension: &DimensionId,
    ) -> Result<BorderSnapshot, WorldLifecycleError> {
        if matches!(
            self.state,
            WorldLifecycleState::Bootstrapping | WorldLifecycleState::Closed
        ) {
            return Err(WorldLifecycleError::InvalidWorldState);
        }
        let level = self
            .levels
            .get_mut(dimension)
            .ok_or_else(|| WorldLifecycleError::UnknownDimension(dimension.clone()))?;
        level.border.tick_if_running(true);
        Ok(level.border.snapshot())
    }

    pub fn prepare_levels(&mut self) -> Result<PrepareOutcome, WorldLifecycleError> {
        if self.state != WorldLifecycleState::Bootstrapping {
            return Err(WorldLifecycleError::InvalidWorldState);
        }
        let pending_work = self.levels.values().map(|level| level.pending_work).sum();
        if pending_work != 0 {
            return Ok(PrepareOutcome::Waiting { pending_work });
        }
        self.reserve_events(self.order.len().saturating_mul(2))?;
        for dimension in &self.order {
            let level = self
                .levels
                .get_mut(dimension)
                .expect("ordered dimension has a control state");
            level.lifecycle = LevelLifecycleState::Ready;
            self.events
                .push_back(WorldLifecycleEvent::TicketsReactivated {
                    dimension: dimension.clone(),
                });
            self.events.push_back(WorldLifecycleEvent::LevelReady {
                dimension: dimension.clone(),
            });
        }
        self.state = WorldLifecycleState::Running;
        Ok(PrepareOutcome::Ready)
    }

    pub fn border_mut(
        &mut self,
        control_region: &SimulationRegionKey,
        generation: ActivationGeneration,
    ) -> Result<&mut WorldBorder, WorldLifecycleError> {
        let level = self
            .levels
            .get_mut(control_region.dimension())
            .ok_or_else(|| {
                WorldLifecycleError::UnknownDimension(control_region.dimension().clone())
            })?;
        if &level.control_region != control_region {
            return Err(WorldLifecycleError::WrongControlRegion);
        }
        if level.generation != generation {
            return Err(WorldLifecycleError::StaleGeneration);
        }
        Ok(&mut level.border)
    }

    pub fn set_no_save(
        &mut self,
        control_region: &SimulationRegionKey,
        generation: ActivationGeneration,
        no_save: bool,
    ) -> Result<(), WorldLifecycleError> {
        let level = self
            .levels
            .get_mut(control_region.dimension())
            .ok_or_else(|| {
                WorldLifecycleError::UnknownDimension(control_region.dimension().clone())
            })?;
        if &level.control_region != control_region {
            return Err(WorldLifecycleError::WrongControlRegion);
        }
        if level.generation != generation {
            return Err(WorldLifecycleError::StaleGeneration);
        }
        level.no_save = no_save;
        Ok(())
    }

    pub fn begin_shutdown(&mut self, player_count: usize) -> Result<(), WorldLifecycleError> {
        if self.state != WorldLifecycleState::Running {
            return Err(WorldLifecycleError::InvalidWorldState);
        }
        let event_count = 4usize.saturating_add(self.order.len().saturating_mul(2));
        self.reserve_events(event_count)?;
        self.events
            .push_back(WorldLifecycleEvent::NetworkAdmissionClosed);
        self.events.push_back(WorldLifecycleEvent::PlayersSaved {
            count: player_count,
        });
        self.events.push_back(WorldLifecycleEvent::LevelsSaved);
        self.events.push_back(WorldLifecycleEvent::PlayersRemoved {
            count: player_count,
        });
        for dimension in &self.order {
            let level = self
                .levels
                .get_mut(dimension)
                .expect("ordered dimension has a control state");
            level.no_save = false;
            level.lifecycle = LevelLifecycleState::Closing;
            self.events.push_back(WorldLifecycleEvent::NoSaveCleared {
                dimension: dimension.clone(),
            });
            self.events
                .push_back(WorldLifecycleEvent::ClosingTicketsDeactivated {
                    dimension: dimension.clone(),
                });
        }
        self.state = WorldLifecycleState::Closing;
        Ok(())
    }

    pub fn finish_shutdown(
        &mut self,
        close_results: &BTreeMap<DimensionId, bool>,
    ) -> Result<(), WorldLifecycleError> {
        if self.state != WorldLifecycleState::Closing {
            return Err(WorldLifecycleError::InvalidWorldState);
        }
        if self.levels.values().any(|level| level.pending_work != 0) {
            return Err(WorldLifecycleError::WorkStillPending);
        }
        for dimension in &self.order {
            if !close_results.contains_key(dimension) {
                return Err(WorldLifecycleError::MissingCloseResult(dimension.clone()));
            }
        }
        self.reserve_events(self.order.len().saturating_add(5))?;
        self.events.push_back(WorldLifecycleEvent::WorkDrained);
        self.events.push_back(WorldLifecycleEvent::LevelsFlushed);
        for dimension in &self.order {
            let succeeded = close_results[dimension];
            self.levels
                .get_mut(dimension)
                .expect("ordered dimension has a control state")
                .lifecycle = LevelLifecycleState::Closed;
            self.events.push_back(WorldLifecycleEvent::LevelClosed {
                dimension: dimension.clone(),
                succeeded,
            });
        }
        self.events.push_back(WorldLifecycleEvent::SavedDataClosed);
        self.events.push_back(WorldLifecycleEvent::ResourcesClosed);
        self.events
            .push_back(WorldLifecycleEvent::StorageLockClosed);
        self.state = WorldLifecycleState::Closed;
        Ok(())
    }

    pub fn level_records(&self) -> Result<Vec<SnapshotRecord>, WorldLifecycleError> {
        if self.levels.values().any(|level| level.pending_work != 0) {
            return Err(WorldLifecycleError::WorkStillPending);
        }
        self.order
            .iter()
            .map(|dimension| {
                let level = &self.levels[dimension];
                encode_level_record(dimension, level)
            })
            .collect()
    }

    pub fn level_record(
        &self,
        control_region: &SimulationRegionKey,
        generation: ActivationGeneration,
    ) -> Result<SnapshotRecord, WorldLifecycleError> {
        let level = self.levels.get(control_region.dimension()).ok_or_else(|| {
            WorldLifecycleError::UnknownDimension(control_region.dimension().clone())
        })?;
        if &level.control_region != control_region {
            return Err(WorldLifecycleError::WrongControlRegion);
        }
        if level.generation != generation {
            return Err(WorldLifecycleError::StaleGeneration);
        }
        if level.pending_work != 0 {
            return Err(WorldLifecycleError::WorkStillPending);
        }
        encode_level_record(control_region.dimension(), level)
    }

    pub fn apply_level_records(
        &mut self,
        records: &[SnapshotRecord],
    ) -> Result<(), WorldLifecycleError> {
        let normalized = normalize_records(records)?;
        let mut seen = BTreeSet::new();
        let mut decoded = Vec::new();
        for record in normalized.records() {
            let Some(level) = decode_level_record(record)? else {
                continue;
            };
            if !seen.insert(level.dimension.clone()) {
                return Err(WorldLifecycleError::DuplicateDimension(level.dimension));
            }
            if !self.levels.contains_key(&level.dimension) {
                return Err(WorldLifecycleError::UnknownDimension(level.dimension));
            }
            decoded.push(level);
        }
        for decoded in decoded {
            let level = self
                .levels
                .get_mut(&decoded.dimension)
                .expect("validated dimension");
            level.border = WorldBorder::from_saved(decoded.border, 0);
            level.no_save = decoded.no_save;
            if let Some(environment) = decoded.environment {
                level.environment = environment;
            }
        }
        Ok(())
    }

    pub fn take_events(&mut self, maximum: usize) -> Vec<WorldLifecycleEvent> {
        let count = maximum.min(self.events.len());
        self.events.drain(..count).collect()
    }

    fn reserve_events(&self, count: usize) -> Result<(), WorldLifecycleError> {
        if self.events.len().saturating_add(count) > self.event_capacity {
            Err(WorldLifecycleError::EventCapacity)
        } else {
            Ok(())
        }
    }
}

#[must_use]
pub fn level_domain() -> ResourceId {
    domain_id(ContinuityDomain::WorldLevel, ContinuityGeneration::Current)
}

fn encode_level_record(
    dimension: &DimensionId,
    level: &LevelControlState,
) -> Result<SnapshotRecord, WorldLifecycleError> {
    let saved = level.border.saved();
    let mut value = Vec::new();
    value.extend_from_slice(LEVEL_MAGIC);
    value.push(u8::from(level.no_save));
    push_f64(&mut value, saved.center_x);
    push_f64(&mut value, saved.center_z);
    push_f64(&mut value, saved.size);
    push_f64(&mut value, saved.target_size);
    value.extend_from_slice(&saved.remaining_ticks.to_be_bytes());
    push_f64(&mut value, saved.damage_per_block);
    push_f64(&mut value, saved.safe_zone);
    value.extend_from_slice(&saved.warning_blocks.to_be_bytes());
    value.extend_from_slice(&saved.warning_time.to_be_bytes());
    let environment = level.environment;
    value.extend_from_slice(&environment.game_time().to_be_bytes());
    value.extend_from_slice(&environment.day_time().to_be_bytes());
    let weather = environment.weather();
    value.extend_from_slice(&weather.clear_weather_time.to_be_bytes());
    value.extend_from_slice(&weather.rain_time.to_be_bytes());
    value.extend_from_slice(&weather.thunder_time.to_be_bytes());
    value.push(u8::from(weather.raining));
    value.push(u8::from(weather.thundering));
    let strengths = environment.strengths();
    for strength in [
        strengths.previous_rain,
        strengths.rain,
        strengths.previous_thunder,
        strengths.thunder,
    ] {
        value.extend_from_slice(&strength.to_bits().to_be_bytes());
    }
    value.extend_from_slice(&environment.random_state().to_be_bytes());
    SnapshotRecord::new(
        SnapshotRecordKind::Extension,
        level_domain(),
        dimension.to_string().into_bytes(),
        value,
    )
    .map_err(Into::into)
}

struct DecodedLevel {
    dimension: DimensionId,
    border: SavedBorder,
    no_save: bool,
    environment: Option<LevelEnvironment>,
}

fn decode_level_record(
    record: &SnapshotRecord,
) -> Result<Option<DecodedLevel>, WorldLifecycleError> {
    if record.kind() != SnapshotRecordKind::Extension || record.domain() != &level_domain() {
        return Ok(None);
    }
    let dimension = std::str::from_utf8(record.key())
        .map_err(|_| WorldLifecycleError::InvalidLevelRecord)?
        .parse()
        .map_err(|_| WorldLifecycleError::InvalidLevelRecord)?;
    let mut cursor = Cursor::new(record.value());
    let legacy = match cursor.fixed::<4>()? {
        value if value == *LEGACY_LEVEL_MAGIC => true,
        value if value == *LEVEL_MAGIC => false,
        _ => return Err(WorldLifecycleError::InvalidLevelRecord),
    };
    let no_save = match cursor.u8()? {
        0 => false,
        1 => true,
        _ => return Err(WorldLifecycleError::InvalidLevelRecord),
    };
    let saved = SavedBorder {
        center_x: cursor.f64()?,
        center_z: cursor.f64()?,
        size: cursor.f64()?,
        target_size: cursor.f64()?,
        remaining_ticks: cursor.i64()?,
        damage_per_block: cursor.f64()?,
        safe_zone: cursor.f64()?,
        warning_blocks: cursor.i32()?,
        warning_time: cursor.i32()?,
    };
    let environment = if legacy {
        None
    } else {
        let game_time = cursor.i64()?;
        let day_time = cursor.i64()?;
        let weather = WeatherData {
            clear_weather_time: cursor.i32()?,
            rain_time: cursor.i32()?,
            thunder_time: cursor.i32()?,
            raining: cursor.bool()?,
            thundering: cursor.bool()?,
        };
        let strengths = WeatherStrengths {
            previous_rain: cursor.f32()?,
            rain: cursor.f32()?,
            previous_thunder: cursor.f32()?,
            thunder: cursor.f32()?,
        };
        Some(LevelEnvironment::from_durable(
            game_time,
            day_time,
            weather,
            strengths,
            cursor.u64()?,
        ))
    };
    cursor.finish()?;
    Ok(Some(DecodedLevel {
        dimension,
        border: saved,
        no_save,
        environment,
    }))
}

fn push_f64(output: &mut Vec<u8>, value: f64) {
    output.extend_from_slice(&value.to_bits().to_be_bytes());
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], WorldLifecycleError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(WorldLifecycleError::InvalidLevelRecord)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(WorldLifecycleError::InvalidLevelRecord)?;
        self.offset = end;
        Ok(value)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], WorldLifecycleError> {
        self.take(N)?
            .try_into()
            .map_err(|_| WorldLifecycleError::InvalidLevelRecord)
    }

    fn bool(&mut self) -> Result<bool, WorldLifecycleError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(WorldLifecycleError::InvalidLevelRecord),
        }
    }

    fn u64(&mut self) -> Result<u64, WorldLifecycleError> {
        Ok(u64::from_be_bytes(self.fixed()?))
    }

    fn f32(&mut self) -> Result<f32, WorldLifecycleError> {
        Ok(f32::from_bits(u32::from_be_bytes(self.fixed()?)))
    }

    fn u8(&mut self) -> Result<u8, WorldLifecycleError> {
        Ok(self.take(1)?[0])
    }

    fn f64(&mut self) -> Result<f64, WorldLifecycleError> {
        Ok(f64::from_bits(u64::from_be_bytes(self.fixed()?)))
    }

    fn i64(&mut self) -> Result<i64, WorldLifecycleError> {
        Ok(i64::from_be_bytes(self.fixed()?))
    }

    fn i32(&mut self) -> Result<i32, WorldLifecycleError> {
        Ok(i32::from_be_bytes(self.fixed()?))
    }

    fn finish(self) -> Result<(), WorldLifecycleError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(WorldLifecycleError::InvalidLevelRecord)
        }
    }
}

#[derive(Debug, Error)]
pub enum WorldLifecycleError {
    #[error("world lifecycle event capacity cannot be zero")]
    ZeroEventCapacity,
    #[error("world lifecycle event capacity is exhausted")]
    EventCapacity,
    #[error("dimension {0} appears more than once")]
    DuplicateDimension(DimensionId),
    #[error("dimension {0} is not part of this world")]
    UnknownDimension(DimensionId),
    #[error("level-global mutation targets another control Region")]
    WrongControlRegion,
    #[error("level-global mutation uses a stale activation generation")]
    StaleGeneration,
    #[error("world lifecycle operation is invalid in the current state")]
    InvalidWorldState,
    #[error("world lifecycle is already closed")]
    AlreadyClosed,
    #[error("world lifecycle still has pending work")]
    WorkStillPending,
    #[error("shutdown has no close result for dimension {0}")]
    MissingCloseResult(DimensionId),
    #[error("level-global durable record is invalid")]
    InvalidLevelRecord,
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Migration(#[from] ContinuityMigrationError),
    #[error(transparent)]
    Environment(#[from] LevelEnvironmentError),
}
