use crate::java_26_2::play::clientbound::world_border::packet::{
    SetBorderCenter, SetBorderLerpSize, SetBorderSize, SetBorderWarningDelay,
    SetBorderWarningDistance,
};
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderDelta {
    Center(SetBorderCenter),
    LerpSize(SetBorderLerpSize),
    Size(SetBorderSize),
    WarningDelay(SetBorderWarningDelay),
    WarningDistance(SetBorderWarningDistance),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderAuthoritativeEvent {
    Center {
        center_x: f64,
        center_z: f64,
    },
    LerpSize {
        old_size: f64,
        new_size: f64,
        duration_millis: i64,
    },
    Size {
        size: f64,
    },
    WarningDelay {
        warning_time: i32,
    },
    WarningDistance {
        warning_blocks: i32,
    },
    DamagePerBlock,
    SafeZone,
    MovingTick,
}

impl BorderAuthoritativeEvent {
    #[must_use]
    pub const fn packet(self) -> Option<BorderDelta> {
        match self {
            Self::Center { center_x, center_z } => {
                Some(BorderDelta::Center(SetBorderCenter { center_x, center_z }))
            }
            Self::LerpSize {
                old_size,
                new_size,
                duration_millis,
            } => Some(BorderDelta::LerpSize(SetBorderLerpSize {
                old_size,
                new_size,
                duration_millis,
            })),
            Self::Size { size } => Some(BorderDelta::Size(SetBorderSize { size })),
            Self::WarningDelay { warning_time } => {
                Some(BorderDelta::WarningDelay(SetBorderWarningDelay {
                    warning_time,
                }))
            }
            Self::WarningDistance { warning_blocks } => {
                Some(BorderDelta::WarningDistance(SetBorderWarningDistance {
                    warning_blocks,
                }))
            }
            Self::DamagePerBlock | Self::SafeZone | Self::MovingTick => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorderViewer {
    pub player_id: u128,
    pub dimension: Identifier,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorderDelivery {
    pub recipient: u128,
    pub packet: BorderDelta,
}

#[must_use]
pub fn publish_event(
    event: BorderAuthoritativeEvent,
    dimension: &Identifier,
    viewers: &[BorderViewer],
) -> Vec<BorderDelivery> {
    let Some(packet) = event.packet() else {
        return Vec::new();
    };
    viewers
        .iter()
        .filter(|viewer| &viewer.dimension == dimension)
        .map(|viewer| BorderDelivery {
            recipient: viewer.player_id,
            packet,
        })
        .collect()
}
