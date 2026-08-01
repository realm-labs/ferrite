//! Executable NetworkIngress × PlayerLifecycle conformance.

use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use ferrite_gameplay::player::lifecycle::runtime::PlayerLifecycle;
use ferrite_protocol::java_26_2::login::profile::GameProfile;
use ferrite_protocol::semantic::{
    JoinRequest, SessionEgress, SessionId, SessionIngress, VirtualHost,
};
use ferrite_server_runtime::lifecycle::NodeLifecycle;
use ferrite_server_runtime::session::admission::{AdmissionContext, AdmissionPolicy};
use ferrite_server_runtime::session::bridge::{SessionBridge, SessionBridgeError, SessionState};
use ferrite_server_runtime::session::command::{SessionJoinPayload, SessionLeavePayload};
use ferrite_server_runtime::session::route::{InitialWorldRoute, VirtualHostRoutes};
use ferrite_server_runtime::session::router::{RegionCommandRouter, RegionRouteError};
use ferrite_simulation::command::RegionCommand;
use ferrite_simulation::tick::GameTick;

use crate::player_service::fixtures::{identity, player, region, settings, spawn_chunk};

const PROPERTY_CASES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressPlayerLifecycleReport {
    pub property_cases: usize,
    pub fault_cases: usize,
    pub join_effects: usize,
    pub leave_effects: usize,
    pub routed_commands: usize,
}

pub fn run_network_ingress_player_lifecycle() -> IngressPlayerLifecycleReport {
    run_transition_properties();
    let fault_cases = run_fault_vectors();
    let (join_effects, leave_effects, routed_commands) = run_golden_transition();
    IngressPlayerLifecycleReport {
        property_cases: PROPERTY_CASES,
        fault_cases,
        join_effects,
        leave_effects,
        routed_commands,
    }
}

fn run_golden_transition() -> (usize, usize, usize) {
    let (mut bridge, session, mut policy) = routed_bridge();
    bridge
        .login_admission(session, &profile(1), &mut policy)
        .unwrap();
    bridge
        .apply_ingress(
            session,
            SessionIngress::ConfigurationStarted(identity(1)),
            GameTick::ZERO,
            &mut policy,
        )
        .unwrap();
    let egress = bridge
        .apply_ingress(
            session,
            SessionIngress::JoinRequested(JoinRequest {
                identity: identity(1),
                settings: settings(),
                transferred: false,
            }),
            GameTick::new(1),
            &mut policy,
        )
        .unwrap()
        .expect("join produces play installation");
    assert!(matches!(egress, SessionEgress::CompletePlayInstallation(_)));
    assert_eq!(bridge.state(session), Some(SessionState::Play));
    let join = SessionJoinPayload::decode(bridge.router_mut().commands[0].payload()).unwrap();
    assert_eq!(join.player, player(1));

    let mut lifecycle = PlayerLifecycle::new(1).unwrap();
    let join_effects = lifecycle
        .join(join.player, 1, join.transferred)
        .unwrap()
        .len();
    bridge
        .apply_ingress(
            session,
            SessionIngress::Closed,
            GameTick::new(2),
            &mut policy,
        )
        .unwrap();
    assert!(bridge.is_empty());
    let leave = SessionLeavePayload::decode(bridge.router_mut().commands[1].payload()).unwrap();
    assert_eq!(leave.player, join.player);
    let leave_effects = lifecycle.disconnect(leave.player).unwrap().len();
    assert!(lifecycle.snapshot().players.is_empty());
    (
        join_effects,
        leave_effects,
        bridge.router_mut().commands.len(),
    )
}

fn run_transition_properties() {
    for case in 0..PROPERTY_CASES {
        let value = case as u128 + 1;
        let (mut bridge, session, mut policy) = routed_bridge();
        bridge
            .login_admission(session, &profile(value), &mut policy)
            .unwrap();
        bridge
            .apply_ingress(
                session,
                SessionIngress::ConfigurationStarted(identity(value)),
                GameTick::ZERO,
                &mut policy,
            )
            .unwrap();
        bridge
            .apply_ingress(
                session,
                SessionIngress::JoinRequested(JoinRequest {
                    identity: identity(value),
                    settings: settings(),
                    transferred: case & 1 != 0,
                }),
                GameTick::new(1),
                &mut policy,
            )
            .unwrap();
        assert_eq!(bridge.state(session), Some(SessionState::Play));
        bridge
            .apply_ingress(
                session,
                SessionIngress::Closed,
                GameTick::new(2),
                &mut policy,
            )
            .unwrap();
        assert!(bridge.is_empty());
        assert_eq!(bridge.router_mut().commands.len(), 2);
        let router = bridge.router_mut();
        assert_eq!(router.commands[0].target(), router.commands[1].target());
    }
}

fn run_fault_vectors() -> usize {
    let (mut connected, session, mut policy) = fresh_bridge();
    assert!(matches!(
        connected.apply_ingress(
            session,
            SessionIngress::JoinRequested(JoinRequest {
                identity: identity(1),
                settings: settings(),
                transferred: false,
            }),
            GameTick::new(1),
            &mut policy,
        ),
        Err(SessionBridgeError::MissingRoute(_))
    ));

    let (mut routed, session, mut policy) = routed_bridge();
    assert!(matches!(
        routed.apply_ingress(
            session,
            SessionIngress::ConfigurationStarted(identity(2)),
            GameTick::ZERO,
            &mut policy,
        ),
        Ok(None)
    ));
    assert!(matches!(
        routed.apply_ingress(
            session,
            SessionIngress::JoinRequested(JoinRequest {
                identity: identity(1),
                settings: settings(),
                transferred: false,
            }),
            GameTick::new(1),
            &mut policy,
        ),
        Err(SessionBridgeError::IdentityChanged)
    ));

    let (mut bridge, session, mut policy) = routed_bridge();
    bridge
        .login_admission(session, &profile(3), &mut policy)
        .unwrap();
    bridge
        .apply_ingress(
            session,
            SessionIngress::ConfigurationStarted(identity(3)),
            GameTick::ZERO,
            &mut policy,
        )
        .unwrap();
    bridge
        .apply_ingress(
            session,
            SessionIngress::JoinRequested(JoinRequest {
                identity: identity(3),
                settings: settings(),
                transferred: false,
            }),
            GameTick::new(1),
            &mut policy,
        )
        .unwrap();
    assert!(matches!(
        bridge.apply_ingress(
            session,
            SessionIngress::JoinRequested(JoinRequest {
                identity: identity(3),
                settings: settings(),
                transferred: false,
            }),
            GameTick::new(1),
            &mut policy,
        ),
        Err(SessionBridgeError::UnexpectedState { .. })
    ));
    bridge.router_mut().fail_next = true;
    assert!(matches!(
        bridge.apply_ingress(
            session,
            SessionIngress::Closed,
            GameTick::new(2),
            &mut policy,
        ),
        Err(SessionBridgeError::RegionRoute(
            RegionRouteError::Unavailable
        ))
    ));
    assert_eq!(bridge.state(session), Some(SessionState::Play));
    bridge
        .apply_ingress(
            session,
            SessionIngress::Closed,
            GameTick::new(2),
            &mut policy,
        )
        .unwrap();
    assert!(bridge.is_empty());
    5
}

fn fresh_bridge() -> (SessionBridge<CapturingRouter>, SessionId, CountingPolicy) {
    let lifecycle = Arc::new(NodeLifecycle::new(BTreeSet::new()));
    lifecycle.mark_membership_ready().unwrap();
    let route = InitialWorldRoute {
        world: region().world(),
        dimension: region().dimension().clone(),
        spawn_chunk: spawn_chunk(),
        mapping: ferrite_foundation::region::RegionMapping::V1,
    };
    let routes = VirtualHostRoutes::new(route, 1).unwrap();
    let mut bridge = SessionBridge::new(routes, lifecycle, 2, CapturingRouter::default()).unwrap();
    let session = crate::player_service::fixtures::session(1);
    bridge
        .register(
            session,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 25_565),
        )
        .unwrap();
    (bridge, session, CountingPolicy)
}

fn routed_bridge() -> (SessionBridge<CapturingRouter>, SessionId, CountingPolicy) {
    let (mut bridge, session, mut policy) = fresh_bridge();
    bridge
        .apply_ingress(
            session,
            SessionIngress::Routed(VirtualHost {
                host: "player-service.example".to_owned(),
                port: 25_565,
            }),
            GameTick::ZERO,
            &mut policy,
        )
        .unwrap();
    (bridge, session, policy)
}

fn profile(value: u128) -> GameProfile {
    GameProfile {
        id: value,
        name: identity(value).name,
        properties: Vec::new(),
    }
}

#[derive(Default)]
struct CapturingRouter {
    commands: Vec<RegionCommand>,
    fail_next: bool,
}

impl RegionCommandRouter for CapturingRouter {
    fn route(&mut self, command: RegionCommand) -> Result<(), RegionRouteError> {
        if self.fail_next {
            self.fail_next = false;
            Err(RegionRouteError::Unavailable)
        } else {
            self.commands.push(command);
            Ok(())
        }
    }
}

#[derive(Default)]
struct CountingPolicy;

impl AdmissionPolicy for CountingPolicy {
    fn deny_reason(&mut self, _context: &AdmissionContext<'_>) -> Option<String> {
        None
    }
}
