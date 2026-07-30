//! Piecewise player experience normalization and level-side effects.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExperienceData {
    pub level: i32,
    pub progress: f32,
    pub total: i32,
    pub score: i32,
    pub last_level_up_tick: i32,
    pub tick_count: i32,
    pub enchantment_seed: i32,
}

impl ExperienceData {
    pub const fn new(enchantment_seed: i32) -> Self {
        Self {
            level: 0,
            progress: 0.0,
            total: 0,
            score: 0,
            last_level_up_tick: 0,
            tick_count: 0,
            enchantment_seed,
        }
    }

    pub fn needed_for_next_level(&self) -> i32 {
        points_for_level(self.level)
    }

    pub fn give_points(&mut self, points: i32) -> Vec<LevelUpSound> {
        self.score = self.score.wrapping_add(points);
        self.progress += points as f32 / self.needed_for_next_level() as f32;
        self.total = self.total.wrapping_add(points).clamp(0, i32::MAX);
        let mut sounds = Vec::new();
        while self.progress < 0.0 {
            let remaining = self.progress * self.needed_for_next_level() as f32;
            if self.level > 0 {
                if let Some(sound) = self.give_levels(-1) {
                    sounds.push(sound);
                }
                self.progress = 1.0 + remaining / self.needed_for_next_level() as f32;
            } else {
                if let Some(sound) = self.give_levels(-1) {
                    sounds.push(sound);
                }
                self.progress = 0.0;
            }
        }
        while self.progress >= 1.0 {
            self.progress = (self.progress - 1.0) * self.needed_for_next_level() as f32;
            if let Some(sound) = self.give_levels(1) {
                sounds.push(sound);
            }
            self.progress /= self.needed_for_next_level() as f32;
        }
        sounds
    }

    pub fn give_levels(&mut self, amount: i32) -> Option<LevelUpSound> {
        self.level = self.level.saturating_add(amount);
        if self.level < 0 {
            self.level = 0;
            self.progress = 0.0;
            self.total = 0;
        }
        if amount > 0
            && self.level % 5 == 0
            && (self.last_level_up_tick as f32) < self.tick_count as f32 - 100.0
        {
            let volume = if self.level > 30 {
                0.75
            } else {
                self.level as f32 / 30.0 * 0.75
            };
            self.last_level_up_tick = self.tick_count;
            Some(LevelUpSound { volume, pitch: 1.0 })
        } else {
            None
        }
    }

    pub fn on_enchantment_performed(&mut self, cost: i32, refreshed_seed: i32) {
        self.level = self.level.wrapping_sub(cost);
        if self.level < 0 {
            self.level = 0;
            self.progress = 0.0;
            self.total = 0;
        }
        self.enchantment_seed = refreshed_seed;
    }

    pub fn load_seed(&mut self, loaded_seed: i32, replacement: i32) {
        self.enchantment_seed = if loaded_seed == 0 {
            replacement
        } else {
            loaded_seed
        };
    }

    pub fn death_reward(&self, spectator: bool, keep_inventory: bool) -> i32 {
        if spectator || keep_inventory {
            0
        } else {
            self.level.wrapping_mul(7).min(100)
        }
    }
}

pub const fn points_for_level(level: i32) -> i32 {
    if level >= 30 {
        112_i32.wrapping_add(level.wrapping_sub(30).wrapping_mul(9))
    } else if level >= 15 {
        37_i32.wrapping_add(level.wrapping_sub(15).wrapping_mul(5))
    } else {
        7_i32.wrapping_add(level.wrapping_mul(2))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelUpSound {
    pub volume: f32,
    pub pitch: f32,
}
