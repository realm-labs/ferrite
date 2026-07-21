# Client-observable behavior mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `CLI-PREDICT-001` — Input is queued into tick actions; block and movement predictions converge at explicit acknowledgements

**Parent:** `CLI-001`, `CLI-002`, `CLI-003`, `CLI-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — frame/tick separation, focus/screen admission, key and mouse state, gameplay
action priority, sequence-scoped block prediction, movement packet selection and teleport correction
are fixed by locked client/server source. The possible rendered transient between a block ACK and a
later authoritative block update remains separately owned by `PLY-BLOCK-BREAK-001`.

**Applies when:**

The window receives an input callback, the main loop consumes client ticks or accumulated mouse
motion, a gameplay action predicts a block mutation, the local player publishes movement, or the
server corrects blocks or player position.

**Authoritative state:**

Window focus, overlay/screen and mouse-grab state; each `KeyMapping` down bit and click count; client
tick, right-click and miss timers; hit result, player hand/use/movement state; prediction sequence
and retained server states; last-sent position/rotation/ground/collision/input/sprint values; server
packet order and correction-relative flags.

**Transition and ordering:**

**Frame, tick, focus and event ingress:**

The main loop may execute zero or more `Minecraft#tick` calls before a rendered frame. After those
ticks and before frame extraction/rendering it updates the sound listener and consumes all mouse
deltas accumulated by callbacks. Mouse look is therefore frame-consumed, while gameplay actions
are tick-consumed. Pause is true only for an unpublished integrated server with a pausing GUI; it
suppresses game-mode, renderer/entity/block-entity, level, animation, particle and tick-end work,
but GUI/text, music, sound-with-paused-flag and keyboard-handler ticks still run. Lost focus with
`pauseOnLostFocus` opens pause only after strictly more than `500 ms`.

Keyboard and mouse callbacks first reject a different window handle and are scheduled onto the
client executor. `KeyMapping.click` increments every mapping bound to that physical key;
`consumeClick` returns true and decrements exactly one, while release clears both count and down
state. A non-release key action that reaches game input sets down and increments click count, so OS
repeat actions also enqueue clicks. Release clears the binding even when a screen is present unless
that screen consumed the release first. A screen receives press/repeat first and can consume it;
ordinary gameplay mapping is admitted only with no screen, while Escape/debug global paths have the
declared pause/game-mode-screen exceptions. On platforms with
`RESTORE_KEY_STATE_AFTER_MOUSE_GRAB`, grabbing gameplay focus resamples supported keyboard bindings
from current OS state.

Mouse buttons go to a non-overlay screen first. A consumed click records its screen, button and
wall-clock time; a later press is marked double only for the same screen and button at strict
elapsed `<250 ms`. With no screen/overlay, the first press grabs the mouse if needed, then updates
left/middle/right state, the bound mapping down bit, and one click. Movement callbacks ignore the
first sample after grab/entry, otherwise accumulate deltas only while focused. Frame consumption
delivers screen move/drag before player turn, then always zeros both accumulators. Normal look uses
`((sensitivity*0.6+0.2)^3)*8`; scoping omits the final factor `8`; smooth camera uses its time-based
filters. X/Y inversion is applied only at `player.turn`.

**Client tick and action priority:**

Each tick increments the client tick, decrements positive right-click delay, ticks game mode before
input, refreshes hit result/tutorial and GUI, and sets miss time to `10000` while a screen exists.
Only with neither overlay nor screen does it consume gameplay mappings, then decrement positive
miss time. It processes perspective, smooth camera, GUI bindings, spectator shader, hotbar `0..8`,
inventory, quick actions, offhand swap and drop in that order. If using an item, release occurs when
use is no longer down and all queued attack/use/pick clicks are discarded. Otherwise all queued
attacks run, then all uses, then all picks, then spectator action clicks. Held use repeats only when
right-click delay is zero and no item is in use; starting use sets delay `4`. Held attack continues
last and is suppressed for that tick after an attack that instantly removed its block.

Attack rejects positive miss time, null target as declared, busy hands, disabled item or a forbidden
weapon. Spectator action precedes ordinary handling. Piercing attack is a separate immediate path;
otherwise entity precedes block/miss dispatch, then the client swings. Use rejects active block
destruction and busy hands, iterates main then off hand, and within each hand tries in-range entity,
block `useItemOn`, then air/item use. Success returns immediately; block `Fail` aborts both hands;
client-swing success performs the swing and item-in-hand renderer notification as declared.

**Block prediction and acknowledgement:**

Every predicted start/use-on/destroy closure calls `startPredicting`, pre-increments a signed int
sequence, marks prediction active, performs local logic, sends the packet carrying that sequence,
then clears the active flag. During the closure, every successful client-level `setBlock` retains
the prewrite state and player position under that block position; a later prediction of the same
position replaces its retained sequence but preserves the evolving server-verified state object.

An authoritative block update for a retained position only rewrites that retained state; otherwise
it writes immediately with flags `19`. A cumulative ACK removes every retained entry whose sequence
is `<=` the ACK and syncs its latest server state with flags `19`. If the state differs and the
saved player position is still eligible, collision with the restored state snaps the player to that
position. A teleport records the current prediction sequence, so ACK restoration for a sequence at
or before that marker omits the old player position. Sequence overflow uses ordinary Java int
ordering; no wrap-specific comparator exists.

**Movement publication and correction:**

After local-player simulation, changed input bits are sent first. A passenger always sends rotation;
an authoritative local root vehicle additionally sends vehicle movement and then sprint change.
Otherwise sprint change precedes player movement. For a controlled camera, let position delta be
from last published coordinates. Position is changed when squared delta is strictly greater than
`(2e-4)^2` or the reminder reaches `20`; rotation is changed by exact float inequality. The client
sends `PosRot`, `Pos`, `Rot`, or, when only ground/horizontal-collision changed, `StatusOnly`.
Changed fields update their baselines; a position send resets the reminder to zero.

On `ClientboundPlayerPosition`, a nonpassenger resolves absolute values from current state and the
packet's relative flags, snaps without interpolation, applies velocity and old pose, then sends
`AcceptTeleportation(id)` followed immediately by an absolute `PosRot` with both ground and
horizontal-collision false. A passenger skips the local correction but sends the same ACK and
current-player position packet. Only after both sends does the block predictor record the teleport.
Player-rotation correction similarly resolves relative yaw/pitch, updates old rotation and replies
with a rotation packet whose two booleans are false.

**Branches and aborts:**

Wrong window; inactive focus; overlay/screen consumption; pause; ungrabbed mouse; click count zero;
item use/attack mutual exclusion; miss/right-click delay; spectator, border, reach, feature or item
gate; prediction write failure; unretained block update; unchanged movement fields; uncontrolled
camera; passenger correction; missing target/entity.

**Constants and randomness:**

Double click strict `<250 ms`; lost-focus pause strict `>500 ms`; right-click delay `4`; ordinary
miss delay `10` and screen sentinel `10000`; movement epsilon `2e-4`, heartbeat `20`; correction
interpolation cutoff `4096` is used by the shared helper but player correction requests no
interpolation. Input/prediction have no gameplay RNG; render/smoothing time is not server time.

**Side effects:**

Mapping state/counters, screens/focus/grab, local look/swing/use/break states, provisional blocks,
retained snapshots, action/movement/ACK packets, authoritative block writes, collision snap,
position/rotation/velocity and renderer/tutorial feedback.

**Gates:**

Client executor/window, focus, overlay/screen and pause state, mapping policy, player/game mode/item/
hit result, world border and interaction range, prediction active flag and sequence, retained entry,
controlled camera/passenger state and changed-field predicates.

**Boundary cases and quirks:**

Clicks are counted, not a boolean, and repeat can enqueue more than one tick action. Mouse look can
change between logical ticks because it is frame-consumed. Block updates received before their ACK
replace retained authority without immediately replacing the predicted display. ACK is cumulative.
TCP packet order remains authoritative; latency can create provisional frames but cannot authorize
client state. A teleport ACK is sent before prediction history is marked teleported.

**Evidence:**

`OFF-CLIENT-001`; `OFF-SERVER-001`; `Minecraft#render`, `#tick`, `#handleKeybinds`, `#startAttack`,
`#continueAttack`, `#startUseItem`; `KeyboardHandler#keyPress`; `MouseHandler#onButton`, `#onMove`,
`#handleAccumulatedMovement`, `#turnPlayer`; `KeyMapping`; `MultiPlayerGameMode#startPrediction`;
`BlockStatePredictionHandler`; `ClientLevel#setBlock`, `#setServerVerifiedBlockState`,
`#syncBlockState`; `LocalPlayer#tick`, `#sendPosition`; `ClientPacketListener#handleMovePlayer`;
`PLY-BLOCK-BREAK-001`; `EXP-CLI-001`.

**Test vectors:**

Press/repeat/release across no screen, consuming screen, overlay and focus changes; multiple mappings
on one key; mouse first move, frame/tick ratios and the `250/500 ms` boundaries; action priority and
both-hand use results; same-position sequences around int limits; update-before/after cumulative ACK
and teleport; movement one below/equal/above epsilon and tick `19/20`; every movement packet form,
passenger/vehicle and correction relative-flag combination.
