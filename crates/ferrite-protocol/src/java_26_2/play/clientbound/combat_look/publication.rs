use crate::java_26_2::play::clientbound::combat_look::packet::{
    EntityAnchor, LookEntity, LookPosition, PlayerCombatEnd, PlayerCombatKill, PlayerLookAt,
};
use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::value::nbt::TextComponentNbt;

#[must_use]
pub const fn publish_combat_enter() -> PlayClientboundPacket {
    PlayClientboundPacket::PlayerCombatEnter
}

#[must_use]
pub const fn publish_combat_end(duration: i32) -> PlayClientboundPacket {
    PlayClientboundPacket::PlayerCombatEnd(PlayerCombatEnd { duration })
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeathPublication {
    pub primary: PlayClientboundPacket,
    pub fallback: Option<PlayClientboundPacket>,
    pub broadcast_public_message: bool,
}

#[must_use]
pub fn publish_death(
    player_entity_id: i32,
    death_message: TextComponentNbt,
    exceptional_fallback: TextComponentNbt,
    show_death_messages: bool,
) -> DeathPublication {
    let selected = if show_death_messages {
        death_message
    } else {
        TextComponentNbt::literal("").expect("empty literal component is valid")
    };
    DeathPublication {
        primary: PlayClientboundPacket::PlayerCombatKill(PlayerCombatKill {
            player_entity_id,
            message: selected,
        }),
        fallback: show_death_messages.then_some({
            PlayClientboundPacket::PlayerCombatKill(PlayerCombatKill {
                player_entity_id,
                message: exceptional_fallback,
            })
        }),
        broadcast_public_message: show_death_messages,
    }
}

#[must_use]
pub const fn publish_coordinate_look(
    from_anchor: EntityAnchor,
    target: LookPosition,
) -> PlayClientboundPacket {
    PlayClientboundPacket::PlayerLookAt(PlayerLookAt {
        from_anchor,
        fallback: target,
        entity: None,
    })
}

#[must_use]
pub const fn publish_entity_look(
    from_anchor: EntityAnchor,
    target_entity_id: i32,
    target_anchor: EntityAnchor,
    fallback: LookPosition,
) -> PlayClientboundPacket {
    PlayClientboundPacket::PlayerLookAt(PlayerLookAt {
        from_anchor,
        fallback,
        entity: Some(LookEntity {
            entity_id: target_entity_id,
            anchor: target_anchor,
        }),
    })
}
