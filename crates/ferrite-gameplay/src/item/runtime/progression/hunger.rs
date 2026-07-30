//! Exhaustion-first hunger, regeneration, and starvation state.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodData {
    pub food_level: i32,
    pub saturation_level: f32,
    pub exhaustion_level: f32,
    pub tick_timer: u32,
}

impl FoodData {
    pub const fn new() -> Self {
        Self {
            food_level: 20,
            saturation_level: 5.0,
            exhaustion_level: 0.0,
            tick_timer: 0,
        }
    }

    pub fn eat(&mut self, nutrition: i32, saturation_modifier: f32) {
        self.add_food(nutrition, nutrition as f32 * saturation_modifier * 2.0);
    }

    pub fn add_food(&mut self, nutrition: i32, saturation: f32) {
        self.food_level = self.food_level.saturating_add(nutrition).clamp(0, 20);
        self.saturation_level =
            (self.saturation_level + saturation).clamp(0.0, self.food_level as f32);
    }

    pub fn add_exhaustion(&mut self, amount: f32) {
        self.exhaustion_level = (self.exhaustion_level + amount).min(40.0);
    }

    pub fn cause_exhaustion(&mut self, amount: f32, server_side: bool, invulnerable: bool) -> bool {
        if !server_side || invulnerable {
            return false;
        }
        self.add_exhaustion(amount);
        true
    }

    pub const fn has_enough_food(&self) -> bool {
        self.food_level > 6
    }

    pub const fn needs_food(&self) -> bool {
        self.food_level < 20
    }

    pub fn tick(&mut self, input: FoodTickInput) -> FoodTickOutcome {
        let mut spent_exhaustion = false;
        if self.exhaustion_level > 4.0 {
            self.exhaustion_level -= 4.0;
            spent_exhaustion = true;
            if self.saturation_level > 0.0 {
                self.saturation_level = (self.saturation_level - 1.0).max(0.0);
            } else if input.difficulty != Difficulty::Peaceful {
                self.food_level = (self.food_level - 1).max(0);
            }
        }

        let mut healed = 0.0;
        let mut starvation_damage = 0.0;
        let branch = if input.natural_regeneration
            && self.saturation_level > 0.0
            && input.hurt
            && self.food_level >= 20
        {
            self.tick_timer += 1;
            if self.tick_timer >= 10 {
                let saturation_spent = self.saturation_level.min(6.0);
                healed = saturation_spent / 6.0;
                self.add_exhaustion(saturation_spent);
                self.tick_timer = 0;
            }
            FoodBranch::SaturatedRegeneration
        } else if input.natural_regeneration && self.food_level >= 18 && input.hurt {
            self.tick_timer += 1;
            if self.tick_timer >= 80 {
                healed = 1.0;
                self.add_exhaustion(6.0);
                self.tick_timer = 0;
            }
            FoodBranch::SlowRegeneration
        } else if self.food_level <= 0 {
            self.tick_timer += 1;
            if self.tick_timer >= 80 {
                if input.health > 10.0
                    || input.difficulty == Difficulty::Hard
                    || (input.health > 1.0 && input.difficulty == Difficulty::Normal)
                {
                    starvation_damage = 1.0;
                }
                self.tick_timer = 0;
            }
            FoodBranch::Starvation
        } else {
            self.tick_timer = 0;
            FoodBranch::Idle
        };
        FoodTickOutcome {
            branch,
            spent_exhaustion,
            healed,
            starvation_damage,
        }
    }
}

impl Default for FoodData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodTickInput {
    pub difficulty: Difficulty,
    pub natural_regeneration: bool,
    pub hurt: bool,
    pub health: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodBranch {
    SaturatedRegeneration,
    SlowRegeneration,
    Starvation,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoodTickOutcome {
    pub branch: FoodBranch,
    pub spent_exhaustion: bool,
    pub healed: f32,
    pub starvation_damage: f32,
}
