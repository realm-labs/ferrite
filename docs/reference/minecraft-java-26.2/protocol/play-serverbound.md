# C1-C2 Serverbound Play

This page source-specifies the serverbound acknowledgement that closes the initial C1 play
position handshake and the C2 movement, terrain-readiness, chunk-flow-feedback, liveness, and
block-interaction families. C3-C4 gameplay requests remain independently owned by the completion
ledger.

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
batch/chunk grammar remains owned by `PROTO-PLAY-CLIENTBOUND-TERRAIN-001`.

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

## C2 block-interaction packet inventory

| ID | Identity | Fields in exact order |
|---:|---|---|
| `36` | `minecraft:pick_item_from_block` | packed block-position long; include-data boolean |
| `41` | `minecraft:player_action` | action VarInt; packed block-position long; direction unsigned byte; sequence VarInt |
| `63` | `minecraft:swing` | hand VarInt |
| `66` | `minecraft:use_item_on` | hand VarInt; block-hit record; sequence VarInt |
| `67` | `minecraft:use_item` | hand VarInt; sequence VarInt; yaw float; pitch float |

A packed block position stores signed 26-bit X in bits `38..=63`, signed 26-bit Z in bits
`12..=37`, and signed 12-bit Y in bits `0..=11`. Hands are strict indexed enums:
`main_hand=0`, `off_hand=1`. Actions are strict indexed enums:
`start_destroy_block=0`, `abort_destroy_block=1`, `stop_destroy_block=2`, `drop_all_items=3`,
`drop_item=4`, `release_use_item=5`, `swap_item_with_offhand=6`, and `stab=7`.

The player-action direction is different from every other direction field on this page: it is one
unsigned byte mapped by `value % 6` to `down=0`, `up=1`, `north=2`, `south=3`, `west=4`, or
`east=5`, so all 256 byte values decode. The block-hit direction is instead a strict VarInt enum in
that same `0..=5` order. The block-hit body is:

```text
position:packed_block_position_i64
direction:VarInt
offset_x:f32
offset_y:f32
offset_z:f32
inside:boolean
world_border_hit:boolean
```

The offsets are relative to the integer block origin and reconstruct the hit location by float to
double promotion plus the block coordinate. Both booleans remain part of the `BlockHitResult`
passed to block/item behavior; the server does not replace them with a fresh ray cast.

Pick-block, destroy actions, and use-on all pass buffer `1.0` to the same reach test. Let
`R = current block_interaction_range attribute + 1.0`; admission is the squared distance from the
player eye position to the target's unit block AABB **strictly less than** `R * R`. Equality fails.
The syncable attribute has base `4.5` and range `[0,64]`; a creative server player receives a
transient additive `0.5` modifier before the packet-specific buffer. Other attribute modifiers are
already reflected in the current value. Use-in-air and swing have no block reach field or test.

Primary codec anchors are the five identically named classes under
`net.minecraft.network.protocol.game`, `FriendlyByteBuf#readBlockHitResult`, `BlockPos#STREAM_CODEC`,
`Direction#from3DDataValue`, `Player#isWithinBlockInteractionRange`, and
`ServerPlayer#updatePlayerAttributes`.

## Prediction sequence admission

The vanilla client opens a prediction scope for destroy start/stop, use-on-block, and use-in-air.
It pre-increments a wrapping signed int, performs the local prediction, constructs the request with
that sequence, sends it, and then closes the scope. Pick-block and swing have no sequence. The
client's explicit destroy abort packets use the convenience constructor and therefore carry
sequence zero even though they are outside a prediction scope.

The server accepts a sequence only when it is nonnegative. For destroy actions it calls the
authoritative break handler first and registers the sequence after that handler returns. Use-on and
use-in-air register it before item, reach, hit, rotation, cooldown, or action validation. The
listener retains the maximum registered value since its preceding connection tick; at the first
statement of the next tick it sends one clientbound ID `4` ACK and resets the accumulator to `-1`.
It does not remember a global greatest-ever value, so an adversarial later request may elicit a
smaller later ACK. A negative predictive sequence faults at registration. A negative sequence on a
destroy action can therefore fault only **after** its authoritative handler has already mutated or
published corrections; use-on/use-in-air fault before their post-registration work. A negative
sequence on a non-destroy player action is ignored because those branches never register it.

Destroy, use-on, and use-in-air requests are dropped before registration while the 60-tick
client-loaded gate is closed, leaving their client prediction pending. A later accepted sequence
can cumulatively release it. Exact client retention, authoritative-update staging, teleport
interaction, duplicate/stale behavior, and update-versus-ACK order are specified in
[ordering and acknowledgements](ordering-and-acknowledgements.md).

Primary anchors are `MultiPlayerGameMode#startPrediction`, `BlockStatePredictionHandler`, and
`ServerGamePacketListenerImpl#ackBlockChangesUpTo` and `#tick`.

## Destroy and auxiliary player actions

Loaded `start_destroy_block`, `stop_destroy_block`, and `abort_destroy_block` requests reset the
idle timer and route position, action, decoded direction, current level maximum Y, and sequence to
the authoritative breaking state machine. Direction is unused after routing. Admission, progress,
commit, correction, crack publication, and content consequences are exactly
[`BLK-BREAK-001`](../mechanics/blocks/blk-break-001.md); client prediction is exactly
[`PLY-BREAK-001`](../mechanics/player/ply-break-001.md). In particular, ordinary range failure has
no block correction, high-Y and selected permission failures do, every returned handler branch is
still acknowledged, and a successful block mutation normally converges through the later
clientbound update family.

The other loaded player-action branches ignore position, direction, and sequence:

- `swap_item_with_offhand` swaps both hands and stops item use unless spectator;
- `drop_item` and `drop_all_items` drop one or the selected stack unless spectator;
- `release_use_item` calls release even for a spectator;
- `stab` does nothing for a spectator, an item blocked by the five-tick attack gate, or a main-hand
  stack without `piercing_weapon`; otherwise that component attacks from the main-hand slot.

These branches reset idle time before their branch gates and never produce block-change ACKs.
Impossible post-decode action dispatch is a handler fault.

ID 63 has no client-loaded or spectator gate. It resets idle time and starts the selected server
swing. The call publishes only while idle, at/after half the current swing duration, or at negative
swing time; publication excludes the sender because the sender already predicted the animation.
The `ServerPlayer` one-argument override resets attack strength after the call even when animation
publication was suppressed. The strict hand enum rejects every ordinal outside `0..=1`.
`LocalPlayer#swing` sends ID 63 after its local swing call even when that local animation call was
suppressed by the same timing gate.

Primary anchors are `ServerGamePacketListenerImpl#handlePlayerAction`, `#handleAnimate`,
`ServerPlayerGameMode#handleBlockBreakAction`, and `LivingEntity#swing`.

## Pick block

The vanilla client sends ID 36 only when its pick binding has a block hit rather than a miss/entity
hit; `include_data` is exactly whether the control key is held. The send method adds no game-mode,
loaded-state, or second reach check beyond the hit selection.

ID 36 has no client-loaded gate and does not reset the idle timer. It first requires the target
within block-interaction range with padding `1.0`, then an actually loaded server position. It asks
the current state for its clone
stack. `include_data` is effective only when the sender has infinite materials; when effective,
the server saves custom block-entity data without components, removes component-backed values from
that tag, installs typed block-entity data on the stack, and then applies the entity's collected
components. Empty or feature-disabled results stop.

For an enabled result, an exact existing inventory stack selects its hotbar slot or is picked into
the hotbar. With no match, only an infinite-materials player adds and selects the result. The server
then sends the current held-slot projection and broadcasts inventory-menu changes even when a
non-infinite player had no matching stack and therefore changed nothing. Range, unloaded, empty,
and disabled-item exits send neither projection. Position, include-data choice, and inventory
search remain normalized gameplay inputs; no raw packed coordinate or item registry ID is stored.

Primary anchors are `ServerGamePacketListenerImpl#handlePickItemFromBlock`, `#addBlockDataToItem`,
`#tryPickItem`, and `BlockState#getCloneItemStack`.

## Use on block

The vanilla client first sends any changed carried-slot selection and rejects a target outside its
local world border without allocating a sequence. Otherwise it opens a prediction scope, executes
the local block/empty-hand/item precedence, and sends ID 66 with that scope's sequence regardless
of the local interaction result.

After the loaded gate, ID 66 registers its sequence before all remaining checks. A disabled held
item exits without resetting idle time or sending block corrections. The server then requires the
target within block-interaction range with padding `1.0` and every reconstructed hit-location
component to differ from the block center by strictly less than `1.0000001`; NaN and infinities
therefore fail this comparison. Those early exits also send no correction. Only then does it reset
idle time.

Target Y strictly above maximum or below minimum sends the matching build-limit message and exits.
Spawn protection sends its message but continues to the common final corrections. Otherwise the
action runs only with no pending teleport and when `level.mayInteract` succeeds. Failure of either
gate takes the shared `else` and sends the **upper** build-limit message even when height was not
the reason. The accepted action uses the locked block-first, optional main-hand empty interaction,
then held-item precedence; spectator menu behavior, secondary-use suppression, cooldown,
infinite-material restoration, placement transaction, and criteria are exactly
[`BLK-PLACE-001`](../mechanics/blocks/blk-place-001.md) and the relevant item mechanics.

A consuming result triggers `ANY_BLOCK_USE`. A success whose swing source is server invokes
`player.swing(hand,true)`. The locked handler repeats that same server-swing branch in its second
post-result conditional, so it calls swing twice. Each call publishes only when not already
swinging, at/after half the current swing duration, or at negative swing time. When the first call
publishes, it sets swing time to `-1`, guaranteeing that the second publishes too; when an early
active swing suppresses the first, it also suppresses the second. The ordinary idle result is thus
two animations. This duplicate is an observable 26.2 handler defect. The repeated condition also
makes a nonconsuming placement attempt on the upper face at `pos.y>=maxY` send the upper build-limit
message twice; the analogous lower-face attempt at `pos.y<=minY` sends its lower message once.

After every path that reaches the protection/interaction block, the server sends two immediate
authoritative ID-8 updates in order: the hit position, then `hit_position.relative(direction)`.
They are queued before the next-tick ACK. Redirected or multi-position placement targets outside
those two positions converge through ordinary chunk change publication, not these mandatory
corrections.

Primary anchors are `ServerGamePacketListenerImpl#handleUseItemOn`, `#wasBlockPlacementAttempt`,
`ServerPlayerGameMode#useItemOn`, and `InteractionResult.Success#swingSource`.

## Use in air

The vanilla client returns `PASS` without a packet for a spectator. Every other mode first sends
any changed carried-slot selection, then opens a prediction scope and sends ID 67 even when local
cooldown makes the predicted result `PASS`.

After the loaded gate, ID 67 registers its sequence, resets idle time, and exits for an empty or
feature-disabled held stack. It wraps both supplied rotations to `[-180,180)`, compares them with
the current server rotation, and when either differs snaps yaw to the wrapped value and pitch to
`[-90,90]`. Entity setters discard non-finite values rather than disconnecting, so NaN or infinity
in this packet does not share the movement packet's invalid-rotation fault.

The game-mode action passes for spectators and cooldown. Otherwise it runs the held stack's use
transaction. A successful transformed stack replaces the hand; an empty result installs the shared
empty stack. It takes the no-resync fast return only when the result object is the original stack,
count and damage are unchanged, and its resulting use duration is nonpositive. A failed result with
positive duration while the player did not begin using also returns. Otherwise it installs any
transformed/empty hand and, when the player is not continuing item use, sends a full inventory-menu
resynchronization. A server-swing success makes one self-inclusive swing call, subject to the
normal idle/half-duration/negative-time admission above. There is no pending-teleport gate and no
pair of immediate block updates; any world mutations use normal authoritative delta publication.

The vanilla client predicts the same held-item use before sending the packet, including a cooldown
pass and transformed hand. Ferrite maps the request to a normalized hand/use command plus supplied
look intent, while the sequence, floats, raw item/component IDs, and menu wire forms remain in the
26.2 adapter.

Primary anchors are `ServerGamePacketListenerImpl#handleUseItem`,
`ServerPlayerGameMode#useItem`, `MultiPlayerGameMode#useItem`, and `Entity#absSnapRotationTo`.

## Fault boundary

Malformed/truncated primitives, trailing bytes, strict enum ordinals outside their ranges, invalid
predictive sequences, and handler faults fail the packet. The player-action direction byte's modulo
mapping and the block-hit/player-action direction distinction are explicit exceptions, not lenient
enum policy. Movement, use, and hit offsets accept every IEEE-754 bit pattern at codec level; their
different handler checks above are semantic validation. A valid stale teleport ID is not malformed
and follows the C1 ignore rule. State-transition and handler faults use the play disconnect path and
do not retroactively reopen configuration.

Ferrite maps accepted player/vehicle requests to normalized connection-scoped movement inputs and
authoritative collision moves, and block/item requests to normalized authoritative gameplay
commands. Liveness, client options, chunk quota, tick boundaries, entity numbers, packet IDs,
packed coordinates, raw bitfields, and acknowledgement counters stay inside the version-locked
session adapter. None enters ECS persistence or replay commands as a wire type.
