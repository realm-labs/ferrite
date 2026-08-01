use crate::java_26_2::play::clientbound::common_services::packet::{
    CommonCustomPayload, CommonServicePacket, CommonServicePacketKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonService {
    Cookies,
    CustomPayload,
    Pong,
    ResourcePacks,
    Transfer,
    ReportDetails,
    ServerLinks,
    Dialogs,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommonServiceGates {
    pub cookies: bool,
    pub custom_payload: bool,
    pub pong: bool,
    pub resource_packs: bool,
    pub transfer: bool,
    pub report_details: bool,
    pub server_links: bool,
    pub dialogs: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommonServiceContext {
    pub singleplayer: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonServiceEffect {
    RequestCookieResponse,
    ReplaceBrand,
    DiscardCustomPayload,
    LogPongSample,
    UpdateResourcePackState,
    StoreConnectionCookie,
    TransferConnection,
    ReplaceReportDetails,
    ReplaceValidatedServerLinks,
    ClearDialogPresentation,
    ShowDialogPresentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonServiceDecision {
    OmitDisabled(CommonService),
    RefuseTransferInSingleplayer,
    Emit(CommonServiceEffect),
}

impl CommonServiceGates {
    #[must_use]
    pub const fn decide(
        self,
        packet: &CommonServicePacket,
        context: CommonServiceContext,
    ) -> CommonServiceDecision {
        let service = service(packet.kind());
        if !self.enabled(service) {
            return CommonServiceDecision::OmitDisabled(service);
        }
        if matches!(packet, CommonServicePacket::Transfer { .. }) && context.singleplayer {
            return CommonServiceDecision::RefuseTransferInSingleplayer;
        }
        CommonServiceDecision::Emit(match packet {
            CommonServicePacket::CookieRequest { .. } => CommonServiceEffect::RequestCookieResponse,
            CommonServicePacket::CustomPayload(CommonCustomPayload::Brand(_)) => {
                CommonServiceEffect::ReplaceBrand
            }
            CommonServicePacket::CustomPayload(CommonCustomPayload::Discarded { .. }) => {
                CommonServiceEffect::DiscardCustomPayload
            }
            CommonServicePacket::PongResponse { .. } => CommonServiceEffect::LogPongSample,
            CommonServicePacket::ResourcePackPop { .. }
            | CommonServicePacket::ResourcePackPush(_) => {
                CommonServiceEffect::UpdateResourcePackState
            }
            CommonServicePacket::StoreCookie { .. } => CommonServiceEffect::StoreConnectionCookie,
            CommonServicePacket::Transfer { .. } => CommonServiceEffect::TransferConnection,
            CommonServicePacket::CustomReportDetails(_) => {
                CommonServiceEffect::ReplaceReportDetails
            }
            CommonServicePacket::ServerLinks(_) => CommonServiceEffect::ReplaceValidatedServerLinks,
            CommonServicePacket::ClearDialog => CommonServiceEffect::ClearDialogPresentation,
            CommonServicePacket::ShowDialog { .. } => CommonServiceEffect::ShowDialogPresentation,
        })
    }

    const fn enabled(self, service: CommonService) -> bool {
        match service {
            CommonService::Cookies => self.cookies,
            CommonService::CustomPayload => self.custom_payload,
            CommonService::Pong => self.pong,
            CommonService::ResourcePacks => self.resource_packs,
            CommonService::Transfer => self.transfer,
            CommonService::ReportDetails => self.report_details,
            CommonService::ServerLinks => self.server_links,
            CommonService::Dialogs => self.dialogs,
        }
    }
}

const fn service(kind: CommonServicePacketKind) -> CommonService {
    match kind {
        CommonServicePacketKind::CookieRequest | CommonServicePacketKind::StoreCookie => {
            CommonService::Cookies
        }
        CommonServicePacketKind::CustomPayload => CommonService::CustomPayload,
        CommonServicePacketKind::PongResponse => CommonService::Pong,
        CommonServicePacketKind::ResourcePackPop | CommonServicePacketKind::ResourcePackPush => {
            CommonService::ResourcePacks
        }
        CommonServicePacketKind::Transfer => CommonService::Transfer,
        CommonServicePacketKind::CustomReportDetails => CommonService::ReportDetails,
        CommonServicePacketKind::ServerLinks => CommonService::ServerLinks,
        CommonServicePacketKind::ClearDialog | CommonServicePacketKind::ShowDialog => {
            CommonService::Dialogs
        }
    }
}
