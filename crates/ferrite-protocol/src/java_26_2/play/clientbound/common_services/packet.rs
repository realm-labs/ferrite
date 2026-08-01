use std::collections::BTreeMap;

use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::{NetworkNbt, TextComponentNbt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonServicePacketKind {
    CookieRequest,
    CustomPayload,
    PongResponse,
    ResourcePackPop,
    ResourcePackPush,
    StoreCookie,
    Transfer,
    CustomReportDetails,
    ServerLinks,
    ClearDialog,
    ShowDialog,
}

impl CommonServicePacketKind {
    pub const ALL: [Self; 11] = [
        Self::CookieRequest,
        Self::CustomPayload,
        Self::PongResponse,
        Self::ResourcePackPop,
        Self::ResourcePackPush,
        Self::StoreCookie,
        Self::Transfer,
        Self::CustomReportDetails,
        Self::ServerLinks,
        Self::ClearDialog,
        Self::ShowDialog,
    ];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::CookieRequest => 21,
            Self::CustomPayload => 24,
            Self::PongResponse => 62,
            Self::ResourcePackPop => 80,
            Self::ResourcePackPush => 81,
            Self::StoreCookie => 120,
            Self::Transfer => 129,
            Self::CustomReportDetails => 136,
            Self::ServerLinks => 137,
            Self::ClearDialog => 139,
            Self::ShowDialog => 140,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::CookieRequest => "minecraft:cookie_request",
            Self::CustomPayload => "minecraft:custom_payload",
            Self::PongResponse => "minecraft:pong_response",
            Self::ResourcePackPop => "minecraft:resource_pack_pop",
            Self::ResourcePackPush => "minecraft:resource_pack_push",
            Self::StoreCookie => "minecraft:store_cookie",
            Self::Transfer => "minecraft:transfer",
            Self::CustomReportDetails => "minecraft:custom_report_details",
            Self::ServerLinks => "minecraft:server_links",
            Self::ClearDialog => "minecraft:clear_dialog",
            Self::ShowDialog => "minecraft:show_dialog",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommonServicePacket {
    CookieRequest { key: Identifier },
    CustomPayload(CommonCustomPayload),
    PongResponse { token: i64 },
    ResourcePackPop { pack_id: Option<u128> },
    ResourcePackPush(ResourcePackPush),
    StoreCookie { key: Identifier, value: Vec<u8> },
    Transfer { host: String, port: i32 },
    CustomReportDetails(BTreeMap<String, String>),
    ServerLinks(Vec<ServerLink>),
    ClearDialog,
    ShowDialog { dialog: DialogHolder },
}

impl CommonServicePacket {
    #[must_use]
    pub const fn kind(&self) -> CommonServicePacketKind {
        match self {
            Self::CookieRequest { .. } => CommonServicePacketKind::CookieRequest,
            Self::CustomPayload(_) => CommonServicePacketKind::CustomPayload,
            Self::PongResponse { .. } => CommonServicePacketKind::PongResponse,
            Self::ResourcePackPop { .. } => CommonServicePacketKind::ResourcePackPop,
            Self::ResourcePackPush(_) => CommonServicePacketKind::ResourcePackPush,
            Self::StoreCookie { .. } => CommonServicePacketKind::StoreCookie,
            Self::Transfer { .. } => CommonServicePacketKind::Transfer,
            Self::CustomReportDetails(_) => CommonServicePacketKind::CustomReportDetails,
            Self::ServerLinks(_) => CommonServicePacketKind::ServerLinks,
            Self::ClearDialog => CommonServicePacketKind::ClearDialog,
            Self::ShowDialog { .. } => CommonServicePacketKind::ShowDialog,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommonCustomPayload {
    Brand(String),
    Discarded {
        channel: Identifier,
        payload: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePackPush {
    pub pack_id: u128,
    pub url: String,
    pub hash: String,
    pub required: bool,
    pub prompt: Option<TextComponentNbt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLink {
    pub label: ServerLinkLabel,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerLinkLabel {
    Known(i32),
    Custom(TextComponentNbt),
}

impl ServerLinkLabel {
    #[must_use]
    pub const fn effective_known_type(&self) -> Option<i32> {
        match self {
            Self::Known(raw_id @ 0..=9) => Some(*raw_id),
            Self::Known(_) => Some(0),
            Self::Custom(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogHolder {
    Registered(i32),
    Direct(NetworkNbt),
}
