//! Join snapshots and live player-facing game-rule projection.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerRuleValues {
    pub immediate_respawn: bool,
    pub locator_bar: bool,
    pub reduced_debug_info: bool,
}

impl Default for PlayerRuleValues {
    fn default() -> Self {
        Self {
            immediate_respawn: false,
            locator_bar: true,
            reduced_debug_info: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoinRuleProjection {
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
}

#[must_use]
pub const fn join_projection(rules: PlayerRuleValues) -> JoinRuleProjection {
    JoinRuleProjection {
        reduced_debug_info: rules.reduced_debug_info,
        show_death_screen: !rules.immediate_respawn,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerRule {
    ImmediateRespawn,
    ReducedDebugInfo,
    LocatorBar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    Player,
    Misc,
}

impl PlayerRule {
    #[must_use]
    pub const fn category(self) -> RuleCategory {
        match self {
            Self::ImmediateRespawn | Self::LocatorBar => RuleCategory::Player,
            Self::ReducedDebugInfo => RuleCategory::Misc,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RulePlayer {
    pub id: u64,
    pub level: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuleProjectionStep {
    Notify(PlayerRule),
    ImmediateRespawn { player: u64, value: f32 },
    ReducedDebugInfo { player: u64, event: i8 },
    LocatorPlayerAdded { level: u64, player: u64 },
    LocatorConnectionsBroken { level: u64, count: usize },
}

pub fn project_rule_change(
    rule: PlayerRule,
    enabled: bool,
    players: &[RulePlayer],
    waypoint_managers: &mut [WaypointManager],
) -> Vec<RuleProjectionStep> {
    let mut steps = vec![RuleProjectionStep::Notify(rule)];
    match rule {
        PlayerRule::ImmediateRespawn => {
            let value = if enabled { 1.0 } else { 0.0 };
            steps.extend(
                players
                    .iter()
                    .map(|player| RuleProjectionStep::ImmediateRespawn {
                        player: player.id,
                        value,
                    }),
            );
        }
        PlayerRule::ReducedDebugInfo => {
            let event = if enabled { 22 } else { 23 };
            steps.extend(
                players
                    .iter()
                    .map(|player| RuleProjectionStep::ReducedDebugInfo {
                        player: player.id,
                        event,
                    }),
            );
        }
        PlayerRule::LocatorBar => {
            for manager in waypoint_managers {
                manager.enabled = enabled;
                if enabled {
                    let level = manager.level;
                    for player in players.iter().filter(|player| player.level == level) {
                        let _ = manager.add_player(player.id);
                        steps.push(RuleProjectionStep::LocatorPlayerAdded {
                            level,
                            player: player.id,
                        });
                    }
                } else {
                    let count = manager.break_all_connections().len();
                    steps.push(RuleProjectionStep::LocatorConnectionsBroken {
                        level: manager.level,
                        count,
                    });
                }
            }
        }
    }
    steps
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatKillAction {
    Ignored,
    DeathScreen { hardcore: bool },
    PerformRespawnAndResetToggleKeys,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRuleProjection {
    pub local_player_id: i32,
    pub local_player_present: bool,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub hardcore: bool,
    pub death_screens: u32,
    pub respawn_requests: u32,
    pub toggle_key_resets: u32,
}

impl ClientRuleProjection {
    #[must_use]
    pub fn from_join(local_player_id: i32, hardcore: bool, join: JoinRuleProjection) -> Self {
        Self {
            local_player_id,
            local_player_present: true,
            reduced_debug_info: join.reduced_debug_info,
            show_death_screen: join.show_death_screen,
            hardcore,
            death_screens: 0,
            respawn_requests: 0,
            toggle_key_resets: 0,
        }
    }

    #[must_use]
    pub fn respawn_replacement(&self) -> Self {
        Self {
            death_screens: 0,
            respawn_requests: 0,
            toggle_key_resets: 0,
            ..self.clone()
        }
    }

    pub fn immediate_respawn_event(&mut self, value: f32) {
        self.show_death_screen = value == 0.0;
    }

    pub fn reduced_debug_entity_event(&mut self, entity_id: i32, event: i8) {
        if entity_id != self.local_player_id || !self.local_player_present {
            return;
        }
        match event {
            22 => self.reduced_debug_info = true,
            23 => self.reduced_debug_info = false,
            _ => {}
        }
    }

    pub fn combat_kill(&mut self, player_entity_id: i32) -> CombatKillAction {
        if player_entity_id != self.local_player_id || !self.local_player_present {
            return CombatKillAction::Ignored;
        }
        if self.show_death_screen {
            self.death_screens = self.death_screens.saturating_add(1);
            CombatKillAction::DeathScreen {
                hardcore: self.hardcore,
            }
        } else {
            self.respawn_requests = self.respawn_requests.saturating_add(1);
            self.toggle_key_resets = self.toggle_key_resets.saturating_add(1);
            CombatKillAction::PerformRespawnAndResetToggleKeys
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WaypointConnection {
    pub player: u64,
    pub transmitter: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionUpdate {
    Connected { representation: u64 },
    Retained,
    Disconnected,
    Ineligible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointManager {
    pub level: u64,
    pub enabled: bool,
    players: BTreeSet<u64>,
    transmitters: BTreeMap<u64, Option<u64>>,
    connections: BTreeMap<WaypointConnection, u64>,
}

impl WaypointManager {
    #[must_use]
    pub fn new(level: u64, enabled: bool) -> Self {
        Self {
            level,
            enabled,
            players: BTreeSet::new(),
            transmitters: BTreeMap::new(),
            connections: BTreeMap::new(),
        }
    }

    pub fn track_transmitter(&mut self, transmitter: u64, representation: Option<u64>) {
        self.transmitters.insert(transmitter, representation);
    }

    pub fn add_player(&mut self, player: u64) -> Vec<ConnectionUpdate> {
        self.players.insert(player);
        let transmitters = self.transmitters.keys().copied().collect::<Vec<_>>();
        transmitters
            .into_iter()
            .map(|transmitter| self.rebuild_connection(player, transmitter))
            .collect()
    }

    pub fn add_player_transmitter(
        &mut self,
        player: u64,
        representation: Option<u64>,
    ) -> Vec<ConnectionUpdate> {
        self.track_transmitter(player, representation);
        self.add_player(player)
    }

    pub fn update_connection(
        &mut self,
        player: u64,
        transmitter: u64,
        broken: bool,
    ) -> ConnectionUpdate {
        let key = WaypointConnection {
            player,
            transmitter,
        };
        if self.connections.contains_key(&key) && !broken {
            return ConnectionUpdate::Retained;
        }
        self.rebuild_connection(player, transmitter)
    }

    pub fn set_representation(&mut self, transmitter: u64, representation: Option<u64>) {
        self.transmitters.insert(transmitter, representation);
    }

    pub fn remove_player(&mut self, player: u64) -> Vec<WaypointConnection> {
        self.players.remove(&player);
        self.transmitters.remove(&player);
        let removed = self
            .connections
            .keys()
            .filter(|connection| connection.player == player || connection.transmitter == player)
            .copied()
            .collect::<Vec<_>>();
        for connection in &removed {
            self.connections.remove(connection);
        }
        removed
    }

    pub fn break_all_connections(&mut self) -> Vec<WaypointConnection> {
        let disconnected = self.connections.keys().copied().collect();
        self.connections.clear();
        disconnected
    }

    #[must_use]
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    fn rebuild_connection(&mut self, player: u64, transmitter: u64) -> ConnectionUpdate {
        let key = WaypointConnection {
            player,
            transmitter,
        };
        if player == transmitter || !self.enabled || !self.players.contains(&player) {
            self.connections.remove(&key);
            return ConnectionUpdate::Ineligible;
        }
        let representation = self.transmitters.get(&transmitter).copied().flatten();
        if let Some(representation) = representation {
            self.connections.insert(key, representation);
            ConnectionUpdate::Connected { representation }
        } else if self.connections.remove(&key).is_some() {
            ConnectionUpdate::Disconnected
        } else {
            ConnectionUpdate::Ineligible
        }
    }
}
