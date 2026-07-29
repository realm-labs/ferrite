use ferrite_protocol::java_26_2::configuration::serverbound::packet::{
    ChatVisibility as JavaChatVisibility, ClientInformation, MainHand as JavaMainHand,
    ParticleStatus as JavaParticleStatus,
};
use ferrite_protocol::java_26_2::connection::output::ServerConnectionEvent;
use ferrite_protocol::java_26_2::login::profile::GameProfile;
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, JoinRequest, MainHand, ParticleStatus, SessionIdentity,
    SessionIngress, VirtualHost,
};

#[must_use]
pub fn normalize_java_event(event: ServerConnectionEvent) -> Option<SessionIngress> {
    match event {
        ServerConnectionEvent::Routed(context) => Some(SessionIngress::Routed(VirtualHost {
            host: context.host,
            port: context.port,
        })),
        ServerConnectionEvent::DisconnectExisting { profile_id } => {
            Some(SessionIngress::DisconnectDuplicate { profile_id })
        }
        ServerConnectionEvent::ConfigurationStarted { profile } => Some(
            SessionIngress::ConfigurationStarted(normalize_identity(&profile)),
        ),
        ServerConnectionEvent::RegistrySelection { .. } => None,
        ServerConnectionEvent::LatencyUpdated { latency_millis } => {
            Some(SessionIngress::LatencyUpdated { latency_millis })
        }
        ServerConnectionEvent::PlayInstallationRequested(request) => {
            Some(SessionIngress::JoinRequested(JoinRequest {
                identity: normalize_identity(&request.profile),
                settings: normalize_client_settings(request.client_information),
                transferred: request.transferred,
            }))
        }
        ServerConnectionEvent::PlayPacket { .. }
        | ServerConnectionEvent::TeleportAcknowledged(_) => None,
        ServerConnectionEvent::Closed(_) => Some(SessionIngress::Closed),
    }
}

#[must_use]
pub fn normalize_identity(profile: &GameProfile) -> SessionIdentity {
    SessionIdentity {
        profile_id: profile.id,
        name: profile.name.clone(),
    }
}

#[must_use]
pub fn normalize_client_settings(information: ClientInformation) -> ClientSettings {
    ClientSettings {
        language: information.language,
        view_distance: information.view_distance,
        chat_visibility: match information.chat_visibility {
            JavaChatVisibility::Full => ChatVisibility::Full,
            JavaChatVisibility::System => ChatVisibility::System,
            JavaChatVisibility::Hidden => ChatVisibility::Hidden,
        },
        chat_colors: information.chat_colors,
        model_customization: information.model_customization,
        main_hand: match information.main_hand {
            JavaMainHand::Left => MainHand::Left,
            JavaMainHand::Right => MainHand::Right,
        },
        text_filtering: information.text_filtering,
        allows_listing: information.allows_listing,
        particle_status: match information.particle_status {
            JavaParticleStatus::All => ParticleStatus::All,
            JavaParticleStatus::Decreased => ParticleStatus::Decreased,
            JavaParticleStatus::Minimal => ParticleStatus::Minimal,
        },
    }
}
