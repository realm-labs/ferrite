use crate::java_26_2::play::clientbound::admin_presentation::packet::{
    AdminPresentationPacket, AdminPresentationPacketKind,
};

pub const LOW_DISK_WARNING_THRESHOLD_BYTES: u64 = 67_108_864;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdminPresentationGates {
    pub game_rule_values: bool,
    pub game_test_highlight: bool,
    pub low_disk_warning: bool,
    pub test_instance_status: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdminPresentationContext {
    pub authorized_request: bool,
    pub direct_recipient: bool,
    pub dedicated_server: bool,
    pub administrator: bool,
    pub usable_space_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminPresentationEffect {
    PresentGameRuleValues,
    HighlightGameTestPosition,
    ShowLowDiskToast,
    PresentTestInstanceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminPresentationDecision {
    OmitDisabled(AdminPresentationPacketKind),
    RefuseUnauthorizedRequest,
    OmitNonRequester,
    OmitLowDiskWarningConditions,
    Emit(AdminPresentationEffect),
}

impl AdminPresentationGates {
    #[must_use]
    pub const fn decide(
        self,
        packet: &AdminPresentationPacket,
        context: AdminPresentationContext,
    ) -> AdminPresentationDecision {
        let kind = packet.kind();
        if !self.enabled(kind) {
            return AdminPresentationDecision::OmitDisabled(kind);
        }
        match packet {
            AdminPresentationPacket::GameRuleValues(_) => {
                if !context.authorized_request {
                    return AdminPresentationDecision::RefuseUnauthorizedRequest;
                }
                if !context.direct_recipient {
                    return AdminPresentationDecision::OmitNonRequester;
                }
                AdminPresentationDecision::Emit(AdminPresentationEffect::PresentGameRuleValues)
            }
            AdminPresentationPacket::GameTestHighlightPosition { .. } => {
                if !context.direct_recipient {
                    return AdminPresentationDecision::OmitNonRequester;
                }
                AdminPresentationDecision::Emit(AdminPresentationEffect::HighlightGameTestPosition)
            }
            AdminPresentationPacket::LowDiskSpaceWarning => {
                let below_threshold = matches!(
                    context.usable_space_bytes,
                    Some(bytes) if bytes < LOW_DISK_WARNING_THRESHOLD_BYTES
                );
                if !context.dedicated_server || !context.administrator || !below_threshold {
                    return AdminPresentationDecision::OmitLowDiskWarningConditions;
                }
                AdminPresentationDecision::Emit(AdminPresentationEffect::ShowLowDiskToast)
            }
            AdminPresentationPacket::TestInstanceBlockStatus { .. } => {
                if !context.authorized_request {
                    return AdminPresentationDecision::RefuseUnauthorizedRequest;
                }
                if !context.direct_recipient {
                    return AdminPresentationDecision::OmitNonRequester;
                }
                AdminPresentationDecision::Emit(AdminPresentationEffect::PresentTestInstanceStatus)
            }
        }
    }

    const fn enabled(self, kind: AdminPresentationPacketKind) -> bool {
        match kind {
            AdminPresentationPacketKind::GameRuleValues => self.game_rule_values,
            AdminPresentationPacketKind::GameTestHighlightPosition => self.game_test_highlight,
            AdminPresentationPacketKind::LowDiskSpaceWarning => self.low_disk_warning,
            AdminPresentationPacketKind::TestInstanceBlockStatus => self.test_instance_status,
        }
    }
}
