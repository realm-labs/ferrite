use crate::java_26_2::play::serverbound::debug_subscription::packet::{
    DebugSubscriptionRequest, DebugSubscriptionRequestKind, DebugSubscriptionSet,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugSubscriptionRequestGates {
    pub diagnostics: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugSubscriptionRequestContext {
    /// True only after the optional diagnostics service has been registered.
    pub service_registered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSubscriptionRequestDecision {
    OmitDisabled(DebugSubscriptionRequestKind),
    DegradeServiceUnavailable,
    ReplaceOnLevelThread { requested: DebugSubscriptionSet },
}

impl DebugSubscriptionRequestGates {
    #[must_use]
    pub const fn decide(
        self,
        request: DebugSubscriptionRequest,
        context: DebugSubscriptionRequestContext,
    ) -> DebugSubscriptionRequestDecision {
        if !self.diagnostics {
            return DebugSubscriptionRequestDecision::OmitDisabled(
                DebugSubscriptionRequestKind::Replace,
            );
        }
        if !context.service_registered {
            return DebugSubscriptionRequestDecision::DegradeServiceUnavailable;
        }
        DebugSubscriptionRequestDecision::ReplaceOnLevelThread {
            requested: request.requested,
        }
    }
}
