use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::ResourceId;
use ferrite_region_runtime::transfer::{EntityTransfer, TransferRole};

use crate::composite::model::{CompositeOwner, CompositeProjection};
use crate::composite::services::CompositeServiceAction;
use crate::entity_service::model::{
    EntityCommandHeader, EntityProjection, EntityProjectionKind, RemovalReason,
};
use crate::player::block::replication::AuthoritativeBlockUpdate;
use crate::player_service::model::{
    PlayerActionHeader, PlayerPersistentState, PlayerProjection, ProjectionKind, ResyncReason,
};
use crate::simulation::boundary::BoundaryMechanic;
use crate::simulation::continuity::ScheduledQueueKind;

pub(super) fn encode_player_projection(projection: PlayerProjection) -> CompositeProjection {
    let mut payload = Vec::new();
    payload.extend_from_slice(&projection.player.to_be_bytes());
    payload.extend_from_slice(&projection.session_epoch.to_be_bytes());
    match projection.kind {
        ProjectionKind::InventoryDelta { inventory_revision } => {
            payload.push(0);
            payload.extend_from_slice(&inventory_revision.to_be_bytes());
        }
        ProjectionKind::MenuDelta {
            container_id,
            state_id,
            inventory_revision,
        } => {
            payload.push(1);
            payload.push(container_id);
            payload.extend_from_slice(&state_id.to_be_bytes());
            payload.extend_from_slice(&inventory_revision.to_be_bytes());
        }
        ProjectionKind::FullState {
            reason,
            inventory_revision,
            menu,
        } => {
            payload.push(2);
            payload.push(resync_reason_tag(reason));
            payload.extend_from_slice(&inventory_revision.to_be_bytes());
            match menu {
                None => payload.push(0),
                Some(menu) => {
                    payload.push(1);
                    payload.push(menu.container_id);
                    payload.extend_from_slice(&menu.state_id.to_be_bytes());
                }
            }
        }
    }
    CompositeProjection::new(
        CompositeOwner::PlayerService,
        projection.revision,
        ResourceId::new("ferrite", "composite/player/projection_v1")
            .expect("static player projection identity is valid"),
        payload,
    )
}

pub(super) fn encode_entity_projection(projection: EntityProjection) -> CompositeProjection {
    let mut payload = Vec::new();
    payload.extend_from_slice(&projection.observer.to_be_bytes());
    payload.extend_from_slice(&projection.entity.to_be_bytes());
    match projection.kind {
        EntityProjectionKind::Spawn {
            kind,
            chunk,
            revision,
            state_digest,
        } => {
            payload.push(0);
            encode_bytes(&mut payload, kind.to_string().as_bytes());
            encode_chunk(&mut payload, chunk);
            payload.extend_from_slice(&revision.to_be_bytes());
            payload.extend_from_slice(&state_digest);
        }
        EntityProjectionKind::Update {
            chunk,
            revision,
            state_digest,
        } => {
            payload.push(1);
            encode_chunk(&mut payload, chunk);
            payload.extend_from_slice(&revision.to_be_bytes());
            payload.extend_from_slice(&state_digest);
        }
        EntityProjectionKind::Remove { revision, reason } => {
            payload.push(2);
            payload.extend_from_slice(&revision.to_be_bytes());
            payload.push(removal_reason_tag(reason));
        }
    }
    CompositeProjection::new(
        CompositeOwner::EntityService,
        projection.sequence,
        ResourceId::new("ferrite", "composite/entity/projection_v1")
            .expect("static entity projection identity is valid"),
        payload,
    )
}

pub(super) fn encode_block_projection(
    sequence: u64,
    update: AuthoritativeBlockUpdate,
) -> CompositeProjection {
    let mut payload = Vec::with_capacity(16);
    encode_block(&mut payload, update.position);
    payload.extend_from_slice(&update.state.get().to_be_bytes());
    CompositeProjection::new(
        CompositeOwner::Simulation,
        sequence,
        ResourceId::new("ferrite", "composite/simulation/block_update_v1")
            .expect("static block projection identity is valid"),
        payload,
    )
}

pub(super) fn encode_action(action: &CompositeServiceAction) -> Vec<u8> {
    let mut output = Vec::new();
    match action {
        CompositeServiceAction::JoinPlayer { player, state } => {
            output.extend_from_slice(&player.to_be_bytes());
            encode_player_state(&mut output, state);
        }
        CompositeServiceAction::LeavePlayer { player } => {
            output.extend_from_slice(&player.to_be_bytes());
        }
        CompositeServiceAction::ApplyPlayerAction { header, mutation } => {
            encode_player_header(&mut output, header);
            output.extend_from_slice(&mutation.expected_inventory_revision.to_be_bytes());
            encode_bytes(&mut output, mutation.inventory.bytes());
            output.push(mutation.selected_slot);
            output.extend_from_slice(&mutation.experience_points.to_be_bytes());
            output.extend_from_slice(&mutation.experience_level.to_be_bytes());
            output.extend_from_slice(&mutation.food_level.to_be_bytes());
            output.extend_from_slice(&mutation.saturation_bits.to_be_bytes());
            output.extend_from_slice(&mutation.exhaustion_bits.to_be_bytes());
            encode_bytes(&mut output, mutation.progression.bytes());
        }
        CompositeServiceAction::OpenMenu {
            header,
            container_id,
        } => {
            encode_player_header(&mut output, header);
            output.push(*container_id);
        }
        CompositeServiceAction::CloseMenu { header } => encode_player_header(&mut output, header),
        CompositeServiceAction::ScheduleSimulation {
            kind,
            type_identity,
            position,
            delay,
            priority,
        } => {
            output.push(scheduled_kind_tag(*kind));
            encode_bytes(&mut output, type_identity.to_string().as_bytes());
            encode_block(&mut output, *position);
            output.extend_from_slice(&delay.to_be_bytes());
            output.push(priority.value() as u8);
        }
        CompositeServiceAction::InsertEntity { entity, state } => {
            let record = crate::entity_service::continuity::encode_entity(*entity, state)
                .expect("validated entity state has a bounded continuity encoding");
            output.extend_from_slice(&entity.to_be_bytes());
            encode_bytes(&mut output, record.value());
        }
        CompositeServiceAction::AddEntityObserver { observer } => {
            output.extend_from_slice(&observer.to_be_bytes());
        }
        CompositeServiceAction::MutateEntity { header, mutation } => {
            encode_entity_header(&mut output, header);
            encode_chunk(&mut output, mutation.chunk);
            encode_bytes(&mut output, mutation.payload.bytes());
        }
        CompositeServiceAction::DemandChunk { position } => encode_chunk(&mut output, *position),
        CompositeServiceAction::SetWorldBlock {
            expected_revision,
            position,
            state,
        } => {
            output.extend_from_slice(&expected_revision.get().to_be_bytes());
            encode_block(&mut output, *position);
            output.extend_from_slice(&state.get().to_be_bytes());
        }
        CompositeServiceAction::SetWorldBlocks {
            expected_revisions,
            writes,
        } => {
            output.extend_from_slice(&(expected_revisions.len() as u64).to_be_bytes());
            for (position, revision) in expected_revisions {
                encode_chunk(&mut output, *position);
                output.extend_from_slice(&revision.get().to_be_bytes());
            }
            output.extend_from_slice(&(writes.len() as u64).to_be_bytes());
            for write in writes {
                encode_block(&mut output, write.position);
                output.extend_from_slice(&write.state.get().to_be_bytes());
            }
        }
        CompositeServiceAction::ApplyBoundaryTransaction { transaction } => {
            output.extend_from_slice(&transaction.tick().get().to_be_bytes());
            encode_region(&mut output, transaction.source());
            encode_region(&mut output, transaction.target());
            output.extend_from_slice(&transaction.source_generation().get().to_be_bytes());
            output.extend_from_slice(&transaction.target_generation().get().to_be_bytes());
            output.extend_from_slice(&transaction.source_sequence().to_be_bytes());
            output.push(boundary_mechanic_tag(transaction.mechanic()));
            output.extend_from_slice(&(transaction.mutations().len() as u64).to_be_bytes());
            for mutation in transaction.mutations() {
                output.extend_from_slice(&mutation.order.to_be_bytes());
                encode_block(&mut output, mutation.position);
                output.extend_from_slice(&mutation.expected.get().to_be_bytes());
                output.extend_from_slice(&mutation.replacement.get().to_be_bytes());
            }
            output.extend_from_slice(&(transaction.schedules().len() as u64).to_be_bytes());
            for schedule in transaction.schedules() {
                output.extend_from_slice(&schedule.order.to_be_bytes());
                output.push(scheduled_kind_tag(schedule.kind));
                encode_bytes(&mut output, schedule.type_identity.to_string().as_bytes());
                encode_block(&mut output, schedule.position);
                output.extend_from_slice(&schedule.delay.to_be_bytes());
                output.push(schedule.priority.value() as u8);
            }
        }
        CompositeServiceAction::PrepareEntityTransfer { request } => {
            output.extend_from_slice(&request.tick.get().to_be_bytes());
            encode_region(&mut output, &request.source);
            encode_region(&mut output, &request.target);
            output.extend_from_slice(&request.source_generation.get().to_be_bytes());
            output.extend_from_slice(&request.target_generation.get().to_be_bytes());
            output.extend_from_slice(&request.entity.to_be_bytes());
            output.extend_from_slice(&request.expected_revision.to_be_bytes());
            output.extend_from_slice(&request.sequence.to_be_bytes());
            encode_chunk(&mut output, request.candidate.chunk);
            encode_bytes(&mut output, request.candidate.payload.bytes());
        }
        CompositeServiceAction::AcceptEntityTransfer { transfer } => {
            encode_transfer(&mut output, transfer);
        }
        CompositeServiceAction::CommitEntityTransfer { receipt } => {
            output.extend_from_slice(&receipt.tick.get().to_be_bytes());
            encode_region(&mut output, &receipt.source);
            encode_region(&mut output, &receipt.target);
            output.extend_from_slice(&receipt.source_generation.get().to_be_bytes());
            output.extend_from_slice(&receipt.target_generation.get().to_be_bytes());
            output.extend_from_slice(&receipt.source_sequence.to_be_bytes());
            output.extend_from_slice(&receipt.entity.to_be_bytes());
        }
    }
    output
}

fn encode_player_header(output: &mut Vec<u8>, header: &PlayerActionHeader) {
    output.extend_from_slice(&header.player.to_be_bytes());
    output.extend_from_slice(&header.generation.get().to_be_bytes());
    output.extend_from_slice(&header.session_epoch.to_be_bytes());
    output.extend_from_slice(&header.sequence.to_be_bytes());
}

fn encode_entity_header(output: &mut Vec<u8>, header: &EntityCommandHeader) {
    encode_region(output, &header.region);
    output.extend_from_slice(&header.generation.get().to_be_bytes());
    output.extend_from_slice(&header.entity.to_be_bytes());
    output.extend_from_slice(&header.expected_revision.to_be_bytes());
    output.extend_from_slice(&header.sequence.to_be_bytes());
}

fn encode_transfer(output: &mut Vec<u8>, transfer: &EntityTransfer) {
    output.extend_from_slice(&transfer.tick().get().to_be_bytes());
    encode_region(output, transfer.source());
    encode_region(output, transfer.target());
    output.extend_from_slice(&transfer.source_generation().get().to_be_bytes());
    output.extend_from_slice(&transfer.target_generation().get().to_be_bytes());
    output.extend_from_slice(&transfer.source_sequence().to_be_bytes());
    output.extend_from_slice(&transfer.stable_id().to_be_bytes());
    output.push(match transfer.role() {
        TransferRole::Entity => 0,
        TransferRole::Player => 1,
    });
    encode_bytes(output, transfer.kind().to_string().as_bytes());
    encode_bytes(output, transfer.state());
}

fn encode_region(output: &mut Vec<u8>, region: &SimulationRegionKey) {
    output.extend_from_slice(&region.world().to_be_bytes());
    encode_bytes(output, region.dimension().resource().to_string().as_bytes());
    output.extend_from_slice(&region.coordinate().x().to_be_bytes());
    output.extend_from_slice(&region.coordinate().z().to_be_bytes());
    output.extend_from_slice(&region.mapping_version().get().to_be_bytes());
}

fn encode_player_state(output: &mut Vec<u8>, state: &PlayerPersistentState) {
    output.extend_from_slice(&state.inventory_revision.to_be_bytes());
    encode_bytes(output, state.inventory.bytes());
    output.push(state.selected_slot);
    output.extend_from_slice(&state.experience_points.to_be_bytes());
    output.extend_from_slice(&state.experience_level.to_be_bytes());
    output.extend_from_slice(&state.food_level.to_be_bytes());
    output.extend_from_slice(&state.saturation_bits.to_be_bytes());
    output.extend_from_slice(&state.exhaustion_bits.to_be_bytes());
    encode_bytes(output, state.progression.bytes());
    output.extend_from_slice(&state.last_action_sequence.to_be_bytes());
    output.extend_from_slice(&state.last_session_epoch.to_be_bytes());
}

const fn scheduled_kind_tag(kind: ScheduledQueueKind) -> u8 {
    match kind {
        ScheduledQueueKind::Block => 0,
        ScheduledQueueKind::Fluid => 1,
    }
}

const fn boundary_mechanic_tag(mechanic: BoundaryMechanic) -> u8 {
    match mechanic {
        BoundaryMechanic::Neighbor => 0,
        BoundaryMechanic::Fluid => 1,
        BoundaryMechanic::Redstone => 2,
        BoundaryMechanic::Piston => 3,
        BoundaryMechanic::Explosion => 4,
        BoundaryMechanic::Lighting => 5,
    }
}

const fn removal_reason_tag(reason: RemovalReason) -> u8 {
    match reason {
        RemovalReason::Deactivated => 0,
        RemovalReason::Despawned => 1,
        RemovalReason::Transferred => 2,
    }
}

const fn resync_reason_tag(reason: ResyncReason) -> u8 {
    match reason {
        ResyncReason::Join => 0,
        ResyncReason::Reload => 1,
        ResyncReason::InventoryRevision => 2,
        ResyncReason::MenuState => 3,
    }
}

fn encode_chunk(output: &mut Vec<u8>, chunk: ChunkPos) {
    output.extend_from_slice(&chunk.x.to_be_bytes());
    output.extend_from_slice(&chunk.z.to_be_bytes());
}

fn encode_block(output: &mut Vec<u8>, position: BlockPos) {
    output.extend_from_slice(&position.x.to_be_bytes());
    output.extend_from_slice(&position.y.to_be_bytes());
    output.extend_from_slice(&position.z.to_be_bytes());
}

fn encode_bytes(output: &mut Vec<u8>, bytes: &[u8]) {
    output.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    output.extend_from_slice(bytes);
}
