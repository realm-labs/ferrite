/// The required serverbound packet that closes the initial Play position handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayServerboundEntryPacket {
    AcceptTeleportation(AcceptTeleportation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptTeleportation {
    pub challenge: i32,
}
