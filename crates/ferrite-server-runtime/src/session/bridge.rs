use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

use ferrite_foundation::identity::{StableEntityId, StableIdError};
use ferrite_protocol::java_26_2::connection::output::ServerConnectionEvent;
use ferrite_protocol::java_26_2::login::component_json::{
    LoginDisconnectReason, LoginDisconnectReasonError,
};
use ferrite_protocol::java_26_2::login::profile::GameProfile;
use ferrite_protocol::java_26_2::login::serverbound::session::AdmissionSnapshot;
use ferrite_protocol::semantic::{
    JoinRequest, PlayAdmission, SessionDisconnectReason, SessionEgress, SessionId, SessionIdentity,
    SessionIngress, VirtualHost,
};
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

use crate::lifecycle::{AdmissionRejection, LifecycleError, NodeLifecycle};
use crate::session::admission::{AdmissionContext, AdmissionPolicy};
use crate::session::command::{SessionCommandError, SessionJoinPayload};
use crate::session::normalize::{normalize_identity, normalize_java_event};
use crate::session::route::{InitialWorldRoute, VirtualHostRoutes};
use crate::session::router::{RegionCommandRouter, RegionRouteError};

const MAX_ADMISSION_REASON_CODE_UNITS: usize = 1_024;

pub struct SessionBridge<R> {
    routes: VirtualHostRoutes,
    lifecycle: Arc<NodeLifecycle>,
    maximum_sessions: usize,
    router: R,
    sessions: BTreeMap<SessionId, SessionRecord>,
    profile_owners: BTreeMap<u128, SessionId>,
}

impl<R: RegionCommandRouter> SessionBridge<R> {
    pub fn new(
        routes: VirtualHostRoutes,
        lifecycle: Arc<NodeLifecycle>,
        maximum_sessions: usize,
        router: R,
    ) -> Result<Self, SessionBridgeError> {
        if maximum_sessions == 0 {
            return Err(SessionBridgeError::ZeroCapacity);
        }
        Ok(Self {
            routes,
            lifecycle,
            maximum_sessions,
            router,
            sessions: BTreeMap::new(),
            profile_owners: BTreeMap::new(),
        })
    }

    pub fn register(
        &mut self,
        session: SessionId,
        peer: SocketAddr,
    ) -> Result<(), SessionBridgeError> {
        if self.sessions.contains_key(&session) {
            return Err(SessionBridgeError::DuplicateSession(session));
        }
        self.lifecycle.admit_session(self.maximum_sessions)?;
        self.sessions.insert(
            session,
            SessionRecord {
                peer,
                state: SessionState::Connected,
                route: None,
                identity: None,
                latency_millis: 0,
                next_command_sequence: 0,
            },
        );
        Ok(())
    }

    pub fn unregister(&mut self, session: SessionId) -> Result<(), SessionBridgeError> {
        let record = self
            .sessions
            .remove(&session)
            .ok_or(SessionBridgeError::UnknownSession(session))?;
        if let Some(identity) = record.identity
            && self.profile_owners.get(&identity.profile_id) == Some(&session)
        {
            self.profile_owners.remove(&identity.profile_id);
        }
        self.lifecycle.complete_session()?;
        Ok(())
    }

    pub fn login_admission(
        &mut self,
        session: SessionId,
        profile: &GameProfile,
        policy: &mut impl AdmissionPolicy,
    ) -> Result<AdmissionSnapshot, SessionBridgeError> {
        let identity = normalize_identity(profile);
        let (peer, destination, state) = {
            let record = self.record(session)?;
            (
                record.peer,
                record
                    .route
                    .clone()
                    .ok_or(SessionBridgeError::MissingRoute(session))?,
                record.state,
            )
        };
        if state != SessionState::Routed {
            return Err(SessionBridgeError::UnexpectedState {
                operation: "login admission",
                expected: SessionState::Routed,
                actual: state,
            });
        }
        let denial = policy.deny_reason(&AdmissionContext {
            session,
            peer,
            identity: &identity,
            destination: &destination,
        });
        let policy_reason = if let Some(reason) = denial {
            validate_admission_reason(&reason)?;
            Some(LoginDisconnectReason::literal(&reason)?)
        } else {
            None
        };
        self.record_mut(session)?.identity = Some(identity.clone());
        Ok(AdmissionSnapshot {
            policy_reason,
            duplicate_active: self
                .profile_owners
                .get(&identity.profile_id)
                .is_some_and(|owner| *owner != session),
        })
    }

    pub fn apply_java_event(
        &mut self,
        session: SessionId,
        event: ServerConnectionEvent,
        tick: GameTick,
        policy: &mut impl AdmissionPolicy,
    ) -> Result<Option<SessionEgress>, SessionBridgeError> {
        let Some(ingress) = normalize_java_event(event) else {
            return Ok(None);
        };
        self.apply_ingress(session, ingress, tick, policy)
    }

    pub fn apply_ingress(
        &mut self,
        session: SessionId,
        ingress: SessionIngress,
        tick: GameTick,
        policy: &mut impl AdmissionPolicy,
    ) -> Result<Option<SessionEgress>, SessionBridgeError> {
        match ingress {
            SessionIngress::Routed(host) => {
                self.apply_route(session, host)?;
                Ok(None)
            }
            SessionIngress::DisconnectDuplicate { profile_id } => {
                Ok(self.profile_owners.get(&profile_id).copied().map(|owner| {
                    SessionEgress::Disconnect {
                        session: owner,
                        reason: SessionDisconnectReason::DuplicateLogin,
                    }
                }))
            }
            SessionIngress::ConfigurationStarted(identity) => {
                self.start_configuration(session, identity)?;
                Ok(None)
            }
            SessionIngress::LatencyUpdated { latency_millis } => {
                self.record_mut(session)?.latency_millis = latency_millis;
                Ok(None)
            }
            SessionIngress::JoinRequested(request) => {
                self.join(session, request, tick, policy).map(Some)
            }
            SessionIngress::Closed => {
                self.unregister(session)?;
                Ok(None)
            }
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn router_mut(&mut self) -> &mut R {
        &mut self.router
    }

    #[must_use]
    pub fn latency_millis(&self, session: SessionId) -> Option<i32> {
        self.sessions
            .get(&session)
            .map(|record| record.latency_millis)
    }

    #[must_use]
    pub fn state(&self, session: SessionId) -> Option<SessionState> {
        self.sessions.get(&session).map(|record| record.state)
    }

    fn apply_route(
        &mut self,
        session: SessionId,
        host: VirtualHost,
    ) -> Result<(), SessionBridgeError> {
        let state = self.record(session)?.state;
        if state != SessionState::Connected {
            return Err(SessionBridgeError::UnexpectedState {
                operation: "route virtual host",
                expected: SessionState::Connected,
                actual: state,
            });
        }
        let route = self.routes.resolve(&host).clone();
        let record = self.record_mut(session)?;
        record.route = Some(route);
        record.state = SessionState::Routed;
        Ok(())
    }

    fn start_configuration(
        &mut self,
        session: SessionId,
        identity: SessionIdentity,
    ) -> Result<(), SessionBridgeError> {
        let record = self.record_mut(session)?;
        if record.state != SessionState::Routed {
            return Err(SessionBridgeError::UnexpectedState {
                operation: "start configuration",
                expected: SessionState::Routed,
                actual: record.state,
            });
        }
        if record
            .identity
            .as_ref()
            .is_some_and(|admitted| admitted != &identity)
        {
            return Err(SessionBridgeError::IdentityChanged);
        }
        record.identity = Some(identity);
        record.state = SessionState::Configuration;
        Ok(())
    }

    fn join(
        &mut self,
        session: SessionId,
        request: JoinRequest,
        tick: GameTick,
        policy: &mut impl AdmissionPolicy,
    ) -> Result<SessionEgress, SessionBridgeError> {
        let (peer, destination, state, identity, sequence) = {
            let record = self.record(session)?;
            (
                record.peer,
                record
                    .route
                    .clone()
                    .ok_or(SessionBridgeError::MissingRoute(session))?,
                record.state,
                record
                    .identity
                    .clone()
                    .ok_or(SessionBridgeError::MissingIdentity(session))?,
                record.next_command_sequence,
            )
        };
        if state != SessionState::Configuration {
            return Err(SessionBridgeError::UnexpectedState {
                operation: "join Region",
                expected: SessionState::Configuration,
                actual: state,
            });
        }
        if identity != request.identity {
            return Err(SessionBridgeError::IdentityChanged);
        }
        if let Some(reason) = policy.deny_reason(&AdmissionContext {
            session,
            peer,
            identity: &identity,
            destination: &destination,
        }) {
            validate_admission_reason(&reason)?;
            return Ok(SessionEgress::Disconnect {
                session,
                reason: SessionDisconnectReason::AdmissionDenied(reason),
            });
        }
        if self
            .profile_owners
            .get(&identity.profile_id)
            .is_some_and(|owner| *owner != session)
        {
            return Ok(SessionEgress::Disconnect {
                session,
                reason: SessionDisconnectReason::DuplicateLogin,
            });
        }

        let player = StableEntityId::new(identity.profile_id)?;
        let region = destination.region();
        let next_sequence = sequence
            .checked_add(1)
            .ok_or(SessionBridgeError::SequenceExhausted)?;
        let payload = SessionJoinPayload {
            session,
            player,
            identity: identity.clone(),
            settings: request.settings,
            transferred: request.transferred,
        };
        let command = payload.into_region_command(region.clone(), tick, sequence)?;
        self.router.route(command)?;

        let record = self.record_mut(session)?;
        record.next_command_sequence = next_sequence;
        record.state = SessionState::Play;
        self.profile_owners.insert(identity.profile_id, session);
        Ok(SessionEgress::CompletePlayInstallation(PlayAdmission {
            session,
            identity,
            player,
            region,
            transferred: request.transferred,
        }))
    }

    fn record(&self, session: SessionId) -> Result<&SessionRecord, SessionBridgeError> {
        self.sessions
            .get(&session)
            .ok_or(SessionBridgeError::UnknownSession(session))
    }

    fn record_mut(&mut self, session: SessionId) -> Result<&mut SessionRecord, SessionBridgeError> {
        self.sessions
            .get_mut(&session)
            .ok_or(SessionBridgeError::UnknownSession(session))
    }
}

#[derive(Debug)]
struct SessionRecord {
    peer: SocketAddr,
    state: SessionState,
    route: Option<InitialWorldRoute>,
    identity: Option<SessionIdentity>,
    latency_millis: i32,
    next_command_sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Connected,
    Routed,
    Configuration,
    Play,
}

#[derive(Debug, Error)]
pub enum SessionBridgeError {
    #[error("session bridge capacity cannot be zero")]
    ZeroCapacity,
    #[error("session {0:?} is already registered")]
    DuplicateSession(SessionId),
    #[error("session {0:?} is not registered")]
    UnknownSession(SessionId),
    #[error("session {0:?} has no selected virtual-host route")]
    MissingRoute(SessionId),
    #[error("session {0:?} has no normalized identity")]
    MissingIdentity(SessionId),
    #[error("{operation} requires state {expected:?}, but session is {actual:?}")]
    UnexpectedState {
        operation: &'static str,
        expected: SessionState,
        actual: SessionState,
    },
    #[error("normalized identity changed within one connection")]
    IdentityChanged,
    #[error("admission reason exceeds {maximum} UTF-16 code units")]
    AdmissionReasonTooLong { maximum: usize },
    #[error("per-session Region command sequence is exhausted")]
    SequenceExhausted,
    #[error(transparent)]
    Admission(#[from] AdmissionRejection),
    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),
    #[error(transparent)]
    LoginReason(#[from] LoginDisconnectReasonError),
    #[error(transparent)]
    StableIdentity(#[from] StableIdError),
    #[error(transparent)]
    Command(#[from] SessionCommandError),
    #[error(transparent)]
    RegionRoute(#[from] RegionRouteError),
}

fn validate_admission_reason(reason: &str) -> Result<(), SessionBridgeError> {
    if reason.encode_utf16().count() > MAX_ADMISSION_REASON_CODE_UNITS {
        Err(SessionBridgeError::AdmissionReasonTooLong {
            maximum: MAX_ADMISSION_REASON_CODE_UNITS,
        })
    } else {
        Ok(())
    }
}
