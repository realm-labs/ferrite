use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::player_projection::packet::{
    AwardStats, Cooldown, SetExperience, SetHealth, StatisticKey,
};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CooldownInterval {
    pub start_tick: i32,
    pub end_tick: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthProjection {
    pub initialized: bool,
    pub maximum: f32,
    pub current: f32,
    pub food: i32,
    pub saturation: f32,
    pub invulnerable_ticks: i32,
    pub hurt_duration: i32,
    pub hurt_time: i32,
    pub last_hurt_amount: f32,
}

impl Default for HealthProjection {
    fn default() -> Self {
        Self {
            initialized: false,
            maximum: 20.0,
            current: 20.0,
            food: 20,
            saturation: 5.0,
            invulnerable_ticks: 0,
            hurt_duration: 0,
            hurt_time: 0,
            last_hurt_amount: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExperienceProjection {
    pub progress: f32,
    pub level: i32,
    pub total: i32,
    pub display_start_tick: Option<i32>,
}

impl Default for ExperienceProjection {
    fn default() -> Self {
        Self {
            progress: 0.0,
            level: 0,
            total: 0,
            display_start_tick: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthReaction {
    FirstValue,
    Hurt { amount: f32 },
    Increase,
    Equal,
    NonFiniteNondamage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatisticsApplication {
    pub updated: usize,
    pub screen_callback: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerProjection {
    tick_count: i32,
    health: HealthProjection,
    experience: ExperienceProjection,
    cooldowns: BTreeMap<Identifier, CooldownInterval>,
    statistics: BTreeMap<StatisticKey, i32>,
    stats_screen_callbacks: usize,
}

impl PlayerProjection {
    #[must_use]
    pub const fn tick_count(&self) -> i32 {
        self.tick_count
    }

    #[must_use]
    pub const fn health(&self) -> HealthProjection {
        self.health
    }

    #[must_use]
    pub const fn experience(&self) -> ExperienceProjection {
        self.experience
    }

    #[must_use]
    pub fn cooldowns(&self) -> &BTreeMap<Identifier, CooldownInterval> {
        &self.cooldowns
    }

    #[must_use]
    pub fn statistics(&self) -> &BTreeMap<StatisticKey, i32> {
        &self.statistics
    }

    #[must_use]
    pub const fn stats_screen_callbacks(&self) -> usize {
        self.stats_screen_callbacks
    }

    pub fn apply_health(&mut self, packet: SetHealth) -> HealthReaction {
        let reaction = if !self.health.initialized {
            HealthReaction::FirstValue
        } else if packet.health.is_nan() {
            self.health.invulnerable_ticks = 10;
            HealthReaction::NonFiniteNondamage
        } else if packet.health < self.health.current {
            let amount = self.health.current - packet.health;
            self.health.last_hurt_amount = amount;
            self.health.invulnerable_ticks = 20;
            self.health.hurt_duration = 10;
            self.health.hurt_time = 10;
            HealthReaction::Hurt { amount }
        } else if packet.health > self.health.current {
            self.health.invulnerable_ticks = 10;
            HealthReaction::Increase
        } else {
            HealthReaction::Equal
        };
        self.health.current = clamp_health(packet.health, self.health.maximum);
        self.health.food = packet.food;
        self.health.saturation = packet.saturation;
        self.health.initialized = true;
        reaction
    }

    pub fn apply_experience(&mut self, packet: SetExperience) {
        if packet.progress != self.experience.progress {
            self.experience.display_start_tick = Some(self.tick_count);
        }
        self.experience.progress = packet.progress;
        self.experience.total = packet.total_experience;
        self.experience.level = packet.level;
    }

    pub fn apply_cooldown(&mut self, packet: Cooldown) {
        if packet.duration_ticks == 0 {
            self.cooldowns.remove(&packet.group);
        } else {
            self.cooldowns.insert(
                packet.group,
                CooldownInterval {
                    start_tick: self.tick_count,
                    end_tick: self.tick_count.wrapping_add(packet.duration_ticks),
                },
            );
        }
    }

    pub fn apply_statistics(
        &mut self,
        packet: AwardStats,
        stats_screen_open: bool,
    ) -> StatisticsApplication {
        let updated = packet.values.len();
        for (statistic, value) in packet.values {
            self.statistics.insert(statistic, value);
        }
        if stats_screen_open {
            self.stats_screen_callbacks += 1;
        }
        StatisticsApplication {
            updated,
            screen_callback: stats_screen_open,
        }
    }

    pub fn tick_cooldowns(&mut self) -> Vec<Identifier> {
        self.tick_count = self.tick_count.wrapping_add(1);
        let expired = self
            .cooldowns
            .iter()
            .filter(|(_, interval)| interval.end_tick <= self.tick_count)
            .map(|(group, _)| group.clone())
            .collect::<Vec<_>>();
        for group in &expired {
            self.cooldowns.remove(group);
        }
        expired
    }

    #[must_use]
    pub fn cooldown_percentage(&self, group: &Identifier, partial_tick: f32) -> Option<f32> {
        self.cooldowns.get(group).map(|interval| {
            let remaining = interval.end_tick as f32 - (self.tick_count as f32 + partial_tick);
            let duration = interval.end_tick.wrapping_sub(interval.start_tick) as f32;
            (remaining / duration).clamp(0.0, 1.0)
        })
    }
}

fn clamp_health(value: f32, maximum: f32) -> f32 {
    if value.is_nan() || value == f32::NEG_INFINITY {
        0.0
    } else if value == f32::INFINITY || value > maximum {
        maximum
    } else if value < 0.0 {
        0.0
    } else {
        value
    }
}
