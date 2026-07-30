# Play Serverbound Entity Session

`G01-P7-F007` implements the six packets in
`PROTO-PLAY-SERVERBOUND-ENTITY-SESSION-001` for Minecraft Java 26.2:

| ID | Identity | Authoritative request |
|---:|---|---|
| 1 | `minecraft:attack` | current-level entity attack |
| 12 | `minecraft:client_command` | respawn, statistics, or game-rule session request |
| 26 | `minecraft:interact` | hand/entity-origin-relative interaction |
| 37 | `minecraft:pick_item_from_entity` | authoritative inventory selection |
| 62 | `minecraft:spectator_action` | nearby spectator camera selection |
| 64 | `minecraft:teleport_to_entity` | UUID-targeted spectator teleport |

All six are legal only in the installed Play codec. None has a prediction sequence, request ID, or
wire acknowledgement.

## Wire boundary

Entity numbers remain signed current-level lookup keys. ID 12 rejects action ordinals outside
`0..=2`; ID 26 maps hand ordinal `1` to off hand and every other signed value to main hand. Boolean
bytes use nonzero-as-true. ID 62 encodes absence as zero and every present ID as wrapping
`entity_id + 1`; decoding performs the inverse wrapping subtraction. ID 64 preserves the UUID as
two big-endian signed-long bit patterns represented by one adapter-local `u128`.

The interaction location uses the locked `LpVec3` grammar. Its zero form is one zero byte. A
nonzero form combines six fixed bytes with an optional unsigned interpretation of a signed VarInt
scale continuation. Decode accepts zero-scale and saturated `32767` fields and always produces
finite components. Canonical encode changes NaN to zero, clamps infinities and finite overflow to
`±17_179_869_183`, selects the compact scale, and uses the zero form below the audited threshold.
Malformed/truncated VarInts and UUIDs, overlong scale continuations, invalid client commands, and
trailing bytes fault the packet.

## Attack and interaction

Attack first requires a loaded nonspectator player. Current-level lookup follows that gate and idle
reset follows lookup even when no target exists. Border and closed custom attack-range checks run
before piercing-weapon rejection. A reached, nonpiercing item, experience orb, self, or
nonattackable abstract arrow causes the explicit invalid-attack disconnect. Feature enablement and
the five-tick-tolerant minimum charge gate precede the normalized combat command.

The attack-range component belongs to the handler-time main-hand stack. Creative players select its
creative endpoints; other living attackers use its mob factor. The default derives its maximum
from the current entity-interaction-range attribute. Hitbox margin and the audited three-block pad
are applied to both closed endpoints.

Interact requires only the loaded gate before lookup. Idle reset and the packet's secondary-action
shift state are retained even when the target is missing or later rejected. Border and strict
padded eye-to-AABB distance precede handler-time hand feature admission. Spectators may open a
target menu provider but otherwise pass.

For ordinary interaction, target behavior runs first. Only a nonconsuming target result permits a
nonempty stack to interact with a living entity. Consuming item interaction installs its returned
stack, restores count for infinite materials, and emits `ENTITY_INTERACT`. Only success records
the criterion: item success uses the pre-action stack while target success uses empty. A
server-swing success publishes one self-inclusive hand animation. Convergence otherwise comes from
ordinary entity and inventory deltas.

## Pick, camera, and teleport

Pick entity deliberately has no loaded, mode, border, or idle-reset gate. It rejects removed and
strict-boundary/far targets, then applies the same enabled exact-stack/hotbar/inventory/infinite
materials selection used by block pick. A valid enabled result publishes held-slot and full menu
convergence even when survival inventory has no match. `include_data` is independent of the pick
result and only prints Avatar profile data for a game-master-authorized sender; it never mutates
the picked stack.

Spectator action requires loaded spectator state and then resets idle. Absence stops there. A
present current-level target must pass border, strict range, removal, and pickable checks. Camera
selection relocates the player first, publishes camera ID 93 second, and resets known position
last.

UUID teleport requires only spectator mode and scans server levels in their installed iteration
order. A nonself camera is reset and may publish self-camera before teleport. Same-level targets use
the ordinary position challenge. Cross-level targets publish respawn keep mask `3`, then position
and level reprojection. Missing UUIDs and nonspectators are silent no-ops; loaded, idle, border,
range, and pickable state do not participate.

## Client commands and ownership

Every valid client command resets idle without a loaded gate. Accepted dead or post-win respawn
replaces the player generation, resets known position, reopens the 60-tick loaded grace, and orders
respawn before position and state reprojection. Post-win retains player data and records the
End-to-Overworld criterion; alive nonwinning players stop after idle reset; hardcore death selects
spectator.

Statistics requests drain the current dirty map and publish a packet even when it is empty.
Game-rule requests publish every current-level rule only with game-master command permission and
otherwise record the denial/log branch. Repeated requests are independent.

`ferrite-protocol::java_26_2::play::serverbound::entity_session` owns packet grammar, compact-vector
quantization, current-level/UUID lookup adaptation, admission gates, and publication ordering.
Authoritative combat, inventory, respawn, entity, and Region ownership remain downstream domain
responsibilities; raw packet IDs, entity numbers, UUID lookup form, hand/action ordinals, and
compact-vector bits do not cross that boundary.

The conformance owner is
`crates/ferrite-protocol/tests/c3/play_serverbound_entity_session.rs`.
