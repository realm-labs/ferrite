use crate::java_26_2::play::serverbound::entity_session::model::{
    EntitySessionAction, EntitySessionDisposition, PlayerMode,
};
use crate::java_26_2::play::serverbound::entity_session::packet::{
    SpectatorAction, TeleportToEntity,
};
use crate::java_26_2::play::serverbound::entity_session::projection::EntitySessionProjection;

const RANGE_PADDING: f64 = 3.0;
const KEEP_ALL_DATA_MASK: u8 = 3;

impl EntitySessionProjection {
    pub fn handle_spectator_action(&mut self, packet: SpectatorAction) -> EntitySessionDisposition {
        if !self.player.client_loaded || self.player.mode != PlayerMode::Spectator {
            return EntitySessionDisposition::Ignored;
        }
        self.reset_idle();
        let Some(target_id) = packet.target_entity_id else {
            return EntitySessionDisposition::Ignored;
        };
        let Some(target) = self.current_entity(target_id).cloned() else {
            return EntitySessionDisposition::Ignored;
        };
        let maximum_distance = self.player.interaction_range + RANGE_PADDING;
        if !target.inside_world_border
            || target.removed
            || !target.pickable
            || target.eye_to_aabb_distance_squared.is_nan()
            || target.eye_to_aabb_distance_squared >= maximum_distance * maximum_distance
        {
            return EntitySessionDisposition::Ignored;
        }

        self.player.position = target.position;
        self.player.camera_entity_id = target.entity_id;
        self.actions
            .push(EntitySessionAction::CameraTargetRelocated {
                target_entity_id: target.entity_id,
            });
        self.actions.push(EntitySessionAction::CameraPublished {
            target_entity_id: target.entity_id,
        });
        self.actions.push(EntitySessionAction::KnownPositionReset);
        EntitySessionDisposition::Handled
    }

    pub fn handle_teleport_to_entity(
        &mut self,
        packet: TeleportToEntity,
    ) -> EntitySessionDisposition {
        if self.player.mode != PlayerMode::Spectator {
            return EntitySessionDisposition::Ignored;
        }
        let Some((level, target)) = self.uuid_entity(packet.target_uuid) else {
            return EntitySessionDisposition::Ignored;
        };
        if self.player.camera_entity_id != self.player.entity_id {
            self.player.camera_entity_id = self.player.entity_id;
            self.actions.push(EntitySessionAction::CameraResetToSelf);
            self.actions.push(EntitySessionAction::CameraPublished {
                target_entity_id: self.player.entity_id,
            });
        }
        let cross_dimension = level != self.player.current_level;
        if !cross_dimension {
            self.actions
                .push(EntitySessionAction::SameDimensionTeleport {
                    target_entity_id: target.entity_id,
                });
        } else {
            self.player.current_level = level;
            self.actions
                .push(EntitySessionAction::CrossDimensionRespawn {
                    keep_mask: KEEP_ALL_DATA_MASK,
                });
        }
        self.player.position = target.position;
        self.actions.push(EntitySessionAction::PositionChallenge);
        if cross_dimension {
            self.actions.push(EntitySessionAction::LevelReprojection);
        }
        EntitySessionDisposition::Handled
    }
}
