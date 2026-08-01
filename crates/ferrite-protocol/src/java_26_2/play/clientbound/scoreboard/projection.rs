use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::scoreboard::packet::{
    DisplaySlot, NumberFormat, ObjectiveParameters, ResetScore, ScoreboardPacket,
    SetDisplayObjective, SetObjective, SetPlayerTeam, SetScore, TeamParameters,
};
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientObjective {
    pub parameters: ObjectiveParameters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientScore {
    pub value: i32,
    pub display: Option<TextComponentNbt>,
    pub number_format: Option<NumberFormat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientTeam {
    pub parameters: TeamParameters,
    pub members: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreboardEffect {
    pub mutated: bool,
    pub warned: bool,
}

impl ScoreboardEffect {
    const MUTATED: Self = Self {
        mutated: true,
        warned: false,
    };
    const NO_OP: Self = Self {
        mutated: false,
        warned: false,
    };
    const WARNING: Self = Self {
        mutated: false,
        warned: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormatSource {
    Entry,
    Objective,
    RedDecimalDefault,
    YellowDecimalDefault,
    UnstyledDecimalDefault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentedScore {
    pub owner: String,
    pub display: Option<TextComponentNbt>,
    pub team_name: Option<String>,
    pub team_parameters: Option<TeamParameters>,
    pub value: i32,
    pub hearts: bool,
    pub number_format: Option<NumberFormat>,
    pub format_source: NumberFormatSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BelowNamePresentation {
    pub score: PresentedScore,
    pub objective_display_name: TextComponentNbt,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScoreboardProjection {
    objectives: BTreeMap<String, ClientObjective>,
    scores: BTreeMap<(String, String), ClientScore>,
    display_slots: BTreeMap<DisplaySlot, String>,
    teams: BTreeMap<String, ClientTeam>,
    member_teams: BTreeMap<String, String>,
}

impl ScoreboardProjection {
    #[must_use]
    pub fn objectives(&self) -> &BTreeMap<String, ClientObjective> {
        &self.objectives
    }

    #[must_use]
    pub fn scores(&self) -> &BTreeMap<(String, String), ClientScore> {
        &self.scores
    }

    #[must_use]
    pub fn display_slots(&self) -> &BTreeMap<DisplaySlot, String> {
        &self.display_slots
    }

    #[must_use]
    pub fn teams(&self) -> &BTreeMap<String, ClientTeam> {
        &self.teams
    }

    #[must_use]
    pub fn member_team(&self, member: &str) -> Option<&str> {
        self.member_teams.get(member).map(String::as_str)
    }

    pub fn apply(
        &mut self,
        packet: ScoreboardPacket,
    ) -> Result<ScoreboardEffect, ScoreboardProjectionError> {
        match packet {
            ScoreboardPacket::ResetScore(packet) => self.apply_reset(packet),
            ScoreboardPacket::SetDisplayObjective(packet) => Ok(self.apply_display(packet)),
            ScoreboardPacket::SetObjective(packet) => self.apply_objective(packet),
            ScoreboardPacket::SetPlayerTeam(packet) => self.apply_team(packet),
            ScoreboardPacket::SetScore(packet) => Ok(self.apply_score(packet)),
        }
    }

    pub fn apply_objective(
        &mut self,
        packet: SetObjective,
    ) -> Result<ScoreboardEffect, ScoreboardProjectionError> {
        match packet.method {
            0 => {
                if self.objectives.contains_key(&packet.objective_name) {
                    return Err(ScoreboardProjectionError::DuplicateObjective {
                        objective: packet.objective_name,
                    });
                }
                let parameters = packet
                    .parameters
                    .ok_or(ScoreboardProjectionError::MissingObjectiveParameters { method: 0 })?;
                self.objectives
                    .insert(packet.objective_name, ClientObjective { parameters });
                Ok(ScoreboardEffect::MUTATED)
            }
            1 => {
                if self.objectives.remove(&packet.objective_name).is_none() {
                    return Ok(ScoreboardEffect::NO_OP);
                }
                self.display_slots
                    .retain(|_, objective| objective != &packet.objective_name);
                self.scores
                    .retain(|(_, objective), _| objective != &packet.objective_name);
                Ok(ScoreboardEffect::MUTATED)
            }
            2 => {
                let Some(objective) = self.objectives.get_mut(&packet.objective_name) else {
                    return Ok(ScoreboardEffect::WARNING);
                };
                objective.parameters = packet
                    .parameters
                    .ok_or(ScoreboardProjectionError::MissingObjectiveParameters { method: 2 })?;
                Ok(ScoreboardEffect::MUTATED)
            }
            _ => Ok(ScoreboardEffect::NO_OP),
        }
    }

    pub fn apply_score(&mut self, packet: SetScore) -> ScoreboardEffect {
        if !self.objectives.contains_key(&packet.objective_name) {
            return ScoreboardEffect::WARNING;
        }
        self.scores.insert(
            (packet.owner, packet.objective_name),
            ClientScore {
                value: packet.score,
                display: packet.display,
                number_format: packet.number_format,
            },
        );
        ScoreboardEffect::MUTATED
    }

    pub fn apply_reset(
        &mut self,
        packet: ResetScore,
    ) -> Result<ScoreboardEffect, ScoreboardProjectionError> {
        if let Some(objective) = packet.objective_name {
            if !self.objectives.contains_key(&objective) {
                return Ok(ScoreboardEffect::WARNING);
            }
            let removed = self.scores.remove(&(packet.owner, objective)).is_some();
            return Ok(if removed {
                ScoreboardEffect::MUTATED
            } else {
                ScoreboardEffect::NO_OP
            });
        }
        let before = self.scores.len();
        self.scores.retain(|(owner, _), _| owner != &packet.owner);
        Ok(if self.scores.len() == before {
            ScoreboardEffect::NO_OP
        } else {
            ScoreboardEffect::MUTATED
        })
    }

    pub fn apply_display(&mut self, packet: SetDisplayObjective) -> ScoreboardEffect {
        let resolved = packet
            .objective_name
            .filter(|objective| self.objectives.contains_key(objective));
        match resolved {
            Some(objective) => {
                self.display_slots.insert(packet.slot, objective);
            }
            None => {
                self.display_slots.remove(&packet.slot);
            }
        }
        ScoreboardEffect::MUTATED
    }

    pub fn apply_team(
        &mut self,
        packet: SetPlayerTeam,
    ) -> Result<ScoreboardEffect, ScoreboardProjectionError> {
        match packet.method {
            0 => {
                let duplicate = self.teams.contains_key(&packet.team_name);
                let parameters = packet
                    .parameters
                    .ok_or(ScoreboardProjectionError::MissingTeamParameters { method: 0 })?;
                self.teams
                    .entry(packet.team_name.clone())
                    .and_modify(|team| team.parameters = parameters.clone())
                    .or_insert(ClientTeam {
                        parameters,
                        members: BTreeSet::new(),
                    });
                self.add_members(&packet.team_name, packet.players);
                Ok(ScoreboardEffect {
                    mutated: true,
                    warned: duplicate,
                })
            }
            1 => {
                let Some(team) = self.teams.remove(&packet.team_name) else {
                    return Ok(ScoreboardEffect::WARNING);
                };
                for member in team.members {
                    self.member_teams.remove(&member);
                }
                Ok(ScoreboardEffect::MUTATED)
            }
            2 => {
                let Some(team) = self.teams.get_mut(&packet.team_name) else {
                    return Ok(ScoreboardEffect::WARNING);
                };
                team.parameters = packet
                    .parameters
                    .ok_or(ScoreboardProjectionError::MissingTeamParameters { method: 2 })?;
                Ok(ScoreboardEffect::MUTATED)
            }
            3 => {
                if !self.teams.contains_key(&packet.team_name) {
                    return Ok(ScoreboardEffect::WARNING);
                }
                self.add_members(&packet.team_name, packet.players);
                Ok(ScoreboardEffect::MUTATED)
            }
            4 => {
                if !self.teams.contains_key(&packet.team_name) {
                    return Ok(ScoreboardEffect::WARNING);
                }
                for member in packet.players {
                    if self.member_teams.get(&member).map(String::as_str)
                        != Some(packet.team_name.as_str())
                    {
                        return Err(ScoreboardProjectionError::InvalidTeamMemberRemoval {
                            team: packet.team_name,
                            member,
                        });
                    }
                    self.member_teams.remove(&member);
                    self.teams
                        .get_mut(&packet.team_name)
                        .expect("team existence was checked")
                        .members
                        .remove(&member);
                }
                Ok(ScoreboardEffect::MUTATED)
            }
            _ if self.teams.contains_key(&packet.team_name) => Ok(ScoreboardEffect::NO_OP),
            _ => Ok(ScoreboardEffect::WARNING),
        }
    }

    #[must_use]
    pub fn sidebar_entries(&self, local_owner: &str) -> Vec<PresentedScore> {
        let objective_name = self
            .member_team(local_owner)
            .and_then(|team| self.teams.get(team))
            .and_then(|team| team.parameters.color)
            .and_then(|color| self.display_slots.get(&DisplaySlot::SidebarTeam(color)))
            .or_else(|| self.display_slots.get(&DisplaySlot::Sidebar));
        let Some(objective_name) = objective_name else {
            return Vec::new();
        };
        let Some(objective) = self.objectives.get(objective_name) else {
            return Vec::new();
        };
        let mut entries = self
            .scores
            .iter()
            .filter(|((owner, score_objective), _)| {
                score_objective == objective_name && !owner.starts_with('#')
            })
            .map(|((owner, _), score)| {
                let (number_format, format_source) = if let Some(format) = &score.number_format {
                    (Some(format.clone()), NumberFormatSource::Entry)
                } else if let Some(format) = &objective.parameters.number_format {
                    (Some(format.clone()), NumberFormatSource::Objective)
                } else {
                    (None, NumberFormatSource::RedDecimalDefault)
                };
                PresentedScore {
                    owner: owner.clone(),
                    display: score.display.clone(),
                    team_name: self.member_team(owner).map(str::to_owned),
                    team_parameters: self
                        .member_team(owner)
                        .and_then(|team| self.teams.get(team))
                        .map(|team| team.parameters.clone()),
                    value: score.value,
                    hearts: false,
                    number_format,
                    format_source,
                }
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            right.value.cmp(&left.value).then_with(|| {
                let folded = left.owner.to_lowercase().cmp(&right.owner.to_lowercase());
                if folded == Ordering::Equal {
                    left.owner.cmp(&right.owner)
                } else {
                    folded
                }
            })
        });
        entries.truncate(15);
        entries
    }

    #[must_use]
    pub fn player_list_entries(&self, listed_owners: &[String]) -> Vec<PresentedScore> {
        let Some(objective_name) = self.display_slots.get(&DisplaySlot::List) else {
            return Vec::new();
        };
        let Some(objective) = self.objectives.get(objective_name) else {
            return Vec::new();
        };
        listed_owners
            .iter()
            .take(80)
            .filter_map(|owner| {
                self.scores
                    .get(&(owner.clone(), objective_name.clone()))
                    .map(|score| {
                        self.present_score(
                            owner,
                            score,
                            objective,
                            NumberFormatSource::YellowDecimalDefault,
                        )
                    })
            })
            .collect()
    }

    #[must_use]
    pub fn below_name_entry(
        &self,
        owner: &str,
        inside_distance: bool,
    ) -> Option<BelowNamePresentation> {
        if !inside_distance {
            return None;
        }
        let objective_name = self.display_slots.get(&DisplaySlot::BelowName)?;
        let objective = self.objectives.get(objective_name)?;
        let score = self
            .scores
            .get(&(owner.to_owned(), objective_name.clone()))?;
        Some(BelowNamePresentation {
            score: self.present_score(
                owner,
                score,
                objective,
                NumberFormatSource::UnstyledDecimalDefault,
            ),
            objective_display_name: objective.parameters.display_name.clone(),
        })
    }

    fn present_score(
        &self,
        owner: &str,
        score: &ClientScore,
        objective: &ClientObjective,
        default_source: NumberFormatSource,
    ) -> PresentedScore {
        let hearts = objective.parameters.render_type
            == crate::java_26_2::play::clientbound::scoreboard::packet::ObjectiveRenderType::Hearts;
        let (number_format, format_source) = if hearts {
            (None, default_source)
        } else if let Some(format) = &score.number_format {
            (Some(format.clone()), NumberFormatSource::Entry)
        } else if let Some(format) = &objective.parameters.number_format {
            (Some(format.clone()), NumberFormatSource::Objective)
        } else {
            (None, default_source)
        };
        let team_name = self.member_team(owner).map(str::to_owned);
        PresentedScore {
            owner: owner.to_owned(),
            display: score.display.clone(),
            team_parameters: team_name
                .as_deref()
                .and_then(|team| self.teams.get(team))
                .map(|team| team.parameters.clone()),
            team_name,
            value: score.value,
            hearts,
            number_format,
            format_source,
        }
    }

    fn add_members(&mut self, team_name: &str, members: Vec<String>) {
        for member in members {
            if let Some(old_team) = self
                .member_teams
                .insert(member.clone(), team_name.to_owned())
                && old_team != team_name
                && let Some(team) = self.teams.get_mut(&old_team)
            {
                team.members.remove(&member);
            }
            self.teams
                .get_mut(team_name)
                .expect("target team existence was checked")
                .members
                .insert(member);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScoreboardProjectionError {
    #[error("scoreboard objective {objective} was added twice")]
    DuplicateObjective { objective: String },
    #[error("scoreboard objective method {method} requires parameters")]
    MissingObjectiveParameters { method: i8 },
    #[error("scoreboard team method {method} requires parameters")]
    MissingTeamParameters { method: i8 },
    #[error("member {member} is not currently assigned to team {team}")]
    InvalidTeamMemberRemoval { team: String, member: String },
}
