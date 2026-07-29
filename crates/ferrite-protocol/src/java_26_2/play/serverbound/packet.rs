use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

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
    MoveVehicle(MoveVehicle),
    PickItemFromBlock(PickItemFromBlock),
    PlayerAction(PlayerAction),
    PlayerLoaded,
    Pong(Pong),
    Swing(Swing),
    UseItemOn(UseItemOn),
    UseItem(UseItem),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pong {
    pub payload: i32,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveVehicle {
    pub position: PlayerPosition,
    pub rotation: PlayerRotation,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickItemFromBlock {
    pub position: BlockPos,
    pub include_data: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerAction {
    pub action: PlayerActionKind,
    pub position: BlockPos,
    pub direction: Direction,
    pub sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerActionKind {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapItemWithOffhand,
    Stab,
}

impl PlayerActionKind {
    pub const fn from_index(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::StartDestroyBlock),
            1 => Some(Self::AbortDestroyBlock),
            2 => Some(Self::StopDestroyBlock),
            3 => Some(Self::DropAllItems),
            4 => Some(Self::DropItem),
            5 => Some(Self::ReleaseUseItem),
            6 => Some(Self::SwapItemWithOffhand),
            7 => Some(Self::Stab),
            _ => None,
        }
    }

    #[must_use]
    pub const fn index(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn is_destroy(self) -> bool {
        matches!(
            self,
            Self::StartDestroyBlock | Self::AbortDestroyBlock | Self::StopDestroyBlock
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Main,
    Off,
}

impl Hand {
    pub const fn from_index(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Main),
            1 => Some(Self::Off),
            _ => None,
        }
    }

    #[must_use]
    pub const fn index(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Swing {
    pub hand: Hand,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockHit {
    pub position: BlockPos,
    pub direction: Direction,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    pub inside: bool,
    pub world_border_hit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UseItemOn {
    pub hand: Hand,
    pub hit: BlockHit,
    pub sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UseItem {
    pub hand: Hand,
    pub sequence: i32,
    pub yaw: f32,
    pub pitch: f32,
}
