//! Completed portal contact, destination geometry, and transfer planning.

pub mod end_portal;
pub mod gateway;
pub mod nether;
pub mod processor;
pub mod transfer;

use ferrite_foundation::coordinate::BlockPos;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizontalAxis {
    X,
    Z,
}

impl HorizontalAxis {
    pub(crate) const fn positive_step(self) -> [i32; 3] {
        match self {
            Self::X => [1, 0, 0],
            Self::Z => [0, 0, 1],
        }
    }

    pub(crate) const fn clockwise_step(self) -> [i32; 3] {
        match self {
            Self::X => [0, 0, 1],
            Self::Z => [-1, 0, 0],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn containing(self) -> BlockPos {
        BlockPos::new(
            self.x.floor() as i32,
            self.y.floor() as i32,
            self.z.floor() as i32,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChunkTicket {
    pub position: BlockPos,
    pub radius: u8,
}

impl ChunkTicket {
    pub(crate) const fn portal(position: BlockPos) -> Self {
        Self {
            position,
            radius: 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PortalRectangle {
    pub minimum: BlockPos,
    pub axis: HorizontalAxis,
    pub width: u8,
    pub height: u8,
}

impl PortalRectangle {
    pub fn contains(self, position: BlockPos) -> bool {
        if position.y < self.minimum.y || position.y >= self.minimum.y + i32::from(self.height) {
            return false;
        }
        match self.axis {
            HorizontalAxis::X => {
                position.z == self.minimum.z
                    && (self.minimum.x..self.minimum.x + i32::from(self.width))
                        .contains(&position.x)
            }
            HorizontalAxis::Z => {
                position.x == self.minimum.x
                    && (self.minimum.z..self.minimum.z + i32::from(self.width))
                        .contains(&position.z)
            }
        }
    }
}

pub(crate) fn offset(position: BlockPos, step: [i32; 3], amount: i32) -> BlockPos {
    BlockPos::new(
        position.x.saturating_add(step[0].saturating_mul(amount)),
        position.y.saturating_add(step[1].saturating_mul(amount)),
        position.z.saturating_add(step[2].saturating_mul(amount)),
    )
}
