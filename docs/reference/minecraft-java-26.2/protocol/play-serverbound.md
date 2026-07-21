# C1-C3 Serverbound Play

This page source-specifies the serverbound acknowledgement that closes the initial C1 play
position handshake, the C2 movement, terrain-readiness, chunk-flow-feedback, liveness, and
block-interaction families, and the C3 container, entity-session, sign-update, and recipe-book
requests. Other C3-C4 gameplay requests remain independently owned by the completion ledger.

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

# C3 Container Input and Convergence

This slice specifies the five serverbound packets that mutate or close the current menu and select
the carried hotbar slot. Container IDs use the ordinary signed VarInt codec; the name
`CONTAINER_ID` adds no unsigned range or byte-width restriction.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `17` | `minecraft:container_button_click` | container ID VarInt; button ID VarInt |
| `18` | `minecraft:container_click` | container ID VarInt; state ID VarInt; slot signed big-endian short; button signed byte; input VarInt; changed-slot map; carried hashed stack |
| `19` | `minecraft:container_close` | container ID VarInt |
| `20` | `minecraft:container_slot_state_changed` | slot ID VarInt; container ID VarInt; new-state boolean |
| `53` | `minecraft:set_carried_item` | selected slot signed big-endian short |

The input IDs are `0=PICKUP`, `1=QUICK_MOVE`, `2=SWAP`, `3=CLONE`, `4=THROW`,
`5=QUICK_CRAFT`, and `6=PICKUP_ALL`; every other signed VarInt maps to `PICKUP` by the enum's zero
fallback. The signed short and byte remain widened signed values in the handler. The vanilla client
checked-casts its click slot and button, so its local call throws rather than emitting values
outside those widths. Exact seven-input cursor/slot behavior, sentinels and quick-craft phases are
source-specified by `ITM-CONTAINER-CLICK-001`.

## Hashed predicted stacks

The changed-slot map begins with a VarInt count restricted above to 128. Every entry is a signed
big-endian short slot key and a `HashedStack`; duplicate keys replace the earlier map value. A
negative count fails allocation. A hashed stack begins with a boolean: false is empty; true is a
strict `minecraft:item` holder VarInt, count VarInt and hashed component patch. The patch contains:

1. an added-component map count restricted above to 256, then strict
   `minecraft:data_component_type` VarInt and big-endian signed 32-bit hash pairs;
2. a removed-component set count restricted above to 256, then strict component-type VarInts.

Duplicate added types replace earlier hashes; duplicate removed types collapse in the set. Negative
counts fail collection allocation. A present hash matches only an authoritative stack with exactly
the same count, item holder, removed-type set and number of added components, and with every added
typed component producing the transmitted 32-bit value. Both peers serialize each typed component
value through its registry-aware data codec into `HashOps.CRC32C_INSTANCE`, then use
`HashCode.asInt()`. The client keeps one generator over the received registries. The server uses the
same operation and a per-player loading cache of at most 256 complete typed components. This is a
32-bit comparison: a CRC32C collision is accepted as a match.

These hashes are prediction evidence only. Receiving one clears the server's exact remote snapshot
for that slot/cursor and stores the hash. At the next comparison, a match promotes a copy of the
current authoritative stack into that snapshot; a mismatch emits the normal correction and
replaces the snapshot. The hash never writes an authoritative item or component.

Primary codec anchors are `ServerboundContainerButtonClickPacket#STREAM_CODEC`,
`ServerboundContainerClickPacket#STREAM_CODEC`, `ServerboundContainerClosePacket#STREAM_CODEC`,
`ServerboundContainerSlotStateChangedPacket#STREAM_CODEC`,
`ServerboundSetCarriedItemPacket#STREAM_CODEC`, `HashedStack#STREAM_CODEC`,
`HashedPatchMap#STREAM_CODEC`, `HashOps#CRC32C_INSTANCE`, and `RemoteSlot.Synchronized`.

## Client prediction and authoritative click

Before ID 18, the client requires the supplied container ID to equal its current menu ID. It copies
every slot, performs the complete click locally, compares every before/after stack by count, item and
components, and hashes only changed indices plus the resulting cursor. It sends the current menu
state ID after prediction. There is no click sequence or separate acknowledgement.

The server resets idle time before testing the packet container ID. A mismatch stops. Spectator or
dead/dying players receive a full current snapshot and no click. Otherwise the menu must pass
`stillValid`. The slot admission helper accepts `-1`, `-999`, and every integer below the slot-list
size; consequently other negative signed-short values pass this outer check and are owned by the
selected click branch, while values at or above the size are logged and ignored.
There is no client-loaded or pending-teleport gate.

For an admitted click, the server records whether the packet state ID differs, suppresses remote
updates, executes the source-specified click authoritatively, installs every in-range client hash
and ignores/logs out-of-range changed-slot keys, installs the cursor hash, then resumes updates. A
stale state ID does **not** reject or roll back the click: it causes `broadcastFullState`, producing
one complete content/cursor snapshot and every menu data value. A matching state ID calls
`broadcastChanges`; matching hashes suppress corrections, while mismatches produce authoritative
slot/cursor updates. Hash map iteration order affects only remote-snapshot installation, not the
authoritative click.

## Buttons, crafter state, close and carried slot

ID 17 resets idle, then requires the current container ID, nonspectator state and `stillValid`. It
calls the concrete menu button and broadcasts deltas only when that call returns true. Exact
enchantment, loom, stonecutter, lectern and other menu controls are in
`ITM-CONTAINER-CONTROL-001`; the packet itself does not carry a state ID.

ID 20 has no idle reset or validity check. It requires nonspectator state and the current container
ID, then only acts on a `CrafterMenu` backed by a real `CrafterBlockEntity`. The block entity changes
only an empty slot `0..=8`; true stores enabled value zero, false stores disabled value one, and a
successful request dirties it. Every other menu/backing/slot/nonempty branch is ignored.

ID 19 ignores its decoded container ID and does not reset idle. It removes the **current** menu,
transfers its shared inventory-menu remote state, and selects the inventory menu without sending a
clientbound close response. A delayed close for an old menu can therefore close a newly opened
current menu. Canonical server-initiated close instead sends clientbound ID 17 for the current ID
before doing the same removal.

ID 53 accepts only slots `0..=8`; invalid signed shorts warn and stop without resetting idle. A
change away from the selected slot stops active main-hand use, then installs the selection. Every
valid request, including the already-selected slot, resets idle. Inventory/equipment dirty
projection provides convergence; this packet has no direct ACK.

Primary handler anchors are `MultiPlayerGameMode#handleContainerInput`,
`ServerGamePacketListenerImpl#handleContainerClick`, `#handleContainerButtonClick`,
`#handleContainerClose`, `#handleContainerSlotStateChanged`, `#handleSetCarriedItem`,
`AbstractContainerMenu#setRemoteSlotUnsafe`, `#broadcastChanges`, `#broadcastFullState`,
`CrafterBlockEntity#setSlotState`, and `ServerPlayer#doCloseContainer`.

Malformed/truncated fields, residual bytes, overlong VarInts, strict item/component holder failures,
oversized or negative collections and truncated component hashes fail the play packet. Invalid
input ordinals deliberately become pickup. Wrong container IDs, invalid menu state, rejected slot
indices, spectator/dead branches and stale state IDs follow the semantic paths above rather than a
decode fault. Ferrite maps admitted requests to normalized menu, click and selection commands;
container/state IDs, signed slot/button widths, raw input/registry IDs, hashes and remote snapshots
remain connection-adapter state and never enter ECS persistence.

# C3 Entity Interaction and Session Requests

The first C3 serverbound slice contains six packets. They are legal only under the installed play
codec and have no prediction sequence or acknowledgement domain.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `1` | `minecraft:attack` | target entity ID VarInt |
| `12` | `minecraft:client_command` | action VarInt enum: `0=perform_respawn`, `1=request_stats`, `2=request_gamerule_values` |
| `26` | `minecraft:interact` | target entity ID VarInt; hand VarInt; low-precision relative hit vector; secondary-action boolean |
| `37` | `minecraft:pick_item_from_entity` | target entity ID VarInt; include-data boolean |
| `62` | `minecraft:spectator_action` | optional target entity ID as zero for absent or `entity_id + 1` otherwise |
| `64` | `minecraft:teleport_to_entity` | UUID as two big-endian signed 64-bit words |

The ID-12 enum is read by direct array index and rejects every ordinal outside `0..=2`. ID 26's
`InteractionHand.STREAM_CODEC` is deliberately different: `0` is main hand, `1` is off hand, and
every other signed VarInt maps to **main hand** through the enum's zero fallback. Booleans use the
common nonzero-is-true byte decoder. Optional target zero is the sole absent form; every other
signed VarInt is decremented with wrapping int arithmetic and remains present, including negative
wire values. Entity IDs and UUIDs are connection/world lookup keys, never durable Ferrite entity
identities.

## Low-precision entity hit vector

The interact vector is the locked `LpVec3` grammar, not three floats or doubles. A canonical
all-zero vector is one zero byte. Otherwise it is six fixed bytes followed conditionally by a
VarInt:

```text
lowest:u8
if lowest == 0 { vector = (0, 0, 0); stop }
middle:u8
highest:u32
packed = highest << 16 | middle << 8 | lowest
scale = lowest & 3
if lowest & 4 { scale |= (VarInt_as_unsigned_u32 << 2) }
x = unpack15(packed >> 3)  * scale
y = unpack15(packed >> 18) * scale
z = unpack15(packed >> 33) * scale
unpack15(v) = min(v & 0x7fff, 32766) * 2 / 32766 - 1
```

The canonical writer changes NaN to zero, clamps each component to
`[-17_179_869_183, 17_179_869_183]`, and emits the one-byte zero form when the largest absolute
component is below `3.051944088384301e-5`. Otherwise its unsigned scale is the ceiling of that
largest component, the three normalized values are rounded into 15-bit fields, and a scale above
three uses the continuation VarInt. Decode accepts noncanonical 15-bit `32767` as the same endpoint
as `32766`, accepts zero-scale nonzero forms, and always produces finite values. Truncation or an
overlong continuation VarInt faults. The vanilla client supplies the ray hit position minus the
target entity's current X/Y/Z, so the vector is entity-origin-relative.

Primary codec anchors are `ServerboundAttackPacket`, `ServerboundClientCommandPacket`,
`ServerboundInteractPacket`, `ServerboundPickItemFromEntityPacket`,
`ServerboundSpectatorActionPacket`, `ServerboundTeleportToEntityPacket`, `InteractionHand`, and
`LpVec3`.

## Attack admission

The locked client converges a changed carried slot first, sends ID 1, applies its local attack, and
resets local attack strength. The ordinary click path has already rejected a disabled held item or
insufficient current charge; a piercing weapon instead uses its separate local piercing path and
does not send ID 1. These client gates are not trusted server admission.

The server first requires `hasClientLoaded` and a nonspectator sender. It looks up the target or
entity part in the current level, then resets idle time even if the lookup failed. A present target
must have its block position inside the world border. Reach uses the current main-hand
`minecraft:attack_range` component or a default derived from the sender's current
`minecraft:entity_interaction_range` attribute. For eye-to-target-AABB distance `d`, the accepted
closed interval is:

```text
d >= effective_min_reach - hitbox_margin - 3
d <= effective_max_reach + hitbox_margin + 3
```

Player creative mode selects the component's creative minimum/maximum; other living attackers use
its mob factor. The default component has minimum zero, maximum equal to current entity-interaction
range, zero margin, and factor one. The base attribute is `3.0`, is constrained to `0..=64`, and a
creative `+2.0` transient modifier is already in its current value. Unlike ordinary entity
interaction's strict squared test, the attack maximum is inclusive.

A main-hand piercing-weapon component rejects this ordinary attack before invalid-target
classification. If reach and that gate pass, attacking an item entity, experience orb, self, or a
nonattackable abstract arrow disconnects with `multiplayer.disconnect.invalid_entity_attacked`.
Missing/out-of-border/out-of-range and piercing targets are only ignored. The held item must still
be feature-enabled, and its minimum-attack-charge test uses an optimistic five-tick tolerance.
Only then does `Player#attack` execute the source-specified combat/damage rules. Raw entity ID,
reach component encoding and disconnect packet form remain adapter concerns.

Primary anchors are `ServerGamePacketListenerImpl#handleAttack`, `Player#isWithinAttackRange`,
`LivingEntity#getAttackRangeWith`, `AttackRange#isInRange`, and `Player#cannotAttackWithItem`.

## Entity interaction

ID 26 first requires the client-loaded gate, then looks up a current-level entity or part, resets
idle, and copies the packet's secondary-action boolean into the authoritative player's shift flag.
Those two player mutations happen even for a missing or rejected target. A present target must be
inside the world border and satisfy the strict eye-to-AABB condition
`distance_squared < (current_entity_interaction_range + 3)^2`. The held stack selected by the
decoded hand must be feature-enabled.

The handler copies that stack and calls `Player#interactOn(target, hand, relative_location)`. A
spectator may open a target `MenuProvider` but returns pass. Otherwise target interaction runs
first; only a nonconsuming target result permits a nonempty held stack to run
`interactLivingEntity` on a living target. Infinite materials restore the documented stack count;
a consuming item interaction emits `ENTITY_INTERACT` and installs empty when appropriate.

Only an `InteractionResult.Success` triggers the player-interacted-with-entity criterion. Its
criterion stack is the pre-action copy for item interactions and empty for target interactions. A
success selecting server swing publishes one self-inclusive hand animation. There is no sequence,
mandatory inventory resync, target correction, or explicit spectator rejection at the listener;
downstream interaction result and ordinary authoritative entity/inventory deltas provide
convergence. The vanilla client performs the same interaction locally after send (but returns pass
locally for spectator), and its input path sends a client swing only when the result selects client
swing.

Primary anchors are `MultiPlayerGameMode#interact`, `Minecraft#startUseItem`,
`ServerGamePacketListenerImpl#handleInteract`, and `Player#interactOn`.

## Pick entity

ID 37 has no client-loaded, game-mode, world-border, or idle-reset gate. It resolves an entity or
part in the current level and requires `Player#isWithinEntityInteractionRange(entity,3)`, which
rejects removed entities and uses the same strict padded AABB formula. A nonempty target pick
result enters the same feature, exact-stack, hotbar-selection, infinite-material add, held-slot,
and inventory-menu convergence path as block pick. Consequently a valid enabled result publishes
held-slot/menu state even if a survival inventory had no match and did not change.

`include_data` does **not** attach entity state to the picked item. Independently of whether the
target produced an item, a true flag from a sender allowed to use game-master blocks prints profile
data only when the target is an `Avatar`. The ordinary vanilla pick binding sends the packet from
an entity hit and sets the flag from the control key; the server still owns all target, reach,
permission, feature and inventory decisions.

Primary anchors are `ServerGamePacketListenerImpl#handlePickItemFromEntity`, `#tryPickItem`,
`MultiPlayerGameMode#handlePickItemFromEntity`, and `Entity#getPickResult`.

## Spectator camera and UUID teleport

ID 62 requires both the client-loaded gate and spectator mode, then resets idle. An absent optional
ID stops there. A present target is looked up in the current level (including entity parts), must
be inside the world border, strictly within the padded interaction range above, and pickable.
`ServerPlayer#setCamera` then relocates the server player to the target before publishing
clientbound ID 93 and resetting known position. The client's spectator left-click dispatch uses
the present form for an entity hit and the absent form for a block hit or miss. The absent form is
a no-op and does **not** leave the current camera; it is acknowledged only in the colloquial sense
that the request was processed—there is no wire ACK.

ID 64 instead requires only spectator mode. It scans every server level in iteration order for the
UUID and teleports to the first match's exact position and rotation with camera reset. It has no
client-loaded, idle, world-border, reach, or pickable check. If already viewing another camera,
reset can publish self-camera before teleport. Same-dimension teleport uses the ordinary position
challenge; cross-dimension teleport uses clientbound respawn with keep mask `3` before its position
and level reprojection. A missing UUID or nonspectator sender is silently ignored. This packet is
the spectator player-menu teleport, not a camera-by-UUID request.

Primary anchors are `ServerGamePacketListenerImpl#handleSpectatorAction`,
`#handleTeleportToEntityPacket`, `ServerPlayer#setCamera`, `ServerPlayer#teleportTo`,
`MultiPlayerGameMode#spectate`, and `PlayerMenuItem#selectItem`.

## Client command

Every valid ID-12 action resets idle without a client-loaded gate:

- `perform_respawn`: a player who won the game clears that flag, respawns with retained player data,
  restarts the 60-tick client-load grace, and triggers End-to-Overworld dimension criteria. A
  nonwinning player with positive health is ignored after the idle reset. A dead player respawns
  without retained data; hardcore then forces spectator mode. Both accepted branches replace the
  connection's `ServerPlayer`, reset known position, and begin the clientbound respawn flow.
- `request_stats`: drains the server stats counter's current dirty set into one clientbound
  `award_stats` packet, including an empty map, and clears that dirty set. New placement initially
  marks every stored stat dirty.
- `request_gamerule_values`: without game-master command permission, logs and sends nothing. With
  permission, serializes every available current-level game rule by namespaced key and sends one
  clientbound `game_rule_values` map. That response remains in its C4 optional family.

The vanilla death/win UI sends the respawn action; the stats screen and in-world game-rule screen
send their corresponding requests. None carries a request ID, so repeated requests are independent
and responses correlate only by client UI/session state.

Primary anchors are `ServerGamePacketListenerImpl#handleClientCommand`, `#sendGameRuleValues`,
`PlayerList#respawn`, `ServerStatsCounter#sendStats`, `LocalPlayer#respawn`, `StatsScreen`, and
`InWorldGameRulesScreen`.

## C3 ingress fault boundary

Malformed/truncated VarInts or UUIDs, overlong compact-vector scale, trailing bytes, and invalid
ID-12 ordinals fault the play packet. Invalid ID-26 hand ordinals instead map to main hand by
design; noncanonical finite compact vectors, negative/missing entity IDs, an absent spectator
target, failed permissions and stale targets follow their semantic ignore/no-op branches. The
explicit invalid-attack target disconnect is a semantic policy response, not a decode fault.

# C3 Sign Text Submission

Play serverbound ID `61`, `minecraft:sign_update`, has the exact grammar:

```text
position:packed_block_position_i64
front_text:boolean
lines[4]:UTF (server decode bound 384)
```

Exactly four strings are present with no count. The private server decoder calls `readUtf(384)`, so
each received field permits at most 384 decoded Java UTF-16 code units and at most 1,152 encoded
UTF-8 bytes. Its official member encoder asymmetrically calls default-bound `writeUtf`, whose
per-string limits are 32,767 UTF-16 units and 98,301 bytes; it can therefore produce bytes that the
same packet decoder rejects. The vanilla editor applies only its rendered-width predicate and does
not independently enforce 384 code units. Malformed UTF-8 byte sequences decode with replacement
characters rather than faulting; a negative/over-limit encoded length, truncation, a decoded length
above 384, or trailing packet bytes faults. Packed position uses signed 26-bit X, signed 26-bit Z
and signed 12-bit Y; the side boolean uses zero-false/nonzero-true. The decoder does not validate
sign existence, edit authority, distance, wax state, rendered width or line semantics.

The vanilla sign editor sends the packet from its `removed()` callback, not directly from the Done
button. Any normal screen exit therefore submits the current four strings once when a connection
exists, including Done, Escape, invalid local distance/entity state, and screen replacement. The UI
constrains edits by the sign's rendered line width; a nonvanilla sender can still use any codec-valid
line. The position and front/back flag are copied from the editor's original activation state.

Primary codec/client anchors are `ServerboundSignUpdatePacket#STREAM_CODEC` and
`AbstractSignEditScreen#removed`.

## Filtering, authorization, and world convergence

The listener first strips legacy `ChatFormatting` codes from all four strings, preserving their
order, then submits the resulting list to the player's asynchronous text filter. Only after that
future completes does the server executor reset player idle time and inspect the player's then
current level. The completion-time state, not receipt-time state, decides acceptance:

1. an unloaded position is ignored;
2. a missing or non-sign block entity is ignored;
3. a sign accepts only when it is not waxed, has a level, and its stored allowed-editor UUID equals
   the sender's UUID;
4. every other submission logs and returns without changing text or clearing the stored editor;
5. a successful submission replaces exactly the selected front/back side with a newly constructed
   `SignText`, so `setText` marks the entity changed and calls `sendBlockUpdated` with flags `3`;
6. it then clears the allowed editor UUID and unconditionally calls `sendBlockUpdated` with flags
   `3` again. Even semantically unchanged text therefore reaches both calls.

The sign block-entity tick clears a stored editor when that player is absent or no longer within
the block-interaction range padded by `4.0`; the vanilla client independently closes its editor
using the same predicate. The submission handler itself adds no direct distance or player-build
check, so authorization at async completion is the decisive gate.

For each line the server retains the prior selected presentation's `Style`. With player text
filtering enabled it stores the filtered-or-empty literal as the single displayed message. Without
filtering it stores both the raw and filtered-or-empty literal forms. The two accepted update calls
then feed ordinary block-entity synchronization to converge viewers; there is no
direct response, submission ID, menu state, replay protection or corrective packet for rejection.
Concurrent wax, side, block-entity, player-level, range-tick and allowed-editor changes during
filtering take effect before commit.

Primary server anchors are `ServerGamePacketListenerImpl#handleSignUpdate/#updateSignText`,
`ChatFormatting#stripFormatting`, `SignBlockEntity#updateSignText/#setMessages/#tick`, and
`Level#sendBlockUpdated`.

## Sign-update normalized boundary

Ferrite accepts this as a connection-local request against a currently authorized namespaced sign
entity and side, then projects accepted normalized literal text through ordinary world mutation and
block-entity convergence. Packed coordinates and the side selector are decoded adapter inputs;
allowed-editor UUIDs, filter futures, raw/filtered dual forms and packet IDs remain transaction and
projection state rather than persistent gameplay identity.

# C3 Recipe-Book Requests and Placement

The three requests operate on the server's current feature-filtered recipe display map and the
player's authoritative recipe book. A recipe display ID is a signed VarInt index local to that map,
not a registry ID or namespaced recipe key.

| ID | Identity | Fields in exact order |
|---:|---|---|
| `39` | `minecraft:place_recipe` | container ID signed VarInt; recipe-display ID signed VarInt; use-maximum-items boolean |
| `46` | `minecraft:recipe_book_change_settings` | recipe-book-type ordinal signed VarInt; open boolean; filtering boolean |
| `47` | `minecraft:recipe_book_seen_recipe` | recipe-display ID signed VarInt |

Recipe-book type is strict indexed-enum decode: `0=crafting`, `1=furnace`, `2=blast_furnace`, and
`3=smoker`; every other signed VarInt faults. Booleans use the common zero-false/nonzero-true rule.
Container and display IDs have no codec range restriction. Negative or out-of-current-list display
IDs decode successfully but later resolve to no recipe.

Primary codec anchors are `ServerboundPlaceRecipePacket#STREAM_CODEC`,
`ServerboundRecipeBookChangeSettingsPacket#STREAM_CODEC`, `FriendlyByteBuf#readEnum`,
`ServerboundRecipeBookSeenRecipePacket#STREAM_CODEC`, and `RecipeDisplayId#STREAM_CODEC`.

## Display-to-parent mapping and recipe publication

`RecipeManager#unpackRecipeInfo` assigns contiguous display IDs from zero while walking the loaded
recipes and each recipe's display list, skipping feature-disabled displays. Each valid index maps
to a `ServerDisplayInfo` containing the complete display entry and its namespaced parent recipe.
One parent may own several display IDs; reload reconstructs all indices. Negative and out-of-range
indices return no mapping. The detailed entry/group/category/placement mapping is specified in the
clientbound recipe-book section.

Adding a previously unknown nonspecial parent recipe stores its namespaced key as known and
highlighted, resolves all current displays and publishes them in ID 74 with `replace=false` when
the display list is nonempty. Removing a known parent clears both parent-key sets and publishes all
resolved display IDs in ID 75 when nonempty. Initial join first publishes ID 76 settings and then
ID 74 with `replace=true`, even when its entry list is empty. Server persistence stores known and
highlighted parent keys plus book settings, never display IDs.

Primary anchors are `RecipeManager#unpackRecipeInfo/#getRecipeFromDisplay`,
`RecipeManager.ServerDisplayInfo`, and `ServerRecipeBook#addRecipes/#removeRecipes/#sendInitialRecipeBook/#pack`.

## Place-recipe admission

ID 39 returns to the main server thread, then resets the player's last-action time before any
semantic gate. It silently ignores the request when:

1. the player is a spectator or the packet container ID differs from the exact current menu ID;
2. the current menu is no longer valid for the player (with a debug log);
3. the display ID has no current mapping;
4. the mapped parent recipe is not known/unlocked in this player's server recipe book;
5. the current menu does not implement `RecipeBookMenu`; or
6. the parent recipe's placement information is impossible to place (with a debug log).

Only after all gates does the server call the current recipe menu's `handlePlacement`, passing the
packet's use-maximum flag, current creative-mode flag, mapped parent recipe, current level and
player inventory. There is no request state ID and no validation that the display variant itself
matches a selected client tab; authority is through the mapped parent recipe and current menu.

Crafting menus bracket placement with `beginPlacingRecipe`/`finishPlacingRecipe`, use the complete
crafting grid as input and clear targets, and run `finish` even on exceptional exit. Furnace menus
use the single ingredient slot as the input grid while treating both ingredient and result slots
as clear-capacity targets. These are the locked vanilla placement implementations.

Primary anchors are `ServerGamePacketListenerImpl#handlePlaceRecipe`, `RecipeBookMenu`,
`AbstractCraftingMenu#handlePlacement`, and `AbstractFurnaceMenu#handlePlacement`.

## Placement mutation and ghost branch

For a noncreative player, placement first proves that every current clear-target stack can be
returned to existing compatible inventory stacks or ordinary free inventory slots. Failure
returns `NOTHING` without clearing or publishing a ghost. Creative mode skips only this capacity
test; it does not conjure missing recipe ingredients. The helper then aggregates player inventory
and current craft inputs into `StackedItemContents`.

When the aggregate cannot craft the recipe, the helper returns every clear-target stack to the
inventory, writes any remainder back to its slot, clears crafting content, marks inventory changed,
and returns `PLACE_GHOST_RECIPE`. The listener immediately sends clientbound ID 63 with the current
menu ID and the valid display payload from the request's mapping. This is the only direct response
branch.

When the aggregate can craft, the request returns `NOTHING` whether it mutates or takes a later
guard return. If the grid already matches, any nonempty input whose `count + 1` exceeds the lesser
of biggest-craftable count and that stack's maximum aborts without mutation. Otherwise the target
amount is biggest-craftable for `use_maximum=true`, the minimum nonempty current input count plus
one for an already matching grid, or one for a nonmatching grid. The helper resolves ingredient
item holders for that amount, clamps it to the minimum holder `max_stack_size` component (fallback
one), recomputes holders when clamped, clears the grid, then distributes shaped or shapeless slots
through the recipe's placement map. Each slot move removes matching items from inventory and
installs or grows the grid stack. Inventory is marked changed after this craftable path, including
an early inner placement guard.

Placement has no explicit success packet, full resync, or state acknowledgement. Ordinary menu
change detection later publishes authoritative slot/cursor/data deltas using the already specified
container rules. On the ghost branch the immediate ID 63 can precede those later deltas that show
the cleared grid and returned inventory.

Primary anchors are `ServerPlaceRecipe#placeRecipe/#testClearGrid/#tryPlaceRecipe/#placeRecipe`,
`#calculateAmountToCraft/#clampToMaxStackSize/#clearGrid/#addItemToSlot`, `StackedItemContents`,
`PlaceRecipeHelper`, and `Inventory#placeItemBackInInventory/#findSlotMatchingCraftingIngredient`.

## Settings and seen-highlight requests

ID 46 has no client-loaded, idle-reset, menu, mode or recipe gate. After main-thread dispatch it
directly replaces the server recipe book's open and filtering booleans for the decoded strict book
type. The server sends no echo. The vanilla recipe-book UI first changes its local setting and,
when a connection exists, sends the current type/open/filtering tuple. A later initial-session or
other explicit settings projection can replace client settings independently.

ID 47 likewise has no client-loaded, idle-reset, menu or mode gate. It resolves the display ID in
the current recipe manager; invalid indices are silent no-ops. A valid index removes the mapped
parent recipe key from the server highlight set even when several displays share that parent. The
vanilla client sends this only when the exact display ID is currently highlighted: it first removes
that one local display highlight, then sends ID 47. Thus one request immediately clears one client
display but clears the shared parent highlight server-side; later server projections no longer
highlight any display of that parent. No echo or acknowledgement follows.

Primary anchors are `ServerGamePacketListenerImpl#handleRecipeBookChangeSettingsPacket/#handleRecipeBookSeenRecipePacket`,
`ServerRecipeBook#setBookSetting/#removeHighlight`, `RecipeBookComponent#sendUpdateSettings/#recipeShown`,
and `LocalPlayer#removeRecipeHighlight`.

## Failure, ordering, and Ferrite boundary

Malformed/truncated or overlong VarInts, invalid recipe-book ordinals and residual bytes fault the
play packet. Any nonzero boolean byte is true. Signed container/display IDs otherwise reach the
semantic current-menu or no-mapping gates; stale recipe knowledge, spectator mode, an invalid menu,
insufficient return capacity and insufficient ingredients follow their documented no-op or ghost
branches rather than becoming decode errors.

The settings and highlight requests are tokenless local-first UI notifications. Placement resets
idle, performs no client inventory prediction and receives only the conditional full ghost display;
authoritative inventory convergence remains ordinary container traffic. Ferrite maps these inputs
to normalized parent recipe knowledge, recipe-menu operations and per-player settings. Raw display/
container IDs, book-type ordinals, GUI highlights, placement caches and helper results remain
version-local and never enter ECS or persistence identities.
