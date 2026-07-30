use std::mem;

use crate::java_26_2::play::serverbound::entity_session::model::{
    EntitySessionAction, EntitySessionDisposition, PlayerMode,
};
use crate::java_26_2::play::serverbound::entity_session::packet::{
    ClientCommand, ClientCommandKind,
};
use crate::java_26_2::play::serverbound::entity_session::projection::EntitySessionProjection;

const CLIENT_LOAD_GRACE_TICKS: u32 = 60;

impl EntitySessionProjection {
    pub fn handle_client_command(&mut self, packet: ClientCommand) -> EntitySessionDisposition {
        self.reset_idle();
        match packet.action {
            ClientCommandKind::PerformRespawn => self.perform_respawn(),
            ClientCommandKind::RequestStats => {
                let values = mem::take(&mut self.player.stats_dirty);
                self.actions
                    .push(EntitySessionAction::StatsPublished { values });
                EntitySessionDisposition::Handled
            }
            ClientCommandKind::RequestGameruleValues => {
                if self.player.gamerule_permission {
                    self.actions.push(EntitySessionAction::GamerulesPublished {
                        values: self.player.gamerules.clone(),
                    });
                    EntitySessionDisposition::Handled
                } else {
                    self.actions
                        .push(EntitySessionAction::GameruleRequestDenied);
                    EntitySessionDisposition::Ignored
                }
            }
        }
    }

    fn perform_respawn(&mut self) -> EntitySessionDisposition {
        let retain_player_data = if self.player.won_game {
            self.player.won_game = false;
            true
        } else if self.player.health > 0.0 {
            return EntitySessionDisposition::Ignored;
        } else {
            false
        };

        if !retain_player_data && self.player.hardcore {
            self.player.mode = PlayerMode::Spectator;
        }
        self.player.health = 20.0;
        self.player.client_loaded = false;
        self.player.load_grace_ticks = CLIENT_LOAD_GRACE_TICKS;
        self.player.generation = self.player.generation.wrapping_add(1);
        self.player.camera_entity_id = self.player.entity_id;
        self.actions
            .push(EntitySessionAction::PlayerRespawned { retain_player_data });
        self.actions.push(EntitySessionAction::KnownPositionReset);
        self.actions
            .push(EntitySessionAction::ClientLoadGraceRestarted {
                ticks: CLIENT_LOAD_GRACE_TICKS,
            });
        self.actions.push(EntitySessionAction::RespawnPublished);
        self.actions.push(EntitySessionAction::PositionChallenge);
        self.actions.push(EntitySessionAction::LevelReprojection);
        if retain_player_data {
            self.actions
                .push(EntitySessionAction::EndToOverworldCriterion);
        }
        EntitySessionDisposition::Handled
    }
}
