use crate::java_26_2::play::clientbound::debug_projection::packet::{
    DebugProjectionPacket, DebugProjectionPacketKind, DebugRetention, DebugSubscription,
    DebugValueState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugProjectionTarget {
    Block,
    Chunk,
    Entity,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugProjectionGates {
    pub diagnostics: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugProjectionContext {
    /// True only after the optional diagnostics implementation has been registered.
    pub service_registered: bool,
    /// The requester is an operator or the integrated-server owner.
    pub authorized: bool,
    /// The connection currently requests this packet's subscription.
    pub requested: bool,
    /// The source block, chunk, entity, or event is in the recipient's tracking audience.
    pub target_tracked: bool,
    /// The entity ID still resolves in the recipient's current projection.
    pub entity_resolved: bool,
    /// The server is dedicated, which is required for the remote tick-time sample.
    pub dedicated_server: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugProjectionEffect {
    ReplaceValue {
        target: DebugProjectionTarget,
        retention: DebugRetention,
    },
    ClearValue {
        target: DebugProjectionTarget,
    },
    AppendEvent {
        retention: DebugRetention,
    },
    LogSampleImmediately,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugProjectionDecision {
    OmitDisabled(DebugProjectionPacketKind),
    RefuseUnauthorized,
    DegradeServiceUnavailable,
    RefuseSampleOnlyValue,
    OmitUnrequested,
    OmitUntrackedTarget,
    OmitMissingEntity,
    OmitUnsupportedEnvironment,
    Emit(DebugProjectionEffect),
}

impl DebugProjectionGates {
    #[must_use]
    pub const fn decide(
        self,
        packet: DebugProjectionPacket,
        context: DebugProjectionContext,
    ) -> DebugProjectionDecision {
        if !self.diagnostics {
            return DebugProjectionDecision::OmitDisabled(packet.kind());
        }
        if !context.authorized {
            return DebugProjectionDecision::RefuseUnauthorized;
        }
        if !context.service_registered {
            return DebugProjectionDecision::DegradeServiceUnavailable;
        }
        if !context.requested {
            return DebugProjectionDecision::OmitUnrequested;
        }
        if !matches!(packet, DebugProjectionPacket::Sample)
            && matches!(
                packet.subscription(),
                DebugSubscription::DedicatedServerTickTime
            )
        {
            return DebugProjectionDecision::RefuseSampleOnlyValue;
        }
        if matches!(packet, DebugProjectionPacket::Sample) && !context.dedicated_server {
            return DebugProjectionDecision::OmitUnsupportedEnvironment;
        }
        if matches!(packet, DebugProjectionPacket::EntityValue { .. }) && !context.entity_resolved {
            return DebugProjectionDecision::OmitMissingEntity;
        }
        if !matches!(packet, DebugProjectionPacket::Sample) && !context.target_tracked {
            return DebugProjectionDecision::OmitUntrackedTarget;
        }
        DebugProjectionDecision::Emit(effect(packet))
    }
}

const fn effect(packet: DebugProjectionPacket) -> DebugProjectionEffect {
    match packet {
        DebugProjectionPacket::BlockValue {
            subscription,
            state,
        } => value_effect(DebugProjectionTarget::Block, subscription, state),
        DebugProjectionPacket::ChunkValue {
            subscription,
            state,
        } => value_effect(DebugProjectionTarget::Chunk, subscription, state),
        DebugProjectionPacket::EntityValue {
            subscription,
            state,
        } => value_effect(DebugProjectionTarget::Entity, subscription, state),
        DebugProjectionPacket::Event { subscription } => DebugProjectionEffect::AppendEvent {
            retention: subscription.retention(),
        },
        DebugProjectionPacket::Sample => DebugProjectionEffect::LogSampleImmediately,
    }
}

const fn value_effect(
    target: DebugProjectionTarget,
    subscription: DebugSubscription,
    state: DebugValueState,
) -> DebugProjectionEffect {
    match state {
        DebugValueState::Replace => DebugProjectionEffect::ReplaceValue {
            target,
            retention: subscription.retention(),
        },
        DebugValueState::Clear => DebugProjectionEffect::ClearValue { target },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugProjectionLifecycleEvent {
    Reconfiguration,
    Reconnect,
    Disconnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugProjectionLifecycleEffect {
    ClearProjectionAndRequestedSubscriptions,
}

#[must_use]
pub const fn lifecycle_effect(
    _event: DebugProjectionLifecycleEvent,
) -> DebugProjectionLifecycleEffect {
    DebugProjectionLifecycleEffect::ClearProjectionAndRequestedSubscriptions
}
