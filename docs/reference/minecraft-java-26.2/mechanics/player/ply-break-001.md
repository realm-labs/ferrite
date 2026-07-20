# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-BREAK-001` — Client breaking predicts progress and rolls retained states forward on acknowledgement

**Parent:** `PLY-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed` for the state machine; `Cross-checked` for whether an
ACK-then-update intermediate state reaches a rendered frame

**SourceConclusion:**

`SourceInconclusive` — locked client/server control flow specifies every input, progress, local
mutation, packet, sequence, retention and logical correction transition below. The sole unknown is
whether the transient old-state restoration produced by a successful break's ACK is rendered before
the later authoritative air update under a given client/network scheduling trace; source fixes
packet/task order but does not promise a render between those tasks. `EXP-PLY-003` owns that exact
visible-frame question.

**Applies when:**

The local attack binding is clicked or held over a block, the target/tool changes or attack stops, a
creative delay is active, a predicted block write occurs, an authoritative block update arrives, or
a block-change acknowledgement is handled. The authoritative server transaction is `BLK-BREAK-001`;
this rule specifies only client input, prediction and convergence.

**Authoritative state:**

`MultiPlayerGameMode` owns `destroyBlockPos`, a retained reference `destroyingItem`, accumulating
`float destroyProgress`, `float destroyTicks`, signed-int `destroyDelay`, and `isDestroying`.
`BlockStatePredictionHandler` owns wrapping signed-int `currentSequenceNr`, `isPredicting`,
`lastTeleportSequence` (initially `-1`) and a position-keyed map entry
`(latest sequence, retained server state, player position at first retention)`. The server remains
authoritative for block state, permissions and the `BLK-BREAK-001` session.

**Transition and ordering:**

During keybind processing, all attack click events are consumed first. `startAttack` is rejected by
positive miss time, null hit, busy hands, disabled held item, the held item's immediate attack
restriction, or spectator/non-block dispatch. For a non-air block hit it calls `startDestroyBlock`;
it then rereads the position, records whether prediction made it air, swings main hand regardless of
the method's boolean result, and returns that air result. If any click returned true during this
keybind pass, held continuation is suppressed for this same client tick. Otherwise
`continueAttack(true)` requires no miss delay, no active item use, no piercing-weapon component, no
screen, attack held and mouse captured. A non-air block hit calls `continueDestroyBlock`; a true
result emits one breaking-block effect and swings. When none of the three early-return gates
(positive miss time, active item use, piercing weapon) applies, releasing attack, losing the block
hit, opening a screen, or losing mouse capture calls `stopDestroyBlock`. A held hit whose currently
read state is air returns without that stop call.

`startDestroyBlock(pos,face)` first rejects local `blockActionRestricted` and world-border exclusion
without a packet; when replacing a target, either rejection therefore leaves the old active session
untouched. In instabuild mode it reports tutorial progress `1`, opens a prediction scope, locally
attempts `destroyBlock(pos)`, sends sequenced `START_DESTROY_BLOCK`, closes the scope, sets
`destroyDelay=5`, and returns true even when local destruction returned false. In other modes, an
already active identical target returns true with no callback, packet or reset. If a different
target is active, send an **unsequenced** (`sequence=0`) `ABORT_DESTROY_BLOCK` for the old position
using the new hit face; local fields/crack are not cleared before the new start. Then report
tutorial progress `0` and open a prediction scope. Inside it:

1. Snapshot the currently read state for this start callback. If it is non-air and
   `destroyProgress==0.0f`, call its client-side `attack` once.
2. If non-air and its current one-tick `getDestroyProgress` is `>=1.0f`, call local `destroyBlock`
   immediately and do not open an active progress record.
3. Otherwise set `isDestroying=true`, store the position, retain the **current stack object
   reference** as `destroyingItem`, reset progress/ticks to `0.0f`, and publish local crack stage
   `-1`. This branch also runs for air.
4. Return and send a sequenced `START_DESTROY_BLOCK`, then close prediction. A survival one-tick
   instant break does not set the five-call delay.

`continueDestroyBlock(pos,face)` first sends a carried-slot packet if the selected index changed. If
`destroyDelay>0`, decrement it and return true without checking mode, border, target or state and
without progress/packets. Otherwise instabuild inside the border sets delay back to `5`, reports
tutorial progress `1`, predictively destroys the current target and sends sequenced `START`; it
never opens `isDestroying`. Non-instabuild continuation considers the same target only when
positions are equal and current main hand is `ItemStack.isSameItemSameComponents` to retained
`destroyingItem` (count is ignored). A mismatch delegates to `startDestroyBlock`, including its
old-target abort/new-target start ordering.

For a matching target, air clears `isDestroying` and returns false without resetting progress/ticks
or publishing `-1`. Otherwise add the **current** state's one-tick progress to accumulated
`destroyProgress` using Java float addition. Before incrementing `destroyTicks`, when
`destroyTicks % 4.0f == 0.0f`, request the current state's hit sound at volume
`(soundVolume+1.0f)/8.0f` and pitch `soundPitch*0.5f` with a fresh unseeded sound RNG. Increment
`destroyTicks` by `1.0f`, report tutorial progress clamped to `[0,1]`, then:

- below `1.0f`, publish local crack stage `java_f2i(destroyProgress*10.0f)` and return true;
- at/above `1.0f` (including the bytecode's NaN comparison outcome), clear `isDestroying`, open
  prediction, locally call `destroyBlock`, send sequenced `STOP_DESTROY_BLOCK`, close prediction,
  reset progress/ticks to zero, set delay `5`, publish crack `-1`, and return true.

`stopDestroyBlock` does nothing unless active. When active it reports tutorial `-1`, sends
unsequenced `ABORT` for the stored position with face `DOWN`, clears only `isDestroying` and
progress, publishes crack `-1`, and resets the player's attack-strength ticker. It does not reset
`destroyTicks`, `destroyDelay`, target or retained item.

**Predicted mutation and acknowledgement:**

A prediction scope pre-increments the wrapping sequence, marks the handler predicting, runs local
mutation before constructing/sending its sequenced action packet, then always clears the predicting
flag. Local `destroyBlock` independently repeats adventure restriction, held-item `canDestroyBlock`,
game-master permission and air gates; on success it calls current block `playerWillDestroy`, rereads
fluid after that callback, writes the fluid legacy state with flags `11`, and calls original-block
`destroy` only when the write succeeds. It performs no survival tool, stat, exhaustion, loot or XP
work. Every successful client `setBlock` while predicting stores that position's pre-write state and
current player position. A later prediction at the same position preserves the original
state/position but replaces its sequence with the newest one; callback writes to counterpart
positions are retained separately.

Authoritative single/section block updates call `setServerVerifiedBlockState(...,19)`. With a
retained prediction they update only its saved server state and leave the predicted local state
visible; without one they mutate the level immediately. The server stores the maximum received
nonnegative sequence and sends one cumulative ACK at the next connection tick. Client ACK `N`
removes every retained entry with `sequence<=N` and calls `syncBlockState` with its latest saved
state. Multi-position restoration follows locked fastutil `8.5.18` `Long2ObjectOpenHashMap` iterator
order, not insertion or coordinate order: the zero packed key first when present, then occupied
hash-table slots from highest index downward; iterator removal preserves that implementation's
wrapped-key handling. A differing local state is written with flags `19`; if the entry's captured
position is supplied, the player is in that level, and now collides with the restored state, the
client snaps exactly to the captured position. The captured position is supplied only when
`lastTeleportSequence<N`; `onTeleport` records the then-current sequence so older acknowledgements
cannot undo the teleport. An unchanged local state causes neither write nor snap.

**Branches and aborts:**

Input/miss/use/piercing/screen/mouse gates; local adventure/world border; instabuild delay;
same/different position; item/component equality; air; instant and accumulated completion; predicted
eligibility/write failure; authoritative update before/after ACK; teleport and collision correction.
Server rejection mechanics and direct correction packets are exactly `BLK-BREAK-001`, not duplicated
here.

**Constants and randomness:**

Miss delay is `10` client ticks in every non-creative local game type; post-break/creative delay is
five successful continuation calls. Hit sounds occur at float counters `0,4,8,...`; crack factor is
`10`; tutorial clamp is `[0,1]`; local destroy flags are `11`, authoritative/sync flags are `19`,
and update depth is `512`. Progress/stage use Java float and `f2i` semantics with no clamp on the
internal value. Sequence increment wraps signed 32-bit with no guard; the first prediction uses `1`,
while unsequenced aborts use `0`. A wrapped negative sequence reaches a server API that rejects
negative values, so vanilla has no safe session beyond that overflow. Breaking progression consumes
no level gameplay RNG. The hit-sound request creates an unseeded sound RNG; sound selection and
breaking particles own their presentation RNG under `CLI-SOUND-001`.

**Side effects:**

Attack swing/strength reset; tutorial callbacks; carried-slot and player-action packets; client
block callbacks/writes, fluid substitution, block-entity/update effects; local crack map; hit sounds
and per-held-tick breaking effects; prediction retention; server-state staging; ACK restoration and
collision snap. Client prediction never awards the server-only durability, stats, exhaustion, loot
or XP from `BLK-BREAK-001`.

**Gates:**

Client input focus and held-state gates, current hit/state, local game mode/abilities/adventure
predicate, world border, item enabled/immediate-attack/piercing components, stack
identity+components, local destroy eligibility, sequence/update receipt, teleport sequence and
restored-state collision. Reach and spawn protection are server authoritative; the local start
method itself has no reach or spawn-protection test beyond the target supplied by picking.
Difficulty and drop gamerules do not change client progress/prediction.

**Boundary cases and quirks:**

Client progress accumulates historical per-tick values, whereas the server recomputes current speed
over its entire elapsed interval; mid-break speed changes can therefore make completion disagree.
State replacement at the same position does not reset client progress, but item/component
replacement does; mutating the retained stack object in place also mutates the comparison reference.
Delay returns true before target validation, so held attack can emit effects/swing on a new non-air
target for five calls without a destroy packet. New-target abort uses the new face; explicit stop
uses `DOWN`. Air matching clears active without crack cleanup, and explicit stop leaves
`destroyTicks` intact. On a zero-progress target, float `destroyTicks` eventually reaches
`16,777,216`; adding `1.0f` no longer changes it, its modulo-four test remains zero, and a held
session requests a hit sound every continuation thereafter. Prediction may retain multiple positions
changed by one block callback. A rejection update received before ACK changes the saved state
without exposing it until ACK. A destroy action dropped by the server's not-loaded gate receives no
ACK, so its retained state persists until a later cumulative ACK reaches that sequence or the level
is replaced. For a successful break, the next server connection tick sends ACK before the later
per-player chunk-change sending phase, so logical client processing restores the retained state and
then applies the authoritative update; whether a rendered frame observes that intermediate
restoration is the isolated `SourceInconclusive` item.

**Evidence:**

`Confirmed` except the stated rendered-frame question; `OFF-CLIENT-001`, `OFF-SERVER-001`, and
locked library `it.unimi.dsi:fastutil:8.5.18`. Anchors:
`net.minecraft.client.Minecraft#handleKeybinds()`, `net.minecraft.client.Minecraft#startAttack()`,
`net.minecraft.client.Minecraft#continueAttack(boolean)`,
`net.minecraft.client.multiplayer.MultiPlayerGameMode#startDestroyBlock(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`,
`net.minecraft.client.multiplayer.MultiPlayerGameMode#continueDestroyBlock(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`,
`net.minecraft.client.multiplayer.MultiPlayerGameMode#stopDestroyBlock()`,
`net.minecraft.client.multiplayer.MultiPlayerGameMode#destroyBlock(net.minecraft.core.BlockPos)`,
`net.minecraft.client.multiplayer.MultiPlayerGameMode#startPrediction(net.minecraft.client.multiplayer.ClientLevel,net.minecraft.client.multiplayer.prediction.PredictiveAction)`,
`net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#retainKnownServerState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.client.player.LocalPlayer)`,
`net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler#endPredictionsUpTo(int,net.minecraft.client.multiplayer.ClientLevel)`,
`net.minecraft.client.multiplayer.ClientLevel#setBlock(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int,int)`,
`net.minecraft.client.multiplayer.ClientLevel#setServerVerifiedBlockState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,int)`,
`net.minecraft.client.multiplayer.ClientLevel#syncBlockState(net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.phys.Vec3)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#ackBlockChangesUpTo(int)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#tick()`, and
`it.unimi.dsi.fastutil.longs.Long2ObjectOpenHashMap$MapIterator#nextEntry()`.

**Test vectors:**

(1) Click/hold/release across screen, miss, use, piercing, air and mouse-focus gates; assert
same-tick continuation suppression, swings/effects and explicit stop. (2) Trace survival start,
same-target repeated start, new-target abort/start, air, one-tick instant and accumulated
completion; compare packet faces/sequences and every field. (3) Change only count, replace equal
components, replace different components, mutate the same stack object and replace target state
while preserving position. (4) Change tool/effects after several ticks and compare client
accumulation with server whole-interval recomputation. (5) Assert hit sound calls at counters 0/4/8
and exact volume/pitch; exercise progress `0`, threshold, infinity and NaN. (6) Force local
adventure/item/game-master/write denial while the server accepts/rejects independently. (7) Predict
one callback that writes two positions; deliver correction before ACK, ACK before update, teleport
before ACK and collision-producing restoration. (8) In `EXP-PLY-003`, delay/reorder within
protocol-valid server send order and capture render frames between ACK handling and the later block
update; record whether the transient retained state is ever rendered. (9) Set sequence near signed
overflow in an instrumented client and assert wrap/no guard without using that invalid state as an
implementation requirement for ordinary sessions.
