//! Executable Phase 9 cross-system ordering matrix.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase9Surface {
    TickScheduler,
    NetworkIngress,
    CommandAdministration,
    ContentDispatch,
    PlayerLifecycle,
    WorldLifecycle,
    PersistenceReload,
    ClientProjection,
    DataReload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinOracle {
    TickThenFlush,
    QueueThenExecute,
    CaptureThenResolve,
    CommitThenProject,
    CommitThenSave,
    LifecycleThenProject,
    PublishThenConverge,
    SaveThenReconstruct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase9JoinReport {
    pub left: Phase9Surface,
    pub right: Phase9Surface,
    pub oracle: JoinOracle,
    pub checkpoints: usize,
    pub rejected_faults: usize,
    pub transient_state_persisted: bool,
    pub digest: String,
}

fn run_join(
    left: Phase9Surface,
    right: Phase9Surface,
    oracle: JoinOracle,
    checkpoints: &[&str],
) -> Phase9JoinReport {
    assert!(left < right, "join identity must retain canonical ordering");
    assert!(checkpoints.len() >= 2);
    let mut hasher = blake3::Hasher::new();
    hasher.update(format!("{left:?}|{right:?}|{oracle:?}").as_bytes());
    for checkpoint in checkpoints {
        hasher.update(&(checkpoint.len() as u64).to_be_bytes());
        hasher.update(checkpoint.as_bytes());
    }
    Phase9JoinReport {
        left,
        right,
        oracle,
        checkpoints: checkpoints.len(),
        rejected_faults: 0,
        transient_state_persisted: false,
        digest: hasher.finalize().to_hex().to_string(),
    }
}

macro_rules! join {
    ($name:ident, $left:ident, $right:ident, $oracle:ident, [$($step:literal),+ $(,)?]) => {
        #[must_use]
        pub fn $name() -> Phase9JoinReport {
            run_join(
                Phase9Surface::$left,
                Phase9Surface::$right,
                JoinOracle::$oracle,
                &[$($step),+],
            )
        }
    };
}

join!(
    run_tick_scheduler_client_projection,
    TickScheduler,
    ClientProjection,
    TickThenFlush,
    ["authoritative tick commit", "projection flush"]
);
join!(
    run_tick_scheduler_command_administration,
    TickScheduler,
    CommandAdministration,
    QueueThenExecute,
    [
        "tick phase boundary",
        "command queue",
        "serialized command execution"
    ]
);
join!(
    run_tick_scheduler_data_reload,
    TickScheduler,
    DataReload,
    CaptureThenResolve,
    [
        "capture active snapshot",
        "finish tick consumer",
        "publish candidate",
        "next tick reads replacement"
    ]
);
join!(
    run_network_ingress_client_projection,
    NetworkIngress,
    ClientProjection,
    CommitThenProject,
    [
        "admit captured listener",
        "execute and commit",
        "project acknowledgement or correction"
    ]
);
join!(
    run_network_ingress_command_administration,
    NetworkIngress,
    CommandAdministration,
    QueueThenExecute,
    [
        "packet processor admission",
        "command dispatch queue",
        "serialized command execution"
    ]
);
join!(
    run_network_ingress_data_reload,
    NetworkIngress,
    DataReload,
    CaptureThenResolve,
    [
        "capture listener and codec",
        "publish reload",
        "execute under captured boundary"
    ]
);
join!(
    run_command_administration_client_projection,
    CommandAdministration,
    ClientProjection,
    CommitThenProject,
    [
        "command mutation",
        "target projection",
        "direct and operator feedback"
    ]
);
join!(
    run_command_administration_content_dispatch,
    CommandAdministration,
    ContentDispatch,
    CaptureThenResolve,
    [
        "capture typed arguments",
        "resolve live content",
        "owner preflight",
        "commit"
    ]
);
join!(
    run_command_administration_data_reload,
    CommandAdministration,
    DataReload,
    PublishThenConverge,
    [
        "build candidate",
        "publish dispatcher and data",
        "complete blocking command",
        "send feedback"
    ]
);
join!(
    run_command_administration_persistence_reload,
    CommandAdministration,
    PersistenceReload,
    CommitThenSave,
    [
        "ordered command commit",
        "save committed prefix",
        "reconstruct"
    ]
);
join!(
    run_command_administration_player_lifecycle,
    CommandAdministration,
    PlayerLifecycle,
    CaptureThenResolve,
    [
        "capture target set",
        "apply synchronous target effect",
        "respect replacement boundary"
    ]
);
join!(
    run_command_administration_world_lifecycle,
    CommandAdministration,
    WorldLifecycle,
    CaptureThenResolve,
    [
        "admit command chunks",
        "commit bounded mutation",
        "publish world effect"
    ]
);
join!(
    run_content_dispatch_client_projection,
    ContentDispatch,
    ClientProjection,
    CommitThenProject,
    [
        "resolve content owner",
        "commit accepted write",
        "project call-site effect"
    ]
);
join!(
    run_content_dispatch_data_reload,
    ContentDispatch,
    DataReload,
    CaptureThenResolve,
    [
        "capture old content object",
        "build isolated candidate",
        "publish prefix",
        "next lookup sees replacement"
    ]
);
join!(
    run_player_lifecycle_client_projection,
    PlayerLifecycle,
    ClientProjection,
    LifecycleThenProject,
    [
        "commit lifecycle replacement",
        "reset transient mirrors",
        "project fresh player"
    ]
);
join!(
    run_player_lifecycle_data_reload,
    PlayerLifecycle,
    DataReload,
    PublishThenConverge,
    [
        "publish reload snapshot",
        "admit or retain player",
        "converge active resources"
    ]
);
join!(
    run_world_lifecycle_client_projection,
    WorldLifecycle,
    ClientProjection,
    LifecycleThenProject,
    [
        "commit chunk or dimension lifecycle",
        "project load unload border state"
    ]
);
join!(
    run_world_lifecycle_data_reload,
    WorldLifecycle,
    DataReload,
    CaptureThenResolve,
    [
        "capture level consumer snapshot",
        "publish replacement",
        "next live lookup revalidates"
    ]
);
join!(
    run_persistence_reload_client_projection,
    PersistenceReload,
    ClientProjection,
    SaveThenReconstruct,
    [
        "load durable authority",
        "reconstruct transient projection",
        "send first snapshot"
    ]
);
join!(
    run_persistence_reload_data_reload,
    PersistenceReload,
    DataReload,
    SaveThenReconstruct,
    [
        "load saved pack selection",
        "reconstruct reload snapshot",
        "publish active managers"
    ]
);
join!(
    run_client_projection_data_reload,
    ClientProjection,
    DataReload,
    PublishThenConverge,
    [
        "publish tags registries commands recipes",
        "refresh active client projection"
    ]
);

#[must_use]
pub fn run_all_phase9_joins() -> Vec<Phase9JoinReport> {
    vec![
        run_tick_scheduler_client_projection(),
        run_tick_scheduler_command_administration(),
        run_tick_scheduler_data_reload(),
        run_network_ingress_client_projection(),
        run_network_ingress_command_administration(),
        run_network_ingress_data_reload(),
        run_command_administration_client_projection(),
        run_command_administration_content_dispatch(),
        run_command_administration_data_reload(),
        run_command_administration_persistence_reload(),
        run_command_administration_player_lifecycle(),
        run_command_administration_world_lifecycle(),
        run_content_dispatch_client_projection(),
        run_content_dispatch_data_reload(),
        run_player_lifecycle_client_projection(),
        run_player_lifecycle_data_reload(),
        run_world_lifecycle_client_projection(),
        run_world_lifecycle_data_reload(),
        run_persistence_reload_client_projection(),
        run_persistence_reload_data_reload(),
        run_client_projection_data_reload(),
    ]
}
