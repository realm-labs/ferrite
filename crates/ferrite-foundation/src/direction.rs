//! Stable axis and direction identities.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AxisDirection {
    Negative,
    Positive,
}

impl AxisDirection {
    pub const fn sign(self) -> i32 {
        match self {
            Self::Negative => -1,
            Self::Positive => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Direction {
    pub const ALL: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::North,
        Self::South,
        Self::West,
        Self::East,
    ];
    pub const HORIZONTAL: [Self; 4] = [Self::North, Self::South, Self::West, Self::East];

    pub const fn axis(self) -> Axis {
        match self {
            Self::Down | Self::Up => Axis::Y,
            Self::North | Self::South => Axis::Z,
            Self::West | Self::East => Axis::X,
        }
    }

    pub const fn axis_direction(self) -> AxisDirection {
        match self {
            Self::Down | Self::North | Self::West => AxisDirection::Negative,
            Self::Up | Self::South | Self::East => AxisDirection::Positive,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    pub const fn step(self) -> [i32; 3] {
        match self {
            Self::Down => [0, -1, 0],
            Self::Up => [0, 1, 0],
            Self::North => [0, 0, -1],
            Self::South => [0, 0, 1],
            Self::West => [-1, 0, 0],
            Self::East => [1, 0, 0],
        }
    }

    pub const fn is_horizontal(self) -> bool {
        !matches!(self, Self::Down | Self::Up)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_is_an_involution() {
        for direction in Direction::ALL {
            assert_eq!(direction.opposite().opposite(), direction);
            assert_eq!(
                direction.axis_direction().sign(),
                direction.step().into_iter().sum::<i32>()
            );
        }
    }
}
