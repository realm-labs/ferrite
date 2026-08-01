use crate::java_26_2::play::clientbound::chat_presentation::packet::MessageSignature;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatAck {
    pub offset: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatCommand {
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentSignature {
    pub name: String,
    pub signature: MessageSignature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LastSeenUpdate {
    pub offset: i32,
    pub acknowledged: [u8; 3],
    pub checksum: i8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatCommandSigned {
    pub command: String,
    pub timestamp_millis: i64,
    pub salt: i64,
    pub argument_signatures: Vec<ArgumentSignature>,
    pub last_seen: LastSeenUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub message: String,
    pub timestamp_millis: i64,
    pub salt: i64,
    pub signature: Option<MessageSignature>,
    pub last_seen: LastSeenUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePublicKeyData {
    pub expires_at_millis: i64,
    pub public_key: Vec<u8>,
    pub key_signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatSessionUpdate {
    pub session_id: u128,
    pub profile_key: ProfilePublicKeyData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSuggestion {
    pub transaction_id: i32,
    pub input: String,
}
