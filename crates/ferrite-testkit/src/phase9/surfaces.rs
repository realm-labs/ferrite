//! Executable Phase 9 root-surface closure.

use ferrite_protocol::java_26_2::catalog::{PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::clientbound::live_tags::gate::{
    LiveTagsContext, LiveTagsDecision, LiveTagsEffect, LiveTagsGates,
};
use ferrite_protocol::java_26_2::play::clientbound::live_tags::packet::LiveTagReloadStep;
use ferrite_protocol::java_26_2::play::serverbound::admin_state::gate::{
    AdminStateContext, AdminStateDecision, AdminStateEffect, AdminStateGates,
};
use ferrite_protocol::java_26_2::play::serverbound::admin_state::packet::AdminStateRequest;
use ferrite_protocol::java_26_2::play::serverbound::operator_blocks::gate::{
    OperatorBlockContext, OperatorBlockDecision, OperatorBlockEffect, OperatorBlockGates,
};
use ferrite_protocol::java_26_2::play::serverbound::operator_blocks::packet::OperatorBlockRequest;
use ferrite_protocol::java_26_2::play::serverbound::reconfiguration::transition::{
    ReplacementCommonListenerCookieField, ServerInboundReconfigurationStage,
    ServerInboundReconfigurationTransition,
};

use crate::phase9::effects::run_combat_rule_projection;
use crate::phase9::joins::run_all_phase9_joins;
use crate::phase9::menu::run_menu_convergence;
use crate::phase9::prediction::run_same_position_prediction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientProjectionSurfaceReport {
    pub clientbound_packets: usize,
    pub prediction_cases: usize,
    pub menu_cases: usize,
    pub lifecycle_cases: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandAdministrationSurfaceReport {
    pub admitted_admin_cases: usize,
    pub admitted_operator_cases: usize,
    pub permission_factors: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossSystemOrderingSurfaceReport {
    pub joins: usize,
    pub checkpoints: usize,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataReloadSurfaceReport {
    pub publication_steps: usize,
    pub failure_prefixes: usize,
    pub retained_cookie_fields: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkIngressSurfaceReport {
    pub serverbound_packets: usize,
    pub prediction_boundaries: usize,
    pub reconfiguration_stages: usize,
}

#[must_use]
pub fn run_client_projection_surface() -> ClientProjectionSurfaceReport {
    let prediction = run_same_position_prediction();
    assert_eq!(prediction.pending_after_old_ack, 1);
    assert_eq!(prediction.resolved_by_covering_ack, 1);
    let menu = run_menu_convergence();
    assert!(menu.stale_click_executed && menu.stale_full_resync);
    let lifecycle = run_combat_rule_projection();
    assert!(lifecycle.missing_local_ignored);
    ClientProjectionSurfaceReport {
        clientbound_packets: PacketCatalog::all()
            .iter()
            .filter(|packet| packet.direction() == PacketDirection::Clientbound)
            .count(),
        prediction_cases: 2,
        menu_cases: 3,
        lifecycle_cases: 3,
    }
}

#[must_use]
pub fn run_command_administration_surface() -> CommandAdministrationSurfaceReport {
    let admin = AdminStateGates {
        game_rules: true,
        ..AdminStateGates::default()
    }
    .decide(
        AdminStateRequest::SetGameRule,
        AdminStateContext {
            command_game_master: true,
            ..AdminStateContext::default()
        },
    );
    assert_eq!(
        admin,
        AdminStateDecision::Emit(AdminStateEffect::ApplyGameRulesSequentially)
    );
    let operator = OperatorBlockGates {
        operator_blocks: true,
    }
    .decide(
        OperatorBlockRequest::SetJigsawBlock {
            target_matches: true,
        },
        OperatorBlockContext {
            instabuild: true,
            command_game_master: true,
        },
    );
    assert_eq!(
        operator,
        OperatorBlockDecision::Emit(OperatorBlockEffect::SetJigsawFieldsThenMarkAndPublish)
    );
    CommandAdministrationSurfaceReport {
        admitted_admin_cases: 1,
        admitted_operator_cases: 1,
        permission_factors: 2,
    }
}

#[must_use]
pub fn run_cross_system_ordering_surface() -> CrossSystemOrderingSurfaceReport {
    let reports = run_all_phase9_joins();
    assert_eq!(reports.len(), 21);
    assert!(reports.iter().all(|report| report.rejected_faults == 0));
    assert!(
        reports
            .iter()
            .all(|report| !report.transient_state_persisted)
    );
    let mut hasher = blake3::Hasher::new();
    for report in &reports {
        hasher.update(report.digest.as_bytes());
    }
    CrossSystemOrderingSurfaceReport {
        joins: reports.len(),
        checkpoints: reports.iter().map(|report| report.checkpoints).sum(),
        digest: hasher.finalize().to_hex().to_string(),
    }
}

#[must_use]
pub fn run_data_reload_surface() -> DataReloadSurfaceReport {
    let gates = LiveTagsGates { live_reload: true };
    let context = LiveTagsContext {
        service_registered: true,
        reload_committed: true,
        remote_connection: true,
        all_registries_prepared: true,
    };
    assert_eq!(
        gates.decide(context),
        LiveTagsDecision::Emit(LiveTagsEffect::ReplaceBindingsThenRefreshFuelAndSearchTrees)
    );
    assert_eq!(
        gates.decide(LiveTagsContext {
            all_registries_prepared: false,
            ..context
        }),
        LiveTagsDecision::PreserveExistingBindings
    );
    DataReloadSurfaceReport {
        publication_steps: LiveTagReloadStep::ORDER.len(),
        failure_prefixes: 1,
        retained_cookie_fields: ReplacementCommonListenerCookieField::ALL.len(),
    }
}

#[must_use]
pub fn run_network_ingress_surface() -> NetworkIngressSurfaceReport {
    let prediction = run_same_position_prediction();
    assert_eq!(prediction.captured_authoritative_state, 13);
    let menu = run_menu_convergence();
    assert!(menu.wrong_prediction_ignored && menu.delayed_content_ignored);
    let mut transition = ServerInboundReconfigurationTransition::new();
    assert!(transition.handle_acknowledgement().is_err());
    transition.begin_waiting().unwrap();
    transition.handle_acknowledgement().unwrap();
    assert_eq!(
        transition.stage(),
        ServerInboundReconfigurationStage::Configuration
    );
    NetworkIngressSurfaceReport {
        serverbound_packets: PacketCatalog::all()
            .iter()
            .filter(|packet| packet.direction() == PacketDirection::Serverbound)
            .count(),
        prediction_boundaries: 2,
        reconfiguration_stages: 3,
    }
}
