use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::chat_presentation::packet::MessageSignature;
use crate::java_26_2::play::serverbound::chat::packet::ChatCommandSigned;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageBody {
    pub content: String,
    pub timestamp_millis: i64,
    pub salt: i64,
    pub last_seen: Vec<MessageSignature>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedMessageLink {
    pub sender: u128,
    pub session: u128,
    pub index: i32,
}

impl SignedMessageLink {
    #[must_use]
    pub const fn root(sender: u128, session: u128) -> Self {
        Self {
            sender,
            session,
            index: 0,
        }
    }

    #[must_use]
    pub const fn advance(self) -> Option<Self> {
        if self.index == i32::MAX {
            None
        } else {
            Some(Self {
                index: self.index + 1,
                ..self
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignedDecodeError {
    MissingProfileKey,
    MissingSignature,
    ExpiredProfileKey,
    BrokenChain,
    OutOfOrderTimestamp,
    InvalidSignature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedDecodeOutcome {
    pub signed: bool,
    pub expired_warning: bool,
    pub link: Option<SignedMessageLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageDecoder {
    Unsigned {
        enforce_secure_profile: bool,
    },
    Authenticated {
        next_link: Option<SignedMessageLink>,
        key_expires_at_millis: i64,
        previous_timestamp_millis: Option<i64>,
    },
}

impl MessageDecoder {
    #[must_use]
    pub const fn unsigned(enforce_secure_profile: bool) -> Self {
        Self::Unsigned {
            enforce_secure_profile,
        }
    }

    #[must_use]
    pub const fn authenticated(sender: u128, session: u128, key_expires_at_millis: i64) -> Self {
        Self::Authenticated {
            next_link: Some(SignedMessageLink::root(sender, session)),
            key_expires_at_millis,
            previous_timestamp_millis: None,
        }
    }

    pub fn decode(
        &mut self,
        body: &SignedMessageBody,
        signature: Option<&MessageSignature>,
        server_now_millis: i64,
        mut verify: impl FnMut(&[u8], &MessageSignature) -> bool,
    ) -> Result<SignedDecodeOutcome, SignedDecodeError> {
        match self {
            Self::Unsigned {
                enforce_secure_profile,
            } => {
                if *enforce_secure_profile {
                    Err(SignedDecodeError::MissingProfileKey)
                } else {
                    Ok(SignedDecodeOutcome {
                        signed: false,
                        expired_warning: false,
                        link: None,
                    })
                }
            }
            Self::Authenticated {
                next_link,
                key_expires_at_millis,
                previous_timestamp_millis,
            } => {
                let signature = signature.ok_or(SignedDecodeError::MissingSignature)?;
                if *key_expires_at_millis < server_now_millis {
                    return Err(SignedDecodeError::ExpiredProfileKey);
                }
                let link = next_link.ok_or(SignedDecodeError::BrokenChain)?;
                if previous_timestamp_millis
                    .is_some_and(|previous| body.timestamp_millis < previous)
                {
                    *next_link = None;
                    return Err(SignedDecodeError::OutOfOrderTimestamp);
                }
                let payload = signed_payload(link, body);
                if !verify(&payload, signature) {
                    *next_link = None;
                    return Err(SignedDecodeError::InvalidSignature);
                }
                *previous_timestamp_millis = Some(body.timestamp_millis);
                *next_link = link.advance();
                Ok(SignedDecodeOutcome {
                    signed: true,
                    expired_warning: body.timestamp_millis
                        < server_now_millis.saturating_sub(300_000),
                    link: Some(link),
                })
            }
        }
    }

    pub fn break_chain(&mut self) {
        if let Self::Authenticated { next_link, .. } = self {
            *next_link = None;
        }
    }
}

#[must_use]
pub fn signed_payload(link: SignedMessageLink, body: &SignedMessageBody) -> Vec<u8> {
    let content = body.content.as_bytes();
    let capacity = 4 + 16 + 16 + 4 + 8 + 8 + 4 + content.len() + 4 + body.last_seen.len() * 256;
    let mut payload = Vec::with_capacity(capacity);
    payload.extend_from_slice(&1_i32.to_be_bytes());
    payload.extend_from_slice(&link.sender.to_be_bytes());
    payload.extend_from_slice(&link.session.to_be_bytes());
    payload.extend_from_slice(&link.index.to_be_bytes());
    payload.extend_from_slice(&body.salt.to_be_bytes());
    payload.extend_from_slice(&body.timestamp_millis.div_euclid(1_000).to_be_bytes());
    payload.extend_from_slice(&(content.len() as i32).to_be_bytes());
    payload.extend_from_slice(content);
    payload.extend_from_slice(&(body.last_seen.len() as i32).to_be_bytes());
    for signature in &body.last_seen {
        payload.extend_from_slice(signature.0.as_ref());
    }
    payload
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignableArgument<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedArgument {
    pub content: String,
    pub outcome: SignedDecodeOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignedArgumentError {
    Decode {
        name: String,
        error: SignedDecodeError,
    },
    Mismatch,
}

pub fn collect_signed_arguments(
    decoder: &mut MessageDecoder,
    packet: &ChatCommandSigned,
    authoritative: &[SignableArgument<'_>],
    last_seen: &[MessageSignature],
    server_now_millis: i64,
    mut verify: impl FnMut(&[u8], &MessageSignature) -> bool,
) -> Result<BTreeMap<String, DecodedArgument>, SignedArgumentError> {
    let mut decoded = BTreeMap::new();
    if packet.argument_signatures.is_empty() {
        for argument in authoritative {
            let body = argument_body(packet, argument.value, last_seen);
            let outcome = decoder
                .decode(&body, None, server_now_millis, &mut verify)
                .map_err(|error| SignedArgumentError::Decode {
                    name: argument.name.to_owned(),
                    error,
                })?;
            decoded.insert(
                argument.name.to_owned(),
                DecodedArgument {
                    content: argument.value.to_owned(),
                    outcome,
                },
            );
        }
        return Ok(decoded);
    }

    for supplied in &packet.argument_signatures {
        let Some(argument) = authoritative
            .iter()
            .find(|argument| argument.name == supplied.name)
        else {
            decoder.break_chain();
            return Err(SignedArgumentError::Mismatch);
        };
        let body = argument_body(packet, argument.value, last_seen);
        let outcome = decoder
            .decode(
                &body,
                Some(&supplied.signature),
                server_now_millis,
                &mut verify,
            )
            .map_err(|error| SignedArgumentError::Decode {
                name: supplied.name.clone(),
                error,
            })?;
        decoded.insert(
            supplied.name.clone(),
            DecodedArgument {
                content: argument.value.to_owned(),
                outcome,
            },
        );
    }
    if authoritative
        .iter()
        .any(|argument| !decoded.contains_key(argument.name))
    {
        return Err(SignedArgumentError::Mismatch);
    }
    Ok(decoded)
}

fn argument_body(
    packet: &ChatCommandSigned,
    content: &str,
    last_seen: &[MessageSignature],
) -> SignedMessageBody {
    SignedMessageBody {
        content: content.to_owned(),
        timestamp_millis: packet.timestamp_millis,
        salt: packet.salt,
        last_seen: last_seen.to_vec(),
    }
}

#[must_use]
pub fn unsigned_command_allowed(
    enforce_secure_profile: bool,
    authoritative_signable_arguments: usize,
) -> bool {
    !enforce_secure_profile || authoritative_signable_arguments == 0
}
