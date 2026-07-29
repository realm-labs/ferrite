/// Required Play entry and C2 session packets decoded by the 26.2 adapter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayServerboundEntryPacket {
    AcceptTeleportation(AcceptTeleportation),
    ChunkBatchReceived(ChunkBatchReceived),
    ClientTickEnd,
    KeepAlive(KeepAlive),
    MovePlayerPosition(MovePlayerPosition),
    MovePlayerPositionRotation(MovePlayerPositionRotation),
    MovePlayerRotation(MovePlayerRotation),
    MovePlayerStatusOnly(MovePlayerStatusOnly),
    PlayerLoaded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptTeleportation {
    pub challenge: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkBatchReceived {
    pub desired_chunks_per_tick: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeepAlive {
    pub challenge: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovementFlags {
    pub on_ground: bool,
    pub horizontal_collision: bool,
}

impl MovementFlags {
    #[must_use]
    pub const fn from_wire(value: u8) -> Self {
        Self {
            on_ground: value & 0x01 != 0,
            horizontal_collision: value & 0x02 != 0,
        }
    }

    #[must_use]
    pub const fn to_wire(self) -> u8 {
        self.on_ground as u8 | ((self.horizontal_collision as u8) << 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovePlayerPosition {
    pub position: PlayerPosition,
    pub flags: MovementFlags,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovePlayerPositionRotation {
    pub position: PlayerPosition,
    pub rotation: PlayerRotation,
    pub flags: MovementFlags,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovePlayerRotation {
    pub rotation: PlayerRotation,
    pub flags: MovementFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovePlayerStatusOnly {
    pub flags: MovementFlags,
}
