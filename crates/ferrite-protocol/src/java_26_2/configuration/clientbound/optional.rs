#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalConfigurationPacket {
    CookieRequest,
    ResetChat,
    ResourcePackPop,
    ResourcePackPush,
    StoreCookie,
    Transfer,
    CustomReportDetails,
    ServerLinks,
    ClearDialog,
    ShowDialog,
    CodeOfConduct,
}

impl OptionalConfigurationPacket {
    pub const ALL: [Self; 11] = [
        Self::CookieRequest,
        Self::ResetChat,
        Self::ResourcePackPop,
        Self::ResourcePackPush,
        Self::StoreCookie,
        Self::Transfer,
        Self::CustomReportDetails,
        Self::ServerLinks,
        Self::ClearDialog,
        Self::ShowDialog,
        Self::CodeOfConduct,
    ];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::CookieRequest => 0,
            Self::ResetChat => 6,
            Self::ResourcePackPop => 8,
            Self::ResourcePackPush => 9,
            Self::StoreCookie => 10,
            Self::Transfer => 11,
            Self::CustomReportDetails => 15,
            Self::ServerLinks => 16,
            Self::ClearDialog => 17,
            Self::ShowDialog => 18,
            Self::CodeOfConduct => 19,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::CookieRequest => "minecraft:cookie_request",
            Self::ResetChat => "minecraft:reset_chat",
            Self::ResourcePackPop => "minecraft:resource_pack_pop",
            Self::ResourcePackPush => "minecraft:resource_pack_push",
            Self::StoreCookie => "minecraft:store_cookie",
            Self::Transfer => "minecraft:transfer",
            Self::CustomReportDetails => "minecraft:custom_report_details",
            Self::ServerLinks => "minecraft:server_links",
            Self::ClearDialog => "minecraft:clear_dialog",
            Self::ShowDialog => "minecraft:show_dialog",
            Self::CodeOfConduct => "minecraft:code_of_conduct",
        }
    }

    #[must_use]
    pub const fn service(self) -> ConfigurationOptionalService {
        match self {
            Self::CookieRequest | Self::StoreCookie => ConfigurationOptionalService::Cookies,
            Self::ResetChat => ConfigurationOptionalService::Reconfiguration,
            Self::ResourcePackPop | Self::ResourcePackPush => {
                ConfigurationOptionalService::ResourcePacks
            }
            Self::Transfer => ConfigurationOptionalService::Transfer,
            Self::CustomReportDetails => ConfigurationOptionalService::ReportDetails,
            Self::ServerLinks => ConfigurationOptionalService::ServerLinks,
            Self::ClearDialog | Self::ShowDialog => ConfigurationOptionalService::Dialogs,
            Self::CodeOfConduct => ConfigurationOptionalService::CodeOfConduct,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationOptionalService {
    Cookies,
    Reconfiguration,
    ResourcePacks,
    Transfer,
    ReportDetails,
    ServerLinks,
    Dialogs,
    CodeOfConduct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationPhase {
    Fresh,
    Reconfiguration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionalConfigurationContext {
    pub phase: ConfigurationPhase,
    pub singleplayer: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalClientboundEffect {
    RequestResponse,
    BlockingTask,
    PresentationOnly,
    StoreConnectionState,
    TransferConnection,
    ResetRetainedChat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalClientboundDecision {
    OmitDisabled(ConfigurationOptionalService),
    OmitOutsideReconfiguration,
    RefuseTransferInSingleplayer,
    Emit(OptionalClientboundEffect),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigurationClientboundGates {
    pub cookies: bool,
    pub reconfiguration: bool,
    pub resource_packs: bool,
    pub transfer: bool,
    pub report_details: bool,
    pub server_links: bool,
    pub dialogs: bool,
    pub code_of_conduct: bool,
}

impl ConfigurationClientboundGates {
    #[must_use]
    pub const fn decide(
        self,
        packet: OptionalConfigurationPacket,
        context: OptionalConfigurationContext,
    ) -> OptionalClientboundDecision {
        let service = packet.service();
        if !self.enabled(service) {
            return OptionalClientboundDecision::OmitDisabled(service);
        }
        if matches!(packet, OptionalConfigurationPacket::ResetChat)
            && !matches!(context.phase, ConfigurationPhase::Reconfiguration)
        {
            return OptionalClientboundDecision::OmitOutsideReconfiguration;
        }
        if matches!(packet, OptionalConfigurationPacket::Transfer) && context.singleplayer {
            return OptionalClientboundDecision::RefuseTransferInSingleplayer;
        }
        OptionalClientboundDecision::Emit(match packet {
            OptionalConfigurationPacket::CookieRequest => {
                OptionalClientboundEffect::RequestResponse
            }
            OptionalConfigurationPacket::ResourcePackPush
            | OptionalConfigurationPacket::CodeOfConduct => OptionalClientboundEffect::BlockingTask,
            OptionalConfigurationPacket::StoreCookie => {
                OptionalClientboundEffect::StoreConnectionState
            }
            OptionalConfigurationPacket::Transfer => OptionalClientboundEffect::TransferConnection,
            OptionalConfigurationPacket::ResetChat => OptionalClientboundEffect::ResetRetainedChat,
            OptionalConfigurationPacket::ResourcePackPop
            | OptionalConfigurationPacket::CustomReportDetails
            | OptionalConfigurationPacket::ServerLinks
            | OptionalConfigurationPacket::ClearDialog
            | OptionalConfigurationPacket::ShowDialog => {
                OptionalClientboundEffect::PresentationOnly
            }
        })
    }

    const fn enabled(self, service: ConfigurationOptionalService) -> bool {
        match service {
            ConfigurationOptionalService::Cookies => self.cookies,
            ConfigurationOptionalService::Reconfiguration => self.reconfiguration,
            ConfigurationOptionalService::ResourcePacks => self.resource_packs,
            ConfigurationOptionalService::Transfer => self.transfer,
            ConfigurationOptionalService::ReportDetails => self.report_details,
            ConfigurationOptionalService::ServerLinks => self.server_links,
            ConfigurationOptionalService::Dialogs => self.dialogs,
            ConfigurationOptionalService::CodeOfConduct => self.code_of_conduct,
        }
    }
}
