use crate::player::dispatch::ServerboundDispatchOutcome;
use crate::runtime_status::{MinecraftSessionStatus, ServerboundDispatchStatus};

use super::NetworkSession;

impl NetworkSession {
    pub(super) fn status(&self) -> MinecraftSessionStatus {
        let player = self.player.as_ref();
        let region = player.map(|connection| connection.player().region().coordinate());
        let state = player.map(|connection| connection.player().committed_state());
        let position = state.map(|state| state.pose().position);
        let interest = player.map(|connection| connection.chunks().stream().interest());
        MinecraftSessionStatus {
            session_id: self.id.get(),
            player: player.map(|connection| connection.stable_id().to_string()),
            region_x: region.map(ferrite_foundation::region::RegionCoord::x),
            region_z: region.map(ferrite_foundation::region::RegionCoord::z),
            dimension: player
                .map(|connection| connection.player().region().dimension().to_string()),
            x: position.map(|position| position.x),
            y: position.map(|position| position.y),
            z: position.map(|position| position.z),
            on_ground: state.map(ferrite_gameplay::player::state::PlayerSessionState::on_ground),
            view_chunks: interest.map(|interest| interest.view().len()),
            pending_chunks: interest.map(|interest| {
                interest
                    .known()
                    .values()
                    .filter(|state| {
                        matches!(state, crate::chunk::interest::KnownChunkState::Pending)
                    })
                    .count()
            }),
            sent_chunks: interest.map(|interest| {
                interest
                    .known()
                    .values()
                    .filter(|state| {
                        matches!(state, crate::chunk::interest::KnownChunkState::Sent { .. })
                    })
                    .count()
            }),
            unacknowledged_chunk_batches: player
                .map(|connection| connection.chunks().stream().unacknowledged_batches()),
            pending_outbound_frames: self.connection.pending_outbound(),
            pending_write_bytes: self.pending_write.as_ref().map_or(0, |pending| {
                pending.frame.bytes.len().saturating_sub(pending.offset)
            }),
            region_transfers: self.region_transfers,
            last_dispatch: self.last_dispatch.map(dispatch_status),
            last_unsupported_dispatch: self.last_unsupported_dispatch.map(dispatch_status),
            last_block_result: self.last_block_result,
        }
    }
}

const fn dispatch_status(outcome: ServerboundDispatchOutcome) -> ServerboundDispatchStatus {
    ServerboundDispatchStatus {
        packet: outcome.packet(),
        responsibility: outcome.responsibility_name(),
        disposition: outcome.disposition_name(),
        detail: outcome.disposition_detail(),
    }
}
