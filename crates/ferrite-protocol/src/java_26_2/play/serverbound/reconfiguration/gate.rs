use crate::java_26_2::play::serverbound::reconfiguration::packet::ServerboundReconfigurationPacketKind;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServerboundReconfigurationGates {
    pub reconfiguration: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServerboundReconfigurationContext {
    /// True only after the optional reconfiguration service has been registered.
    pub service_registered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundReconfigurationDecision {
    OmitDisabled(ServerboundReconfigurationPacketKind),
    DegradeServiceUnavailable,
    AdmitTerminalAcknowledgement,
}

impl ServerboundReconfigurationGates {
    #[must_use]
    pub const fn decide(
        self,
        context: ServerboundReconfigurationContext,
    ) -> ServerboundReconfigurationDecision {
        if !self.reconfiguration {
            return ServerboundReconfigurationDecision::OmitDisabled(
                ServerboundReconfigurationPacketKind::ConfigurationAcknowledged,
            );
        }
        if !context.service_registered {
            return ServerboundReconfigurationDecision::DegradeServiceUnavailable;
        }
        ServerboundReconfigurationDecision::AdmitTerminalAcknowledgement
    }
}
