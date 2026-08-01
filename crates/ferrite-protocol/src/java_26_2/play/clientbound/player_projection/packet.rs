use std::collections::BTreeMap;

use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatisticKey {
    pub statistic_type: Identifier,
    pub value: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwardStats {
    pub values: BTreeMap<StatisticKey, i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cooldown {
    pub group: Identifier,
    pub duration_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetExperience {
    pub progress: f32,
    pub level: i32,
    pub total_experience: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetHealth {
    pub health: f32,
    pub food: i32,
    pub saturation: f32,
}
