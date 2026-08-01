use crate::java_26_2::play::clientbound::reconfiguration::packet::ReconfigurationPacketKind;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReconfigurationGates {
    pub reconfiguration: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReconfigurationContext {
    /// True only after the optional reconfiguration service has been registered.
    pub service_registered: bool,
    /// The request originated from an administrator-only command path.
    pub administrator: bool,
    /// Ordinary removal has saved the player and removed all old Play authority.
    pub play_removal_committed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconfigurationEffect {
    BeginTerminalPlayToConfigurationTransition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconfigurationDecision {
    OmitDisabled(ReconfigurationPacketKind),
    RefuseUnauthorized,
    DegradeServiceUnavailable,
    RefuseBeforeCommittedPlayRemoval,
    Emit(ReconfigurationEffect),
}

impl ReconfigurationGates {
    #[must_use]
    pub const fn decide(self, context: ReconfigurationContext) -> ReconfigurationDecision {
        if !self.reconfiguration {
            return ReconfigurationDecision::OmitDisabled(
                ReconfigurationPacketKind::StartConfiguration,
            );
        }
        if !context.administrator {
            return ReconfigurationDecision::RefuseUnauthorized;
        }
        if !context.service_registered {
            return ReconfigurationDecision::DegradeServiceUnavailable;
        }
        if !context.play_removal_committed {
            return ReconfigurationDecision::RefuseBeforeCommittedPlayRemoval;
        }
        ReconfigurationDecision::Emit(
            ReconfigurationEffect::BeginTerminalPlayToConfigurationTransition,
        )
    }
}
