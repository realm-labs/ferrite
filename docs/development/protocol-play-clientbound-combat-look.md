# Play Clientbound Combat and Look Protocol

`G01-P7-F001` implements all four packets in
`PROTO-PLAY-CLIENTBOUND-COMBAT-LOOK-001` for Minecraft Java 26.2:

| ID | Identity | Fields |
|---:|---|---|
| 66 | `minecraft:player_combat_end` | signed duration VarInt |
| 67 | `minecraft:player_combat_enter` | empty |
| 68 | `minecraft:player_combat_kill` | signed player entity VarInt and trusted death component |
| 71 | `minecraft:player_look_at` | from anchor, three doubles, entity flag, optional entity and to anchor |

Feet and eyes are strict anchor ordinals zero and one. Every other signed value faults. The entity
flag accepts any nonzero byte and canonical encoding writes one. Durations and entity IDs retain the
complete signed domain, coordinates retain all IEEE-754 bit patterns, and death text uses the
shared trusted component-NBT boundary. Malformed fields, truncation and trailing bytes fail before
semantic use.

## Combat and death projection

Combat-enter and combat-end are transport-visible but intentionally inert in the locked client.
They do not switch threads, inspect duration, update combat UI or send a response.

Combat-kill resolves the signed ID against the handler-time level and continues only when the
resolved object is the current local player. Missing entities, other entities and numeric-ID reuse
leave current presentation unchanged. When login state permits the death screen, the handler
installs a fresh screen with the supplied message and the current level's hardcore flag. Otherwise
it immediately emits the uncorrelated perform-respawn request and resets toggle keys. Repeated
qualifying packets repeat either branch; there is no generation or duplicate suppression.

Canonical death publication sends the combat-kill packet to the dying connection before later
cleanup. With death messages enabled it retains the resolved component, an exceptional-send
fallback supplied by the normalized death publisher and the separate public broadcast. With the
rule disabled it sends the shared empty component and omits fallback and broadcast. Health,
inventory, statistics and respawn authority remain outside this family.

## Look-at projection

Coordinate form always uses its three packet doubles. Entity form first carries the selected target
anchor coordinates as fallback, then resolves its signed entity ID at handling time. A current
entity replaces the fallback with its current feet or eyes position; a missing entity rotates once
to fallback without creating a later binding.

The local origin is current feet or eyes, with the `f32` eye height widened to `f64`. For deltas
`dx`, `dy`, and `dz`, horizontal distance is `sqrt(dx²+dz²)`. Pitch is Java
`wrapDegrees((float)(-atan2(dy,horizontal)*57.2957763671875))`; yaw is
`wrapDegrees((float)(atan2(dz,dx)*57.2957763671875)-90)`. Current and previous pitch/yaw plus head
yaw are replaced, and living handling aligns current/previous body and previous head rotation.
Coincident, NaN and infinite targets follow the same arithmetic without finite-value rejection.

Server publication first applies authoritative player rotation, then directly sends one tokenless
packet to that player's connection. Entity form includes the target's current local ID and anchor
fallback without distance, dimension or tracking admission.

## Ownership

`ferrite-protocol::java_26_2::play::clientbound::combat_look` separates:

- `packet`: stable version-local packet values and strict anchor mapping;
- `codec`: exact field-order decoding and encoding;
- `projection`: handler-time entity lookup, death UI/respawn behavior and Java rotations;
- `publication`: direct combat/death/look packet construction and exceptional death fallback.

Raw entity IDs, fallback coordinates, GUI objects, prior rotations and packet order remain
version-local adapter state rather than durable gameplay identity.

## Evidence

`crates/ferrite-protocol/tests/c3/play_clientbound_combat_look.rs` owns four exact packet goldens,
signed/IEEE/enum/boolean/malformed codec boundaries, inert lifecycle handling, death object and
screen gates, repeated immediate respawn, coordinate and handler-time entity look resolution,
nonfinite arithmetic, publication branches and end-to-end decode-to-projection behavior.
