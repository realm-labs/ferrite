use crate::java_26_2::play::serverbound::admin_state::packet::{
    AdminStatePacketKind, AdminStateRequest, CreativeStackClass, Difficulty, GameMode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminStateService {
    TagQueries,
    Difficulty,
    GameMode,
    CreativeInventory,
    GameRules,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdminStateGates {
    pub tag_queries: bool,
    pub difficulty: bool,
    pub game_mode: bool,
    pub creative_inventory: bool,
    pub game_rules: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdminStateContext {
    pub command_game_master: bool,
    pub singleplayer_owner: bool,
    pub infinite_materials: bool,
    pub difficulty_locked: bool,
    pub hardcore: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminStateEffect {
    ReplyBlockEntityTag {
        present: bool,
    },
    UpdateDifficultyAndBroadcast {
        effective: Difficulty,
    },
    NoopLockedDifficulty,
    UpdateGameMode {
        effective: GameMode,
        update_server_default: bool,
    },
    ReplaceDifficultyLockAndBroadcast,
    ReplyEntityTag,
    ClearInventorySlotAndRemoteMirror,
    SetInventorySlotAndRemoteMirror,
    ConsumeEmptyDropThrottle,
    DropItemAndConsumeThrottle,
    ApplyGameRulesSequentially,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminStateDecision {
    OmitDisabled(AdminStateService),
    RefuseUnauthorizedWithWarning(AdminStatePacketKind),
    OmitUnauthorized(AdminStatePacketKind),
    OmitMissingEntity,
    OmitFeatureDisabledStack,
    OmitInvalidStackCount,
    OmitInvalidCreativeSlot,
    OmitDropThrottled,
    Emit(AdminStateEffect),
}

impl AdminStateGates {
    #[must_use]
    pub const fn decide(
        self,
        request: AdminStateRequest,
        context: AdminStateContext,
    ) -> AdminStateDecision {
        let service = service(request.kind());
        if !self.enabled(service) {
            return AdminStateDecision::OmitDisabled(service);
        }
        match request {
            AdminStateRequest::BlockEntityTagQuery { target_exists } => {
                if !context.command_game_master {
                    return AdminStateDecision::OmitUnauthorized(request.kind());
                }
                AdminStateDecision::Emit(AdminStateEffect::ReplyBlockEntityTag {
                    present: target_exists,
                })
            }
            AdminStateRequest::ChangeDifficulty { requested } => {
                if !context.command_game_master && !context.singleplayer_owner {
                    return AdminStateDecision::RefuseUnauthorizedWithWarning(request.kind());
                }
                if context.difficulty_locked {
                    return AdminStateDecision::Emit(AdminStateEffect::NoopLockedDifficulty);
                }
                AdminStateDecision::Emit(AdminStateEffect::UpdateDifficultyAndBroadcast {
                    effective: if context.hardcore {
                        Difficulty::Hard
                    } else {
                        requested
                    },
                })
            }
            AdminStateRequest::ChangeGameMode { requested } => {
                if !context.command_game_master {
                    return AdminStateDecision::RefuseUnauthorizedWithWarning(request.kind());
                }
                AdminStateDecision::Emit(AdminStateEffect::UpdateGameMode {
                    effective: requested,
                    update_server_default: context.singleplayer_owner,
                })
            }
            AdminStateRequest::EntityTagQuery { target_exists } => {
                if !context.command_game_master {
                    return AdminStateDecision::OmitUnauthorized(request.kind());
                }
                if !target_exists {
                    return AdminStateDecision::OmitMissingEntity;
                }
                AdminStateDecision::Emit(AdminStateEffect::ReplyEntityTag)
            }
            AdminStateRequest::LockDifficulty => {
                if !context.command_game_master && !context.singleplayer_owner {
                    return AdminStateDecision::OmitUnauthorized(request.kind());
                }
                AdminStateDecision::Emit(AdminStateEffect::ReplaceDifficultyLockAndBroadcast)
            }
            AdminStateRequest::SetCreativeModeSlot {
                slot,
                stack,
                feature_enabled,
                count_within_maximum,
                drop_throttle,
            } => decide_creative_slot(
                slot,
                stack,
                feature_enabled,
                count_within_maximum,
                drop_throttle,
                context,
            ),
            AdminStateRequest::SetGameRule => {
                if !context.command_game_master {
                    return AdminStateDecision::OmitUnauthorized(request.kind());
                }
                AdminStateDecision::Emit(AdminStateEffect::ApplyGameRulesSequentially)
            }
        }
    }

    const fn enabled(self, service: AdminStateService) -> bool {
        match service {
            AdminStateService::TagQueries => self.tag_queries,
            AdminStateService::Difficulty => self.difficulty,
            AdminStateService::GameMode => self.game_mode,
            AdminStateService::CreativeInventory => self.creative_inventory,
            AdminStateService::GameRules => self.game_rules,
        }
    }
}

const fn decide_creative_slot(
    slot: i16,
    stack: CreativeStackClass,
    feature_enabled: bool,
    count_within_maximum: bool,
    drop_throttle: i32,
    context: AdminStateContext,
) -> AdminStateDecision {
    if !context.infinite_materials {
        return AdminStateDecision::OmitUnauthorized(AdminStatePacketKind::SetCreativeModeSlot);
    }
    if !feature_enabled {
        return AdminStateDecision::OmitFeatureDisabledStack;
    }
    if !count_within_maximum {
        return AdminStateDecision::OmitInvalidStackCount;
    }
    if slot >= 1 && slot <= 45 {
        return AdminStateDecision::Emit(match stack {
            CreativeStackClass::EmptyOrAir => AdminStateEffect::ClearInventorySlotAndRemoteMirror,
            CreativeStackClass::Item => AdminStateEffect::SetInventorySlotAndRemoteMirror,
        });
    }
    if slot >= 0 {
        return AdminStateDecision::OmitInvalidCreativeSlot;
    }
    if drop_throttle >= 1_480 {
        return AdminStateDecision::OmitDropThrottled;
    }
    AdminStateDecision::Emit(match stack {
        CreativeStackClass::EmptyOrAir => AdminStateEffect::ConsumeEmptyDropThrottle,
        CreativeStackClass::Item => AdminStateEffect::DropItemAndConsumeThrottle,
    })
}

const fn service(kind: AdminStatePacketKind) -> AdminStateService {
    match kind {
        AdminStatePacketKind::BlockEntityTagQuery | AdminStatePacketKind::EntityTagQuery => {
            AdminStateService::TagQueries
        }
        AdminStatePacketKind::ChangeDifficulty | AdminStatePacketKind::LockDifficulty => {
            AdminStateService::Difficulty
        }
        AdminStatePacketKind::ChangeGameMode => AdminStateService::GameMode,
        AdminStatePacketKind::SetCreativeModeSlot => AdminStateService::CreativeInventory,
        AdminStatePacketKind::SetGameRule => AdminStateService::GameRules,
    }
}
