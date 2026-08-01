use std::collections::{BTreeMap, BTreeSet};

use crate::java_26_2::play::clientbound::player_projection::packet::{
    AwardStats, Cooldown, SetExperience, SetHealth, StatisticKey,
};
use crate::java_26_2::play::clientbound::player_projection::projection::CooldownInterval;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerProjectionDelivery {
    Cooldown(Cooldown),
    Health(SetHealth),
    Experience(SetExperience),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerProjectionPublisher {
    tick_count: i32,
    health: f32,
    food: i32,
    saturation: f32,
    experience: SetExperience,
    last_sent_health: Option<f32>,
    last_sent_food: Option<i32>,
    last_saturation_zero: Option<bool>,
    last_sent_experience: i32,
    cooldowns: BTreeMap<Identifier, CooldownInterval>,
    statistics: BTreeMap<StatisticKey, i32>,
    dirty_statistics: BTreeSet<StatisticKey>,
}

impl Default for PlayerProjectionPublisher {
    fn default() -> Self {
        Self {
            tick_count: 0,
            health: 20.0,
            food: 20,
            saturation: 5.0,
            experience: SetExperience {
                progress: 0.0,
                level: 0,
                total_experience: 0,
            },
            last_sent_health: None,
            last_sent_food: None,
            last_saturation_zero: None,
            last_sent_experience: -1,
            cooldowns: BTreeMap::new(),
            statistics: BTreeMap::new(),
            dirty_statistics: BTreeSet::new(),
        }
    }
}

impl PlayerProjectionPublisher {
    pub const fn set_vitals(&mut self, health: f32, food: i32, saturation: f32) {
        self.health = health;
        self.food = food;
        self.saturation = saturation;
    }

    pub fn set_experience(&mut self, experience: SetExperience) {
        self.experience = experience;
    }

    pub fn set_experience_progress(&mut self, progress: f32) {
        self.experience.progress = progress;
        self.last_sent_experience = -1;
    }

    pub const fn set_experience_level(&mut self, level: i32) {
        self.experience.level = level;
        self.last_sent_experience = -1;
    }

    pub fn reset_vitals_markers(&mut self) {
        self.last_sent_health = None;
        self.last_sent_food = None;
        self.last_saturation_zero = None;
    }

    #[must_use]
    pub const fn explicit_respawn_experience(&self) -> SetExperience {
        self.experience
    }

    pub fn start_cooldown(&mut self, group: Identifier, duration_ticks: i32) -> Cooldown {
        self.cooldowns.insert(
            group.clone(),
            CooldownInterval {
                start_tick: self.tick_count,
                end_tick: self.tick_count.wrapping_add(duration_ticks),
            },
        );
        Cooldown {
            group,
            duration_ticks,
        }
    }

    pub fn remove_cooldown(&mut self, group: Identifier) -> Cooldown {
        self.cooldowns.remove(&group);
        Cooldown {
            group,
            duration_ticks: 0,
        }
    }

    pub fn set_statistic(&mut self, statistic: StatisticKey, value: i32) {
        self.statistics.insert(statistic.clone(), value);
        self.dirty_statistics.insert(statistic);
    }

    pub fn increment_statistic(&mut self, statistic: StatisticKey, delta: i32) {
        let current = self.statistics.get(&statistic).copied().unwrap_or(0);
        let widened = i64::from(current) + i64::from(delta);
        let narrowed = widened.min(i64::from(i32::MAX)) as i32;
        self.set_statistic(statistic, narrowed);
    }

    pub fn mark_all_statistics_dirty(&mut self) {
        self.dirty_statistics
            .extend(self.statistics.keys().cloned());
    }

    pub fn request_statistics(&mut self) -> AwardStats {
        let values = std::mem::take(&mut self.dirty_statistics)
            .into_iter()
            .filter_map(|statistic| {
                self.statistics
                    .get(&statistic)
                    .copied()
                    .map(|value| (statistic, value))
            })
            .collect();
        AwardStats { values }
    }

    pub fn publish_tick(&mut self) -> Vec<PlayerProjectionDelivery> {
        self.tick_count = self.tick_count.wrapping_add(1);
        let mut deliveries = Vec::new();
        let expired = self
            .cooldowns
            .iter()
            .filter(|(_, interval)| interval.end_tick <= self.tick_count)
            .map(|(group, _)| group.clone())
            .collect::<Vec<_>>();
        for group in expired {
            self.cooldowns.remove(&group);
            deliveries.push(PlayerProjectionDelivery::Cooldown(Cooldown {
                group,
                duration_ticks: 0,
            }));
        }

        let saturation_zero = self.saturation == 0.0;
        if self.last_sent_health != Some(self.health)
            || self.last_sent_food != Some(self.food)
            || self.last_saturation_zero != Some(saturation_zero)
        {
            deliveries.push(PlayerProjectionDelivery::Health(SetHealth {
                health: self.health,
                food: self.food,
                saturation: self.saturation,
            }));
            self.last_sent_health = Some(self.health);
            self.last_sent_food = Some(self.food);
            self.last_saturation_zero = Some(saturation_zero);
        }
        if self.experience.total_experience != self.last_sent_experience {
            self.last_sent_experience = self.experience.total_experience;
            deliveries.push(PlayerProjectionDelivery::Experience(self.experience));
        }
        deliveries
    }
}
