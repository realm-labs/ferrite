# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-DELAY-001` — Repeaters, observers, and torches schedule component-owned transitions

**Parent:** `RED-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — repeater, observer and both torch forms have closed source state machines;
comparator behavior remains separately closed by `RED-COMPARATOR-001`.

**Applies when:**

A delayed redstone component receives a relevant neighbor/input change.

**Authoritative state:**

Facing, powered/delay/locked state, pending scheduled tick, input signals and observer/torch
transition history.

**Transition and ordering:**

The diode base samples input from the block in its facing direction using ordinary/conductor signal,
then takes the maximum with that block's dust power unless already 15. If unlocked and current
`POWERED` differs from desired input and the same block is not already due this tick, it schedules
after `getDelay`: priority high normally, extremely-high when the output-side neighbor is a diode
not facing back toward this diode, and very-high for a currently powered falling edge. The due tick
resamples lock and input. A powered/no-input diode turns off. An unpowered diode always turns on; if
input is already absent it also schedules a very-high tick after the same delay, producing the
locked pulse behavior rather than cancelling the rise. Writes use flag 2. Placement with live input
schedules one tick after 1 tick.

A repeater delay property 1..4 maps to 2, 4, 6 or 8 game ticks. Its two side positions are queried
with diode-only control input; either positive direct signal locks it. Horizontal side-shape updates
replace the `LOCKED` property immediately on the server. Player use with build permission cycles
delay 1→2→3→4→1 even while locked. Repeater output is 15 only from its facing output side.

An observer reacts only when `directionToNeighbour == FACING` and it is not powered. On the server it
schedules a tick 2 ticks later only if none is already scheduled. A due unpowered tick writes powered
and schedules another tick after 2; a due powered tick writes unpowered. Each due edge then notifies
the position opposite `FACING`, first direct `neighborChanged`, then every face except the observer
side. Thus its high pulse lasts two game ticks and multiple watched changes while either scheduled or
powered do not enqueue extra pulses. Replacement of an already-powered observer without a pending
tick clears it with flag 18 and notifies; removal of powered state with a pending tick emits the
unpowered output notification.

Floor torch input is ordinary signal from below toward down; wall torch input is from the supporting
position along the opposite of its facing. A neighbor callback schedules exactly one tick after 2
when `LIT == hasNeighborSignal` and the same torch is not already due this tick. The due tick purges
the level-wide weak toggle list while `gameTime-oldest.when > 60`. Lit plus input writes unlit with
flag 3 and appends `(position,gameTime)`; reaching eight retained entries for that position emits
level event 1502 and schedules a restart tick after 160. Unlit plus no input relights only when the
retained count is below eight. The restart tick uses the same logic: if the oldest entries have not
yet aged out it can remain off without another restart schedule; ordinary neighbor activity can
schedule a later retry.

**Branches and aborts:**

Repeater locked callbacks and due ticks no-op; already-due diode/torch and already-scheduled observer
requests do not duplicate; diode desired equality no-ops; observer reacts only on its watched face;
torch with attachment power stays off and burnout count suppresses relight. Unsupported diodes drop,
remove and notify all six adjacent positions; unsupported wall torches become air through shape
update.

**Constants and randomness:**

Repeater delay is exactly property×2; observer edge delay/high duration and torch toggle delay are 2;
torch entries remain through age 60 and purge at age 61, burnout threshold is 8, restart delay 160.
No RNG selects state/output. Torch neighbor notifications use the locked orientation helper; client
particles are independent presentation RNG.

**Side effects:**

Powered/output state, new scheduled ticks, neighbor notifications, click/torch sounds and particles.

**Gates:**

Facing geometry, lock predicate, chunk schedule eligibility, freeze and experimental mode.

**Boundary cases and quirks:**

A scheduled callback resamples rather than applying the enqueue-time desire. Diode's unpowered due
tick turns on even if the input vanished, then queues its own fall, whereas a powered due tick only
falls when input is absent. Repeater lock blocks both enqueue and execution but does not remove an
already queued tick. Torch burnout history is per `Level` in a weak map and per position within one
shared chronological list; removal/replacement does not clear it.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `DiodeBlock`, `RepeaterBlock`, `ObserverBlock`,
`RedstoneTorchBlock`, `RedstoneWallTorchBlock`; `EXP-RED-002`.

**Test vectors:**

Repeater delay 1..4 and input pulse shorter/equal/longer; every priority branch; lock before enqueue,
between enqueue/due and at due; due unpowered with vanished input; observer watched/other face,
duplicate changes, placement/removal during each pulse phase; floor/wall torch support direction,
age 60/61 purge, 7/8 toggles, 160-tick retry with input present/absent and history retained/expired.
