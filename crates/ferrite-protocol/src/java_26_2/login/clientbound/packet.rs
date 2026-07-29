use crate::java_26_2::login::component_json::LoginDisconnectReason;
use crate::java_26_2::login::profile::GameProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginClientboundPacket {
    Disconnect(LoginDisconnectReason),
    Finished(LoginFinished),
    Compression(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginFinished {
    pub profile: GameProfile,
    pub server_session_id: u128,
}
