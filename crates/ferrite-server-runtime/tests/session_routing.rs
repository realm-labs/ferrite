use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use ferrite_foundation::coordinate::ChunkPos;
use ferrite_foundation::identity::{ActivationGeneration, DimensionId, StableEntityId, WorldId};
use ferrite_foundation::region::{RegionCoord, RegionMapping, SimulationRegionKey};
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::state::{PlayerPose, Rotation, Vec3};
use ferrite_protocol::java_26_2::catalog::PROTOCOL_VERSION;
use ferrite_protocol::java_26_2::configuration::serverbound::packet::ClientInformation;
use ferrite_protocol::java_26_2::connection::output::{
    ConnectionCloseReason, PlayInstallationRequest, ServerConnectionEvent,
};
use ferrite_protocol::java_26_2::handshake::packet::ClientIntention;
use ferrite_protocol::java_26_2::handshake::transition::RoutingContext;
use ferrite_protocol::java_26_2::login::profile::GameProfile;
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, JoinRequest, MainHand, ParticleStatus, SessionDisconnectReason,
    SessionEgress, SessionId, SessionIdentity, SessionIngress, VirtualHost,
};
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerConfig};
use ferrite_region_runtime::logic::{
    ImmediateEffectContext, RegionLogic, RegionLogicError, RegionPhaseContext, RegionPhaseOutput,
};
use ferrite_server_runtime::lifecycle::NodeLifecycle;
use ferrite_server_runtime::session::admission::{AdmissionContext, AdmissionPolicy, AllowAll};
use ferrite_server_runtime::session::bridge::{SessionBridge, SessionBridgeError, SessionState};
use ferrite_server_runtime::session::command::{
    SessionCommandError, SessionJoinPayload, SessionLeavePayload,
};
use ferrite_server_runtime::session::normalize::{normalize_client_settings, normalize_java_event};
use ferrite_server_runtime::session::route::{
    InitialWorldRoute, RouteTableError, VirtualHostRoutes,
};
use ferrite_server_runtime::session::router::{RegionCommandRouter, RegionRouteError};
use ferrite_simulation::command::{CommandSource, RegionCommand};
use ferrite_simulation::region::RegionSimulationState;
use ferrite_simulation::tick::{GameTick, TickPhase};
use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::region::RegionVoxelState;

fn world(value: u128) -> WorldId {
    WorldId::new(value).unwrap()
}

fn dimension(name: &str) -> DimensionId {
    DimensionId::new(ResourceId::minecraft(name).unwrap())
}

fn route(world_id: u128, dimension_name: &str, spawn_chunk: ChunkPos) -> InitialWorldRoute {
    InitialWorldRoute {
        world: world(world_id),
        dimension: dimension(dimension_name),
        spawn: ferrite_foundation::coordinate::BlockPos::new(
            spawn_chunk.x * 16 + 8,
            64,
            spawn_chunk.z * 16 + 8,
        ),
        mapping: RegionMapping::V1,
    }
}

fn routes(initial: InitialWorldRoute) -> VirtualHostRoutes {
    VirtualHostRoutes::new(initial, 8).unwrap()
}

fn ready_lifecycle() -> Arc<NodeLifecycle> {
    let lifecycle = Arc::new(NodeLifecycle::new(BTreeSet::new()));
    lifecycle.mark_membership_ready().unwrap();
    lifecycle
}

fn session(value: u64) -> SessionId {
    SessionId::new(value).unwrap()
}

fn identity(value: u128, name: &str) -> SessionIdentity {
    SessionIdentity {
        profile_id: value,
        name: name.to_owned(),
    }
}

fn profile(value: u128, name: &str) -> GameProfile {
    GameProfile {
        id: value,
        name: name.to_owned(),
        properties: Vec::new(),
    }
}

fn settings() -> ClientSettings {
    ClientSettings {
        language: "zh_cn".to_owned(),
        view_distance: 12,
        chat_visibility: ChatVisibility::System,
        chat_colors: true,
        model_customization: 0x7f,
        main_hand: MainHand::Left,
        text_filtering: true,
        allows_listing: false,
        particle_status: ParticleStatus::Decreased,
    }
}

fn join_request(profile_id: u128, name: &str) -> JoinRequest {
    JoinRequest {
        identity: identity(profile_id, name),
        settings: settings(),
        transferred: false,
    }
}

fn peer(octet: u8) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, octet)), 25_565)
}

fn region_state(key: SimulationRegionKey) -> RegionSimulationState {
    RegionSimulationState::new(
        RegionVoxelState::new(
            key,
            RegionMapping::V1,
            ChunkLayout::new(
                VerticalSectionRange::new(-4, 24).unwrap(),
                BlockStateId::new(0),
                BiomeId::new(0),
            ),
        )
        .unwrap(),
    )
}

#[test]
fn join_payload_round_trips_and_builds_a_bounded_semantic_command() {
    let payload = SessionJoinPayload {
        session: session(7),
        player: StableEntityId::new(42).unwrap(),
        identity: identity(42, "Alex"),
        settings: settings(),
        transferred: true,
        spawn_pose: PlayerPose::new(Vec3::new(8.5, 65.0, 8.5), Rotation::default()),
    };
    let bytes = payload.encode().unwrap();
    assert_eq!(SessionJoinPayload::decode(&bytes).unwrap(), payload);

    let target = route(1, "overworld", ChunkPos::new(-9, 17)).region();
    let command = payload
        .clone()
        .into_region_command(target.clone(), GameTick::new(3), 9)
        .unwrap();
    assert_eq!(command.target(), &target);
    assert_eq!(command.tick(), GameTick::new(3));
    assert_eq!(
        command.source(),
        &CommandSource::Player(StableEntityId::new(42).unwrap())
    );
    assert_eq!(command.sequence(), 9);
    assert_eq!(
        command.kind(),
        &ResourceId::new("ferrite", "session/join").unwrap()
    );
    assert_eq!(
        SessionJoinPayload::decode(command.payload()).unwrap(),
        payload
    );

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(matches!(
        SessionJoinPayload::decode(&trailing),
        Err(SessionCommandError::TrailingBytes)
    ));
    let mut invalid_boolean = bytes;
    invalid_boolean[44] = 2;
    assert!(matches!(
        SessionJoinPayload::decode(&invalid_boolean),
        Err(SessionCommandError::InvalidBoolean { value: 2 })
    ));

    let mut oversized = payload;
    oversized.identity.name = "a".repeat(usize::from(u16::MAX) + 1);
    assert!(matches!(
        oversized.encode(),
        Err(SessionCommandError::StringTooLong {
            field: "profile name",
            ..
        })
    ));

    let leave = SessionLeavePayload {
        session: session(7),
        player: StableEntityId::new(42).unwrap(),
    };
    let leave_bytes = leave.encode();
    assert_eq!(SessionLeavePayload::decode(&leave_bytes).unwrap(), leave);
    let leave_command = leave
        .into_region_command(target, GameTick::new(4), 10)
        .unwrap();
    assert_eq!(
        leave_command.kind(),
        &ResourceId::new("ferrite", "session/leave").unwrap()
    );
    let mut trailing_leave = leave_bytes;
    trailing_leave.push(0);
    assert!(matches!(
        SessionLeavePayload::decode(&trailing_leave),
        Err(SessionCommandError::TrailingBytes)
    ));
}

#[test]
fn virtual_hosts_select_world_dimension_and_negative_region_deterministically() {
    let fallback = route(1, "overworld", ChunkPos::new(0, 0));
    let selected = route(2, "the_nether", ChunkPos::new(-9, 17));
    let mut table = VirtualHostRoutes::new(fallback.clone(), 1).unwrap();
    table
        .insert("shard.example".to_owned(), 25_565, selected.clone())
        .unwrap();

    let resolved = table.resolve(&VirtualHost {
        host: "shard.example".to_owned(),
        port: 25_565,
    });
    assert_eq!(resolved, &selected);
    assert_eq!(resolved.region().coordinate(), RegionCoord::new(-2, 2));
    assert_eq!(
        table.resolve(&VirtualHost {
            host: "shard.example".to_owned(),
            port: 25_566,
        }),
        &fallback
    );
    assert!(matches!(
        table.insert("shard.example".to_owned(), 25_565, fallback.clone()),
        Err(RouteTableError::Duplicate { .. })
    ));
    assert!(matches!(
        table.insert("other.example".to_owned(), 25_565, fallback),
        Err(RouteTableError::Full { capacity: 1 })
    ));
    assert!(matches!(
        VirtualHostRoutes::new(selected, 0),
        Err(RouteTableError::ZeroCapacity)
    ));
}

#[derive(Default)]
struct JoinLogic {
    joins: Vec<SessionJoinPayload>,
}

impl RegionLogic for JoinLogic {
    fn execute_phase(
        &mut self,
        context: RegionPhaseContext<'_>,
        _output: &mut RegionPhaseOutput,
    ) -> Result<(), RegionLogicError> {
        if context.phase() == TickPhase::Ingress {
            self.joins.extend(
                context
                    .commands()
                    .iter()
                    .map(|command| SessionJoinPayload::decode(command.payload()).unwrap()),
            );
        }
        Ok(())
    }

    fn apply_immediate_effect(
        &mut self,
        _context: ImmediateEffectContext<'_>,
    ) -> Result<(), RegionLogicError> {
        Ok(())
    }
}

#[derive(Default)]
struct CountingPolicy {
    calls: usize,
    denial: Option<String>,
}

impl AdmissionPolicy for CountingPolicy {
    fn deny_reason(&mut self, _context: &AdmissionContext<'_>) -> Option<String> {
        self.calls += 1;
        self.denial.clone()
    }
}

#[test]
fn java_session_events_reach_the_selected_local_region_without_packet_types() {
    let exact_route = route(8, "the_end", ChunkPos::new(-1, 8));
    let target = exact_route.region();
    let mut route_table = routes(route(1, "overworld", ChunkPos::new(0, 0)));
    route_table
        .insert("play.example".to_owned(), 25_565, exact_route)
        .unwrap();

    let mut runner = LocalRegionRunner::new(LocalRunnerConfig::testing()).unwrap();
    runner
        .insert_region(
            region_state(target.clone()),
            ActivationGeneration::INITIAL,
            GameTick::ZERO,
        )
        .unwrap();
    let lifecycle = ready_lifecycle();
    let mut bridge = SessionBridge::new(route_table, lifecycle.clone(), 4, runner).unwrap();
    let session = session(1);
    bridge.register(session, peer(1)).unwrap();

    let mut policy = CountingPolicy::default();
    bridge
        .apply_java_event(
            session,
            ServerConnectionEvent::Routed(RoutingContext {
                host: "play.example".to_owned(),
                port: 25_565,
                protocol_version: PROTOCOL_VERSION as i32,
                intention: ClientIntention::Login,
            }),
            GameTick::ZERO,
            &mut policy,
        )
        .unwrap();
    assert_eq!(bridge.state(session), Some(SessionState::Routed));
    assert!(
        !bridge
            .login_admission(session, &profile(99, "Steve"), &mut policy)
            .unwrap()
            .duplicate_active
    );
    bridge
        .apply_java_event(
            session,
            ServerConnectionEvent::ConfigurationStarted {
                profile: profile(99, "Steve"),
            },
            GameTick::ZERO,
            &mut policy,
        )
        .unwrap();
    let egress = bridge
        .apply_java_event(
            session,
            ServerConnectionEvent::PlayInstallationRequested(PlayInstallationRequest {
                profile: profile(99, "Steve"),
                client_information: ClientInformation::default(),
                transferred: false,
            }),
            GameTick::new(1),
            &mut policy,
        )
        .unwrap()
        .unwrap();
    let SessionEgress::CompletePlayInstallation(admission) = egress else {
        panic!("the admitted session must install play");
    };
    assert_eq!(admission.region, target);
    assert_eq!(bridge.state(session), Some(SessionState::Play));
    assert_eq!(policy.calls, 2);

    let mut logic = JoinLogic::default();
    bridge
        .router_mut()
        .run_tick(GameTick::new(1), &mut logic)
        .unwrap();
    assert_eq!(logic.joins.len(), 1);
    assert_eq!(logic.joins[0].identity, identity(99, "Steve"));
    assert_eq!(logic.joins[0].settings.language, "en_us");

    bridge
        .apply_java_event(
            session,
            ServerConnectionEvent::LatencyUpdated { latency_millis: 37 },
            GameTick::new(1),
            &mut policy,
        )
        .unwrap();
    assert_eq!(bridge.latency_millis(session), Some(37));
    bridge
        .apply_java_event(
            session,
            ServerConnectionEvent::Closed(ConnectionCloseReason::StatusRequestHandled),
            GameTick::new(2),
            &mut policy,
        )
        .unwrap();
    assert!(bridge.is_empty());
    assert_eq!(lifecycle.snapshot().unwrap().active_sessions, 0);
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

fn configured_bridge(
    router: CapturingRouter,
    profile_id: u128,
) -> (SessionBridge<CapturingRouter>, SessionId, CountingPolicy) {
    let mut bridge = SessionBridge::new(
        routes(route(1, "overworld", ChunkPos::new(0, 0))),
        ready_lifecycle(),
        4,
        router,
    )
    .unwrap();
    let session = session(1);
    let mut policy = CountingPolicy::default();
    bridge.register(session, peer(1)).unwrap();
    bridge
        .apply_ingress(
            session,
            SessionIngress::Routed(VirtualHost {
                host: "fallback.example".to_owned(),
                port: 25_565,
            }),
            GameTick::ZERO,
            &mut policy,
        )
        .unwrap();
    bridge
        .login_admission(session, &profile(profile_id, "Alex"), &mut policy)
        .unwrap();
    bridge
        .apply_ingress(
            session,
            SessionIngress::ConfigurationStarted(identity(profile_id, "Alex")),
            GameTick::ZERO,
            &mut policy,
        )
        .unwrap();
    (bridge, session, policy)
}

#[test]
fn failed_or_denied_region_admission_does_not_advance_session_state() {
    let router = CapturingRouter {
        fail_next: true,
        ..CapturingRouter::default()
    };
    let (mut bridge, session, mut policy) = configured_bridge(router, 11);
    let request = join_request(11, "Alex");
    assert!(matches!(
        bridge.apply_ingress(
            session,
            SessionIngress::JoinRequested(request.clone()),
            GameTick::new(1),
            &mut policy,
        ),
        Err(SessionBridgeError::RegionRoute(
            RegionRouteError::Unavailable
        ))
    ));
    assert_eq!(bridge.state(session), Some(SessionState::Configuration));
    assert!(bridge.router_mut().commands.is_empty());

    let admitted = bridge
        .apply_ingress(
            session,
            SessionIngress::JoinRequested(request),
            GameTick::new(1),
            &mut policy,
        )
        .unwrap()
        .unwrap();
    assert!(matches!(
        admitted,
        SessionEgress::CompletePlayInstallation(_)
    ));
    assert_eq!(bridge.router_mut().commands.len(), 1);

    let (mut denied, session, mut policy) = configured_bridge(CapturingRouter::default(), 12);
    policy.denial = Some("maintenance".to_owned());
    let egress = denied
        .apply_ingress(
            session,
            SessionIngress::JoinRequested(join_request(12, "Alex")),
            GameTick::new(1),
            &mut policy,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        egress,
        SessionEgress::Disconnect {
            session,
            reason: SessionDisconnectReason::AdmissionDenied("maintenance".to_owned()),
        }
    );
    assert_eq!(denied.state(session), Some(SessionState::Configuration));
    assert!(denied.router_mut().commands.is_empty());
}

#[test]
fn duplicate_profile_resolution_targets_the_existing_play_session() {
    let (mut bridge, first, mut policy) = configured_bridge(CapturingRouter::default(), 21);
    bridge
        .apply_ingress(
            first,
            SessionIngress::JoinRequested(join_request(21, "Alex")),
            GameTick::new(1),
            &mut policy,
        )
        .unwrap();

    let second = session(2);
    bridge.register(second, peer(2)).unwrap();
    bridge
        .apply_ingress(
            second,
            SessionIngress::Routed(VirtualHost {
                host: "fallback.example".to_owned(),
                port: 25_565,
            }),
            GameTick::ZERO,
            &mut policy,
        )
        .unwrap();
    assert!(
        bridge
            .login_admission(second, &profile(21, "Alex"), &mut policy)
            .unwrap()
            .duplicate_active
    );
    let egress = bridge
        .apply_java_event(
            second,
            ServerConnectionEvent::DisconnectExisting { profile_id: 21 },
            GameTick::ZERO,
            &mut policy,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        egress,
        SessionEgress::Disconnect {
            session: first,
            reason: SessionDisconnectReason::DuplicateLogin,
        }
    );
}

#[test]
fn java_client_settings_are_normalized_and_registry_selection_stays_local() {
    let information = ClientInformation {
        language: "de_de".to_owned(),
        view_distance: -1,
        allows_listing: true,
        ..ClientInformation::default()
    };
    let normalized = normalize_client_settings(information);
    assert_eq!(normalized.language, "de_de");
    assert_eq!(normalized.view_distance, -1);
    assert!(normalized.allows_listing);

    assert!(
        normalize_java_event(ServerConnectionEvent::RegistrySelection {
            selected_packs: Vec::new(),
            exact_offer_match: true,
        })
        .is_none()
    );
    assert!(SessionId::new(0).is_err());

    let mut policy = AllowAll;
    let context_route = route(1, "overworld", ChunkPos::new(0, 0));
    assert_eq!(
        policy.deny_reason(&AdmissionContext {
            session: session(1),
            peer: peer(1),
            identity: &identity(1, "Alex"),
            destination: &context_route,
        }),
        None
    );
}
