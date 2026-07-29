//! Chorus plant/flower growth and Chorus Fruit teleport admission.

use ferrite_foundation::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChorusConnections {
    pub down: bool,
    pub up: bool,
    pub north: bool,
    pub south: bool,
    pub west: bool,
    pub east: bool,
}

impl ChorusConnections {
    pub fn state_id(self) -> u32 {
        14_642
            + 32 * u32::from(!self.down)
            + 16 * u32::from(!self.east)
            + 8 * u32::from(!self.north)
            + 4 * u32::from(!self.south)
            + 2 * u32::from(!self.up)
            + u32::from(!self.west)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChorusTick {
    Blocked,
    GrowUp { next_age: u8 },
    Branch { attempts: u8, next_age: u8 },
    Die,
}

pub fn chorus_flower_tick(
    age: u8,
    y_below_build_max: bool,
    above_clear: bool,
    vertical_growth_admitted: bool,
    rooted: bool,
    branch_draw_0_to_3: u8,
) -> ChorusTick {
    if age >= 5 || !y_below_build_max {
        return ChorusTick::Blocked;
    }
    if above_clear && vertical_growth_admitted {
        return ChorusTick::GrowUp { next_age: age };
    }
    if age >= 4 {
        ChorusTick::Die
    } else {
        ChorusTick::Branch {
            attempts: 1 + branch_draw_0_to_3.min(3) + u8::from(rooted),
            next_age: age + 1,
        }
    }
}

pub fn distinct_chorus_branches(draws: &[Direction]) -> Vec<Direction> {
    let mut branches = Vec::new();
    for direction in draws
        .iter()
        .copied()
        .filter(|direction| direction.is_horizontal())
    {
        if !branches.contains(&direction) {
            branches.push(direction);
        }
    }
    branches
}

pub fn chorus_fruit_offset(next_double: f64) -> f64 {
    (next_double - 0.5) * 16.0
}

pub fn chorus_flower_state_id(age: u8) -> Option<u32> {
    (age <= 5).then_some(14_706 + age as u32)
}

pub fn chorus_teleport_attempts() -> u8 {
    16
}
