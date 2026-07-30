use crate::java_26_2::play::serverbound::entity_session::model::{
    AttackRangeProjection, EntitySessionAction, EntitySessionDisposition, EntitySessionPlayer,
    PlayerMode, SessionEntityKind, SessionEntityProjection, SessionItemStack, SwingSource,
};
use crate::java_26_2::play::serverbound::entity_session::packet::{
    Attack, Interact, PickItemFromEntity,
};
use crate::java_26_2::play::serverbound::entity_session::projection::EntitySessionProjection;

const RANGE_PADDING: f64 = 3.0;

impl EntitySessionProjection {
    pub fn handle_attack(&mut self, packet: Attack) -> EntitySessionDisposition {
        if !self.player.client_loaded || self.player.mode == PlayerMode::Spectator {
            return EntitySessionDisposition::Ignored;
        }
        let target = self.current_entity(packet.target_entity_id).cloned();
        self.reset_idle();
        let Some(target) = target else {
            return EntitySessionDisposition::Ignored;
        };
        if !target.inside_world_border || !attack_reach_allows(&self.player, &target) {
            return EntitySessionDisposition::Ignored;
        }
        if self.player.main_hand.piercing_weapon {
            return EntitySessionDisposition::Ignored;
        }
        if invalid_attack_target(self.player.entity_id, target.entity_id, target.kind) {
            self.actions
                .push(EntitySessionAction::DisconnectInvalidAttack {
                    target_entity_id: target.entity_id,
                });
            return EntitySessionDisposition::DisconnectInvalidAttack;
        }
        if !self.player.main_hand.is_empty() && !self.player.main_hand.feature_enabled {
            return EntitySessionDisposition::Ignored;
        }
        if !self.player.minimum_attack_charge_met {
            return EntitySessionDisposition::Ignored;
        }
        self.actions.push(EntitySessionAction::AttackExecuted {
            target_entity_id: target.entity_id,
        });
        EntitySessionDisposition::Handled
    }

    pub fn handle_interact(&mut self, packet: Interact) -> EntitySessionDisposition {
        if !self.player.client_loaded {
            return EntitySessionDisposition::Ignored;
        }
        let target = self.current_entity(packet.target_entity_id).cloned();
        self.reset_idle();
        self.player.shift_key_down = packet.secondary_action;
        let Some(target) = target else {
            return EntitySessionDisposition::Ignored;
        };
        if !target.inside_world_border
            || !strict_interaction_reach(
                target.eye_to_aabb_distance_squared,
                self.player.interaction_range,
            )
        {
            return EntitySessionDisposition::Ignored;
        }
        let before = self.player.hand(packet.hand).clone();
        if !before.is_empty() && !before.feature_enabled {
            return EntitySessionDisposition::Ignored;
        }
        if self.player.mode == PlayerMode::Spectator {
            if target.menu_provider {
                self.actions.push(EntitySessionAction::SpectatorMenuOpened {
                    target_entity_id: target.entity_id,
                });
                return EntitySessionDisposition::Handled;
            }
            return EntitySessionDisposition::Ignored;
        }

        self.actions.push(EntitySessionAction::TargetInteraction {
            target_entity_id: target.entity_id,
            hand: packet.hand,
            location: packet.location,
        });
        let mut result = target.target_interaction;
        let mut item_result = false;
        if !result.consumes_action()
            && !before.is_empty()
            && target.kind.is_living()
            && let Some(item_interaction) = target.item_interaction
        {
            self.actions.push(EntitySessionAction::ItemInteraction {
                target_entity_id: target.entity_id,
                hand: packet.hand,
            });
            result = item_interaction.result;
            item_result = true;
            if result.consumes_action() {
                let mut resulting_stack = item_interaction.resulting_stack;
                if self.player.infinite_materials {
                    resulting_stack.count = before.count;
                }
                *self.player.hand_mut(packet.hand) = resulting_stack;
                self.actions
                    .push(EntitySessionAction::EntityInteractGameEvent {
                        target_entity_id: target.entity_id,
                    });
            }
        }
        if result.is_success() {
            self.actions
                .push(EntitySessionAction::InteractionCriterion {
                    target_entity_id: target.entity_id,
                    stack: if item_result {
                        before
                    } else {
                        SessionItemStack::default()
                    },
                });
        }
        if result.swing_source() == SwingSource::Server {
            self.actions.push(EntitySessionAction::SwingPublished {
                hand: packet.hand,
                include_self: true,
            });
        }
        if result.consumes_action() {
            EntitySessionDisposition::Handled
        } else {
            EntitySessionDisposition::Ignored
        }
    }

    pub fn handle_pick(&mut self, packet: PickItemFromEntity) -> EntitySessionDisposition {
        let target = self.current_entity(packet.target_entity_id).cloned();
        let Some(target) = target else {
            return EntitySessionDisposition::Ignored;
        };
        if target.removed
            || !strict_interaction_reach(
                target.eye_to_aabb_distance_squared,
                self.player.interaction_range,
            )
        {
            return EntitySessionDisposition::Ignored;
        }

        let mut handled = false;
        if let Some(stack) = target.pick_result.clone()
            && !stack.is_empty()
            && stack.feature_enabled
        {
            self.select_picked_stack(stack);
            handled = true;
        }
        if packet.include_data && self.player.can_use_game_master_blocks && target.kind.is_avatar()
        {
            self.actions
                .push(EntitySessionAction::AvatarProfilePrinted {
                    target_entity_id: target.entity_id,
                });
            handled = true;
        }
        if handled {
            EntitySessionDisposition::Handled
        } else {
            EntitySessionDisposition::Ignored
        }
    }

    fn select_picked_stack(&mut self, stack: SessionItemStack) {
        if let Some(slot) = self
            .player
            .hotbar
            .iter()
            .position(|candidate| candidate.same_item_and_components(&stack))
        {
            self.player.selected_hotbar = slot;
        } else if let Some(slot) = self
            .player
            .inventory
            .iter()
            .position(|candidate| candidate.same_item_and_components(&stack))
        {
            let selected = self.player.selected_hotbar;
            std::mem::swap(
                &mut self.player.hotbar[selected],
                &mut self.player.inventory[slot],
            );
        } else if self.player.infinite_materials {
            self.player.hotbar[self.player.selected_hotbar] = stack;
        }
        self.player.main_hand = self.player.hotbar[self.player.selected_hotbar].clone();
        self.actions.push(EntitySessionAction::HeldSlotConvergence {
            slot: self.player.selected_hotbar,
        });
        self.actions
            .push(EntitySessionAction::InventoryMenuConvergence);
    }
}

fn invalid_attack_target(player_id: i32, target_id: i32, kind: SessionEntityKind) -> bool {
    player_id == target_id
        || matches!(
            kind,
            SessionEntityKind::Item
                | SessionEntityKind::ExperienceOrb
                | SessionEntityKind::AbstractArrow { attackable: false }
        )
}

fn attack_reach_allows(player: &EntitySessionPlayer, target: &SessionEntityProjection) -> bool {
    let distance = target.eye_to_aabb_distance_squared.max(0.0).sqrt();
    let range = player
        .main_hand
        .attack_range
        .unwrap_or(AttackRangeProjection {
            minimum: 0.0,
            maximum: player.interaction_range,
            creative_minimum: 0.0,
            creative_maximum: player.interaction_range,
            hitbox_margin: 0.0,
            mob_factor: 1.0,
        });
    let (minimum, maximum) = if player.mode == PlayerMode::Creative {
        (range.creative_minimum, range.creative_maximum)
    } else {
        (
            range.minimum * range.mob_factor,
            range.maximum * range.mob_factor,
        )
    };
    distance >= minimum - range.hitbox_margin - RANGE_PADDING
        && distance <= maximum + range.hitbox_margin + RANGE_PADDING
}

fn strict_interaction_reach(distance_squared: f64, interaction_range: f64) -> bool {
    !distance_squared.is_nan() && distance_squared < (interaction_range + RANGE_PADDING).powi(2)
}
