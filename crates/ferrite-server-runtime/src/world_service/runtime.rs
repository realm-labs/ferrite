use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_persistence::recovery::RecoveredRegion;
use ferrite_persistence::snapshot::{
    PersistenceRevision, RegionCommitSnapshot, RegionRecoveryPoint, RegionSnapshotHeader,
    SnapshotError, SnapshotRecord,
};
use ferrite_persistence::store::CommitReceipt;
use ferrite_world::chunk::{ChunkColumn, ChunkRevision};
use ferrite_world::generation::status::ChunkStatus;
use ferrite_world::id::BlockStateId;
use ferrite_world::projection::{ChunkProjectionError, ChunkSnapshot};
use ferrite_world::region::{RegionVoxelError, RegionVoxelState};
use thiserror::Error;

use crate::continuity::migration::{ContinuityMigrationError, normalize_recovery_point};
use crate::world_service::continuity::{
    WorldServiceContinuityError, canonical_state_hash, chunk_domain, decode_chunk_record,
    encode_chunk_record, materialized_records,
};
use crate::world_service::model::{
    ChunkActivity, ChunkEvent, ChunkEventKind, ChunkLifecycle, GENERATION_CONTINUATION_VERSION_V1,
    GenerationOutcome, GenerationRequest, GenerationResult, PendingGeneration, PendingUnload,
    PendingUnloadIdentity, PreparedWorldSave, TicketOutcome, WorldServiceRuntimeConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldBlockWrite {
    pub position: BlockPos,
    pub state: BlockStateId,
}

#[derive(Debug)]
pub struct WorldServiceRegionRuntime {
    key: SimulationRegionKey,
    generation: ActivationGeneration,
    config: WorldServiceRuntimeConfig,
    voxels: RegionVoxelState,
    lifecycle: BTreeMap<ChunkPos, ChunkLifecycle>,
    auxiliary_records: Vec<SnapshotRecord>,
    events: VecDeque<ChunkEvent>,
    next_request_id: u64,
    next_unload_token: u64,
    next_event_sequence: u64,
}

impl WorldServiceRegionRuntime {
    pub fn new(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        config: WorldServiceRuntimeConfig,
    ) -> Result<Self, WorldServiceRuntimeError> {
        validate_config(&key, &config)?;
        let voxels = RegionVoxelState::new(key.clone(), config.mapping, config.layout)?;
        Ok(Self {
            key,
            generation,
            config,
            voxels,
            lifecycle: BTreeMap::new(),
            auxiliary_records: Vec::new(),
            events: VecDeque::new(),
            next_request_id: 1,
            next_unload_token: 1,
            next_event_sequence: 1,
        })
    }

    pub fn restore_recovered(
        recovered: RecoveredRegion,
        config: WorldServiceRuntimeConfig,
    ) -> Result<Self, WorldServiceRuntimeError> {
        let key = recovered.key().clone();
        let generation = recovered.generation();
        let point = recovered.into_recovery_point();
        Self::restore(key, generation, &point, config)
    }

    pub fn restore(
        key: SimulationRegionKey,
        generation: ActivationGeneration,
        point: &RegionRecoveryPoint,
        config: WorldServiceRuntimeConfig,
    ) -> Result<Self, WorldServiceRuntimeError> {
        let normalized = normalize_recovery_point(point)?;
        let point = &normalized;
        validate_config(&key, &config)?;
        let header = point.snapshot().header();
        if header.key != key {
            return Err(WorldServiceRuntimeError::WrongRegion);
        }
        if generation <= header.generation {
            return Err(WorldServiceRuntimeError::GenerationNotNewer);
        }
        if header.region_side_chunks != config.region_side_chunks {
            return Err(WorldServiceRuntimeError::RegionSideMismatch);
        }
        if header.content_manifest != config.content_manifest {
            return Err(WorldServiceRuntimeError::ContentManifestMismatch);
        }
        if canonical_state_hash(point.snapshot().records()) != header.state_hash {
            return Err(WorldServiceRuntimeError::StateHashMismatch);
        }
        let records = materialized_records(point);
        let mut runtime = Self::new(key, generation, config)?;
        let mut maximum_unload_token = 0;
        let mut maximum_request_id = 0;
        for record in records {
            if let Some((chunk, mut lifecycle)) = decode_chunk_record(&record)? {
                let position = chunk.position();
                if lifecycle.pending_generation.is_some_and(|pending| {
                    pending.content_manifest != runtime.config.content_manifest
                }) {
                    return Err(WorldServiceRuntimeError::ContentManifestMismatch);
                }
                runtime.validate_chunk(position)?;
                if chunk.layout() != runtime.config.layout {
                    return Err(WorldServiceRuntimeError::LayoutMismatch);
                }
                if runtime.lifecycle.len() >= runtime.config.chunk_capacity {
                    return Err(WorldServiceRuntimeError::ChunkCapacity);
                }
                if lifecycle.status >= ChunkStatus::InitializeLight && chunk.light().is_none() {
                    lifecycle.status = ChunkStatus::Features;
                    lifecycle.activity = ChunkActivity::Dormant;
                    lifecycle.pending_generation = None;
                    lifecycle.pending_unload = None;
                }
                if runtime.lifecycle.insert(position, lifecycle).is_some() {
                    return Err(WorldServiceRuntimeError::DuplicateChunk(position));
                }
                runtime.voxels.insert_chunk(chunk)?;
                maximum_unload_token = maximum_unload_token
                    .max(lifecycle.pending_unload.map_or(0, |pending| pending.token));
                maximum_request_id = maximum_request_id.max(
                    lifecycle
                        .pending_generation
                        .map_or(0, |pending| pending.request_id),
                );
            } else {
                runtime.auxiliary_records.push(record);
            }
        }
        runtime.next_unload_token = maximum_unload_token
            .checked_add(1)
            .ok_or(WorldServiceRuntimeError::SequenceExhausted)?;
        runtime.next_request_id = maximum_request_id
            .checked_add(1)
            .ok_or(WorldServiceRuntimeError::SequenceExhausted)?;
        Ok(runtime)
    }

    pub const fn key(&self) -> &SimulationRegionKey {
        &self.key
    }

    pub const fn generation(&self) -> ActivationGeneration {
        self.generation
    }

    pub fn chunk(&self, position: ChunkPos) -> Option<&ChunkColumn> {
        self.voxels.view().chunk(position)
    }

    pub fn lifecycle(&self, position: ChunkPos) -> Option<ChunkLifecycle> {
        self.lifecycle.get(&position).copied()
    }

    pub fn chunks(&self) -> impl Iterator<Item = (ChunkPos, ChunkLifecycle)> + '_ {
        self.lifecycle
            .iter()
            .map(|(position, lifecycle)| (*position, *lifecycle))
    }

    pub fn projectable_snapshot(
        &self,
        position: ChunkPos,
    ) -> Result<Option<ChunkSnapshot>, WorldServiceRuntimeError> {
        let Some(lifecycle) = self.lifecycle.get(&position).copied() else {
            return Ok(None);
        };
        if lifecycle.status != ChunkStatus::Full
            || lifecycle.activity < ChunkActivity::Accessible
            || lifecycle.pending_generation.is_some()
            || lifecycle.pending_unload.is_some()
        {
            return Ok(None);
        }
        let chunk = self
            .chunk(position)
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(position))?;
        let light = chunk
            .light()
            .ok_or(WorldServiceRuntimeError::MissingAuthoritativeLight(
                position,
            ))?
            .snapshot(chunk.layout().sections().count())?;
        Ok(Some(chunk.snapshot(light, |kind, state| {
            use ferrite_world::id::{FIRE, LAVA, WATER};
            state != chunk.layout().default_block()
                && (kind == ferrite_world::projection::ClientHeightmap::WorldSurface
                    || !matches!(state, WATER | LAVA | FIRE))
        })?))
    }

    pub(crate) fn voxels_mut(&mut self) -> &mut RegionVoxelState {
        &mut self.voxels
    }

    pub fn snapshot_records(&self) -> Result<Vec<SnapshotRecord>, WorldServiceRuntimeError> {
        let mut records = self.auxiliary_records.clone();
        for (position, lifecycle) in &self.lifecycle {
            let chunk = self
                .chunk(*position)
                .expect("lifecycle and voxel state stay aligned");
            records.push(encode_chunk_record(chunk, *lifecycle)?);
        }
        Ok(records)
    }

    pub fn demand_chunk(
        &mut self,
        position: ChunkPos,
    ) -> Result<TicketOutcome, WorldServiceRuntimeError> {
        self.validate_chunk(position)?;
        if let Some(lifecycle) = self.lifecycle.get(&position).copied() {
            if let Some(pending) = lifecycle.pending_unload {
                self.reserve_events(1)?;
                self.lifecycle
                    .get_mut(&position)
                    .expect("validated lifecycle exists")
                    .pending_unload = None;
                self.push_reserved_events(
                    position,
                    [ChunkEventKind::UnloadCancelled {
                        token: pending.token,
                    }],
                )?;
                return Ok(TicketOutcome::CancelledUnload {
                    token: pending.token,
                });
            }
            return Ok(TicketOutcome::AlreadyLoaded);
        }
        if self.lifecycle.len() >= self.config.chunk_capacity {
            return Err(WorldServiceRuntimeError::ChunkCapacity);
        }
        self.voxels.ensure_chunk(position)?;
        self.lifecycle.insert(position, ChunkLifecycle::empty());
        Ok(TicketOutcome::Loaded)
    }

    pub fn begin_generation(
        &mut self,
        position: ChunkPos,
        target_status: ChunkStatus,
    ) -> Result<GenerationRequest, WorldServiceRuntimeError> {
        self.validate_chunk(position)?;
        let lifecycle = self
            .lifecycle
            .get(&position)
            .copied()
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(position))?;
        if lifecycle.pending_unload.is_some() || lifecycle.pending_generation.is_some() {
            return Err(WorldServiceRuntimeError::ChunkBusy(position));
        }
        let expected_target = next_status(lifecycle.status)
            .ok_or(WorldServiceRuntimeError::GenerationAlreadyFull(position))?;
        if target_status != expected_target {
            return Err(WorldServiceRuntimeError::NonSequentialStatus {
                current: lifecycle.status,
                target: target_status,
            });
        }
        let source = self
            .chunk(position)
            .cloned()
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(position))?;
        let request_id = self.take_request_id()?;
        let pending = PendingGeneration {
            continuation_version: GENERATION_CONTINUATION_VERSION_V1,
            request_id,
            expected_revision: source.revision().get(),
            target_status,
            content_manifest: self.config.content_manifest,
        };
        self.lifecycle
            .get_mut(&position)
            .expect("validated lifecycle exists")
            .pending_generation = Some(pending);
        Ok(GenerationRequest {
            region: self.key.clone(),
            generation: self.generation,
            chunk: position,
            continuation_version: pending.continuation_version,
            request_id,
            expected_revision: pending.expected_revision,
            target_status,
            content_manifest: self.config.content_manifest,
            source,
        })
    }

    pub fn resume_generation(
        &self,
        position: ChunkPos,
    ) -> Result<GenerationRequest, WorldServiceRuntimeError> {
        self.validate_chunk(position)?;
        let lifecycle = self
            .lifecycle
            .get(&position)
            .copied()
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(position))?;
        let pending = lifecycle
            .pending_generation
            .ok_or(WorldServiceRuntimeError::NoPendingGeneration(position))?;
        if pending.continuation_version != GENERATION_CONTINUATION_VERSION_V1 {
            return Err(WorldServiceRuntimeError::UnsupportedGenerationContinuation(
                pending.continuation_version,
            ));
        }
        let source = self
            .chunk(position)
            .cloned()
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(position))?;
        if source.revision().get() != pending.expected_revision
            || next_status(lifecycle.status) != Some(pending.target_status)
        {
            return Err(WorldServiceRuntimeError::InvalidGenerationContinuation(
                position,
            ));
        }
        Ok(GenerationRequest {
            region: self.key.clone(),
            generation: self.generation,
            chunk: position,
            continuation_version: pending.continuation_version,
            request_id: pending.request_id,
            expected_revision: pending.expected_revision,
            target_status: pending.target_status,
            content_manifest: pending.content_manifest,
            source,
        })
    }

    pub fn apply_generated(
        &mut self,
        result: GenerationResult,
    ) -> Result<GenerationOutcome, WorldServiceRuntimeError> {
        self.validate_generation_result(&result)?;
        let lifecycle = self
            .lifecycle
            .get(&result.chunk)
            .copied()
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(result.chunk))?;
        let pending = lifecycle
            .pending_generation
            .ok_or(WorldServiceRuntimeError::NoPendingGeneration(result.chunk))?;
        if pending.request_id != result.request_id
            || pending.continuation_version != result.continuation_version
            || pending.expected_revision != result.expected_revision
            || pending.target_status != result.target_status
            || pending.content_manifest != result.content_manifest
        {
            return Err(WorldServiceRuntimeError::GenerationIdentityMismatch);
        }
        let actual_revision = self
            .chunk(result.chunk)
            .expect("lifecycle and voxel state stay aligned")
            .revision()
            .get();
        if actual_revision != result.expected_revision {
            self.lifecycle
                .get_mut(&result.chunk)
                .expect("validated lifecycle exists")
                .pending_generation = None;
            return Ok(GenerationOutcome::StaleRevision {
                expected: result.expected_revision,
                actual: actual_revision,
            });
        }
        if result.generated.position() != result.chunk {
            return Err(WorldServiceRuntimeError::GeneratedPositionMismatch);
        }
        if result.generated.layout() != self.config.layout {
            return Err(WorldServiceRuntimeError::LayoutMismatch);
        }
        if result.generated.revision().get() < result.expected_revision {
            return Err(WorldServiceRuntimeError::GeneratedRevisionRegressed);
        }
        if result.target_status >= ChunkStatus::InitializeLight
            && result.generated.light().is_none()
        {
            return Err(WorldServiceRuntimeError::GeneratedLightMissing);
        }
        self.reserve_events(1)?;
        self.voxels.remove_chunk(result.chunk);
        let revision = result.generated.revision().get();
        self.voxels.insert_chunk(result.generated)?;
        let lifecycle = self
            .lifecycle
            .get_mut(&result.chunk)
            .expect("validated lifecycle exists");
        lifecycle.status = result.target_status;
        lifecycle.pending_generation = None;
        self.push_reserved_events(
            result.chunk,
            [ChunkEventKind::GenerationPublished {
                status: result.target_status,
                revision,
            }],
        )?;
        Ok(GenerationOutcome::Published { revision })
    }

    pub fn set_block(
        &mut self,
        region: &SimulationRegionKey,
        generation: ActivationGeneration,
        expected_revision: ChunkRevision,
        position: BlockPos,
        state: BlockStateId,
    ) -> Result<ChunkRevision, WorldServiceRuntimeError> {
        self.validate_authority(region, generation)?;
        let chunk_position = position.chunk();
        let chunk = self
            .chunk(chunk_position)
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(chunk_position))?;
        if self
            .lifecycle
            .get(&chunk_position)
            .is_some_and(|lifecycle| lifecycle.pending_unload.is_some())
        {
            return Err(WorldServiceRuntimeError::ChunkBusy(chunk_position));
        }
        if chunk.revision() != expected_revision {
            return Err(WorldServiceRuntimeError::RevisionMismatch {
                expected: expected_revision.get(),
                actual: chunk.revision().get(),
            });
        }
        let relight = chunk.light().is_some();
        self.voxels.set_block(position, state)?;
        if relight {
            self.voxels.recompute_chunk_light(chunk_position)?;
        }
        Ok(self
            .chunk(chunk_position)
            .expect("mutated chunk remains loaded")
            .revision())
    }

    pub fn set_blocks(
        &mut self,
        region: &SimulationRegionKey,
        generation: ActivationGeneration,
        expected_revisions: &BTreeMap<ChunkPos, ChunkRevision>,
        writes: &[WorldBlockWrite],
    ) -> Result<BTreeMap<ChunkPos, ChunkRevision>, WorldServiceRuntimeError> {
        self.validate_authority(region, generation)?;
        if writes.is_empty() {
            return Err(WorldServiceRuntimeError::EmptyBlockTransaction);
        }
        let mut positions = BTreeSet::new();
        let mut touched = BTreeSet::new();
        for write in writes {
            if !positions.insert(write.position) {
                return Err(WorldServiceRuntimeError::DuplicateBlockWrite(
                    write.position,
                ));
            }
            let chunk_position = write.position.chunk();
            self.validate_chunk(chunk_position)?;
            let chunk = self
                .chunk(chunk_position)
                .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(chunk_position))?;
            if self
                .lifecycle
                .get(&chunk_position)
                .is_some_and(|lifecycle| lifecycle.pending_unload.is_some())
            {
                return Err(WorldServiceRuntimeError::ChunkBusy(chunk_position));
            }
            chunk
                .block_state(write.position)
                .map_err(RegionVoxelError::from)?;
            touched.insert(chunk_position);
        }
        if expected_revisions.len() != touched.len()
            || touched.iter().any(|position| {
                self.chunk(*position).map(ChunkColumn::revision)
                    != expected_revisions.get(position).copied()
            })
        {
            return Err(WorldServiceRuntimeError::BlockTransactionRevisionMismatch);
        }

        let mut candidate = self.voxels.clone();
        let mut relight = BTreeSet::new();
        for write in writes {
            let chunk_position = write.position.chunk();
            if self
                .chunk(chunk_position)
                .is_some_and(|chunk| chunk.light().is_some())
            {
                relight.insert(chunk_position);
            }
            candidate.set_block(write.position, write.state)?;
        }
        for position in relight {
            candidate.recompute_chunk_light(position)?;
        }
        self.voxels = candidate;
        Ok(touched
            .into_iter()
            .map(|position| {
                let revision = self
                    .chunk(position)
                    .expect("transactional chunk remains loaded")
                    .revision();
                (position, revision)
            })
            .collect())
    }

    pub fn promote(
        &mut self,
        position: ChunkPos,
        target: ChunkActivity,
    ) -> Result<(), WorldServiceRuntimeError> {
        let lifecycle = self
            .lifecycle
            .get(&position)
            .copied()
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(position))?;
        if lifecycle.status != ChunkStatus::Full {
            return Err(WorldServiceRuntimeError::ChunkNotFull(position));
        }
        if lifecycle.pending_unload.is_some() {
            return Err(WorldServiceRuntimeError::ChunkBusy(position));
        }
        let events: &[ChunkEventKind] = match (lifecycle.activity, target) {
            (ChunkActivity::Dormant, ChunkActivity::Accessible) => &[ChunkEventKind::Accessible],
            (ChunkActivity::Accessible, ChunkActivity::BlockTicking) => &[
                ChunkEventKind::PersistedTicksUnpacked,
                ChunkEventKind::BlockTicking,
            ],
            (ChunkActivity::BlockTicking, ChunkActivity::EntityTicking) => {
                &[ChunkEventKind::EntityTicking]
            }
            _ => return Err(WorldServiceRuntimeError::InvalidActivityTransition),
        };
        self.reserve_events(events.len())?;
        self.lifecycle
            .get_mut(&position)
            .expect("validated lifecycle exists")
            .activity = target;
        self.push_reserved_events(position, events.iter().copied())?;
        Ok(())
    }

    pub fn demote(
        &mut self,
        position: ChunkPos,
        target: ChunkActivity,
    ) -> Result<(), WorldServiceRuntimeError> {
        let lifecycle = self
            .lifecycle
            .get(&position)
            .copied()
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(position))?;
        if target >= lifecycle.activity {
            return Err(WorldServiceRuntimeError::InvalidActivityTransition);
        }
        self.reserve_events(1)?;
        self.lifecycle
            .get_mut(&position)
            .expect("validated lifecycle exists")
            .activity = target;
        self.push_reserved_events(position, [ChunkEventKind::Demoted { activity: target }])?;
        Ok(())
    }

    pub fn schedule_unload(&mut self, position: ChunkPos) -> Result<u64, WorldServiceRuntimeError> {
        let lifecycle = self
            .lifecycle
            .get(&position)
            .copied()
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(position))?;
        if lifecycle.pending_generation.is_some() || lifecycle.pending_unload.is_some() {
            return Err(WorldServiceRuntimeError::ChunkBusy(position));
        }
        let token = self.take_unload_token()?;
        let expected_revision = self
            .chunk(position)
            .expect("lifecycle and voxel state stay aligned")
            .revision()
            .get();
        self.lifecycle
            .get_mut(&position)
            .expect("validated lifecycle exists")
            .pending_unload = Some(PendingUnload {
            token,
            expected_revision,
        });
        Ok(token)
    }

    pub fn replace_auxiliary_records(
        &mut self,
        records: Vec<SnapshotRecord>,
    ) -> Result<(), WorldServiceRuntimeError> {
        validate_auxiliary_records(&records)?;
        self.auxiliary_records = records;
        Ok(())
    }

    pub fn prepare_save(
        &self,
        committed_tick: u64,
        persistence_revision: PersistenceRevision,
    ) -> Result<PreparedWorldSave, WorldServiceRuntimeError> {
        let records = self.snapshot_records()?;
        let mut pending_unloads = Vec::new();
        for (position, lifecycle) in &self.lifecycle {
            if let Some(pending) = lifecycle.pending_unload {
                pending_unloads.push(PendingUnloadIdentity {
                    chunk: *position,
                    token: pending.token,
                });
            }
        }
        let state_hash = canonical_state_hash(&records);
        let snapshot = RegionCommitSnapshot::new(
            RegionSnapshotHeader {
                key: self.key.clone(),
                generation: self.generation,
                committed_tick,
                persistence_revision,
                region_side_chunks: self.config.region_side_chunks,
                content_manifest: self.config.content_manifest,
                state_hash,
            },
            records,
        )?;
        Ok(PreparedWorldSave::new(
            RegionRecoveryPoint::new(snapshot, Vec::new())?,
            pending_unloads,
        ))
    }

    pub(crate) fn confirm_composite_save(
        &self,
        point: &RegionRecoveryPoint,
    ) -> Result<PreparedWorldSave, WorldServiceRuntimeError> {
        let header = point.snapshot().header();
        if header.key != self.key {
            return Err(WorldServiceRuntimeError::WrongRegion);
        }
        if header.generation != self.generation {
            return Err(WorldServiceRuntimeError::StaleGeneration);
        }
        if header.region_side_chunks != self.config.region_side_chunks {
            return Err(WorldServiceRuntimeError::RegionSideMismatch);
        }
        if header.content_manifest != self.config.content_manifest {
            return Err(WorldServiceRuntimeError::ContentManifestMismatch);
        }
        if canonical_state_hash(point.snapshot().records()) != header.state_hash {
            return Err(WorldServiceRuntimeError::StateHashMismatch);
        }
        let committed_records = materialized_records(point);
        for record in self.snapshot_records()? {
            if !committed_records.contains(&record) {
                return Err(WorldServiceRuntimeError::CompositeSaveStateMismatch);
            }
        }
        let pending_unloads = self
            .lifecycle
            .iter()
            .filter_map(|(chunk, lifecycle)| {
                lifecycle
                    .pending_unload
                    .map(|pending| PendingUnloadIdentity {
                        chunk: *chunk,
                        token: pending.token,
                    })
            })
            .collect();
        Ok(PreparedWorldSave::new(point.clone(), pending_unloads))
    }

    pub fn apply_save_receipt(
        &mut self,
        prepared: PreparedWorldSave,
        receipt: CommitReceipt,
    ) -> Result<usize, WorldServiceRuntimeError> {
        if prepared.recovery_point().snapshot().key() != &self.key
            || prepared.recovery_point().snapshot().generation() != self.generation
            || receipt.revision() != prepared.persistence_revision()
            || receipt.committed_tick() != prepared.committed_tick()
            || receipt.digest() != prepared.digest()?
        {
            return Err(WorldServiceRuntimeError::SaveReceiptMismatch);
        }
        let admitted = prepared
            .pending_unloads()
            .iter()
            .filter(|identity| {
                self.lifecycle
                    .get(&identity.chunk)
                    .and_then(|lifecycle| lifecycle.pending_unload)
                    .is_some_and(|pending| pending.token == identity.token)
            })
            .copied()
            .collect::<Vec<_>>();
        self.reserve_events(admitted.len().saturating_mul(2))?;
        for identity in &admitted {
            self.push_reserved_events(
                identity.chunk,
                [ChunkEventKind::Saved {
                    token: identity.token,
                }],
            )?;
            self.voxels.remove_chunk(identity.chunk);
            self.lifecycle.remove(&identity.chunk);
            self.push_reserved_events(
                identity.chunk,
                [ChunkEventKind::Unloaded {
                    token: identity.token,
                }],
            )?;
        }
        Ok(admitted.len())
    }

    pub fn take_events(&mut self, maximum: usize) -> Vec<ChunkEvent> {
        let count = maximum.min(self.events.len());
        self.events.drain(..count).collect()
    }

    fn validate_generation_result(
        &self,
        result: &GenerationResult,
    ) -> Result<(), WorldServiceRuntimeError> {
        self.validate_authority(&result.region, result.generation)?;
        self.validate_chunk(result.chunk)?;
        if result.content_manifest != self.config.content_manifest {
            return Err(WorldServiceRuntimeError::ContentManifestMismatch);
        }
        Ok(())
    }

    fn validate_authority(
        &self,
        region: &SimulationRegionKey,
        generation: ActivationGeneration,
    ) -> Result<(), WorldServiceRuntimeError> {
        if region != &self.key {
            return Err(WorldServiceRuntimeError::WrongRegion);
        }
        if generation != self.generation {
            return Err(WorldServiceRuntimeError::StaleGeneration);
        }
        Ok(())
    }

    fn validate_chunk(&self, position: ChunkPos) -> Result<(), WorldServiceRuntimeError> {
        let actual = self.config.mapping.region_for_chunk(
            self.key.world(),
            self.key.dimension().clone(),
            position,
        );
        if actual != self.key {
            return Err(WorldServiceRuntimeError::WrongChunkOwner(position));
        }
        Ok(())
    }

    fn reserve_events(&self, count: usize) -> Result<(), WorldServiceRuntimeError> {
        if self.events.len().saturating_add(count) > self.config.event_capacity {
            return Err(WorldServiceRuntimeError::EventCapacity);
        }
        let count =
            u64::try_from(count).map_err(|_| WorldServiceRuntimeError::SequenceExhausted)?;
        self.next_event_sequence
            .checked_add(count)
            .ok_or(WorldServiceRuntimeError::SequenceExhausted)?;
        Ok(())
    }

    fn push_reserved_events(
        &mut self,
        position: ChunkPos,
        events: impl IntoIterator<Item = ChunkEventKind>,
    ) -> Result<(), WorldServiceRuntimeError> {
        for kind in events {
            let sequence = self.take_event_sequence()?;
            self.events.push_back(ChunkEvent {
                sequence,
                chunk: position,
                kind,
            });
        }
        Ok(())
    }

    fn take_request_id(&mut self) -> Result<u64, WorldServiceRuntimeError> {
        take_sequence(&mut self.next_request_id)
    }

    fn take_unload_token(&mut self) -> Result<u64, WorldServiceRuntimeError> {
        take_sequence(&mut self.next_unload_token)
    }

    fn take_event_sequence(&mut self) -> Result<u64, WorldServiceRuntimeError> {
        take_sequence(&mut self.next_event_sequence)
    }
}

fn next_status(status: ChunkStatus) -> Option<ChunkStatus> {
    ChunkStatus::ALL.get(status as usize + 1).copied()
}

fn take_sequence(sequence: &mut u64) -> Result<u64, WorldServiceRuntimeError> {
    let value = *sequence;
    *sequence = sequence
        .checked_add(1)
        .ok_or(WorldServiceRuntimeError::SequenceExhausted)?;
    Ok(value)
}

fn validate_config(
    key: &SimulationRegionKey,
    config: &WorldServiceRuntimeConfig,
) -> Result<(), WorldServiceRuntimeError> {
    if key.mapping_version() != config.mapping.version() {
        return Err(WorldServiceRuntimeError::MappingVersionMismatch);
    }
    if config.region_side_chunks == 0 || config.chunk_capacity == 0 || config.event_capacity == 0 {
        return Err(WorldServiceRuntimeError::ZeroCapacity);
    }
    Ok(())
}

fn validate_auxiliary_records(records: &[SnapshotRecord]) -> Result<(), WorldServiceRuntimeError> {
    let mut identities = BTreeSet::new();
    for record in records {
        if record.domain() == &chunk_domain() {
            return Err(WorldServiceRuntimeError::ReservedAuxiliaryDomain);
        }
        if !identities.insert((
            record.kind(),
            record.domain().clone(),
            record.key().to_vec(),
        )) {
            return Err(WorldServiceRuntimeError::DuplicateAuxiliaryRecord);
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum WorldServiceRuntimeError {
    #[error("world-service operation targets another Region")]
    WrongRegion,
    #[error("world-service operation uses a stale activation generation")]
    StaleGeneration,
    #[error("restored activation generation is not newer than the durable generation")]
    GenerationNotNewer,
    #[error("chunk {0:?} is owned by another Region")]
    WrongChunkOwner(ChunkPos),
    #[error("chunk {0:?} is not loaded")]
    ChunkNotLoaded(ChunkPos),
    #[error("chunk {0:?} has no authoritative propagated light")]
    MissingAuthoritativeLight(ChunkPos),
    #[error("chunk {0:?} is duplicated in durable state")]
    DuplicateChunk(ChunkPos),
    #[error("chunk {0:?} has generation or unload work in flight")]
    ChunkBusy(ChunkPos),
    #[error("chunk {0:?} has no pending generation request")]
    NoPendingGeneration(ChunkPos),
    #[error("chunk {0:?} already reached FULL")]
    GenerationAlreadyFull(ChunkPos),
    #[error("generation must advance one status from {current:?}, not directly to {target:?}")]
    NonSequentialStatus {
        current: ChunkStatus,
        target: ChunkStatus,
    },
    #[error("generation result does not match the pending request")]
    GenerationIdentityMismatch,
    #[error("generation continuation version {0} is unsupported")]
    UnsupportedGenerationContinuation(u16),
    #[error("chunk {0:?} has a generation continuation inconsistent with durable state")]
    InvalidGenerationContinuation(ChunkPos),
    #[error("generation result chunk position does not match its request")]
    GeneratedPositionMismatch,
    #[error("generation result revision regressed below its input revision")]
    GeneratedRevisionRegressed,
    #[error("generation result reached a light-dependent status without propagated light")]
    GeneratedLightMissing,
    #[error("chunk {0:?} is not FULL and cannot become accessible")]
    ChunkNotFull(ChunkPos),
    #[error("chunk activity transition is not adjacent and ordered")]
    InvalidActivityTransition,
    #[error("chunk revision mismatch: expected {expected}, actual {actual}")]
    RevisionMismatch { expected: u64, actual: u64 },
    #[error("world block transaction cannot be empty")]
    EmptyBlockTransaction,
    #[error("world block transaction writes position {0:?} more than once")]
    DuplicateBlockWrite(BlockPos),
    #[error("world block transaction revision set does not match its touched chunks")]
    BlockTransactionRevisionMismatch,
    #[error("world content manifest does not match the locked runtime")]
    ContentManifestMismatch,
    #[error("Region mapping version does not match the Region key")]
    MappingVersionMismatch,
    #[error("durable Region side does not match runtime configuration")]
    RegionSideMismatch,
    #[error("durable chunk layout does not match runtime configuration")]
    LayoutMismatch,
    #[error("durable snapshot state hash does not match its records")]
    StateHashMismatch,
    #[error("save receipt does not match the prepared recovery point")]
    SaveReceiptMismatch,
    #[error("composite durable point does not contain the current world-service state")]
    CompositeSaveStateMismatch,
    #[error("world-service chunk capacity is exhausted")]
    ChunkCapacity,
    #[error("world-service lifecycle event capacity is exhausted")]
    EventCapacity,
    #[error("world-service capacities and Region side must be nonzero")]
    ZeroCapacity,
    #[error("world-service monotonic sequence is exhausted")]
    SequenceExhausted,
    #[error("auxiliary continuity cannot use the reserved world-service chunk domain")]
    ReservedAuxiliaryDomain,
    #[error("auxiliary continuity contains a duplicate record identity")]
    DuplicateAuxiliaryRecord,
    #[error(transparent)]
    Region(#[from] RegionVoxelError),
    #[error(transparent)]
    Projection(#[from] ChunkProjectionError),
    #[error(transparent)]
    Continuity(#[from] WorldServiceContinuityError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Migration(#[from] ContinuityMigrationError),
}

#[cfg(test)]
mod tests {
    use ferrite_foundation::identity::{DimensionId, WorldId};
    use ferrite_foundation::region::{RegionCoord, RegionMapping, RegionMappingVersion};
    use ferrite_foundation::resource::ResourceId;
    use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
    use ferrite_world::id::{AIR, BiomeId, BlockStateId};
    use ferrite_world::projection::{ChunkLightState, LightSnapshot};

    use super::*;

    fn key() -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        )
    }

    fn config() -> WorldServiceRuntimeConfig {
        WorldServiceRuntimeConfig {
            mapping: RegionMapping::V1,
            layout: ChunkLayout::new(
                VerticalSectionRange::new(-4, 24).unwrap(),
                BlockStateId::new(0),
                BiomeId::new(0),
            ),
            region_side_chunks: 8,
            chunk_capacity: 8,
            event_capacity: 32,
            content_manifest: [9; 32],
        }
    }

    #[test]
    fn generation_continuation_survives_commit_and_resumes_under_new_activation() {
        let position = ChunkPos::new(0, 0);
        let mut runtime =
            WorldServiceRegionRuntime::new(key(), ActivationGeneration::INITIAL, config()).unwrap();
        runtime.demand_chunk(position).unwrap();
        let original = runtime
            .begin_generation(position, ChunkStatus::StructureStarts)
            .unwrap();
        let prepared = runtime
            .prepare_save(7, PersistenceRevision::INITIAL)
            .unwrap();
        let next_generation = ActivationGeneration::INITIAL.checked_next().unwrap();
        let mut restored = WorldServiceRegionRuntime::restore(
            key(),
            next_generation,
            prepared.recovery_point(),
            config(),
        )
        .unwrap();
        let resumed = restored.resume_generation(position).unwrap();
        assert_eq!(resumed.request_id, original.request_id);
        assert_eq!(
            resumed.continuation_version,
            GENERATION_CONTINUATION_VERSION_V1
        );
        assert_eq!(resumed.generation, next_generation);
        assert_eq!(resumed.expected_revision, original.expected_revision);

        let other = ChunkPos::new(1, 0);
        restored.demand_chunk(other).unwrap();
        let later = restored
            .begin_generation(other, ChunkStatus::StructureStarts)
            .unwrap();
        assert!(later.request_id > resumed.request_id);
    }

    #[test]
    fn only_accessible_full_authority_produces_a_revision_matched_snapshot() {
        let position = ChunkPos::new(0, 0);
        let block = BlockPos::new(3, 70, 4);
        let mut runtime =
            WorldServiceRegionRuntime::new(key(), ActivationGeneration::INITIAL, config()).unwrap();
        runtime.demand_chunk(position).unwrap();
        runtime
            .set_block(
                &key(),
                ActivationGeneration::INITIAL,
                runtime.chunk(position).unwrap().revision(),
                block,
                BlockStateId::new(2),
            )
            .unwrap();
        assert!(runtime.projectable_snapshot(position).unwrap().is_none());
        for status in ChunkStatus::ALL.into_iter().skip(1) {
            let request = runtime.begin_generation(position, status).unwrap();
            let mut generated = request.source.clone();
            if status == ChunkStatus::InitializeLight {
                let snapshot = LightSnapshot::full_sky(config().layout.sections().count()).unwrap();
                generated
                    .replace_light(
                        ChunkLightState::new(
                            snapshot.sky().to_vec(),
                            snapshot.block().to_vec(),
                            config().layout.sections().count(),
                        )
                        .unwrap(),
                    )
                    .unwrap();
            }
            runtime
                .apply_generated(request.complete(generated))
                .unwrap();
        }
        assert!(runtime.projectable_snapshot(position).unwrap().is_none());
        runtime
            .promote(position, ChunkActivity::Accessible)
            .unwrap();
        let snapshot = runtime.projectable_snapshot(position).unwrap().unwrap();
        assert_eq!(
            snapshot.revision(),
            runtime.chunk(position).unwrap().revision()
        );
        assert!(
            snapshot
                .heightmaps()
                .values()
                .all(|heightmap| heightmap[4 * 16 + 3] == 71)
        );
        runtime.schedule_unload(position).unwrap();
        assert!(runtime.projectable_snapshot(position).unwrap().is_none());
    }

    #[test]
    fn block_transaction_preflights_every_revision_and_commits_all_writes_together() {
        let position = ChunkPos::new(0, 0);
        let mut runtime =
            WorldServiceRegionRuntime::new(key(), ActivationGeneration::INITIAL, config()).unwrap();
        runtime.demand_chunk(position).unwrap();
        let revision = runtime.chunk(position).unwrap().revision();
        let writes = [
            WorldBlockWrite {
                position: BlockPos::new(1, 70, 1),
                state: BlockStateId::new(8),
            },
            WorldBlockWrite {
                position: BlockPos::new(2, 70, 1),
                state: BlockStateId::new(9),
            },
        ];
        let wrong = BTreeMap::from([(position, ChunkRevision::INITIAL)]);
        runtime
            .set_block(
                &key(),
                ActivationGeneration::INITIAL,
                revision,
                BlockPos::new(0, 70, 0),
                BlockStateId::new(2),
            )
            .unwrap();
        assert!(matches!(
            runtime.set_blocks(&key(), ActivationGeneration::INITIAL, &wrong, &writes),
            Err(WorldServiceRuntimeError::BlockTransactionRevisionMismatch)
        ));
        assert_eq!(
            runtime
                .chunk(position)
                .unwrap()
                .block_state(writes[0].position)
                .unwrap(),
            AIR
        );

        let expected = BTreeMap::from([(position, runtime.chunk(position).unwrap().revision())]);
        let revisions = runtime
            .set_blocks(&key(), ActivationGeneration::INITIAL, &expected, &writes)
            .unwrap();
        assert_eq!(revisions[&position].get(), expected[&position].get() + 2);
        for write in writes {
            assert_eq!(
                runtime
                    .chunk(position)
                    .unwrap()
                    .block_state(write.position)
                    .unwrap(),
                write.state
            );
        }
    }
}
