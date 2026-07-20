# C1-C2 Serverbound Play

This page source-specifies the serverbound acknowledgement that closes the initial C1 play
position handshake and the C2 movement, terrain-readiness, chunk-flow-feedback, and liveness
families. Block interaction and C3-C4 gameplay requests remain independently owned by the
completion ledger.

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

## C2 packet inventory

| ID | Identity | Fields in exact order |
|---:|---|---|
| `11` | `minecraft:chunk_batch_received` | desired chunks per tick float |
| `13` | `minecraft:client_tick_end` | no fields |
| `14` | `minecraft:client_information` | client-information record |
| `28` | `minecraft:keep_alive` | signed big-endian challenge long |
| `30` | `minecraft:move_player_pos` | X/Y/Z doubles; movement-flags byte |
| `31` | `minecraft:move_player_pos_rot` | X/Y/Z doubles; yaw/pitch floats; movement-flags byte |
| `32` | `minecraft:move_player_rot` | yaw/pitch floats; movement-flags byte |
| `33` | `minecraft:move_player_status_only` | movement-flags byte |
| `34` | `minecraft:move_vehicle` | X/Y/Z doubles; yaw/pitch floats; on-ground boolean |
| `35` | `minecraft:paddle_boat` | left-paddle boolean; right-paddle boolean |
| `40` | `minecraft:player_abilities` | ability-flags byte |
| `42` | `minecraft:player_command` | entity VarInt; action VarInt; data VarInt |
| `43` | `minecraft:player_input` | input-flags byte |
| `44` | `minecraft:player_loaded` | no fields |
| `45` | `minecraft:pong` | signed big-endian payload int |

The four player-movement forms are distinct packet identities. A missing position or rotation
means retain the server's current value; it is not encoded as zero. Movement flag bit `0` is
on-ground and bit `1` is horizontal collision. Higher bits are decoded and ignored. Vehicle
on-ground is a full boolean byte and vehicle movement has no horizontal-collision field.

Client information is language `UTF(16)`, signed view-distance byte, chat-visibility VarInt,
chat-colors boolean, unsigned model-customization byte, main-hand VarInt, text-filtering boolean,
server-listing boolean, and particle-status VarInt. Chat visibility ordinals are `full=0`,
`system=1`, and `hidden=2`; main hand is `left=0`, `right=1`; particle status is `all=0`,
`decreased=1`, `minimal=2`. Invalid enum ordinals fail by indexed enum lookup. The latest valid
record updates the player's session options; a hat-bit transition broadcasts a player-info
`update_hat` delta.

Ability bit `0x02` alone means flying; every other bit is ignored. Input bits are forward `0x01`,
backward `0x02`, left `0x04`, right `0x08`, jump `0x10`, shift `0x20`, and sprint `0x40`; bit
`0x80` is ignored. Player-command actions are `stop_sleeping=0`, `start_sprinting=1`,
`stop_sprinting=2`, `start_riding_jump=3`, `stop_riding_jump=4`, `open_inventory=5`, and
`start_fall_flying=6`. The leading entity ID is decoded but the locked handler does not compare or
use it; the command always targets the sending player. Only riding-jump uses `data`, and only when
it is positive and the controlled vehicle can jump.

Primary codec anchors are the identically named classes under
`net.minecraft.network.protocol.game`, the common packet classes under
`net.minecraft.network.protocol.common`, `net.minecraft.server.level.ClientInformation`, and
`net.minecraft.world.entity.player.Input`.

## Player movement validation and correction

The vanilla client sends input only when its seven input booleans change. When locally controlled
and not riding, it chooses exactly one movement form per client tick: position changes exceeding
squared distance `(2e-4)^2`, or the 20-tick position reminder, select a position form; changed yaw
or pitch selects a rotation form; otherwise an on-ground or horizontal-collision transition selects
status-only. A riding client sends a rotation packet every player tick and sends vehicle movement
when it owns the root vehicle. It sends `client_tick_end` at the end of every unpaused game tick.

The server applies these rules in order:

1. Any NaN position or non-finite yaw/pitch disconnects with
   `multiplayer.disconnect.invalid_player_movement`. Position infinities are not rejected here;
   X/Z then clamp to `[-30_000_000, 30_000_000]` and Y to
   `[-20_000_000, 20_000_000]`. Rotations wrap in degrees.
2. Won-game input is ignored. Before the client-load gate opens, movement is ignored after the
   invalid-value check. While a teleport challenge is pending, only rotation is installed and
   position processing remains suppressed; the C1 resend lifecycle still applies.
3. A passenger retains its server position, accepts the supplied/fallback rotation, updates chunk
   tracking, and stops. A sleeping player is corrected to its current location when squared
   displacement from the tick baseline exceeds `1.0`, otherwise ignored.
4. While the level tick rate runs normally, the listener counts movement packets. More than five
   since its prior server tick logs a frequency warning and uses a multiplier of one. Unless this
   is the singleplayer owner, a dimension change, disabled `playerMovementCheck`, or fall flight
   with disabled `elytraMovementCheck`, it rejects squared displacement minus squared server
   velocity above `100 * packet_count`, or `300 * packet_count` while fall flying, by issuing a
   fresh ID-72 correction to the current authoritative pose.
5. Otherwise the server moves through collision from its last-good position. An on-ground to
   airborne packet with positive Y delta triggers ground jump. Residual squared displacement above
   `0.0625` is “moved wrongly” except during dimension change, sleep, creative, spectator, or
   post-impulse grace. A newly introduced collision, or moved-wrongly result while the old box was
   collision-free, corrects to the pre-packet pose and records fall damage from that correction.
6. Success snaps to the clamped target, updates chunk tracking, server on-ground/horizontal
   collision state, fall damage, known movement, impulse context, statistics, and last-good
   position. A positive requested Y delta resets fall distance.

The locked residual-Y test uses `y > -0.5 || y < 0.5`, so it always zeroes residual Y before the
`0.0625` moved-wrongly test. This observable implementation defect is part of the 26.2 validator;
Ferrite must not silently replace the `||` with `&&` in its compatibility adapter.

A successful movement with requested vertical delta at least `-0.03125` begins the floating test
only when there was no prior supporting collision, no nearby block below, and no spectator,
server-flight, may-fly, levitation, fall-flight, or spin-attack exemption. Consecutive server ticks
then disconnect for `multiplayer.disconnect.flying` after more than
`ceil(80 * max(0.08 / gravity, 1))` ticks; gravity below `1e-5` disables the limit. Sleep,
passenger, death, a later nonfloating result, or an exemption resets player floating state.

`client_tick_end` closes the client-side input interval: if no accepted player or vehicle movement
called the known-movement path since the preceding tick-end packet, the server sets known movement
to zero; it then clears the interval marker. It does not advance the authoritative server tick.
Movement above squared length `1e-5` resets the idle timer. Player input is retained even before
the load gate opens, but shift state and idle time update only after it opens. Player abilities set
flying only when the server-side `mayfly` capability is true.

Primary anchors are `net.minecraft.client.player.LocalPlayer#sendPosition`,
`net.minecraft.client.Minecraft#tick`, and
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleMovePlayer`, `#tickPlayer`,
`#handleClientTickEnd`, and `#handlePlayerKnownMovement`.

## Vehicle, paddle, and command semantics

Vehicle movement first rejects NaN position or non-finite rotation with
`multiplayer.disconnect.invalid_vehicle_movement`, even while the teleport/load gate is closed.
It otherwise acts only when the player's root vehicle is not the player, is the listener's
tick-tracked vehicle, and is controlled by the player. Position clamps and rotation wrapping match
player movement. Squared displacement from the tick baseline minus squared vehicle velocity above
`100` sends clientbound ID 57 without moving, except for the singleplayer owner. Collision movement
then uses the same always-zero residual-Y defect and `0.0625` residual threshold. A bad residual
from a collision-free old box or a newly introduced collision restores the old position, applies
the supplied rotation, removes the latest movement recording, and sends ID 57. Success snaps to the
target, updates player chunk tracking, known movement, vehicle ground/fall state, player movement
statistics, and the vehicle last-good position.

Vehicle floating uses requested Y delta at least `-0.03125`, absence of prior support/nearby blocks,
server flight disabled, and a vehicle that is neither flying nor gravity-free. It shares the
gravity-scaled timeout above. Losing control/root-vehicle identity resets its floating state.
Paddle input independently updates left/right state only when the controlled vehicle is an
`AbstractBoat`; otherwise it is ignored.

Loaded-state player commands reset idle time, then toggle sprinting, leave sleep (and install a
pending current-position correction), start/stop a capable ride jump, open a vehicle custom
inventory, or try to start fall flight. Failed fall-flight start explicitly stops fall flight.
Commands before the load gate are ignored. An invalid action ordinal fails decode; a handler-level
impossible enum branch is a protocol fault.

Primary anchors are
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleMoveVehicle`,
`#handlePaddleBoat`, `#handlePlayerCommand`, `#handlePlayerInput`, and
`#handlePlayerAbilities`.

## Terrain-ready boundary

Serverbound ID `44`, `minecraft:player_loaded`, is a fieldless C2 packet, not part of the immediate
teleport acknowledgement. The vanilla client sends it only after its level-load tracker has seen
the load-start event and considers terrain ready. The server listener starts with a 60-server-tick
grace timer; death closes the gate until respawn restarts another 60-tick timer. ID 44 idempotently
sets that timer to zero. Timer expiry also opens the gate, so a missing ID 44 delays rather than
permanently deadlocks movement. The packet does not prove a particular chunk coordinate or batch.

Primary anchors are `net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket`,
`net.minecraft.client.multiplayer.ClientPacketListener#notifyPlayerLoaded`, and
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleAcceptPlayerLoad`.

## Chunk-batch feedback

After clientbound chunk-batch finish, the vanilla client sends ID 11 with
`7_000_000 / estimated_nanos_per_chunk`. Its estimator starts at 2,000,000 ns/chunk, clamps each
positive-batch sample to one third through three times the prior aggregate, and weights at most 49
old samples. The server treats the float as flow-control advice, never world state: it decrements
the count of unacknowledged batches without going below zero, maps NaN to `0.01`, clamps every other
value to `[0.01, 64]` (therefore infinities become an endpoint), restores one chunk of quota when no
batch remains unacknowledged, and raises the maximum concurrent unacknowledged batches from one to
ten. Extra feedback is tolerated and merely floors the count at zero.

The sender starts at nine desired chunks/tick, one allowed in-flight batch, and accumulates quota
up to `max(1, desired)`. It emits batch-start, one or more full chunk/light packets, then
batch-finished; ID 11 acknowledges that batch and controls later scheduling. The exact clientbound
batch/chunk grammar remains owned by `PROTO-PLAY-CLIENTBOUND-CHUNK-BLOCK-001`.

Primary anchors are `net.minecraft.client.multiplayer.ChunkBatchSizeCalculator`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleChunkBatchFinished`, and
`net.minecraft.server.network.PlayerChunkSender`.

## Liveness and common echoes

For a non-singleplayer-owner connection, the server sends clientbound ID 44 with the current
millisecond clock as a signed long after at least 15,000 ms. A second 15,000-ms interval with the
challenge still pending disconnects for timeout. Serverbound ID 28 must echo the exact signed long:
a match clears pending state and replaces latency with `(old_latency * 3 + round_trip_ms) / 4`
using integer arithmetic; an unsolicited, stale, or mismatched value disconnects for timeout.
Singleplayer-owner sessions neither originate these challenges nor disconnect on invalid echoes.

The client normally echoes immediately. If rendering is frozen at event polling, it defers the
echo until that condition clears and drops the deferred packet after one minute. Clientbound ID 61
ping is separate diagnostics: the client immediately returns its signed int unchanged in
serverbound ID 45 pong. The base server handler intentionally ignores pong, so it does not satisfy
keepalive or mutate latency.

Primary anchors are `net.minecraft.server.network.ServerCommonPacketListenerImpl`,
`net.minecraft.client.multiplayer.ClientCommonPacketListenerImpl`, and the four common packet
classes.

## Fault boundary

Malformed/truncated primitives, trailing bytes, and invalid enum ordinals fail the packet. The
movement codecs accept every IEEE-754 bit pattern; the listener's explicit NaN/finite behavior is
therefore semantic validation, not codec validation. A valid stale teleport ID is not malformed
and follows the C1 ignore rule. State-transition and handler faults use the play disconnect path
and do not retroactively reopen configuration.

Ferrite maps accepted player/vehicle requests to normalized connection-scoped movement inputs and
authoritative collision moves; liveness, client options, chunk quota, tick boundaries, entity
numbers, packet IDs, raw bitfields, and acknowledgement counters stay inside the version-locked
session adapter. None enters ECS persistence or replay commands as a wire type.
