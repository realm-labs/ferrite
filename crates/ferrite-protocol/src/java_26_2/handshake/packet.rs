#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientIntentionPacket {
    pub protocol_version: i32,
    pub host: String,
    pub port: u16,
    pub intention: ClientIntention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIntention {
    Status,
    Login,
    Transfer,
}

impl ClientIntention {
    pub(crate) const fn id(self) -> i32 {
        match self {
            Self::Status => 1,
            Self::Login => 2,
            Self::Transfer => 3,
        }
    }

    pub(crate) const fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Status),
            2 => Some(Self::Login),
            3 => Some(Self::Transfer),
            _ => None,
        }
    }
}
