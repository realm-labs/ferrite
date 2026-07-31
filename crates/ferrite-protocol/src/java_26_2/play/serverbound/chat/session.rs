use crate::java_26_2::play::serverbound::chat::packet::{ChatSessionUpdate, ProfilePublicKeyData};
use crate::java_26_2::play::serverbound::chat::signing::MessageDecoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileKeyValidation {
    ValidatorUnavailable,
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledChatSession {
    pub session_id: u128,
    pub profile_key: ProfilePublicKeyData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatSessionAction {
    NoOpEqualKeyData,
    WarnAndIgnoreMissingValidator,
    DisconnectExpiredPublicKey,
    DisconnectInvalidPublicKey,
    InstalledAndBroadcastInitializeChat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatSessionState {
    player_id: u128,
    enforce_secure_profile: bool,
    installed: Option<InstalledChatSession>,
    decoder: MessageDecoder,
}

impl ChatSessionState {
    #[must_use]
    pub const fn new(player_id: u128, enforce_secure_profile: bool) -> Self {
        Self {
            player_id,
            enforce_secure_profile,
            installed: None,
            decoder: MessageDecoder::unsigned(enforce_secure_profile),
        }
    }

    #[must_use]
    pub const fn installed(&self) -> Option<&InstalledChatSession> {
        self.installed.as_ref()
    }

    #[must_use]
    pub const fn decoder(&self) -> &MessageDecoder {
        &self.decoder
    }

    pub fn decoder_mut(&mut self) -> &mut MessageDecoder {
        &mut self.decoder
    }

    pub fn apply_update(
        &mut self,
        packet: ChatSessionUpdate,
        validate: impl FnOnce(&[u8], &[u8]) -> ProfileKeyValidation,
    ) -> ChatSessionAction {
        if self
            .installed
            .as_ref()
            .is_some_and(|current| current.profile_key == packet.profile_key)
        {
            return ChatSessionAction::NoOpEqualKeyData;
        }
        if self.installed.as_ref().is_some_and(|current| {
            packet.profile_key.expires_at_millis < current.profile_key.expires_at_millis
        }) {
            return ChatSessionAction::DisconnectExpiredPublicKey;
        }
        match validate(
            &profile_key_signed_payload(self.player_id, &packet.profile_key),
            &packet.profile_key.key_signature,
        ) {
            ProfileKeyValidation::ValidatorUnavailable => {
                ChatSessionAction::WarnAndIgnoreMissingValidator
            }
            ProfileKeyValidation::Invalid => ChatSessionAction::DisconnectInvalidPublicKey,
            ProfileKeyValidation::Valid => {
                self.decoder = MessageDecoder::authenticated(
                    self.player_id,
                    packet.session_id,
                    packet.profile_key.expires_at_millis,
                );
                self.installed = Some(InstalledChatSession {
                    session_id: packet.session_id,
                    profile_key: packet.profile_key,
                });
                ChatSessionAction::InstalledAndBroadcastInitializeChat
            }
        }
    }

    #[must_use]
    pub const fn enforce_secure_profile(&self) -> bool {
        self.enforce_secure_profile
    }
}

#[must_use]
pub fn profile_key_signed_payload(player_id: u128, key: &ProfilePublicKeyData) -> Vec<u8> {
    let mut payload = Vec::with_capacity(24 + key.public_key.len());
    payload.extend_from_slice(&player_id.to_be_bytes());
    payload.extend_from_slice(&key.expires_at_millis.to_be_bytes());
    payload.extend_from_slice(&key.public_key);
    payload
}

#[must_use]
pub fn verify_sha256_rsa(public_key_der: &[u8], payload: &[u8], signature: &[u8]) -> bool {
    let Ok(public_key) = RsaPublicKey::from_public_key_der(public_key_der) else {
        return false;
    };
    let Ok(signature) = Signature::try_from(signature) else {
        return false;
    };
    VerifyingKey::<Sha256>::new(public_key)
        .verify(payload, &signature)
        .is_ok()
}
use rsa::RsaPublicKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::pkcs8::DecodePublicKey;
use rsa::signature::Verifier;
use sha2::Sha256;
