use thiserror::Error;

pub const MINECRAFT_VERSION: &str = "26.2";
pub const PROTOCOL_VERSION: u32 = 776;
pub const PACKET_COUNT: usize = 256;
pub const PACKET_CATALOG_SHA1: &str = "f34b0956b6399c749d4638cd6d3c9226685f41fa";

/// A packet registry state. Packet IDs are meaningful only within one state and direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
}

impl ConnectionState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Handshake => "handshake",
            Self::Status => "status",
            Self::Login => "login",
            Self::Configuration => "configuration",
            Self::Play => "play",
        }
    }
}

/// Direction relative to the Minecraft server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PacketDirection {
    Clientbound,
    Serverbound,
}

impl PacketDirection {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clientbound => "clientbound",
            Self::Serverbound => "serverbound",
        }
    }
}

/// A nonnegative packet ID validated to fit Ferrite's locked catalog representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketId(u16);

impl PacketId {
    const fn new(value: usize) -> Self {
        Self(value as u16)
    }

    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl From<PacketId> for i32 {
    fn from(value: PacketId) -> Self {
        i32::from(value.0)
    }
}

impl TryFrom<i32> for PacketId {
    type Error = PacketIdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        u16::try_from(value)
            .map(Self)
            .map_err(|_| PacketIdError { value })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("packet ID {value} is outside the nonnegative 16-bit catalog boundary")]
pub struct PacketIdError {
    value: i32,
}

impl PacketIdError {
    #[must_use]
    pub const fn value(self) -> i32 {
        self.value
    }
}

/// One locked Minecraft 26.2 packet identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketDescriptor {
    state: ConnectionState,
    direction: PacketDirection,
    id: PacketId,
    identity: &'static str,
}

impl PacketDescriptor {
    const fn new(
        state: ConnectionState,
        direction: PacketDirection,
        id: PacketId,
        identity: &'static str,
    ) -> Self {
        Self {
            state,
            direction,
            id,
            identity,
        }
    }

    #[must_use]
    pub const fn state(self) -> ConnectionState {
        self.state
    }

    #[must_use]
    pub const fn direction(self) -> PacketDirection {
        self.direction
    }

    #[must_use]
    pub const fn id(self) -> PacketId {
        self.id
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        self.identity
    }
}

#[derive(Debug, Clone, Copy)]
struct LaneDescriptor {
    state: ConnectionState,
    direction: PacketDirection,
    start: usize,
    end: usize,
}

impl LaneDescriptor {
    const fn new(
        state: ConnectionState,
        direction: PacketDirection,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            state,
            direction,
            start,
            end,
        }
    }
}

mod generated {
    use crate::java_26_2::catalog::{
        ConnectionState, LaneDescriptor, PacketDescriptor, PacketDirection, PacketId,
    };

    include!(concat!(
        env!("OUT_DIR"),
        "/minecraft_java_26_2_packet_catalog.rs"
    ));
}

/// Read-only access to the complete locked packet catalog.
#[derive(Debug, Clone, Copy)]
pub struct PacketCatalog;

impl PacketCatalog {
    #[must_use]
    pub fn all() -> &'static [PacketDescriptor] {
        generated::PACKETS
    }

    #[must_use]
    pub fn lane(state: ConnectionState, direction: PacketDirection) -> &'static [PacketDescriptor] {
        lane_descriptor(state, direction)
            .map(|lane| &generated::PACKETS[lane.start..lane.end])
            .unwrap_or(&[])
    }

    #[must_use]
    pub fn by_id(
        state: ConnectionState,
        direction: PacketDirection,
        id: PacketId,
    ) -> Option<&'static PacketDescriptor> {
        let lane = lane_descriptor(state, direction)?;
        let index = lane.start.checked_add(usize::from(id.value()))?;
        if index >= lane.end {
            return None;
        }
        let packet = generated::PACKETS.get(index)?;
        (packet.id == id).then_some(packet)
    }

    #[must_use]
    pub fn by_wire_id(
        state: ConnectionState,
        direction: PacketDirection,
        wire_id: i32,
    ) -> Option<&'static PacketDescriptor> {
        Self::by_id(state, direction, PacketId::try_from(wire_id).ok()?)
    }

    #[must_use]
    pub fn by_identity(
        state: ConnectionState,
        direction: PacketDirection,
        identity: &str,
    ) -> Option<&'static PacketDescriptor> {
        Self::lane(state, direction)
            .iter()
            .find(|packet| packet.identity == identity)
    }
}

fn lane_descriptor(
    state: ConnectionState,
    direction: PacketDirection,
) -> Option<&'static LaneDescriptor> {
    generated::LANES
        .iter()
        .find(|lane| lane.state == state && lane.direction == direction)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::java_26_2::catalog::{
        ConnectionState, MINECRAFT_VERSION, PACKET_CATALOG_SHA1, PACKET_COUNT, PROTOCOL_VERSION,
        PacketCatalog, PacketDirection, PacketId,
    };

    #[test]
    fn locked_metadata_and_lane_counts_match_the_reference_inventory() {
        assert_eq!(MINECRAFT_VERSION, "26.2");
        assert_eq!(PROTOCOL_VERSION, 776);
        assert_eq!(
            PACKET_CATALOG_SHA1,
            "f34b0956b6399c749d4638cd6d3c9226685f41fa"
        );
        assert_eq!(PacketCatalog::all().len(), PACKET_COUNT);
        assert_eq!(
            PacketCatalog::lane(ConnectionState::Configuration, PacketDirection::Clientbound).len(),
            20
        );
        assert_eq!(
            PacketCatalog::lane(ConnectionState::Configuration, PacketDirection::Serverbound).len(),
            10
        );
        assert_eq!(
            PacketCatalog::lane(ConnectionState::Handshake, PacketDirection::Serverbound).len(),
            1
        );
        assert_eq!(
            PacketCatalog::lane(ConnectionState::Login, PacketDirection::Clientbound).len(),
            6
        );
        assert_eq!(
            PacketCatalog::lane(ConnectionState::Login, PacketDirection::Serverbound).len(),
            5
        );
        assert_eq!(
            PacketCatalog::lane(ConnectionState::Play, PacketDirection::Clientbound).len(),
            141
        );
        assert_eq!(
            PacketCatalog::lane(ConnectionState::Play, PacketDirection::Serverbound).len(),
            69
        );
        assert_eq!(
            PacketCatalog::lane(ConnectionState::Status, PacketDirection::Clientbound).len(),
            2
        );
        assert_eq!(
            PacketCatalog::lane(ConnectionState::Status, PacketDirection::Serverbound).len(),
            2
        );
    }

    #[test]
    fn every_lane_is_contiguous_and_round_trips_both_lookup_keys() {
        let mut tuples = BTreeSet::new();
        for packet in PacketCatalog::all() {
            assert!(tuples.insert((
                packet.state(),
                packet.direction(),
                packet.id(),
                packet.identity(),
            )));
            assert_eq!(
                PacketCatalog::by_id(packet.state(), packet.direction(), packet.id()),
                Some(packet)
            );
            assert_eq!(
                PacketCatalog::by_identity(packet.state(), packet.direction(), packet.identity()),
                Some(packet)
            );
        }
    }

    #[test]
    fn ids_are_state_and_direction_local() {
        let intention =
            PacketCatalog::by_wire_id(ConnectionState::Handshake, PacketDirection::Serverbound, 0)
                .unwrap();
        assert_eq!(intention.identity(), "minecraft:intention");
        assert!(
            PacketCatalog::lane(ConnectionState::Handshake, PacketDirection::Clientbound)
                .is_empty()
        );

        let status_response =
            PacketCatalog::by_wire_id(ConnectionState::Status, PacketDirection::Clientbound, 0)
                .unwrap();
        let status_request =
            PacketCatalog::by_wire_id(ConnectionState::Status, PacketDirection::Serverbound, 0)
                .unwrap();
        assert_eq!(status_response.identity(), "minecraft:status_response");
        assert_eq!(status_request.identity(), "minecraft:status_request");
    }

    #[test]
    fn invalid_and_unknown_ids_fail_closed() {
        assert!(PacketId::try_from(-1).is_err());
        assert!(PacketId::try_from(i32::MAX).is_err());
        assert!(
            PacketCatalog::by_wire_id(ConnectionState::Status, PacketDirection::Serverbound, 2)
                .is_none()
        );
        assert!(
            PacketCatalog::by_identity(
                ConnectionState::Status,
                PacketDirection::Serverbound,
                "minecraft:unknown"
            )
            .is_none()
        );
    }
}
