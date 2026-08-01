use crate::java_26_2::configuration::serverbound::optional::ResourcePackAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayCommonServerboundPacketKind {
    CookieResponse,
    CustomPayload,
    PingRequest,
    ResourcePack,
    CustomClickAction,
}

impl PlayCommonServerboundPacketKind {
    pub const ALL: [Self; 5] = [
        Self::CookieResponse,
        Self::CustomPayload,
        Self::PingRequest,
        Self::ResourcePack,
        Self::CustomClickAction,
    ];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::CookieResponse => 21,
            Self::CustomPayload => 22,
            Self::PingRequest => 38,
            Self::ResourcePack => 49,
            Self::CustomClickAction => 68,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::CookieResponse => "minecraft:cookie_response",
            Self::CustomPayload => "minecraft:custom_payload",
            Self::PingRequest => "minecraft:ping_request",
            Self::ResourcePack => "minecraft:resource_pack",
            Self::CustomClickAction => "minecraft:custom_click_action",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayCustomPayloadKind {
    Brand,
    Discarded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayCommonServerboundPacket {
    CookieResponse,
    CustomPayload(PlayCustomPayloadKind),
    PingRequest { token: i64 },
    ResourcePack { action: ResourcePackAction },
    CustomClickAction,
}

impl PlayCommonServerboundPacket {
    #[must_use]
    pub const fn kind(self) -> PlayCommonServerboundPacketKind {
        match self {
            Self::CookieResponse => PlayCommonServerboundPacketKind::CookieResponse,
            Self::CustomPayload(_) => PlayCommonServerboundPacketKind::CustomPayload,
            Self::PingRequest { .. } => PlayCommonServerboundPacketKind::PingRequest,
            Self::ResourcePack { .. } => PlayCommonServerboundPacketKind::ResourcePack,
            Self::CustomClickAction => PlayCommonServerboundPacketKind::CustomClickAction,
        }
    }
}
