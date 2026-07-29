use std::net::SocketAddr;

use ferrite_protocol::semantic::{SessionId, SessionIdentity};

use crate::session::route::InitialWorldRoute;

pub struct AdmissionContext<'a> {
    pub session: SessionId,
    pub peer: SocketAddr,
    pub identity: &'a SessionIdentity,
    pub destination: &'a InitialWorldRoute,
}

pub trait AdmissionPolicy {
    /// `None` admits. `Some` returns a bounded player-visible denial message.
    fn deny_reason(&mut self, context: &AdmissionContext<'_>) -> Option<String>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AllowAll;

impl AdmissionPolicy for AllowAll {
    fn deny_reason(&mut self, _context: &AdmissionContext<'_>) -> Option<String> {
        None
    }
}
