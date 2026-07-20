# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-MOVE-VALIDATE-001` — Server movement-packet admission and correction are a distinct authority transaction

**Parent:** `PLY-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — client movement-message selection, server admission/probe/correction, teleport
resend/acknowledgement, final state adoption and logical client convergence are complete below.
`EXP-PLY-005` remains a regression probe, not an implementation blocker.

**Applies when:**

A controlled-camera local player selects its ordinary movement message; the server handles
`ServerboundMovePlayerPacket`; the server issues/resends a `ClientboundPlayerPositionPacket`; or
either side handles its acknowledgement. Vehicle movement has a separate packet/state machine. The
independent `ServerboundPlayerInputPacket` communicates current button intent and is not substituted
by coordinates here.

**Authoritative state:**

Client last-sent XYZ/yaw/pitch, 20-tick position reminder, last-sent on-ground/horizontal-collision
flags and current predicted transform. Server `firstGood*` (start-of-server-tick position),
`lastGood*` (last accepted packet position), received/known packet counters, stored player velocity
and collision box, floating counters, game rules, and pending teleport
`(awaitingTeleport, awaitingPositionFromClient, awaitingTeleportTime)`. Position packets may omit
position or rotation; getters substitute the current server component before validation.

**Transition and ordering:**

Select at most one ordinary client movement message from exact position/rotation/status deltas; on
the server validate values, pending state, mode, displacement and collision in the numbered order
below; either correct and await an acknowledgement or adopt the target/status and update
fall/chunk/known-movement state; apply corrections directly on the client before sending ACK, forced
PosRot and the prediction barrier in that order.

**Client message selection:**

Each `LocalPlayer#sendPosition` first sends a changed sprint command, then does nothing unless the
player is the controlled camera. Compute double position deltas from last-sent coordinates and
widened float rotation deltas from last-sent angles, then increment `positionReminder`. Position is
changed when `dx*dx+dy*dy+dz*dz > (2.0e-4)^2` or reminder is `>=20`; rotation is changed when either
widened delta is exactly nonzero. Send exactly one of: `PosRot` when both changed, `Pos` for
position only, `Rot` for rotation only, or `StatusOnly` only when neither changed but on-ground or
horizontal-collision differs. Every form carries both status bits. No message is sent if none
changed. A position message updates all last-sent coordinates and resets reminder to zero; a
rotation message updates both last angles; status baselines update after every selection pass.

**Server admission and ordering:**

Move handling first returns to the level thread. Resolve absent fields against zero solely for
validation; disconnect with `multiplayer.disconnect.invalid_player_movement` when any coordinate is
NaN or either rotation is non-finite. Coordinate infinities are **not** rejected here. Resolve
absent rotations against current angles and wrap both degrees. Then, in order:

1. Return for `wonGame`; on connection tick zero reset both first/last-good coordinates to current
   position; return until client-loaded acknowledgement.
2. If a teleport is pending, optionally resend it as described below, snap only server yaw/pitch to
   this packet's wrapped rotation, and return. Coordinates and status are ignored.
3. Resolve omitted coordinates against current position, then clamp X/Z into
   `[-30,000,000,+30,000,000]` and Y into `[-20,000,000,+20,000,000]`. Thus infinities become finite
   limits.
4. A passenger is snapped to its unchanged XYZ plus packet rotation, updates chunk tracking and
   returns; packet coordinates/status are ignored. A sleeping player computes squared target
   distance from `firstGood`; above `1` it sends a correction to current XYZ with packet rotation,
   otherwise it returns without even applying rotation.
5. Let `fromFirst = target-firstGood`, `D=fromFirst.lengthSqr`, and `V=currentVelocity.lengthSqr`.
   When the level tick rate runs normally, increment received packet count and let
   `N=received-known`; if `N>5`, log and replace `N` with `1`. Movement-speed checking is disabled
   for the singleplayer owner, dimension change, false `minecraft:player_movement_check`, or (while
   fall flying) false `minecraft:elytra_movement_check`. Otherwise correct to current transform when
   `D-V > N*(300 when fall flying else 100)`. The two game rules default true. A frozen level skips
   both this counter increment and moved-too-quickly branch.
6. Save the pre-probe AABB and current XYZ. Recompute requested displacement from `lastGood`. If the
   server currently says grounded, the packet says airborne and requested Y is positive, call
   `jumpFromGround` before probing. Save the old `verticalCollisionBelow`, then call
   `player.move(PLAYER,requested)` using `PLY-COLLISION-001`.
7. Compute target minus probe result. Vanilla unconditionally replaces every finite Y residual with
   zero due its `residualY>-0.5 || residualY<0.5` condition; moved-wrongly therefore uses horizontal
   residual squared. Mark wrong only when this squared residual is `>0.0625` and the player is not
   changing dimension, sleeping, creative, spectator or in post-impulse grace time.
8. Unless no-physics or sleeping, reject when `(wrong && preProbeAabb was collision-free)` or when
   the target introduces a new pre-move collision. The latter moves the final AABB to target,
   deflates it by `9.999999747378752e-6`, obtains pre-move collision shapes relative to the old box
   bottom center, and rejects if any such shape has no intersection with the old box deflated by the
   same epsilon. On rejection, teleport to pre-probe XYZ with packet rotation, perform a
   zero-displacement fall check using packet on-ground, remove the latest probe movement recording,
   and return.
9. On acceptance, snap exactly to target XYZ and packet rotation; the collision probe does not leave
   the player at its clipped result. Compute accepted movement `accepted=currentXYZ-preProbeXYZ`.
   Set final on-ground and horizontal-collision/support movement from the **packet's two booleans**
   plus `accepted`, run fall damage and known-movement bookkeeping, movement statistics, and update
   `lastGood` to final XYZ. Positive requested Y resets fall distance. On-ground, liquid landing,
   climbable, spectator, fall-flying or auto-spin conditions reset current impulse grace as
   specified by the player helper. Finally update chunk tracking.

`knownMovement` becomes `accepted`; accepted length squared `>9.999999747378752e-6` resets idle
time. On `ServerboundClientTickEndPacket`, if no movement has arrived since the preceding
client-tick-end reset, known movement becomes zero, then the received-this-client-tick flag is
cleared. Anti-floating marks the player only when pre-probe requested Y is `>=-0.03125`, old
`verticalCollisionBelow` was false, the player is not
spectator/allowed-to-fly/mayfly/levitating/fall-flying/auto-spinning, and every block state in
`AABB.inflate(0.0625).expandTowards(0,-0.55,0)` is air. Each server connection tick with that mark,
while not sleeping/passenger/dead, increments `aboveGroundTickCount`; exceeding
`ceil(80*max(0.08/gravity,1))` disconnects for flying, while gravity `<9.999999747378752e-6` makes
the limit `Integer.MAX_VALUE`. A failed gate resets mark and counter.

**Teleport/correction state:**

Sending an absolute correction records current connection tick, increments the teleport ID and
resets it to zero instead of allowing `Integer.MAX_VALUE`, applies the server transform, stores the
resulting absolute position as awaiting, then sends the position packet. While awaiting is non-null,
every move packet returns through the pending branch; after strictly more than 20 connection ticks
the server resends by creating a new correction/ID for the same awaiting XYZ and current rotation. A
mismatched acknowledgement is ignored. A matching ID with null awaiting state disconnects invalid
movement; otherwise snap to awaiting XYZ with current rotation, copy it to `lastGood`, run
dimension-change completion and clear awaiting.

On the client, a correction is applied immediately without interpolation unless the local player is
a passenger; relative components are resolved against the current transform/velocity and distance
does not change that non-interpolated choice. The client sets position, velocity and rotation, then
resolves the same relatives against `oldPosition/yRotO/xRotO` and stores that old transform too,
preventing an unintended interpolation trail. It then sends the matching teleport acknowledgement,
immediately sends a full PosRot of its resulting transform with both status flags false, and finally
notifies block prediction of the teleport. A passenger skips local correction application but still
sends both messages using its current player transform. Reliable message order makes the
acknowledgement precede the full movement message; normally the acknowledgement clears pending
before that movement is handled.

**Branches and aborts:**

Controlled camera; four client packet forms/no packet; omitted fields; NaN/infinity;
won-game/unloaded client; pending/resend/ack mismatch; passenger; sleeping; frozen tick rate; packet
frequency; movement-check rules/singleplayer owner/dimension change/fall flying; too-quick; jump
inference; moved-wrongly exemptions; old/new collision; no-physics; accepted/rejected; floating
exemptions and disconnect.

**Constants and randomness:**

Position threshold is strict squared `>4e-8`; reminder `>=20`; coordinate clamps are horizontal
`±3e7`, vertical `±2e7`; sleeping threshold squared `>1`; packet-frequency cutoff `>5`; too-quick
factors `100`/`300`; wrong residual squared `>0.0625`; collision deflation and known-movement idle
threshold `9.999999747378752e-6`; floating Y `-0.03125`, inflate `0.0625`, downward scan `0.55`;
teleport resend age `>20`; base floating time `80`; gravity reference `0.08`. All arithmetic follows
stated Java double/float/int conversions. Movement validation consumes no RNG.

**Side effects:**

Movement/status/sprint/teleport messages; disconnect and logs; exact server/client transform,
rotation, velocity from relative correction, on-ground/support/horizontal flags, fall
distance/damage, movement statistics, idle time, known movement, chunk tracking, dimension
completion, impulse grace, probe movement records and block-prediction teleport barrier. Rejected
probes remove only their latest movement recording.

**Gates:**

Client camera ownership; client-loaded and connection tick; player
riding/sleeping/modes/abilities/effects; level running state; two movement-check game rules;
collision shapes; world bounds; pending teleport; server flight permission and gravity. Difficulty
and gameplay RNG do not affect the transaction.

**Boundary cases and quirks:**

Coordinates use NaN rejection but allow infinity-to-clamp; rotations require finiteness. More than
five packets since the tick baseline uses anti-cheat multiplier one, not five or the actual count.
Vertical moved-wrongly residual is always zero for finite values because the source condition uses
OR. The probe collision result is discarded on acceptance in favor of exact packet coordinates, and
packet on-ground/horizontal flags overwrite probe flags; the server nevertheless uses geometry for
rejection and fall checks. Sleeping movement within squared distance one is ignored. Pending
movement can rotate the server while it cannot change position. Client correction acknowledgment
precedes its forced false-status PosRot.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`. Anchors:
`net.minecraft.client.player.LocalPlayer#sendPosition()`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleMovePlayer(net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket)`,
`net.minecraft.client.multiplayer.ClientPacketListener#setValuesFromPositionPacket(net.minecraft.world.entity.PositionMoveRotation,java.util.Set,net.minecraft.world.entity.Entity,boolean)`,
`net.minecraft.network.protocol.game.ServerboundMovePlayerPacket#getX(double)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleMovePlayer(net.minecraft.network.protocol.game.ServerboundMovePlayerPacket)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#shouldCheckPlayerMovement(boolean)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#isEntityCollidingWithAnythingNew(net.minecraft.world.level.LevelReader,net.minecraft.world.entity.Entity,net.minecraft.world.phys.AABB,double,double,double)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleClientTickEnd(net.minecraft.network.protocol.game.ServerboundClientTickEndPacket)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlayerKnownMovement(net.minecraft.world.phys.Vec3)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#noBlocksAround(net.minecraft.world.entity.Entity)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#getMaximumFlyingTicks(net.minecraft.world.entity.Entity)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#teleport(net.minecraft.world.entity.PositionMoveRotation,java.util.Set)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#updateAwaitingTeleport()`, and
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleAcceptTeleportPacket(net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket)`.

**Test vectors:**

(1) Client deltas exactly around `4e-8`, reminder 19/20, exact/sub-ULP rotation and each status
change; assert one selected form and baseline mutation. (2) Each omitted-field form, NaN and
infinities, clamp edges, unloaded/won-game, passenger and sleeping distances around one. (3) Packet
counts 1/5/6, frozen/running levels, velocity subtraction and normal/elytra rule combinations around
`100N`/`300N`. (4) Horizontal probe residuals around `0.0625`, arbitrary finite Y residuals, every
exemption, old-box collision and newly introduced/overlapping shapes around the deflation epsilon.
(5) False/true on-ground and horizontal flags after an accepted clipped probe; verify final snap,
fall state, statistics, movement records and chunk update. (6) Floating Y around `-0.03125`, support
scan boundaries, every exemption and gravity below/equal/above the threshold through the computed
kick tick. (7) Teleport IDs around max reset, matching/mismatched ACK, null invariant, resend ages
20/21, rotation while pending, passenger correction and exact client ACK/PosRot/prediction order.
