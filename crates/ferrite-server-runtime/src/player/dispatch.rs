//! Exhaustive application disposition for decoded Play serverbound packets.

use ferrite_gameplay::player::movement::MovementOutcome;
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;

use crate::player::block::session::BlockInteractionAction;
use crate::player::session::PlayerSessionAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundResponsibility {
    ProtocolTeleport,
    ProtocolKeepAlive,
    BlockInteraction,
    ChunkBatchFeedback,
    ClientLifecycle,
    Movement,
    ChatAndCommand,
    EntityInteraction,
    InventoryAndContainer,
    PlayerModeAndInput,
    Pong,
    VehicleInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundGate {
    ClientLoaded,
    RegionTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundRejection {
    InvalidMovement,
    Flying,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundDisposition {
    Handled,
    Rejected(ServerboundRejection),
    Gated(ServerboundGate),
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundDispatchOutcome {
    packet: &'static str,
    responsibility: ServerboundResponsibility,
    disposition: ServerboundDisposition,
}

impl ServerboundDispatchOutcome {
    #[must_use]
    pub const fn packet(self) -> &'static str {
        self.packet
    }

    #[must_use]
    pub const fn responsibility(self) -> ServerboundResponsibility {
        self.responsibility
    }

    #[must_use]
    pub const fn disposition(self) -> ServerboundDisposition {
        self.disposition
    }

    pub(crate) const fn from_block(
        route: ServerboundRoute,
        action: BlockInteractionAction,
    ) -> Self {
        let disposition = match action {
            BlockInteractionAction::DroppedBeforeClientLoaded => {
                ServerboundDisposition::Gated(ServerboundGate::ClientLoaded)
            }
            _ => ServerboundDisposition::Handled,
        };
        route.outcome(disposition)
    }

    pub(crate) const fn from_player(route: ServerboundRoute, action: PlayerSessionAction) -> Self {
        let disposition = match action {
            PlayerSessionAction::AwaitingRegionTransfer => {
                ServerboundDisposition::Gated(ServerboundGate::RegionTransfer)
            }
            PlayerSessionAction::Movement(MovementOutcome::DisconnectInvalidMovement) => {
                ServerboundDisposition::Rejected(ServerboundRejection::InvalidMovement)
            }
            PlayerSessionAction::Movement(MovementOutcome::DisconnectFlying) => {
                ServerboundDisposition::Rejected(ServerboundRejection::Flying)
            }
            _ => ServerboundDisposition::Handled,
        };
        route.outcome(disposition)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ServerboundRoute {
    packet: &'static str,
    responsibility: ServerboundResponsibility,
    supported: bool,
}

impl ServerboundRoute {
    pub(crate) const fn outcome(
        self,
        disposition: ServerboundDisposition,
    ) -> ServerboundDispatchOutcome {
        ServerboundDispatchOutcome {
            packet: self.packet,
            responsibility: self.responsibility,
            disposition,
        }
    }

    pub(crate) const fn default_outcome(self) -> ServerboundDispatchOutcome {
        self.outcome(if self.supported {
            ServerboundDisposition::Handled
        } else {
            ServerboundDisposition::Unsupported
        })
    }

    pub(crate) const fn is_block(self) -> bool {
        matches!(
            self.responsibility,
            ServerboundResponsibility::BlockInteraction
        )
    }

    pub(crate) const fn is_application_supported(self) -> bool {
        self.supported
            && !matches!(
                self.responsibility,
                ServerboundResponsibility::ProtocolTeleport
                    | ServerboundResponsibility::ProtocolKeepAlive
            )
    }
}

pub(crate) const fn route(packet: &PlayServerboundEntryPacket) -> ServerboundRoute {
    use PlayServerboundEntryPacket as Packet;
    match packet {
        Packet::AcceptTeleportation(_) => supported(
            "AcceptTeleportation",
            ServerboundResponsibility::ProtocolTeleport,
        ),
        Packet::KeepAlive(_) => {
            supported("KeepAlive", ServerboundResponsibility::ProtocolKeepAlive)
        }
        Packet::PickItemFromBlock(_) => block("PickItemFromBlock"),
        Packet::PlayerAction(_) => block("PlayerAction"),
        Packet::Swing(_) => block("Swing"),
        Packet::UseItem(_) => block("UseItem"),
        Packet::UseItemOn(_) => block("UseItemOn"),
        Packet::ChunkBatchReceived(_) => supported(
            "ChunkBatchReceived",
            ServerboundResponsibility::ChunkBatchFeedback,
        ),
        Packet::ClientTickEnd => {
            supported("ClientTickEnd", ServerboundResponsibility::ClientLifecycle)
        }
        Packet::PlayerLoaded => {
            supported("PlayerLoaded", ServerboundResponsibility::ClientLifecycle)
        }
        Packet::MovePlayerPosition(_) => movement("MovePlayerPosition"),
        Packet::MovePlayerPositionRotation(_) => movement("MovePlayerPositionRotation"),
        Packet::MovePlayerRotation(_) => movement("MovePlayerRotation"),
        Packet::MovePlayerStatusOnly(_) => movement("MovePlayerStatusOnly"),
        Packet::ChatAck(_) => unsupported("ChatAck", ServerboundResponsibility::ChatAndCommand),
        Packet::ChatCommand(_) => {
            unsupported("ChatCommand", ServerboundResponsibility::ChatAndCommand)
        }
        Packet::ChatCommandSigned(_) => unsupported(
            "ChatCommandSigned",
            ServerboundResponsibility::ChatAndCommand,
        ),
        Packet::ChatMessage(_) => {
            unsupported("ChatMessage", ServerboundResponsibility::ChatAndCommand)
        }
        Packet::ChatSessionUpdate(_) => unsupported(
            "ChatSessionUpdate",
            ServerboundResponsibility::ChatAndCommand,
        ),
        Packet::CommandSuggestion(_) => unsupported(
            "CommandSuggestion",
            ServerboundResponsibility::ChatAndCommand,
        ),
        Packet::Attack(_) => unsupported("Attack", ServerboundResponsibility::EntityInteraction),
        Packet::Interact(_) => {
            unsupported("Interact", ServerboundResponsibility::EntityInteraction)
        }
        Packet::PickItemFromEntity(_) => unsupported(
            "PickItemFromEntity",
            ServerboundResponsibility::EntityInteraction,
        ),
        Packet::TeleportToEntity(_) => unsupported(
            "TeleportToEntity",
            ServerboundResponsibility::EntityInteraction,
        ),
        Packet::BundleItemSelected(_) => inventory("BundleItemSelected"),
        Packet::ContainerButtonClick(_) => inventory("ContainerButtonClick"),
        Packet::ContainerClick(_) => inventory("ContainerClick"),
        Packet::ContainerClose(_) => inventory("ContainerClose"),
        Packet::ContainerSlotStateChanged(_) => inventory("ContainerSlotStateChanged"),
        Packet::EditBook(_) => inventory("EditBook"),
        Packet::PlaceRecipe(_) => inventory("PlaceRecipe"),
        Packet::RecipeBookChangeSettings(_) => inventory("RecipeBookChangeSettings"),
        Packet::RecipeBookSeenRecipe(_) => inventory("RecipeBookSeenRecipe"),
        Packet::RenameItem(_) => inventory("RenameItem"),
        Packet::SeenAdvancements(_) => inventory("SeenAdvancements"),
        Packet::SelectTrade(_) => inventory("SelectTrade"),
        Packet::SetBeacon(_) => inventory("SetBeacon"),
        Packet::SetCarriedItem(_) => inventory("SetCarriedItem"),
        Packet::SignUpdate(_) => inventory("SignUpdate"),
        Packet::ClientCommand(_) => player_mode("ClientCommand"),
        Packet::ClientInformation(_) => player_mode("ClientInformation"),
        Packet::PlayerAbilities(_) => player_mode("PlayerAbilities"),
        Packet::PlayerCommand(_) => player_mode("PlayerCommand"),
        Packet::PlayerInput(_) => player_mode("PlayerInput"),
        Packet::SpectatorAction(_) => player_mode("SpectatorAction"),
        Packet::Pong(_) => unsupported("Pong", ServerboundResponsibility::Pong),
        Packet::MoveVehicle(_) => {
            unsupported("MoveVehicle", ServerboundResponsibility::VehicleInput)
        }
        Packet::PaddleBoat(_) => unsupported("PaddleBoat", ServerboundResponsibility::VehicleInput),
    }
}

#[must_use]
pub const fn classify_serverbound(
    packet: &PlayServerboundEntryPacket,
) -> ServerboundDispatchOutcome {
    route(packet).default_outcome()
}

const fn supported(
    packet: &'static str,
    responsibility: ServerboundResponsibility,
) -> ServerboundRoute {
    ServerboundRoute {
        packet,
        responsibility,
        supported: true,
    }
}

const fn unsupported(
    packet: &'static str,
    responsibility: ServerboundResponsibility,
) -> ServerboundRoute {
    ServerboundRoute {
        packet,
        responsibility,
        supported: false,
    }
}

const fn block(packet: &'static str) -> ServerboundRoute {
    supported(packet, ServerboundResponsibility::BlockInteraction)
}

const fn movement(packet: &'static str) -> ServerboundRoute {
    supported(packet, ServerboundResponsibility::Movement)
}

const fn inventory(packet: &'static str) -> ServerboundRoute {
    unsupported(packet, ServerboundResponsibility::InventoryAndContainer)
}

const fn player_mode(packet: &'static str) -> ServerboundRoute {
    unsupported(packet, ServerboundResponsibility::PlayerModeAndInput)
}

#[cfg(test)]
mod tests {
    use ferrite_gameplay::player::movement::MovementOutcome;
    use ferrite_protocol::java_26_2::play::serverbound::packet::{
        Hand, MovePlayerStatusOnly, MovementFlags, PlayServerboundEntryPacket, Pong, Swing,
    };

    use super::*;

    #[test]
    fn supported_and_future_packet_families_are_explicit() {
        let handled = classify_serverbound(&PlayServerboundEntryPacket::PlayerLoaded);
        assert_eq!(handled.packet(), "PlayerLoaded");
        assert_eq!(handled.disposition(), ServerboundDisposition::Handled);
        assert_eq!(
            handled.responsibility(),
            ServerboundResponsibility::ClientLifecycle
        );

        let unsupported =
            classify_serverbound(&PlayServerboundEntryPacket::Pong(Pong { payload: 7 }));
        assert_eq!(unsupported.packet(), "Pong");
        assert_eq!(
            unsupported.disposition(),
            ServerboundDisposition::Unsupported
        );
        assert_eq!(
            unsupported.responsibility(),
            ServerboundResponsibility::Pong
        );
    }

    #[test]
    fn dynamic_rejection_and_gate_outcomes_are_not_reported_as_handled() {
        let movement = route(&PlayServerboundEntryPacket::MovePlayerStatusOnly(
            MovePlayerStatusOnly {
                flags: MovementFlags {
                    on_ground: false,
                    horizontal_collision: false,
                },
            },
        ));
        assert_eq!(
            ServerboundDispatchOutcome::from_player(
                movement,
                PlayerSessionAction::Movement(MovementOutcome::DisconnectInvalidMovement),
            )
            .disposition(),
            ServerboundDisposition::Rejected(ServerboundRejection::InvalidMovement)
        );
        assert_eq!(
            ServerboundDispatchOutcome::from_player(
                movement,
                PlayerSessionAction::AwaitingRegionTransfer,
            )
            .disposition(),
            ServerboundDisposition::Gated(ServerboundGate::RegionTransfer)
        );

        let block = route(&PlayServerboundEntryPacket::Swing(Swing {
            hand: Hand::Main,
        }));
        assert_eq!(
            ServerboundDispatchOutcome::from_block(
                block,
                BlockInteractionAction::DroppedBeforeClientLoaded,
            )
            .disposition(),
            ServerboundDisposition::Gated(ServerboundGate::ClientLoaded)
        );
    }
}
