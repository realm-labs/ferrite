use crate::java_26_2::configuration::serverbound::optional::ResourcePackAction;
use crate::java_26_2::play::serverbound::common_services::packet::{
    PlayCommonServerboundPacket, PlayCommonServerboundPacketKind,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayCommonServerboundGates {
    pub ping: bool,
    pub resource_packs: bool,
    pub custom_click_dispatch: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayCommonServerboundContext {
    pub required_resource_pack: bool,
    /// Base Minecraft has no handler; an optional child service must register one explicitly.
    pub custom_click_handler_registered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayCommonExecutionLane {
    ReceivingThreadDirect,
    ServerProcessor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayCommonServerboundEffect {
    DisconnectUnexpectedCookieResponse,
    IgnoreCustomPayload,
    EchoPingDirect { token: i64 },
    RecordResourcePackStatus { action: ResourcePackAction },
    DisconnectRequiredPackDeclined,
    LogCustomClickOnly,
    DispatchRegisteredCustomClick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayCommonServerboundDecision {
    OmitDisabled(PlayCommonServerboundPacketKind),
    DegradeNoCustomClickHandler,
    Emit(PlayCommonServerboundEffect),
}

impl PlayCommonServerboundGates {
    #[must_use]
    pub const fn decide(
        self,
        packet: PlayCommonServerboundPacket,
        context: PlayCommonServerboundContext,
    ) -> PlayCommonServerboundDecision {
        match packet {
            PlayCommonServerboundPacket::CookieResponse => PlayCommonServerboundDecision::Emit(
                PlayCommonServerboundEffect::DisconnectUnexpectedCookieResponse,
            ),
            PlayCommonServerboundPacket::CustomPayload(_) => PlayCommonServerboundDecision::Emit(
                PlayCommonServerboundEffect::IgnoreCustomPayload,
            ),
            PlayCommonServerboundPacket::PingRequest { token } => {
                if !self.ping {
                    return PlayCommonServerboundDecision::OmitDisabled(packet.kind());
                }
                PlayCommonServerboundDecision::Emit(PlayCommonServerboundEffect::EchoPingDirect {
                    token,
                })
            }
            PlayCommonServerboundPacket::ResourcePack { action } => {
                if !self.resource_packs {
                    return PlayCommonServerboundDecision::OmitDisabled(packet.kind());
                }
                if context.required_resource_pack && matches!(action, ResourcePackAction::Declined)
                {
                    return PlayCommonServerboundDecision::Emit(
                        PlayCommonServerboundEffect::DisconnectRequiredPackDeclined,
                    );
                }
                PlayCommonServerboundDecision::Emit(
                    PlayCommonServerboundEffect::RecordResourcePackStatus { action },
                )
            }
            PlayCommonServerboundPacket::CustomClickAction => {
                if !self.custom_click_dispatch {
                    return PlayCommonServerboundDecision::Emit(
                        PlayCommonServerboundEffect::LogCustomClickOnly,
                    );
                }
                if !context.custom_click_handler_registered {
                    return PlayCommonServerboundDecision::DegradeNoCustomClickHandler;
                }
                PlayCommonServerboundDecision::Emit(
                    PlayCommonServerboundEffect::DispatchRegisteredCustomClick,
                )
            }
        }
    }
}

#[must_use]
pub const fn execution_lane(kind: PlayCommonServerboundPacketKind) -> PlayCommonExecutionLane {
    match kind {
        PlayCommonServerboundPacketKind::CookieResponse
        | PlayCommonServerboundPacketKind::CustomPayload
        | PlayCommonServerboundPacketKind::PingRequest => {
            PlayCommonExecutionLane::ReceivingThreadDirect
        }
        PlayCommonServerboundPacketKind::ResourcePack
        | PlayCommonServerboundPacketKind::CustomClickAction => {
            PlayCommonExecutionLane::ServerProcessor
        }
    }
}
