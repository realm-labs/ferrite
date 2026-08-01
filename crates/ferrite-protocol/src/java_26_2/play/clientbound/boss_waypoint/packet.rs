use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossOverlay {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BossOperation {
    Add {
        name: TextComponentNbt,
        progress: f32,
        color: BossColor,
        overlay: BossOverlay,
        properties: u8,
    },
    Remove,
    UpdateProgress(f32),
    UpdateName(TextComponentNbt),
    UpdateStyle {
        color: BossColor,
        overlay: BossOverlay,
    },
    UpdateProperties(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BossEvent {
    pub id: u128,
    pub operation: BossOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointOperation {
    Track,
    Untrack,
    Update,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaypointIdentifier {
    Uuid(u128),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointIcon {
    pub style: Identifier,
    /// Opaque `0xffRRGGBB`; only RGB is carried on the wire.
    pub color: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaypointLocation {
    Empty,
    Position { x: i32, y: i32, z: i32 },
    Chunk { x: i32, z: i32 },
    Azimuth { angle: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackedWaypoint {
    pub identifier: WaypointIdentifier,
    pub icon: WaypointIcon,
    pub location: WaypointLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaypointPacket {
    pub operation: WaypointOperation,
    pub waypoint: TrackedWaypoint,
}
