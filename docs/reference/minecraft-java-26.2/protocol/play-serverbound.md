# C1 Serverbound Play Entry

This page source-specifies the serverbound acknowledgement that closes the initial C1 play
position handshake. Full movement, terrain readiness, chunk-flow feedback, liveness, block
interaction, and C3-C4 gameplay requests remain independently owned by the completion ledger.

## Teleport acknowledgement

Play serverbound ID `0`, `minecraft:accept_teleportation`, contains exactly one signed VarInt
challenge. It acknowledges the ID from clientbound ID `72`, `minecraft:player_position`; it is not
a position, tick, transaction sequence, or world identifier.

The locked server owns one `awaitingTeleport` int and nullable `awaitingPositionFromClient` vector:

1. Sending a correction increments the int, wrapping `Integer.MAX_VALUE` to zero, stores the
   authoritative post-correction position, records the current listener tick, and sends ID 72.
2. A response whose ID does not equal the current int is stale and is ignored without clearing the
   pending correction.
3. A matching response with a pending position snaps the player to that stored position, updates
   all three last-good coordinates, completes dimension-change bookkeeping, and clears the pending
   position.
4. A matching response when no position is pending disconnects with
   `multiplayer.disconnect.invalid_player_movement`. Therefore an exact duplicate after a successful
   acknowledgement is a fault, while an arbitrary nonmatching duplicate is ignored.
5. While a position remains pending, movement handling is suppressed. After more than 20 listener
   ticks, the server sends a fresh correction, increments the challenge again, and refreshes the
   resend tick.

The vanilla client applies ID 72, sends ID 0 immediately, then sends serverbound ID `31`,
`minecraft:move_player_pos_rot`. That initial movement echo is three doubles, yaw and pitch floats,
then one byte where bit `0` is on-ground and bit `1` is horizontal collision. If it arrives before
the acknowledgement, the server ignores it because a teleport is pending; after the valid
acknowledgement it enters the C2 movement validator. Ferrite must preserve the two-packet order and
must not use the movement echo as an implicit teleport acknowledgement.

The acknowledgement changes only connection-local synchronization state. Its challenge and stored
wire position never enter ECS components, world persistence, replay commands, or gameplay APIs.

Primary anchors are
`net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleAcceptTeleportPacket`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#updateAwaitingTeleport`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#teleport`, and
`net.minecraft.client.multiplayer.ClientPacketListener#handleMovePlayer`.

## Terrain-ready boundary

Serverbound ID `44`, `minecraft:player_loaded`, is a fieldless C2 packet, not part of the immediate
teleport acknowledgement. The vanilla client sends it only after its level-load tracker has seen
the load-start event and considers terrain ready. It is idempotent server-side and clears the
post-respawn client-load timeout; a fresh connection already has a zero server timeout. Its full
chunk-readiness relationship remains owned by `PROTO-PLAY-SERVERBOUND-MOVEMENT-001`.

Primary anchors are `net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket`,
`net.minecraft.client.multiplayer.ClientPacketListener#notifyPlayerLoaded`, and
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleAcceptPlayerLoad`.

## Fault boundary

A malformed or truncated VarInt fails the packet. A valid stale ID is not a malformed packet and
must follow the ignore rule. State transition faults use the play disconnect path and do not
retroactively reopen configuration. Packet structs, numeric IDs, and acknowledgement counters stay
inside the version-locked session adapter.
