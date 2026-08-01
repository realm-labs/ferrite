use std::collections::{BTreeMap, BTreeSet};

use crate::java_26_2::play::clientbound::scoreboard::packet::{
    DisplaySlot, ObjectiveParameters, ResetScore, ScoreboardPacket, SetDisplayObjective,
    SetObjective, SetPlayerTeam, SetScore, TeamParameters,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoritativeObjective {
    pub parameters: ObjectiveParameters,
    pub scores: BTreeMap<String, SetScore>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoritativeTeam {
    pub parameters: TeamParameters,
    pub members: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardDelivery {
    pub recipient: u128,
    pub packet: ScoreboardPacket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamPublication {
    pub deliveries: Vec<ScoreboardDelivery>,
    pub waypoint_remakes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerScoreboardPublisher {
    players: Vec<u128>,
    objectives: BTreeMap<String, AuthoritativeObjective>,
    display_slots: BTreeMap<DisplaySlot, String>,
    tracked_objectives: BTreeSet<String>,
    teams: BTreeMap<String, AuthoritativeTeam>,
}

impl ServerScoreboardPublisher {
    #[must_use]
    pub fn new(players: Vec<u128>) -> Self {
        Self {
            players,
            ..Self::default()
        }
    }

    pub fn define_objective(&mut self, name: String, parameters: ObjectiveParameters) {
        self.objectives.insert(
            name,
            AuthoritativeObjective {
                parameters,
                scores: BTreeMap::new(),
            },
        );
    }

    pub fn define_team(&mut self, name: String, team: AuthoritativeTeam) {
        self.teams.insert(name, team);
    }

    #[must_use]
    pub fn is_tracked(&self, objective: &str) -> bool {
        self.tracked_objectives.contains(objective)
    }

    pub fn set_score(&mut self, score: SetScore) -> Vec<ScoreboardDelivery> {
        let objective_name = score.objective_name.clone();
        let Some(objective) = self.objectives.get_mut(&objective_name) else {
            return Vec::new();
        };
        objective.scores.insert(score.owner.clone(), score.clone());
        if self.tracked_objectives.contains(&objective_name) {
            self.broadcast([ScoreboardPacket::SetScore(score)])
        } else {
            Vec::new()
        }
    }

    pub fn remove_score(&mut self, owner: &str, objective: &str) -> Vec<ScoreboardDelivery> {
        let Some(objective_state) = self.objectives.get_mut(objective) else {
            return Vec::new();
        };
        if objective_state.scores.remove(owner).is_none()
            || !self.tracked_objectives.contains(objective)
        {
            return Vec::new();
        }
        self.broadcast([ScoreboardPacket::ResetScore(ResetScore {
            owner: owner.to_owned(),
            objective_name: Some(objective.to_owned()),
        })])
    }

    pub fn remove_all_scores(&mut self, owner: &str) -> Vec<ScoreboardDelivery> {
        for objective in self.objectives.values_mut() {
            objective.scores.remove(owner);
        }
        self.broadcast([ScoreboardPacket::ResetScore(ResetScore {
            owner: owner.to_owned(),
            objective_name: None,
        })])
    }

    pub fn change_objective(
        &mut self,
        name: &str,
        parameters: ObjectiveParameters,
    ) -> Vec<ScoreboardDelivery> {
        let Some(objective) = self.objectives.get_mut(name) else {
            return Vec::new();
        };
        if objective.parameters == parameters {
            return Vec::new();
        }
        objective.parameters = parameters.clone();
        if self.tracked_objectives.contains(name) {
            self.broadcast([ScoreboardPacket::SetObjective(SetObjective {
                objective_name: name.to_owned(),
                method: 2,
                parameters: Some(parameters),
            })])
        } else {
            Vec::new()
        }
    }

    pub fn set_display(
        &mut self,
        slot: DisplaySlot,
        objective: Option<String>,
    ) -> Vec<ScoreboardDelivery> {
        let previous = self.display_slots.get(&slot).cloned();
        if previous == objective {
            return Vec::new();
        }
        match &objective {
            Some(objective) => {
                self.display_slots.insert(slot, objective.clone());
            }
            None => {
                self.display_slots.remove(&slot);
            }
        }

        let mut packets = Vec::new();
        if let Some(previous) = previous
            && !self.display_slots.values().any(|value| value == &previous)
            && self.tracked_objectives.remove(&previous)
        {
            packets.push(ScoreboardPacket::SetObjective(SetObjective {
                objective_name: previous,
                method: 1,
                parameters: None,
            }));
        }
        if let Some(objective) = objective {
            if self.tracked_objectives.contains(&objective) {
                packets.push(ScoreboardPacket::SetDisplayObjective(SetDisplayObjective {
                    slot,
                    objective_name: Some(objective),
                }));
            } else {
                packets.extend(self.start_tracking_packets(&objective));
                self.tracked_objectives.insert(objective);
            }
        } else if let Some(previous) = self.display_slots.get(&slot) {
            packets.push(ScoreboardPacket::SetDisplayObjective(SetDisplayObjective {
                slot,
                objective_name: Some(previous.clone()),
            }));
        } else if packets.is_empty() {
            packets.push(ScoreboardPacket::SetDisplayObjective(SetDisplayObjective {
                slot,
                objective_name: None,
            }));
        }
        self.broadcast(packets)
    }

    pub fn publish_team_add(&self, team_name: &str) -> TeamPublication {
        self.publish_team_snapshot(team_name, 0)
    }

    pub fn publish_team_change(&self, team_name: &str) -> TeamPublication {
        self.publish_team_snapshot(team_name, 2)
    }

    pub fn publish_team_remove(&self, team_name: &str) -> TeamPublication {
        let members = self
            .teams
            .get(team_name)
            .map(|team| team.members.iter().cloned().collect())
            .unwrap_or_default();
        TeamPublication {
            deliveries: self.broadcast([ScoreboardPacket::SetPlayerTeam(SetPlayerTeam {
                team_name: team_name.to_owned(),
                method: 1,
                parameters: None,
                players: Vec::new(),
            })]),
            waypoint_remakes: members,
        }
    }

    pub fn publish_member_change(
        &self,
        team_name: &str,
        member: String,
        added: bool,
    ) -> TeamPublication {
        TeamPublication {
            deliveries: self.broadcast([ScoreboardPacket::SetPlayerTeam(SetPlayerTeam {
                team_name: team_name.to_owned(),
                method: if added { 3 } else { 4 },
                parameters: None,
                players: vec![member.clone()],
            })]),
            waypoint_remakes: vec![member],
        }
    }

    #[must_use]
    pub fn joining_packets(&self) -> Vec<ScoreboardPacket> {
        let mut packets = self
            .teams
            .iter()
            .map(|(name, team)| team_packet(name, team, 0))
            .collect::<Vec<_>>();
        let mut visited = BTreeSet::new();
        for objective in self.display_slots.values() {
            if visited.insert(objective.clone()) {
                packets.extend(self.start_tracking_packets(objective));
            }
        }
        packets
    }

    fn publish_team_snapshot(&self, team_name: &str, method: i8) -> TeamPublication {
        let Some(team) = self.teams.get(team_name) else {
            return TeamPublication {
                deliveries: Vec::new(),
                waypoint_remakes: Vec::new(),
            };
        };
        TeamPublication {
            deliveries: self.broadcast([team_packet(team_name, team, method)]),
            waypoint_remakes: team.members.iter().cloned().collect(),
        }
    }

    fn start_tracking_packets(&self, objective_name: &str) -> Vec<ScoreboardPacket> {
        let Some(objective) = self.objectives.get(objective_name) else {
            return Vec::new();
        };
        let mut packets = vec![ScoreboardPacket::SetObjective(SetObjective {
            objective_name: objective_name.to_owned(),
            method: 0,
            parameters: Some(objective.parameters.clone()),
        })];
        packets.extend(
            self.display_slots
                .iter()
                .filter(|(_, objective)| objective.as_str() == objective_name)
                .map(|(slot, objective)| {
                    ScoreboardPacket::SetDisplayObjective(SetDisplayObjective {
                        slot: *slot,
                        objective_name: Some(objective.clone()),
                    })
                }),
        );
        packets.extend(
            objective
                .scores
                .values()
                .cloned()
                .map(ScoreboardPacket::SetScore),
        );
        packets
    }

    fn broadcast(
        &self,
        packets: impl IntoIterator<Item = ScoreboardPacket>,
    ) -> Vec<ScoreboardDelivery> {
        let packets = packets.into_iter().collect::<Vec<_>>();
        self.players
            .iter()
            .flat_map(|recipient| {
                packets.iter().cloned().map(|packet| ScoreboardDelivery {
                    recipient: *recipient,
                    packet,
                })
            })
            .collect()
    }
}

fn team_packet(name: &str, team: &AuthoritativeTeam, method: i8) -> ScoreboardPacket {
    ScoreboardPacket::SetPlayerTeam(SetPlayerTeam {
        team_name: name.to_owned(),
        method,
        parameters: matches!(method, 0 | 2).then(|| team.parameters.clone()),
        players: if method == 0 {
            team.members.iter().cloned().collect()
        } else {
            Vec::new()
        },
    })
}
